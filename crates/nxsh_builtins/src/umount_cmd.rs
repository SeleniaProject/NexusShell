//! `umount` command  Eunmount filesystem.
//! Usage: umount DIR

use anyhow::{anyhow, Result};

pub async fn umount_cli(args: &[String]) -> Result<()> {
    if args.is_empty() { return Err(anyhow!("umount: missing operand")); }
    #[cfg(windows)]
    {
        println!("umount: not supported on Windows");
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::ffi::CString;
        let dir = CString::new(args[0].as_bytes())?;
        let res = unsafe { libc::umount2(dir.as_ptr(), 0) };
        if res != 0 {
            return Err(anyhow!("umount: failed (errno {})", std::io::Error::last_os_error()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests { use super::*; #[tokio::test] async fn umount_stub(){ let _ = umount_cli(&["/mnt/null".into()]).await; }} 

