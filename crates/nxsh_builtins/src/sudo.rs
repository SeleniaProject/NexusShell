//! `sudo` builtin  Eprivilege escalation wrapper.
//!
//! Simply delegates to system `sudo` binary (or `runas` on Windows). No fallback
//! implementation is provided.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn sudo_cli(args: &[String]) -> Result<()> {
    let candidates = if cfg!(windows) {
        vec!["sudo.exe", "sudo", "runas.exe"]
    } else {
        vec!["sudo"]
    };
    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path).args(args).status().map_err(|e| anyhow!("sudo: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }
    Err(anyhow!("sudo: backend not found; please install sudo"))
} 

