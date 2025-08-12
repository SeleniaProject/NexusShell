//! `umask` builtin  Eget or set the file creation mask.
//! Usage:
//!   umask           # print current mask in 4-digit octal (e.g., 0022)
//!   umask MODE      # set mask to MODE (octal)
//!   umask -S        # symbolic output (rwx style, not yet implemented)
//!
//! On Unix uses `libc::umask`. On Windows prints unsupported message.

use anyhow::Result;

#[cfg(unix)]
use nix::sys::stat::{umask, Mode};

pub fn umask_cli(_args: &[String]) -> Result<()> {
    #[cfg(windows)]
    {
        println!("umask: not supported on Windows");
        Ok(())
    }
    #[cfg(unix)]
    {
        if args.is_empty() {
            // Get current umask by setting to 0 and restoring
            let current = umask(Mode::empty());
            umask(current); // restore
            println!("{:04o}", current.bits());
            return Ok(());
        }
        if args[0] == "-S" {
            // symbolic representation not implemented yet
            let current = umask(Mode::empty());
            umask(current);
            println!("current mask {:04o}", current.bits());
            return Ok(());
        }
        let new_mask = u32::from_str_radix(&args[0], 8).map_err(|_| anyhow!("umask: invalid mode"))?;
        let _prev = umask(Mode::from_bits_truncate(new_mask));
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
