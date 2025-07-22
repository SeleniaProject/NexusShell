//! `kill` builtin — send signals to processes.
//!
//! Usage: `kill [-SIGNAL] PID1 [PID2 ...]`
//! If `-SIGNAL` is omitted, defaults to `-15` (SIGTERM).
//! Only numeric signal values are currently supported for simplicity.
//!
//! Cross-platform notes:
//!   • On Unix-like targets, uses libc::kill.
//!   • On Windows, maps common signals to `TerminateProcess` or `GenerateConsoleCtrlEvent` where possible.
//!     For now, it only supports terminating the process (equivalent to SIGKILL).

use anyhow::{anyhow, Result};
use std::num::ParseIntError;

#[cfg(unix)]
use libc::{c_int, kill as libc_kill, pid_t};
#[cfg(windows)]
use windows_sys::Win32::{Foundation::HANDLE, System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE}};

pub fn kill_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("kill: missing PID"));
    }

    // Determine signal and starting index for PIDs.
    let (sig_num, pid_args) = if let Some(first) = args.first() {
        if first.starts_with('-') {
            let sig_str = &first[1..];
            let num: i32 = parse_signal(sig_str)?;
            (num, &args[1..])
        } else {
            (default_sig(), &args[..])
        }
    } else {
        (default_sig(), &args[..])
    };

    if pid_args.is_empty() {
        return Err(anyhow!("kill: no PID specified"));
    }

    for pid_str in pid_args {
        let pid: i32 = pid_str.parse().map_err(|e: ParseIntError| anyhow!("kill: invalid PID '{pid_str}': {e}"))?;
        send_signal(pid, sig_num)?;
    }
    Ok(())
}

fn parse_signal(s: &str) -> Result<i32> {
    if s.is_empty() {
        return Err(anyhow!("kill: invalid signal ''"));
    }
    if let Ok(num) = s.parse::<i32>() {
        Ok(num)
    } else {
        Err(anyhow!("kill: only numeric signals supported for now"))
    }
}

#[cfg(unix)]
fn default_sig() -> i32 {
    15 // SIGTERM
}
#[cfg(windows)]
fn default_sig() -> i32 {
    9 // emulate SIGKILL (terminate)
}

#[cfg(unix)]
fn send_signal(pid: i32, sig: i32) -> Result<()> {
    let res = unsafe { libc_kill(pid as pid_t, sig as c_int) };
    if res == 0 {
        Ok(())
    } else {
        Err(anyhow!("kill: failed to signal PID {pid}: errno {res}"))
    }
}

#[cfg(windows)]
fn send_signal(pid: i32, _sig: i32) -> Result<()> {
    unsafe {
        let handle: HANDLE = OpenProcess(PROCESS_TERMINATE, 0, pid as u32);
        if handle == 0 {
            return Err(anyhow!("kill: could not open process {pid}"));
        }
        if TerminateProcess(handle, 1) == 0 {
            return Err(anyhow!("kill: failed to terminate process {pid}"));
        }
    }
    Ok(())
} 