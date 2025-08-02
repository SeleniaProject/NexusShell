//! `cpio` builtin  Earchive tool compatible with POSIX cpio format.
//!
//! For complete feature coverage (copy-in/out, various formats, SELinux/xattrs
//! preservation, etc.) we delegate execution to the platform `cpio` binary.
//! When `cpio` is not found in `PATH`, we return an informative error because
//! faithfully re-implementing cpio would require a large codebase.
//!
//! All command-line arguments are forwarded verbatim.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn cpio_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("cpio") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("cpio: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    Err(anyhow!("cpio: backend not found; please install cpio package"))
} 
