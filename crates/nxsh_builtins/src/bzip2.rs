//! Pure Rust `bzip2` builtin for decompression (decode-only)
//! 
//! This module provides a Pure Rust implementation of bzip2 decompression
//! using the bzip2-rs crate (decompression-only). Compression is not
//! available in this build. Use gzip or xz for compression, or an external
//! bzip2 binary if bzip2 compression is required.

use anyhow::{Context, Result};
use bzip2_rs::DecoderReader;
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Bzip2Options {
    pub decompress: bool,
    pub stdout: bool,
    pub keep: bool,
    pub force: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub test: bool,
    pub level: u32,
    pub small: bool,
}

impl Default for Bzip2Options {
    fn default() -> Self {
        Self {
            decompress: false,
            stdout: false,
            keep: false,
            force: false,
            verbose: false,
            quiet: false,
            test: false,
            level: 9,  // Default compression level
            small: false,
        }
    }
}

/// CLI wrapper function for bzip2 decompression
/// Provides bzip2-utils compatibility (decompression-only) with Pure Rust implementation
pub fn bzip2_cli(args: &[String]) -> Result<()> {
    let mut options = Bzip2Options::default();
    let mut input_files = Vec::new();
    let mut i = 0;

    // Parse command line arguments with full bzip2 compatibility
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
            "-s" | "--small" => {
                options.small = true;
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
            "-h" | "--help" => {
                print_bzip2_help();
                return Ok(());
            }
            "-V" | "--version" | "-L" | "--license" => {
                print_bzip2_version();
                return Ok(());
            }
            arg if arg.starts_with('-') => {
                // Handle combined short options
                if arg.len() > 2 && arg.starts_with('-') && !arg.starts_with("--") {
                    for ch in arg[1..].chars() {
                        match ch {
                            'd' => options.decompress = true,
                            'z' => options.decompress = false,
                            'c' => options.stdout = true,
                            'k' => options.keep = true,
                            'f' => options.force = true,
                            'v' => options.verbose = true,
                            'q' => options.quiet = true,
                            't' => options.test = true,
                            's' => options.small = true,
                            '1'..='9' => options.level = ch.to_digit(10).unwrap(),
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

/// Process stdin to stdout with compression/decompression
fn process_stdio(options: &Bzip2Options) -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());

    if options.decompress {
        decompress_stream(&mut reader, &mut writer, options)
            .context("Failed to decompress from stdin")?;
    } else {
        compress_stream(&mut reader, &mut writer, options)
            .context("Failed to compress from stdin")?;
    }

    writer.flush().context("Failed to flush output")?;
    Ok(())
}

/// Process multiple files with compression/decompression
fn process_files(input_files: &[String], options: &Bzip2Options) -> Result<()> {
    let mut all_success = true;
    
    for filename in input_files {
        if let Err(e) = process_single_file(filename, options) {
            if !options.quiet {
                eprintln!("bzip2: {filename}: {e}");
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
fn process_single_file(filename: &str, options: &Bzip2Options) -> Result<()> {
    let input_path = Path::new(filename);
    
    if !input_path.exists() {
        return Err(anyhow::anyhow!("No such file or directory"));
    }

    let original_size = input_path.metadata()?.len();

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
            compress_stream(&mut reader, &mut writer, options)?;
        }
        writer.flush()?;
    } else if let Some(output_file) = output_filename {
        let out_file = File::create(&output_file)
            .with_context(|| format!("Cannot create output file '{output_file}'"))?;
        let mut writer = BufWriter::new(out_file);
        
        if options.decompress {
            decompress_stream(&mut reader, &mut writer, options)?;
        } else {
            compress_stream(&mut reader, &mut writer, options)?;
        }
        writer.flush()?;

        // Get final size for statistics
        let final_size = Path::new(&output_file).metadata()?.len();

        // Remove input file if not keeping it
        if !options.keep {
            std::fs::remove_file(input_path)
                .with_context(|| format!("Cannot remove input file '{filename}'"))?;
        }

        // Print compression statistics
        if !options.quiet && options.verbose {
            if options.decompress {
                println!("  {filename}: done");
            } else {
                let ratio = if original_size > 0 {
                    final_size as f64 / original_size as f64
                } else {
                    0.0
                };
                let compression_ratio = if ratio > 0.0 { 1.0 / ratio } else { 0.0 };
                let bits_per_byte = if original_size > 0 {
                    (final_size * 8) as f64 / original_size as f64
                } else {
                    0.0
                };
                let saved_percent = (1.0 - ratio) * 100.0;

                println!(
                    "  {filename}: {compression_ratio:.1}:1, {bits_per_byte:.1} bits/byte, {saved_percent:.2}% saved, {original_size} in, {final_size} out."
                );
            }
        }
    }

    Ok(())
}

/// Compress data stream (not supported in this build)
fn compress_stream<R: Read, W: Write>(
    _reader: &mut R,
    _writer: &mut W,
    _options: &Bzip2Options,
) -> Result<()> {
    // bzip2-rs is decompression-only, compression not available
    Err(anyhow::anyhow!(
        "bzip2 compression not supported (decode-only). Use gzip or xz for compression, or an external bzip2 binary."
    ))
}

/// Decompress data stream using Pure Rust implementation
fn decompress_stream<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    _options: &Bzip2Options,
) -> Result<()> {
    let mut decoder = DecoderReader::new(reader);
    std::io::copy(&mut decoder, writer)
        .context("Failed to decompress bzip2 data")?;
    
    Ok(())
}

/// Determine compressed filename
fn determine_compressed_filename(input: &str) -> String {
    if input.ends_with(".tar") {
        format!("{}.tbz2", input.strip_suffix(".tar").unwrap())
    } else {
        format!("{input}.bz2")
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
                Ok(parent.join(format!("{stem}.tar")).to_string_lossy().to_string())
            } else {
                Ok(format!("{stem}.tar"))
            }
        } else if file_name.ends_with(".tbz") {
            let stem = file_name.strip_suffix(".tbz").unwrap();
            if let Some(parent) = path.parent() {
                Ok(parent.join(format!("{stem}.tar")).to_string_lossy().to_string())
            } else {
                Ok(format!("{stem}.tar"))
            }
        } else {
            // If no recognized extension, add .out
            Ok(format!("{input}.out"))
        }
    } else {
        Err(anyhow::anyhow!("Cannot determine output filename"))
    }
}

/// Test integrity of compressed files
fn test_bzip2_files(files: &[String], options: &Bzip2Options) -> Result<()> {
    for filename in files {
        match test_single_file(filename, options) {
            Ok(()) => {
                if !options.quiet {
                    println!("{filename}: ok");
                }
            }
            Err(e) => {
                if !options.quiet {
                    eprintln!("bzip2: {filename}: {e}");
                }
                return Err(e);
            }
        }
    }
    Ok(())
}

/// Test integrity of a single compressed file
fn test_single_file(filename: &str, _options: &Bzip2Options) -> Result<()> {
    let file = File::open(filename)
        .with_context(|| format!("Cannot open file '{filename}'"))?;
    
    let mut reader = BufReader::new(file);
    let mut null_writer = NullWriter;
    let test_options = Bzip2Options::default();
    
    decompress_stream(&mut reader, &mut null_writer, &test_options)
        .with_context(|| format!("Integrity test failed for '{filename}'"))?;
    
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
fn print_bzip2_help() {
    println!("bzip2 (NexusShell Pure Rust implementation) - decompression only");
    println!();
    println!("usage: bzip2 [flags and input files in any order]");
    println!();
    println!("   -h --help           print this message");
    println!("   -d --decompress     decompress (supported)");
    println!("   -z --compress       compress (not available in this build)");
    println!("   -k --keep           keep (don't delete) input files");
    println!("   -f --force          overwrite existing output files");
    println!("   -t --test           test compressed file integrity");
    println!("   -c --stdout         output to standard out");
    println!("   -q --quiet          suppress noncritical error messages");
    println!("   -v --verbose        be verbose (a 2nd -v gives more)");
    println!("   -L --license        display software version & license");
    println!("   -V --version        display software version & license");
    println!("   -s --small          use less memory (at most 2500k)");
    println!("   -1 .. -9            set block size to 100k .. 900k (compression not available)");
    println!("   --fast              alias for -1 (compression not available)");
    println!("   --best              alias for -9 (compression not available)");
    println!();
    println!("   If no file names are given, bzip2 reads from standard input and writes to standard output.");
    println!("   This build is decompression-only. For compression, use gzip/xz or an external bzip2 binary.");
    println!();
    println!("Pure Rust decompression using bzip2-rs (decode-only).");
    println!("   Report bugs to: <https://github.com/SeleniaProject/NexusShell/issues>");
}

/// Print version information
fn print_bzip2_version() {
    println!("bzip2 (NexusShell Pure Rust implementation) {}", env!("CARGO_PKG_VERSION"));
    println!("Decompression-only via bzip2-rs (decode-only backend)");
    println!();
    println!("This is free software; you can redistribute it and/or modify");
    println!("it under the terms of the MIT or Apache License 2.0.");
    println!();
    println!("Compression is not available in this build. Use gzip or xz for compression,");
    println!("or an external bzip2 binary if needed.");
}
