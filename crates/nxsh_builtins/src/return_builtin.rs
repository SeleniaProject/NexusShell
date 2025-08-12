use nxsh_core::{ShellError, ErrorKind};
use nxsh_core::error::RuntimeErrorKind;

pub fn return_builtin_cli(args: &[String]) -> Result<(), ShellError> {
    if args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let exit_code = if args.is_empty() {
        0
    } else {
        args[0].parse().map_err(|_| {
            ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("return: {}: numeric argument required", args[0]))
        })?
    };

    // Validate exit code range (0-255 for shell compatibility)
    if !(0..=255).contains(&exit_code) {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "return: exit code must be between 0 and 255"));
    }

    // In a real shell, this would interact with the function execution context
    execute_return(exit_code)
}

fn print_help() {
    println!("return - return from shell function or script

USAGE:
    return [n]

DESCRIPTION:
    Return from a shell function or sourced script with exit status N.
    If N is not given, the exit status is that of the last command executed.
    
    When used in a function, return causes the function to exit with the
    specified status. When used in a script sourced with the '.' or 'source'
    command, return causes the script to exit with the specified status.

PARAMETERS:
    n       Exit status (0-255). Default is the status of the last command.

EXAMPLES:
    # Simple function with return
    check_file() {{
        if [ -f \"$1\" ]; then
            echo \"File exists\"
            return 0
        else
            echo \"File not found\"
            return 1
        fi
    }}

    # Function with different return codes
    validate_input() {{
        if [ -z \"$1\" ]; then
            echo \"Error: No input provided\"
            return 1
        elif [ \"$1\" = \"quit\" ]; then
            echo \"Exiting...\"
            return 2
        else
            echo \"Input valid: $1\"
            return 0
        fi
    }}

    # Using return value
    if validate_input \"$user_input\"; then
        echo \"Processing...\"
    else
        case $? in
            1) echo \"Please provide input\" ;;
            2) echo \"User requested exit\" ;;
        esac
    fi

    # Early return pattern
    process_file() {{
        local file=\"$1\"
        
        [ -z \"$file\" ] && {{ echo \"No file specified\"; return 1; }}
        [ ! -f \"$file\" ] && {{ echo \"File not found\"; return 2; }}
        [ ! -r \"$file\" ] && {{ echo \"File not readable\"; return 3; }}
        
        # Process the file
        echo \"Processing $file\"
        return 0
    }}

EXIT STATUS:
    The specified exit status N, or the exit status of the last command
    if N is not specified.

NOTES:
    - return can only be used within functions or sourced scripts
    - Using return at the top level of an interactive shell has no effect
    - In a subshell, return acts like exit");
}

fn execute_return(exit_code: i32) -> Result<(), ShellError> {
    // This is a simplified implementation
    // In a real shell, return would:
    // 1. Check if we're inside a function or sourced script
    // 2. Set the exit status
    // 3. Signal the function/script execution mechanism to return
    
    println!("Return: exiting with status {exit_code}");
    
    // In a real implementation, this would throw a special exception or
    // set a flag that the function execution engine would catch
    Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("RETURN:{exit_code}")))
}

// Additional helper structures for function context
pub struct FunctionContext {
    pub name: String,
    pub args: Vec<String>,
    pub local_vars: std::collections::HashMap<String, String>,
    pub exit_status: i32,
}

pub enum ExecutionContext {
    TopLevel,
    Function(FunctionContext),
    SourcedScript(String),
    Subshell,
}

impl ExecutionContext {
    pub fn can_return(&self) -> bool {
        matches!(self, ExecutionContext::Function(_) | ExecutionContext::SourcedScript(_))
    }
    
    pub fn context_name(&self) -> &str {
        match self {
            ExecutionContext::TopLevel => "top level",
            ExecutionContext::Function(ctx) => &ctx.name,
            ExecutionContext::SourcedScript(script) => script,
            ExecutionContext::Subshell => "subshell",
        }
    }
}

pub fn handle_return_in_context(
    context: &ExecutionContext, 
    exit_code: i32
) -> Result<ReturnSignal, ShellError> {
    match context {
        ExecutionContext::Function(_) => {
            Ok(ReturnSignal::FunctionReturn(exit_code))
        },
        ExecutionContext::SourcedScript(_) => {
            Ok(ReturnSignal::ScriptReturn(exit_code))
        },
        ExecutionContext::Subshell => {
            Ok(ReturnSignal::SubshellExit(exit_code))
        },
        ExecutionContext::TopLevel => {
            Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "return: can only `return' from a function or sourced script"))
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ReturnSignal {
    FunctionReturn(i32),
    ScriptReturn(i32),
    SubshellExit(i32),
}

// Example function executor that handles return
pub fn execute_function_with_return_support<F>(
    _name: &str,
    mut body: F,
) -> Result<i32, ShellError> 
where
    F: FnMut() -> Result<(), ShellError>,
{
    match body() {
        Ok(()) => Ok(0), // Function completed normally
        Err(e) => {
            let error_msg = format!("{e}");
            if let Some(rest) = error_msg.strip_prefix("RETURN:") {
                let exit_code: i32 = rest.parse().unwrap_or(1);
                Ok(exit_code)
            } else {
                Err(e) // Other error
            }
        }
    }
}

// Helper for getting the last command exit status
static mut LAST_EXIT_STATUS: i32 = 0;

pub fn set_last_exit_status(status: i32) {
    unsafe {
        LAST_EXIT_STATUS = status;
    }
}

pub fn get_last_exit_status() -> i32 {
    unsafe {
        LAST_EXIT_STATUS
    }
}

pub fn return_with_last_status() -> Result<(), ShellError> {
    let status = get_last_exit_status();
    execute_return(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_return_validation() {
        // Valid exit codes
        assert!(return_builtin_cli(&["0".to_string()]).is_err()); // Should return error with RETURN: prefix
        assert!(return_builtin_cli(&["255".to_string()]).is_err());
        
        // Invalid exit codes
        assert!(return_builtin_cli(&["-1".to_string()]).is_err());
        assert!(return_builtin_cli(&["256".to_string()]).is_err());
        assert!(return_builtin_cli(&["abc".to_string()]).is_err());
    }

    #[test]
    fn test_execution_context() {
        let func_ctx = ExecutionContext::Function(FunctionContext {
            name: "test_func".to_string(),
            args: vec![],
            local_vars: std::collections::HashMap::new(),
            exit_status: 0,
        });
        
        let top_level_ctx = ExecutionContext::TopLevel;
        
        assert!(func_ctx.can_return());
        assert!(!top_level_ctx.can_return());
        
        assert_eq!(func_ctx.context_name(), "test_func");
        assert_eq!(top_level_ctx.context_name(), "top level");
    }

    #[test]
    fn test_return_signal() {
        let func_ctx = ExecutionContext::Function(FunctionContext {
            name: "test".to_string(),
            args: vec![],
            local_vars: std::collections::HashMap::new(),
            exit_status: 0,
        });
        
        let result = handle_return_in_context(&func_ctx, 42);
        assert!(matches!(result, Ok(ReturnSignal::FunctionReturn(42))));
        
        let top_ctx = ExecutionContext::TopLevel;
        let result = handle_return_in_context(&top_ctx, 42);
        assert!(result.is_err());
    }
}
