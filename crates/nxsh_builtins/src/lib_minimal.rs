//! NexusShell Built-in Commands
//!
//! This crate provides a comprehensive collection of built-in commands for NexusShell.

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

// Core file operations
pub mod ls;
pub mod pwd;
pub mod cd;
pub mod touch;
pub mod mkdir;

// Text processing
pub mod cat;
pub mod echo;

// Enhanced features
pub mod smart_alias_simple;
pub mod ui_design;
pub mod alias;

// System utilities
pub mod sleep;
pub mod which;

// Shell features
pub mod history;
pub mod help;

// Utility commands
pub mod clear;

// Built-in management
pub mod builtin;

// Common utilities
pub mod common;

// Re-export commonly used types
pub use common::{BuiltinResult, BuiltinError, BuiltinContext};

/// Check if a command is a built-in
pub fn is_builtin(name: &str) -> bool {
    matches!(name,
        "ls" | "pwd" | "cd" | "touch" | "mkdir" |
        "cat" | "echo" | "sleep" | "which" |
        "history" | "help" | "clear" | "alias" | "smart_alias" | "ui_design"
    )
}

/// List all available built-in commands
pub fn list_builtins() -> Vec<&'static str> {
    vec![
        "ls", "pwd", "cd", "touch", "mkdir",
        "cat", "echo", "sleep", "which",
        "history", "help", "clear", "alias", "smart_alias", "ui_design"
    ]
}

/// Execute a built-in command by name
pub fn execute_builtin(name: &str, args: &[String], context: &BuiltinContext) -> BuiltinResult<i32> {
    match name {
        "ls" => ls::execute(args, context),
        "pwd" => pwd::execute(args, context),
        "cd" => cd::execute(args, context),
        "cat" => cat::execute(args, context),
        "echo" => echo::execute(args, context),
        "touch" => touch::execute(args, context),
        "mkdir" => mkdir::execute(args, context),
        "sleep" => sleep::execute(args, context),
        "which" => which::execute(args, context),
        "clear" => clear::execute(args, context),
        "help" => help::execute(args, context),
        "alias" => alias::execute(args, context),
        "smart_alias" => smart_alias_simple::execute(args, context),
        "ui_design" => ui_design::execute(args, context),
        _ => {
            eprintln!("{}: command not found", name);
            Ok(127)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_builtin() {
        assert!(is_builtin("ls"));
        assert!(is_builtin("echo"));
        assert!(!is_builtin("nonexistent"));
    }

    #[test]
    fn test_list_builtins() {
        let builtins = list_builtins();
        assert!(!builtins.is_empty());
        assert!(builtins.contains(&"ls"));
        assert!(builtins.contains(&"echo"));
    }
}
