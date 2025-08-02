//! `nslookup` builtin  EDNS lookup utility.
//!
//! Implements delegation to external `nslookup` binary for full functionality.
//! If not present, returns error.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn nslookup_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("nslookup") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("nslookup: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    Err(anyhow!("nslookup: backend not found in PATH"))
} 
