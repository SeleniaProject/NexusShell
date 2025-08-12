//! `export` builtin command - export environment variables
//!
//! This module implements the export builtin command for setting
//! and exporting environment variables to child processes.

use crate::common        Ok(ExecutionResult::success(0))
    }

    fn help(&self) -> String {
        "export - set environment variables".to_string()
    }

    fn usage(&self) -> String {
        "export [name[=value] ...]".to_string()
    }
}

impl ExportCommand {
    /// Create a new export command instance
    pub fn new() -> Self {
        Self
    }ng::*};
use std::io::Write;
use std::collections::HashMap;
use nxsh_core::{Builtin, Context, ShellContext, ExecutionResult, ShellResult, ShellError, ErrorKind, StreamData};
use nxsh_core::context::ShellVariable;
use nxsh_core::error::{RuntimeErrorKind, IoErrorKind, InternalErrorKind};

/// The `export` builtin command implementation
pub struct ExportCommand;

impl Builtin for ExportCommand {
    fn name(&self) -> &'static str {
        "export"
    }

    fn synopsis(&self) -> &'static str {
        "Export environment variables"
    }

    fn description(&self) -> &'static str {
        "Set environment variables and mark them for export to child processes."
    }

    fn usage(&self) -> &'static str {
        "export [-p] [name[=value] ...]"
    }

    fn affects_shell_state(&self) -> bool {
        true // export modifies environment variables
    }

    fn help(&self) -> &'static str {
        "Export environment variables. Use 'export --help' for detailed usage information."
    }

    // The legacy Context shim has been removed; implementation below uses ShellContext directly.
    fn execute(&self, ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        let mut print_all = false;
        let mut assignments = Vec::new();

        // Parse arguments
        let mut i = 0; // Start from 0 since args doesn't include command name
        while i < args.len() {
            let arg = &args[i];
            
            if arg == "-p" {
                print_all = true;
            } else if arg.starts_with('-') {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("export: invalid option: {}", arg)
                ));
            } else {
                assignments.push(arg.clone());
            }
            
            i += 1;
        }

        // If -p flag is used or no arguments, print all exported variables
        if print_all || assignments.is_empty() {
            self.print_exported_variables(ctx)?;
            return Ok(ExecutionResult::success(0));
        }

        // Process variable assignments
        for assignment in assignments {
            self.process_assignment(&assignment, ctx)?;
        }

        Ok(ExecutionResult::success(0))
    }
}

impl ExportCommand {
    /// Create a new export command instance
    pub fn new() -> Self {
        Self
    }

    /// Process a variable assignment
    fn process_assignment(&self, assignment: &str, ctx: &mut ShellContext) -> ShellResult<()> {
        if let Some(eq_pos) = assignment.find('=') {
            // Variable assignment: name=value
            let name = &assignment[..eq_pos];
            let value = &assignment[eq_pos + 1..];

            // Validate variable name
            if !self.is_valid_variable_name(name) {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("export: `{}': not a valid identifier", name)
                ));
            }

            // Create shell variable and mark as exported
            let shell_var = ShellVariable::new(value).exported();
            ctx.env.set_shell_var(name, shell_var);
            
            // Also set in environment for immediate use
            ctx.env.set_var(name, value);
        } else {
            // Just export existing variable
            let name = assignment;

            // Check if variable exists
            if let Some(value) = ctx.env.get_var(name) {
                // Mark existing variable as exported
                let shell_var = ShellVariable::new(value).exported();
                ctx.env.set_shell_var(name, shell_var);
            } else {
                // Variable doesn't exist, create empty exported variable
                let shell_var = ShellVariable::new("").exported();
                ctx.env.set_shell_var(name, shell_var);
                ctx.env.set_var(name, "");
            }
        }

        Ok(())
    }

    /// Print all exported variables
    fn print_exported_variables(&self, ctx: &mut ShellContext) -> ShellResult<()> {
        let mut output = String::new();

        // Get all environment variables and format them
        if let Ok(env_guard) = ctx.env.env.read() {
            for (key, value) in env_guard.iter() {
                // Check if this variable is marked as exported
                if let Ok(vars_guard) = ctx.env.vars.read() {
                    if let Some(shell_var) = vars_guard.get(key) {
                        if shell_var.exported {
                            output.push_str(&format!("declare -x {}=\"{}\"\n", key, self.escape_value(value)));
                        }
                    } else {
                        // If not in shell variables, assume it's exported (from system environment)
                        output.push_str(&format!("declare -x {}=\"{}\"\n", key, self.escape_value(value)));
                    }
                }
            }
        }

        // Write to stdout
        ctx.stdout.write(StreamData::Text(output))
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::FileWriteError), format!("Failed to write output: {}", e)))?;

        Ok(())
    }

    /// Validate variable name according to shell rules
    fn is_valid_variable_name(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // First character must be letter or underscore
        let mut chars = name.chars();
        if let Some(first) = chars.next() {
            if !first.is_alphabetic() && first != '_' {
                return false;
            }
        }

        // Remaining characters must be alphanumeric or underscore
        for ch in chars {
            if !ch.is_alphanumeric() && ch != '_' {
                return false;
            }
        }

        true
    }

    /// Escape special characters in variable values for display
    fn escape_value(&self, value: &str) -> String {
        let mut result = String::new();
        
        for ch in value.chars() {
            match ch {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '$' => result.push_str("\\$"),
                '`' => result.push_str("\\`"),
                _ => result.push(ch),
            }
        }
        
        result
    }
}

impl Default for ExportCommand {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create an export command
pub fn export_cli(args: &[String], ctx: &mut nxsh_core::context::ShellContext) -> ShellResult<()> {
    let export_cmd = ExportCommand::new();
    let result = export_cmd.execute(ctx, args)?;
    
    match result.exit_code {
        0 => Ok(()),
        code => Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound),
            format!("export command failed with exit code {}", code)
        ))
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
    fn test_export_new_variable() {
        let export_cmd = ExportCommand::new();
        let (mut ctx, _shell_ctx) = create_test_context(vec!["export".to_string(), "TEST_VAR=hello".to_string()]);
        
        let result = export_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        // Check that the variable was set and exported
        assert_eq!(ctx.env.get_var("TEST_VAR"), Some("hello".to_string()));
        
        // Check that it's marked as exported
        if let Some(var) = ctx.env.variables.get("TEST_VAR", None) {
            assert!(var.exported);
        }
    }

    #[test]
    fn test_export_existing_variable() {
        let export_cmd = ExportCommand::new();
        let (mut ctx, _shell_ctx) = create_test_context(vec!["export".to_string(), "EXISTING_VAR".to_string()]);
        
        // Set up an existing variable
        ctx.env.set_var("EXISTING_VAR", "existing_value");
        
        let result = export_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        // Check that the variable is still there and exported
        assert_eq!(ctx.env.get_var("EXISTING_VAR"), Some("existing_value".to_string()));
        
        // Check that it's marked as exported
        if let Some(var) = ctx.env.variables.get("EXISTING_VAR", None) {
            assert!(var.exported);
        }
    }

    #[test]
    fn test_export_invalid_name() {
        let export_cmd = ExportCommand::new();
        let (mut ctx, _shell_ctx) = create_test_context(vec!["export".to_string(), "123INVALID=value".to_string()]);
        
        let result = export_cmd.invoke(&mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_print_all() {
        let export_cmd = ExportCommand::new();
        let (mut ctx, _shell_ctx) = create_test_context(vec!["export".to_string(), "-p".to_string()]);
        
        // Set up some exported variables
        ctx.env.set_var("TEST1", "value1");
        ctx.env.set_var("TEST2", "value2");
        
        let result = export_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        // Check that output was generated
        let output = ctx.stdout.collect().unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_valid_variable_names() {
        let export_cmd = ExportCommand::new();
        
        assert!(export_cmd.is_valid_variable_name("VAR"));
        assert!(export_cmd.is_valid_variable_name("_VAR"));
        assert!(export_cmd.is_valid_variable_name("VAR123"));
        assert!(export_cmd.is_valid_variable_name("_VAR_123"));
        
        assert!(!export_cmd.is_valid_variable_name("123VAR"));
        assert!(!export_cmd.is_valid_variable_name("VAR-NAME"));
        assert!(!export_cmd.is_valid_variable_name("VAR.NAME"));
        assert!(!export_cmd.is_valid_variable_name(""));
    }

    #[test]
    fn test_escape_value() {
        let export_cmd = ExportCommand::new();
        
        assert_eq!(export_cmd.escape_value("simple"), "simple");
        assert_eq!(export_cmd.escape_value("with\"quotes"), "with\\\"quotes");
        assert_eq!(export_cmd.escape_value("with\\backslash"), "with\\\\backslash");
        assert_eq!(export_cmd.escape_value("with\nnewline"), "with\\nnewline");
        assert_eq!(export_cmd.escape_value("with$dollar"), "with\\$dollar");
    }
} 
