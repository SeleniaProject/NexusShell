//! `pkill` builtin — send signals to processes matched by name (regex).
//!
//! Usage: `pkill [-SIGNAL] PATTERN`
//! If `-SIGNAL` is omitted, defaults to SIGTERM (15).
//! Currently supports numeric signal only, pattern is POSIX ERE (regex).

use anyhow::{anyhow, Result};
use regex::Regex;
use sysinfo::{ProcessExt, System, SystemExt, PidExt};
use std::num::ParseIntError;

#[cfg(unix)]
use libc::{c_int, kill as libc_kill, pid_t};
#[cfg(windows)]
use windows_sys::Win32::{Foundation::HANDLE, System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE}};

pub fn pkill_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("pkill: missing PATTERN"));
    }
    let (sig_num, pattern) = if args[0].starts_with('-') {
        let sig_str = &args[0][1..];
        let num: i32 = parse_signal(sig_str)?;
        if args.len() < 2 {
            return Err(anyhow!("pkill: missing PATTERN"));
        }
        (num, &args[1])
    } else {
        (default_sig(), &args[0])
    };

    let re = Regex::new(pattern).map_err(|e| anyhow!("pkill: invalid regex: {e}"))?;

    let mut sys = System::new_all();
    sys.refresh_processes();

    let mut matched = false;
    for (pid, proc_) in sys.processes() {
        if re.is_match(proc_.name()) {
            matched = true;
            send_signal(pid.as_u32() as i32, sig_num)?;
        }
    }
    if !matched {
        return Err(anyhow!("pkill: no process matched"));
    }
    Ok(())
}

fn parse_signal(s: &str) -> Result<i32> {
    if let Ok(num) = s.parse::<i32>() {
        Ok(num)
    } else {
        Err(anyhow!("pkill: only numeric signals supported for now"))
    }
}

#[cfg(unix)]
fn default_sig() -> i32 { 15 }
#[cfg(windows)]
fn default_sig() -> i32 { 9 }

#[cfg(unix)]
fn send_signal(pid: i32, sig: i32) -> Result<()> {
    let res = unsafe { libc_kill(pid as pid_t, sig as c_int) };
    if res == 0 {
        Ok(())
    } else {
        Err(anyhow!("pkill: failed to signal PID {pid}: errno {res}"))
    }
}

#[cfg(windows)]
fn send_signal(pid: i32, _sig: i32) -> Result<()> {
    unsafe {
        let handle: HANDLE = OpenProcess(PROCESS_TERMINATE, 0, pid as u32);
        if handle == 0 {
            return Err(anyhow!("pkill: could not open process {pid}"));
        }
        if TerminateProcess(handle, 1) == 0 {
            return Err(anyhow!("pkill: failed to terminate process {pid}"));
        }
    }
    Ok(())
} 