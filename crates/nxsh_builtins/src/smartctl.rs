//! `smartctl` builtin  Edisplay S.M.A.R.T. information for a disk.
//!
//! This is a thin wrapper around the external `smartctl` utility from
//! smartmontools. It forwards arguments and prints the command output, allowing
//! NexusShell users to get detailed health data without leaving the shell.
//!
//! Usage:
//!     smartctl DEVICE                 # full SMART report
//!     smartctl -H DEVICE              # health summary
//!
//! Limitations:
//! * Requires `smartctl` binary in PATH. If not present, a helpful message is
//!   shown.
//! * No parsing is done  Eoutput is streamed directly.
//! * On non-Unix systems the command is currently unsupported.

use anyhow::{anyhow, Result};
#[cfg(unix)] use std::process::Command;

pub async fn smartctl_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("smartctl: missing DEVICE"));
    }

    #[cfg(not(unix))]
    { println!("smartctl: unsupported on this platform"); Ok(()) }
    #[cfg(unix)] {
        if which::which("smartctl").is_err() { println!("smartctl: external 'smartctl' command not found. Install smartmontools."); return Ok(()); }
        let mut cmd = Command::new("smartctl");
        if args[0].starts_with('-') { for a in args { cmd.arg(a); } }
        else { cmd.arg("-a"); for a in args { cmd.arg(a); } }
        let status = cmd.status()?;
        if !status.success() { return Err(anyhow!("smartctl: external command exited with status {}", status)); }
        Ok(())
    }
} 
