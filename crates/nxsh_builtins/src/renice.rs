//! `renice` builtin  Echange priority of running processes.
//!
//! Usage: `renice [-n] ADJUST PID...`
//! Accepts numeric nice value and list of PIDs. Positive values lower priority.
//!
//! Unix-only implementation; Windows not yet supported.

use anyhow::{anyhow, Result};
use std::num::ParseIntError;

#[cfg(unix)]
use nix::libc::{c_int, setpriority, PRIO_PROCESS};

pub fn renice_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("renice: missing arguments"));
    }

    let (adjust_str, pid_start) = if args[0] == "-n" {
        if args.len() < 3 {
            return Err(anyhow!("renice: -n requires ARG and PID"));
        }
        (&args[1], 2)
    } else {
        (&args[0], 1)
    };

    let adjust: i32 = adjust_str
        .parse()
        .map_err(|e: ParseIntError| anyhow!("renice: invalid adjustment '{adjust_str}': {e}"))?;

    if pid_start >= args.len() {
        return Err(anyhow!("renice: missing PID"));
    }

    #[cfg(windows)]
    {
        Err(anyhow!("renice: not supported on Windows yet"))
    }

    #[cfg(unix)]
    {
    for pid_str in &args[pid_start..] {
            let pid: i32 = pid_str.parse().map_err(|e: ParseIntError| anyhow!("renice: invalid PID '{pid_str}': {e}"))?;
            let res = unsafe { setpriority(PRIO_PROCESS as libc::__priority_which_t, pid as libc::id_t, adjust as c_int) };
            if res == -1 {
                return Err(anyhow!("renice: failed to set priority for PID {pid}"));
            }
        }
        Ok(())
    }
} 
