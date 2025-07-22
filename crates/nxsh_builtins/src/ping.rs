//! `ping` builtin — ICMP reachability test.
//!
//! Rather than implementing raw ICMP (requires elevated privileges),
//! this builtin delegates to the system `ping` executable, preserving
//! all command-line arguments for compatibility.
//! Works on both Unix and Windows (`ping.exe`).

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn ping_cli(args: &[String]) -> Result<()> {
    // candidate names in order of preference
    let candidates = if cfg!(windows) {
        vec!["ping.exe", "ping"]
    } else {
        vec!["ping", "ping6"]
    };

    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("ping: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    Err(anyhow!("ping: no compatible backend found in PATH"))
} 