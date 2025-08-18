//! Kill builtin placeholder for nxsh_core
//!
//! This is a basic kill command implementation for nxsh_core's builtin system.
//! For now, it returns an error directing users to the full kill implementation.

use crate::context::ShellContext;
use crate::executor::{Builtin, ExecutionResult, ExecutionStrategy, ExecutionMetrics};
use crate::error::ShellResult;
use std::time::Instant;

/// Basic kill builtin
pub struct KillBuiltin;

impl Builtin for KillBuiltin {
    fn name(&self) -> &'static str {
        "kill"
    }

    fn synopsis(&self) -> &'static str {
        "send a signal to processes"
    }

    fn description(&self) -> &'static str {
        "Send signals to processes identified by PID (basic implementation)"
    }

    fn usage(&self) -> &'static str {
        "kill [OPTION] PID..."
    }

    fn execute(&self, _context: &mut ShellContext, _args: &[String]) -> ShellResult<ExecutionResult> {
        let start_time = Instant::now();
        
        // For now, return error directing to the full implementation
        Ok(ExecutionResult {
            exit_code: 1,
            stdout: String::new(),
            stderr: "kill: basic implementation - use the full shell interface for kill functionality".to_string(),
            execution_time: start_time.elapsed().as_micros() as u64,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        })
    }

    fn help(&self) -> &'static str {
        "kill - terminate processes by PID or job ID (basic implementation)
        
This is a placeholder implementation in nxsh_core.
For full kill functionality including job control and signal names,
use the nxsh shell interface or nxsh_builtins directly.

BASIC USAGE:
    kill PID        Terminate process by PID
    
For advanced features like job control (%1), signal names (-TERM),
and process groups, use the full shell environment."
    }
}
