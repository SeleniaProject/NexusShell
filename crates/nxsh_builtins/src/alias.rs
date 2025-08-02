//! `alias` builtin command - manage command aliases
//!
//! This module implements the alias builtin command for creating
//! and managing command aliases with cycle detection.

use crate::common::{i18n::*, logging::*};
use std::io::Write;
use std::collections::HashMap;
use nxsh_core::{Builtin, Context, ExecutionResult, ShellResult, ShellError, ErrorKind, StreamData};
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

    fn invoke(&self, ctx: &mut Context) -> ShellResult<ExecutionResult> {
        let mut print_all = false;
        let mut assignments = Vec::new();
        let mut queries = Vec::new();

        // Parse arguments
        let mut i = 1; // Skip command name
        while i < ctx.args.len() {
            let arg = &ctx.args[i];
            
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
            return self.print_all_aliases(ctx);
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
}

impl AliasCommand {
    /// Create a new alias command instance
    pub fn new() -> Self {
        Self
    }

    /// Process an alias assignment
    fn process_assignment(&self, assignment: &str, ctx: &mut Context) -> ShellResult<()> {
        if let Some(eq_pos) = assignment.find('=') {
            let name = &assignment[..eq_pos];
            let value = &assignment[eq_pos + 1..];

            // Validate alias name
            if !self.is_valid_alias_name(name) {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("alias: `{}': invalid alias name", name)
                ));
            }

            // Check for cycles before setting the alias
            if self.would_create_cycle(name, value, ctx) {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("alias: `{}': would create a cycle", name)
                ));
            }

            // Set the alias
            ctx.env.set_alias(name, value)?;
        } else {
            return Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                format!("alias: invalid assignment: {}", assignment)
            ));
        }

        Ok(())
    }

    /// Print a specific alias
    fn print_alias(&self, name: &str, ctx: &mut Context) -> ShellResult<()> {
        if let Some(value) = ctx.env.get_alias(name) {
            let output = format!("alias {}='{}'\n", name, self.escape_value(&value));
            ctx.stdout.write(StreamData::Text(output))
                .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::FileWriteError), format!("Failed to write output: {}", e)))?;
        } else {
            return Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::VariableNotFound),
                format!("alias: {}: not found", name)
            ));
        }

        Ok(())
    }

    /// Print all aliases
    fn print_all_aliases(&self, ctx: &mut Context) -> ShellResult<ExecutionResult> {
        let mut output = String::new();
        
        if let Ok(aliases_lock) = ctx.env.aliases.read() {
            let mut aliases: Vec<_> = aliases_lock.iter().collect();
            
            // Sort aliases by name for consistent output
            aliases.sort_by(|a, b| a.0.cmp(b.0));

            for (name, value) in aliases {
                output.push_str(&format!("alias {}='{}'\n", name, self.escape_value(value)));
            }
        } else {
            return Err(ShellError::new(
                ErrorKind::InternalError(InternalErrorKind::LockError),
                "Failed to read aliases"
            ));
        }

        if !output.is_empty() {
            ctx.stdout.write(StreamData::Text(output))
                .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::FileWriteError), format!("Failed to write output: {}", e)))?;
        }

        Ok(ExecutionResult::success(0))
    }

    /// Check if setting an alias would create a cycle
    fn would_create_cycle(&self, name: &str, value: &str, ctx: &Context) -> bool {
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
            
            while let Some(alias_value) = ctx.env.get_alias(&current) {
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

/// Convenience function to create an alias command
pub fn alias_cli(args: &[String], ctx: &mut nxsh_core::context::ShellContext) -> ShellResult<()> {
    use nxsh_core::stream::{Stream, StreamType};
    
    let mut context = Context::new(
        args.to_vec(),
        ctx,
        Stream::new(StreamType::Byte),
        Stream::new(StreamType::Text),
        Stream::new(StreamType::Byte),
    )?;

    let alias_cmd = AliasCommand::new();
    let result = alias_cmd.invoke(&mut context)?;
    
    // Output the result to stdout if any
    if let Ok(data) = context.stdout.collect() {
        for item in data {
            if let Ok(text) = item.to_string() {
                print!("{}", text);
            }
        }
    }

    if result.is_success() {
        Ok(())
    } else {
        Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound), format!("alias failed with exit code {}", result.exit_code)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nxsh_core::context::ShellContext;
    use nxsh_core::stream::{Stream, StreamType};

    fn create_test_context(args: Vec<String>) -> (Context, ShellContext) {
        let mut shell_ctx = ShellContext::new();
        let context = Context::new(
            args,
            &mut shell_ctx,
            Stream::new(StreamType::Byte),
            Stream::new(StreamType::Text),
            Stream::new(StreamType::Byte),
        ).unwrap();
        (context, shell_ctx)
    }

    #[test]
    fn test_alias_assignment() {
        let alias_cmd = AliasCommand::new();
        let (mut ctx, _shell_ctx) = create_test_context(vec!["alias".to_string(), "ll=ls -l".to_string()]);
        
        let result = alias_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        // Check that the alias was set
        assert_eq!(ctx.env.get_alias("ll"), Some("ls -l".to_string()));
    }

    #[test]
    fn test_alias_query() {
        let alias_cmd = AliasCommand::new();
        let (mut ctx, _shell_ctx) = create_test_context(vec!["alias".to_string(), "ll".to_string()]);
        
        // Set up an alias first
        ctx.env.set_alias("ll", "ls -l").unwrap();
        
        let result = alias_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        // Check that output was generated
        let output = ctx.stdout.collect().unwrap();
        assert!(!output.is_empty());
        let output_text = output[0].to_string().unwrap();
        assert!(output_text.contains("alias ll='ls -l'"));
    }

    #[test]
    fn test_alias_print_all() {
        let alias_cmd = AliasCommand::new();
        let (mut ctx, _shell_ctx) = create_test_context(vec!["alias".to_string()]);
        
        // Set up some aliases
        ctx.env.set_alias("ll", "ls -l").unwrap();
        ctx.env.set_alias("la", "ls -a").unwrap();
        
        let result = alias_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        // Check that output was generated
        let output = ctx.stdout.collect().unwrap();
        if !output.is_empty() {
            let output_text = output[0].to_string().unwrap();
            assert!(output_text.contains("ll") && output_text.contains("la"));
        }
    }

    #[test]
    fn test_alias_cycle_detection() {
        let alias_cmd = AliasCommand::new();
        let (mut ctx, _shell_ctx) = create_test_context(vec!["alias".to_string(), "ls=ls -l".to_string()]);
        
        let result = alias_cmd.invoke(&mut ctx);
        assert!(result.is_err()); // Should fail due to cycle
    }

    #[test]
    fn test_valid_alias_names() {
        let alias_cmd = AliasCommand::new();
        
        assert!(alias_cmd.is_valid_alias_name("ll"));
        assert!(alias_cmd.is_valid_alias_name("my-alias"));
        assert!(alias_cmd.is_valid_alias_name("alias123"));
        assert!(alias_cmd.is_valid_alias_name("_alias"));
        
        assert!(!alias_cmd.is_valid_alias_name(""));
        assert!(!alias_cmd.is_valid_alias_name("alias with spaces"));
        assert!(!alias_cmd.is_valid_alias_name("alias|pipe"));
        assert!(!alias_cmd.is_valid_alias_name("alias&background"));
    }

    #[test]
    fn test_escape_value() {
        let alias_cmd = AliasCommand::new();
        
        assert_eq!(alias_cmd.escape_value("simple"), "simple");
        assert_eq!(alias_cmd.escape_value("with'quote"), "with'\"'\"'quote");
        assert_eq!(alias_cmd.escape_value("with\\backslash"), "with\\\\backslash");
    }

    #[test]
    fn test_alias_expansion() {
        let mut shell_ctx = ShellContext::new();
        shell_ctx.set_alias("ll", "ls -l").unwrap();
        
        let expansion = AliasCommand::expand_alias("ll", &shell_ctx);
        assert_eq!(expansion, Some("ls -l".to_string()));
        
        let no_expansion = AliasCommand::expand_alias("nonexistent", &shell_ctx);
        assert_eq!(no_expansion, None);
    }

    #[test]
    fn test_is_alias() {
        let mut shell_ctx = ShellContext::new();
        shell_ctx.set_alias("ll", "ls -l").unwrap();
        
        assert!(AliasCommand::is_alias("ll", &shell_ctx));
        assert!(!AliasCommand::is_alias("nonexistent", &shell_ctx));
    }
} 
