//! `setfacl` builtin — set file access control lists.
//!
//! For full capability support, this builtin simply delegates to the system
//! `setfacl` binary. No portable fallback is provided because ACL
//! implementation details vary across filesystems and operating systems.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn setfacl_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("setfacl") {
        let status = Command::new(path).args(args).status().map_err(|e| anyhow!("setfacl: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    Err(anyhow!("setfacl: backend not found; please install setfacl package"))
} 