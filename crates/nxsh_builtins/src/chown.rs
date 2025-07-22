//! `chown` builtin — change file owner and group.
//!
//! Primary behaviour:
//! 1. Execute system `chown` binary to leverage full option support.
//! 2. Fallback: support numeric UID[:GID] ownership change for files provided,
//!    using `libc::chown`. This requires sufficient privileges.
//!    Symbolic owner names and recursion are not handled in fallback.
//!
//! Example fallback usage: `chown 1000:1000 file.txt`.

use anyhow::{anyhow, Context, Result};
use std::{ffi::CString, os::unix::ffi::OsStrExt, path::Path, process::Command};
use which::which;

pub fn chown_cli(args: &[String]) -> Result<()> {
    // 1. Delegate if system chown exists
    if let Ok(path) = which("chown") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("chown: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // 2. Minimal fallback: chown UID[:GID] FILE...
    if args.len() < 2 {
        return Err(anyhow!("chown: missing OWNER or FILE"));
    }

    let owner_spec = &args[0];
    let mut split = owner_spec.split(':');
    let uid_str = split.next().unwrap();
    let gid_str = split.next();
    let uid: u32 = uid_str.parse().map_err(|_| anyhow!("chown: fallback expects numeric UID"))?;
    let gid: i32 = if let Some(gid_s) = gid_str {
        gid_s.parse().map_err(|_| anyhow!("chown: fallback expects numeric GID"))?
    } else {
        -1
    } as i32;

    for file in &args[1..] {
        let path = Path::new(file);
        if !path.exists() {
            return Err(anyhow!("chown: '{}' does not exist", file));
        }
        let c_path = CString::new(path.as_os_str().as_bytes())?;
        let res = unsafe { libc::chown(c_path.as_ptr(), uid, gid as u32) };
        if res != 0 {
            return Err(anyhow!("chown: failed to change ownership for '{}'", file));
        }
    }
    Ok(())
} 