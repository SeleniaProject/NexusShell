//! `netstat` builtin — socket status overview.
//!
//! Delegates to the platform's `netstat` (or equivalent) command. On many
//! modern Linux distributions `ss` replaces `netstat`; therefore, if `netstat`
//! is missing we also attempt to call `ss` with a compatible argument set.
//!
//! All command-line arguments are forwarded verbatim to preserve familiar
//! behaviour. No internal implementation is attempted because parsing kernel
//! socket tables is highly platform-specific and already handled efficiently by
//! existing utilities.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn netstat_cli(args: &[String]) -> Result<()> {
    // Preferred backends in order.
    let backends = if cfg!(windows) {
        vec!["netstat.exe", "netstat"]
    } else {
        vec!["netstat", "ss"]
    };

    for bin in backends {
        if let Ok(path) = which(bin) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("netstat: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    Err(anyhow!(
        "netstat: no suitable backend (`netstat` or `ss`) found in PATH"
    ))
} 