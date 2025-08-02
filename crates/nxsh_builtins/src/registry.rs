//! Built-in command registry for efficient command lookup and management

use crate::*;
use nxsh_core::{ShellResult, ExecutionResult, context::Context};
use std::collections::HashMap;

/// Trait for all built-in commands to implement
pub trait Builtin: Send + Sync {
    /// Execute the builtin command with provided arguments and context
    fn execute(&self, args: &[String], ctx: &mut Context) -> ShellResult<ExecutionResult>;
    
    /// Get the command name
    fn name(&self) -> &'static str;
    
    /// Get command description for help
    fn description(&self) -> &'static str;
    
    /// Get command usage information
    fn usage(&self) -> &'static str;
}

/// Registry of all built-in commands for efficient lookup and management
pub struct BuiltinRegistry {
    /// Map of command names to builtin implementations
    commands: HashMap<String, Box<dyn Builtin>>,
}

impl BuiltinRegistry {
    /// Create a new builtin registry with all standard built-in commands
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        
        // Register all standard built-in commands
        registry.register_standard_builtins();
        
        registry
    }
    
    /// Register a new builtin command
    pub fn register(&mut self, builtin: Box<dyn Builtin>) {
        self.commands.insert(builtin.name().to_string(), builtin);
    }
    
    /// Get a builtin command by name
    pub fn get(&self, name: &str) -> Option<&dyn Builtin> {
        self.commands.get(name).map(|b| b.as_ref())
    }
    
    /// Check if a command is a builtin
    pub fn is_builtin(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }
    
    /// Get list of all builtin command names
    pub fn list_commands(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }
    
    /// Execute a builtin command
    pub fn execute(&self, name: &str, args: &[String], ctx: &mut Context) -> ShellResult<ExecutionResult> {
        match self.get(name) {
            Some(builtin) => builtin.execute(args, ctx),
            None => Err(nxsh_core::ShellError::command_not_found(name)),
        }
    }
    
    /// Register all standard built-in commands
    fn register_standard_builtins(&mut self) {
        // This is where we would register all the built-in commands
        // For now, we'll leave this empty and add commands as needed
        // TODO: Add wrapper structs for all builtin functions
    }
}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}
