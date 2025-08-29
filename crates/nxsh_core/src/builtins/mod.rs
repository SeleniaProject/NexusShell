//! Built-in commands for NexusShell
//!
//! This module provides implementations of shell built-in commands,
//! including job control commands like jobs, fg, bg, etc.

use crate::executor::Builtin;
use std::sync::Arc;

pub mod bg;
pub mod fg;
pub mod id;
pub mod jobs;
pub mod kill;
pub mod testutils;

pub use id::IdBuiltin;
use kill::KillBuiltin;
use testutils::ArgDumpBuiltin;

/// Register all built-in commands
pub fn register_all_builtins() -> Vec<Arc<dyn Builtin>> {
    vec![
        Arc::new(jobs::JobsBuiltin),
        Arc::new(fg::FgBuiltin),
        Arc::new(bg::BgBuiltin),
        Arc::new(IdBuiltin),
        Arc::new(ArgDumpBuiltin),
        Arc::new(KillBuiltin),
        // Minimal echo builtin to ensure tests relying on `echo` run under strict timeout env
        Arc::new(testutils::EchoBuiltin),
    ]
}
