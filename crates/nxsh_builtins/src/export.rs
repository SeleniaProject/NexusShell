use std::collections::HashMap;
use std::env;
use anyhow::Result;
use nxsh_core::{ErrorKind, ShellError};
use crate::function::{get_function, list_functions};


pub fn export_cli(args: Vec<String>) -> Result<()> {
    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    // If no arguments, print all exported variables
    if args.is_empty() {
        print_all_exported_vars();
        return Ok(());
    }

    let mut print_mode = false;
    let mut function_mode = false;
    let mut name_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" => {
                print_mode = true;
            }
            "-f" => {
                function_mode = true;
            }
            "-n" => {
                name_mode = true;
            }
            arg => {
                if arg.starts_with('-') {
                    return Err(ShellError::new(
                        ErrorKind::InvalidArgument,
                        format!("Unknown option: {}", arg)
                    ).into());
                }

                if function_mode {
                    // export -f NAME [...]  (mark shell functions for export)
                    return export_functions(&args[i..]);
                }

                if name_mode {
                    return remove_from_export(&arg);
                }

                // Handle variable assignment
                return handle_export_assignment(&arg);
            }
        }
        i += 1;
    }

    if print_mode {
        print_all_exported_vars();
    }

    Ok(())
}

fn print_help() {
    println!("Usage: export [-p] [-n] [name[=value] ...]");
    println!();
    println!("Mark variables for automatic export to the environment of subsequently");
    println!("executed commands.");
    println!();
    println!("Options:");
    println!("  -p      Display all exported variables in a form that can be reused as input");
    println!("  -n      Remove the export property from named variables");
    println!("  -f      Names refer to functions (export shell functions)");
    println!("  -h, --help  Show this help message");
    println!();
    println!("Arguments:");
    println!("  name=value    Set variable name to value and mark for export");
    println!("  name          Mark existing variable for export");
    println!();
    println!("Examples:");
    println!("  export                     # List all exported variables");
    println!("  export PATH=/usr/bin       # Export PATH with new value");
    println!("  export EDITOR              # Export existing EDITOR variable");
    println!("  export -n PATH             # Remove PATH from exports");
    println!("  export -p                  # Print exportable format");
}

fn print_all_exported_vars() {
    let mut vars: Vec<(String, String)> = env::vars().collect();
    vars.sort_by(|a, b| a.0.cmp(&b.0));

    for (name, value) in vars {
        println!("declare -x {}=\"{}\"", name, escape_value(&value));
    }

    // Also print exported functions in a POSIX-compatible form
    // We treat all defined functions as exportable for subshells in this simplified model
    for fname in list_functions() {
        println!("declare -fx {}", fname);
    }
}

fn handle_export_assignment(arg: &str) -> Result<()> {
    if let Some(eq_pos) = arg.find('=') {
        // Variable assignment: name=value
        let name = &arg[..eq_pos];
        let value = &arg[eq_pos + 1..];

        if !is_valid_var_name(name) {
            return Err(ShellError::new(
                ErrorKind::InvalidArgument,
                format!("Invalid variable name: {}", name)
            ).into());
        }

        env::set_var(name, value);
        println!("Exported {}={}", name, value);
    } else {
        // Just variable name: export existing variable
        let name = arg;

        if !is_valid_var_name(name) {
            return Err(ShellError::new(
                ErrorKind::InvalidArgument,
                format!("Invalid variable name: {}", name)
            ).into());
        }

        match env::var(name) {
            Ok(value) => {
                // Variable already exists, just mark as exported
                println!("Exported {}={}", name, value);
            }
            Err(_) => {
                // Variable doesn't exist, create with empty value
                env::set_var(name, "");
                println!("Exported {}=", name);
            }
        }
    }

    Ok(())
}

fn remove_from_export(name: &str) -> Result<()> {
    if !is_valid_var_name(name) {
        return Err(ShellError::new(
            ErrorKind::InvalidArgument,
            format!("Invalid variable name: {}", name)
        ).into());
    }

    // Remove environment variable from exported set
    match env::var(name) {
        Ok(_) => {
            // We cannot truly "unexport" while preserving a local-only var here; emulate by clearing from process env
            env::remove_var(name);
            println!("Removed {} from exports", name);
        }
        Err(_) => {
            println!("Variable {} not found", name);
        }
    }

    Ok(())
}

fn is_valid_var_name(name: &str) -> bool {
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

fn escape_value(value: &str) -> String {
    let mut escaped = String::new();
    
    for c in value.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\t' => escaped.push_str("\\t"),
            '\r' => escaped.push_str("\\r"),
            '$' => escaped.push_str("\\$"),
            '`' => escaped.push_str("\\`"),
            _ => escaped.push(c),
        }
    }
    
    escaped
}

// Additional utilities for export management

pub fn is_exported(name: &str) -> bool {
    // In a real implementation, we'd maintain a set of exported variable names
    // For now, check if it exists in environment
    env::var(name).is_ok()
}

pub fn get_exported_vars() -> HashMap<String, String> {
    env::vars().collect()
}

pub fn export_var(name: &str, value: &str) -> Result<()> {
    if !is_valid_var_name(name) {
        return Err(ShellError::new(
            ErrorKind::InvalidArgument,
            format!("Invalid variable name: {}", name)
        ).into());
    }

    env::set_var(name, value);
    Ok(())
}

pub fn unexport_var(name: &str) -> Result<()> {
    if !is_valid_var_name(name) {
        return Err(ShellError::new(
            ErrorKind::InvalidArgument,
            format!("Invalid variable name: {}", name)
        ).into());
    }

    if env::var(name).is_ok() {
        env::remove_var(name);
        println!("Variable {} is no longer exported", name);
    }

    Ok(())
}

// Shell built-in specific functions

pub fn handle_shell_assignment(assignment: &str) -> Result<()> {
    if let Some(eq_pos) = assignment.find('=') {
        let name = &assignment[..eq_pos];
        let value = &assignment[eq_pos + 1..];
        
        if !is_valid_var_name(name) {
            return Err(ShellError::new(
                ErrorKind::InvalidArgument,
                format!("Invalid variable name: {}", name)
            ).into());
        }

        env::set_var(name, value);
        Ok(())
    } else {
        Err(ShellError::new(
            ErrorKind::InvalidArgument,
            "Invalid assignment format"
        ).into())
    }
}

pub fn expand_variable(name: &str) -> Option<String> {
    env::var(name).ok()
}

pub fn substitute_variables(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some(&next_char) = chars.peek() {
                if next_char == '{' {
                    // Handle ${var} format
                    chars.next(); // consume '{'
                    let mut var_name = String::new();
                    
                    while let Some(c) = chars.next() {
                        if c == '}' {
                            break;
                        }
                        var_name.push(c);
                    }
                    
                    if let Some(value) = expand_variable(&var_name) {
                        result.push_str(&value);
                    }
                } else if next_char.is_ascii_alphabetic() || next_char == '_' {
                    // Handle $var format
                    let mut var_name = String::new();
                    
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_alphanumeric() || c == '_' {
                            var_name.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    
                    if let Some(value) = expand_variable(&var_name) {
                        result.push_str(&value);
                    }
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    
    result
}

// --- Function export support ---

fn export_functions(names: &[String]) -> Result<()> {
    if names.is_empty() {
        // Print all exported functions
        for fname in list_functions() {
            println!("declare -fx {}", fname);
        }
        return Ok(());
    }

    for name in names {
        if name.starts_with('-') { break; }
        if !is_valid_var_name(name) {
            return Err(ShellError::new(
                ErrorKind::InvalidArgument,
                format!("Invalid function name: {}", name)
            ).into());
        }
        match get_function(name) {
            Some(_func) => {
                // In Bash, function bodies are exported via FUNCNAME() { ...; }; here we mark by emitting declare -fx
                println!("declare -fx {}", name);
            }
            None => {
                return Err(ShellError::new(
                    ErrorKind::InvalidArgument,
                    format!("Function not found: {}", name)
                ).into());
            }
        }
    }
    Ok(())
}


/// Execute function stub
pub fn execute(_args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}
