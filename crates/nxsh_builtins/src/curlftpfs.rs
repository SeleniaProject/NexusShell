//! `curlftpfs` builtin  Emount FTP server via FUSE.
//!
//! This builtin simply invokes the system `curlftpfs` binary if present in
//! `PATH`, forwarding all arguments. No fallback is provided because mounting a
//! FUSE filesystem requires kernel/userland components that cannot be
//! replicated easily from the shell.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn curlftpfs_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("curlftpfs") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("curlftpfs: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    Err(anyhow!("curlftpfs: backend not found; ensure curlftpfs is installed"))
} 
