//! `arp` builtin  EARP table manipulation and inspection.
//!
//! Most modern Linux distributions deprecate the standalone `arp` binary in
//! favour of `ip neigh` from the `iproute2` package. This builtin therefore
//! attempts to execute the first available backend in the following order:
//! 1. Native `arp` binary (`arp` or `arp.exe`).
//! 2. `ip neigh` with the supplied arguments.
//!
//! All arguments provided by the caller are forwarded verbatim. Implementing a
//! cross-platform ARP stack in pure Rust is out of scope for NexusShell.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn arp_cli(args: &[String]) -> Result<()> {
    // Preferred backend list depending on OS.
    let candidates = if cfg!(windows) {
        vec!["arp.exe", "arp"]
    } else {
        vec!["arp"]
    };

    // Try native arp.
    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("arp: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    // Fallback to `ip neigh` if available.
    if let Ok(ip_bin) = which("ip") {
        let mut forwarded = vec!["neigh".to_string()];
        forwarded.extend_from_slice(args);
        let status = Command::new(ip_bin)
            .args(&forwarded)
            .status()
            .map_err(|e| anyhow!("arp: fallback 'ip neigh' failed: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    Err(anyhow!("arp: no suitable backend found (tried arp/ip neigh)"))
} 
