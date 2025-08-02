//! `getfacl` builtin  Edisplay file access control lists.
//!
//! Delegates to system `getfacl` for accurate ACL display. No fallback.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn getfacl_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("getfacl") {
        let status = Command::new(path).args(args).status().map_err(|e| anyhow!("getfacl: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    Err(anyhow!("getfacl: backend not found; please install getfacl package"))
} 
