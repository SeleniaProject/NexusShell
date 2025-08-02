//! `visudo` builtin  Eedit sudoers file safely.
//!
//! Delegates to system `visudo` binary; no fallback.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn visudo_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("visudo") {
        let status = Command::new(path).args(args).status().map_err(|e| anyhow!("visudo: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    Err(anyhow!("visudo: backend not found; install sudo package"))
} 
