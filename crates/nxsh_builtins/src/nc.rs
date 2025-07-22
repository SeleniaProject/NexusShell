//! `nc` builtin — Netcat utility wrapper.
//!
//! Attempts to execute one of the common Netcat variants: `nc`, `netcat`, or
//! `ncat` (Nmap). All arguments are forwarded verbatim, allowing users to rely
//! on familiar behaviour (e.g. port listening, relaying, etc.).

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn nc_cli(args: &[String]) -> Result<()> {
    let candidates = if cfg!(windows) {
        vec!["ncat.exe", "nc.exe", "netcat.exe", "ncat", "nc", "netcat"]
    } else {
        vec!["nc", "netcat", "ncat"]
    };

    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("nc: failed to launch backend {bin}: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    Err(anyhow!("nc: no compatible netcat variant found in PATH"))
} 