//! `blkid` builtin — locate/print block device attributes.
//!
//! Delegates execution to the system `blkid` binary for comprehensive output.
//! No fallback implementation is provided.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn blkid_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("blkid") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("blkid: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    Err(anyhow!("blkid: backend not found; install util-linux"))
} 