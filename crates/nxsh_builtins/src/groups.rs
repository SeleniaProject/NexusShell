//! `groups` builtin — display group memberships.
//!
//! Strategy: if external `groups` binary exists, delegate to it, passing all args.
//! Otherwise, on Unix fallback to parsing `id -Gn` output.
//! Windows not supported yet.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn groups_cli(args: &[String]) -> Result<()> {
    // Try external binary first for full feature parity.
    if let Ok(path) = which("groups") {
        let status = Command::new(path).args(args).status()?;
        if status.success() {
            return Ok(());
        } else {
            return Err(anyhow!("groups: external binary exited with status {:?}", status.code()));
        }
    }

    // Fallback Unix-only: use `id -Gn [USER]`.
    #[cfg(unix)]
    {
        let mut cmd = Command::new("id");
        cmd.arg("-Gn");
        cmd.args(args);
        let status = cmd.status().map_err(|e| anyhow!("groups: failed fallback: {e}"))?;
        if status.success() {
            return Ok(());
        }
    }

    Err(anyhow!("groups: no backend available"))
} 