//! `umask` builtin – get or set the file creation mask.
//! Usage:
//!   umask           # print current mask in 4-digit octal (e.g., 0022)
//!   umask MODE      # set mask to MODE (octal)
//!   umask -S        # symbolic output (rwx style, not yet implemented)
//!
//! On Unix uses `libc::umask`. On Windows prints unsupported message.

use anyhow::{anyhow, Result};

pub fn umask_cli(args: &[String]) -> Result<()> {
    #[cfg(windows)]
    {
        println!("umask: not supported on Windows");
        return Ok(());
    }
    #[cfg(unix)]
    unsafe {
        if args.is_empty() {
            let current = libc::umask(0);
            libc::umask(current); // restore
            println!("{:04o}", current);
            return Ok(());
        }
        if args[0] == "-S" {
            // symbolic representation not implemented yet
            let current = libc::umask(0);
            libc::umask(current);
            println!("current mask {:04o}", current);
            return Ok(());
        }
        let new_mask = u32::from_str_radix(&args[0], 8).map_err(|_| anyhow!("umask: invalid mode"))?;
        let _prev = libc::umask(new_mask as libc::mode_t);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_umask() {
        let _ = umask_cli(&[]);
    }
} 