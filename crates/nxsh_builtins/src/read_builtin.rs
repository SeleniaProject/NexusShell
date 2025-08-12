use std::io::{self, Write, Read};
use anyhow::Result;
use nxsh_core::{ErrorKind, ShellError};
use nxsh_core::error::RuntimeErrorKind;

pub fn read_builtin_cli(args: Vec<String>) -> Result<()> {
    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let mut prompt = None;
    let mut timeout = None;
    let mut delimiter = '\n';
    let mut silent = false;
    let mut array_mode = false;
    let mut raw_mode = false;
    let mut var_names = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" => {
                i += 1;
                if i < args.len() {
                    prompt = Some(args[i].clone());
                }
            }
            "-t" => {
                i += 1;
                if i < args.len() {
                    timeout = args[i].parse().ok();
                }
            }
            "-d" => {
                i += 1;
                if i < args.len() && !args[i].is_empty() {
                    delimiter = args[i].chars().next().unwrap_or('\n');
                }
            }
            "-s" => {
                silent = true;
            }
            "-a" => {
                array_mode = true;
            }
            "-r" => {
                raw_mode = true;
            }
            "-n" => {
                i += 1;
                if i < args.len() {
                    // Number of characters to read (not implemented in this basic version)
                    eprintln!("Warning: -n option not fully implemented");
                }
            }
            arg if !arg.starts_with('-') => {
                var_names.push(arg.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    // Default variable name if none provided
    if var_names.is_empty() {
        var_names.push("REPLY".to_string());
    }

    // Display prompt if provided
    if let Some(p) = &prompt {
        print!("{p}");
        io::stdout().flush()?;
    }

    // Read input
    let input = if let Some(timeout_secs) = timeout {
        read_with_timeout(timeout_secs, delimiter, silent)?
    } else {
        read_input(delimiter, silent)?
    };

    // Process the input based on options
    let processed_input = if raw_mode {
        input
    } else {
        process_escapes(&input)
    };

    // Assign to variables
    if array_mode {
        assign_to_array(&var_names[0], &processed_input)?;
    } else {
        assign_to_variables(&var_names, &processed_input)?;
    }

    Ok(())
}

fn print_help() {
    println!("Usage: read [options] [name ...]");
    println!();
    println!("Read a line from standard input and assign it to variables.");
    println!();
    println!("Options:");
    println!("  -a              Assign the words read to sequential indices of array variable");
    println!("  -d delim        Use delimiter instead of newline to terminate the input");
    println!("  -p prompt       Display prompt before reading");
    println!("  -r              Do not interpret backslash escape sequences");
    println!("  -s              Do not echo input (silent mode)");
    println!("  -t timeout      Time out after timeout seconds");
    println!("  -n nchars       Read at most nchars characters");
    println!("  -h, --help      Show this help message");
    println!();
    println!("Arguments:");
    println!("  name            Variable name(s) to assign input to (default: REPLY)");
    println!();
    println!("Examples:");
    println!("  read            # Read into REPLY variable");
    println!("  read name       # Read into 'name' variable");
    println!("  read -p \"Enter your name: \" name");
    println!("  read -s password   # Silent input for passwords");
    println!("  read -t 10 input   # Timeout after 10 seconds");
    println!("  read -d ':' fields # Use colon as delimiter");
}

fn read_input(delimiter: char, silent: bool) -> Result<String> {
    let stdin = io::stdin();
    let mut input = String::new();

    if delimiter == '\n' {
        // Standard line reading
        stdin.read_line(&mut input)?;
        // Remove trailing newline
        if input.ends_with('\n') {
            input.pop();
            if input.ends_with('\r') {
                input.pop();
            }
        }
    } else {
        // Read character by character until delimiter
        let handle = stdin.lock();
        for byte_result in handle.bytes() {
            let byte = byte_result?;
            let ch = char::from(byte);
            
            if ch == delimiter {
                break;
            }
            
            input.push(ch);
            
            if !silent {
                print!("{ch}");
                io::stdout().flush()?;
            }
        }
    }

    Ok(input)
}

fn read_with_timeout(timeout_secs: u64, delimiter: char, silent: bool) -> Result<String> {
    use std::time::Duration;
    use std::thread;
    use std::sync::{Arc, Mutex};
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    let input_ref = Arc::new(Mutex::new(String::new()));
    let input_clone = Arc::clone(&input_ref);

    // Spawn thread to read input
    thread::spawn(move || {
        let result = read_input(delimiter, silent);
        match result {
            Ok(input) => {
                *input_clone.lock().unwrap() = input;
                tx.send(true).ok();
            }
            Err(_) => {
                tx.send(false).ok();
            }
        }
    });

    // Wait for input or timeout
    match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
        Ok(true) => {
            let input = input_ref.lock().unwrap();
            Ok(input.clone())
        }
        Ok(false) => {
            Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Failed to read input").into())
        }
        Err(_) => {
            println!(); // New line after timeout
            Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Read operation timed out").into())
        }
    }
}

fn process_escapes(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(&next_ch) = chars.peek() {
                match next_ch {
                    'n' => {
                        chars.next();
                        result.push('\n');
                    }
                    't' => {
                        chars.next();
                        result.push('\t');
                    }
                    'r' => {
                        chars.next();
                        result.push('\r');
                    }
                    '\\' => {
                        chars.next();
                        result.push('\\');
                    }
                    '"' => {
                        chars.next();
                        result.push('"');
                    }
                    '\'' => {
                        chars.next();
                        result.push('\'');
                    }
                    _ => {
                        result.push(ch);
                    }
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn assign_to_variables(var_names: &[String], input: &str) -> Result<()> {
    if var_names.len() == 1 {
        // Single variable gets the entire input
        set_shell_variable(&var_names[0], input)?;
    } else {
        // Multiple variables: split input by whitespace
        let words: Vec<&str> = input.split_whitespace().collect();
        
        for (i, var_name) in var_names.iter().enumerate() {
            if i < words.len() {
                if i == var_names.len() - 1 {
                    // Last variable gets remaining words
                    let remaining = words[i..].join(" ");
                    set_shell_variable(var_name, &remaining)?;
                } else {
                    set_shell_variable(var_name, words[i])?;
                }
            } else {
                // No more words, set to empty string
                set_shell_variable(var_name, "")?;
            }
        }
    }

    Ok(())
}

fn assign_to_array(array_name: &str, input: &str) -> Result<()> {
    let words: Vec<&str> = input.split_whitespace().collect();
    
    // In a real shell, this would create an array
    // For simplicity, we'll create indexed variables
    for (i, word) in words.iter().enumerate() {
        let var_name = format!("{array_name}[{i}]");
        set_shell_variable(&var_name, word)?;
    }

    // Set array length
    let length_var = format!("{array_name}[#]");
    set_shell_variable(&length_var, &words.len().to_string())?;

    Ok(())
}

fn set_shell_variable(name: &str, value: &str) -> Result<()> {
    if !is_valid_var_name(name) {
        return Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            format!("Invalid variable name: {name}")
        ).into());
    }

    // In a real shell implementation, this would set shell variables
    // For now, we'll set environment variables
    std::env::set_var(name, value);
    
    Ok(())
}

fn is_valid_var_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Allow array syntax like var[0]
    if let Some(bracket_pos) = name.find('[') {
        let base_name = &name[..bracket_pos];
        return is_valid_identifier(base_name);
    }

    is_valid_identifier(name)
}

fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // First character must be letter or underscore
    let first_char = name.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }

    // Remaining characters must be alphanumeric or underscore
    for c in name.chars().skip(1) {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return false;
        }
    }

    true
}

// Utility functions for shell integration

pub fn read_from_stdin() -> Result<String> {
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)?;
    
    // Remove trailing newline
    if input.ends_with('\n') {
        input.pop();
        if input.ends_with('\r') {
            input.pop();
        }
    }
    
    Ok(input)
}

pub fn read_with_prompt(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    read_from_stdin()
}

pub fn read_password() -> Result<String> {
    // This is a simplified version - real password input would disable echo
    read_from_stdin()
}

pub fn read_single_char() -> Result<char> {
    use std::io::Read;
    
    let mut buffer = [0; 1];
    io::stdin().read_exact(&mut buffer)?;
    Ok(char::from(buffer[0]))
}

pub fn confirm_action(prompt: &str) -> Result<bool> {
    loop {
        let input = read_with_prompt(&format!("{prompt} (y/n): "))?;
        match input.to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Please enter 'y' or 'n'"),
        }
    }
}
