//! `rsync` builtin  Efast incremental file transfer.
//!
//! For maximum feature parity and performance, this builtin simply re-executes
//! the system `rsync` binary, forwarding all arguments verbatim. When `rsync`
//! is not available an error is returned advising installation. Implementing a
//! full rsync algorithm in Rust is out of scope.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn rsync_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("rsync") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("rsync: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    Err(anyhow!("rsync: backend not found; please install rsync"))
} 

