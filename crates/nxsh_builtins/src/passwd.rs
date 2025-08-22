//! `passwd` builtin  Echange user password.
//!
//! Delegates to system `passwd` binary. No fallback implementation because
//! password database handling is platform-specific and privileged.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn passwd_cli(args: &[String]) -> Result<()> {
    let candidates = if cfg!(windows) {
        vec!["passwd.exe", "passwd"]
    } else {
        vec!["passwd"]
    };
    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path).args(args).status().map_err(|e| anyhow!("passwd: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }
    Err(anyhow!("passwd: backend not found; please install passwd utility"))
} 

