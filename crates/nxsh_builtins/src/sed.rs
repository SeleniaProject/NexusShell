use std::fs;
use std::io::{BufRead, BufReader, Write};
use fancy_regex::Regex;
use crate::common::{BuiltinResult, BuiltinContext};

/// Stream editor for filtering and transforming text
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    if args.is_empty() {
        eprintln!("sed: missing script");
        return Ok(1);
    }

    let mut script = String::new();
    let mut files = Vec::new();
    let mut in_place = false;
    let mut quiet = false;
    let mut extended_regexp = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-i" | "--in-place" => in_place = true,
            "-n" | "--quiet" | "--silent" => quiet = true,
            "-E" | "-r" | "--regexp-extended" => extended_regexp = true,
            "-e" | "--expression" => {
                if i + 1 >= args.len() {
                    eprintln!("sed: option '{}' requires an argument", args[i]);
                    return Ok(1);
                }
                i += 1;
                if !script.is_empty() {
                    script.push('\n');
                }
                script.push_str(&args[i]);
            }
            "-h" | "--help" => {
                print_help();
                return Ok(0);
            }
            arg if arg.starts_with('-') => {
                eprintln!("sed: invalid option '{}'", arg);
                return Ok(1);
            }
            _ => {
                if script.is_empty() {
                    script = args[i].clone();
                } else {
                    files.push(&args[i]);
                }
            }
        }
        i += 1;
    }

    if script.is_empty() {
        eprintln!("sed: no script specified");
        return Ok(1);
    }

    if files.is_empty() {
        return process_stdin(&script, quiet, extended_regexp);
    }

    let mut exit_code = 0;
    for &filename in &files {
        if let Err(e) = process_file(filename, &script, in_place, quiet, extended_regexp) {
            eprintln!("sed: {}: {}", filename, e);
            exit_code = 1;
        }
    }

    Ok(exit_code)
}

fn process_stdin(script: &str, quiet: bool, extended_regexp: bool) -> BuiltinResult<i32> {
    let stdin = std::io::stdin();
    let reader = stdin.lock();
    
    match process_reader(reader, script, quiet, extended_regexp) {
        Ok(lines) => {
            for line in lines {
                println!("{}", line);
            }
            Ok(0)
        }
        Err(e) => {
            eprintln!("sed: {}", e);
            Ok(1)
        }
    }
}

fn process_file(filename: &str, script: &str, in_place: bool, quiet: bool, extended_regexp: bool) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(filename)?;
    let reader = BufReader::new(file);
    
    let processed_lines = process_reader(reader, script, quiet, extended_regexp)?;
    
    if in_place {
        let mut output_file = fs::File::create(filename)?;
        for line in processed_lines {
            writeln!(output_file, "{}", line)?;
        }
    } else {
        for line in processed_lines {
            println!("{}", line);
        }
    }
    
    Ok(())
}

fn process_reader<R: BufRead>(reader: R, script: &str, quiet: bool, _extended_regexp: bool) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    
    // Parse the script - this is a simplified version
    // Real sed supports much more complex scripting
    let commands = parse_script(script)?;
    
    for line_result in reader.lines() {
        let line = line_result?;
        let mut current_line = line;
        let mut should_print = !quiet;
        
        for command in &commands {
            match apply_command(&current_line, command) {
                Ok((new_line, print_flag)) => {
                    current_line = new_line;
                    if let Some(print) = print_flag {
                        should_print = print;
                    }
                }
                Err(e) => return Err(e),
            }
        }
        
        if should_print {
            result.push(current_line);
        }
    }
    
    Ok(result)
}

#[derive(Debug, Clone)]
enum SedCommand {
    Substitute { pattern: String, replacement: String, global: bool },
    Delete,
    Print,
    Next,
}

fn parse_script(script: &str) -> Result<Vec<SedCommand>, Box<dyn std::error::Error>> {
    let mut commands = Vec::new();
    
    for command_str in script.split('\n') {
        let cmd = command_str.trim();
        if cmd.is_empty() {
            continue;
        }
        
        if cmd.starts_with('s') {
            // Parse substitute command: s/pattern/replacement/flags
            let parts: Vec<&str> = cmd.splitn(4, '/').collect();
            if parts.len() < 3 {
                return Err("Invalid substitute command syntax".into());
            }
            
            let pattern = parts[1].to_string();
            let replacement = parts[2].to_string();
            let flags = if parts.len() > 3 { parts[3] } else { "" };
            let global = flags.contains('g');
            
            commands.push(SedCommand::Substitute { pattern, replacement, global });
        } else if cmd == "d" {
            commands.push(SedCommand::Delete);
        } else if cmd == "p" {
            commands.push(SedCommand::Print);
        } else if cmd == "n" {
            commands.push(SedCommand::Next);
        } else {
            // Simple substitute without delimiters (GNU sed extension)
            if cmd.contains('=') {
                let parts: Vec<&str> = cmd.splitn(2, '=').collect();
                if parts.len() == 2 {
                    commands.push(SedCommand::Substitute {
                        pattern: parts[0].to_string(),
                        replacement: parts[1].to_string(),
                        global: false,
                    });
                }
            }
        }
    }
    
    if commands.is_empty() {
        return Err("No valid commands found in script".into());
    }
    
    Ok(commands)
}

fn apply_command(line: &str, command: &SedCommand) -> Result<(String, Option<bool>), Box<dyn std::error::Error>> {
    match command {
        SedCommand::Substitute { pattern, replacement, global } => {
            let regex = Regex::new(pattern)?;
            let result = if *global {
                regex.replace_all(line, replacement.as_str()).to_string()
            } else {
                regex.replace(line, replacement.as_str()).to_string()
            };
            Ok((result, None))
        }
        SedCommand::Delete => {
            Ok((line.to_string(), Some(false)))
        }
        SedCommand::Print => {
            Ok((line.to_string(), Some(true)))
        }
        SedCommand::Next => {
            Ok((line.to_string(), None))
        }
    }
}

fn print_help() {
    println!("Usage: sed [OPTION]... SCRIPT [FILE]...");
    println!("Stream editor for filtering and transforming text.");
    println!();
    println!("Options:");
    println!("  -e, --expression=SCRIPT  add the script to the commands to be executed");
    println!("  -i, --in-place          edit files in place");
    println!("  -n, --quiet, --silent   suppress automatic printing of pattern space");
    println!("  -E, -r, --regexp-extended  use extended regular expressions");
    println!("  -h, --help              display this help and exit");
    println!();
    println!("SCRIPT is a sequence of sed commands. Common commands:");
    println!("  s/pattern/replacement/  substitute pattern with replacement");
    println!("  s/pattern/replacement/g substitute all occurrences");
    println!("  d                       delete pattern space");
    println!("  p                       print pattern space");
    println!();
    println!("Examples:");
    println!("  sed 's/old/new/g' file.txt      Replace all 'old' with 'new'");
    println!("  sed -i 's/foo/bar/' file.txt     Replace first 'foo' with 'bar' in-place");
    println!("  sed -n '1,5p' file.txt           Print lines 1-5 only");
}
