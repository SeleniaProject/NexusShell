use anyhow::Result;
use std::fs::File;
use std::io::{self, Read, BufReader};
use crc32fast::Hasher as Crc32;

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
        // Stream from stdin
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        let (checksum, size) = compute_checksum_stream(&mut reader, algorithm)?;
        println!("{checksum} {size} -");
    } else {
        // Stream from files
        for filename in &files {
            let file = File::open(filename)?;
            let mut reader = BufReader::new(file);
            let (checksum, size) = compute_checksum_stream(&mut reader, algorithm)?;
            println!("{checksum} {size} {filename}");
        }
    }
    
    Ok(())
}

fn compute_checksum_stream<R: Read>(reader: &mut R, algorithm: &str) -> Result<(String, usize)> {
    match algorithm {
        "crc32" => {
            // Fast CRC32 with streaming; matches common IEEE CRC32 (zlib) used widely.
            let mut hasher = Crc32::new();
            let mut size: usize = 0;
            let mut buf = [0u8; 64 * 1024];
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 { break; }
                hasher.update(&buf[..n]);
                size += n;
            }
            let crc = hasher.finalize();
            Ok((format!("{}", crc), size))
        }
        "md5" => {
            let mut hasher = md5::Context::new();
            let mut size: usize = 0;
            let mut buf = [0u8; 64 * 1024];
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 { break; }
                hasher.consume(&buf[..n]);
                size += n;
            }
            let digest = hasher.compute();
            Ok((format!("{:x}", digest), size))
        }
        "sha1" => {
            use sha1::{Digest, Sha1};
            let mut hasher = Sha1::new();
            let mut size: usize = 0;
            let mut buf = [0u8; 64 * 1024];
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 { break; }
                hasher.update(&buf[..n]);
                size += n;
            }
            let out = hasher.finalize();
            Ok((format!("{:x}", out), size))
        }
        "sha256" => {
            use sha2::digest::Digest;
            let mut hasher = sha2::Sha256::new();
            let mut size: usize = 0;
            let mut buf = [0u8; 64 * 1024];
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 { break; }
                hasher.update(&buf[..n]);
                size += n;
            }
            let out = hasher.finalize();
            Ok((format!("{:x}", out), size))
        }
        _ => Err(anyhow::anyhow!("Unsupported algorithm: {}", algorithm)),
    }
}
