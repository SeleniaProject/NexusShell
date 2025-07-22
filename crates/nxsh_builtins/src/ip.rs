//! `ip` builtin — network configuration utility wrapper.
//!
//! On Linux and other Unix‐like systems, this builtin looks for the `ip`
//! command from the `iproute2` suite. All command-line arguments are forwarded
//! unchanged to preserve behaviour (e.g. `ip addr`, `ip route`).
//!
//! Platform fallbacks:
//! • macOS: attempts to use `ifconfig` / `route` when `ip` is unavailable.
//!   Only a subset of sub-commands (addr/route) can be mapped heuristically.
//! • Windows: under PowerShell/CMD, falls back to `ipconfig` for basic
//!   interface display. Advanced features are not mapped.
//!
//! Because kernel networking APIs differ drastically per OS, the builtin does
//! not attempt a Rust-level re-implementation.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn ip_cli(args: &[String]) -> Result<()> {
    // 1. Preferred backend: ip (iproute2)
    if let Ok(ip_bin) = which("ip") {
        let status = Command::new(ip_bin)
            .args(args)
            .status()
            .map_err(|e| anyhow!("ip: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // 2. macOS fallback: map common invocations
    #[cfg(target_os = "macos")]
    {
        if !args.is_empty() {
            match args[0].as_str() {
                "addr" | "address" => {
                    let status = Command::new("ifconfig")
                        .args(&args[1..])
                        .status()
                        .map_err(|e| anyhow!("ip: fallback ifconfig failed: {e}"))?;
                    std::process::exit(status.code().unwrap_or(1));
                }
                "route" => {
                    let status = Command::new("route")
                        .args(&args[1..])
                        .status()
                        .map_err(|e| anyhow!("ip: fallback route failed: {e}"))?;
                    std::process::exit(status.code().unwrap_or(1));
                }
                _ => {}
            }
        }
    }

    // 3. Windows fallback: ipconfig (read-only info)
    #[cfg(windows)]
    {
        if let Ok(ipconfig) = which("ipconfig.exe") {
            let status = Command::new(ipconfig)
                .args(&args)
                .status()
                .map_err(|e| anyhow!("ip: fallback ipconfig failed: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    Err(anyhow!("ip: no suitable backend found (tried ip/ipconfig/ifconfig)"))
} 