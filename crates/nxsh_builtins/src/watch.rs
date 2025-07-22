//! `watch` builtin – periodically execute a command and display the output.
//!
//! Basic syntax (subset of GNU watch):
//!     watch [-n SEC] COMMAND [ARGS...]
//!
//! Options:
//!   -n SEC   Interval in seconds (default 2)
//!   -t       Disable header showing interval/time
//!
//! Implementation details:
//! • Clears the terminal between iterations using `crossterm` to avoid flicker.
//! • Runs the command via system shell for convenience.
//! • Stops on Ctrl-C (propagated by parent shell runtime).
//!
//! Limitations:
//! – Does not support highlighting differences or precise alignment with wall-clock second boundaries.

use anyhow::{anyhow, Result};
use crossterm::{execute, terminal::{Clear, ClearType}};
use std::io::{stdout, Write};
use std::process::Command;
use tokio::time::{sleep, Duration};
use chrono::Local;

pub async fn watch_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("watch: usage: watch [-n SEC] command ..."));
    }

    let mut idx = 0;
    let mut interval = 2.0f64;
    let mut header = true;

    while idx < args.len() {
        match args[idx].as_str() {
            "-n" => {
                idx += 1;
                if idx >= args.len() { return Err(anyhow!("watch: -n requires argument")); }
                interval = args[idx].parse()?;
            }
            "-t" => header = false,
            _ => break,
        }
        idx += 1;
    }

    if idx >= args.len() {
        return Err(anyhow!("watch: missing command"));
    }

    let cmd_string = args[idx..].join(" ");

    loop {
        // Clear screen
        execute!(stdout(), Clear(ClearType::All))?;
        if header {
            println!("Every {:.1}s  {}\n", interval, Local::now().format("%Y-%m-%d %H:%M:%S"));
        }
        #[cfg(unix)]
        let status = Command::new("sh").arg("-c").arg(&cmd_string).status();
        #[cfg(windows)]
        let status = Command::new("cmd").arg("/C").arg(&cmd_string).status();
        match status {
            Ok(_) => {},
            Err(e) => eprintln!("watch: command error: {}", e),
        }
        sleep(Duration::from_secs_f64(interval)).await;
    }
} 