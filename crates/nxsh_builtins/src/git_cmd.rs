//! `git` builtin – wrapper around the external git command.
//!
//! All arguments are forwarded verbatim to the `git` binary found in PATH. The
//! builtin exists mainly to keep command discovery inside NexusShell while
//! delegating real work to git.

use anyhow::{anyhow, Result};
use std::process::Command;

pub async fn git_cli(args: &[String]) -> Result<()> {
    if which::which("git").is_err() {
        return Err(anyhow!("git: external 'git' command not found"));
    }
    let status = Command::new("git").args(args).status()?;
    if !status.success() {
        return Err(anyhow!("git: exited with status {}", status));
    }
    Ok(())
} 