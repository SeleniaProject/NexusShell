//! `who` builtin — list logged-in users.
//!
//! Delegates to external `who` or `who.exe` binary if found.
//! If not present, returns error.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn who_cli(args: &[String]) -> Result<()> {
    for bin in ["who", "who.exe"].iter() {
        if let Ok(path) = which(bin) {
            let status = Command::new(path).args(args).status()?;
            if status.success() {
                return Ok(());
            } else {
                return Err(anyhow!("who: external binary exited with status {:?}", status.code()));
            }
        }
    }
    Err(anyhow!("who: no compatible backend found"))
} 