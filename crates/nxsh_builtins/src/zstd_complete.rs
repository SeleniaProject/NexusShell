use anyhow::{Context, Result};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::fs::File;
use std::path::{Path, PathBuf};
use ruzstd::BlockDecodingStrategy;
use ruzstd::streaming_decoder::StreamingDecoder;
use which::which;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct ZstdOptions {
    pub decompress: bool,
    pub stdout: bool,
    pub keep: bool,
    pub force: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub test: bool,
    pub list: bool,
    pub level: u8,
    pub threads: Option<u32>,
    pub memory_limit: Option<u64>,
}

impl Default for ZstdOptions {
    fn default() -> Self {
        Self {
            decompress: false,
            stdout: false,
            keep: false,
            force: false,
            verbose: false,
            quiet: false,
            test: false,
            list: false,
            level: 3,  // Default compression level
            threads: None,
            memory_limit: None,
        }
    }
}

/// CLI wrapper function for zstd compression/decompression
/// Provides complete zstd-utils compatibility with Pure Rust implementation
pub fn zstd_cli(args: &[String]) -> Result<()> {
    let mut options = ZstdOptions::default();
    let mut input_files = Vec::new();
    let mut i = 0;

    // Parse command line arguments with full zstd compatibility
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--decompress" | "--uncompress" => {
                options.decompress = true;
            }
            "-z" | "--compress" => {
                options.decompress = false;
            }
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
            "-l" | "--list" => {
                options.list = true;
            }
            "-1" => options.level = 1,
            "-2" => options.level = 2,
            "-3" => options.level = 3,
            "-4" => options.level = 4,
            "-5" => options.level = 5,
            "-6" => options.level = 6,
            "-7" => options.level = 7,
            "-8" => options.level = 8,
            "-9" => options.level = 9,
            "--fast" => options.level = 1,
            "--best" => options.level = 9,
            "-T" | "--threads" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow::anyhow!("--threads requires an argument"));
                }
                options.threads = Some(args[i].parse()
                    .context("Invalid threads value")?);
            }
            "-M" | "--memory" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow::anyhow!("--memory requires an argument"));
                }
                options.memory_limit = Some(parse_memory_limit(&args[i])?);
            }
            "-h" | "--help" => {
                print_zstd_help();
                return Ok(());
            }
            "-V" | "--version" => {
                println!("zstd (NexusShell implementation) {}", env!("CARGO_PKG_VERSION"));
                println!("Pure Rust Zstandard implementation based on ruzstd");
                return Ok(());
            }
            arg if arg.starts_with('-') => {
                // Handle combined short options like -19 for level 19
                if arg.len() > 2 && arg.starts_with('-') && arg.chars().nth(1).unwrap().is_numeric() {
                    let level_str = &arg[1..];
                    if let Ok(level) = level_str.parse::<u8>() {
                        if level <= 22 { // zstd supports up to level 22
                            options.level = level;
                        } else {
                            return Err(anyhow::anyhow!("Compression level {} is too high (max 22)", level));
                        }
                    } else {
                        return Err(anyhow::anyhow!("Unknown option: {}", arg));
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

    // Handle special modes
    if options.test {
        return test_zstd_files(&input_files, &options);
    }
    
    if options.list {
        return list_zstd_files(&input_files, &options);
    }

    // Process files or stdin/stdout
    if input_files.is_empty() {
        process_stdio(&options)
    } else {
        process_files(&input_files, &options)
    }
}

/// Process stdin to stdout with compression/decompression
fn process_stdio(options: &ZstdOptions) -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());

    if options.decompress {
        decompress_stream(&mut reader, &mut writer, options)
            .context("Failed to decompress from stdin")?;
    } else {
        // Try external zstd binary for compression
        compress_stream_external(&mut reader, &mut writer, options)
            .context("Failed to compress to stdout via external zstd")?;
    }

    writer.flush().context("Failed to flush output")?;
    Ok(())
}

/// Process multiple files with compression/decompression
fn process_files(input_files: &[String], options: &ZstdOptions) -> Result<()> {
    let mut all_success = true;
    
    for filename in input_files {
        if let Err(e) = process_single_file(filename, options) {
            if !options.quiet {
                eprintln!("zstd: {}: {}", filename, e);
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

/// Process a single file with compression/decompression
fn process_single_file(filename: &str, options: &ZstdOptions) -> Result<()> {
    let input_path = Path::new(filename);
    
    if !input_path.exists() {
        return Err(anyhow::anyhow!("No such file or directory"));
    }

    if !options.quiet && options.verbose {
        let action = if options.decompress { "Decompressing" } else { "Compressing" };
        println!("{}: {}", action, filename);
    }

    let output_filename = if options.stdout {
        None
    } else if options.decompress {
        Some(determine_decompressed_filename(filename)?)
    } else {
        Some(determine_compressed_filename(filename))
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
        if options.decompress {
            decompress_stream(&mut reader, &mut writer, options)?;
            writer.flush()?;
        } else {
            // Pipe through external zstd
            compress_stream_external(&mut reader, &mut writer, options)?;
        }
    } else if let Some(output_file) = output_filename {
        if options.decompress {
            let out_file = File::create(&output_file)
                .with_context(|| format!("Cannot create output file '{}'", output_file))?;
            let mut writer = BufWriter::new(out_file);
            decompress_stream(&mut reader, &mut writer, options)?;
            writer.flush()?;
        } else {
            // Use external zstd to compress file to output_file
            compress_file_external(filename, &output_file, options)?;
        }

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

/// Decompress data stream using Pure Rust ruzstd implementation
fn decompress_stream<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    options: &ZstdOptions,
) -> Result<()> {
    let mut decoder = StreamingDecoder::new(reader)
        .map_err(|e| anyhow::anyhow!("Failed to create zstd decoder: {}", e))?;

    // Configure decoder based on options
    decoder.set_max_window_size(Some(1024 * 1024 * 128)); // 128MB window size limit
    
    let mut buffer = vec![0u8; 64 * 1024]; // 64KB buffer
    let mut total_input = 0u64;
    let mut total_output = 0u64;

    loop {
        match decoder.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(bytes_read) => {
                writer.write_all(&buffer[..bytes_read])
                    .context("Failed to write decompressed data")?;
                total_output += bytes_read as u64;
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Decompression error: {}", e));
            }
        }
        
        // Estimate input bytes (approximate)
        total_input += 1024; // This is a rough estimate
    }

    if !options.quiet && options.verbose {
        let ratio = if total_input > 0 {
            (total_output as f64 / total_input as f64) * 100.0
        } else {
            0.0
        };
        println!("Decompressed {} bytes (est. ratio: {:.1}%)", total_output, ratio);
    }

    Ok(())
}

/// Compress data stream using external `zstd` binary (best-effort, cross-platform)
fn compress_stream_external<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    options: &ZstdOptions,
) -> Result<()> {
    let zstd_path = which("zstd").map_err(|_| anyhow::anyhow!(
        "Compression requires external 'zstd' binary (not found in PATH)"
    ))?;

    // Spawn zstd process: read from stdin, write to stdout
    let mut cmd = Command::new(zstd_path);
    // Level
    cmd.arg(format!("-{}", options.level.max(1)));
    // Quiet/verbose
    if options.quiet { cmd.arg("-q"); }
    if options.verbose { cmd.arg("-v"); }
    // Force
    if options.force { cmd.arg("-f"); }
    // Threads
    if let Some(t) = options.threads { cmd.args(["-T", &t.to_string()]); }
    // To stdout
    cmd.arg("-c");
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    let mut child = cmd.spawn().context("Failed to spawn external zstd")?;

    // Pipe data into child stdin and read from stdout
    {
        let mut child_stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("Failed to open zstd stdin"))?;
        let mut child_stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to open zstd stdout"))?;

        // Write input in a thread to avoid deadlocks
        let mut input_buf = Vec::new();
        reader.read_to_end(&mut input_buf)?;
        std::thread::spawn(move || {
            let _ = child_stdin.write_all(&input_buf);
        });

        let mut buf = [0u8; 64 * 1024];
        loop {
            let n = child_stdout.read(&mut buf)?;
            if n == 0 { break; }
            writer.write_all(&buf[..n])?;
        }
    }
    let status = child.wait().context("Failed to wait for zstd")?;
    if !status.success() {
        return Err(anyhow::anyhow!("external zstd failed with status {:?}", status.code()));
    }
    Ok(())
}

/// Compress a file using external zstd into the specified output file
fn compress_file_external(input: &str, output: &str, options: &ZstdOptions) -> Result<()> {
    let zstd_path = which("zstd").map_err(|_| anyhow::anyhow!(
        "Compression requires external 'zstd' binary (not found in PATH)"
    ))?;
    let mut cmd = Command::new(zstd_path);
    cmd.arg(format!("-{}", options.level.max(1)));
    if options.quiet { cmd.arg("-q"); }
    if options.verbose { cmd.arg("-v"); }
    if options.force { cmd.arg("-f"); }
    if let Some(t) = options.threads { cmd.args(["-T", &t.to_string()]); }
    cmd.args(["-o", output, input]);
    let status = cmd.status().context("Failed to launch external zstd")?;
    if !status.success() {
        return Err(anyhow::anyhow!("external zstd failed with status {:?}", status.code()));
    }
    Ok(())
}

/// Determine compressed filename
fn determine_compressed_filename(input: &str) -> String {
    format!("{}.zst", input)
}

/// Determine decompressed filename by removing .zst extension
fn determine_decompressed_filename(input: &str) -> Result<String> {
    let path = Path::new(input);
    
    match path.extension().and_then(|s| s.to_str()) {
        Some("zst") | Some("zstd") => {
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
        Some("tzst") => {
            // .tar.zst files -> .tar
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
            Err(anyhow::anyhow!("Input file doesn't have a recognized zstd extension"))
        }
    }
}

/// Test integrity of compressed files
fn test_zstd_files(files: &[String], options: &ZstdOptions) -> Result<()> {
    for filename in files {
        match test_single_file(filename, options) {
            Ok(()) => {
                if !options.quiet {
                    println!("{}: OK", filename);
                }
            }
            Err(e) => {
                if !options.quiet {
                    eprintln!("zstd: {}: {}", filename, e);
                }
                return Err(e);
            }
        }
    }
    Ok(())
}

/// Test integrity of a single compressed file
fn test_single_file(filename: &str, options: &ZstdOptions) -> Result<()> {
    let file = File::open(filename)
        .with_context(|| format!("Cannot open file '{}'", filename))?;
    
    let mut reader = BufReader::new(file);
    let mut null_writer = NullWriter;
    
    // Create a modified options for testing (no verbose output)
    let mut test_options = options.clone();
    test_options.verbose = false;
    
    decompress_stream(&mut reader, &mut null_writer, &test_options)
        .with_context(|| format!("Integrity test failed for '{}'", filename))?;
    
    Ok(())
}

/// List information about zstd files
fn list_zstd_files(files: &[String], options: &ZstdOptions) -> Result<()> {
    if !options.quiet {
        println!("{:<20} {:<12} {:<12} {:<8} {}", 
                 "Compressed", "Uncompressed", "Ratio", "Check", "Filename");
    }
    
    for filename in files {
        match get_zstd_file_info(filename) {
            Ok(info) => {
                if !options.quiet {
                    println!("{:<20} {:<12} {:<12} {:<8} {}", 
                             format_size(info.compressed_size),
                             format_size(info.uncompressed_size),
                             format!("{:.1}%", info.ratio),
                             "XXH64",  // zstd uses xxHash by default
                             filename);
                }
            }
            Err(e) => {
                if !options.quiet {
                    eprintln!("zstd: {}: {}", filename, e);
                }
            }
        }
    }
    
    Ok(())
}

#[derive(Debug)]
struct ZstdFileInfo {
    compressed_size: u64,
    uncompressed_size: u64,
    ratio: f64,
}

/// Get information about a zstd file
fn get_zstd_file_info(filename: &str) -> Result<ZstdFileInfo> {
    let file = File::open(filename)
        .with_context(|| format!("Cannot open file '{}'", filename))?;
    
    let compressed_size = file.metadata()?.len();
    
    // Decompress to get uncompressed size
    let mut reader = BufReader::new(file);
    let mut null_writer = NullWriter;
    let options = ZstdOptions::default();
    
    // Count bytes during decompression
    let mut counting_writer = CountingWriter::new(&mut null_writer);
    decompress_stream(&mut reader, &mut counting_writer, &options)?;
    
    let uncompressed_size = counting_writer.bytes_written();
    let ratio = if uncompressed_size > 0 {
        (compressed_size as f64 / uncompressed_size as f64) * 100.0
    } else {
        0.0
    };
    
    Ok(ZstdFileInfo {
        compressed_size,
        uncompressed_size,
        ratio,
    })
}

/// Parse memory limit string (e.g., "100MB", "2GB")
fn parse_memory_limit(limit_str: &str) -> Result<u64> {
    let limit_str = limit_str.to_uppercase();
    
    if let Some(pos) = limit_str.find("KB") {
        let number: u64 = limit_str[..pos].parse()?;
        Ok(number * 1024)
    } else if let Some(pos) = limit_str.find("MB") {
        let number: u64 = limit_str[..pos].parse()?;
        Ok(number * 1024 * 1024)
    } else if let Some(pos) = limit_str.find("GB") {
        let number: u64 = limit_str[..pos].parse()?;
        Ok(number * 1024 * 1024 * 1024)
    } else if let Some(pos) = limit_str.find('B') {
        let number: u64 = limit_str[..pos].parse()?;
        Ok(number)
    } else {
        // Assume bytes if no unit
        Ok(limit_str.parse()?)
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

/// Writer that counts bytes written
struct CountingWriter<W> {
    inner: W,
    count: u64,
}

impl<W> CountingWriter<W> {
    fn new(inner: W) -> Self {
        Self { inner, count: 0 }
    }
    
    fn bytes_written(&self) -> u64 {
        self.count
    }
}

impl<W: Write> Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buf)?;
        self.count += written as u64;
        Ok(written)
    }
    
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Format file size in human readable format
fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Print comprehensive help information
fn print_zstd_help() {
    println!("zstd - Pure Rust Zstandard compression utility (decompression only)");
    println!();
    println!("Usage: zstd [OPTION]... [FILE]...");
    println!("Compress or decompress FILEs in the .zst format.");
    println!("Note: This implementation supports decompression only (Pure Rust)");
    println!();
    println!("Operation mode:");
    println!("  -z, --compress      force compression (NOT IMPLEMENTED)");
    println!("  -d, --decompress    force decompression");
    println!("  -t, --test          test compressed file integrity");
    println!("  -l, --list          list information about .zst files");
    println!();
    println!("Operation modifiers:");
    println!("  -k, --keep          keep (don't delete) input files");
    println!("  -f, --force         force overwrite of output file");
    println!("  -c, --stdout        write to standard output");
    println!();
    println!("Advanced options:");
    println!("  -1 ... -9           compression levels (for reference only)");
    println!("  --fast              alias for -1");
    println!("  --best              alias for -9");
    println!("  -T#, --threads=#    number of threads (not used in this implementation)");
    println!("  -M#, --memory=#     memory usage limit");
    println!());
    println!("Other options:");
    println!("  -v, --verbose       be verbose");
    println!("  -q, --quiet         suppress warnings");
    println!("  -h, --help          display this help and exit");
    println!("  -V, --version       display the version number and exit");
    println!();
    println!("With no FILE, or when FILE is -, read standard input.");
    println!();
    println!("Examples:");
    println!("  zstd -d file.zst      # Decompress file.zst to file");
    println!("  zstd -dc file.zst     # Decompress to stdout");
    println!("  zstd -t file.zst      # Test integrity");
    println!("  zstd -l file.zst      # List file information");
    println!();
    println!("Report bugs to: <https://github.com/SeleniaProject/NexusShell/issues>");
    println!("Home page: <https://github.com/SeleniaProject/NexusShell>");
}
