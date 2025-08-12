//! `alias` builtin command - manage command aliases
//!
//! This module implements the alias builtin command for creating
//! and managing command aliases with cycle detection.

use crate::common::{i18n::*, logging::*};
use std::io::Write;
use std::collections::HashMap;
use nxsh_core::{Builtin, Context, ExecutionResult, ShellResult, ShellError, ErrorKind, StreamData, context::ShellContext};
use nxsh_core::error::{RuntimeErrorKind, IoErrorKind, InternalErrorKind};

/// The `alias` builtin command implementation
pub struct AliasCommand;

impl Builtin for AliasCommand {
    fn name(&self) -> &'static str {
        "alias"
    }

    fn synopsis(&self) -> &'static str {
        "Define or display aliases"
    }

    fn description(&self) -> &'static str {
        "Define aliases for commands or display existing aliases."
    }

    fn usage(&self) -> &'static str {
        "alias [-p] [name[=value] ...]"
    }

    fn affects_shell_state(&self) -> bool {
        true // alias modifies shell aliases
    }

    fn help(&self) -> &'static str {
        "Define or display aliases. Use 'alias --help' for detailed usage information."
    }

    // The legacy Context shim has been removed; implementation below uses ShellContext directly.
    fn execute(&self, ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        let mut print_all = false;
        let mut assignments = Vec::new();
        let mut queries = Vec::new();

        // Parse arguments
        let mut i = 0; // Start from 0 since args doesn't include command name
        while i < args.len() {
            let arg = &args[i];
            
            if arg == "-p" {
                print_all = true;
            } else if arg.starts_with('-') {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("alias: invalid option: {}", arg)
                ));
            } else if arg.contains('=') {
                assignments.push(arg.clone());
            } else {
                queries.push(arg.clone());
            }
            
            i += 1;
        }

        // If no arguments or -p flag, print all aliases
        if (assignments.is_empty() && queries.is_empty()) || print_all {
            self.print_all_aliases(ctx)?;
            return Ok(ExecutionResult::success(0));
        }

        // Process alias assignments
        for assignment in assignments {
            self.process_assignment(&assignment, ctx)?;
        }

        // Process alias queries
        for query in queries {
            self.print_alias(&query, ctx)?;
        }

        Ok(ExecutionResult::success(0))
    }

    fn help(&self) -> String {
        "alias - define or display aliases".to_string()
    }

    fn usage(&self) -> String {
        "alias [-p] [name[=value] ...]".to_string()
    }
}

impl AliasCommand {
    /// Create a new alias command instance
    pub fn new() -> Self {
        Self
    }

    /// Process an alias assignment
    fn process_assignment(&self, assignment: &str, ctx: &mut ShellContext) -> ShellResult<()> {
        if let Some(eq_pos) = assignment.find('=') {
            let name = &assignment[..eq_pos];
            let value = &assignment[eq_pos + 1..];

            // Validate alias name
            if !self.is_valid_alias_name(name) {
                return Err(ShellError::new(
                    nxsh_core::error::ErrorKind::RuntimeError(nxsh_core::error::RuntimeErrorKind::InvalidArgument),
                    format!("alias: `{}': invalid alias name", name)
                ));
            }

            // Check for cycles before setting the alias
            if self.would_create_cycle(name, value, ctx) {
                return Err(ShellError::new(
                    nxsh_core::error::ErrorKind::RuntimeError(nxsh_core::error::RuntimeErrorKind::InvalidArgument),
                    format!("alias: `{}': would create a cycle", name)
                ));
            }

            // Set the alias
            ctx.aliases.write().unwrap().insert(name.to_string(), value.to_string());
        } else {
            return Err(ShellError::new(
                nxsh_core::error::ErrorKind::RuntimeError(nxsh_core::error::RuntimeErrorKind::InvalidArgument),
                format!("alias: invalid assignment: {}", assignment)
            ));
        }

        Ok(())
    }

    /// Print a specific alias
    fn print_alias(&self, name: &str, ctx: &mut ShellContext) -> ShellResult<()> {
        if let Some(value) = ctx.aliases.read().unwrap().get(name) {
            let output = format!("alias {}='{}'\n", name, self.escape_value(&value));
            ctx.stdout.write(nxsh_core::stream::StreamData::Text(output))
                .map_err(|e| ShellError::new(nxsh_core::error::ErrorKind::IoError(nxsh_core::error::IoErrorKind::FileWriteError), format!("Failed to write output: {}", e)))?;
        } else {
            return Err(ShellError::new(
                nxsh_core::error::ErrorKind::RuntimeError(nxsh_core::error::RuntimeErrorKind::VariableNotFound),
                format!("alias: {}: not found", name)
            ));
        }

        Ok(())
    }

        Ok(())
    }

    /// Print all aliases
    fn print_all_aliases(&self, ctx: &mut ShellContext) -> ShellResult<()> {
        let mut output = String::new();
        
        if let Ok(aliases_lock) = ctx.aliases.read() {
            let mut aliases: Vec<_> = aliases_lock.iter().collect();
            
            // Sort aliases by name for consistent output
            aliases.sort_by(|a, b| a.0.cmp(b.0));

            for (name, value) in aliases {
                output.push_str(&format!("alias {}='{}'\n", name, self.escape_value(value)));
            }
        } else {
            return Err(ShellError::new(
                nxsh_core::error::ErrorKind::InternalError(nxsh_core::error::InternalErrorKind::LockError),
                "Failed to read aliases"
            ));
        }

        if !output.is_empty() {
            ctx.stdout.write(nxsh_core::stream::StreamData::Text(output))
                .map_err(|e| ShellError::new(nxsh_core::error::ErrorKind::IoError(nxsh_core::error::IoErrorKind::FileWriteError), format!("Failed to write output: {}", e)))?;
        }

        Ok(())
    }

    /// Check if setting an alias would create a cycle
    fn would_create_cycle(&self, name: &str, value: &str, ctx: &ShellContext) -> bool {
        // Simple cycle detection: check if the alias value starts with the alias name
        // This is a basic implementation - more sophisticated cycle detection could be added
        
        let value_parts: Vec<&str> = value.split_whitespace().collect();
        if let Some(first_word) = value_parts.first() {
            if *first_word == name {
                return true; // Direct self-reference
            }
            
            // Check for indirect cycles by following the alias chain
            let mut visited = std::collections::HashSet::new();
            let mut current = first_word.to_string();
            
            while let Some(alias_value) = ctx.aliases.read().unwrap().get(&current) {
                if visited.contains(&current) {
                    return true; // Cycle detected
                }
                visited.insert(current.clone());
                
                // Get the first word of the alias value
                let alias_parts: Vec<&str> = alias_value.split_whitespace().collect();
                if let Some(next_word) = alias_parts.first() {
                    if *next_word == name {
                        return true; // Would create cycle
                    }
                    current = next_word.to_string();
                } else {
                    break;
                }
                
                // Prevent infinite loops with a reasonable limit
                if visited.len() > 100 {
                    return true;
                }
            }
        }
        
        false
    }

    /// Validate alias name
    fn is_valid_alias_name(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Alias names can contain more characters than variable names
        // but should not contain shell metacharacters
        for ch in name.chars() {
            match ch {
                // Disallow shell metacharacters
                '|' | '&' | ';' | '(' | ')' | '<' | '>' | ' ' | '\t' | '\n' | '\r' |
                '"' | '\'' | '\\' | '$' | '`' | '*' | '?' | '[' | ']' | '{' | '}' |
                '~' | '#' => return false,
                _ => {}
            }
        }

        true
    }

    /// Escape special characters in alias values for display
    fn escape_value(&self, value: &str) -> String {
        let mut result = String::new();
        
        for ch in value.chars() {
            match ch {
                '\'' => result.push_str("'\"'\"'"), // End quote, escaped quote, start quote
                '\\' => result.push_str("\\\\"),
                _ => result.push(ch),
            }
        }
        
        result
    }

    /// Expand an alias if it exists
    pub fn expand_alias(name: &str, ctx: &Context) -> Option<String> {
        ctx.env.get_alias(name)
    }

    /// Check if a command name is an alias
    pub fn is_alias(name: &str, ctx: &Context) -> bool {
        ctx.env.get_alias(name).is_some()
    }
}

impl Default for AliasCommand {
    fn default() -> Self {
        Self::new()
    }
}
