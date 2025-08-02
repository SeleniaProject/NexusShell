//! `ulimit` builtin  Eview or set resource limits.
//! Supported options (subset):
//!   -a  : list all limits
//!   -n N: set/open files soft limit
//!   -c N: set core file size soft limit
//! When setting, `N` may be `unlimited`.
//! On Windows this builtin only prints an unsupported message.

use anyhow::{anyhow, Result};

#[cfg(unix)]
use nix::libc::{getrlimit, setrlimit, rlimit, RLIMIT_CORE, RLIMIT_NOFILE};

pub fn ulimit_cli(args: &[String]) -> Result<()> {
    #[cfg(windows)]
    {
        println!("ulimit: not supported on Windows");
        return Ok(());
    }
    #[cfg(unix)]
    {
        if args.is_empty() || args[0] == "-a" {
            print_limit("core file size", RLIMIT_CORE as i32)?;
            print_limit("open files", RLIMIT_NOFILE as i32)?;
            return Ok(());
        }
        if args.len() == 2 {
            let limit_type = match args[0].as_str() {
                "-n" => RLIMIT_NOFILE,
                "-c" => RLIMIT_CORE,
                _ => return Err(anyhow!("ulimit: unsupported option")),
            };
            let val = if args[1] == "unlimited" { libc::RLIM_INFINITY } else { args[1].parse::<u64>()? };
            let new_lim = rlimit { rlim_cur: val, rlim_max: val };
            unsafe { setrlimit(limit_type, &new_lim) };
            return Ok(());
        }
        Err(anyhow!("ulimit: invalid usage"))
    }
}

#[cfg(unix)]
fn print_limit(name: &str, res: i32) -> Result<()> {
    unsafe {
        let mut lim: rlimit = std::mem::zeroed();
        getrlimit(res as u32, &mut lim);
        let v = if lim.rlim_cur == libc::RLIM_INFINITY { "unlimited".into() } else { lim.rlim_cur.to_string() };
        println!("{}: {}", name, v);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn list_limits() {
        let _ = ulimit_cli(&["-a".into()]);
    }
} 
