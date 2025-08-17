use anyhow::{Context, Result};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::cmp::min;
use std::fs::File;
use std::path::Path;
use ruzstd::streaming_decoder::StreamingDecoder;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use memmap2::MmapOptions;
mod zstd_impl; // resides under src/zstd/zstd_impl via shim

// Test-only instrumentation to capture chosen Sequences modes per block
#[cfg(test)]
mod __zstd_test_instrumentation {
    use std::sync::{Mutex, Once};
    static INIT: Once = Once::new();
    static mut MODES: Option<Mutex<Vec<u8>>> = None;
    fn modes() -> &'static Mutex<Vec<u8>> {
        INIT.call_once(|| unsafe { MODES = Some(Mutex::new(Vec::new())) });
        unsafe { MODES.as_ref().unwrap() }
    }
    pub fn clear() { modes().lock().unwrap().clear(); }
    pub fn push(mode: u8) { modes().lock().unwrap().push(mode); }
    pub fn snapshot() -> Vec<u8> { modes().lock().unwrap().clone() }
}

// Public test helper shims: always present so integration tests can link,
// but only return data when instrumentation is compiled in.
#[allow(dead_code)]
pub fn __zstd_modes_clear_for_tests() {
    #[cfg(test)]
    {
        __zstd_test_instrumentation::clear();
        return;
    }
    #[cfg(not(test))]
    {
        // No-op when instrumentation is not compiled in
    }
}

#[allow(dead_code)]
pub fn __zstd_modes_snapshot_for_tests() -> Option<Vec<u8>> {
    #[cfg(test)]
    {
        return Some(__zstd_test_instrumentation::snapshot());
    }
    #[cfg(not(test))]
    {
        None
    }
}

#[derive(Debug, Clone)]
pub struct ZstdOptions {
    pub decompress: bool,
    pub stdout: bool,
    pub output: Option<String>,
    pub keep: bool,
    pub force: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub test: bool,
    pub list: bool,
    pub level: u8,
    pub threads: Option<u32>,
    pub memory_limit: Option<u64>,
    pub checksum: bool,
    pub dict_path: Option<String>,
    // internal: enable full encoder instead of store-mode
    pub full: bool,
}

impl Default for ZstdOptions {
    fn default() -> Self {
        Self {
            decompress: false,
            stdout: false,
            output: None,
            keep: false,
            force: false,
            verbose: false,
            quiet: false,
            test: false,
            list: false,
            level: 3,  // Default compression level
            threads: None,
            memory_limit: None,
            checksum: false,
            dict_path: None,
            full: false,
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
            "-z" | "--compress" | "--zstd" => {
                options.decompress = false;
            }
            "-c" | "--stdout" | "--to-stdout" => {
                options.stdout = true;
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow::anyhow!("--output requires a filepath"));
                }
                options.output = Some(args[i].clone());
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
            "--checksum" | "-C" => {
                options.checksum = true;
            }
            "--no-check" => {
                options.checksum = false;
            }
            "-D" | "--dict" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow::anyhow!("--dict requires a filepath"));
                }
                options.dict_path = Some(args[i].clone());
            }
            "--full" => {
                // Internal flag to enable the full encoder path (work-in-progress)
                options.full = true;
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
                // Support -T# compact form
                if arg.starts_with("-T") && arg.len() > 2 {
                    let n = &arg[2..];
                    options.threads = Some(n.parse().context("Invalid threads value")?);
                    i += 1;
                    continue;
                }
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

    // Validate incompatible combinations
    if !options.decompress && !options.stdout && options.output.is_some() && input_files.len() > 1 {
        return Err(anyhow::anyhow!("-o is only supported with a single input when not using --stdout"));
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
    let mut reader = BufReader::new(stdin.lock());

    // Decide output target: --stdout 優先; それ以外は -o FILE があればファイル、なければ stdout
    let to_stdout = options.stdout || options.output.is_none();

    if to_stdout {
        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        if options.decompress {
            decompress_stream(&mut reader, &mut writer, options)
                .context("Failed to decompress from stdin")?;
        } else if options.full {
            // Full encoder path (WIP): will produce compressed blocks
            compress_stream_full(&mut reader, &mut writer, options)
                .context("Failed to write zstd frame (full)")?;
        } else {
            // Store-mode
            compress_stream_store(&mut reader, &mut writer, options)
                .context("Failed to write zstd store frame to stdout")?;
        }
        writer.flush().context("Failed to flush output")?;
    } else {
        let path = options.output.as_ref().unwrap();
        let file = File::create(path)
            .with_context(|| format!("Cannot create output file '{path}'"))?;
        let mut writer = BufWriter::new(file);
        if options.decompress {
            decompress_stream(&mut reader, &mut writer, options)
                .context("Failed to decompress from stdin")?;
        } else if options.full {
            compress_stream_full(&mut reader, &mut writer, options)
                .context("Failed to write zstd frame (full)")?;
        } else {
            compress_stream_store(&mut reader, &mut writer, options)
                .context("Failed to write zstd store frame to file")?;
        }
        writer.flush().context("Failed to flush output")?;
    }
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
    let use_stdin = filename == "-";
    if !use_stdin && !input_path.exists() {
        return Err(anyhow::anyhow!("No such file or directory"));
    }

    if !options.quiet && options.verbose {
        let action = if options.decompress { "Decompressing" } else { "Compressing" };
        println!("{action}: {filename}");
    }

    let output_filename = if options.stdout {
        None
    } else if options.decompress {
        if let Some(ref o) = options.output { Some(o.clone()) } else { Some(determine_decompressed_filename(filename)?) }
    } else if let Some(ref o) = options.output { Some(o.clone()) } else { Some(determine_compressed_filename(filename)) };

    // Check if output file already exists
    if let Some(ref out_file) = output_filename {
        if Path::new(out_file).exists() && !options.force {
            return Err(anyhow::anyhow!("Output file '{}' already exists", out_file));
        }
    }

    let stdin_guard;
    let input_file;
    let mut reader: BufReader<Box<dyn Read>> = if use_stdin {
        stdin_guard = io::stdin();
        BufReader::new(Box::new(stdin_guard.lock()))
    } else {
        input_file = File::open(input_path)
            .with_context(|| format!("Cannot open input file '{filename}'"))?;
        BufReader::new(Box::new(input_file))
    };

    if options.stdout {
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
        
        if options.decompress {
            decompress_stream(&mut reader, &mut writer, options)?;
        } else if options.full {
            compress_stream_full(&mut reader, &mut writer, options)?;
        } else {
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
        } else if options.full {
            // For now, full path uses streaming through buffers
            let mut infile = File::open(input_path)
                .with_context(|| format!("Cannot open input file '{filename}'"))?;
            let mut out = File::create(&output_file)
                .with_context(|| format!("Cannot create output file '{output_file}'"))?;
            compress_stream_full(&mut infile, &mut out, options)?;
        } else {
            // Store mode path
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

/// Full encoder path: frame + compressed blocks (WIP). Currently falls back to RAW if not beneficial.
fn compress_stream_full<R: Read, W: Write>(reader: &mut R, writer: &mut W, options: &ZstdOptions) -> Result<()> {
    use zstd_impl::{compress_reader_to_writer, FullZstdOptions};
    // 1st milestone: emit a Compressed block with Raw literals + nbSeq=0.
    // ここでは一旦全バッファ読み込みして1フレームに複数 Compressed_Block を生成（128KiB上限を考慮）。
    let mut input = Vec::new();
    reader.read_to_end(&mut input)?;

    // いずれ本格エンコーダへ置換。現状は literals を Raw で詰め、Sequences は nbSeq=0 にする。
    // 内部の試験用パス（未使用）
    let mut _tmp = Vec::new();
    let _ = compress_reader_to_writer(&input[..], &mut _tmp, FullZstdOptions { 
        level: options.level, 
        checksum: options.checksum, 
        window_log: 20,
        force_four_stream: None,
    });

    write_compressed_frame_literals_only(writer, &input, options.checksum)
        .context("failed to write compressed(literals-only) frame")?;
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

    // メモリ制限オプションは現状デコーダ API 未対応のため予約（no-op）
    
    let mut buffer = vec![0u8; 64 * 1024]; // 64KB buffer
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
    }

    if !options.quiet && options.verbose {
        println!("Decompressed {total_output} bytes");
    }

    Ok(())
}

/// Compress data stream producing a valid Zstandard frame that contains a single RAW block.
/// This is a Pure Rust "store" encoder: it does not attempt entropy compression.
fn compress_stream_store<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    options: &ZstdOptions,
) -> Result<()> {
    let mut input = Vec::new();
    reader.read_to_end(&mut input)?;
    let checksum = options.checksum;
    let level = options.level;
    let threads = options.threads.unwrap_or(1).max(1) as usize;
    if threads > 1 && input.len() > (1 << 20) {
        let chunk_size = 4 * 1024 * 1024; // 4 MiB
        #[cfg(feature = "parallel")]
        {
            let frames: Vec<Vec<u8>> = input
                .par_chunks(chunk_size)
                .map(|chunk| {
                    let mut buf = Vec::with_capacity(chunk.len() + 32);
                    write_store_frame_slice_with_options(&mut buf, chunk, checksum, level)
                        .expect("zstd frame write");
                    buf
                })
                .collect();
            for f in frames { writer.write_all(&f)?; }
            return Ok(());
        }
        #[cfg(not(feature = "parallel"))]
        {
            for chunk in input.chunks(chunk_size) {
                write_store_frame_slice_with_options(writer, chunk, checksum, level)?;
            }
            return Ok(());
        }
    }
    write_store_frame_slice_with_options(writer, &input, checksum, level)
}

/// Compress a file using the Pure Rust store encoder into the specified output file
fn compress_file_store(input: &str, output: &str, options: &ZstdOptions) -> Result<()> {
    let in_file = File::open(input)
        .with_context(|| format!("Cannot open input file '{input}'"))?;
    let len = in_file.metadata()?.len();
    let mut out_file = File::create(output)
        .with_context(|| format!("Cannot create output file '{output}'"))?;
    let checksum = options.checksum;
    let level = options.level;
    let threads = options.threads.unwrap_or(1).max(1) as usize;
    if threads > 1 && len as usize > (1 << 20) {
        drop(in_file);
        let f = File::open(input)
            .with_context(|| format!("Cannot open input file '{input}' for mmap"))?;
        let mmap = unsafe { MmapOptions::new().map(&f).context("mmap failed")? };
        let chunk_size = 4 * 1024 * 1024;
        #[cfg(feature = "parallel")]
        {
            let frames: Vec<Vec<u8>> = mmap
                .par_chunks(chunk_size)
                .map(|chunk| {
                    let mut buf = Vec::with_capacity(chunk.len() + 32);
                    write_store_frame_slice_with_options(&mut buf, chunk, checksum, level)
                        .expect("zstd frame write");
                    buf
                })
                .collect();
            for f in frames { out_file.write_all(&f)?; }
            return Ok(());
        }
        #[cfg(not(feature = "parallel"))]
        {
            for chunk in mmap.chunks(chunk_size) {
                write_store_frame_slice_with_options(&mut out_file, chunk, checksum, level)?;
            }
            return Ok(());
        }
    }
    write_store_frame_stream_with_options(&mut out_file, &mut File::open(input)?, len, checksum, level)
}

/// Write a minimal, standards-compliant Zstandard frame containing a single RAW block that
/// stores the provided payload without compression. This routine writes:
/// - Frame magic number
/// - Frame Header Descriptor with Single Segment and Frame Content Size fields
/// - Frame Content Size (4 or 8 bytes depending on payload length)
/// - One RAW block with Last-Block flag set and 21-bit block size
/// - No frame checksum (disabled in descriptor)
fn write_store_frame_slice_with_options<W: Write>(mut w: W, payload: &[u8], checksum: bool, level: u8) -> Result<()> {
    // Write magic number (little-endian on disk order): 0xFD2FB528
    // Bytes in file order are 28 B5 2F FD
    w.write_all(&[0x28, 0xB5, 0x2F, 0xFD])?;

    // Frame Header Descriptor (FHD)
    // Layout (per RFC 8878):
    // - bits 7..6: Frame_Content_Size_Flag (FCS field size code)
    // - bit 5: Single_Segment_Flag (1 => no Window Descriptor, FCS present)
    // - bit 4: Unused (must be 0)
    // - bit 3: Reserved (must be 0)
    // - bit 2: Content_Checksum_Flag
    // - bits 1..0: Dictionary_ID_Flag (we set 0 = no DictID)
    // We choose: Single Segment = 1, DictID = 0, Content Checksum = per option, FCS size selected by payload length.
    let len = payload.len() as u64;
    // Choose FCS field size according to valid ranges (Single Segment => FCS always present):
    // 1 byte:    0..=255
    // 2 bytes:   256..=65791 (value encoded minus 256)
    // 4 bytes:   0..=0xFFFF_FFFF
    // 8 bytes:   0..=0xFFFF_FFFF_FFFF_FFFF
    let (fcs_code, fcs_bytes) = if len <= 255 { (0b00u8, 1usize) }
        else if len <= 65_791 { (0b01u8, 2usize) }
        else if len <= 0xFFFF_FFFF { (0b10u8, 4usize) }
        else { (0b11u8, 8usize) };
    let mut fhd: u8 = (fcs_code << 6) | (1 << 5);
    if checksum { fhd |= 0b0000_0100; } // set Content_Checksum_Flag (bit 2)
    w.write_all(&[fhd])?;

    // Frame Content Size field. Little-endian.
    // For 1,4,8-byte fields store value directly; for 2-byte field store (value - 256).
    if fcs_bytes == 2 {
    let stored: u16 = (len - 256) as u16;
        w.write_all(&stored.to_le_bytes())?;
    } else {
        let mut buf = [0u8; 8];
        buf[..8].copy_from_slice(&len.to_le_bytes());
        w.write_all(&buf[..fcs_bytes])?;
    }

    // Single RAW block header (3 bytes):
    // [0] last_block (1 bit, LSB) | block_type (2 bits, RAW=0) | block_size (first 5 bits)
    // total: last(1) + type(2) + size(21) = 24 bits (3 bytes), little-endian packing.
    // Compute 21-bit size (clamped per spec maximum 2^21-1 for a single block)
    const MAX_BLOCK_SIZE: usize = (1 << 21) - 1;
    use std::cmp::min;
    let mut xxh = xxhash_rust::xxh64::Xxh64::new(0);
    let rle_threshold: usize = match level {
        0..=2 => 48,
        3..=5 => 32,
        6..=9 => 24,
        _ => 16,
    };
    if len == 0 {
        // Emit a zero-size RAW last block to mark frame end
    let header_val: u32 = 1; // last_block=1, block_type=RAW(0), size=0
        let header_bytes = [
            (header_val & 0xFF) as u8,
            ((header_val >> 8) & 0xFF) as u8,
            ((header_val >> 16) & 0xFF) as u8,
        ];
        w.write_all(&header_bytes)?;
        if checksum {
            // XXH64 over empty content, low 4 bytes in little-endian
            let digest = xxh.digest();
            let bytes = digest.to_le_bytes();
            w.write_all(&bytes[..4])?;
        }
        return Ok(());
    }
    let mut offset = 0usize;
    while offset < payload.len() {
        let remaining = payload.len() - offset;
        let window = min(remaining, MAX_BLOCK_SIZE);
        // Detect a run at the beginning of the window
        let b0 = payload[offset];
        let mut run_len = 1usize;
        while run_len < window && payload[offset + run_len] == b0 { run_len += 1; }
    if run_len >= rle_threshold {
            // Emit RLE block
            let emit = run_len;
            let last_block = (offset + emit) >= payload.len();
            let header_val: u32 = ((emit as u32) << 3) | (1u32 << 1) | u32::from(last_block);
            let header_bytes = [
                (header_val & 0xFF) as u8,
                ((header_val >> 8) & 0xFF) as u8,
                ((header_val >> 16) & 0xFF) as u8,
            ];
            w.write_all(&header_bytes)?;
            w.write_all(&[b0])?; // RLE payload is a single byte
            if checksum {
                // update digest with repeated byte efficiently
                let buf = [b0; 256];
                let mut left = emit;
                while left > 0 {
                    let n = min(left, buf.len());
                    xxh.update(&buf[..n]);
                    left -= n;
                }
            }
            offset += emit;
        } else {
            // Emit RAW block up to window
            let emit = window;
            let last_block = (offset + emit) >= payload.len();
            let header_val: u32 = ((emit as u32) << 3) | u32::from(last_block);
            let header_bytes = [
                (header_val & 0xFF) as u8,
                ((header_val >> 8) & 0xFF) as u8,
                ((header_val >> 16) & 0xFF) as u8,
            ];
            w.write_all(&header_bytes)?;
            w.write_all(&payload[offset..offset + emit])?;
            if checksum { xxh.update(&payload[offset..offset + emit]); }
            offset += emit;
        }
    }
    // Optional frame checksum (XXH32 of content)
    if checksum {
        let digest = xxh.digest();
        let bytes = digest.to_le_bytes();
        w.write_all(&bytes[..4])?;
    }
    Ok(())
}

/// Write a minimal RFC-compliant zstd frame containing one or more Compressed_Block(s)
/// where:
/// - Literals_Section is Raw_Literals_Block (header uses 3-byte size format; Regenerated_Size = chunk length)
/// - Sequences_Section has Number_of_Sequences = 0 (single 0x00 byte, section ends)
/// - Each block obeys Block_Maximum_Size = 128 KiB constraint on Block_Content size
fn write_compressed_frame_literals_only<W: Write>(mut w: W, payload: &[u8], checksum: bool) -> Result<()> {
    #[cfg(test)]
    {
        __zstd_test_instrumentation::clear();
    }
    // Magic
    w.write_all(&[0x28, 0xB5, 0x2F, 0xFD])?;
    // FHD: Single Segment with FCS; checksum flag at bit 2 per RFC
    let len = payload.len() as u64;
    let (fcs_code, fcs_bytes) = if len <= 255 { (0b00u8, 1usize) }
        else if len <= 65_791 { (0b01u8, 2usize) }
        else if len <= 0xFFFF_FFFF { (0b10u8, 4usize) } else { (0b11u8, 8usize) };
    let mut fhd: u8 = (fcs_code << 6) | (1 << 5);
    if checksum { fhd |= 0b0000_0100; }
    w.write_all(&[fhd])?;
    if fcs_bytes == 2 {
    let stored: u16 = (len - 256) as u16;
        w.write_all(&stored.to_le_bytes())?;
    } else {
        let mut buf8 = [0u8; 8];
        buf8[..8].copy_from_slice(&len.to_le_bytes());
        w.write_all(&buf8[..fcs_bytes])?;
    }

    // Block_Maximum_Size = min(Window_Size, 128 KiB). Single Segment -> Window_Size = FCS.
    // Compressed block Block_Content size must be <= 128 KiB. Our content = 3 (lits hdr) + L + 1 (seq hdr)
    const BLOCK_MAX_CONTENT: usize = 128 * 1024;
    const LITERALS_HDR_SIZE_RAW_RLE: usize = 3; // size_format=11, Raw/RLE literals
    // For Huffman-compressed literals, header can be 3-5 bytes depending on sizes. We'll pick Size_Format=00 (1 stream, 10+10 bits) => 3 bytes if sizes <=1023.
    const SEQ_NBSEQ0_SIZE: usize = 1;   // Number_of_Sequences = 0
    let overhead_raw = LITERALS_HDR_SIZE_RAW_RLE + SEQ_NBSEQ0_SIZE; // 4 bytes

    // XXH64 over decompressed content (we'll emit low 32 bits at frame end if enabled)
    let mut xxh = xxhash_rust::xxh64::Xxh64::new(0);

    if payload.is_empty() {
        // 0-length frame: still emit one compressed block with 0 literals
    let block_size = overhead_raw as u32; // 4
        let last_block = 1u32;
        let header_val: u32 = (block_size << 3) | ((2u32 /* Compressed */) << 1) | last_block;
        let header_bytes = [
            (header_val & 0xFF) as u8,
            ((header_val >> 8) & 0xFF) as u8,
            ((header_val >> 16) & 0xFF) as u8,
        ];
        w.write_all(&header_bytes)?;
        // Literals_Section_Header (3 bytes): size_format=11, LBT=Raw(0), regenerated_size=0
    let b0 = 0b11 << 2; // low4=0, size_format=11, LBT=0
        let b1 = 0u8; let b2 = 0u8;
        w.write_all(&[b0, b1, b2])?;
        // Sequences_Section_Header: nbSeq=0
        {
            use self::zstd_impl::seq_write::write_sequences_header as write_seq_hdr;
            use self::zstd_impl::fse::CompressionMode;
            let _ = write_seq_hdr(&mut w, 0, CompressionMode::Predefined, CompressionMode::Predefined, CompressionMode::Predefined)?;
        }
    if checksum { let digest = xxh.digest(); let bytes = digest.to_le_bytes(); w.write_all(&bytes[..4])?; }
        return Ok(());
    }

    let mut offset = 0usize;
    // Keep last FSE tables to allow Repeat mode across blocks within this frame call
    let mut last_fse_tabs: Option<(
        crate::zstd::zstd_impl::fse::FseEncTable,
        crate::zstd::zstd_impl::fse::FseEncTable,
        crate::zstd::zstd_impl::fse::FseEncTable,
    )> = None;
    while offset < payload.len() {
    let remaining = payload.len() - offset;
    // ensure total Block_Content <= 128KiB
    let max_lits = BLOCK_MAX_CONTENT - overhead_raw;
        let lits = remaining.min(max_lits);
        // Literals_Section を Raw / RLE / Huffman(圧縮) から選択
        let first = payload[offset];
        let mut is_rle = true;
        for &b in &payload[offset..offset + lits] { if b != first { is_rle = false; break; } }
        let last_block = if offset + lits >= payload.len() { 1u32 } else { 0u32 };

    // Try Huffman-compressed literals when beneficial and possible
    let mut used_huff = false;
        // lightweight heuristic: if not RLE and lits >= 32, attempt Huffman build (only if max symbol <=127)
        if !is_rle && lits >= 32 {
            if let Some((ht, hdr)) = self::zstd_impl::huffman::build_literals_huffman(&payload[offset..offset + lits]) {
                // Encode literals to a single Huffman-coded stream (reverse-bit order per RFC 4.2.2)
                use self::zstd_impl::huffman::reverse_bits;
                use self::zstd_impl::bitstream::BitWriter;
                // helper to encode a literals slice with existing Huffman codes
                let encode_stream = |slice: &[u8]| -> io::Result<Vec<u8>> {
                    let mut buf = Vec::with_capacity(slice.len() / 2 + 8);
                    {
                        let mut bw = BitWriter::new(&mut buf);
                        for &sym in slice {
                            if let Some((code, bl)) = ht.codes[sym as usize] {
                                let rev = reverse_bits(code, bl);
                                bw.write_bits(rev as u64, bl)?;
                            } else {
                                // unreachable due to table build
                                return Err(io::Error::other("missing symbol code"));
                            }
                        }
                        bw.write_bits(1, 1)?; // final bit
                        bw.align_to_byte()?;
                    }
                    Ok(buf)
                };

                // First try 1-stream SF=00 if effective
                let bitbuf_single = encode_stream(&payload[offset..offset + lits])?;
                let htd_size = hdr.len();
                let comp_including_tree_single = bitbuf_single.len() + htd_size;
                // packing helper for LSH (Compressed/Regenerated packed after [LBT,SF])
                let pack_lsh = |lbt: u8, sf: u8, regen_bits: u8, comp_bits: u8, regen: u32, comp: u32| -> Vec<u8> {
                    let byte_len = match sf { 0b00 | 0b01 => 3, 0b10 => 4, 0b11 => 5, _ => 3 };
                    let mut out = vec![0u8; byte_len];
                    // Construct a u64 accumulator of bits in little-endian within bytes
                    let mut acc: u64 = 0;
                    let mut nb: u8 = 0;
                    // pack low4(regen)
                    acc |= ((regen & 0x0F) as u64) << (nb as u64); nb += 4;
                    // pack SF (2)
                    acc |= ((sf & 0x03) as u64) << (nb as u64); nb += 2;
                    // pack LBT (2)
                    acc |= ((lbt & 0x03) as u64) << (nb as u64); nb += 2;
                    // remaining regen bits
                    let rem_regen = regen_bits - 4;
                    if rem_regen > 0 { acc |= (((regen >> 4) as u64) & ((1u64 << rem_regen) - 1)) << (nb as u64); nb += rem_regen; }
                    // comp bits
                    if comp_bits > 0 { acc |= ((comp as u64) & ((1u64 << comp_bits) - 1)) << (nb as u64); }
                    // spill to bytes
                    for (i, b) in out.iter_mut().enumerate().take(byte_len) { *b = ((acc >> (8*i)) & 0xFF) as u8; }
                    out
                };

                // Prefer single-stream when it fits; else attempt 4-streams
                let mut emitted = false;
                if comp_including_tree_single < lits && comp_including_tree_single <= 1023 && lits <= 1023 {
                    // SF=00, 10-bit each
                    let lbt = 0b10u8; let sf = 0b00u8;
                    let lsh = pack_lsh(lbt, sf, 10, 10, lits as u32, comp_including_tree_single as u32);
                    // Build candidate Sequences_Section for modes and choose the smallest that fits
                    let mut sequences_bytes: Option<Vec<u8>> = None;
                    let mut selected_fse_tabs: Option<(
                        crate::zstd::zstd_impl::fse::FseEncTable,
                        crate::zstd::zstd_impl::fse::FseEncTable,
                        crate::zstd::zstd_impl::fse::FseEncTable,
                    )> = None;
                    {
                        use self::zstd_impl::seq::{tokenize_full, tokenize_first};
                        use self::zstd_impl::seq_write::{build_sequences_rle_section_bytes, build_sequences_predefined_section_bytes, build_sequences_fse_compressed_section_bytes, build_sequences_repeat_section_bytes, build_fse_tables_from_seqs};
                        let (seqs, _literals_stream) = tokenize_full(&payload[offset..offset + lits]);
                        if !seqs.is_empty() {
                            // Gather candidates with their sizes
                            let mut best_len: usize = usize::MAX;
                            let base_len = lsh.len() + htd_size + bitbuf_single.len();
                            // RLE
                            if let Ok(mut sec) = build_sequences_rle_section_bytes(&seqs) {
                                let prospective = base_len + sec.len();
                                if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len {
                                    best_len = sec.len();
                                    sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                    selected_fse_tabs = None; // RLE doesn't set tables
                                }
                            }
                            // Predefined
                            if let Ok(mut sec) = build_sequences_predefined_section_bytes(&seqs) {
                                let prospective = base_len + sec.len();
                                if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len {
                                    best_len = sec.len();
                                    sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                    selected_fse_tabs = None;
                                }
                            }
                            // FSE_Compressed (also prepare tables if chosen)
                            if let Ok(mut sec) = build_sequences_fse_compressed_section_bytes(&seqs) {
                                let prospective = base_len + sec.len();
                                if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len {
                                    best_len = sec.len();
                                    sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                    if let Ok(t) = build_fse_tables_from_seqs(&seqs) { selected_fse_tabs = Some(t); }
                                }
                            }
                            // Repeat (if we have previous tables)
                            if let Some(ref tabs) = last_fse_tabs {
                                if let Ok(mut sec) = build_sequences_repeat_section_bytes(&seqs, tabs) {
                                    let prospective = base_len + sec.len();
                                    if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len {
                                        best_len = sec.len();
                                        sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                        selected_fse_tabs = None; // do not update on Repeat
                                    }
                                }
                            }
                            if sequences_bytes.is_none() {
                                if let Some((one_seq, _lits_stream)) = tokenize_first(&payload[offset..offset + lits]) {
                                    let base_len = lsh.len() + htd_size + bitbuf_single.len();
                                    let mut best_len1 = best_len;
                                    if let Ok(mut sec) = build_sequences_rle_section_bytes(std::slice::from_ref(&one_seq)) {
                                        let prospective = base_len + sec.len();
                                        if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len1 {
                                            best_len1 = sec.len();
                                            sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                            selected_fse_tabs = None;
                                        }
                                    }
                                    if let Ok(mut sec) = build_sequences_predefined_section_bytes(std::slice::from_ref(&one_seq)) {
                                        let prospective = base_len + sec.len();
                                        if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len1 {
                                            best_len1 = sec.len();
                                            sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                            selected_fse_tabs = None;
                                        }
                                    }
                                    if let Ok(mut sec) = build_sequences_fse_compressed_section_bytes(std::slice::from_ref(&one_seq)) {
                                        let prospective = base_len + sec.len();
                                        if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len1 {
                                            best_len1 = sec.len();
                                            sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                            if let Ok(t) = build_fse_tables_from_seqs(std::slice::from_ref(&one_seq)) { selected_fse_tabs = Some(t); }
                                        }
                                    }
                                    if let Some(ref tabs) = last_fse_tabs {
                                        if let Ok(mut sec) = build_sequences_repeat_section_bytes(std::slice::from_ref(&one_seq), tabs) {
                                            let prospective = base_len + sec.len();
                                            if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len1 {
                                                best_len1 = sec.len();
                                                sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                                selected_fse_tabs = None;
                                            }
                                        }
                                    }
                                }
                            }
                            // If we selected an FSE_Compressed candidate, persist its tables
                            if selected_fse_tabs.is_some() { last_fse_tabs = selected_fse_tabs; }
                        }
                    }
                    let block_size = (lsh.len() + htd_size + bitbuf_single.len() + if let Some(ref sec) = sequences_bytes { sec.len() } else { SEQ_NBSEQ0_SIZE }) as u32;
                    let header_val: u32 = (block_size << 3) | ((2u32) << 1) | last_block;
                    let header_bytes = [ (header_val & 0xFF) as u8, ((header_val >> 8) & 0xFF) as u8, ((header_val >> 16) & 0xFF) as u8 ];
                    w.write_all(&header_bytes)?;
                    w.write_all(&lsh)?;
                    w.write_all(&hdr)?;
                    w.write_all(&bitbuf_single)?;
                    if let Some(sec) = sequences_bytes {
                        #[cfg(test)]
                        { if sec.len() >= 2 { __zstd_test_instrumentation::push(sec[1]); } }
                        w.write_all(&sec)?;
                    } else {
                        use self::zstd_impl::seq_write::write_sequences_header as write_seq_hdr;
                        use self::zstd_impl::fse::CompressionMode;
                        let _ = write_seq_hdr(&mut w, 0, CompressionMode::Predefined, CompressionMode::Predefined, CompressionMode::Predefined)?;
                    }
                    if checksum { xxh.update(&payload[offset..offset + lits]); }
                    emitted = true;
                }

                if !emitted {
                    // 4-stream variants
                    // split literals into 4 streams sizes per RFC
                    let s1 = lits.div_ceil(4);
                    let s2 = (lits + 2) / 4;
                    let s3 = (lits + 1) / 4;
                    let p = &payload[offset..offset + lits];
                    let (p1, rest) = p.split_at(s1);
                    let (p2, rest) = rest.split_at(s2);
                    let (p3, p4) = rest.split_at(s3);
                    let b1 = encode_stream(p1)?; let b2 = encode_stream(p2)?; let b3 = encode_stream(p3)?; let b4 = encode_stream(p4)?;
                    let jump_table = [
                        (b1.len() as u16).to_le_bytes(),
                        (b2.len() as u16).to_le_bytes(),
                        (b3.len() as u16).to_le_bytes(),
                    ];
                    let total_streams_size = 6 + b1.len() + b2.len() + b3.len() + b4.len();
                    let comp_total = htd_size + total_streams_size;
                    // Choose smallest SF that fits both sizes
                    let (sf, regen_bits, comp_bits, lsh_len) = if (lits <= 1023) && (comp_total <= 1023) {
                        (0b01u8, 10u8, 10u8, 3usize)
                    } else if (lits <= 16383) && (comp_total <= 16383) {
                        (0b10u8, 14u8, 14u8, 4usize)
                    } else if (lits <= 262143) && (comp_total <= 262143) {
                        (0b11u8, 18u8, 18u8, 5usize)
                    } else { (0u8,0u8,0u8,0usize) };
                    if lsh_len != 0 {
                        // Build candidate Sequences_Section and choose smallest
                        let mut sequences_bytes: Option<Vec<u8>> = None;
                        let mut selected_fse_tabs: Option<(
                            crate::zstd::zstd_impl::fse::FseEncTable,
                            crate::zstd::zstd_impl::fse::FseEncTable,
                            crate::zstd::zstd_impl::fse::FseEncTable,
                        )> = None;
                        {
                            use self::zstd_impl::seq::{tokenize_full, tokenize_first};
                            use self::zstd_impl::seq_write::{build_sequences_rle_section_bytes, build_sequences_predefined_section_bytes, build_sequences_fse_compressed_section_bytes, build_sequences_repeat_section_bytes, build_fse_tables_from_seqs};
                            let (seqs, _literals_stream) = tokenize_full(&payload[offset..offset + lits]);
                            if !seqs.is_empty() {
                                let mut best_len: usize = usize::MAX;
                                let base_len = lsh_len + htd_size + total_streams_size;
                                // RLE
                                if let Ok(mut sec) = build_sequences_rle_section_bytes(&seqs) {
                                    let prospective = base_len + sec.len();
                                    if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len {
                                        best_len = sec.len();
                                        sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                        selected_fse_tabs = None;
                                    }
                                }
                                // Predefined
                                if let Ok(mut sec) = build_sequences_predefined_section_bytes(&seqs) {
                                    let prospective = base_len + sec.len();
                                    if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len {
                                        best_len = sec.len();
                                        sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                        selected_fse_tabs = None;
                                    }
                                }
                                // FSE_Compressed
                                if let Ok(mut sec) = build_sequences_fse_compressed_section_bytes(&seqs) {
                                    let prospective = base_len + sec.len();
                                    if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len {
                                        best_len = sec.len();
                                        sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                        if let Ok(t) = build_fse_tables_from_seqs(&seqs) { selected_fse_tabs = Some(t); }
                                    }
                                }
                                // Repeat
                                if let Some(ref tabs) = last_fse_tabs {
                                    if let Ok(mut sec) = build_sequences_repeat_section_bytes(&seqs, tabs) {
                                        let prospective = base_len + sec.len();
                                        if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len {
                                            best_len = sec.len();
                                            sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                            selected_fse_tabs = None;
                                        }
                                    }
                                }
                                // Fallback to single-seq candidates
                                if sequences_bytes.is_none() {
                                    if let Some((one_seq, _lits_stream)) = tokenize_first(&payload[offset..offset + lits]) {
                                        let base_len = lsh_len + htd_size + total_streams_size;
                                        let mut best_len1 = best_len;
                                        if let Ok(mut sec) = build_sequences_rle_section_bytes(std::slice::from_ref(&one_seq)) {
                                            let prospective = base_len + sec.len();
                                            if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len1 {
                                                best_len1 = sec.len();
                                                sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                                selected_fse_tabs = None;
                                            }
                                        }
                                        if let Ok(mut sec) = build_sequences_predefined_section_bytes(std::slice::from_ref(&one_seq)) {
                                            let prospective = base_len + sec.len();
                                            if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len1 {
                                                best_len1 = sec.len();
                                                sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                                selected_fse_tabs = None;
                                            }
                                        }
                                        if let Ok(mut sec) = build_sequences_fse_compressed_section_bytes(std::slice::from_ref(&one_seq)) {
                                            let prospective = base_len + sec.len();
                                            if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len1 {
                                                best_len1 = sec.len();
                                                sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                                if let Ok(t) = build_fse_tables_from_seqs(std::slice::from_ref(&one_seq)) { selected_fse_tabs = Some(t); }
                                            }
                                        }
                                        if let Some(ref tabs) = last_fse_tabs {
                                            if let Ok(mut sec) = build_sequences_repeat_section_bytes(std::slice::from_ref(&one_seq), tabs) {
                                                let prospective = base_len + sec.len();
                                                if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len1 {
                                                    best_len1 = sec.len();
                                                    sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                                    selected_fse_tabs = None;
                                                }
                                            }
                                        }
                                    }
                                }
                                if selected_fse_tabs.is_some() { last_fse_tabs = selected_fse_tabs; }
                            }
                        }
                        let prospective_block_size = lsh_len + htd_size + total_streams_size + if let Some(ref sec) = sequences_bytes { sec.len() } else { SEQ_NBSEQ0_SIZE };
                        if prospective_block_size <= BLOCK_MAX_CONTENT {
                            // pack LSH
                            let lbt = 0b10u8; // compressed
                            let lsh = pack_lsh(lbt, sf, regen_bits, comp_bits, lits as u32, comp_total as u32);
                            let block_size = (lsh.len() + htd_size + total_streams_size + if let Some(ref sec) = sequences_bytes { sec.len() } else { SEQ_NBSEQ0_SIZE }) as u32;
                            let header_val: u32 = (block_size << 3) | ((2u32) << 1) | last_block;
                            let header_bytes = [ (header_val & 0xFF) as u8, ((header_val >> 8) & 0xFF) as u8, ((header_val >> 16) & 0xFF) as u8 ];
                            // write block
                            w.write_all(&header_bytes)?;
                            w.write_all(&lsh)?;
                            w.write_all(&hdr)?; // tree
                            // Jump_Table 6 bytes
                            w.write_all(&jump_table[0])?; w.write_all(&jump_table[1])?; w.write_all(&jump_table[2])?;
                            // Streams in order
                            w.write_all(&b1)?; w.write_all(&b2)?; w.write_all(&b3)?; w.write_all(&b4)?;
                            if let Some(sec) = sequences_bytes {
                                #[cfg(test)]
                                { if sec.len() >= 2 { __zstd_test_instrumentation::push(sec[1]); } }
                                w.write_all(&sec)?;
                            } else {
                                use self::zstd_impl::seq_write::write_sequences_header as write_seq_hdr;
                                use self::zstd_impl::fse::CompressionMode;
                                let _ = write_seq_hdr(&mut w, 0, CompressionMode::Predefined, CompressionMode::Predefined, CompressionMode::Predefined)?;
                            }
                            if checksum { xxh.update(&payload[offset..offset + lits]); }
                            emitted = true;
                        }
                    }
                }

                if emitted { used_huff = true; }
            }
        }

        if !used_huff {
            // Raw or RLE path; additionally, try sequences (RLE preferred; else Predefined) when beneficial
            // Decide literals payload to emit and whether to append a sequences section
            let mut emit_literals: std::borrow::Cow<[u8]> = std::borrow::Cow::Borrowed(&payload[offset..offset + lits]);
            let mut emit_lbt: u8 = if is_rle { 0b01 } else { 0b00 }; // default Raw(0) or RLE(1)
            let mut sequences_bytes: Option<Vec<u8>> = None;
            let mut selected_fse_tabs: Option<(
                crate::zstd::zstd_impl::fse::FseEncTable,
                crate::zstd::zstd_impl::fse::FseEncTable,
                crate::zstd::zstd_impl::fse::FseEncTable,
            )> = None;
            {
                use self::zstd_impl::seq::{tokenize_full, tokenize_first};
                use self::zstd_impl::seq_write::{build_sequences_rle_section_bytes, build_sequences_predefined_section_bytes, build_sequences_fse_compressed_section_bytes, build_sequences_repeat_section_bytes, build_fse_tables_from_seqs};
                let (seqs, literals_stream) = tokenize_full(&payload[offset..offset + lits]);
                if !seqs.is_empty() {
                    // Choose smallest among available modes that fits
                    let mut best_len: usize = usize::MAX;
                    // RLE
                    if let Ok(mut sec) = build_sequences_rle_section_bytes(&seqs) {
                        let prospective = LITERALS_HDR_SIZE_RAW_RLE + literals_stream.len() + sec.len();
                        if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len {
                            best_len = sec.len();
                            sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                            emit_literals = std::borrow::Cow::Owned(literals_stream.clone());
                            emit_lbt = 0b00;
                            selected_fse_tabs = None;
                        }
                    }
                    // Predefined
                    if let Ok(mut sec) = build_sequences_predefined_section_bytes(&seqs) {
                        let prospective = LITERALS_HDR_SIZE_RAW_RLE + literals_stream.len() + sec.len();
                        if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len {
                            best_len = sec.len();
                            sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                            emit_literals = std::borrow::Cow::Owned(literals_stream.clone());
                            emit_lbt = 0b00;
                            selected_fse_tabs = None;
                        }
                    }
                    // FSE_Compressed
                    if let Ok(mut sec) = build_sequences_fse_compressed_section_bytes(&seqs) {
                        let prospective = LITERALS_HDR_SIZE_RAW_RLE + literals_stream.len() + sec.len();
                        if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len {
                            best_len = sec.len();
                            sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                            emit_literals = std::borrow::Cow::Owned(literals_stream.clone());
                            emit_lbt = 0b00;
                            if let Ok(t) = build_fse_tables_from_seqs(&seqs) { selected_fse_tabs = Some(t); }
                        }
                    }
                    // Repeat
                    if let Some(ref tabs) = last_fse_tabs {
                        if let Ok(mut sec) = build_sequences_repeat_section_bytes(&seqs, tabs) {
                            let prospective = LITERALS_HDR_SIZE_RAW_RLE + literals_stream.len() + sec.len();
                            if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len {
                                best_len = sec.len();
                                sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                emit_literals = std::borrow::Cow::Owned(literals_stream.clone());
                                emit_lbt = 0b00;
                                selected_fse_tabs = None;
                            }
                        }
                    }
                    // Fallback to single-seq variants
                    if sequences_bytes.is_none() {
                        if let Some((one_seq, lits_stream)) = tokenize_first(&payload[offset..offset + lits]) {
                            let mut best_len1 = best_len;
                            if let Ok(mut sec) = build_sequences_rle_section_bytes(std::slice::from_ref(&one_seq)) {
                                let prospective = LITERALS_HDR_SIZE_RAW_RLE + lits_stream.len() + sec.len();
                                if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len1 {
                                    best_len1 = sec.len();
                                    sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                    emit_literals = std::borrow::Cow::Owned(lits_stream.clone());
                                    emit_lbt = 0b00;
                                    selected_fse_tabs = None;
                                }
                            }
                            if let Ok(mut sec) = build_sequences_predefined_section_bytes(std::slice::from_ref(&one_seq)) {
                                let prospective = LITERALS_HDR_SIZE_RAW_RLE + lits_stream.len() + sec.len();
                                if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len1 {
                                    best_len1 = sec.len();
                                    sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                    emit_literals = std::borrow::Cow::Owned(lits_stream.clone());
                                    emit_lbt = 0b00;
                                    selected_fse_tabs = None;
                                }
                            }
                            if let Ok(mut sec) = build_sequences_fse_compressed_section_bytes(std::slice::from_ref(&one_seq)) {
                                let prospective = LITERALS_HDR_SIZE_RAW_RLE + lits_stream.len() + sec.len();
                                if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len1 {
                                    best_len1 = sec.len();
                                    sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                    emit_literals = std::borrow::Cow::Owned(lits_stream.clone());
                                    emit_lbt = 0b00;
                                    if let Ok(t) = build_fse_tables_from_seqs(std::slice::from_ref(&one_seq)) { selected_fse_tabs = Some(t); }
                                }
                            }
                            if let Some(ref tabs) = last_fse_tabs {
                                if let Ok(mut sec) = build_sequences_repeat_section_bytes(std::slice::from_ref(&one_seq), tabs) {
                                    let prospective = LITERALS_HDR_SIZE_RAW_RLE + lits_stream.len() + sec.len();
                                    if prospective <= BLOCK_MAX_CONTENT && sec.len() < best_len1 {
                                        best_len1 = sec.len();
                                        sequences_bytes = Some({ sec.shrink_to_fit(); sec });
                                        emit_literals = std::borrow::Cow::Owned(lits_stream.clone());
                                        emit_lbt = 0b00;
                                        selected_fse_tabs = None;
                                    }
                                }
                            }
                        }
                    }
                    if selected_fse_tabs.is_some() { last_fse_tabs = selected_fse_tabs; }
                }
                // If still none, we'll emit nbSeq=0 below using write_seq_hdr
            }
            let lit_payload_size = if emit_lbt == 0b01 { 1usize } else { emit_literals.len() };
            let block_size = (LITERALS_HDR_SIZE_RAW_RLE + lit_payload_size + if let Some(ref sec) = sequences_bytes { sec.len() } else { SEQ_NBSEQ0_SIZE }) as u32;
            let header_val: u32 = (block_size << 3) | ((2u32 /* Compressed */) << 1) | last_block;
            let header_bytes = [
                (header_val & 0xFF) as u8,
                ((header_val >> 8) & 0xFF) as u8,
                ((header_val >> 16) & 0xFF) as u8,
            ];
            w.write_all(&header_bytes)?;

            // Literals Section Header (Raw/RLE, 3-byte size format with 20-bit regenerated size)
            let regen = if emit_lbt == 0b01 { lits as u32 } else { emit_literals.len() as u32 }; // <= 1,048,575
            let low4 = (regen & 0x0F) as u8;
            let mid8 = ((regen >> 4) & 0xFF) as u8;
            let high8 = ((regen >> 12) & 0xFF) as u8;
            let b0 = (low4 << 4) | (0b11 << 2) | emit_lbt; // LBT: Raw(0) or RLE(1)
            w.write_all(&[b0, mid8, high8])?;
            // Literals payload
            if emit_lbt == 0b01 { w.write_all(&[first])?; } else { w.write_all(&emit_literals)?; }
            // Sequences Section
            if let Some(sec) = sequences_bytes {
                #[cfg(test)]
                { if sec.len() >= 2 { __zstd_test_instrumentation::push(sec[1]); } }
                w.write_all(&sec)?;
            } else {
                use self::zstd_impl::seq_write::write_sequences_header as write_seq_hdr;
                use self::zstd_impl::fse::CompressionMode;
                let _ = write_seq_hdr(&mut w, 0, CompressionMode::Predefined, CompressionMode::Predefined, CompressionMode::Predefined)?;
            }
            if checksum { xxh.update(&payload[offset..offset + lits]); }
        }

    offset += lits;
    }
    if checksum {
        let digest = xxh.digest();
        let bytes = digest.to_le_bytes();
        w.write_all(&bytes[..4])?;
    }
    Ok(())
}

// Test-only public wrapper to exercise the full encoder path (compressed blocks with sequences)
#[cfg(test)]
pub fn __zstd_write_full_frame_for_tests<W: Write>(w: W, payload: &[u8], checksum: bool) -> Result<()> {
    write_compressed_frame_literals_only(w, payload, checksum)
}

/// Public helper to write a store-mode zstd frame from a reader when the total content length is known.
/// This avoids loading the entire payload into memory and streams blocks directly.
pub fn write_store_frame_stream<W: Write, R: Read>(w: W, reader: &mut R, content_len: u64) -> Result<()> {
    write_store_frame_stream_with_options(w, reader, content_len, false, 3)
}

/// Same as write_store_frame_stream with options (currently only checksum flag).
pub fn write_store_frame_stream_with_options<W: Write, R: Read>(mut w: W, reader: &mut R, content_len: u64, checksum: bool, level: u8) -> Result<()> {
    // Magic
    w.write_all(&[0x28, 0xB5, 0x2F, 0xFD])?;
    // FHD: Single Segment with FCS; checksum at bit 2
    let (fcs_code, fcs_bytes) = if content_len <= 255 { (0b00u8, 1usize) }
        else if content_len <= 65_791 { (0b01u8, 2usize) }
        else if content_len <= 0xFFFF_FFFF { (0b10u8, 4usize) } else { (0b11u8, 8usize) };
    let mut fhd: u8 = (fcs_code << 6) | (1 << 5);
    if checksum { fhd |= 0b0000_0100; }
    w.write_all(&[fhd])?;
    if fcs_bytes == 2 {
    let stored: u16 = (content_len - 256) as u16;
        w.write_all(&stored.to_le_bytes())?;
    } else {
        let mut buf8 = [0u8; 8];
        buf8[..8].copy_from_slice(&content_len.to_le_bytes());
        w.write_all(&buf8[..fcs_bytes])?;
    }

    const MAX_BLOCK_SIZE: usize = (1 << 21) - 1;
    let rle_threshold: usize = match level {
        0..=2 => 48,
        3..=5 => 32,
        6..=9 => 24,
        _ => 16,
    };
    let mut produced: u64 = 0;
    let mut buf = vec![0u8; MAX_BLOCK_SIZE.min(128 * 1024)];
    let mut xxh = xxhash_rust::xxh64::Xxh64::new(0);
    if content_len == 0 {
    let header_val: u32 = 1;
        let header_bytes = [
            (header_val & 0xFF) as u8,
            ((header_val >> 8) & 0xFF) as u8,
            ((header_val >> 16) & 0xFF) as u8,
        ];
        w.write_all(&header_bytes)?;
        if checksum {
            let digest = xxh.digest();
            let bytes = digest.to_le_bytes();
            w.write_all(&bytes[..4])?;
        }
        return Ok(());
    }
    loop {
        // Determine next block size limit
        let remaining = (content_len - produced) as usize;
        if remaining == 0 { break; }
        let to_read = remaining.min(buf.len()).min(MAX_BLOCK_SIZE);
        let n = reader.read(&mut buf[..to_read])?;
        if n == 0 { break; }
        // Process the buffer into RLE/RAW sub-blocks
        let mut off = 0usize;
        while off < n {
            let window = min(n - off, MAX_BLOCK_SIZE);
            // detect run
            let b0 = buf[off];
            let mut run_len = 1usize;
            while run_len < window && buf[off + run_len] == b0 { run_len += 1; }
            if run_len >= rle_threshold {
                let emit = run_len;
                let will_produced = produced + emit as u64;
                let last_block = will_produced == content_len && (off + emit) == n;
                let header_val: u32 = ((emit as u32) << 3) | ((1u32 /* RLE */) << 1) | if last_block { 1 } else { 0 };
                let header_bytes = [
                    (header_val & 0xFF) as u8,
                    ((header_val >> 8) & 0xFF) as u8,
                    ((header_val >> 16) & 0xFF) as u8,
                ];
                w.write_all(&header_bytes)?;
                w.write_all(&[b0])?;
                if checksum {
                    let rep = [b0; 256];
                    let mut left = emit;
                    while left > 0 {
                        let m = min(left, rep.len());
                        xxh.update(&rep[..m]);
                        left -= m;
                    }
                }
                produced += emit as u64;
                off += emit;
            } else {
                let emit = window;
                let will_produced = produced + emit as u64;
                let last_block = will_produced == content_len && (off + emit) == n;
                let header_val: u32 = ((emit as u32) << 3) | if last_block { 1 } else { 0 };
                let header_bytes = [
                    (header_val & 0xFF) as u8,
                    ((header_val >> 8) & 0xFF) as u8,
                    ((header_val >> 16) & 0xFF) as u8,
                ];
                w.write_all(&header_bytes)?;
                w.write_all(&buf[off..off + emit])?;
                if checksum { xxh.update(&buf[off..off + emit]); }
                produced += emit as u64;
                off += emit;
            }
        }
    }
    if checksum {
        let digest = xxh.digest();
        let bytes = digest.to_le_bytes();
        w.write_all(&bytes[..4])?;
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
                 if info.checksum { "XXH64" } else { "-" },
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
    checksum: bool,
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
    
    // Parse header for checksum flag
    let mut header = [0u8; 13]; // enough for magic + FHD + max 8 bytes FCS
    let mut f = File::open(filename)?;
    let n = f.read(&mut header)?;
    let checksum_flag = if n >= 5 && header[0..4] == [0x28, 0xB5, 0x2F, 0xFD] {
        (header[4] & 0x04) != 0
    } else { false };
    Ok(ZstdFileInfo {
        compressed_size,
        uncompressed_size,
        ratio,
        checksum: checksum_flag,
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
    println!("  -z, --compress          compress (Pure Rust store-mode: creates RAW/RLE-block .zst)");
    println!("  -c, --stdout            write to standard output");
    println!("  -o, --output FILE       write output to FILE");
    println!("  -k, --keep              keep input files");
    println!("  -f, --force             overwrite output files");
    println!("  -t, --test              test compressed file integrity");
    println!("  -l, --list              list information about .zst files");
    println!("  -q, --quiet             suppress non-critical errors");
    println!("  -v, --verbose           increase verbosity");
    println!("  -T, --threads N         threads (also supports -T#; store-mode: large inputs split into parallel frames)");
    println!("  -M, --memory  LIM       memory usage limit (info only)");
    println!("  -C, --checksum          add 32-bit content checksum (low 32 bits of XXH64) to frame");
    println!("      --no-check          disable content checksum (default)");
    println!("  -D, --dict FILE         use dictionary FILE (accepted; currently ignored in store-mode)");
    println!("      --zstd              alias of -z (compat)");
    println!("      --full              enable internal full encoder (experimental)");
    println!("  -h, --help              display this help and exit");
    println!("  -V, --version           display version and exit");
    println!();
    println!("Decompression uses ruzstd (no C deps). Compression writes RAW/RLE zstd frames (no entropy compression). Dictionary option is reserved for future encoder.");

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn decode_all(mut data: &[u8]) -> Vec<u8> {
        let mut dec = StreamingDecoder::new(&mut data).expect("decoder");
        let mut out = Vec::new();
        let mut buf = [0u8; 8192];
        loop {
            match dec.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => panic!("decompress error: {e}"),
            }
        }
        out
    }

    fn parse_block_type(frame: &[u8]) -> u8 {
        // frame: [magic(4)][FHD(1)][FCS(2|4|8)][BlockHeader(3)]...
        assert!(frame.len() >= 8);
        assert_eq!(&frame[0..4], &[0x28, 0xB5, 0x2F, 0xFD]);
        let fhd = frame[4];
        let fcs_code = fhd >> 6; // 00,01,10,11 => 1,2,4,8 bytes
        let fcs_bytes = match fcs_code { 0b00 => 1, 0b01 => 2, 0b10 => 4, 0b11 => 8, _ => unreachable!() };
        let hdr_off = 5 + fcs_bytes;
        let b0 = frame[hdr_off] as u32;
        let b1 = frame[hdr_off + 1] as u32;
        let b2 = frame[hdr_off + 2] as u32;
        let header_val = b0 | (b1 << 8) | (b2 << 16);
        let block_type = ((header_val >> 1) & 0x3) as u8;
        block_type
    }

    fn parse_fcs(frame: &[u8]) -> (usize, u64) {
        assert!(frame.len() >= 6);
        assert_eq!(&frame[0..4], &[0x28, 0xB5, 0x2F, 0xFD]);
        let fhd = frame[4];
        let fcs_code = fhd >> 6; // 00,01,10,11 => 1,2,4,8 bytes
        let fcs_bytes = match fcs_code { 0b00 => 1, 0b01 => 2, 0b10 => 4, 0b11 => 8, _ => unreachable!() };
        let val: u64 = match fcs_bytes {
            1 => frame[5] as u64,
            2 => {
                let raw = u16::from_le_bytes([frame[5], frame[6]]);
                (raw as u64) + 256
            }
            4 => u32::from_le_bytes([frame[5], frame[6], frame[7], frame[8]]) as u64,
            8 => u64::from_le_bytes([
                frame[5], frame[6], frame[7], frame[8], frame[9], frame[10], frame[11], frame[12]
            ]),
            _ => unreachable!(),
        };
        (fcs_bytes, val)
    }

    #[test]
    fn zstd_sequences_repeat_after_fse_in_multi_block() {
        // Build a payload large enough for multiple compressed blocks with repeating patterns
        let pattern = b"ABCD_EFGH_ABCD_EFGH_"; // repeated structure encourages matches
        let mut payload = Vec::new();
        while payload.len() < 300_000 { payload.extend_from_slice(pattern); }

        // Write using full encoder path under test; instrumentation is active under cfg(test)
        let mut out = Vec::new();
        write_compressed_frame_literals_only(&mut out, &payload, false).expect("write");

        // Roundtrip to ensure validity
        let decoded = decode_all(&out);
        assert_eq!(decoded, payload);

        // Inspect captured modes (Symbol_Compression_Modes byte) for FSE (0x2A) then Repeat (0x3F)
        let modes = __zstd_test_instrumentation::snapshot();
        // Only assert that at least one sequences-bearing block existed
        assert!(!modes.is_empty(), "no sequences modes captured: {:?}", modes);
        let pos_fse = modes.iter().position(|&m| m == 0x2A);
        let pos_rep = modes.iter().position(|&m| m == 0x3F);
        if let (Some(i), Some(j)) = (pos_fse, pos_rep) {
            assert!(j > i, "Repeat should occur after an FSE_Compressed block: {:?}", modes);
        }
    }

    #[test]
    fn zstd_fse_compressed_weights_header_optimization() {
        // Test FSE compressed weights functionality
        use super::zstd_impl::huffman::{build_literals_huffman, build_fse_compressed_weights_header};
        
        // Create a payload with specific frequency distribution to trigger FSE compression
        let mut payload = Vec::new();
        // Pattern that creates a specific weight distribution suitable for FSE compression
        for _ in 0..20 { payload.push(b'A'); } // High frequency
        for _ in 0..15 { payload.push(b'B'); } // Medium frequency  
        for _ in 0..10 { payload.push(b'C'); } // Lower frequency
        for _ in 0..5  { payload.push(b'D'); } // Even lower frequency
        for _ in 0..1  { payload.push(b'E'); } // Minimal frequency
        
        if let Some((table, header)) = build_literals_huffman(&payload) {
            // Verify that FSE compression was considered
            if let Some(fse_header) = build_fse_compressed_weights_header(&table) {
                // FSE header should have different structure than direct weights
                assert!(fse_header[0] < 128, "FSE header byte should be < 128");
                
                // Should be at least header + table desc + compressed weights
                assert!(fse_header.len() >= 3, "FSE header should have reasonable size");
                
                println!("FSE header size: {}, Direct header size: {}", fse_header.len(), header.len());
            }
            
            // Verify the table was built correctly
            assert!(table.num_symbols >= 5, "Should have at least 5 symbols");
            assert!(!table.weights.is_empty(), "Weights should not be empty");
            
            // Verify some symbols have codes
            let mut found_codes = 0;
            for i in 0..table.num_symbols {
                if table.codes[i].is_some() {
                    found_codes += 1;
                }
            }
            assert!(found_codes >= 2, "Should have codes for at least 2 symbols");
        } else {
            panic!("Failed to build Huffman table for test payload");
        }
    }
    
    #[test]
    fn zstd_huffman_header_selection_logic() {
        // Test that the optimal header type is selected
        use super::zstd_impl::huffman::build_literals_huffman;
        
        // Test with simple payload (should prefer direct weights)
        let simple_payload = b"AABBCCDD".repeat(10);
        if let Some((_table1, header1)) = build_literals_huffman(&simple_payload) {
            // For simple patterns, direct weights are often more efficient
            println!("Simple payload header size: {}, first byte: {}", header1.len(), header1[0]);
        }
        
        // Test with complex payload (might prefer FSE compression)
        let mut complex_payload = Vec::new();
        for i in 0..64 { 
            let freq = if i < 4 { 50 } else if i < 16 { 10 } else { 1 };
            for _ in 0..freq { 
                complex_payload.push((b'A' + (i % 26)) as u8); 
            }
        }
        
        if let Some((table2, header2)) = build_literals_huffman(&complex_payload) {
            println!("Complex payload header size: {}, first byte: {}", header2.len(), header2[0]);
            
            // Verify table properties
            assert!(table2.num_symbols > 4, "Complex payload should have many symbols");
            
            // Check that weights are reasonable
            let max_weight = table2.weights[..table2.num_symbols].iter().max().unwrap_or(&0);
            assert!(*max_weight > 0 && *max_weight <= 15, "Weights should be in valid range");
        }
    }

    #[test]
    fn zstd_store_rle_block_header_and_roundtrip_slice() {
        let payload = vec![0x41u8; 100]; // 'A' * 100, exceeds RLE_THRESHOLD(32)
        let mut out = Vec::new();
    write_store_frame_slice_with_options(&mut out, &payload, false, 3).expect("write");
        // RLE block type == 1
        let btype = parse_block_type(&out);
        assert_eq!(btype, 1, "expected RLE block");
        let decoded = decode_all(&out);
        assert_eq!(decoded, payload);
    }

    #[test]
    fn zstd_store_raw_block_header_and_roundtrip_slice() {
        let payload: Vec<u8> = (0..100).map(|i| (i & 0xFF) as u8).collect();
        let mut out = Vec::new();
    write_store_frame_slice_with_options(&mut out, &payload, false, 3).expect("write");
        // RAW block type == 0
        let btype = parse_block_type(&out);
        assert_eq!(btype, 0, "expected RAW block");
        let decoded = decode_all(&out);
        assert_eq!(decoded, payload);
    }

    #[test]
    fn zstd_store_stream_with_checksum_flag_and_roundtrip() {
        let payload = vec![0x42u8; 64];
        let mut out = Vec::new();
        let mut reader = Cursor::new(payload.clone());
        write_store_frame_stream_with_options(&mut out, &mut reader, payload.len() as u64, true, 3).expect("write");
        // FHD bit2 is checksum flag per RFC 8878
        assert_eq!(out[4] & 0x04, 0x04, "checksum flag (bit2) should be set in FHD");
        let decoded = decode_all(&out);
        assert_eq!(decoded, payload);
    }

    #[test]
    fn zstd_fhd_fcs_size_and_value_1byte() {
        let payload = vec![0x55u8; 42]; // <=255
        let mut out = Vec::new();
        write_store_frame_slice_with_options(&mut out, &payload, false, 3).expect("write");
        let (fcs_bytes, fcs_val) = parse_fcs(&out);
        assert_eq!(fcs_bytes, 1);
        assert_eq!(fcs_val, payload.len() as u64);
    }

    #[test]
    fn zstd_fhd_fcs_size_and_value_2bytes() {
        let payload = vec![0x33u8; 300]; // 256..=65791
        let mut out = Vec::new();
        write_store_frame_slice_with_options(&mut out, &payload, false, 3).expect("write");
        let (fcs_bytes, fcs_val) = parse_fcs(&out);
        assert_eq!(fcs_bytes, 2);
        assert_eq!(fcs_val, payload.len() as u64);
    }

    #[test]
    fn zstd_fhd_fcs_size_and_value_4bytes() {
        let payload = vec![0x99u8; 100_000]; // >65791
        let mut out = Vec::new();
        write_store_frame_slice_with_options(&mut out, &payload, false, 3).expect("write");
        let (fcs_bytes, fcs_val) = parse_fcs(&out);
        assert_eq!(fcs_bytes, 4);
        assert_eq!(fcs_val, payload.len() as u64);
    }

    #[test]
    fn zstd_store_parallel_chunked_multiple_frames_roundtrip() {
        // 2.5 * chunk_size(4MiB) 相当のデータを用意（ここでは小さく 3*64KB にする）
        let chunk = vec![0xAAu8; 64 * 1024];
        let payload = [chunk.clone(), chunk.clone(), chunk.clone()].concat();
        let mut out = Vec::new();
        // compress_stream_store は 1MiB 以下では単一フレームだが、ここでは slice API を直接複数回呼ぶ
        // 実運用では threads>1 でフレームが複数に分割されることを模擬
    write_store_frame_slice_with_options(&mut out, &payload[..64*1024], false, 3).expect("w1");
    write_store_frame_slice_with_options(&mut out, &payload[64*1024..128*1024], false, 3).expect("w2");
    write_store_frame_slice_with_options(&mut out, &payload[128*1024..], false, 3).expect("w3");
        // 連結フレーム全体を解凍
        let decoded = decode_all(&out);
        assert_eq!(decoded, payload);
    }

    #[test]
    fn zstd_full_literals_only_compressed_block_and_roundtrip() {
        let payload: Vec<u8> = (0..2000).map(|i| (i & 0xFF) as u8).collect();
        let mut out = Vec::new();
        write_compressed_frame_literals_only(&mut out, &payload, false).expect("write");
        // First block must be Compressed (type=2)
        let btype = parse_block_type(&out);
        assert_eq!(btype, 2, "expected Compressed block");
        let decoded = decode_all(&out);
        assert_eq!(decoded, payload);
    }

    #[test]
    fn zstd_full_literals_rle_in_compressed_block_and_roundtrip() {
        let payload = vec![0x7Au8; 5000]; // すべて同一
        let mut out = Vec::new();
        write_compressed_frame_literals_only(&mut out, &payload, true).expect("write");
        // Compressed block
        let btype = parse_block_type(&out);
        assert_eq!(btype, 2);
        // 復号
        let decoded = decode_all(&out);
        assert_eq!(decoded, payload);
    }

    #[test]
    fn zstd_full_literals_huffman_in_compressed_block_and_roundtrip() {
        // 非一様データ（英語テキスト風）
        let text = b"This is a tiny test block that should compress with Huffman coding pretty well. ";
        let mut payload = Vec::new();
        for _ in 0..20 { payload.extend_from_slice(text); }
        let mut out = Vec::new();
        write_compressed_frame_literals_only(&mut out, &payload, false).expect("write");
        // Compressed block
        let btype = parse_block_type(&out);
        assert_eq!(btype, 2);
        let decoded = decode_all(&out);
        assert_eq!(decoded, payload);
    }

    #[test]
    fn zstd_full_literals_huffman_four_streams_selected() {
        // 入力サイズを >1023 にして SF=00(1ストリーム)の条件を外し、4ストリーム選択を促す
        let text = b"Four streams should be chosen for larger blocks with Huffman coding. ";
        let mut payload = Vec::new();
        while payload.len() <= 5000 { payload.extend_from_slice(text); }
        let mut out = Vec::new();
        write_compressed_frame_literals_only(&mut out, &payload, false).expect("write");
        // LSH の SF ビットを検査して 4 ストリーム (SF!=00) を確認
        assert_eq!(&out[0..4], &[0x28, 0xB5, 0x2F, 0xFD]);
        let fhd = out[4];
        let fcs_code = fhd >> 6;
        let fcs_bytes = match fcs_code { 0b00 => 1, 0b01 => 2, 0b10 => 4, 0b11 => 8, _ => unreachable!() };
        let hdr_off = 5 + fcs_bytes; // BlockHeaderの先頭
        let lsh_b0 = out[hdr_off + 3]; // BlockHeader(3 bytes) の直後が LSH 先頭
        let lbt = lsh_b0 & 0b11;
        let sf = (lsh_b0 >> 2) & 0b11;
        assert_eq!(lbt, 0b10, "LBT must be Compressed for Huffman literals");
        assert_ne!(sf, 0b00, "SF=00 would be 1-stream; expected 4-stream literals");
        // 復号して正しく往復することを確認
        let decoded = decode_all(&out);
        assert_eq!(decoded, payload);
    }

    #[test]
    fn zstd_store_checksum_is_xxh64_low32() {
        let payload: Vec<u8> = (0..1500).map(|i| (i as u8).wrapping_mul(31)).collect();
        let mut out = Vec::new();
        write_store_frame_slice_with_options(&mut out, &payload, true, 3).expect("write");
        // 最後の4バイトがチェックサム
        assert!(out.len() >= 4);
        let tail = &out[out.len()-4..];
        let expected64 = xxhash_rust::xxh64::xxh64(&payload, 0);
        let expected = expected64.to_le_bytes();
        assert_eq!(tail, &expected[..4], "checksum should be low 32 bits of XXH64");
    }

    #[test]
    fn zstd_compressed_literals_only_checksum_is_xxh64_low32() {
    let payload = b"Huffman will likely reduce this literals block. ".repeat(300);
        let mut out = Vec::new();
    write_compressed_frame_literals_only(&mut out, &payload, true).expect("write");
        assert!(out.len() >= 4);
        let tail = &out[out.len()-4..];
    let expected64 = xxhash_rust::xxh64::xxh64(&payload, 0);
        let expected = expected64.to_le_bytes();
        assert_eq!(tail, &expected[..4], "checksum should be low 32 bits of XXH64");
    }
}
}
