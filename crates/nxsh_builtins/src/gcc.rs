//! `gcc` builtin – wrapper around GNU C Compiler.
//!
//! Forwards all arguments to external `gcc`. If not found, prints error.

use anyhow::{anyhow, Result};
use std::process::Command;

pub async fn gcc_cli(args: &[String]) -> Result<()> {
    if which::which("gcc").is_err() {
        return Err(anyhow!("gcc: external 'gcc' command not found"));
    }
    let status = Command::new("gcc").args(args).status()?;
    if !status.success() {
        return Err(anyhow!("gcc: exited with status {}", status));
    }
    Ok(())
} 