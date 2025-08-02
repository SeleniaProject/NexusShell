//! `traceroute` builtin  Edisplay route to host.
//!
//! Delegates to system `traceroute` (Unix) or `tracert.exe` (Windows).
//! All arguments are forwarded unchanged for maximum compatibility.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn traceroute_cli(args: &[String]) -> Result<()> {
    let candidates = if cfg!(windows) {
        vec!["tracert.exe", "tracert"]
    } else {
        vec!["traceroute", "tracepath"]
    };

    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("traceroute: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    Err(anyhow!("traceroute: no compatible backend found in PATH"))
} 
