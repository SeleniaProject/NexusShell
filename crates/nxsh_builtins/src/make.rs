//! `make` builtin – wrapper around GNU make / BSD make.
//!
//! Simply forwards all arguments to external `make`. If not available, prints
//! an informative error.

use anyhow::{anyhow, Result};
use std::process::Command;

pub async fn make_cli(args: &[String]) -> Result<()> {
    if which::which("make").is_err() {
        return Err(anyhow!("make: external 'make' command not found"));
    }
    let status = Command::new("make").args(args).status()?;
    if !status.success() {
        return Err(anyhow!("make: exited with status {}", status));
    }
    Ok(())
} 