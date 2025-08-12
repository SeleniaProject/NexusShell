//! Built-in commands for NexusShell
//!
//! This module provides implementations of shell built-in commands,
//! including job control commands like jobs, fg, bg, etc.

use crate::executor::Builtin;
use std::sync::Arc;

pub mod jobs;
pub mod fg;
pub mod bg;
pub mod id;
pub mod testutils;

pub use id::IdBuiltin;

/// Register all built-in commands
pub fn register_all_builtins() -> Vec<Arc<dyn Builtin>> {
    let mut v: Vec<Arc<dyn Builtin>> = vec![
        Arc::new(jobs::JobsBuiltin),
        Arc::new(fg::FgBuiltin),
        Arc::new(bg::BgBuiltin),
        Arc::new(IdBuiltin),
        Arc::new(testutils::ArgDumpBuiltin),
    ];
    v
}
