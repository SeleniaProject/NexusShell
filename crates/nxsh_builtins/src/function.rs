use std::collections::HashMap;
use nxsh_core::{ShellError, ErrorKind};
use nxsh_core::error::RuntimeErrorKind;

pub fn function_cli(args: &[String]) -> Result<(), ShellError> {
    if args.is_empty() || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    // Basic function definition implementation
    // This is a simplified version - full shell function would require complex parsing
    
    if args.len() < 2 {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "function: missing function name or body"));
    }

    let function_name = &args[0];
    
    // Validate function name
    if !is_valid_function_name(function_name) {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("function: '{function_name}': not a valid identifier")));
    }

    // Find the opening brace
    let brace_pos = args.iter().position(|arg| arg == "{")
        .ok_or_else(|| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "function: missing opening brace '{'"))?;
    
    // Find the closing brace
    let close_brace_pos = args.iter().rposition(|arg| arg == "}")
        .ok_or_else(|| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "function: missing closing brace '}'"))?;
    
    if brace_pos >= close_brace_pos {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "function: invalid syntax - '{' must come before '}'"));
    }
    
    let body_args = &args[brace_pos + 1..close_brace_pos];
    // Basic sanity checks on body_args: disallow stray braces and ensure non-empty commands
    if body_args.is_empty() {
        return Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            "function: empty body is not allowed",
        ));
    }
    if body_args.iter().any(|s| s == "{" || s == "}") {
        return Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            "function: body contains unmatched brace tokens",
        ));
    }
    
    create_function(function_name, body_args)
}

fn print_help() {
    println!("function - define shell function

USAGE:
    function NAME {{ COMMANDS; }}
    NAME() {{ COMMANDS; }}

DESCRIPTION:
    Define a shell function named NAME. When NAME is invoked as a command,
    the COMMANDS are executed in the current shell environment.

PARAMETERS:
    NAME        Function name (must be a valid identifier)
    COMMANDS    The commands that make up the function body

EXAMPLES:
    # Simple function
    function greet {{
        echo \"Hello, World!\"
    }}

    # Function with parameters
    function greet_user {{
        echo \"Hello, $1!\"
    }}

    # Function with local variables
    function calculate {{
        local result=$(($(($1 + $2)) * $3))
        echo \"Result: $result\"
        return $result
    }}

    # Function with conditional logic
    function check_file {{
        if [ -f \"$1\" ]; then
            echo \"File $1 exists\"
            return 0
        else
            echo \"File $1 not found\"
            return 1
        fi
    }}

    # Alternative syntax (POSIX style)
    backup() {{
        local src=\"$1\"
        local dst=\"${{src}}.bak\"
        cp \"$src\" \"$dst\"
        echo \"Backed up $src to $dst\"
    }}

FUNCTION FEATURES:
    - Functions have access to positional parameters ($1, $2, etc.)
    - Functions can use local variables with 'local'
    - Functions can return values with 'return'
    - Functions inherit the environment of the calling shell
    - Functions can be recursive
    - Functions can be exported to subshells

BUILT-IN VARIABLES:
    $0          Name of the function
    $1, $2, ... Function arguments
    $#          Number of arguments
    $@          All arguments as separate words
    $*          All arguments as a single word

EXIT STATUS:
    0   Function defined successfully
    1   Error in function definition");
}

fn is_valid_function_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    
    // Must start with letter or underscore
    let first_char = name.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }
    
    // Rest must be alphanumeric or underscore
    name.chars().skip(1).all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn create_function(name: &str, body: &[String]) -> Result<(), ShellError> {
    let function = ShellFunction {
        name: name.to_string(),
        body: body.to_vec(),
        local_vars: HashMap::new(),
    };
    
    // In a real shell, this would store the function in the global function table
    println!("Function '{}' defined with {} commands", name, body.len());
    
    // Store in global function registry (simplified)
    FUNCTION_REGISTRY.lock().unwrap().insert(name.to_string(), function);
    
    Ok(())
}

// Global function registry (simplified)
use std::sync::Mutex;
use once_cell::sync::Lazy;

static FUNCTION_REGISTRY: Lazy<Mutex<HashMap<String, ShellFunction>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone)]
pub struct ShellFunction {
    pub name: String,
    pub body: Vec<String>,
    pub local_vars: HashMap<String, String>,
}

impl ShellFunction {
    pub fn new(name: String, body: Vec<String>) -> Self {
        Self {
            name,
            body,
            local_vars: HashMap::new(),
        }
    }
    
    pub fn execute(&self, args: &[String]) -> Result<i32, ShellError> {
        println!("Executing function '{}' with {} arguments", self.name, args.len());
        
        // Set up function environment
        let mut function_env = FunctionEnvironment::new(&self.name, args);
        
        // Execute function body
        for command in &self.body {
            if let Err(e) = execute_function_command(command, &mut function_env) {
                let error_msg = format!("{e}");
                if let Some(rest) = error_msg.strip_prefix("RETURN:") {
                    let exit_code: i32 = rest.parse().unwrap_or(1);
                    return Ok(exit_code);
                } else {
                    return Err(e);
                }
            }
        }
        
        Ok(0) // Default exit status
    }
}

pub struct FunctionEnvironment {
    pub function_name: String,
    pub args: Vec<String>,
    pub local_vars: HashMap<String, String>,
}

impl FunctionEnvironment {
    pub fn new(name: &str, args: &[String]) -> Self {
        Self {
            function_name: name.to_string(),
            args: args.to_vec(),
            local_vars: HashMap::new(),
        }
    }
    
    pub fn get_positional_param(&self, index: usize) -> Option<&String> {
        if index == 0 {
            Some(&self.function_name)
        } else if index <= self.args.len() {
            self.args.get(index - 1)
        } else {
            None
        }
    }
    
    pub fn get_param_count(&self) -> usize {
        self.args.len()
    }
    
    pub fn set_local_var(&mut self, name: String, value: String) {
        self.local_vars.insert(name, value);
    }
    
    pub fn get_local_var(&self, name: &str) -> Option<&String> {
        self.local_vars.get(name)
    }
}

fn execute_function_command(command: &str, env: &mut FunctionEnvironment) -> Result<(), ShellError> {
    let parts: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
    
    if parts.is_empty() {
        return Ok(());
    }
    
    // Handle built-in commands
    match parts[0].as_str() {
        "local" => handle_local_command(&parts[1..], env),
        "return" => {
            let exit_code = if parts.len() > 1 {
                parts[1].parse().unwrap_or(0)
            } else {
                0
            };
            Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("RETURN:{exit_code}")))
        },
        "echo" => {
            let output = expand_variables(&parts[1..].join(" "), env);
            println!("{output}");
            Ok(())
        },
        _ => {
            // Execute other commands (simplified)
            println!("Executing in function: {command}");
            Ok(())
        }
    }
}

fn handle_local_command(args: &[String], env: &mut FunctionEnvironment) -> Result<(), ShellError> {
    for arg in args {
        if let Some(eq_pos) = arg.find('=') {
            let var_name = arg[..eq_pos].to_string();
            let var_value = arg[eq_pos + 1..].to_string();
            env.set_local_var(var_name, var_value);
        } else {
            // Just declare the variable as local (unset)
            env.set_local_var(arg.clone(), String::new());
        }
    }
    Ok(())
}

fn expand_variables(input: &str, env: &FunctionEnvironment) -> String {
    let mut result = input.to_string();
    
    // Expand positional parameters
    for i in 0..=9 {
        let param = format!("${i}");
        if let Some(value) = env.get_positional_param(i) {
            result = result.replace(&param, value);
        } else {
            result = result.replace(&param, "");
        }
    }
    
    // Expand $# (argument count)
    result = result.replace("$#", &env.get_param_count().to_string());
    
    // Expand $@ and $* (all arguments)
    result = result.replace("$@", &env.args.join(" "));
    result = result.replace("$*", &env.args.join(" "));
    
    // Expand local variables
    for (name, value) in &env.local_vars {
        result = result.replace(&format!("${name}"), value);
        result = result.replace(&format!("${{{name}}}"), value);
    }
    
    result
}

// Public API for function management
pub fn define_function(name: &str, body: &[String]) -> Result<(), ShellError> {
    if !is_valid_function_name(name) {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Invalid function name: {name}")));
    }
    
    let function = ShellFunction::new(name.to_string(), body.to_vec());
    FUNCTION_REGISTRY.lock().unwrap().insert(name.to_string(), function);
    Ok(())
}

pub fn call_function(name: &str, args: &[String]) -> Result<i32, ShellError> {
    let function = {
        let registry = FUNCTION_REGISTRY.lock().unwrap();
        registry.get(name).cloned()
    };
    
    match function {
        Some(func) => func.execute(args),
        None => Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Function '{name}' not defined"))),
    }
}

pub fn list_functions() -> Vec<String> {
    let registry = FUNCTION_REGISTRY.lock().unwrap();
    registry.keys().cloned().collect()
}

pub fn function_exists(name: &str) -> bool {
    let registry = FUNCTION_REGISTRY.lock().unwrap();
    registry.contains_key(name)
}

pub fn undefine_function(name: &str) -> bool {
    let mut registry = FUNCTION_REGISTRY.lock().unwrap();
    registry.remove(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_function_names() {
        assert!(is_valid_function_name("test"));
        assert!(is_valid_function_name("_test"));
        assert!(is_valid_function_name("test123"));
        assert!(is_valid_function_name("Test_Function_2"));
        
        assert!(!is_valid_function_name("123test"));
        assert!(!is_valid_function_name("test-func"));
        assert!(!is_valid_function_name("test.func"));
        assert!(!is_valid_function_name(""));
    }

    #[test]
    fn test_function_environment() {
        let args = vec!["arg1".to_string(), "arg2".to_string()];
        let mut env = FunctionEnvironment::new("testfunc", &args);
        
        assert_eq!(env.get_positional_param(0), Some(&"testfunc".to_string()));
        assert_eq!(env.get_positional_param(1), Some(&"arg1".to_string()));
        assert_eq!(env.get_positional_param(2), Some(&"arg2".to_string()));
        assert_eq!(env.get_positional_param(3), None);
        assert_eq!(env.get_param_count(), 2);
        
        env.set_local_var("test".to_string(), "value".to_string());
        assert_eq!(env.get_local_var("test"), Some(&"value".to_string()));
    }
}
