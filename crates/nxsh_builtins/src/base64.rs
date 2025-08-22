use anyhow::Result;
use std::io::{self, Read, Write};
use std::fs::File;

/// CLI wrapper function for base64 encoding/decoding
pub fn base64_cli(args: &[String]) -> Result<()> {
    let mut decode = false;
    let mut ignore_garbage = false;
    let mut wrap_width = 76;
    let mut files = Vec::new();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--decode" => {
                decode = true;
            }
            "-i" | "--ignore-garbage" => {
                ignore_garbage = true;
            }
            "-w" | "--wrap" => {
                if i + 1 < args.len() {
                    wrap_width = args[i + 1].parse().unwrap_or(76);
                    i += 1;
                }
            }
            "-h" | "--help" => {
                println!("base64 - encode/decode data and print to standard output");
                println!("Usage: base64 [OPTION]... [FILE]");
                println!("  -d, --decode          decode data");
                println!("  -i, --ignore-garbage  ignore non-alphabet characters");
                println!("  -w, --wrap=COLS       wrap encoded lines after COLS characters");
                println!("  -h, --help            display this help and exit");
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                files.push(arg.to_string());
            }
            _ => {
                eprintln!("base64: unrecognized option '{}'", args[i]);
                return Err(anyhow::anyhow!("Invalid option"));
            }
        }
        i += 1;
    }
    
    if files.is_empty() {
        // Read from stdin
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        
        if decode {
            decode_base64(&buffer, ignore_garbage)?;
        } else {
            encode_base64(&buffer, wrap_width)?;
        }
    } else {
        // Read from files
        for filename in files {
            let mut file = File::open(&filename)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            
            if decode {
                decode_base64(&buffer, ignore_garbage)?;
            } else {
                encode_base64(&buffer, wrap_width)?;
            }
        }
    }
    
    Ok(())
}

fn encode_base64(data: &[u8], wrap_width: usize) -> Result<()> {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    let mut result = String::new();
    
    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }
        
        let b = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
        
        result.push(BASE64_CHARS[((b >> 18) & 0x3F) as usize] as char);
        result.push(BASE64_CHARS[((b >> 12) & 0x3F) as usize] as char);
        
        if chunk.len() > 1 {
            result.push(BASE64_CHARS[((b >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        
        if chunk.len() > 2 {
            result.push(BASE64_CHARS[(b & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    
    if wrap_width > 0 {
        for (i, chunk) in result.chars().collect::<Vec<_>>().chunks(wrap_width).enumerate() {
            if i > 0 {
                println!();
            }
            print!("{}", chunk.iter().collect::<String>());
        }
        println!();
    } else {
        println!("{result}");
    }
    
    Ok(())
}

fn decode_base64(data: &[u8], ignore_garbage: bool) -> Result<()> {
    let input = String::from_utf8_lossy(data);
    let cleaned: String = if ignore_garbage {
        input.chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '+' || *c == '/' || *c == '=')
            .collect()
    } else {
        input.chars()
            .filter(|c| !c.is_whitespace())
            .collect()
    };
    
    let mut result = Vec::new();
    
    for chunk in cleaned.chars().collect::<Vec<_>>().chunks(4) {
        if chunk.len() < 4 {
            continue;
        }
        
        let mut values = [0u8; 4];
        for (i, &c) in chunk.iter().enumerate() {
            values[i] = match c {
                'A'..='Z' => (c as u8) - b'A',
                'a'..='z' => (c as u8) - b'a' + 26,
                '0'..='9' => (c as u8) - b'0' + 52,
                '+' => 62,
                '/' => 63,
                '=' => 0,
                _ => {
                    if !ignore_garbage {
                        return Err(anyhow::anyhow!("Invalid character in base64 input"));
                    }
                    0
                }
            };
        }
        
        let b = ((values[0] as u32) << 18) | 
                ((values[1] as u32) << 12) | 
                ((values[2] as u32) << 6) | 
                (values[3] as u32);
        
        result.push((b >> 16) as u8);
        
        if chunk[2] != '=' {
            result.push((b >> 8) as u8);
        }
        
        if chunk[3] != '=' {
            result.push(b as u8);
        }
    }
    
    io::stdout().write_all(&result)?;
    Ok(())
}

