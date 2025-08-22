use anyhow::{Context, Result};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Bunzip2Options {
    pub stdout: bool,
    pub keep: bool,
    pub force: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub test: bool,
    pub small: bool,
}

impl Default for Bunzip2Options {
    fn default() -> Self {
        Self {
            stdout: false,
            keep: false,
            force: false,
            verbose: false,
            quiet: false,
            test: false,
            small: false,
        }
    }
}

/// CLI wrapper function for bunzip2 decompression
/// Provides complete bunzip2-utils compatibility with Pure Rust implementation
pub fn bunzip2_cli(args: &[String]) -> Result<()> {
    let mut options = Bunzip2Options::default();
    let mut input_files = Vec::new();
    let mut i = 0;

    // Parse command line arguments with full bunzip2 compatibility
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--stdout" | "--to-stdout" => {
                options.stdout = true;
            }
            "-k" | "--keep" => {
                options.keep = true;
            }
            "-f" | "--force" => {
                options.force = true;
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
            "-s" | "--small" => {
                options.small = true;
            }
            "-h" | "--help" => {
                print_bunzip2_help();
                return Ok(());
            }
            "-V" | "--version" | "-L" | "--license" => {
                print_bunzip2_version();
                return Ok(());
            }
            arg if arg.starts_with('-') => {
                // Handle combined short options
                if arg.len() > 2 && arg.starts_with('-') && !arg.starts_with("--") {
                    for ch in arg[1..].chars() {
                        match ch {
                            'c' => options.stdout = true,
                            'k' => options.keep = true,
                            'f' => options.force = true,
                            'v' => options.verbose = true,
                            'q' => options.quiet = true,
                            't' => options.test = true,
                            's' => options.small = true,
                            _ => return Err(anyhow::anyhow!("Unknown option: -{}", ch)),
                        }
                    }
                } else {
                    return Err(anyhow::anyhow!("Unknown option: {}", arg));
                }
            }
            filename => {
                input_files.push(filename.to_string());
            }
        }
        i += 1;
    }

    // Handle test mode
    if options.test {
        return test_bzip2_files(&input_files, &options);
    }

    // Process files or stdin/stdout
    if input_files.is_empty() {
        process_stdio(&options)
    } else {
        process_files(&input_files, &options)
    }
}

/// Process stdin to stdout with decompression
fn process_stdio(options: &Bunzip2Options) -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());

    decompress_stream(&mut reader, &mut writer, options)
        .context("Failed to decompress from stdin")?;

    writer.flush().context("Failed to flush output")?;
    Ok(())
}

/// Process multiple files with decompression
fn process_files(input_files: &[String], options: &Bunzip2Options) -> Result<()> {
    let mut all_success = true;
    
    for filename in input_files {
        if let Err(e) = process_single_file(filename, options) {
            if !options.quiet {
                eprintln!("bunzip2: {}: {}", filename, e);
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
fn process_single_file(filename: &str, options: &Bunzip2Options) -> Result<()> {
    let input_path = Path::new(filename);
    
    if !input_path.exists() {
        return Err(anyhow::anyhow!("No such file or directory"));
    }

    // Validate that it's a bzip2 file
    if !is_bzip2_file(input_path)? {
        return Err(anyhow::anyhow!("File is not in bzip2 format"));
    }

    let output_filename = if options.stdout {
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

    if options.stdout {
        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        
        decompress_stream(&mut reader, &mut writer, options)?;
        writer.flush()?;
    } else if let Some(output_file) = output_filename {
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
            println!("{}: done", filename);
        }
    }

    Ok(())
}

/// Decompress data stream using Pure Rust bzip2-rs implementation
fn decompress_stream<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    _options: &Bunzip2Options,
) -> Result<()> {
    // Read all input data
    let mut input_data = Vec::new();
    reader.read_to_end(&mut input_data)?;

    // Validate bzip2 header
    if input_data.len() < 3 || &input_data[0..3] != b"BZh" {
        return Err(anyhow::anyhow!("Not a bzip2 file"));
    }

    // Use bzip2-rs for Pure Rust decompression
    let decompressed_data = bzip2_rs::decode(&input_data)
        .map_err(|e| anyhow::anyhow!("Decompression error: {}", e))?;

    writer.write_all(&decompressed_data)?;
    Ok(())
}

/// Check if file is in bzip2 format
fn is_bzip2_file(path: &Path) -> Result<bool> {
    let mut file = File::open(path)?;
    let mut header = [0u8; 3];
    
    match file.read_exact(&mut header) {
        Ok(()) => Ok(&header == b"BZh"),
        Err(_) => Ok(false), // File too small or read error
    }
}

/// Determine decompressed filename by removing .bz2 extension
fn determine_decompressed_filename(input: &str) -> Result<String> {
    let path = Path::new(input);
    
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        if file_name.ends_with(".bz2") {
            let stem = file_name.strip_suffix(".bz2").unwrap();
            if let Some(parent) = path.parent() {
                Ok(parent.join(stem).to_string_lossy().to_string())
            } else {
                Ok(stem.to_string())
            }
        } else if file_name.ends_with(".tbz2") {
            let stem = file_name.strip_suffix(".tbz2").unwrap();
            if let Some(parent) = path.parent() {
                Ok(parent.join(format!("{}.tar", stem)).to_string_lossy().to_string())
            } else {
                Ok(format!("{}.tar", stem))
            }
        } else if file_name.ends_with(".tbz") {
            let stem = file_name.strip_suffix(".tbz").unwrap();
            if let Some(parent) = path.parent() {
                Ok(parent.join(format!("{}.tar", stem)).to_string_lossy().to_string())
            } else {
                Ok(format!("{}.tar", stem))
            }
        } else {
            // If no recognized extension, add .out
            Ok(format!("{}.out", input))
        }
    } else {
        Err(anyhow::anyhow!("Cannot determine output filename"))
    }
}

/// Test integrity of compressed files
fn test_bzip2_files(files: &[String], options: &Bunzip2Options) -> Result<()> {
    for filename in files {
        match test_single_file(filename, options) {
            Ok(()) => {
                if !options.quiet {
                    println!("{}: ok", filename);
                }
            }
            Err(e) => {
                if !options.quiet {
                    eprintln!("bunzip2: {}: {}", filename, e);
                }
                return Err(e);
            }
        }
    }
    Ok(())
}

/// Test integrity of a single compressed file
fn test_single_file(filename: &str, _options: &Bunzip2Options) -> Result<()> {
    let file = File::open(filename)
        .with_context(|| format!("Cannot open file '{}'", filename))?;
    
    let mut reader = BufReader::new(file);
    let mut null_writer = NullWriter;
    let test_options = Bunzip2Options::default();
    
    decompress_stream(&mut reader, &mut null_writer, &test_options)
        .with_context(|| format!("Integrity test failed for '{}'", filename))?;
    
    Ok(())
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
fn print_bunzip2_help() {
    println!("bunzip2 - Pure Rust bzip2 decompression utility");
    println!();
    println!("Usage: bunzip2 [OPTION]... [FILE]...");
    println!("Decompress FILEs compressed with bzip2.");
    println!();
    println!("Options:");
    println!("  -c, --stdout        write to standard output");
    println!("  -f, --force         force overwrite of output files");
    println!("  -k, --keep          keep (don't delete) input files");
    println!("  -q, --quiet         suppress noncritical error messages");
    println!("  -s, --small         use less memory (at most 2500k)");
    println!("  -t, --test          test compressed file integrity");
    println!("  -v, --verbose       be verbose");
    println!("  -h, --help          display this help and exit");
    println!("  -V, --version       display version information and exit");
    println!("  -L, --license       display license information and exit");
    println!();
    println!("If no FILEs are specified, decompress from standard input to standard");
    println!("output. Input files are removed after successful decompression unless");
    println!("-k is specified.");
    println!();
    println!("Supported file extensions: .bz2, .tbz2, .tbz");
    println!();
    println!("Examples:");
    println!("  bunzip2 file.bz2      # Decompress file.bz2 to file");
    println!("  bunzip2 -c file.bz2   # Decompress to stdout");
    println!("  bunzip2 -k file.bz2   # Decompress but keep original");
    println!("  bunzip2 -t file.bz2   # Test integrity");
    println!();
    println!("Pure Rust implementation using bzip2-rs crate.");
    println!("Report bugs to: <https://github.com/SeleniaProject/NexusShell/issues>");
    println!("Home page: <https://github.com/SeleniaProject/NexusShell>");
}

/// Print version information
fn print_bunzip2_version() {
    println!("bunzip2 (NexusShell Pure Rust implementation) {}", env!("CARGO_PKG_VERSION"));
    println!("Copyright (C) 2024 NexusShell Project");
    println!();
    println!("This is free software; you can redistribute it and/or modify");
    println!("it under the terms of the MIT or Apache License 2.0.");
    println!();
    println!("Pure Rust implementation using bzip2-rs crate.");
    println!("This version eliminates C library dependencies for improved");
    println!("security, portability, and integration with NexusShell.");
}

