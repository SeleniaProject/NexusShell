use anyhow::Result;
use std::io::{self, Read};
use std::fs::File;

/// CLI wrapper function for od command (octal dump)
pub fn od_cli(args: &[String]) -> Result<()> {
    let mut format = "o"; // Default: octal
    let mut address_radix = "o"; // Default: octal addresses
    let mut bytes_per_line = 16;
    let mut files = Vec::new();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-t" | "--format" => {
                if i + 1 < args.len() {
                    format = &args[i + 1];
                    i += 1;
                }
            }
            "-A" | "--address-radix" => {
                if i + 1 < args.len() {
                    address_radix = &args[i + 1];
                    i += 1;
                }
            }
            "-w" | "--width" => {
                if i + 1 < args.len() {
                    bytes_per_line = args[i + 1].parse().unwrap_or(16);
                    i += 1;
                }
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
            "-h" | "--help" => {
                println!("od - dump files in octal and other formats");
                println!("Usage: od [OPTION]... [FILE]...");
                println!("  -t, --format=TYPE      select output format");
                println!("  -A, --address-radix=RADIX  how to print addresses");
                println!("  -w, --width=BYTES      output BYTES bytes per line");
                println!("  -x                     same as -t x2");
                println!("  -d                     same as -t u2");
                println!("  -o                     same as -t o2");
                println!("  -h, --help             display this help and exit");
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                files.push(arg.to_string());
            }
            _ => {
                eprintln!("od: unrecognized option '{}'", args[i]);
                return Err(anyhow::anyhow!("Invalid option"));
            }
        }
        i += 1;
    }
    
    if files.is_empty() {
        // Read from stdin
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        dump_data(&buffer, format, address_radix, bytes_per_line)?;
    } else {
        // Read from files
        for filename in files {
            let mut file = File::open(&filename)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            dump_data(&buffer, format, address_radix, bytes_per_line)?;
        }
    }
    
    Ok(())
}

fn dump_data(data: &[u8], format: &str, address_radix: &str, bytes_per_line: usize) -> Result<()> {
    for (offset, chunk) in data.chunks(bytes_per_line).enumerate() {
        let address = offset * bytes_per_line;
        
        // Print address
        match address_radix {
            "o" => print!("{address:07o} "),
            "x" => print!("{address:07x} "),
            "d" => print!("{address:07} "),
            "n" => print!(""), // No address
            _ => print!("{address:07o} "),
        }
        
        // Print data
        match format {
            "o" | "o2" => {
                for byte_pair in chunk.chunks(2) {
                    if byte_pair.len() == 2 {
                        let value = (byte_pair[0] as u16) | ((byte_pair[1] as u16) << 8);
                        print!("{value:06o} ");
                    } else {
                        print!("{:03o} ", byte_pair[0]);
                    }
                }
            }
            "x" | "x2" => {
                for byte_pair in chunk.chunks(2) {
                    if byte_pair.len() == 2 {
                        let value = (byte_pair[0] as u16) | ((byte_pair[1] as u16) << 8);
                        print!("{value:04x} ");
                    } else {
                        print!("{:02x} ", byte_pair[0]);
                    }
                }
            }
            "d" | "u2" => {
                for byte_pair in chunk.chunks(2) {
                    if byte_pair.len() == 2 {
                        let value = (byte_pair[0] as u16) | ((byte_pair[1] as u16) << 8);
                        print!("{value:5} ");
                    } else {
                        print!("{:3} ", byte_pair[0]);
                    }
                }
            }
            _ => {
                for byte in chunk {
                    print!("{byte:03o} ");
                }
            }
        }
        
        println!();
    }
    
    Ok(())
}
