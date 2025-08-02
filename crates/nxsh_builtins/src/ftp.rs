//! `ftp` builtin  Einteractive file transfer client.
//!
//! NexusShell delegates all FTP functionality to existing mature clients
//! instead of re-implementing the protocol. The builtin examines the PATH for
//! common binaries in the following order:
//! 1. `ftp` (BSD / GNU inetutils) or `ftp.exe` on Windows.
//! 2. `lftp` (feature-rich alternative).
//!
//! All command-line arguments are forwarded unchanged so that advanced flags
//! like `-p`, `-n` or `-g` behave exactly as users expect. If no compatible
//! backend is found, an informative error is returned.
//!
//! Note: For scripted FTP transfers, prefer the more secure `curl` / `wget`
//! with `ftp://` URLs or modern secure protocols (SFTP/FTPS via `scp`, `sftp`,
//! or `curl`).

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn ftp_cli(args: &[String]) -> Result<()> {
    // 1. Try classic ftp binaries
    let candidates = if cfg!(windows) {
        vec!["ftp.exe", "ftp"]
    } else {
        vec!["ftp"]
    };

    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("ftp: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    // 2. Fallback to lftp if present
    if let Ok(lftp) = which("lftp") {
        let status = Command::new(lftp)
            .args(args)
            .status()
            .map_err(|e| anyhow!("ftp: fallback lftp failed: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    Err(anyhow!("ftp: no suitable client found (tried ftp/lftp)"))
} 
