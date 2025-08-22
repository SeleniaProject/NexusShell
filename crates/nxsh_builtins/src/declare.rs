use std::collections::HashMap;
use nxsh_core::{ShellError, ErrorKind}; use nxsh_core::error::RuntimeErrorKind;

pub fn declare_cli(args: &[String]) -> Result<(), ShellError> {
    if args.is_empty() || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let mut options = DeclareOptions::default();
    let mut variables = Vec::new();
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" => options.array = true,
            "-A" => options.associative = true,
            "-f" => options.function = true,
            "-F" => options.function_names_only = true,
            "-g" => options.global = true,
            "-i" => options.integer = true,
            "-l" => options.lowercase = true,
            "-n" => options.nameref = true,
            "-r" => options.readonly = true,
            "-t" => options.trace = true,
            "-u" => options.uppercase = true,
            "-x" => options.export = true,
            "-p" => options.print = true,
            arg if !arg.starts_with('-') => {
                variables.push(arg.to_string());
            },
            _ => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("declare: {}: invalid option", args[i]))),
        }
        i += 1;
    }
    
    execute_declare(&options, &variables)
}

#[derive(Default, Debug)]
struct DeclareOptions {
    array: bool,
    associative: bool,
    function: bool,
    function_names_only: bool,
    global: bool,
    integer: bool,
    lowercase: bool,
    nameref: bool,
    readonly: bool,
    trace: bool,
    uppercase: bool,
    export: bool,
    print: bool,
}

fn print_help() {
    println!("declare - declare variables and give them attributes

USAGE:
    declare [-aAfFgilnrtux] [-p] [name[=value] ...]

DESCRIPTION:
    Declare variables and give them attributes. If no NAMEs are given,
    display the attributes and values of all variables.

OPTIONS:
    -a      Each NAME is an indexed array variable
    -A      Each NAME is an associative array variable  
    -f      Use function names only
    -F      Display function names without definitions
    -g      Create global variables when used in a shell function
    -i      The variable is to be treated as an integer
    -l      Convert NAMEs to lower case on assignment
    -n      Each NAME is a nameref and will be assigned to by reference
    -p      Display the attributes and value of each NAME
    -r      Make NAMEs readonly
    -t      Give each NAME the trace attribute
    -u      Convert NAMEs to upper case on assignment
    -x      Mark NAMEs for export to subsequent commands

EXAMPLES:
    # Declare variables with attributes
    declare -i count=0
    declare -r readonly_var=\"constant\"
    declare -x export_var=\"exported\"
    
    # Declare arrays
    declare -a indexed_array
    declare -A associative_array
    
    # Display all variables
    declare -p
    
    # Display specific variable
    declare -p my_var
    
    # Function-related
    declare -f              # Show all function definitions
    declare -F              # Show all function names only

EXIT STATUS:
    0   Success
    1   Invalid option or assignment error");
}

fn execute_declare(options: &DeclareOptions, variables: &[String]) -> Result<(), ShellError> {
    if options.print {
        return print_declarations(variables);
    }
    
    if options.function || options.function_names_only {
        return handle_function_declarations(options);
    }
    
    if variables.is_empty() {
        return print_all_declarations();
    }
    
    for var_spec in variables {
        process_variable_declaration(var_spec, options)?;
    }
    
    Ok(())
}

fn print_declarations(variables: &[String]) -> Result<(), ShellError> {
    if variables.is_empty() {
        return print_all_declarations();
    }
    
    for var_name in variables {
        if let Some(var_info) = get_variable_info(var_name) {
            println!("{}", format_variable_declaration(&var_info));
        } else {
            eprintln!("declare: {var_name}: not found");
        }
    }
    
    Ok(())
}

fn print_all_declarations() -> Result<(), ShellError> {
    let variables = get_all_variables();
    for var_info in variables {
        println!("{}", format_variable_declaration(&var_info));
    }
    Ok(())
}

fn handle_function_declarations(options: &DeclareOptions) -> Result<(), ShellError> {
    // Get all function names from the function registry
    let functions = crate::function::list_functions();
    
    if options.function_names_only {
        for func_name in functions {
            println!("declare -f {func_name}");
        }
    } else {
        for func_name in functions {
            println!("declare -f {func_name}");
            println!("{func_name} () {{");
            println!("    # Function body would be displayed here");
            println!("}}");
        }
    }
    
    Ok(())
}

fn process_variable_declaration(var_spec: &str, options: &DeclareOptions) -> Result<(), ShellError> {
    let (var_name, var_value) = parse_variable_assignment(var_spec)?;
    
    let mut var_info = VariableInfo {
        name: var_name.clone(),
        value: var_value.unwrap_or_default(),
        attributes: VariableAttributes::default(),
    };
    
    // Apply attributes
    var_info.attributes.array = options.array;
    var_info.attributes.associative = options.associative;
    var_info.attributes.integer = options.integer;
    var_info.attributes.lowercase = options.lowercase;
    var_info.attributes.readonly = options.readonly;
    var_info.attributes.uppercase = options.uppercase;
    var_info.attributes.export = options.export;
    
    // Apply transformations
    if options.lowercase {
        var_info.value = var_info.value.to_lowercase();
    } else if options.uppercase {
        var_info.value = var_info.value.to_uppercase();
    }
    
    // Validate integer assignment
    if options.integer && !var_info.value.is_empty()
        && var_info.value.parse::<i64>().is_err() {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("declare: {}: invalid integer value", var_info.value)));
        }
    
    // Store the variable
    set_variable(var_info)?;
    
    Ok(())
}

fn parse_variable_assignment(spec: &str) -> Result<(String, Option<String>), ShellError> {
    if let Some(eq_pos) = spec.find('=') {
        let name = spec[..eq_pos].to_string();
        let value = spec[eq_pos + 1..].to_string();
        
        if !is_valid_variable_name(&name) {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("declare: '{name}': not a valid identifier")));
        }
        
        Ok((name, Some(value)))
    } else {
        if !is_valid_variable_name(spec) {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("declare: '{spec}': not a valid identifier")));
        }
        
        Ok((spec.to_string(), None))
    }
}

fn is_valid_variable_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    
    let first_char = name.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }
    
    name.chars().skip(1).all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[derive(Debug, Clone, Default)]
pub struct VariableInfo {
    pub name: String,
    pub value: String,
    pub attributes: VariableAttributes,
}

#[derive(Debug, Clone, Default)]
pub struct VariableAttributes {
    pub array: bool,
    pub associative: bool,
    pub integer: bool,
    pub lowercase: bool,
    pub readonly: bool,
    pub uppercase: bool,
    pub export: bool,
}

impl VariableAttributes {
    fn to_flags(&self) -> String {
        let mut flags = String::new();
        
        if self.array { flags.push('a'); }
        if self.associative { flags.push('A'); }
        if self.integer { flags.push('i'); }
        if self.lowercase { flags.push('l'); }
        if self.readonly { flags.push('r'); }
        if self.uppercase { flags.push('u'); }
        if self.export { flags.push('x'); }
        
        flags
    }
}

fn format_variable_declaration(var_info: &VariableInfo) -> String {
    let flags = var_info.attributes.to_flags();
    if flags.is_empty() {
        format!("declare -- {}=\"{}\"", var_info.name, var_info.value)
    } else {
        format!("declare -{} {}=\"{}\"", flags, var_info.name, var_info.value)
    }
}

// Global variable storage (simplified)
use std::sync::Mutex;
use once_cell::sync::Lazy;

static VARIABLE_REGISTRY: Lazy<Mutex<HashMap<String, VariableInfo>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

fn set_variable(var_info: VariableInfo) -> Result<(), ShellError> {
    let mut registry = VARIABLE_REGISTRY.lock().unwrap();
    
    // Check if variable is readonly
    if let Some(existing) = registry.get(&var_info.name) {
        if existing.attributes.readonly {
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("declare: {}: readonly variable", var_info.name)));
        }
    }
    
    registry.insert(var_info.name.clone(), var_info);
    Ok(())
}

fn get_variable_info(name: &str) -> Option<VariableInfo> {
    let registry = VARIABLE_REGISTRY.lock().unwrap();
    registry.get(name).cloned()
}

fn get_all_variables() -> Vec<VariableInfo> {
    let registry = VARIABLE_REGISTRY.lock().unwrap();
    registry.values().cloned().collect()
}

// Public API
pub fn declare_variable(name: &str, value: &str, attributes: VariableAttributes) -> Result<(), ShellError> {
    let var_info = VariableInfo {
        name: name.to_string(),
        value: value.to_string(),
        attributes,
    };
    
    set_variable(var_info)
}

pub fn get_variable_value(name: &str) -> Option<String> {
    get_variable_info(name).map(|info| info.value)
}

pub fn is_variable_readonly(name: &str) -> bool {
    get_variable_info(name)
        .map(|info| info.attributes.readonly)
        .unwrap_or(false)
}

pub fn is_variable_exported(name: &str) -> bool {
    get_variable_info(name)
        .map(|info| info.attributes.export)
        .unwrap_or(false)
}



/// Execute function stub
pub fn execute(_args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}
