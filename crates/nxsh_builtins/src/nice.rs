//! `nice` builtin — run command with modified scheduler priority.
//!
//! Usage: `nice [-n ADJUST] COMMAND [ARGS...]`
//! If `-n` is omitted, default adjustment is `10`. Positive values lower priority.
//!
//! Currently Unix-only implementation; Windows returns an error.

use anyhow::{anyhow, Result};
use std::process::Command;

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
        return Err(anyhow!("nice: not supported on Windows yet"));
    }

    let status = cmd.status().map_err(|e| anyhow!("nice: failed to execute '{}': {e}", command))?;
    std::process::exit(status.code().unwrap_or(1));
} 