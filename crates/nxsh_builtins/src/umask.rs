//! `umask` builtin  Eget or set the file creation mask.
//! Usage:
//!   umask           # print current mask in 4-digit octal (e.g., 0022)
//!   umask MODE      # set mask to MODE (octal)
//!   umask -S        # symbolic output (rwx style, not yet implemented)
//!
//! On Unix uses `libc::umask`. On Windows prints unsupported message.

use anyhow::{Result, anyhow};

#[cfg(unix)]
use nix::sys::stat::{umask, Mode};

pub fn umask_cli(args: &[String]) -> Result<()> {
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
            // Print symbolic permissions allowed (complement of mask)
            let current = umask(Mode::empty());
            umask(current);
            let sym = symbolic_from_mask(current.bits());
            println!("{}", sym);
            return Ok(());
        }
        let new_mask = u32::from_str_radix(&args[0], 8).map_err(|_| anyhow!("umask: invalid mode"))?;
        let _prev = umask(Mode::from_bits_truncate(new_mask));
        Ok(())
    }
}

#[cfg(unix)]
fn symbolic_from_mask(mask_bits: u32) -> String {
    // Allowed permissions = 0777 & !mask
    let allowed = 0o777 & (!mask_bits);
    let u = (allowed >> 6) & 0o7;
    let g = (allowed >> 3) & 0o7;
    let o = allowed & 0o7;
    fn to_rwx(bits: u32) -> String {
        let r = if bits & 0b100 != 0 { 'r' } else { '-' };
        let w = if bits & 0b010 != 0 { 'w' } else { '-' };
        let x = if bits & 0b001 != 0 { 'x' } else { '-' };
        format!("{}{}{}", r,w,x)
    }
    format!("u={},g={},o={}", to_rwx(u), to_rwx(g), to_rwx(o))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_umask() {
        let _ = umask_cli(&[]);
    }
} 
