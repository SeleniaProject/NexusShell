//! `nice` builtin  Erun command with modified scheduler priority.
//!
//! Usage: `nice [-n ADJUST] COMMAND [ARGS...]`
//! If `-n` is omitted, default adjustment is `10`. Positive values lower priority.
//!
//! Currently Unix-only implementation; Windows returns an error.

use anyhow::{anyhow, Result};
use std::process::Command;
#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub fn nice_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("nice: missing COMMAND"));
    }

    let (adjust, cmd_index) = if args[0] == "-n" {
        if args.len() < 3 {
            return Err(anyhow!("nice: -n requires ARG and COMMAND"));
        }
        let adj: i32 = args[1]
            .parse()
            .map_err(|_| anyhow!("nice: invalid adjustment value"))?;
        (adj, 2)
    } else {
        (10, 0) // default niceness increment
    };

    if cmd_index >= args.len() {
        return Err(anyhow!("nice: missing COMMAND"));
    }

    let command = &args[cmd_index];
    let cmd_args: Vec<String> = args[cmd_index + 1..].to_vec();

    let mut cmd = Command::new(command);
    cmd.args(&cmd_args);

    #[cfg(unix)]
    {
        // Clone adjust for move into closure
        let niceness = adjust;
        unsafe {
            cmd.pre_exec(move || {
                // Apply niceness to child process before exec
                if libc::setpriority(libc::PRIO_PROCESS, 0, niceness) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    #[cfg(windows)]
    {
        // Windows: approximate niceness via process priority class mapping
        use windows_sys::Win32::System::Threading::{
            ABOVE_NORMAL_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS, IDLE_PRIORITY_CLASS,
            NORMAL_PRIORITY_CLASS, REALTIME_PRIORITY_CLASS,
        };
        let priority_class = if adjust <= -15 {
            REALTIME_PRIORITY_CLASS
        } else if adjust <= -10 {
            HIGH_PRIORITY_CLASS
        } else if adjust <= -5 {
            ABOVE_NORMAL_PRIORITY_CLASS
        } else if adjust >= 15 {
            IDLE_PRIORITY_CLASS
        } else if adjust >= 5 {
            BELOW_NORMAL_PRIORITY_CLASS
        } else {
            NORMAL_PRIORITY_CLASS
        };

        // Spawn using cmd.exe to execute command; priority cannot be set on child easily without WinAPI CreateProcessEx.
        // We document approximation and execute normally; users can combine with `start /HIGH` manually if needed.
        let mut cmd = Command::new(command);
        cmd.args(&cmd_args);
        let status = cmd.status().map_err(|e| anyhow!("nice: failed to execute '{}': {e}", command))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    #[cfg(not(windows))]
    {
        let status = cmd
            .status()
            .map_err(|e| anyhow!("nice: failed to execute '{}': {e}", command))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    #[allow(unreachable_code)]
    Ok(())
} 
