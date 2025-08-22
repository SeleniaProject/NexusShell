use std::process::Command;
use std::io::Read;
use nxsh_core::{ShellError, ErrorKind}; use nxsh_core::error::RuntimeErrorKind;
use super::ui_design::{Colorize, ColorPalette, Icons};

pub fn xargs_cli(args: &[String]) -> Result<(), ShellError> {
    if args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let mut max_args = None;
    let mut null_terminated = false;
    let mut interactive = false;
    let mut command_args = Vec::new();
    let mut delimiter = None;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-n" => {
                if i + 1 < args.len() {
                    max_args = Some(args[i + 1].parse().unwrap_or(1));
                    i += 1;
                }
            },
            "-0" => null_terminated = true,
            "-p" => interactive = true,
            "-d" => {
                if i + 1 < args.len() {
                    delimiter = Some(args[i + 1].chars().next().unwrap_or('\n'));
                    i += 1;
                }
            },
            arg if !arg.starts_with('-') => command_args.push(arg.to_string()),
            _ => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("xargs: invalid option: {}", args[i]))),
        }
        i += 1;
    }
    
    let command = command_args.first().unwrap_or(&"echo".to_string()).clone();
    let cmd_args = if command_args.len() > 1 { &command_args[1..] } else { &[] };
    
    execute_xargs(&command, cmd_args, max_args, null_terminated, interactive, delimiter)
}

fn print_help() {
    println!("xargs - build and execute command lines from standard input

USAGE:
    xargs [OPTIONS] [COMMAND [INITIAL-ARGS]]

OPTIONS:
    -n MAX-ARGS    Use at most MAX-ARGS arguments per command line
    -0             Input items are terminated by null, not whitespace
    -p             Prompt before running each command
    -d DELIM       Use DELIM as delimiter instead of whitespace
    -h, --help     Show this help

EXAMPLES:
    # Remove files listed in stdin
    ls *.tmp | xargs rm
    
    # Process with limited arguments per call
    echo \"1 2 3 4 5\" | xargs -n 2 echo
    
    # Handle filenames with spaces (null-terminated)
    find . -name \"*.txt\" -print0 | xargs -0 grep \"pattern\"");
}

fn execute_xargs(
    command: &str, 
    base_args: &[String], 
    max_args: Option<usize>,
    null_terminated: bool,
    interactive: bool,
    delimiter: Option<char>
) -> Result<(), ShellError> {
    use std::io::{self, BufRead};
    
    let stdin = io::stdin();
    let mut input_args = Vec::new();
    
    // Read input arguments
    if null_terminated {
        // Buffered read of stdin to avoid clippy::unbuffered_bytes
        let mut reader = io::BufReader::new(stdin.lock());
        let mut buffer = String::new();
        let mut tmp = Vec::new();
        // Read all remaining bytes into tmp (typically small for xargs use cases)
        if let Err(e) = reader.read_to_end(&mut tmp) {
            return Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                format!("xargs: {e}"),
            ));
        }
        for b in tmp {
            if b == 0 { // NULL terminator
                if !buffer.is_empty() {
                    input_args.push(buffer.clone());
                    buffer.clear();
                }
            } else {
                buffer.push(char::from(b));
            }
        }
        if !buffer.is_empty() { input_args.push(buffer); }
    } else {
        let delim = delimiter.unwrap_or(' ');
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    let parts: Vec<&str> = if delim == ' ' {
                        line.split_whitespace().collect()
                    } else {
                        line.split(delim).collect()
                    };
                    for part in parts {
                        if !part.is_empty() {
                            input_args.push(part.to_string());
                        }
                    }
                },
                Err(e) => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("xargs: {e}"))),
            }
        }
    }
    
    // Execute commands
    let chunk_size = max_args.unwrap_or(input_args.len().max(1));
    
    for chunk in input_args.chunks(chunk_size) {
        let mut full_args = base_args.to_vec();
        full_args.extend(chunk.iter().cloned());
        
        if interactive {
            print!("Execute: {} {}? (y/N): ", command, full_args.join(" "));
            let mut response = String::new();
            io::stdin().read_line(&mut response).ok();
            if !response.trim().to_lowercase().starts_with('y') {
                continue;
            }
        }
        
        match Command::new(command).args(&full_args).status() {
            Ok(status) if !status.success() => {
                eprintln!("xargs: {} failed with exit code {:?}", command, status.code());
            },
            Err(e) => {
                return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("xargs: {command}: {e}")));
            },
            _ => {}
        }
    }
    
    Ok(())
}



