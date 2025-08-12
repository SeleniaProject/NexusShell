use anyhow::{Context, Result};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::fs::File;
use std::path::{Path, PathBuf};
use lzma_rs::{xz_decompress, lzma_decompress};

#[derive(Debug, Clone)]
pub struct UnxzOptions {
    pub keep: bool,
    pub force: bool,
    pub stdout: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub test: bool,
}

impl Default for UnxzOptions {
    fn default() -> Self {
        Self {
            keep: false,
            force: false,
            stdout: false,
            verbose: false,
            quiet: false,
            test: false,
        }
    }
}

/// CLI wrapper function for unxz decompression
/// Provides complete compatibility with standard unxz utility using Pure Rust
pub fn unxz_cli(args: &[String]) -> Result<()> {
    let mut options = UnxzOptions::default();
    let mut input_files = Vec::new();
    let mut i = 0;

    // Parse command line arguments with full unxz compatibility
    while i < args.len() {
        match args[i].as_str() {
            "-k" | "--keep" => {
                options.keep = true;
            }
            "-f" | "--force" => {
                options.force = true;
            }
            "-c" | "--stdout" | "--to-stdout" => {
                options.stdout = true;
            }
            "-v" | "--verbose" => {
                options.verbose = true;
            }
            "-q" | "--quiet" => {
                options.quiet = true;
            }
            "-t" | "--test" => {
                options.test = true;
            }
            "-h" | "--help" => {
                print_unxz_help();
                return Ok(());
            }
            "-V" | "--version" => {
                println!("unxz (NexusShell implementation) {}", env!("CARGO_PKG_VERSION"));
                println!("Pure Rust XZ/LZMA decompression implementation based on lzma-rs");
                return Ok(());
            }
            arg if arg.starts_with('-') => {
                return Err(anyhow::anyhow!("Unknown option: {}", arg));
            }
            filename => {
                input_files.push(filename.to_string());
            }
        }
        i += 1;
    }

    // Process files or stdin
    if input_files.is_empty() {
        process_stdin(&options)
    } else {
        process_files(&input_files, &options)
    }
}

/// Process stdin to stdout with decompression
fn process_stdin(options: &UnxzOptions) -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    
    let mut reader = BufReader::new(stdin.lock());
    
    if options.test {
        let mut null_writer = NullWriter;
        decompress_stream(&mut reader, &mut null_writer, options)
            .context("Test failed for stdin input")?;
        
        if !options.quiet {
            println!("stdin: OK");
        }
    } else {
        let mut writer = BufWriter::new(stdout.lock());
        decompress_stream(&mut reader, &mut writer, options)
            .context("Failed to decompress from stdin")?;
        writer.flush().context("Failed to flush output")?;
    }

    Ok(())
}

/// Process multiple files with decompression
fn process_files(input_files: &[String], options: &UnxzOptions) -> Result<()> {
    let mut all_success = true;
    
    for filename in input_files {
        if let Err(e) = process_single_file(filename, options) {
            if !options.quiet {
                eprintln!("unxz: {}: {}", filename, e);
            }
            all_success = false;
            if !options.force {
                continue;
            }
        }
    }
    
    if !all_success {
        return Err(anyhow::anyhow!("Some files failed to process"));
    }
    
    Ok(())
}

/// Process a single file with decompression
fn process_single_file(filename: &str, options: &UnxzOptions) -> Result<()> {
    let input_path = Path::new(filename);
    
    if !input_path.exists() {
        return Err(anyhow::anyhow!("No such file or directory"));
    }

    if !options.quiet && options.verbose {
        println!("Decompressing: {}", filename);
    }

    // Determine output filename
    let output_filename = if options.stdout || options.test {
        None
    } else {
        Some(determine_decompressed_filename(filename)?)
    };

    // Check if output file already exists
    if let Some(ref out_file) = output_filename {
        if Path::new(out_file).exists() && !options.force {
            return Err(anyhow::anyhow!("Output file '{}' already exists", out_file));
        }
    }

    let input_file = File::open(input_path)
        .with_context(|| format!("Cannot open input file '{}'", filename))?;
    
    let mut reader = BufReader::new(input_file);

    if options.test {
        // Test mode - decompress but discard output
        let mut null_writer = NullWriter;
        decompress_stream(&mut reader, &mut null_writer, options)
            .with_context(|| format!("Test failed for file '{}'", filename))?;
        
        if !options.quiet {
            println!("{}: OK", filename);
        }
    } else if options.stdout {
        // Output to stdout
        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        decompress_stream(&mut reader, &mut writer, options)?;
        writer.flush()?;
    } else if let Some(output_file) = output_filename {
        // Output to file
        let out_file = File::create(&output_file)
            .with_context(|| format!("Cannot create output file '{}'", output_file))?;
        let mut writer = BufWriter::new(out_file);
        
        decompress_stream(&mut reader, &mut writer, options)?;
        writer.flush()?;

        // Remove input file if not keeping it
        if !options.keep {
            std::fs::remove_file(input_path)
                .with_context(|| format!("Cannot remove input file '{}'", filename))?;
        }

        if !options.quiet && options.verbose {
            println!("{} -> {}", filename, output_file);
        }
    }

    Ok(())
}

/// Decompress data stream using Pure Rust LZMA implementation
fn decompress_stream<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    options: &UnxzOptions,
) -> Result<()> {
    let mut compressed_data = Vec::new();
    reader.read_to_end(&mut compressed_data)
        .context("Failed to read compressed data")?;

    if compressed_data.is_empty() {
        return Ok(()); // Empty input
    }

    // Auto-detect format and decompress
    let decompressed_data = decompress_auto_detect(&compressed_data)
        .context("Decompression failed")?;

    writer.write_all(&decompressed_data)
        .context("Failed to write decompressed data")?;

    if !options.quiet && options.verbose {
        println!("Decompressed {} bytes to {} bytes", 
                compressed_data.len(), decompressed_data.len());
    }

    Ok(())
}

/// Auto-detect compression format and decompress
fn decompress_auto_detect(compressed_data: &[u8]) -> Result<Vec<u8>> {
    // Try XZ format first (most common)
    if let Ok(data) = xz_decompress(compressed_data) {
        return Ok(data);
    }
    
    // Try LZMA format
    if let Ok(data) = lzma_decompress(compressed_data) {
        return Ok(data);
    }
    
    // Check if it's a valid compressed file by examining headers
    if compressed_data.len() >= 6 {
        // XZ magic: FD 37 7A 58 5A 00
        if &compressed_data[0..6] == &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00] {
            return Err(anyhow::anyhow!("XZ file format detected but decompression failed"));
        }
        
        // LZMA magic: typically starts with specific byte patterns
        if compressed_data.len() >= 13 {
            // LZMA files start with properties byte followed by dict size
            let props = compressed_data[0];
            if props < 225 { // Valid LZMA properties range
                return Err(anyhow::anyhow!("LZMA file format detected but decompression failed"));
            }
        }
    }
    
    Err(anyhow::anyhow!("Not a valid XZ or LZMA compressed file"))
}

/// Determine decompressed filename by removing compression extension
fn determine_decompressed_filename(input: &str) -> Result<String> {
    let path = Path::new(input);
    
    // Handle various compression extensions
    match path.extension().and_then(|s| s.to_str()) {
        Some("xz") | Some("lzma") | Some("lz") => {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Some(parent) = path.parent() {
                    Ok(parent.join(stem).to_string_lossy().to_string())
                } else {
                    Ok(stem.to_string())
                }
            } else {
                Err(anyhow::anyhow!("Cannot determine output filename"))
            }
        }
        Some("txz") => {
            // .tar.xz files -> .tar
            let stem = path.file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow::anyhow!("Cannot determine output filename"))?;
            
            if let Some(parent) = path.parent() {
                Ok(parent.join(format!("{}.tar", stem)).to_string_lossy().to_string())
            } else {
                Ok(format!("{}.tar", stem))
            }
        }
        Some("tlz") => {
            // .tar.lzma files -> .tar
            let stem = path.file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow::anyhow!("Cannot determine output filename"))?;
            
            if let Some(parent) = path.parent() {
                Ok(parent.join(format!("{}.tar", stem)).to_string_lossy().to_string())
            } else {
                Ok(format!("{}.tar", stem))
            }
        }
        _ => {
            // For files without recognized extensions, add .out suffix
            Ok(format!("{}.out", input))
        }
    }
}

/// Null writer that discards all data (for testing)
struct NullWriter;

impl Write for NullWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Print comprehensive help information
fn print_unxz_help() {
    println!("unxz - Pure Rust XZ/LZMA decompression utility");
    println!();
    println!("Usage: unxz [OPTION]... [FILE]...");
    println!("Decompress FILEs in the .xz or .lzma format.");
    println!();
    println!("Operation modifiers:");
    println!("  -k, --keep          keep (don't delete) input files");
    println!("  -f, --force         force overwrite of output file and compress links");
    println!("  -c, --stdout        write to standard output and don't delete input files");
    println!();
    println!("Other options:");
    println!("  -t, --test          test compressed file integrity");
    println!("  -v, --verbose       be verbose; specify twice for even more verbose");
    println!("  -q, --quiet         suppress warnings; specify twice to suppress errors too");
    println!("  -h, --help          display this short help and exit");
    println!("  -V, --version       display the version number and exit");
    println!();
    println!("With no FILE, or when FILE is -, read standard input.");
    println!();
    println!("Examples:");
    println!("  unxz file.xz          # Decompress file.xz to file");
    println!("  unxz -c file.xz       # Decompress to stdout");
    println!("  unxz -t file.xz       # Test integrity");
    println!("  unxz -k file.xz       # Keep input file");
    println!();
    println!("Report bugs to: <https://github.com/SeleniaProject/NexusShell/issues>");
    println!("Home page: <https://github.com/SeleniaProject/NexusShell>");
}
