use nxsh_core::{ShellError, ErrorKind};
use nxsh_core::error::RuntimeErrorKind;

pub fn for_cmd_cli(args: &[String]) -> Result<(), ShellError> {
    if args.is_empty() || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    // Basic for loop implementation
    // This is a simplified version - full shell for would require complex parsing
    
    // Find the 'in' keyword
    let in_pos = args.iter().position(|arg| arg == "in");
    
    // Find the 'do' keyword
    let do_pos = args.iter().position(|arg| arg == "do")
        .ok_or_else(|| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "for: missing 'do' keyword"))?;
    
    // Find the 'done' keyword
    let done_pos = args.iter().position(|arg| arg == "done")
        .ok_or_else(|| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "for: missing 'done' keyword"))?;
    
    if do_pos >= done_pos {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "for: invalid syntax - 'do' must come before 'done'"));
    }
    
    let variable = &args[0];
    let body_args = &args[do_pos + 1..done_pos];
    
    if let Some(in_pos) = in_pos {
        if in_pos >= do_pos {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "for: invalid syntax - 'in' must come before 'do'"));
        }
        let items = &args[in_pos + 1..do_pos];
        execute_for_in_loop(variable, items, body_args)
    } else {
        // C-style for loop: for ((init; condition; increment))
        if args.len() > 1 && args[1].starts_with("((") {
            execute_c_style_for_loop(&args[1..do_pos], body_args)
        } else {
            // Default: iterate over positional parameters
            execute_for_in_loop(variable, &[], body_args)
        }
    }
}

fn print_help() {
    println!("for - execute commands for each item in a list

USAGE:
    for VARIABLE in LIST; do
        COMMANDS
    done

    for VARIABLE; do
        COMMANDS
    done

    for ((INIT; CONDITION; INCREMENT)); do
        COMMANDS
    done

DESCRIPTION:
    Execute COMMANDS for each item in LIST, setting VARIABLE to each item
    in turn. If LIST is omitted, iterate over positional parameters.
    
    The C-style for loop executes INIT once, then repeatedly executes
    COMMANDS while CONDITION is true, executing INCREMENT after each iteration.

EXAMPLES:
    # Iterate over a list of items
    for fruit in apple banana cherry; do
        echo \"I like $fruit\"
    done

    # Iterate over files
    for file in *.txt; do
        echo \"Processing $file\"
    done

    # C-style numeric loop
    for ((i=1; i<=10; i++)); do
        echo \"Count: $i\"
    done

    # Iterate over positional parameters
    for arg; do
        echo \"Argument: $arg\"
    done

    # Iterate over command output
    for line in $(cat file.txt); do
        echo \"Line: $line\"
    done

CONTROL FLOW:
    - break    : Exit the for loop immediately
    - continue : Skip to the next iteration");
}

fn execute_for_in_loop(variable: &str, items: &[String], body_args: &[String]) -> Result<(), ShellError> {
    let iteration_items = if items.is_empty() {
        // Use positional parameters (simulated)
        vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()]
    } else {
        expand_items(items)?
    };
    
    for item in &iteration_items {
        // Set the loop variable (in a real shell, this would set an environment variable)
        println!("Setting {variable} = {item}");
        
        // Execute body
        match execute_body(body_args, variable, item) {
            Ok(ControlFlow::Continue) => continue,
            Ok(ControlFlow::Break) => break,
            Ok(ControlFlow::Normal) => {},
            Err(e) => return Err(e),
        }
    }
    
    Ok(())
}

fn execute_c_style_for_loop(for_expr: &[String], body_args: &[String]) -> Result<(), ShellError> {
    // Parse C-style for loop: ((init; condition; increment))
    let expr_str = for_expr.join(" ");
    
    if !expr_str.starts_with("((") || !expr_str.ends_with("))") {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "for: invalid C-style for loop syntax"));
    }
    
    let inner = &expr_str[2..expr_str.len() - 2];
    let parts: Vec<&str> = inner.split(';').map(|s| s.trim()).collect();
    
    if parts.len() != 3 {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "for: C-style for loop must have exactly 3 parts separated by semicolons"));
    }
    
    let init = parts[0];
    let condition = parts[1];
    let increment = parts[2];
    
    // Execute initialization
    execute_arithmetic_expression(init)?;
    
    let mut iteration_count = 0;
    let max_iterations = 10000; // Safety limit
    
    loop {
        iteration_count += 1;
        
        if iteration_count > max_iterations {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "for: maximum iteration limit reached (safety limit)"));
        }
        
        // Check condition
        if !evaluate_arithmetic_condition(condition)? {
            break;
        }
        
        // Execute body
        match execute_body(body_args, "", "") {
            Ok(ControlFlow::Continue) => {
                execute_arithmetic_expression(increment)?;
                continue;
            },
            Ok(ControlFlow::Break) => break,
            Ok(ControlFlow::Normal) => {
                execute_arithmetic_expression(increment)?;
            },
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

fn expand_items(items: &[String]) -> Result<Vec<String>, ShellError> {
    let mut expanded = Vec::new();
    
    for item in items {
        if item.contains('*') || item.contains('?') || item.contains('[') {
            // Glob expansion (simplified)
            if let Ok(paths) = glob::glob(item) {
                for path in paths.flatten() {
                    expanded.push(path.to_string_lossy().to_string());
                }
            } else {
                expanded.push(item.clone());
            }
        } else if item.starts_with("$(") && item.ends_with(")") {
            // Command substitution (simplified)
            let cmd = &item[2..item.len() - 1];
            if let Ok(output) = execute_command_for_output(&[cmd.to_string()]) {
                for line in output.lines() {
                    if !line.trim().is_empty() {
                        expanded.push(line.trim().to_string());
                    }
                }
            }
        } else {
            expanded.push(item.clone());
        }
    }
    
    if expanded.is_empty() {
        expanded.extend(items.iter().cloned());
    }
    
    Ok(expanded)
}

fn execute_body(body_args: &[String], variable: &str, value: &str) -> Result<ControlFlow, ShellError> {
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
                
                let mut command = body_args[i..cmd_end].to_vec();
                
                // Simple variable substitution
                if !variable.is_empty() {
                    for arg in &mut command {
                        *arg = arg.replace(&format!("${variable}"), value)
                                 .replace(&format!("${{{variable}}}"), value);
                    }
                }
                
                execute_command(&command)?;
                
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
        _ => {
            // For other commands, try to execute them
            use std::process::Command;
            let _ = Command::new(&args[0]).args(&args[1..]).status();
        }
    }

    Ok(())
}

fn execute_command_for_output(args: &[String]) -> Result<String, ShellError> {
    if args.is_empty() {
        return Ok(String::new());
    }

    use std::process::Command;
    
    match Command::new(&args[0]).args(&args[1..]).output() {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Command failed"))
            }
        },
        Err(_) => Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Command not found")),
    }
}

fn execute_arithmetic_expression(expr: &str) -> Result<i64, ShellError> {
    // Simple arithmetic expression evaluation
    let expr = expr.trim();
    
    if expr.contains('=') {
        // Assignment
        let parts: Vec<&str> = expr.splitn(2, '=').collect();
        if parts.len() == 2 {
            let var = parts[0].trim();
            let val_str = parts[1].trim();
            let value = evaluate_arithmetic(val_str)?;
            println!("Setting {var} = {value}");
            return Ok(value);
        }
    }
    
    evaluate_arithmetic(expr)
}

fn evaluate_arithmetic_condition(expr: &str) -> Result<bool, ShellError> {
    let result = evaluate_arithmetic(expr)?;
    Ok(result != 0)
}

fn evaluate_arithmetic(expr: &str) -> Result<i64, ShellError> {
    // Very basic arithmetic evaluation
    let expr = expr.trim();
    
    // Handle increment/decrement
    if let Some(_var) = expr.strip_suffix("++") {
        // In a real implementation, this would get the variable value
        return Ok(1);
    }
    
    if let Some(_var) = expr.strip_suffix("--") {
        // In a real implementation, this would get the variable value
        return Ok(1);
    }
    
    // Handle simple comparisons
    if expr.contains("<=") {
        let parts: Vec<&str> = expr.split("<=").collect();
        if parts.len() == 2 {
            let left = parse_arithmetic_operand(parts[0].trim())?;
            let right = parse_arithmetic_operand(parts[1].trim())?;
            return Ok(if left <= right { 1 } else { 0 });
        }
    }
    
    if expr.contains(">=") {
        let parts: Vec<&str> = expr.split(">=").collect();
        if parts.len() == 2 {
            let left = parse_arithmetic_operand(parts[0].trim())?;
            let right = parse_arithmetic_operand(parts[1].trim())?;
            return Ok(if left >= right { 1 } else { 0 });
        }
    }
    
    if expr.contains('<') {
        let parts: Vec<&str> = expr.split('<').collect();
        if parts.len() == 2 {
            let left = parse_arithmetic_operand(parts[0].trim())?;
            let right = parse_arithmetic_operand(parts[1].trim())?;
            return Ok(if left < right { 1 } else { 0 });
        }
    }
    
    if expr.contains('>') {
        let parts: Vec<&str> = expr.split('>').collect();
        if parts.len() == 2 {
            let left = parse_arithmetic_operand(parts[0].trim())?;
            let right = parse_arithmetic_operand(parts[1].trim())?;
            return Ok(if left > right { 1 } else { 0 });
        }
    }
    
    // Handle simple arithmetic
    if expr.contains('+') {
        let parts: Vec<&str> = expr.split('+').collect();
        if parts.len() == 2 {
            let left = parse_arithmetic_operand(parts[0].trim())?;
            let right = parse_arithmetic_operand(parts[1].trim())?;
            return Ok(left + right);
        }
    }
    
    if expr.contains('-') {
        let parts: Vec<&str> = expr.split('-').collect();
        if parts.len() == 2 {
            let left = parse_arithmetic_operand(parts[0].trim())?;
            let right = parse_arithmetic_operand(parts[1].trim())?;
            return Ok(left - right);
        }
    }
    
    // Simple number or variable
    parse_arithmetic_operand(expr)
}

fn parse_arithmetic_operand(s: &str) -> Result<i64, ShellError> {
    let s = s.trim();
    
    // Try to parse as number
    if let Ok(num) = s.parse::<i64>() {
        return Ok(num);
    }
    
    // In a real shell, this would look up variable values
    // For now, return a default value
    Ok(0)
}
