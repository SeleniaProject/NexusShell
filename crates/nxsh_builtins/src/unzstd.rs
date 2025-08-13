//! `unzstd` builtin - Zstandard decompression utility with Pure Rust implementation
//!
//! Complete Pure Rust implementation using the `ruzstd` crate for streaming
//! Zstandard frame decoding without any C/C++ dependencies.

use anyhow::{anyhow, Context, Result};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use ruzstd::streaming_decoder::StreamingDecoder;

/// unzstd command implementation with Pure Rust decompression
pub fn unzstd_cli(args: &[String]) -> Result<()> {
    let options = parse_unzstd_args(args)?;
    
    for input_file in &options.files {
        decompress_file(input_file, &options)?;
    }
    
    Ok(())
}

#[derive(Debug)]
struct UnzstdOptions {
    files: Vec<PathBuf>,
    keep_input: bool,
    force: bool,
    stdout: bool,
    verbose: bool,
    quiet: bool,
    test: bool,
}

/// Parse unzstd command line arguments
fn parse_unzstd_args(args: &[String]) -> Result<UnzstdOptions> {
    let mut options = UnzstdOptions {
        files: Vec::new(),
        keep_input: false,
        force: false,
        stdout: false,
        verbose: false,
        quiet: false,
        test: false,
    };

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        
        if arg.starts_with('-') && arg.len() > 1 {
            match arg.as_str() {
                "--help" => {
                    print_unzstd_help();
                    std::process::exit(0);
                }
                "--version" => {
                    println!("unzstd 1.5.5 (Pure Rust implementation)");
                    std::process::exit(0);
                }
                "--keep" => options.keep_input = true,
                "--force" => options.force = true,
                "--stdout" => options.stdout = true,
                "--verbose" => options.verbose = true,
                "--quiet" => options.quiet = true,
                "--test" => options.test = true,
                _ => {
                    // Handle single-character flags
                    for ch in arg[1..].chars() {
                        match ch {
                            'k' => options.keep_input = true,
                            'f' => options.force = true,
                            'c' => options.stdout = true,
                            't' => options.test = true,
                            'v' => options.verbose = true,
                            'q' => options.quiet = true,
                            'h' => {
                                print_unzstd_help();
                                std::process::exit(0);
                            }
                            _ => return Err(anyhow!("unzstd: invalid option '{}'", ch)),
                        }
                    }
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

/// Decompress a file using Pure Rust zstd implementation
fn decompress_file(input_path: &Path, options: &UnzstdOptions) -> Result<()> {
    let input_data = if input_path.to_str() == Some("-") {
        // Read from stdin
        let mut buffer = Vec::new();
        std::io::stdin()
            .read_to_end(&mut buffer)
            .context("unzstd: failed to read from stdin")?;
        buffer
    } else {
        // Read from file
        if !input_path.exists() {
            return Err(anyhow!("unzstd: {}: No such file or directory", input_path.display()));
        }
        
        // Verify file has appropriate extension
        if !options.force && input_path.to_str() != Some("-") {
            let extension = input_path.extension().and_then(|s| s.to_str());
            if !matches!(extension, Some("zst") | Some("zstd")) {
                return Err(anyhow!(
                    "unzstd: {}: doesn't end in .zst -- ignored",
                    input_path.display()
                ));
            }
        }
        
        let mut file = File::open(input_path)
            .with_context(|| format!("unzstd: {}: Permission denied", input_path.display()))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .with_context(|| format!("unzstd: {}: Read error", input_path.display()))?;
        buffer
    };

    // Test mode - just verify the file can be decompressed
    if options.test {
        return test_zstd_file(input_path, &input_data);
    }

    if options.verbose && !options.quiet {
        eprintln!("unzstd: decompressing {}", input_path.display());
    }

    // Decompress the data
    let decompressed_data = decompress_zstd_data(&input_data)
        .with_context(|| format!("unzstd: {}: not in zstd format", input_path.display()))?;
    
    if options.stdout || input_path.to_str() == Some("-") {
        // Write to stdout
        std::io::stdout()
            .write_all(&decompressed_data)
            .context("unzstd: Write error")?;
    } else {
        // Write to decompressed file
        let output_path = get_decompressed_filename(input_path)?;
        
        if output_path.exists() && !options.force {
            return Err(anyhow!(
                "unzstd: {}: File exists",
                output_path.display()
            ));
        }
        
        let mut output_file = File::create(&output_path)
            .with_context(|| format!("unzstd: {}: Permission denied", output_path.display()))?;
        output_file
            .write_all(&decompressed_data)
            .with_context(|| format!("unzstd: {}: Write error", output_path.display()))?;
        
        if !options.keep_input {
            std::fs::remove_file(input_path)
                .with_context(|| format!("unzstd: {}: Permission denied", input_path.display()))?;
        }
        
        if options.verbose && !options.quiet {
            eprintln!(
                "  {}: decompressed successfully",
                output_path.display()
            );
        }
    }
    
    Ok(())
}

/// Test a zstd file for integrity
fn test_zstd_file(input_path: &Path, input_data: &[u8]) -> Result<()> {
    match decompress_zstd_data(input_data) {
        Ok(_) => {
            if input_path.to_str() != Some("-") {
                println!("{}: OK", input_path.display());
            } else {
                println!("stdin: OK");
            }
            Ok(())
        }
        Err(e) => {
            if input_path.to_str() != Some("-") {
                println!("{}: FAILED", input_path.display());
                eprintln!("unzstd: {}: {}", input_path.display(), e);
            } else {
                println!("stdin: FAILED");
                eprintln!("unzstd: stdin: {e}");
            }
            Err(anyhow!("unzstd: test failed"))
        }
    }
}

/// Pure Rust zstd decompression using ruzstd streaming decoder
fn decompress_zstd_data(data: &[u8]) -> Result<Vec<u8>> {
    // Create a streaming decoder over the input bytes. ruzstd validates the
    // frame header and will return a descriptive error if the data is not
    // in Zstandard format or is corrupted.
    let mut decoder = StreamingDecoder::new(data)
        .context("unzstd: failed to initialize zstd decoder")?;

    // Decompress fully into memory. For large files, a streaming path to a
    // file handle would be preferable; this variant matches current CLI flow.
    let mut output = Vec::new();
    decoder
        .read_to_end(&mut output)
        .context("unzstd: decompression error")?;

    Ok(output)
}

/// Generate decompressed filename from compressed filename
fn get_decompressed_filename(input: &Path) -> Result<PathBuf> {
    let input_str = input.to_string_lossy();
    
    if input_str.ends_with(".zst") {
        Ok(PathBuf::from(input_str.strip_suffix(".zst").unwrap()))
    } else if input_str.ends_with(".zstd") {
        Ok(PathBuf::from(input_str.strip_suffix(".zstd").unwrap()))
    } else {
        // If no recognized extension, add .out
        let stem = input.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("decompressed");
        Ok(input.with_file_name(format!("{stem}.out")))
    }
}

/// Print unzstd help message
fn print_unzstd_help() {
    println!("Usage: unzstd [OPTION]... [FILE]...");
    println!("Decompress files in the .zst/.zstd format to the original format.");
    println!();
    println!("  -k, --keep          keep (don't delete) input files");
    println!("  -f, --force         force overwrite of output file");
    println!("  -c, --stdout        write to standard output and don't delete input files");
    println!("  -t, --test          test compressed file integrity");
    println!("  -q, --quiet         suppress warnings; specify twice to suppress errors too");
    println!("  -v, --verbose       be verbose");
    println!("  -h, --help          display this short help and exit");
    println!("  -V, --version       display the version number and exit");
    println!();
    println!("With no FILE, or when FILE is -, read standard input.");
    println!();
    println!("unzstd will only attempt to decompress files ending in");
    println!("'.zst' or '.zstd'. Use -f to override this.");
    println!();
    println!("Report bugs at: https://github.com/facebook/zstd/issues");
}
