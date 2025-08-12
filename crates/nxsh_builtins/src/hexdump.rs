use anyhow::Result;
use std::io::{self, Read};
use std::fs::File;

/// CLI wrapper function for hexdump command
pub fn hexdump_cli(args: &[String]) -> Result<()> {
    let mut format = "x"; // Default: hex
    let mut canonical = false;
    let bytes_per_line = 16;
    let mut files = Vec::new();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-C" | "--canonical" => {
                canonical = true;
            }
            "-x" => {
                format = "x";
            }
            "-d" => {
                format = "d";
            }
            "-o" => {
                format = "o";
            }
            "-n" | "--length" => {
                if i + 1 < args.len() {
                    // Skip length for now
                    i += 1;
                }
            }
            "-h" | "--help" => {
                println!("hexdump - display file contents in hexadecimal, decimal, octal, or ascii");
                println!("Usage: hexdump [OPTION]... [FILE]...");
                println!("  -C, --canonical    canonical hex+ASCII display");
                println!("  -x                 two-byte hexadecimal display");
                println!("  -d                 two-byte decimal display");
                println!("  -o                 two-byte octal display");
                println!("  -n, --length=N     only format the first N bytes");
                println!("  -h, --help         display this help and exit");
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                files.push(arg.to_string());
            }
            _ => {
                eprintln!("hexdump: unrecognized option '{}'", args[i]);
                return Err(anyhow::anyhow!("Invalid option"));
            }
        }
        i += 1;
    }
    
    if files.is_empty() {
        // Read from stdin
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        hex_dump(&buffer, format, canonical, bytes_per_line)?;
    } else {
        // Read from files
        for filename in files {
            let mut file = File::open(&filename)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            hex_dump(&buffer, format, canonical, bytes_per_line)?;
        }
    }
    
    Ok(())
}

fn hex_dump(data: &[u8], format: &str, canonical: bool, bytes_per_line: usize) -> Result<()> {
    if canonical {
        // Canonical format (similar to xxd)
        for (offset, chunk) in data.chunks(bytes_per_line).enumerate() {
            let address = offset * bytes_per_line;
            
            print!("{address:08x}  ");
            
            // Print hex bytes
            for (i, byte) in chunk.iter().enumerate() {
                print!("{byte:02x}");
                if i % 2 == 1 {
                    print!(" ");
                }
            }
            
            // Pad incomplete lines
            let remaining = bytes_per_line - chunk.len();
            for i in 0..remaining {
                print!("  ");
                if (chunk.len() + i) % 2 == 1 {
                    print!(" ");
                }
            }
            
            // Print ASCII representation
            print!(" |");
            for byte in chunk {
                if byte.is_ascii_graphic() || *byte == b' ' {
                    print!("{}", *byte as char);
                } else {
                    print!(".");
                }
            }
            println!("|");
        }
        
        // Print final address
        println!("{:08x}", data.len());
    } else {
        // Standard format
        for (offset, chunk) in data.chunks(bytes_per_line).enumerate() {
            let address = offset * bytes_per_line;
            print!("{address:07x} ");
            
            match format {
                "x" => {
                    for byte_pair in chunk.chunks(2) {
                        if byte_pair.len() == 2 {
                            let value = (byte_pair[1] as u16) << 8 | byte_pair[0] as u16;
                            print!("{value:04x} ");
                        } else {
                            print!("{:02x}   ", byte_pair[0]);
                        }
                    }
                }
                "d" => {
                    for byte_pair in chunk.chunks(2) {
                        if byte_pair.len() == 2 {
                            let value = (byte_pair[1] as u16) << 8 | byte_pair[0] as u16;
                            print!("{value:5} ");
                        } else {
                            print!("{:3}   ", byte_pair[0]);
                        }
                    }
                }
                "o" => {
                    for byte_pair in chunk.chunks(2) {
                        if byte_pair.len() == 2 {
                            let value = (byte_pair[1] as u16) << 8 | byte_pair[0] as u16;
                            print!("{value:06o} ");
                        } else {
                            print!("{:03o}   ", byte_pair[0]);
                        }
                    }
                }
                _ => {
                    for byte in chunk {
                        print!("{byte:02x} ");
                    }
                }
            }
            
            println!();
        }
    }
    
    Ok(())
}
