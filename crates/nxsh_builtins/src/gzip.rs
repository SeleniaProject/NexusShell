//! Pure Rust `gzip` builtin for compression/decompression with DEFLATE algorithm
//!
//! This module provides a complete Pure Rust implementation of gzip/gunzip
//! functionality using the flate2 crate, eliminating system dependencies.

use anyhow::{Context, Result};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use std::fs::File;
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GzipOptions {
    pub decompress: bool,
    pub stdout: bool,
    pub keep: bool,
    pub force: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub test: bool,
    pub level: u32,
    pub fast: bool,
    pub best: bool,
    pub no_name: bool,
    pub name: bool,
    pub ascii: bool,
    pub recursive: bool,
}

impl Default for GzipOptions {
    fn default() -> Self {
        Self {
            decompress: false,
            stdout: false,
            keep: false,
            force: false,
            verbose: false,
            quiet: false,
            test: false,
            level: 6, // Default compression level
            fast: false,
            best: false,
            no_name: false,
            name: false,
            ascii: false,
            recursive: false,
        }
    }
}

/// CLI wrapper for gzip compression/decompression
pub fn gzip_cli(args: &[String]) -> Result<()> {
    let mut options = GzipOptions::default();
    let mut input_files = Vec::new();
    let mut i = 0;

    // Parse command line arguments
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--decompress" | "--uncompress" => {
                options.decompress = true;
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
            "-n" | "--no-name" => {
                options.no_name = true;
            }
            "-N" | "--name" => {
                options.name = true;
            }
            "-a" | "--ascii" => {
                options.ascii = true;
            }
            "-r" | "--recursive" => {
                options.recursive = true;
            }
            "-1" => { options.level = 1; options.fast = true; }
            "-2" => options.level = 2,
            "-3" => options.level = 3,
            "-4" => options.level = 4,
            "-5" => options.level = 5,
            "-6" => options.level = 6,
            "-7" => options.level = 7,
            "-8" => options.level = 8,
            "-9" => { options.level = 9; options.best = true; }
            "--fast" => { options.level = 1; options.fast = true; }
            "--best" => { options.level = 9; options.best = true; }
            "-h" | "--help" => {
                print_gzip_help();
                return Ok(());
            }
            "-V" | "--version" => {
                print_gzip_version();
                return Ok(());
            }
            "-L" | "--license" => {
                print_gzip_license();
                return Ok(());
            }
            arg if arg.starts_with('-') => {
                // Handle combined short options
                if arg.len() > 2 && arg.starts_with('-') && !arg.starts_with("--") {
                    for ch in arg[1..].chars() {
                        match ch {
                            'd' => options.decompress = true,
                            'c' => options.stdout = true,
                            'k' => options.keep = true,
                            'f' => options.force = true,
                            'v' => options.verbose = true,
                            'q' => options.quiet = true,
                            't' => options.test = true,
                            'n' => options.no_name = true,
                            'N' => options.name = true,
                            'a' => options.ascii = true,
                            'r' => options.recursive = true,
                            '1'..='9' => {
                                options.level = ch.to_digit(10).unwrap();
                                if options.level == 1 { options.fast = true; }
                                if options.level == 9 { options.best = true; }
                            }
                            _ => return Err(anyhow::anyhow!("gzip: invalid option -- '{}'", ch)),
                        }
                    }
                } else {
                    return Err(anyhow::anyhow!("gzip: invalid option -- '{}'", arg));
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
        return test_gzip_files(&input_files, &options);
    }

    // Process files or stdin/stdout
    if input_files.is_empty() {
        process_stdio(&options)
    } else {
        process_files(&input_files, &options)
    }
}

/// CLI wrapper for gunzip decompression
pub fn gunzip_cli(args: &[String]) -> Result<()> {
    let mut modified_args = args.to_vec();
    modified_args.insert(0, "--decompress".to_string());
    gzip_cli(&modified_args)
}

/// CLI wrapper for zcat (decompress to stdout)
pub fn zcat_cli(args: &[String]) -> Result<()> {
    let mut modified_args = args.to_vec();
    modified_args.insert(0, "--decompress".to_string());
    modified_args.insert(1, "--stdout".to_string());
    gzip_cli(&modified_args)
}

/// Process stdin to stdout with compression/decompression
fn process_stdio(options: &GzipOptions) -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());

    if options.decompress {
        decompress_stream(&mut reader, &mut writer, options)
            .context("gzip: failed to decompress from stdin")?;
    } else {
        compress_stream(&mut reader, &mut writer, options)
            .context("gzip: failed to compress from stdin")?;
    }

    writer.flush().context("gzip: failed to flush output")?;
    Ok(())
}

/// Process multiple files with compression/decompression
fn process_files(input_files: &[String], options: &GzipOptions) -> Result<()> {
    let mut all_success = true;
    
    for filename in input_files {
        if let Err(e) = process_single_file(filename, options) {
            if !options.quiet {
                eprintln!("gzip: {filename}: {e}");
            }
            all_success = false;
            if !options.force {
                continue;
            }
        }
    }
    
    if !all_success {
        return Err(anyhow::anyhow!("gzip: some files failed to process"));
    }
    
    Ok(())
}

/// Process a single file with compression/decompression
fn process_single_file(filename: &str, options: &GzipOptions) -> Result<()> {
    let input_path = Path::new(filename);
    
    if !input_path.exists() {
        return Err(anyhow::anyhow!("No such file or directory"));
    }

    if input_path.is_dir() && !options.recursive {
        return Err(anyhow::anyhow!("{} is a directory -- ignored", filename));
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
            if !options.quiet {
                eprintln!("gzip: {out_file} already exists; not overwritten");
            }
            return Ok(());
        }
    }

    let input_file = File::open(input_path)
        .with_context(|| format!("gzip: can't open {filename}"))?;
    
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
            .with_context(|| format!("gzip: can't create {output_file}"))?;
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
                .with_context(|| format!("gzip: can't remove {filename}"))?;
        }

        // Print compression statistics
        if options.verbose && !options.quiet {
            if options.decompress {
                let ratio = if final_size > 0 {
                    original_size as f64 / final_size as f64
                } else {
                    0.0
                };
                println!("{}:\t{:.1}% -- replaced with {}", 
                    filename, 
                    (1.0 - 1.0/ratio) * 100.0, 
                    output_file
                );
            } else {
                let ratio = if original_size > 0 {
                    final_size as f64 / original_size as f64
                } else {
                    0.0
                };
                println!("{}:\t{:.1}% -- replaced with {}", 
                    filename, 
                    (1.0 - ratio) * 100.0, 
                    output_file
                );
            }
        }
    }

    Ok(())
}

/// Compress data stream using Pure Rust implementation
fn compress_stream<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    options: &GzipOptions,
) -> Result<()> {
    let compression_level = match options.level {
        0 => Compression::none(),
        1 => Compression::fast(),
        2..=8 => Compression::new(options.level),
        9 => Compression::best(),
        _ => Compression::default(),
    };

    let mut encoder = GzEncoder::new(writer, compression_level);
    std::io::copy(reader, &mut encoder)
        .context("Failed to compress data")?;
    
    encoder.finish()
        .context("Failed to finalize compression")?;
    
    Ok(())
}

/// Decompress data stream using Pure Rust implementation
fn decompress_stream<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    _options: &GzipOptions,
) -> Result<()> {
    let mut decoder = GzDecoder::new(reader);
    std::io::copy(&mut decoder, writer)
        .context("Failed to decompress gzip data")?;
    
    Ok(())
}

/// Determine compressed filename
fn determine_compressed_filename(input: &str) -> String {
    // 以前は条件分岐していたが両分岐が同一出力だったため簡素化
    format!("{input}.gz")
}

/// Determine decompressed filename by removing .gz extension
fn determine_decompressed_filename(input: &str) -> Result<String> {
    let path = Path::new(input);
    
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
    if let Some(stem) = file_name.strip_suffix(".gz") {
            if let Some(parent) = path.parent() {
                Ok(parent.join(stem).to_string_lossy().to_string())
            } else {
                Ok(stem.to_string())
            }
    } else if let Some(stem) = file_name.strip_suffix(".tgz") {
            if let Some(parent) = path.parent() {
                Ok(parent.join(format!("{stem}.tar")).to_string_lossy().to_string())
            } else {
                Ok(format!("{stem}.tar"))
            }
    } else if let Some(stem) = file_name.strip_suffix(".Z") {
            if let Some(parent) = path.parent() {
                Ok(parent.join(stem).to_string_lossy().to_string())
            } else {
                Ok(stem.to_string())
            }
        } else {
            // If no recognized extension, strip .gz anyway or add .out
            if input.ends_with(".gz") {
                Ok(input.strip_suffix(".gz").unwrap().to_string())
            } else {
                Ok(format!("{input}.out"))
            }
        }
    } else {
        Err(anyhow::anyhow!("gzip: can't recover original filename"))
    }
}

/// Test integrity of compressed files
fn test_gzip_files(files: &[String], options: &GzipOptions) -> Result<()> {
    let mut all_ok = true;
    
    for filename in files {
        match test_single_file(filename, options) {
            Ok(()) => {
                if !options.quiet {
                    println!("{filename}:\tOK");
                }
            }
            Err(e) => {
                if !options.quiet {
                    eprintln!("gzip: {filename}: {e}");
                }
                all_ok = false;
            }
        }
    }
    
    if !all_ok {
        return Err(anyhow::anyhow!("gzip: some files failed integrity test"));
    }
    
    Ok(())
}

/// Test integrity of a single compressed file
fn test_single_file(filename: &str, _options: &GzipOptions) -> Result<()> {
    let file = File::open(filename)
        .with_context(|| format!("gzip: can't open {filename}"))?;
    
    let mut reader = BufReader::new(file);
    let mut null_writer = NullWriter;
    let test_options = GzipOptions::default();
    
    decompress_stream(&mut reader, &mut null_writer, &test_options)
        .with_context(|| format!("gzip: {filename} is corrupt"))?;
    
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
fn print_gzip_help() {
    println!("Usage: gzip [OPTION]... [FILE]...");
    println!("Compress or uncompress FILEs (by default, compress FILES in-place).");
    println!();
    println!("Mandatory arguments to long options are mandatory for short options too.");
    println!();
    println!("  -c, --stdout      write on standard output, keep original files unchanged");
    println!("  -d, --decompress  decompress");
    println!("  -f, --force       force overwrite of output file and compress links");
    println!("  -h, --help        give this help");
    println!("  -k, --keep        keep (don't delete) input files");
    println!("  -l, --list        list compressed file contents");
    println!("  -L, --license     display software license");
    println!("  -n, --no-name     do not save or restore the original name and time stamp");
    println!("  -N, --name        save or restore the original name and time stamp");
    println!("  -q, --quiet       suppress all warnings");
    println!("  -r, --recursive   operate recursively on directories");
    println!("  -S, --suffix=SUF  use suffix SUF on compressed files");
    println!("  -t, --test        test compressed file integrity");
    println!("  -v, --verbose     verbose mode");
    println!("  -V, --version     display version number");
    println!("  -1, --fast        compress faster");
    println!("  -9, --best        compress better");
    println!();
    println!("With no FILE, or when FILE is -, read standard input.");
    println!();
    println!("Report bugs to <https://github.com/SeleniaProject/NexusShell/issues>");
}

/// Print version information
fn print_gzip_version() {
    println!("gzip (NexusShell Pure Rust) {}", env!("CARGO_PKG_VERSION"));
    println!("Copyright (C) 2024 NexusShell Project");
    println!("This is free software; see the source for copying conditions.");
    println!("There is NO warranty; not even for MERCHANTABILITY or FITNESS FOR A");
    println!("PARTICULAR PURPOSE.");
    println!();
    println!("Written using flate2 crate for Pure Rust DEFLATE implementation.");
}

/// Print license information
fn print_gzip_license() {
    println!("gzip (NexusShell Pure Rust) {}", env!("CARGO_PKG_VERSION"));
    println!("Copyright (C) 2024 NexusShell Project");
    println!();
    println!("This program is free software; you can redistribute it and/or modify");
    println!("it under the terms of the MIT License or Apache License 2.0.");
    println!();
    println!("This program is distributed in the hope that it will be useful,");
    println!("but WITHOUT ANY WARRANTY; without even the implied warranty of");
    println!("MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.");
    println!();
    println!("Pure Rust implementation eliminates C library dependencies.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    #[allow(unused_imports)]
    use std::io::Write as _;

    #[test]
    fn test_compress_decompress() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Hello, World! This is a test for gzip compression.";
        temp_file.write_all(test_data).unwrap();
        
        let options = GzipOptions::default();
        
        // Test compression
        let mut reader = BufReader::new(File::open(temp_file.path()).unwrap());
        let mut compressed = Vec::new();
        {
            let mut writer = BufWriter::new(&mut compressed);
            compress_stream(&mut reader, &mut writer, &options).unwrap();
        }
        
        // Test decompression
        let mut reader = BufReader::new(&compressed[..]);
        let mut decompressed = Vec::new();
        {
            let mut writer = BufWriter::new(&mut decompressed);
            decompress_stream(&mut reader, &mut writer, &options).unwrap();
        }
        
        assert_eq!(test_data, &decompressed[..]);
    }
    
    #[test]
    fn test_filename_operations() {
        assert_eq!(determine_compressed_filename("test.txt"), "test.txt.gz");
        assert_eq!(determine_compressed_filename("test.tar"), "test.tar.gz");
        
        assert_eq!(determine_decompressed_filename("test.txt.gz").unwrap(), "test.txt");
        assert_eq!(determine_decompressed_filename("test.tgz").unwrap(), "test.tar");
    }
}
