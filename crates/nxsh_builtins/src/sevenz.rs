//! `7z` builtin  Emulti-format archive tool (7-Zip).
//!
//! Delegates to system `7z`/`7zr`/`7za` binaries for full functionality.
//! If none are found, returns error (no lightweight fallback provided).

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn sevenz_cli(args: &[String]) -> Result<()> {
    let candidates = if cfg!(windows) {
        vec!["7z.exe", "7za.exe", "7zr.exe", "7z", "7za", "7zr"]
    } else {
        vec!["7z", "7za", "7zr"]
    };
    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path).args(args).status().map_err(|e| anyhow!("7z: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }
    Err(anyhow!("7z: backend not found; please install p7zip/7-Zip"))
} 

