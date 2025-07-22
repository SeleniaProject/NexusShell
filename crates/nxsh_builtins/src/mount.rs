//! `mount` command – mount filesystem (simple wrapper).
//! Usage: mount SRC DIR [TYPE] [OPTIONS]
//! Only works on Unix; Windows outputs unsupported message.

use anyhow::{anyhow, Result};

pub async fn mount_cli(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        return Err(anyhow!("mount: missing operands"));
    }
    #[cfg(windows)]
    {
        println!("mount: not supported on Windows");
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::unix::prelude::OsStrExt;
        let src = CString::new(args[0].as_bytes())?;
        let dir = CString::new(args[1].as_bytes())?;
        let fstype = if args.len() >= 3 { CString::new(args[2].as_bytes())? } else { CString::new("".as_bytes())? };
        let flags = 0;
        let data = std::ptr::null();
        let res = unsafe { libc::mount(src.as_ptr(), dir.as_ptr(), if fstype.as_bytes().is_empty() { std::ptr::null() } else { fstype.as_ptr() }, flags, data) };
        if res != 0 {
            return Err(anyhow!("mount: failed (errno {})", std::io::Error::last_os_error()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests { use super::*; #[tokio::test] async fn mount_stub(){ let _ = mount_cli(&["/dev/null".into(), "/mnt/null".into()]).await; }} 