//! Comprehensive builtin integration for nxsh_core
//! 
//! This module bridges nxsh_core's Builtin trait with nxsh_builtins' comprehensive
//! command implementations, providing access to the full NexusShell command set
//! in normal interactive shell mode.

use crate::executor::Builtin;
use crate::context::ShellContext;
use crate::error::ShellError;
use std::sync::Arc;

/// Wrapper that adapts nxsh_builtins commands to the nxsh_core Builtin trait
pub struct NxshBuiltinWrapper {
    pub name: &'static str,
}

impl NxshBuiltinWrapper {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl Builtin for NxshBuiltinWrapper {
    fn name(&self) -> &str {
        self.name
    }

    fn execute(&self, _context: &mut ShellContext, args: &[String]) -> Result<crate::ExecutionResult, ShellError> {
        // Convert nxsh_builtins result to nxsh_core ExecutionResult
        match nxsh_builtins::execute_builtin(self.name, args) {
            Ok(()) => {
                // Most builtins write directly to stdout/stderr, so we return empty result
                Ok(crate::ExecutionResult {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                })
            }
            Err(shell_error) => {
                // Convert ShellError to ExecutionResult with error output
                Ok(crate::ExecutionResult {
                    stdout: String::new(),
                    stderr: shell_error.to_string(),
                    exit_code: 1,
                })
            }
        }
    }

    fn help(&self) -> &'static str {
        "NexusShell builtin command"
    }

    fn synopsis(&self) -> &'static str {
        "Built-in command"
    }

    fn description(&self) -> &'static str {
        "Comprehensive NexusShell builtin command"
    }

    fn usage(&self) -> &'static str {
        ""
    }
}

/// Register all comprehensive built-in commands including both nxsh_core and nxsh_builtins
pub fn register_comprehensive_builtins() -> Vec<Arc<dyn Builtin>> {
    let mut builtins: Vec<Arc<dyn Builtin>> = Vec::new();
    
    // Add nxsh_core's minimal builtins
    builtins.extend(super::register_all_builtins());
    
    // Add comprehensive nxsh_builtins commands
    for name in nxsh_builtins::list_builtin_names() {
        // Skip duplicates that are already in nxsh_core
        if !["jobs", "fg", "bg", "id", "kill", "echo"].contains(&name) {
            builtins.push(Arc::new(NxshBuiltinWrapper::new(name)));
        }
    }
    
    builtins
}
