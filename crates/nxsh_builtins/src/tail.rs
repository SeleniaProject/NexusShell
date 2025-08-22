use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use crate::common::{BuiltinResult, BuiltinContext};

/// Display the last part of files
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    let mut line_count = 10i64;
    let mut byte_count: Option<u64> = None;
    let mut follow = false;
    let mut quiet = false;
    let mut verbose = false;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--lines" => {
                if i + 1 >= args.len() {
                    eprintln!("tail: option '{}' requires an argument", args[i]);
                    return Ok(1);
                }
                i += 1;
                match args[i].parse::<i64>() {
                    Ok(n) => line_count = n,
                    Err(_) => {
                        eprintln!("tail: invalid number of lines: '{}'", args[i]);
                        return Ok(1);
                    }
                }
            }
            "-c" | "--bytes" => {
                if i + 1 >= args.len() {
                    eprintln!("tail: option '{}' requires an argument", args[i]);
                    return Ok(1);
                }
                i += 1;
                match args[i].parse::<u64>() {
                    Ok(n) => byte_count = Some(n),
                    Err(_) => {
                        eprintln!("tail: invalid number of bytes: '{}'", args[i]);
                        return Ok(1);
                    }
                }
            }
            "-f" | "--follow" => follow = true,
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
                        eprintln!("tail: invalid number of lines: '{}'", num_str);
                        return Ok(1);
                    }
                }
            }
            arg if arg.starts_with("-c") => {
                let num_str = &arg[2..];
                match num_str.parse::<u64>() {
                    Ok(n) => byte_count = Some(n),
                    Err(_) => {
                        eprintln!("tail: invalid number of bytes: '{}'", num_str);
                        return Ok(1);
                    }
                }
            }
            arg if arg.starts_with('-') => {
                eprintln!("tail: invalid option '{}'", arg);
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
            println!("==> {} <==", if filename == "-" { "standard input" } else { filename });
        }

        let result = if filename == "-" {
            read_from_stdin(line_count, byte_count)
        } else {
            read_from_file(filename, line_count, byte_count)
        };

        if let Err(e) = result {
            eprintln!("tail: {}: {}", filename, e);
            exit_code = 1;
        }
    }

    // Note: Follow mode (-f) is not implemented in this basic version
    if follow {
        eprintln!("tail: follow mode (-f) not implemented in this version");
    }

    Ok(exit_code)
}

fn read_from_file(filename: &str, line_count: i64, byte_count: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(filename).exists() {
        return Err(format!("No such file or directory").into());
    }

    let mut file = File::open(filename)?;

    if let Some(bytes) = byte_count {
        read_last_bytes(&mut file, bytes)?;
    } else {
        let reader = BufReader::new(file);
        read_last_lines(reader, line_count)?;
    }

    Ok(())
}

fn read_from_stdin(line_count: i64, byte_count: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = std::io::stdin();

    if let Some(bytes) = byte_count {
        // For stdin with byte count, we need to read all and keep last N bytes
        let mut buffer = Vec::new();
        stdin.lock().read_to_end(&mut buffer)?;
        
        let start = if buffer.len() > bytes as usize {
            buffer.len() - bytes as usize
        } else {
            0
        };
        
        std::io::Write::write_all(&mut std::io::stdout(), &buffer[start..])?;
    } else {
        read_last_lines(stdin.lock(), line_count)?;
    }

    Ok(())
}

fn read_last_lines<R: BufRead>(reader: R, line_count: i64) -> Result<(), Box<dyn std::error::Error>> {
    if line_count <= 0 {
        return Ok(());
    }

    let mut lines = VecDeque::new();
    let max_lines = line_count as usize;

    for line in reader.lines() {
        let line = line?;
        
        if lines.len() >= max_lines {
            lines.pop_front();
        }
        lines.push_back(line);
    }

    for line in lines {
        println!("{}", line);
    }

    Ok(())
}

fn read_last_bytes(file: &mut File, byte_count: u64) -> Result<(), Box<dyn std::error::Error>> {
    let file_size = file.metadata()?.len();
    
    let start_pos = if file_size > byte_count {
        file_size - byte_count
    } else {
        0
    };

    file.seek(SeekFrom::Start(start_pos))?;
    
    let mut buffer = vec![0; 8192];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        std::io::Write::write_all(&mut std::io::stdout(), &buffer[..bytes_read])?;
    }

    Ok(())
}

fn print_help() {
    println!("Usage: tail [OPTION]... [FILE]...");
    println!("Print the last 10 lines of each FILE to standard output.");
    println!("With more than one FILE, precede each with a header giving the file name.");
    println!();
    println!("Options:");
    println!("  -c, --bytes=NUM      output the last NUM bytes");
    println!("  -f, --follow         output appended data as the file grows (not implemented)");
    println!("  -n, --lines=NUM      output the last NUM lines, instead of the last 10");
    println!("  -q, --quiet, --silent never output headers giving file names");
    println!("  -v, --verbose        always output headers giving file names");
    println!("  -h, --help           display this help and exit");
    println!();
    println!("NUM may have a multiplier suffix:");
    println!("b 512, kB 1000, K 1024, MB 1000*1000, M 1024*1024, and so on.");
    println!();
    println!("Examples:");
    println!("  tail file.txt        Show last 10 lines of file.txt");
    println!("  tail -n 5 file.txt   Show last 5 lines of file.txt");
    println!("  tail -c 100 file.txt Show last 100 bytes of file.txt");
}
