//! `lsblk` builtin — list block devices.
//!
//! Delegates to system `lsblk` binary for complete output and option support.
//! No fallback implementation is provided since querying block devices is
//! platform-specific.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn lsblk_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("lsblk") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("lsblk: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    Err(anyhow!("lsblk: backend not found; please install util-linux"))
} 