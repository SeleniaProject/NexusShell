use nxsh_core::{ShellError, ErrorKind};
use nxsh_core::error::RuntimeErrorKind;

pub fn if_cmd_cli(args: &[String]) -> Result<(), ShellError> {
    if args.is_empty() || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    // Basic if command implementation
    // This is a simplified version - full shell if would require complex parsing
    
    if args.len() < 3 {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "if: missing condition or then clause"));
    }

    let condition_result = evaluate_condition(&args[0..args.len()-2])?;
    
    if condition_result {
        // Execute then clause (simplified)
        if let Some(then_pos) = args.iter().position(|arg| arg == "then") {
            if then_pos + 1 < args.len() {
                execute_command(&args[then_pos + 1..])?;
            }
        }
    } else {
        // Execute else clause if present
        if let Some(else_pos) = args.iter().position(|arg| arg == "else") {
            if else_pos + 1 < args.len() {
                let fi_pos = args.iter().position(|arg| arg == "fi").unwrap_or(args.len());
                execute_command(&args[else_pos + 1..fi_pos])?;
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!("if - conditional execution construct

USAGE:
    if CONDITION; then
        COMMANDS
    [else
        COMMANDS]
    fi

DESCRIPTION:
    Execute commands conditionally. The if statement evaluates a condition
    and executes the then clause if the condition is true, otherwise executes
    the else clause if present.

CONDITIONS:
    Conditions are typically test expressions or command exit statuses.
    Common test conditions:
    - [ -f FILE ]     : True if FILE exists and is a regular file
    - [ -d DIR ]      : True if DIR exists and is a directory
    - [ -z STRING ]   : True if STRING is empty
    - [ STRING1 = STRING2 ] : True if strings are equal
    - [ NUM1 -eq NUM2 ] : True if numbers are equal

EXAMPLES:
    if [ -f /etc/passwd ]; then
        echo \"Password file exists\"
    fi

    if [ \"$USER\" = \"root\" ]; then
        echo \"Running as root\"
    else
        echo \"Running as regular user\"
    fi");
}

fn evaluate_condition(condition_args: &[String]) -> Result<bool, ShellError> {
    if condition_args.is_empty() {
        return Ok(false);
    }

    // Handle test expressions
    if condition_args[0] == "[" && condition_args.last() == Some(&"]".to_string()) {
        return evaluate_test_expression(&condition_args[1..condition_args.len()-1]);
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
                let right = evaluate_test_expression(&args[i+1..])?;
                return Ok(left && right);
            },
            "-o" | "||" => {
                let left = evaluate_test_expression(&args[..i])?;
                let right = evaluate_test_expression(&args[i+1..])?;
                return Ok(left || right);
            },
            _ => {}
        }
    }

    Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid test expression"))
}

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
            parent.exists() && OpenOptions::new().write(true).create(true).open(format!("{path}.test")).and_then(|_| std::fs::remove_file(format!("{path}.test"))).is_ok()
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

fn execute_command(args: &[String]) -> Result<(), ShellError> {
    if args.is_empty() {
        return Ok(());
    }

    // This is a simplified command execution
    // In a real shell, this would integrate with the command execution system
    println!("Would execute: {}", args.join(" "));
    Ok(())
}

fn execute_command_check_status(args: &[String]) -> Result<i32, ShellError> {
    if args.is_empty() {
        return Ok(1);
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

