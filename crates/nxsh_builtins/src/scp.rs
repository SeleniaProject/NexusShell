//! `scp` builtin  Esecure copy utility.
//!
//! This builtin defers to the platform's OpenSSH `scp` implementation to ensure
//! robust protocol support, cipher negotiation, progress UI, and compatibility
//! with existing scripts. All command-line arguments are forwarded verbatim.
//!
//! If `scp` is not found in the `PATH`, an informative error is returned.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

/// Entry point for the `scp` builtin.
pub fn scp_cli(args: &[String]) -> Result<()> {
    // Search candidates (Windows may have scp.exe).
    let candidates = if cfg!(windows) {
        vec!["scp.exe", "scp"]
    } else {
        vec!["scp"]
    };

    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("scp: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    Err(anyhow!("scp: no compatible scp client found in PATH; please install OpenSSH"))
} 

