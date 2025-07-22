//! `ifconfig` builtin — legacy network interface configuration.
//!
//! On modern Linux systems, `ifconfig` is deprecated in favour of `ip addr`,
//! but many scripts still rely on it. This builtin simply searches for a
//! platform `ifconfig` (or Windows `ipconfig.exe`) and executes it with all
//! passed arguments, ensuring backward compatibility without needing the user
//! to install an additional package manually.
//!
//! No fallback parsing is attempted beyond basic delegation because interface
//! management is highly platform‐specific.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn ifconfig_cli(args: &[String]) -> Result<()> {
    // Candidate list varies per OS
    let candidates = if cfg!(windows) {
        vec!["ipconfig.exe"]
    } else if cfg!(target_os = "macos") {
        vec!["ifconfig"] // present by default
    } else {
        vec!["ifconfig"] // may be provided by net-tools
    };

    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("ifconfig: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    Err(anyhow!(
        "ifconfig: backend not found; install 'net-tools' or use 'ip addr' instead"
    ))
} 