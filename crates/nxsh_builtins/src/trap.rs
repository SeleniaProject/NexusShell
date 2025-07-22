//! `trap` builtin – set or clear signal handlers.
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
use signal_hook::{consts::*, iterator::Signals};
use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;
use nxsh_core::{context::ShellContext, executor::Executor};

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
        let mut signals = Signals::new(&[sig])?;
        let mut cmd_clone = cmd.clone();
        thread::spawn(move || {
            for _ in signals.forever() {
                let mut ctx = ShellContext::new();
                let mut exec = Executor::new(&mut ctx);
                let _ = exec.run(&cmd_clone);
            }
        });
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