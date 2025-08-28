//! `chmod` builtin  Echange file permissions.
//!
//! Preferred behaviour:
//! 1. Execute platform `chmod` binary for complete POSIX flag support.
//! 2. If `chmod` is absent (rare), provide a minimal fallback supporting
//!    numeric modes (`chmod 644 file`). Symbolic modes and recursion are **not**
//!    supported in the fallback.
//!
//! This approach keeps the codebase small while still functioning in minimal
//! container images where coreutils may be missing.

use anyhow::{anyhow, Context, Result};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::Path, process::Command};
use which::which;

pub fn chmod_cli(args: &[String]) -> Result<()> {
    // 1. Try system chmod
    if let Ok(chmod_bin) = which("chmod") {
        let status = Command::new(chmod_bin)
            .args(args)
            .status()
            .map_err(|e| anyhow!("chmod: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // 2. Minimal fallback: chmod NUMERIC_MODE FILE...
    if args.len() < 2 {
        return Err(anyhow!("chmod: missing MODE or FILE"));
    }

    let mode_str = &args[0];
    let mode = u32::from_str_radix(mode_str, 8)
        .map_err(|_| anyhow!("chmod: fallback supports only octal modes (e.g., 644)"))?;
    for file in &args[1..] {
        let path = Path::new(file);
        let metadata = fs::metadata(path)
            .with_context(|| format!("chmod: cannot access '{file}'"))?;
        let mut perms = metadata.permissions();
        
        // Platform-specific permission setting
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(mode);
        }
        
        #[cfg(windows)]
        {
            // On Windows, we can only set read-only attribute
            // Setting execute/write permissions is more complex
            perms.set_readonly((mode & 0o200) == 0);
        }
        
        fs::set_permissions(path, perms)
            .with_context(|| format!("chmod: failed to set permissions for '{file}'"))?;
    }
    Ok(())
}

/// Execute function for chmod command
pub fn execute(args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    match chmod_cli(args) {
        Ok(_) => Ok(0),
        Err(e) => {
            eprintln!("{e}");
            Ok(1)
        }
    }
}

