//! `chgrp` builtin — change group ownership of files.
//!
//! Primary behaviour:
//! 1. Execute system `chgrp` binary for full flag coverage.
//! 2. Fallback: accept numeric GID and call `libc::chown` with uid=-1.
//!    Recursive and symbolic modes are not supported in the fallback.
//!
//! Example fallback: `chgrp 1000 file.txt`.

use anyhow::{anyhow, Context, Result};
use std::{ffi::CString, os::unix::ffi::OsStrExt, path::Path, process::Command};
use which::which;

pub fn chgrp_cli(args: &[String]) -> Result<()> {
    // First try system chgrp
    if let Ok(path) = which("chgrp") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("chgrp: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // Minimal fallback: chgrp GID FILE...
    if args.len() < 2 {
        return Err(anyhow!("chgrp: missing GROUP or FILE"));
    }

    let gid: i32 = args[0]
        .parse()
        .map_err(|_| anyhow!("chgrp: fallback expects numeric GID"))?;

    for file in &args[1..] {
        let path = Path::new(file);
        if !path.exists() {
            return Err(anyhow!("chgrp: '{}' does not exist", file));
        }
        let c_path = CString::new(path.as_os_str().as_bytes())?;
        let res = unsafe { libc::chown(c_path.as_ptr(), -1_i32 as u32, gid as u32) };
        if res != 0 {
            return Err(anyhow!("chgrp: failed to change group for '{}'", file));
        }
    }

    Ok(())
} 