//! `alias` builtin command - manage command aliases
//!
//! This module implements the alias builtin command for creating
//! and managing command aliases with cycle detection.

use nxsh_core::memory_efficient::MemoryEfficientStringBuilder;
use nxsh_core::{
    context::ShellContext, Builtin, Context, ExecutionResult, ShellError, ShellResult,
};
use std::io::Write;

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

    fn execute(&self, ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        let mut print_aliases = false;
        let mut assignments = Vec::new();
        let mut queries = Vec::new();

        // Parse arguments
        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];

            if arg == "-p" {
                print_aliases = true;
            } else if arg.contains('=') {
                assignments.push(arg.clone());
            } else {
                queries.push(arg.clone());
            }

            i += 1;
        }

        // If no arguments, print all aliases
        if args.is_empty() || print_aliases {
            self.print_all_aliases(ctx)?;
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
        AliasCommand
    }

    /// Process an alias assignment
    fn process_assignment(&self, assignment: &str, ctx: &mut ShellContext) -> ShellResult<()> {
        if let Some(eq_pos) = assignment.find('=') {
            let name = &assignment[..eq_pos];
            let value = &assignment[eq_pos + 1..];

            // Validate alias name
            if !self.is_valid_alias_name(name) {
                // Pre-calculate capacity for optimal memory usage
                let capacity = 15 + name.len() + 21; // "alias: `" + name + "': invalid alias name"
                let mut error_msg = MemoryEfficientStringBuilder::new(capacity);
                error_msg.push_str("alias: `");
                error_msg.push_str(name);
                error_msg.push_str("': invalid alias name");
                return Err(ShellError::new(
                    nxsh_core::error::ErrorKind::RuntimeError(
                        nxsh_core::error::RuntimeErrorKind::InvalidArgument,
                    ),
                    error_msg.into_string(),
                ));
            }

            // Check for cycles before setting the alias
            if self.would_create_cycle(name, value, ctx) {
                // Pre-calculate capacity for optimal memory usage
                let capacity = 15 + name.len() + 19; // "alias: `" + name + "': would create a cycle"
                let mut error_msg = MemoryEfficientStringBuilder::new(capacity);
                error_msg.push_str("alias: `");
                error_msg.push_str(name);
                error_msg.push_str("': would create a cycle");
                return Err(ShellError::new(
                    nxsh_core::error::ErrorKind::RuntimeError(
                        nxsh_core::error::RuntimeErrorKind::InvalidArgument,
                    ),
                    error_msg.into_string(),
                ));
            }

            // Set the alias
            ctx.aliases
                .write()
                .unwrap()
                .insert(name.to_string(), value.to_string());
        } else {
            // Pre-calculate capacity for optimal memory usage
            let capacity = 15 + assignment.len() + 24; // "alias: `" + assignment + "': invalid alias assignment"
            let mut error_msg = MemoryEfficientStringBuilder::new(capacity);
            error_msg.push_str("alias: invalid assignment: ");
            error_msg.push_str(assignment);
            return Err(ShellError::new(
                nxsh_core::error::ErrorKind::RuntimeError(
                    nxsh_core::error::RuntimeErrorKind::InvalidArgument,
                ),
                error_msg.into_string(),
            ));
        }

        Ok(())
    }

    /// Print a specific alias
    fn print_alias(&self, name: &str, ctx: &mut ShellContext) -> ShellResult<()> {
        if let Some(value) = ctx.aliases.read().unwrap().get(name) {
            // Pre-calculate capacity for optimal memory usage
            let capacity = 6 + name.len() + 3 + value.len() + 1; // "alias " + name + "='" + value + "'"
            let mut output = MemoryEfficientStringBuilder::new(capacity);
            output.push_str("alias ");
            output.push_str(name);
            output.push_str("='");
            output.push_str(&self.escape_value(value));
            output.push_str("'\n");
            ctx.stdout
                .write(output.into_string().as_bytes())
                .map_err(|e| {
                    // Pre-calculate capacity for optimal memory usage
                    let err_str = e.to_string();
                    let capacity = 23 + err_str.len(); // "Failed to write output: " + error_message
                    let mut error_msg = MemoryEfficientStringBuilder::new(capacity);
                    error_msg.push_str("Failed to write output: ");
                    error_msg.push_str(&err_str);
                    ShellError::new(
                        nxsh_core::error::ErrorKind::IoError(
                            nxsh_core::error::IoErrorKind::FileWriteError,
                        ),
                        error_msg.into_string(),
                    )
                })?;
        } else {
            // Pre-calculate capacity for optimal memory usage
            let capacity = 8 + name.len() + 11; // "alias: " + name + ": not found"
            let mut error_msg = MemoryEfficientStringBuilder::new(capacity);
            error_msg.push_str("alias: ");
            error_msg.push_str(name);
            error_msg.push_str(": not found");
            return Err(ShellError::new(
                nxsh_core::error::ErrorKind::RuntimeError(
                    nxsh_core::error::RuntimeErrorKind::VariableNotFound,
                ),
                error_msg.into_string(),
            ));
        }

        Ok(())
    }

    /// Print all aliases
    fn print_all_aliases(&self, ctx: &mut ShellContext) -> ShellResult<()> {
        let output = if let Ok(aliases_lock) = ctx.aliases.read() {
            let mut aliases: Vec<_> = aliases_lock.iter().collect();

            // Sort aliases by name for consistent output
            aliases.sort_by_key(|(name, _)| *name);

            // Pre-calculate total capacity needed for better memory efficiency
            let total_capacity = aliases
                .iter()
                .map(|(name, value)| 6 + name.len() + 3 + value.len() + 2) // "alias " + name + "='" + value + "'\n"
                .sum::<usize>();

            let mut output = MemoryEfficientStringBuilder::new(total_capacity);

            for (name, value) in aliases {
                output.push_str("alias ");
                output.push_str(name);
                output.push_str("='");
                output.push_str(&self.escape_value(value));
                output.push_str("'\n");
            }
            output
        } else {
            return Err(ShellError::new(
                nxsh_core::error::ErrorKind::InternalError(
                    nxsh_core::error::InternalErrorKind::LockError,
                ),
                "Failed to read aliases",
            ));
        };

        if !output.as_string().is_empty() {
            ctx.stdout
                .write(output.into_string().as_bytes())
                .map_err(|e| {
                    // Pre-calculate capacity for optimal memory usage
                    let err_str = e.to_string();
                    let capacity = 23 + err_str.len(); // "Failed to write output: " + error_message
                    let mut error_msg = MemoryEfficientStringBuilder::new(capacity);
                    error_msg.push_str("Failed to write output: ");
                    error_msg.push_str(&err_str);
                    ShellError::new(
                        nxsh_core::error::ErrorKind::IoError(
                            nxsh_core::error::IoErrorKind::FileWriteError,
                        ),
                        error_msg.into_string(),
                    )
                })?;
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
                '|' | '&' | ';' | '(' | ')' | '<' | '>' | ' ' | '\t' | '\n' | '\r' | '"' | '\''
                | '\\' | '$' | '`' | '*' | '?' | '[' | ']' | '{' | '}' | '~' | '#' => return false,
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

/// CLI wrapper function for alias command
pub fn alias_cli(args: &[String]) -> anyhow::Result<()> {
    let mut ctx = nxsh_core::context::ShellContext::new();
    let command = AliasCommand::new();
    match command.execute(&mut ctx, args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("alias command failed: {}", e)),
    }
}

/// Execute function stub
pub fn execute(
    _args: &[String],
    _context: &crate::common::BuiltinContext,
) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}
