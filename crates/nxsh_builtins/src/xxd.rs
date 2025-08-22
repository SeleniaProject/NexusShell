use anyhow::Result;
use std::io::{self, Read};
use std::fs::File;

/// CLI wrapper function for xxd command (hex dump)
pub fn xxd_cli(args: &[String]) -> Result<()> {
    let mut cols = 16; // Columns per line
    let mut plain = false;
    let mut uppercase = false;
    let mut files = Vec::new();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--cols" => {
                if i + 1 < args.len() {
                    cols = args[i + 1].parse().unwrap_or(16);
                    i += 1;
                }
            }
            "-p" | "--ps" | "--postscript" | "--plain" => {
                plain = true;
            }
            "-u" | "--upper" => {
                uppercase = true;
            }
            "-h" | "--help" => {
                println!("xxd - make a hexdump or do the reverse");
                println!("Usage: xxd [OPTION]... [FILE]...");
                println!("  -c cols        format <cols> octets per line");
                println!("  -p             output in postscript plain hexdump style");
                println!("  -u             use upper case hex letters");
                println!("  -h, --help     display this help and exit");
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                files.push(arg.to_string());
            }
            _ => {
                eprintln!("xxd: unrecognized option '{}'", args[i]);
                return Err(anyhow::anyhow!("Invalid option"));
            }
        }
        i += 1;
    }
    
    if files.is_empty() {
        // Read from stdin
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        hex_dump(&buffer, cols, plain, uppercase)?;
    } else {
        // Read from files
        for filename in files {
            let mut file = File::open(&filename)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            hex_dump(&buffer, cols, plain, uppercase)?;
        }
    }
    
    Ok(())
}

fn hex_dump(data: &[u8], cols: usize, plain: bool, uppercase: bool) -> Result<()> {
    if plain {
        // Plain hex output
        for byte in data {
            if uppercase {
                print!("{byte:02X}");
            } else {
                print!("{byte:02x}");
            }
        }
        println!();
    } else {
        // Standard xxd format
        for (offset, chunk) in data.chunks(cols).enumerate() {
            let address = offset * cols;
            
            // Print address
            print!("{address:08x}: ");
            
            // Print hex bytes
            for (i, byte) in chunk.iter().enumerate() {
                if uppercase {
                    print!("{byte:02X}");
                } else {
                    print!("{byte:02x}");
                }
                
                if i % 2 == 1 {
                    print!(" ");
                }
            }
            
            // Pad incomplete lines
            let remaining = cols - chunk.len();
            for i in 0..remaining {
                print!("  ");
                if (chunk.len() + i) % 2 == 1 {
                    print!(" ");
                }
            }
            
            // Print ASCII representation
            print!(" ");
            for byte in chunk {
                if byte.is_ascii_graphic() || *byte == b' ' {
                    print!("{}", *byte as char);
                } else {
                    print!(".");
                }
            }
            
            println!();
        }
    }
    
    Ok(())
}

