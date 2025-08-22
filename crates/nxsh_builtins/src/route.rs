//! `route` builtin  Eview or manipulate routing tables.
//!
//! On most Unix-like systems, `route` (from net-tools) is deprecated in favour
//! of `ip route` (from iproute2), but many users still depend on it. This
//! builtin attempts to execute an available backend in the following order:
//! 1. Native `route` binary (`route` or `route.exe`).
//! 2. `ip route` with the supplied arguments.
//!
//! All arguments are forwarded unchanged. Kernel-level route management is
//! complex and platform-specific, so we avoid re-implementing it in Rust.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn route_cli(args: &[String]) -> Result<()> {
    // 1. Try native route binary
    let candidates = if cfg!(windows) {
        vec!["route.exe", "route"]
    } else {
        vec!["route"]
    };

    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("route: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    // 2. Fallback to `ip route` if available
    if let Ok(ip_bin) = which("ip") {
        let mut forwarded = vec!["route".to_string()];
        forwarded.extend_from_slice(args);
        let status = Command::new(ip_bin)
            .args(&forwarded)
            .status()
            .map_err(|e| anyhow!("route: fallback 'ip route' failed: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    Err(anyhow!("route: no suitable backend found (tried route/ip route)"))
} 

