//! Built-in commands for NexusShell
//!
//! This module provides implementations of shell built-in commands,
//! including job control commands like jobs, fg, bg, etc.

pub mod jobs;
pub mod fg;
pub mod bg;

use crate::executor::Builtin;
use std::sync::Arc;

/// Register all built-in commands
pub fn register_all_builtins() -> Vec<Arc<dyn Builtin>> {
    vec![
        Arc::new(jobs::JobsBuiltin),
        Arc::new(fg::FgBuiltin),
        Arc::new(bg::BgBuiltin),
    ]
}
