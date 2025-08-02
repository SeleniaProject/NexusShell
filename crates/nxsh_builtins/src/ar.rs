//! `ar` builtin  Estatic library archiver wrapper.
//!
//! Provides access to platform `ar` command. All arguments are forwarded verbatim.
//! Implementing full archive manipulation internally is out of scope.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn ar_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("ar") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("ar: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    Err(anyhow!("ar: backend not found; please install binutils"))
} 
