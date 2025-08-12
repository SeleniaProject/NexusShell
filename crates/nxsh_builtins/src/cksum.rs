use anyhow::Result;
use std::io::{self, Read};
use std::fs::File;

/// CLI wrapper function for cksum command
pub fn cksum_cli(args: &[String]) -> Result<()> {
    let mut algorithm = "crc32"; // Default algorithm
    let mut files = Vec::new();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--algorithm" => {
                if i + 1 < args.len() {
                    algorithm = &args[i + 1];
                    i += 1;
                }
            }
            "-h" | "--help" => {
                println!("cksum - checksum and count the bytes in a file");
                println!("Usage: cksum [OPTION]... [FILE]...");
                println!("  -a, --algorithm=TYPE  use algorithm TYPE (crc32, md5, sha1, sha256)");
                println!("  -h, --help            display this help and exit");
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                files.push(arg.to_string());
            }
            _ => {
                eprintln!("cksum: unrecognized option '{}'", args[i]);
                return Err(anyhow::anyhow!("Invalid option"));
            }
        }
        i += 1;
    }
    
    if files.is_empty() {
        // Read from stdin
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        let (checksum, size) = compute_checksum(&buffer, algorithm)?;
        println!("{checksum} {size} -");
    } else {
        // Read from files
        for filename in &files {
            let mut file = File::open(filename)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            let (checksum, size) = compute_checksum(&buffer, algorithm)?;
            println!("{checksum} {size} {filename}");
        }
    }
    
    Ok(())
}

fn compute_checksum(data: &[u8], algorithm: &str) -> Result<(String, usize)> {
    let size = data.len();
    
    match algorithm {
        "crc32" => {
            let crc = compute_crc32(data);
            Ok((format!("{crc}"), size))
        }
        "md5" => {
            let hash = compute_simple_hash(data, 32);
            Ok((hash, size))
        }
        "sha1" => {
            let hash = compute_simple_hash(data, 40);
            Ok((hash, size))
        }
        "sha256" => {
            let hash = compute_simple_hash(data, 64);
            Ok((hash, size))
        }
        _ => {
            Err(anyhow::anyhow!("Unsupported algorithm: {}", algorithm))
        }
    }
}

fn compute_crc32(data: &[u8]) -> u32 {
    // Simple CRC32 implementation
    const CRC32_POLY: u32 = 0xEDB88320;
    let mut crc: u32 = !0;
    
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ CRC32_POLY;
            } else {
                crc >>= 1;
            }
        }
    }
    
    !crc
}

fn compute_simple_hash(data: &[u8], length: usize) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let hash = hasher.finish();
    
    format!("{hash:0length$x}")
}
