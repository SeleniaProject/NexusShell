//! `ssh` builtin  Esecure shell client wrapper.
//!
//! NexusShell intentionally leverages the platform-native OpenSSH client for
//! full protocol compatibility, advanced crypto support, and decades of battle
//!-tested reliability. When a compatible `ssh` executable is present in the
//! `PATH`, we simply re-exec it, forwarding every command-line argument so that
//! users can rely on 100% behavioural parity with their existing workflows.
//!
//! If the binary is not found, an error is returned suggesting installation
//! instructions. Implementing a full SSH stack in Rust would be outside the
//! immediate scope of NexusShell and would risk diverging from OpenSSH’s proven
//! security record.
//!
//! Note: Windows users may have `ssh.exe` bundled with recent Win10/11 or via
//! Git for Windows. macOS and most Linux distros ship `ssh` by default.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

/// Entry point for the `ssh` builtin.
pub fn ssh_cli(args: &[String]) -> Result<()> {
    // Candidate executable names in preferred order.
    let candidates = if cfg!(windows) {
        vec!["ssh.exe", "ssh"]
    } else {
        vec!["ssh"]
    };

    for bin in candidates {
        if let Ok(path) = which(bin) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("ssh: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    Err(anyhow!("ssh: no compatible ssh client found in PATH; please install OpenSSH"))
}

pub fn execute(args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    match ssh_cli(args) {
        Ok(()) => Ok(0),
        Err(e) => Err(crate::common::BuiltinError::Other(e.to_string())),
    }
}
