//! `timedatectl` builtin – interface to systemd-timesyncd/time settings.
//!
//! This is a thin pass-through wrapper to the external `timedatectl` command
//! (part of systemd). If the binary is unavailable, an explanatory message is
//! shown. On non-Unix systems, the command is unsupported.
//!
//! All arguments are forwarded verbatim.

use anyhow::{anyhow, Result};
use std::process::Command;

pub async fn timedatectl_cli(args: &[String]) -> Result<()> {
    #[cfg(not(unix))]
    {
        println!("timedatectl: unsupported on this platform");
        return Ok(());
    }

    #[cfg(unix)]
    {
        if which::which("timedatectl").is_err() {
            return Err(anyhow!("timedatectl: external 'timedatectl' not found"));
        }
        let status = Command::new("timedatectl").args(args).status()?;
        if !status.success() {
            return Err(anyhow!("timedatectl: exited with status {}", status));
        }
        Ok(())
    }
} 