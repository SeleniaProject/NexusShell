//! Pure Rust tar implementation
//! Simplified version using only Pure Rust libraries

use anyhow::{anyhow, Result};
use std::{
    fs::File,
    io::{Read, Write, BufReader, BufWriter, Seek, SeekFrom},
    path::{Path, PathBuf},
};

/// tar command implementation with Pure Rust only
pub fn tar_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        print_tar_help();
        return Ok(());
    }

    let options = parse_tar_args(args)?;
    
    match options.mode {
        TarMode::Create => create_archive(&options),
        TarMode::Extract => extract_archive(&options),
        TarMode::List => list_archive(&options),
    }
}

#[derive(Debug, Clone)]
enum TarMode {
    Create,
    Extract,
    List,
}

#[derive(Debug, Clone)]
enum Compression {
    Gzip,
    None,
}

#[derive(Debug)]
struct TarOptions {
    mode: TarMode,
    files: Vec<PathBuf>,
    archive_file: Option<PathBuf>,
    compression: Compression,
    verbose: bool,
    keep_input: bool,
    force: bool,
    stdout: bool,
}

fn parse_tar_args(args: &[String]) -> Result<TarOptions> {
    let mut options = TarOptions {
        mode: TarMode::List,
        files: Vec::new(),
        archive_file: None,
        compression: Compression::None,
        verbose: false,
        keep_input: false,
        force: false,
        stdout: false,
    };

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        
        if arg.starts_with('-') {
            match arg.as_str() {
                "--help" | "-h" => {
                    print_tar_help();
                    std::process::exit(0);
                }
                "--create" | "-c" => options.mode = TarMode::Create,
                "--extract" | "-x" => options.mode = TarMode::Extract,
                "--list" | "-t" => options.mode = TarMode::List,
                "--verbose" | "-v" => options.verbose = true,
                "--file" | "-f" => {
                    i += 1;
                    if i < args.len() {
                        options.archive_file = Some(PathBuf::from(&args[i]));
                    }
                }
                "--gzip" | "-z" => options.compression = Compression::Gzip,
                _ => {
                    // Handle combined flags like -czf
                    for c in arg.chars().skip(1) {
                        match c {
                            'c' => options.mode = TarMode::Create,
                            'x' => options.mode = TarMode::Extract,
                            't' => options.mode = TarMode::List,
                            'v' => options.verbose = true,
                            'z' => options.compression = Compression::Gzip,
                            'f' => {
                                i += 1;
                                if i < args.len() {
                                    options.archive_file = Some(PathBuf::from(&args[i]));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        } else {
            options.files.push(PathBuf::from(arg));
        }
        i += 1;
    }

    Ok(options)
}

fn create_archive(options: &TarOptions) -> Result<()> {
    let archive_path = options.archive_file.as_ref()
        .ok_or_else(|| anyhow!("Archive file not specified"))?;
    
    let mut output = BufWriter::new(File::create(archive_path)?);
    
    // Simple tar creation - write files sequentially
    for file_path in &options.files {
        if file_path.is_file() {
            write_file_to_archive(&mut output, file_path, options)?;
        } else if file_path.is_dir() {
            write_directory_to_archive(&mut output, file_path, options)?;
        }
    }
    
    // Write end-of-archive marker (512 zero bytes)
    output.write_all(&[0u8; 512])?;
    output.flush()?;
    
    if options.verbose {
        println!("Archive created: {}", archive_path.display());
    }
    
    Ok(())
}

fn extract_archive(options: &TarOptions) -> Result<()> {
    let archive_path = options.archive_file.as_ref()
        .ok_or_else(|| anyhow!("Archive file not specified"))?;
    
    let mut input = BufReader::new(File::open(archive_path)?);
    
    loop {
        let mut header = [0u8; 512];
        let bytes_read = input.read(&mut header)?;
        
        if bytes_read == 0 || header.iter().all(|&b| b == 0) {
            break; // End of archive
        }
        
        // Parse header (simplified)
        let filename = extract_filename(&header)?;
        let size = extract_file_size(&header)?;
        
        if options.verbose {
            println!("Extracting: {filename}");
        }
        
        // Read file data
        let mut file_data = vec![0u8; size];
        input.read_exact(&mut file_data)?;
        
        // Write to file
        let output_path = PathBuf::from(&filename);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let mut output_file = File::create(&output_path)?;
        output_file.write_all(&file_data)?;
        
        // Skip padding to 512-byte boundary
        let padding = (512 - (size % 512)) % 512;
        if padding > 0 {
            let mut pad_buf = vec![0u8; padding];
            input.read_exact(&mut pad_buf)?;
        }
    }
    
    if options.verbose {
        println!("Extraction complete");
    }
    
    Ok(())
}

fn list_archive(options: &TarOptions) -> Result<()> {
    let archive_path = options.archive_file.as_ref()
        .ok_or_else(|| anyhow!("Archive file not specified"))?;
    
    let mut input = BufReader::new(File::open(archive_path)?);
    
    loop {
        let mut header = [0u8; 512];
        let bytes_read = input.read(&mut header)?;
        
        if bytes_read == 0 || header.iter().all(|&b| b == 0) {
            break;
        }
        
        let filename = extract_filename(&header)?;
        let size = extract_file_size(&header)?;
        
        if options.verbose {
            println!("{size:>10} {filename}");
        } else {
            println!("{filename}");
        }
        
        // Skip file data and padding
        let total_size = size.div_ceil(512) * 512;
        input.seek(SeekFrom::Current(total_size as i64))?;
    }
    
    Ok(())
}

fn write_file_to_archive<W: Write>(writer: &mut W, file_path: &Path, options: &TarOptions) -> Result<()> {
    let mut file = File::open(file_path)?;
    let metadata = file.metadata()?;
    let size = metadata.len();
    
    // Create tar header (simplified)
    let mut header = [0u8; 512];
    let filename = file_path.to_string_lossy();
    
    // Write filename (up to 100 bytes)
    let name_bytes = filename.as_bytes();
    let copy_len = std::cmp::min(name_bytes.len(), 100);
    header[0..copy_len].copy_from_slice(&name_bytes[0..copy_len]);
    
    // Write file size as octal string
    let size_str = format!("{size:011o}");
    header[124..135].copy_from_slice(size_str.as_bytes());
    
    // Write file mode (644)
    header[100..108].copy_from_slice(b"0000644");
    
    // Calculate and write checksum
    let checksum = calculate_checksum(&header);
    let checksum_str = format!("{checksum:06o}\0 ");
    header[148..156].copy_from_slice(checksum_str.as_bytes());
    
    writer.write_all(&header)?;
    
    // Write file data
    let mut buffer = vec![0u8; size as usize];
    file.read_exact(&mut buffer)?;
    writer.write_all(&buffer)?;
    
    // Write padding to 512-byte boundary
    let padding = (512 - (size % 512)) % 512;
    if padding > 0 {
        writer.write_all(&vec![0u8; padding as usize])?;
    }
    
    if options.verbose {
        println!("Added: {filename}");
    }
    
    Ok(())
}

fn write_directory_to_archive<W: Write>(writer: &mut W, dir_path: &Path, options: &TarOptions) -> Result<()> {
    for entry in std::fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            write_file_to_archive(writer, &path, options)?;
        } else if path.is_dir() {
            write_directory_to_archive(writer, &path, options)?;
        }
    }
    Ok(())
}

fn extract_filename(header: &[u8]) -> Result<String> {
    let name_end = header[0..100].iter().position(|&b| b == 0).unwrap_or(100);
    String::from_utf8(header[0..name_end].to_vec())
        .map_err(|e| anyhow!("Invalid filename encoding: {}", e))
}

fn extract_file_size(header: &[u8]) -> Result<usize> {
    let size_str = std::str::from_utf8(&header[124..135])
        .map_err(|e| anyhow!("Invalid size field: {}", e))?;
    
    usize::from_str_radix(size_str.trim_end_matches('\0'), 8)
        .map_err(|e| anyhow!("Invalid octal size: {}", e))
}

fn calculate_checksum(header: &[u8]) -> u32 {
    let mut sum = 0u32;
    for (i, &byte) in header.iter().enumerate() {
        if (148..156).contains(&i) {
            sum += b' ' as u32; // Checksum field treated as spaces
        } else {
            sum += byte as u32;
        }
    }
    sum
}

fn print_tar_help() {
    println!("Usage: tar [OPTION...] [FILE]...");
    println!("Pure Rust tar implementation");
    println!();
    println!("Main operation modes:");
    println!("  -c, --create     Create a new archive");
    println!("  -x, --extract    Extract files from an archive");
    println!("  -t, --list       List the contents of an archive");
    println!();
    println!("Options:");
    println!("  -f, --file FILE  Use archive file FILE");
    println!("  -v, --verbose    Verbose output");
    println!("  -z, --gzip       Compress with gzip");
    println!("  -h, --help       Display this help");
    println!();
    println!("Examples:");
    println!("  tar -czf archive.tar.gz files/  # Create compressed archive");
    println!("  tar -xzf archive.tar.gz         # Extract compressed archive");
    println!("  tar -tf archive.tar             # List archive contents");
}
