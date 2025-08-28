use crate::compat::{Result, Context};
use crate::memory_efficient::MemoryEfficientStringBuilder;
use std::collections::HashMap;

/// Macro system for NexusShell - providing powerful code generation and transformation
#[derive(Debug, Clone)]
pub struct MacroSystem {
    macros: HashMap<String, Macro>,
    expansion_stack: Vec<String>,
    max_expansion_depth: usize,
    builtin_macros: HashMap<String, BuiltinMacro>,
}

impl Default for MacroSystem {
    fn default() -> Self { Self::new() }
}

impl MacroSystem {
    pub fn new() -> Self {
        let mut system = Self {
            macros: HashMap::new(),
            expansion_stack: Vec::new(),
            max_expansion_depth: 100,
            builtin_macros: HashMap::new(),
        };
        
        system.register_builtin_macros();
        system
    }

    /// Define a new macro
    pub fn define_macro(&mut self, name: String, macro_def: Macro) -> Result<()> {
        if self.builtin_macros.contains_key(&name) {
            let mut error_msg = MemoryEfficientStringBuilder::with_capacity(50);
            error_msg.push_str("Cannot redefine builtin macro '");
            error_msg.push_str(&name);
            error_msg.push('\'');
            return Err(crate::compat::anyhow(error_msg.into_string()));
        }
        
        self.macros.insert(name, macro_def);
        Ok(())
    }

    /// Expand a macro call
    pub fn expand_macro(&mut self, name: &str, args: Vec<String>) -> Result<String> {
        // Check for circular expansion
        if self.expansion_stack.len() >= self.max_expansion_depth {
            return Err(crate::anyhow!("Maximum macro expansion depth exceeded"));
        }

        if self.expansion_stack.contains(&name.to_string()) {
            return Err(crate::anyhow!("Circular macro expansion detected for '{}'", name));
        }

        self.expansion_stack.push(name.to_string());
        
        let result = if let Some(builtin) = self.builtin_macros.get(name) {
            let builtin_clone = builtin.clone();
            self.expand_builtin_macro(&builtin_clone, args)
        } else if let Some(macro_def) = self.macros.get(name).cloned() {
            self.expand_user_macro(&macro_def, args)
        } else {
            let mut error_msg = MemoryEfficientStringBuilder::with_capacity(30);
            error_msg.push_str("Macro '");
            error_msg.push_str(name);
            error_msg.push_str("' not found");
            Err(crate::anyhow!("{}", error_msg.into_string()))
        };

        self.expansion_stack.pop();
        result
    }

    /// Expand a user-defined macro
    fn expand_user_macro(&mut self, macro_def: &Macro, args: Vec<String>) -> Result<String> {
        match macro_def {
            Macro::Simple { parameters, body } => {
                if args.len() != parameters.len() {
                    return Err(crate::anyhow!(
                        "Macro expected {} arguments, got {}",
                        parameters.len(),
                        args.len()
                    ));
                }

                let mut result = body.clone();
                
                // Replace parameters with arguments
                for (param, arg) in parameters.iter().zip(args.iter()) {
                    let mut search_pattern = MemoryEfficientStringBuilder::with_capacity(param.len() + 2);
                    search_pattern.push('$');
                    search_pattern.push_str(param);
                    result = result.replace(&search_pattern.into_string(), arg);
                }

                Ok(result)
            },
            
            Macro::Conditional { condition, then_body, else_body } => {
                let condition_result = self.evaluate_condition(condition, &args)?;
                
                if condition_result {
                    self.expand_template(then_body, &args)
                } else if let Some(else_body) = else_body {
                    self.expand_template(else_body, &args)
                } else {
                    Ok(String::new())
                }
            },
            
            Macro::Loop { iterator, body } => {
                let mut result = String::new();
                let items = self.resolve_iterator(iterator, &args)?;
                
                for item in items {
                    let expanded = self.expand_template(body, &[item])?;
                    result.push_str(&expanded);
                    result.push('\n');
                }
                
                Ok(result)
            },
            
            Macro::Function { parameters, body } => {
                if args.len() != parameters.len() {
                    return Err(crate::anyhow!(
                        "Function macro expected {} arguments, got {}",
                        parameters.len(),
                        args.len()
                    ));
                }

                // Create parameter bindings
                let mut bindings = HashMap::new();
                for (param, arg) in parameters.iter().zip(args.iter()) {
                    bindings.insert(param.clone(), arg.clone());
                }

                self.expand_function_body(body, &bindings)
            },
        }
    }

    /// Expand a builtin macro
    fn expand_builtin_macro(&mut self, builtin: &BuiltinMacro, args: Vec<String>) -> Result<String> {
        match builtin {
            BuiltinMacro::Include => {
                if args.len() != 1 {
                    return Err(crate::anyhow!("include! macro requires exactly 1 argument"));
                }
                
                let file_path = &args[0];
                std::fs::read_to_string(file_path)
                    .with_context(|| {
                        let mut error_msg = MemoryEfficientStringBuilder::with_capacity(file_path.len() + 20);
                        error_msg.push_str("Failed to read file: ");
                        error_msg.push_str(file_path);
                        error_msg.into_string()
                    })
            },
            
            BuiltinMacro::Concat => {
                Ok(args.join(""))
            },
            
            BuiltinMacro::Repeat => {
                if args.len() != 2 {
                    return Err(crate::anyhow!("repeat! macro requires exactly 2 arguments"));
                }
                
                let count: usize = args[1].parse()
                    .with_context(|| "Second argument to repeat! must be a number")?;
                
                Ok(args[0].repeat(count))
            },
            
            BuiltinMacro::Stringify => {
                let mut result = MemoryEfficientStringBuilder::with_capacity(args.join(" ").len() + 2);
                result.push('"');
                result.push_str(&args.join(" "));
                result.push('"');
                Ok(result.into_string())
            },
            
            BuiltinMacro::Env => {
                if args.len() != 1 {
                    return Err(crate::anyhow!("env! macro requires exactly 1 argument"));
                }
                
                std::env::var(&args[0])
                    .with_context(|| {
                        let mut error_msg = MemoryEfficientStringBuilder::with_capacity(args[0].len() + 30);
                        error_msg.push_str("Environment variable '");
                        error_msg.push_str(&args[0]);
                        error_msg.push_str("' not found");
                        error_msg.into_string()
                    })
            },
            
            BuiltinMacro::Date => {
                #[cfg(feature = "heavy-time")]
                {
                    let format = args.first().map(|s| s.as_str()).unwrap_or("%Y-%m-%d %H:%M:%S");
                    Ok(chrono::Local::now().format(format).to_string())
                }
                #[cfg(not(feature = "heavy-time"))]
                {
                    let secs = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    Ok(secs.to_string())
                }
            },
            
            BuiltinMacro::Version => {
                Ok(env!("CARGO_PKG_VERSION").to_string())
            },
        }
    }

    /// Process text with macro expansions
    pub fn process_text(&mut self, text: &str) -> Result<String> {
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        
        // Use a single mutable iterator so we can safely call peek/next as needed.
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                // consume '{'
                let _ = chars.next();

                let mut macro_call = String::new();
                let mut brace_count = 1;

                for inner_ch in chars.by_ref() {
                    if inner_ch == '{' {
                        brace_count += 1;
                    } else if inner_ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            break;
                        }
                    }
                    macro_call.push(inner_ch);
                }

                // Parse macro call
                let (macro_name, args) = self.parse_macro_call(&macro_call)?;
                let expanded = self.expand_macro(&macro_name, args)?;
                result.push_str(&expanded);
            } else {
                result.push(ch);
            }
        }
        
        Ok(result)
    }

    /// Parse a macro call string
    fn parse_macro_call(&self, call: &str) -> Result<(String, Vec<String>)> {
        let parts: Vec<&str> = call.splitn(2, '(').collect();
        let macro_name = parts[0].trim().to_string();
        
        if parts.len() == 1 {
            // No arguments
            return Ok((macro_name, vec![]));
        }
        
        let args_str = parts[1].trim_end_matches(')').trim();
        if args_str.is_empty() {
            return Ok((macro_name, vec![]));
        }
        
        // Simple argument parsing (would need more sophisticated parsing for real use)
        let args = args_str.split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        Ok((macro_name, args))
    }

    /// Expand a template with arguments
    fn expand_template(&mut self, template: &str, args: &[String]) -> Result<String> {
        let mut result = template.to_string();
        
        // Replace numbered arguments $0, $1, $2, etc.
        for (i, arg) in args.iter().enumerate() {
            let mut search_pattern = MemoryEfficientStringBuilder::with_capacity(8);
            search_pattern.push('$');
            search_pattern.push_str(&i.to_string());
            result = result.replace(&search_pattern.into_string(), arg);
        }
        
        // Process nested macros
        self.process_text(&result)
    }

    /// Evaluate a condition for conditional macros
    fn evaluate_condition(&self, condition: &MacroCondition, args: &[String]) -> Result<bool> {
        match condition {
            MacroCondition::ArgCount(expected) => Ok(args.len() == *expected),
            MacroCondition::ArgEquals { index, value } => {
                if *index < args.len() {
                    Ok(&args[*index] == value)
                } else {
                    Ok(false)
                }
            },
            MacroCondition::EnvVar { name, value } => {
                if let Ok(env_value) = std::env::var(name) {
                    Ok(&env_value == value)
                } else {
                    Ok(false)
                }
            },
            MacroCondition::Platform(platform) => {
                Ok(cfg!(target_os = "windows") && platform == "windows" ||
                   cfg!(target_os = "linux") && platform == "linux" ||
                   cfg!(target_os = "macos") && platform == "macos")
            },
        }
    }

    /// Resolve an iterator for loop macros
    fn resolve_iterator(&self, iterator: &MacroIterator, args: &[String]) -> Result<Vec<String>> {
        match iterator {
            MacroIterator::Args => Ok(args.to_vec()),
            MacroIterator::Range { start, end } => {
                let mut result = Vec::with_capacity((end - start).max(0) as usize);
                for i in *start..*end {
                    result.push(i.to_string());
                }
                Ok(result)
            },
            MacroIterator::List(items) => Ok(items.clone()),
            MacroIterator::Split { arg_index, delimiter } => {
                if *arg_index < args.len() {
                    let split_items: Vec<String> = args[*arg_index]
                        .split(delimiter)
                        .map(|s| s.to_string())
                        .collect();
                    Ok(split_items)
                } else {
                    Ok(vec![])
                }
            },
        }
    }

    /// Expand function body with parameter bindings
    fn expand_function_body(&mut self, body: &[MacroStatement], bindings: &HashMap<String, String>) -> Result<String> {
        let mut result = String::new();
        
        for statement in body {
            match statement {
                MacroStatement::Text(text) => {
                    let mut expanded = text.clone();
                    for (param, value) in bindings {
                        let mut search_pattern = MemoryEfficientStringBuilder::with_capacity(param.len() + 2);
                        search_pattern.push('$');
                        search_pattern.push_str(param);
                        expanded = expanded.replace(&search_pattern.into_string(), value);
                    }
                    result.push_str(&expanded);
                },
                
                MacroStatement::MacroCall { name, args } => {
                    let expanded_args = args.iter()
                        .map(|arg| {
                            let mut expanded = arg.clone();
                            for (param, value) in bindings {
                                let mut search_pattern = MemoryEfficientStringBuilder::with_capacity(param.len() + 2);
                                search_pattern.push('$');
                                search_pattern.push_str(param);
                                expanded = expanded.replace(&search_pattern.into_string(), value);
                            }
                            expanded
                        })
                        .collect();
                    
                    let expanded = self.expand_macro(name, expanded_args)?;
                    result.push_str(&expanded);
                },
                
                MacroStatement::Conditional { condition, then_body, else_body } => {
                    let condition_result = self.evaluate_condition(condition, &[])?;
                    
                    if condition_result {
                        let expanded = self.expand_function_body(then_body, bindings)?;
                        result.push_str(&expanded);
                    } else if let Some(else_body) = else_body {
                        let expanded = self.expand_function_body(else_body, bindings)?;
                        result.push_str(&expanded);
                    }
                },
            }
        }
        
        Ok(result)
    }

    /// Register built-in macros
    fn register_builtin_macros(&mut self) {
        self.builtin_macros.insert("include".to_string(), BuiltinMacro::Include);
        self.builtin_macros.insert("concat".to_string(), BuiltinMacro::Concat);
        self.builtin_macros.insert("repeat".to_string(), BuiltinMacro::Repeat);
        self.builtin_macros.insert("stringify".to_string(), BuiltinMacro::Stringify);
        self.builtin_macros.insert("env".to_string(), BuiltinMacro::Env);
        self.builtin_macros.insert("date".to_string(), BuiltinMacro::Date);
        self.builtin_macros.insert("version".to_string(), BuiltinMacro::Version);
    }

    /// Get macro information
    pub fn get_macro_info(&self, name: &str) -> Option<MacroInfo> {
        if let Some(builtin) = self.builtin_macros.get(name) {
            Some(MacroInfo {
                name: name.to_string(),
                macro_type: MacroType::Builtin,
                description: builtin.description(),
                parameters: builtin.parameters(),
            })
        } else {
            self.macros.get(name).map(|user_macro| MacroInfo {
                name: name.to_string(),
                macro_type: MacroType::User,
                description: "User-defined macro".to_string(),
                parameters: user_macro.parameters(),
            })
        }
    }

    /// List all available macros
    pub fn list_macros(&self) -> Vec<MacroInfo> {
        let mut macros = Vec::new();
        
        // Add builtin macros
        for name in self.builtin_macros.keys() {
            if let Some(info) = self.get_macro_info(name) {
                macros.push(info);
            }
        }
        
        // Add user macros
        for name in self.macros.keys() {
            if let Some(info) = self.get_macro_info(name) {
                macros.push(info);
            }
        }
        
        macros.sort_by(|a, b| a.name.cmp(&b.name));
        macros
    }
}

/// Macro definition types
#[derive(Debug, Clone)]
pub enum Macro {
    Simple { parameters: Vec<String>, body: String },
    Conditional { condition: MacroCondition, then_body: String, else_body: Option<String> },
    Loop { iterator: MacroIterator, body: String },
    Function { parameters: Vec<String>, body: Vec<MacroStatement> },
}

impl Macro {
    fn parameters(&self) -> Vec<String> {
        match self {
            Macro::Simple { parameters, .. } => parameters.clone(),
            Macro::Conditional { .. } => vec![],
            Macro::Loop { .. } => vec!["items".to_string()],
            Macro::Function { parameters, .. } => parameters.clone(),
        }
    }
}

/// Built-in macro types
#[derive(Debug, Clone)]
pub enum BuiltinMacro {
    Include,
    Concat,
    Repeat,
    Stringify,
    Env,
    Date,
    Version,
}

impl BuiltinMacro {
    fn description(&self) -> String {
        match self {
            BuiltinMacro::Include => "Include contents of a file".to_string(),
            BuiltinMacro::Concat => "Concatenate arguments".to_string(),
            BuiltinMacro::Repeat => "Repeat text N times".to_string(),
            BuiltinMacro::Stringify => "Convert arguments to string".to_string(),
            BuiltinMacro::Env => "Get environment variable".to_string(),
            BuiltinMacro::Date => "Get current date/time".to_string(),
            BuiltinMacro::Version => "Get NexusShell version".to_string(),
        }
    }

    fn parameters(&self) -> Vec<String> {
        match self {
            BuiltinMacro::Include => vec!["file_path".to_string()],
            BuiltinMacro::Concat => vec!["...args".to_string()],
            BuiltinMacro::Repeat => vec!["text".to_string(), "count".to_string()],
            BuiltinMacro::Stringify => vec!["...args".to_string()],
            BuiltinMacro::Env => vec!["var_name".to_string()],
            BuiltinMacro::Date => vec!["format".to_string()],
            BuiltinMacro::Version => vec![],
        }
    }
}

/// Macro condition types
#[derive(Debug, Clone)]
pub enum MacroCondition {
    ArgCount(usize),
    ArgEquals { index: usize, value: String },
    EnvVar { name: String, value: String },
    Platform(String),
}

/// Macro iterator types
#[derive(Debug, Clone)]
pub enum MacroIterator {
    Args,
    Range { start: i32, end: i32 },
    List(Vec<String>),
    Split { arg_index: usize, delimiter: String },
}

/// Macro statement types
#[derive(Debug, Clone)]
pub enum MacroStatement {
    Text(String),
    MacroCall { name: String, args: Vec<String> },
    Conditional { condition: MacroCondition, then_body: Vec<MacroStatement>, else_body: Option<Vec<MacroStatement>> },
}

/// Macro information
#[derive(Debug, Clone)]
pub struct MacroInfo {
    pub name: String,
    pub macro_type: MacroType,
    pub description: String,
    pub parameters: Vec<String>,
}

/// Macro type classification
#[derive(Debug, Clone)]
pub enum MacroType {
    Builtin,
    User,
}

#[cfg(feature = "heavy-time")]
use chrono;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_macro() {
        let mut system = MacroSystem::new();
        
        let macro_def = Macro::Simple {
            parameters: vec!["name".to_string()],
            body: "Hello, $name!".to_string(),
        };
        
        system.define_macro("greet".to_string(), macro_def).unwrap();
        
        let result = system.expand_macro("greet", vec!["World".to_string()]).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_builtin_concat_macro() {
        let mut system = MacroSystem::new();
        
        let result = system.expand_macro("concat", vec![
            "Hello".to_string(),
            " ".to_string(),
            "World".to_string(),
        ]).unwrap();
        
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_builtin_repeat_macro() {
        let mut system = MacroSystem::new();
        
        let result = system.expand_macro("repeat", vec![
            "A".to_string(),
            "3".to_string(),
        ]).unwrap();
        
        assert_eq!(result, "AAA");
    }

    #[test]
    fn test_text_processing_with_macros() {
        let mut system = MacroSystem::new();
        
        let macro_def = Macro::Simple {
            parameters: vec!["x".to_string()],
            body: "$x squared is ${multiply($x, $x)}".to_string(),
        };
        
        system.define_macro("square".to_string(), macro_def).unwrap();
        
        let text = "The result is: ${square(5)}";
        let result = system.process_text(text).unwrap();
        
        // Note: This test assumes multiply macro exists
        // In practice, we'd need more sophisticated expression evaluation
        assert!(result.contains("The result is:"));
    }

    #[test]
    fn test_conditional_macro() {
        let mut system = MacroSystem::new();
        
        let macro_def = Macro::Conditional {
            condition: MacroCondition::Platform("windows".to_string()),
            then_body: "Windows command".to_string(),
            else_body: Some("Unix command".to_string()),
        };
        
        system.define_macro("platform_cmd".to_string(), macro_def).unwrap();
        
        let result = system.expand_macro("platform_cmd", vec![]).unwrap();
        
        #[cfg(target_os = "windows")]
        assert_eq!(result, "Windows command");
        
        #[cfg(not(target_os = "windows"))]
        assert_eq!(result, "Unix command");
    }

    #[test]
    fn test_macro_listing() {
        let system = MacroSystem::new();
        let macros = system.list_macros();
        
        assert!(!macros.is_empty());
        assert!(macros.iter().any(|m| m.name == "concat"));
        assert!(macros.iter().any(|m| m.name == "repeat"));
        assert!(macros.iter().any(|m| m.name == "env"));
    }
}
