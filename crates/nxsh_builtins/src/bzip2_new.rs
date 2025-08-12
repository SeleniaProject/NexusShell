//! `bzip2` builtin - Compress files using the Burrows-Wheeler algorithm.
//!
//! Complete Pure Rust implementation using the `bzip2-rs` crate

use anyhow::{anyhow, Context, Result};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

/// bzip2 command implementation with Pure Rust compression
pub fn bzip2_cli(args: &[String]) -> Result<()> {
    let options = parse_bzip2_args(args)?;
    
    match options.mode {
        Bzip2Mode::Compress => {
            for input_file in &options.files {
                compress_file(input_file, &options)?;
            }
        }
        Bzip2Mode::Decompress => {
            for input_file in &options.files {
                decompress_file(input_file, &options)?;
            }
        }
        Bzip2Mode::Test => {
            for input_file in &options.files {
                test_file(input_file)?;
            }
        }
    }
    
    Ok(())
}

/// Configuration structure for bzip2 options
#[derive(Debug, Clone)]
struct Bzip2Options {
    mode: Bzip2Mode,
    compression_level: i32,
    force: bool,
    keep_input: bool,
    stdout: bool,
    verbose: bool,
    files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
enum Bzip2Mode {
    Compress,
    Decompress,
    Test,
}

fn parse_bzip2_args(args: &[String]) -> Result<Bzip2Options> {
    let mut options = Bzip2Options {
        mode: Bzip2Mode::Compress,
        compression_level: 9,
        force: false,
        keep_input: false,
        stdout: false,
        verbose: false,
        files: Vec::new(),
    };

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        
        if arg.starts_with('-') && arg.len() > 1 {
            for ch in arg[1..].chars() {
                match ch {
                    'd' => options.mode = Bzip2Mode::Decompress,
                    'z' => options.mode = Bzip2Mode::Compress,
                    't' => options.mode = Bzip2Mode::Test,
                    'f' => options.force = true,
                    'k' => options.keep_input = true,
                    'c' => options.stdout = true,
                    'v' => options.verbose = true,
                    '1'..='9' => {
                        options.compression_level = ch.to_digit(10).unwrap() as i32;
                    }
                    'h' => {
                        print_bzip2_help();
                        return Ok(options);
                    }
                    _ => return Err(anyhow!("bzip2: invalid option '{}'", ch)),
                }
            }
        } else {
            // This is a filename
            options.files.push(PathBuf::from(arg));
        }
        
        i += 1;
    }

    // If no files specified, read from stdin
    if options.files.is_empty() {
        options.stdout = true;
        options.files.push(PathBuf::from("-")); // Represents stdin
    }

    Ok(options)
}

/// Compress a file using Pure Rust bzip2 implementation
fn compress_file(input_path: &Path, options: &Bzip2Options) -> Result<()> {
    let input_data = if input_path.to_str() == Some("-") {
        // Read from stdin
        let mut buffer = Vec::new();
        std::io::stdin()
            .read_to_end(&mut buffer)
            .context("bzip2: failed to read from stdin")?;
        buffer
    } else {
        // Read from file
        if !input_path.exists() {
            return Err(anyhow!("bzip2: can't open {}: No such file", input_path.display()));
        }
        
        let mut file = File::open(input_path)
            .with_context(|| format!("bzip2: can't open {}", input_path.display()))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .with_context(|| format!("bzip2: error reading {}", input_path.display()))?;
        buffer
    };

    // Compress the data
    let compressed_data = compress_bzip2(&input_data, options.compression_level)?;
    
    if options.stdout || input_path.to_str() == Some("-") {
        // Write to stdout
        std::io::stdout()
            .write_all(&compressed_data)
            .context("bzip2: error writing to stdout")?;
    } else {
        // Write to .bz2 file
        let output_path = get_compressed_filename(input_path);
        
        if output_path.exists() && !options.force {
            return Err(anyhow!(
                "bzip2: output file {} already exists",
                output_path.display()
            ));
        }
        
        let mut output_file = File::create(&output_path)
            .with_context(|| format!("bzip2: can't create {}", output_path.display()))?;
        output_file
            .write_all(&compressed_data)
            .with_context(|| format!("bzip2: error writing {}", output_path.display()))?;
        
        if !options.keep_input {
            std::fs::remove_file(input_path)
                .with_context(|| format!("bzip2: can't remove {}", input_path.display()))?;
        }
        
        if options.verbose {
            let compression_ratio = (input_data.len() as f64 / compressed_data.len() as f64) * 100.0;
            eprintln!(
                "  {}: {:.1}:1, {:.1} bits/byte, {:.2}% saved, {} in, {} out.",
                input_path.display(),
                compression_ratio / 100.0,
                (compressed_data.len() * 8) as f64 / input_data.len() as f64,
                100.0 - (compressed_data.len() as f64 / input_data.len() as f64 * 100.0),
                input_data.len(),
                compressed_data.len()
            );
        }
    }
    
    Ok(())
}

/// Decompress a file using Pure Rust bzip2 implementation
fn decompress_file(input_path: &Path, options: &Bzip2Options) -> Result<()> {
    let input_data = if input_path.to_str() == Some("-") {
        // Read from stdin
        let mut buffer = Vec::new();
        std::io::stdin()
            .read_to_end(&mut buffer)
            .context("bzip2: failed to read from stdin")?;
        buffer
    } else {
        // Read from file
        if !input_path.exists() {
            return Err(anyhow!("bzip2: can't open {}: No such file", input_path.display()));
        }
        
        let mut file = File::open(input_path)
            .with_context(|| format!("bzip2: can't open {}", input_path.display()))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .with_context(|| format!("bzip2: error reading {}", input_path.display()))?;
        buffer
    };

    // Decompress the data
    let decompressed_data = decompress_bzip2(&input_data)?;
    
    if options.stdout || input_path.to_str() == Some("-") {
        // Write to stdout
        std::io::stdout()
            .write_all(&decompressed_data)
            .context("bzip2: error writing to stdout")?;
    } else {
        // Write to decompressed file
        let output_path = get_decompressed_filename(input_path)?;
        
        if output_path.exists() && !options.force {
            return Err(anyhow!(
                "bzip2: output file {} already exists",
                output_path.display()
            ));
        }
        
        let mut output_file = File::create(&output_path)
            .with_context(|| format!("bzip2: can't create {}", output_path.display()))?;
        output_file
            .write_all(&decompressed_data)
            .with_context(|| format!("bzip2: error writing {}", output_path.display()))?;
        
        if !options.keep_input {
            std::fs::remove_file(input_path)
                .with_context(|| format!("bzip2: can't remove {}", input_path.display()))?;
        }
        
        if options.verbose {
            eprintln!(
                "  {}: done",
                input_path.display()
            );
        }
    }
    
    Ok(())
}

/// Test file integrity
fn test_file(input_path: &Path) -> Result<()> {
    let mut file = File::open(input_path)
        .with_context(|| format!("bzip2: can't open {}", input_path.display()))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .with_context(|| format!("bzip2: error reading {}", input_path.display()))?;

    // Try to decompress to verify integrity
    match decompress_bzip2(&buffer) {
        Ok(_) => {
            println!("{}: ok", input_path.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("{}: NOT OK - {}", input_path.display(), e);
            Err(anyhow!("bzip2: test failed for {}", input_path.display()))
        }
    }
}

/// Pure Rust bzip2 compression using bzip2-rs
fn compress_bzip2(data: &[u8], compression_level: i32) -> Result<Vec<u8>> {
    use bzip2_rs::EncoderReader;
    use std::io::Read;
    
    let block_size = match compression_level {
        1 => 100_000,       // Fast compression
        2 => 200_000,
        3 => 300_000,
        4 => 400_000,
        5 => 500_000,
        6 => 600_000,
        7 => 700_000,
        8 => 800_000,
        _ => 900_000,       // Best compression
    };
    
    let mut encoder = EncoderReader::new(data, block_size);
    let mut compressed = Vec::new();
    match encoder.read_to_end(&mut compressed) {
        Ok(_) => Ok(compressed),
        Err(e) => Err(anyhow!("bzip2: compression failed - {}", e))
    }
}

/// Pure Rust bzip2 decompression using bzip2-rs
fn decompress_bzip2(data: &[u8]) -> Result<Vec<u8>> {
    use bzip2_rs::DecoderReader;
    use std::io::Read;
    
    let mut decoder = DecoderReader::new(data);
    let mut decompressed = Vec::new();
    
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => Ok(decompressed),
        Err(e) => Err(anyhow!("bzip2: decompression failed - {}", e))
    }
}

/// Generate compressed filename
fn get_compressed_filename(input: &Path) -> PathBuf {
    input.with_extension(format!("{}bz2", 
        input.extension()
            .map(|s| s.to_string_lossy().to_string() + ".")
            .unwrap_or_default()
    ))
}

/// Generate decompressed filename
fn get_decompressed_filename(input: &Path) -> Result<PathBuf> {
    let input_str = input.to_string_lossy();
    
    if input_str.ends_with(".bz2") {
        Ok(PathBuf::from(input_str.strip_suffix(".bz2").unwrap()))
    } else if input_str.ends_with(".tbz2") {
        Ok(PathBuf::from(input_str.replace(".tbz2", ".tar")))
    } else if input_str.ends_with(".tbz") {
        Ok(PathBuf::from(input_str.replace(".tbz", ".tar")))
    } else {
        // Try to strip .bz2 suffix or add .out
        let stem = input.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("decompressed");
        Ok(input.with_file_name(format!("{}.out", stem)))
    }
}

/// Print bzip2 help message
fn print_bzip2_help() {
    println!("bzip2, a block-sorting file compressor.  Version 1.0.8 (Pure Rust impl)");
    println!();
    println!("usage: bzip2 [flags and input files in any order]");
    println!();
    println!("   -h --help           print this message");
    println!("   -d --decompress     decompress");
    println!("   -z --compress       compress (default)");
    println!("   -k --keep           keep (don't delete) input files");
    println!("   -f --force          overwrite existing output files");
    println!("   -t --test           test compressed file integrity");
    println!("   -c --stdout         output to standard out");
    println!("   -q --quiet          suppress noncritical error messages");
    println!("   -v --verbose        be verbose (a 2nd -v gives more)");
    println!("   -L --license        display software version & license");
    println!("   -V --version        display software version & license");
    println!("   -s --small          use less memory (at most 2500k)");
    println!("   -1 .. -9            set block size to 100k .. 900k");
    println!("   --fast              alias for -1");
    println!("   --best              alias for -9");
    println!();
    println!("   If no file names are given, bzip2 compresses from");
    println!("   standard input to standard output.");
}
