//! `export` builtin command - export environment variables
//!
//! This module implements the export builtin command for setting
//! and exporting environment variables to child processes.

use crate::common::{i18n::*, logging::*};
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
        "export [name[=value] ...]"
    }

    fn affects_shell_state(&self) -> bool {
        true // export modifies environment variables
    }

    fn help(&self) -> &'static str {
        "Export environment variables. Use 'export --help' for detailed usage information."
    }

    fn execute(&self, ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        if args.is_empty() {
            // Print all exported environment variables
            self.print_all_exports(ctx)?;
        } else {
            // Process each argument
            for arg in args {
                if arg.contains('=') {
                    // Assignment: name=value
                    self.process_assignment(arg, ctx)?;
                } else {
                    // Export existing variable
                    self.export_variable(arg, ctx)?;
                }
            }
        }

        Ok(ExecutionResult::success(0))
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
    }

    /// Process a variable assignment
    fn process_assignment(&self, assignment: &str, ctx: &mut ShellContext) -> ShellResult<()> {
        if let Some(eq_pos) = assignment.find('=') {
            let name = &assignment[..eq_pos];
            let value = &assignment[eq_pos + 1..];

            // Validate variable name
            if !self.is_valid_variable_name(name) {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("export: `{}': not a valid identifier", name)
                ));
            }

            // Set the variable and mark it as exported
            self.set_and_export_variable(name, value, ctx)?;
        } else {
            return Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                format!("export: invalid assignment: {}", assignment)
            ));
        }

        Ok(())
    }

    /// Export an existing variable
    fn export_variable(&self, name: &str, ctx: &mut ShellContext) -> ShellResult<()> {
        // Validate variable name
        if !self.is_valid_variable_name(name) {
            return Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                format!("export: `{}': not a valid identifier", name)
            ));
        }

        // Check if the variable exists
        if let Some(var) = ctx.environment.get(name) {
            // Mark the variable as exported
            let mut exported_var = var.clone();
            exported_var.exported = true;
            ctx.environment.insert(name.to_string(), exported_var);
        } else {
            // Variable doesn't exist, create it with empty value and mark as exported
            let var = ShellVariable {
                value: String::new(),
                exported: true,
                readonly: false,
            };
            ctx.environment.insert(name.to_string(), var);
        }

        Ok(())
    }

    /// Set a variable value and mark it as exported
    fn set_and_export_variable(&self, name: &str, value: &str, ctx: &mut ShellContext) -> ShellResult<()> {
        let var = ShellVariable {
            value: value.to_string(),
            exported: true,
            readonly: false,
        };
        ctx.environment.insert(name.to_string(), var);
        Ok(())
    }

    /// Print all exported environment variables
    fn print_all_exports(&self, ctx: &mut ShellContext) -> ShellResult<()> {
        let mut output = String::new();
        
        // Get all environment variables
        let mut exports: Vec<_> = ctx.environment.iter()
            .filter(|(_, var)| var.exported)
            .collect();
        
        // Sort by name for consistent output
        exports.sort_by(|a, b| a.0.cmp(b.0));

        for (name, var) in exports {
            output.push_str(&format!("export {}='{}'\n", name, self.escape_value(&var.value)));
        }

        if !output.is_empty() {
            ctx.stdout.write(StreamData::Text(output))
                .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::FileWriteError), format!("Failed to write output: {}", e)))?;
        }

        Ok(())
    }

    /// Validate variable name
    fn is_valid_variable_name(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Variable names must start with a letter or underscore
        if !name.chars().next().unwrap().is_ascii_alphabetic() && name.chars().next().unwrap() != '_' {
            return false;
        }

        // Rest of the name can contain letters, digits, and underscores
        for ch in name.chars().skip(1) {
            if !ch.is_ascii_alphanumeric() && ch != '_' {
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
                '\'' => result.push_str("'\"'\"'"), // End quote, escaped quote, start quote
                '\\' => result.push_str("\\\\"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use nxsh_core::context::ShellContext;

    fn create_test_context() -> ShellContext {
        ShellContext::new()
    }

    #[test]
    fn test_export_basic() {
        let export_cmd = ExportCommand::new();
        let mut ctx = create_test_context();
        
        let result = export_cmd.execute(&mut ctx, &["TEST=value".to_string()]).unwrap();
        assert!(result.is_success());
        
        // Verify the variable was set and exported
        if let Some(var) = ctx.environment.get("TEST") {
            assert_eq!(var.value, "value");
            assert!(var.exported);
        } else {
            panic!("Variable TEST was not set");
        }
    }

    #[test]
    fn test_export_existing_variable() {
        let export_cmd = ExportCommand::new();
        let mut ctx = create_test_context();
        
        // First set a variable without exporting
        ctx.environment.insert("EXISTING".to_string(), ShellVariable {
            value: "existing_value".to_string(),
            exported: false,
            readonly: false,
        });
        
        // Export the existing variable
        let result = export_cmd.execute(&mut ctx, &["EXISTING".to_string()]).unwrap();
        assert!(result.is_success());
        
        // Verify the variable is now exported
        if let Some(var) = ctx.environment.get("EXISTING") {
            assert_eq!(var.value, "existing_value");
            assert!(var.exported);
        } else {
            panic!("Variable EXISTING was not found");
        }
    }

    #[test]
    fn test_export_invalid_name() {
        let export_cmd = ExportCommand::new();
        let mut ctx = create_test_context();
        
        let result = export_cmd.execute(&mut ctx, &["123INVALID=value".to_string()]);
        assert!(result.is_err());
    }
}
