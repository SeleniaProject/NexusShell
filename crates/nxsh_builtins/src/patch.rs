//! `patch` command  Eapply unified context diffs.
//!
//! This built-in is a thin wrapper around the system `patch` utility to avoid
//! re-implementing a full diff parser. It supports the most common workflow:
//!
//!   patch [-pNUM] [< DIFF]            # stdin
//!   patch [-pNUM] -i FILE.diff        # explicit file
//!
//! Recognised options:
//!   -pNUM : strip NUM leading path components (default 0)
//!   -i FILE: read patch from FILE instead of stdin
//!
//! All other options are forwarded verbatim to the backend `patch` command.
//!
//! Backend resolution order:
//!   1. `$NXSH_PATCH_CMD` if set
//!   2. `gpatch` (GNU patch) if found in PATH
//!   3. `patch`
//!
//! If no suitable backend is found an error is returned.

use anyhow::{anyhow, Result};
use std::env;
use std::process::Command;

pub fn patch_cli(args: &[String]) -> Result<()> {
    let backend = find_backend()?;

    let status = Command::new(&backend)
        .args(args)
        .status()
        .map_err(|e| anyhow!("patch: failed to launch '{}': {}", backend, e))?;

    if !status.success() {
        return Err(anyhow!("patch: backend exited with status {:?}", status.code()));
    }
    Ok(())
}

fn find_backend() -> Result<String> {
    if let Ok(cmd) = env::var("NXSH_PATCH_CMD") {
        return Ok(cmd);
    }
    for candidate in ["gpatch", "patch"].iter() {
        if which::which(candidate).is_ok() {
            return Ok(candidate.to_string());
        }
    }
    Err(anyhow!("patch: no backend command found in PATH"))
} 

