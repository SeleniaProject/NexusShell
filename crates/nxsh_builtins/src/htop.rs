//! `htop` command — enhanced dynamic system monitor.
//!
//! Implementation strategy: if the external `htop` binary exists in PATH, delegate
//! execution to it (preserving any CLI args). If not available, fall back to the
//! built-in simpler `top` implementation so that the command always works.
//!
//! Note: future work could replace the fallback with a fully-featured native TUI.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

// Reuse the internal `top` builtin for fallback.
use crate::top::top_cli;

pub fn htop_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("htop") {
        // Delegate to external `htop` binary.
        let status = Command::new(path).args(args).status().map_err(|e| anyhow!("htop: failed to launch external binary: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("htop: external binary exited with status {:?}", status.code()))
        }
    } else {
        // Fallback to internal `top`.
        eprintln!("htop: external binary not found; falling back to builtin top");
        top_cli(args)
    }
} 