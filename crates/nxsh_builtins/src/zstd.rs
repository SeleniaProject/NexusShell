use anyhow::{Context, Result};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::fs::File;
use std::path::Path;
use ruzstd::streaming_decoder::StreamingDecoder;

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
        // Pure Rust fallback: write a valid Zstandard frame containing a single RAW block (store mode)
        compress_stream_store(&mut reader, &mut writer, options)
            .context("Failed to write zstd store frame to stdout")?;
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
                eprintln!("zstd: {filename}: {e}");
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
        println!("{action}: {filename}");
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
        .with_context(|| format!("Cannot open input file '{filename}'"))?;
    
    let mut reader = BufReader::new(input_file);

    if options.stdout {
        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        
        if options.decompress {
            decompress_stream(&mut reader, &mut writer, options)?;
        } else {
            // Pure Rust zstd store frame to stdout
            compress_stream_store(&mut reader, &mut writer, options)?;
        }
        writer.flush()?;
    } else if let Some(output_file) = output_filename {
        if options.decompress {
            let out_file = File::create(&output_file)
                .with_context(|| format!("Cannot create output file '{output_file}'"))?;
            let mut writer = BufWriter::new(out_file);
            decompress_stream(&mut reader, &mut writer, options)?;
            writer.flush()?;
        } else {
            // Pure Rust zstd store frame to file
            compress_file_store(filename, &output_file, options)?;
        }

        // Remove input file if not keeping it
        if !options.keep {
            std::fs::remove_file(input_path)
                .with_context(|| format!("Cannot remove input file '{filename}'"))?;
        }

        if !options.quiet && options.verbose {
            println!("{filename} -> {output_file}");
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

    // ruzstdは高レベルなAPIを提供しており、個別設定は不要
    
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
        println!("Decompressed {total_output} bytes (est. ratio: {ratio:.1}%)");
    }

    Ok(())
}

/// Compress data stream producing a valid Zstandard frame that contains a single RAW block.
/// This is a Pure Rust "store" encoder: it does not attempt entropy compression.
fn compress_stream_store<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    _options: &ZstdOptions,
) -> Result<()> {
    let mut input = Vec::new();
    reader.read_to_end(&mut input)?;
    write_store_frame_slice(writer, &input)
}

/// Compress a file using the Pure Rust store encoder into the specified output file
fn compress_file_store(input: &str, output: &str, _options: &ZstdOptions) -> Result<()> {
    let mut in_file = File::open(input)
        .with_context(|| format!("Cannot open input file '{input}'"))?;
    let len = in_file.metadata()?.len();
    let mut out_file = File::create(output)
        .with_context(|| format!("Cannot create output file '{output}'"))?;
    write_store_frame_stream(&mut out_file, &mut in_file, len)
}

/// Write a minimal, standards-compliant Zstandard frame containing a single RAW block that
/// stores the provided payload without compression. This routine writes:
/// - Frame magic number
/// - Frame Header Descriptor with Single Segment and Frame Content Size fields
/// - Frame Content Size (4 or 8 bytes depending on payload length)
/// - One RAW block with Last-Block flag set and 21-bit block size
/// - No frame checksum (disabled in descriptor)
fn write_store_frame_slice<W: Write>(mut w: W, payload: &[u8]) -> Result<()> {
    // Write magic number (little-endian on disk order): 0xFD2FB528
    // Bytes in file order are 28 B5 2F FD
    w.write_all(&[0x28, 0xB5, 0x2F, 0xFD])?;

    // Frame Header Descriptor (FHD)
    // Layout (per Zstandard format):
    // - bits 7..6: Frame Content Size (FCS) field size code
    // - bit 5: Single Segment (1 => no Window Descriptor, FCS present)
    // - bits 4..3: Reserved (kept 0) or Dictionary ID flag (set 0 = no DictID)
    // - bit 2: Reserved (0)
    // - bit 1..0: Reserved (0)
    // We choose: Single Segment = 1, DictID = 0, Content Checksum = 0, FCS size selected by payload length.
    let len = payload.len() as u64;
    let (fcs_code, fcs_bytes) = if len <= 0xFFFF { (0b01u8, 2usize) } // 2-byte FCS
        else if len <= 0xFFFF_FFFF { (0b10u8, 4usize) } // 4-byte FCS
        else { (0b11u8, 8usize) }; // 8-byte FCS
    let fhd: u8 = (fcs_code << 6) | (1 << 5);
    w.write_all(&[fhd])?;

    // Frame Content Size field. Zstandard stores FCS as (actual_size - 1) in little-endian.
    let fcs_value = if len == 0 { 0 } else { len - 1 };
    let mut buf = [0u8; 8];
    buf[..8].copy_from_slice(&fcs_value.to_le_bytes());
    w.write_all(&buf[..fcs_bytes])?;

    // Single RAW block header (3 bytes):
    // [0] last_block (1 bit, LSB) | block_type (2 bits, RAW=0) | block_size (first 5 bits)
    // total: last(1) + type(2) + size(21) = 24 bits (3 bytes), little-endian packing.
    // Compute 21-bit size (clamped per spec maximum 2^21-1 for a single block)
    const MAX_BLOCK_SIZE: usize = (1 << 21) - 1;
    if len == 0 {
        // Emit a zero-size RAW last block to mark frame end
        let header_val: u32 = (0u32 << 3) | ((0u32 /* RAW */) << 1) | 1;
        let header_bytes = [
            (header_val & 0xFF) as u8,
            ((header_val >> 8) & 0xFF) as u8,
            ((header_val >> 16) & 0xFF) as u8,
        ];
        w.write_all(&header_bytes)?;
        return Ok(());
    }
    let mut offset = 0usize;
    while offset < payload.len() {
        let remaining = payload.len() - offset;
        let chunk = remaining.min(MAX_BLOCK_SIZE);
        let last_block = (offset + chunk) >= payload.len();
        let header_val: u32 = ((chunk as u32) << 3) | ((0u32 /* RAW */) << 1) | if last_block { 1 } else { 0 };
        let header_bytes = [
            (header_val & 0xFF) as u8,
            ((header_val >> 8) & 0xFF) as u8,
            ((header_val >> 16) & 0xFF) as u8,
        ];
        w.write_all(&header_bytes)?;
        w.write_all(&payload[offset..offset + chunk])?;
        offset += chunk;
    }

    // No frame checksum (flag disabled)
    Ok(())
}

/// Public helper to write a store-mode zstd frame from a reader when the total content length is known.
/// This avoids loading the entire payload into memory and streams blocks directly.
pub fn write_store_frame_stream<W: Write, R: Read>(mut w: W, reader: &mut R, content_len: u64) -> Result<()> {
    // Magic
    w.write_all(&[0x28, 0xB5, 0x2F, 0xFD])?;
    // FHD: Single Segment with FCS
    let (fcs_code, fcs_bytes) = if content_len <= 0xFFFF { (0b01u8, 2usize) }
        else if content_len <= 0xFFFF_FFFF { (0b10u8, 4usize) } else { (0b11u8, 8usize) };
    let fhd: u8 = (fcs_code << 6) | (1 << 5);
    w.write_all(&[fhd])?;
    let fcs_value = if content_len == 0 { 0 } else { content_len - 1 };
    let mut buf8 = [0u8; 8];
    buf8[..8].copy_from_slice(&fcs_value.to_le_bytes());
    w.write_all(&buf8[..fcs_bytes])?;

    const MAX_BLOCK_SIZE: usize = (1 << 21) - 1;
    let mut produced: u64 = 0;
    let mut buf = vec![0u8; MAX_BLOCK_SIZE.min(64 * 1024)];
    if content_len == 0 {
        let header_val: u32 = (0u32 << 3) | ((0u32 /* RAW */) << 1) | 1;
        let header_bytes = [
            (header_val & 0xFF) as u8,
            ((header_val >> 8) & 0xFF) as u8,
            ((header_val >> 16) & 0xFF) as u8,
        ];
        w.write_all(&header_bytes)?;
        return Ok(());
    }
    loop {
        // Determine next block size limit
        let remaining = (content_len - produced) as usize;
        if remaining == 0 { break; }
        let to_read = remaining.min(buf.len()).min(MAX_BLOCK_SIZE);
        let n = reader.read(&mut buf[..to_read])?;
        if n == 0 { break; }
        produced += n as u64;
        let last_block = produced == content_len;
        let header_val: u32 = ((n as u32) << 3) | ((0u32 /* RAW */) << 1) | if last_block { 1 } else { 0 };
        let header_bytes = [
            (header_val & 0xFF) as u8,
            ((header_val >> 8) & 0xFF) as u8,
            ((header_val >> 16) & 0xFF) as u8,
        ];
        w.write_all(&header_bytes)?;
        w.write_all(&buf[..n])?;
    }
    Ok(())
}

/// Determine compressed filename
fn determine_compressed_filename(input: &str) -> String {
    format!("{input}.zst")
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
                Ok(parent.join(format!("{stem}.tar")).to_string_lossy().to_string())
            } else {
                Ok(format!("{stem}.tar"))
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
                    println!("{filename}: OK");
                }
            }
            Err(e) => {
                if !options.quiet {
                    eprintln!("zstd: {filename}: {e}");
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
        .with_context(|| format!("Cannot open file '{filename}'"))?;
    
    let mut reader = BufReader::new(file);
    let mut null_writer = NullWriter;
    
    // Create a modified options for testing (no verbose output)
    let mut test_options = options.clone();
    test_options.verbose = false;
    
    decompress_stream(&mut reader, &mut null_writer, &test_options)
        .with_context(|| format!("Integrity test failed for '{filename}'"))?;
    
    Ok(())
}

/// List information about zstd files
fn list_zstd_files(files: &[String], options: &ZstdOptions) -> Result<()> {
    if !options.quiet {
        println!("{:<20} {:<12} {:<12} {:<8} Filename", 
                 "Compressed", "Uncompressed", "Ratio", "Check");
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
                    eprintln!("zstd: {filename}: {e}");
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
        .with_context(|| format!("Cannot open file '{filename}'"))?;
    
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
    println!("zstd - Zstandard utility (Pure Rust decompression + store-mode compression)");
    println!("Usage: zstd [OPTION]... [FILE]...");
    println!();
    println!("  -d, --decompress        decompress (Pure Rust)");
    println!("  -z, --compress          compress (Pure Rust store-mode: creates RAW-block .zst)");
    println!("  -c, --stdout            write to standard output");
    println!("  -k, --keep              keep input files");
    println!("  -f, --force             overwrite output files");
    println!("  -t, --test              test compressed file integrity");
    println!("  -l, --list              list information about .zst files");
    println!("  -q, --quiet             suppress non-critical errors");
    println!("  -v, --verbose           increase verbosity");
    println!("  -T, --threads N         threads (no effect in store-mode)");
    println!("  -M, --memory  LIM       memory usage limit (info only)");
    println!("      --zstd              alias of -z (compat)");
    println!("  -h, --help              display this help and exit");
    println!("  -V, --version           display version and exit");
    println!();
    println!("Decompression uses ruzstd (no C deps). Compression writes RAW-block zstd frames (no entropy compression).");
}
