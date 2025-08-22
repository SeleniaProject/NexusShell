use nxsh_core::{ShellError, ErrorKind}; use nxsh_core::error::RuntimeErrorKind;

pub fn until_cli(args: &[String]) -> Result<(), ShellError> {
    if args.is_empty() || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    // Basic until loop implementation
    // This is a simplified version - full shell until would require complex parsing
    
    // Find the 'do' keyword
    let do_pos = args.iter().position(|arg| arg == "do")
        .ok_or_else(|| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "until: missing 'do' keyword"))?;
    
    // Find the 'done' keyword
    let done_pos = args.iter().position(|arg| arg == "done")
        .ok_or_else(|| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "until: missing 'done' keyword"))?;
    
    if do_pos >= done_pos {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "until: invalid syntax - 'do' must come before 'done'"));
    }
    
    let condition_args = &args[..do_pos];
    let body_args = &args[do_pos + 1..done_pos];
    
    execute_until_loop(condition_args, body_args)
}

fn print_help() {
    println!("until - execute commands until condition becomes true

USAGE:
    until CONDITION; do
        COMMANDS
    done

DESCRIPTION:
    Execute COMMANDS repeatedly as long as CONDITION evaluates to false.
    The condition is tested before each iteration. This is the opposite
    of the while loop.

CONDITIONS:
    Conditions are typically test expressions or command exit statuses.
    Common test conditions:
    - [ -f FILE ]     : True if FILE exists and is a regular file
    - [ -d DIR ]      : True if DIR exists and is a directory
    - [ -z STRING ]   : True if STRING is empty
    - [ STRING1 = STRING2 ] : True if strings are equal
    - [ NUM1 -eq NUM2 ] : True if numbers are equal

EXAMPLES:
    # Wait for a file to appear
    until [ -f /tmp/ready ]; do
        echo \"Waiting for file...\"
        sleep 1
    done

    # Wait for a service to stop
    until ! pgrep myservice > /dev/null; do
        echo \"Waiting for service to stop...\"
        sleep 2
    done

    # Count down
    i=10
    until [ $i -eq 0 ]; do
        echo $i
        i=$((i - 1))
        sleep 1
    done

    # Wait for network connectivity
    until ping -c 1 google.com > /dev/null 2>&1; do
        echo \"Waiting for network...\"
        sleep 5
    done

CONTROL FLOW:
    - break    : Exit the until loop immediately
    - continue : Skip to the next iteration");
}

fn execute_until_loop(condition_args: &[String], body_args: &[String]) -> Result<(), ShellError> {
    let mut iteration_count = 0;
    let max_iterations = 10000; // Safety limit to prevent infinite loops in demo
    
    loop {
        iteration_count += 1;
        
        if iteration_count > max_iterations {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "until: maximum iteration limit reached (safety limit)"));
        }
        
        // Evaluate condition - until loops continue while condition is FALSE
        let condition_result = evaluate_condition(condition_args)?;
        
        if condition_result {
            break; // Exit when condition becomes true
        }
        
        // Execute body
        match execute_body(body_args) {
            Ok(ControlFlow::Continue) => continue,
            Ok(ControlFlow::Break) => break,
            Ok(ControlFlow::Normal) => {},
            Err(e) => return Err(e),
        }
    }
    
    Ok(())
}

#[derive(Debug)]
enum ControlFlow {
    Normal,
    Break,
    Continue,
}

fn evaluate_condition(condition_args: &[String]) -> Result<bool, ShellError> {
    if condition_args.is_empty() {
        return Ok(false);
    }

    // Handle test expressions
    if condition_args[0] == "[" && condition_args.last() == Some(&"]".to_string()) {
        return evaluate_test_expression(&condition_args[1..condition_args.len()-1]);
    }

    // Handle negation
    if condition_args[0] == "!" {
        let result = evaluate_condition(&condition_args[1..])?;
        return Ok(!result);
    }

    // Handle simple command execution (check exit status)
    match execute_command_check_status(condition_args) {
        Ok(status) => Ok(status == 0),
        Err(_) => Ok(false),
    }
}

fn evaluate_test_expression(args: &[String]) -> Result<bool, ShellError> {
    if args.is_empty() {
        return Ok(false);
    }

    if args.len() == 1 {
        // Single argument - test if non-empty
        return Ok(!args[0].is_empty());
    }

    if args.len() == 2 {
        match args[0].as_str() {
            "-f" => return Ok(std::path::Path::new(&args[1]).is_file()),
            "-d" => return Ok(std::path::Path::new(&args[1]).is_dir()),
            "-e" => return Ok(std::path::Path::new(&args[1]).exists()),
            "-r" => return Ok(is_readable(&args[1])),
            "-w" => return Ok(is_writable(&args[1])),
            "-x" => return Ok(is_executable(&args[1])),
            "-s" => return Ok(file_has_size(&args[1])),
            "-z" => return Ok(args[1].is_empty()),
            "-n" => return Ok(!args[1].is_empty()),
            "!" => return Ok(!evaluate_test_expression(&args[1..])?),
            _ => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Unknown test operator: {}", args[0]))),
        }
    }

    if args.len() == 3 {
        let left = &args[0];
        let op = &args[1];
        let right = &args[2];

        match op.as_str() {
            "=" | "==" => return Ok(left == right),
            "!=" => return Ok(left != right),
            "-eq" => return Ok(parse_number(left)? == parse_number(right)?),
            "-ne" => return Ok(parse_number(left)? != parse_number(right)?),
            "-lt" => return Ok(parse_number(left)? < parse_number(right)?),
            "-le" => return Ok(parse_number(left)? <= parse_number(right)?),
            "-gt" => return Ok(parse_number(left)? > parse_number(right)?),
            "-ge" => return Ok(parse_number(left)? >= parse_number(right)?),
            _ => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Unknown comparison operator: {op}"))),
        }
    }

    // Handle logical operators
    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "-a" | "&&" => {
                let left = evaluate_test_expression(&args[..i])?;
                if !left {
                    return Ok(false); // Short circuit
                }
                let right = evaluate_test_expression(&args[i+1..])?;
                return Ok(right);
            },
            "-o" | "||" => {
                let left = evaluate_test_expression(&args[..i])?;
                if left {
                    return Ok(true); // Short circuit
                }
                let right = evaluate_test_expression(&args[i+1..])?;
                return Ok(right);
            },
            _ => {}
        }
    }

    Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid test expression"))
}

fn execute_body(body_args: &[String]) -> Result<ControlFlow, ShellError> {
    let mut i = 0;
    
    while i < body_args.len() {
        // Handle control flow statements
        match body_args[i].as_str() {
            "break" => return Ok(ControlFlow::Break),
            "continue" => return Ok(ControlFlow::Continue),
            _ => {
                // Find the end of this command (until ';' or end of args)
                let mut cmd_end = i + 1;
                while cmd_end < body_args.len() && body_args[cmd_end] != ";" {
                    cmd_end += 1;
                }
                
                execute_command(&body_args[i..cmd_end])?;
                
                i = cmd_end;
                if i < body_args.len() && body_args[i] == ";" {
                    i += 1;
                }
            }
        }
    }
    
    Ok(ControlFlow::Normal)
}

fn execute_command(args: &[String]) -> Result<(), ShellError> {
    if args.is_empty() {
        return Ok(());
    }

    // Handle some basic built-in commands
    match args[0].as_str() {
        "echo" => {
            let output = args[1..].join(" ");
            println!("{output}");
        },
        "sleep" => {
            if args.len() > 1 {
                if let Ok(seconds) = args[1].parse::<u64>() {
                    std::thread::sleep(std::time::Duration::from_secs(seconds));
                }
            }
        },
        "ping" => {
            // Simulate ping command
            if args.len() > 1 {
                println!("PING {} ...", args[args.len() - 1]);
                // For demo, randomly succeed or fail
                use rand::Rng;
                let success = rand::thread_rng().gen_bool(0.7);
                if !success {
                    return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "ping failed"));
                }
            }
        },
        "pgrep" => {
            // Simulate pgrep command
            if args.len() > 1 {
                // For demo, randomly find or not find process
                use rand::Rng;
                let found = rand::thread_rng().gen_bool(0.3);
                if !found {
                    return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "process not found"));
                }
                println!("1234");
            }
        },
        _ => {
            // For other commands, try to execute them
            use std::process::Command;
            let _ = Command::new(&args[0]).args(&args[1..]).status();
        }
    }

    Ok(())
}

fn execute_command_check_status(args: &[String]) -> Result<i32, ShellError> {
    if args.is_empty() {
        return Ok(1);
    }

    // Handle special commands for demonstration
    match args[0].as_str() {
        "ping" => {
            // Simulate ping command
            use rand::Rng;
            let success = rand::thread_rng().gen_bool(0.7);
            return Ok(if success { 0 } else { 1 });
        },
        "pgrep" => {
            // Simulate pgrep command
            use rand::Rng;
            let found = rand::thread_rng().gen_bool(0.3);
            return Ok(if found { 0 } else { 1 });
        },
        _ => {}
    }

    // Try to execute the command and get its exit status
    use std::process::Command;
    
    match Command::new(&args[0]).args(&args[1..]).status() {
        Ok(status) => {
            if let Some(code) = status.code() {
                Ok(code)
            } else {
                Ok(1) // Terminated by signal
            }
        },
        Err(_) => Ok(127), // Command not found
    }
}

// Helper functions for file tests
fn is_readable(path: &str) -> bool {
    use std::fs::OpenOptions;
    OpenOptions::new().read(true).open(path).is_ok()
}

fn is_writable(path: &str) -> bool {
    use std::fs::OpenOptions;
    if std::path::Path::new(path).exists() {
        OpenOptions::new().write(true).open(path).is_ok()
    } else {
        // Check if we can create a file in the parent directory
        if let Some(parent) = std::path::Path::new(path).parent() {
            parent.exists()
        } else {
            false
        }
    }
}

fn is_executable(path: &str) -> bool {
    use std::path::Path;
    let path = Path::new(path);
    
    if !path.exists() {
        return false;
    }

    #[cfg(unix)]
    {
        #[cfg(unix)] use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = path.metadata() {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0;
        }
    }

    #[cfg(windows)]
    {
        // On Windows, check if it's an executable file by extension
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            return matches!(ext_str.as_str(), "exe" | "bat" | "cmd" | "com" | "ps1");
        }
    }

    false
}

fn file_has_size(path: &str) -> bool {
    std::fs::metadata(path)
        .map(|metadata| metadata.len() > 0)
        .unwrap_or(false)
}

fn parse_number(s: &str) -> Result<i64, ShellError> {
    s.parse().map_err(|_| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Invalid number: {s}")))
}



