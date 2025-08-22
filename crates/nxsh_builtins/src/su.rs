//! `su` builtin  Eswitch user.
//!
//! Delegates to system `su` binary; no fallback.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn su_cli(args: &[String]) -> Result<()> {
    let candidates = if cfg!(windows) {
        vec!["su.exe", "su"]
    } else {
        vec!["su"]
    };
    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path).args(args).status().map_err(|e| anyhow!("su: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }
    Err(anyhow!("su: backend not found"))
} 

