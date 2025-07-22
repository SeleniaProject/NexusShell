//! `dig` builtin — detailed DNS query utility.
//!
//! Delegates to system `dig` binary if available, forwarding all arguments.
//! If not found, returns an error.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn dig_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("dig") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("dig: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    Err(anyhow!("dig: backend not found in PATH"))
} 