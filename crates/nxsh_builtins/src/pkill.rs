//! `pkill` builtin  Esend signals to processes matched by name (regex).
//!
//! Usage: `pkill [-SIGNAL] PATTERN`
//! If `-SIGNAL` is omitted, defaults to SIGTERM (15).
//! Currently supports numeric signal only, pattern is POSIX ERE (regex).

use anyhow::{anyhow, Result};
use regex::Regex;
#[cfg(feature = "system-info")]
use sysinfo::{ProcessExt, System, SystemExt, PidExt};

#[cfg(unix)]
use nix::libc::{c_int, kill as libc_kill, pid_t};
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
    // Try numeric first
    if let Ok(num) = s.parse::<i32>() {
        return Ok(num);
    }

    // Handle signal names (with or without SIG prefix)
    let signal_name = if s.starts_with("SIG") {
        &s[3..]
    } else {
        s
    };

    match signal_name.to_uppercase().as_str() {
        "HUP" => Ok(1),
        "INT" => Ok(2),
        "QUIT" => Ok(3),
        "ILL" => Ok(4),
        "TRAP" => Ok(5),
        "ABRT" | "IOT" => Ok(6),
        "BUS" => Ok(7),
        "FPE" => Ok(8),
        "KILL" => Ok(9),
        "USR1" => Ok(10),
        "SEGV" => Ok(11),
        "USR2" => Ok(12),
        "PIPE" => Ok(13),
        "ALRM" => Ok(14),
        "TERM" => Ok(15),
        "STKFLT" => Ok(16),
        "CHLD" | "CLD" => Ok(17),
        "CONT" => Ok(18),
        "STOP" => Ok(19),
        "TSTP" => Ok(20),
        "TTIN" => Ok(21),
        "TTOU" => Ok(22),
        "URG" => Ok(23),
        "XCPU" => Ok(24),
        "XFSZ" => Ok(25),
        "VTALRM" => Ok(26),
        "PROF" => Ok(27),
        "WINCH" => Ok(28),
        "IO" | "POLL" => Ok(29),
        "PWR" => Ok(30),
        "SYS" => Ok(31),
        _ => Err(anyhow!("pkill: unknown signal name '{}'", s)),
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
        if handle == std::ptr::null_mut() {
            return Err(anyhow!("pkill: could not open process {pid}"));
        }
        if TerminateProcess(handle, 1) == 0 {
            return Err(anyhow!("pkill: failed to terminate process {pid}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_signal_numeric() {
        assert_eq!(parse_signal("15").unwrap(), 15);
        assert_eq!(parse_signal("9").unwrap(), 9);
    }

    #[test]
    fn test_parse_signal_names() {
        assert_eq!(parse_signal("TERM").unwrap(), 15);
        assert_eq!(parse_signal("SIGTERM").unwrap(), 15);
        assert_eq!(parse_signal("KILL").unwrap(), 9);
        assert_eq!(parse_signal("SIGKILL").unwrap(), 9);
        assert_eq!(parse_signal("HUP").unwrap(), 1);
        assert_eq!(parse_signal("INT").unwrap(), 2);
    }

    #[test]
    fn test_parse_signal_case_insensitive() {
        assert_eq!(parse_signal("term").unwrap(), 15);
        assert_eq!(parse_signal("kill").unwrap(), 9);
        assert_eq!(parse_signal("hup").unwrap(), 1);
    }

    #[test]
    fn test_parse_signal_invalid() {
        assert!(parse_signal("INVALID").is_err());
        assert!(parse_signal("999").is_ok()); // Large numbers are allowed
    }
} 

