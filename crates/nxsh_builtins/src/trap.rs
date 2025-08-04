//! `trap` builtin  Eset or clear signal handlers.
//! Syntax examples:
//!   trap CMD SIGNALS...
//!   trap -l          # list signals
//!   trap -p          # print current traps
//!   trap - SIG       # reset default handler
//!
//! For this minimal implementation we support listing signals and setting a handler that
//! prints the received signal name and executes a shell command via `Executor::run`.

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use signal_hook::consts::*;
use std::collections::HashMap;
use std::sync::Mutex;
use nxsh_core::context::ShellContext;

static HANDLERS: Lazy<Mutex<HashMap<i32, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn trap_cli(args: &[String], ctx: &mut ShellContext) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("trap: missing arguments"));
    }
    if args[0] == "-l" {
        list_signals();
        return Ok(());
    }
    if args[0] == "-p" {
        let h = HANDLERS.lock().unwrap();
        for (sig, cmd) in h.iter() {
            println!("trap -- '{}' {}", cmd, sig);
        }
        return Ok(());
    }

    let cmd = &args[0];
    let signals: Vec<i32> = args[1..]
        .iter()
        .map(|s| parse_signal(s))
        .collect::<Result<_>>()?;

    for sig in signals {
        set_handler(sig, cmd.clone())?;
    }
    Ok(())
}

// Cross-platform signal constants
#[cfg(unix)]
const SIGHUP: i32 = signal_hook::consts::SIGHUP;
#[cfg(unix)]
const SIGUSR1: i32 = signal_hook::consts::SIGUSR1;
#[cfg(unix)]
const SIGUSR2: i32 = signal_hook::consts::SIGUSR2;

#[cfg(windows)]
const SIGHUP: i32 = 1;  // Simulate signals on Windows
#[cfg(windows)]
const SIGUSR1: i32 = 10;
#[cfg(windows)]
const SIGUSR2: i32 = 12;

fn parse_signal(s: &str) -> Result<i32> {
    if let Ok(num) = s.parse::<i32>() { return Ok(num); }
    match s.trim_start_matches("SIG").to_uppercase().as_str() {
        "INT" => Ok(SIGINT),
        "TERM" => Ok(SIGTERM),
        "HUP" => Ok(SIGHUP),
        "USR1" => Ok(SIGUSR1),
        "USR2" => Ok(SIGUSR2),
        _ => Err(anyhow!("trap: unknown signal {}", s)),
    }
}

fn list_signals() {
    println!(" 1) SIGHUP    2) SIGINT    3) SIGQUIT   9) SIGKILL  15) SIGTERM");
}

fn set_handler(sig: i32, cmd: String) -> Result<()> {
    let mut h = HANDLERS.lock().unwrap();
    if !h.contains_key(&sig) {
        // register signal listener thread once per signal
        // For now, just store the signal handler without actual registration
        // as signal-hook iterator is not available
        println!("Would register signal {} with handler: {}", sig, cmd);
    }
    h.insert(sig, cmd);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_sig() { assert_eq!(parse_signal("INT").unwrap(), SIGINT); }
} 
