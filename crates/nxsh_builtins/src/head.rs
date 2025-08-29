use crate::common::{BuiltinContext, BuiltinResult};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// Display the first part of files
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    let mut line_count = 10i64;
    let mut byte_count: Option<u64> = None;
    let mut quiet = false;
    let mut verbose = false;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--lines" => {
                if i + 1 >= args.len() {
                    eprintln!("head: option '{}' requires an argument", args[i]);
                    return Ok(1);
                }
                i += 1;
                match args[i].parse::<i64>() {
                    Ok(n) => line_count = n,
                    Err(_) => {
                        eprintln!("head: invalid number of lines: '{}'", args[i]);
                        return Ok(1);
                    }
                }
            }
            "-c" | "--bytes" => {
                if i + 1 >= args.len() {
                    eprintln!("head: option '{}' requires an argument", args[i]);
                    return Ok(1);
                }
                i += 1;
                match args[i].parse::<u64>() {
                    Ok(n) => byte_count = Some(n),
                    Err(_) => {
                        eprintln!("head: invalid number of bytes: '{}'", args[i]);
                        return Ok(1);
                    }
                }
            }
            "-q" | "--quiet" | "--silent" => quiet = true,
            "-v" | "--verbose" => verbose = true,
            "-h" | "--help" => {
                print_help();
                return Ok(0);
            }
            arg if arg.starts_with("-n") => {
                let num_str = &arg[2..];
                match num_str.parse::<i64>() {
                    Ok(n) => line_count = n,
                    Err(_) => {
                        eprintln!("head: invalid number of lines: '{num_str}'");
                        return Ok(1);
                    }
                }
            }
            arg if arg.starts_with("-c") => {
                let num_str = &arg[2..];
                match num_str.parse::<u64>() {
                    Ok(n) => byte_count = Some(n),
                    Err(_) => {
                        eprintln!("head: invalid number of bytes: '{num_str}'");
                        return Ok(1);
                    }
                }
            }
            arg if arg.starts_with('-') => {
                eprintln!("head: invalid option '{arg}'");
                return Ok(1);
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    if files.is_empty() {
        files.push("-".to_string()); // stdin
    }

    let multiple_files = files.len() > 1;
    let mut exit_code = 0;

    for (index, filename) in files.iter().enumerate() {
        if multiple_files && (verbose || !quiet) {
            if index > 0 {
                println!();
            }
            println!(
                "==> {} <==",
                if filename == "-" {
                    "standard input"
                } else {
                    filename
                }
            );
        }

        let result = if filename == "-" {
            read_from_stdin(line_count, byte_count)
        } else {
            read_from_file(filename, line_count, byte_count)
        };

        if let Err(e) = result {
            eprintln!("head: {filename}: {e}");
            exit_code = 1;
        }
    }

    Ok(exit_code)
}

fn read_from_file(
    filename: &str,
    line_count: i64,
    byte_count: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(filename).exists() {
        return Err("No such file or directory".to_string().into());
    }

    let file = File::open(filename)?;

    if let Some(bytes) = byte_count {
        read_bytes(Box::new(file), bytes)?;
    } else {
        let reader = BufReader::new(file);
        read_lines(reader, line_count)?;
    }

    Ok(())
}

fn read_from_stdin(
    line_count: i64,
    byte_count: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = std::io::stdin();

    if let Some(bytes) = byte_count {
        read_bytes(Box::new(stdin.lock()), bytes)?;
    } else {
        read_lines(stdin.lock(), line_count)?;
    }

    Ok(())
}

fn read_lines<R: BufRead>(reader: R, line_count: i64) -> Result<(), Box<dyn std::error::Error>> {
    if line_count <= 0 {
        return Ok(());
    }

    let mut count = 0;
    #[allow(clippy::explicit_counter_loop)]
    for line in reader.lines() {
        if count >= line_count {
            break;
        }
        println!("{}", line?);
        count += 1;
    }

    Ok(())
}

fn read_bytes<R: Read>(mut reader: R, byte_count: u64) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = vec![0; std::cmp::min(byte_count, 8192) as usize];
    let mut total_read = 0u64;

    while total_read < byte_count {
        let to_read = std::cmp::min(buffer.len() as u64, byte_count - total_read) as usize;
        let bytes_read = reader.read(&mut buffer[..to_read])?;

        if bytes_read == 0 {
            break; // EOF
        }

        std::io::Write::write_all(&mut std::io::stdout(), &buffer[..bytes_read])?;
        total_read += bytes_read as u64;
    }

    Ok(())
}

fn print_help() {
    println!("Usage: head [OPTION]... [FILE]...");
    println!("Print the first 10 lines of each FILE to standard output.");
    println!("With more than one FILE, precede each with a header giving the file name.");
    println!();
    println!("Options:");
    println!("  -c, --bytes=NUM      print the first NUM bytes of each file");
    println!("  -n, --lines=NUM      print the first NUM lines instead of the first 10");
    println!("  -q, --quiet, --silent never print headers giving file names");
    println!("  -v, --verbose        always print headers giving file names");
    println!("  -h, --help           display this help and exit");
    println!();
    println!("NUM may have a multiplier suffix:");
    println!("b 512, kB 1000, K 1024, MB 1000*1000, M 1024*1024, and so on.");
    println!();
    println!("Examples:");
    println!("  head file.txt        Show first 10 lines of file.txt");
    println!("  head -n 5 file.txt   Show first 5 lines of file.txt");
    println!("  head -c 100 file.txt Show first 100 bytes of file.txt");
}
