//! `ss` builtin — socket statistics utility.
//!
//! This wrapper attempts to invoke the modern `ss` command (from `iproute2`)
//! which supersedes `netstat` on most Linux distributions. If `ss` is not
//! available, we gracefully fall back to `netstat -an` to provide similar
//! information.
//!
//! All arguments provided to the builtin are forwarded unchanged to the chosen
//! backend binary.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn ss_cli(args: &[String]) -> Result<()> {
    // Preferred: ss
    if let Ok(path) = which("ss") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("ss: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // Fallback: netstat -an plus user args
    if let Ok(netstat) = which(if cfg!(windows) { "netstat.exe" } else { "netstat" }) {
        // Pre-prepend default flags when no args are specified to mimic `ss` default.
        let mut forwarded: Vec<String> = if args.is_empty() {
            vec!["-an".to_string()] // show all sockets numerical
        } else {
            Vec::new()
        };
        forwarded.extend_from_slice(args);

        let status = Command::new(netstat)
            .args(&forwarded)
            .status()
            .map_err(|e| anyhow!("ss: fallback netstat failed: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    Err(anyhow!("ss: neither 'ss' nor 'netstat' found in PATH"))
} 