//! `telnet` builtin  Esimple TCP debugging client.
//!
//! This builtin forwards execution to the platform's `telnet` binary when
//! available. If `telnet` is missing but `ncat` (from Nmap) is installed, we
//! transparently invoke `ncat --telnet` to provide a comparable interactive
//! session.
//!
//! The function does not attempt to implement a Telnet protocol client in Rust
//! because full compliance (IAC negotiation, option handling) is non-trivial
//! and existing tools already solve this problem.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn telnet_cli(args: &[String]) -> Result<()> {
    // Preferred binaries in order.
    let candidates = if cfg!(windows) {
        vec!["telnet.exe", "telnet"]
    } else {
        vec!["telnet"]
    };

    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("telnet: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    // Fallback to ncat --telnet if available
    if let Ok(ncat) = which("ncat") {
        // Prepend --telnet flag before user args
        let mut forwarded = vec!["--telnet".to_string()];
        forwarded.extend_from_slice(args);
        let status = Command::new(ncat)
            .args(&forwarded)
            .status()
            .map_err(|e| anyhow!("telnet: fallback ncat failed: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    Err(anyhow!(
        "telnet: no suitable client found (tried telnet/ncat --telnet)"
    ))
} 
