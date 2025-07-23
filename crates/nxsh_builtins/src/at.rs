//! `at` builtin – schedule a one-shot command to run at a specified time.
//!
//! Simplified syntax compatible with common `at`:
//!     at HH:MM COMMAND [ARGS...]
//!     at +MINUTES COMMAND
//!
//! • `HH:MM` interpreted in local time; if the time has already passed today,
//!   schedule for tomorrow.
//! • `+MINUTES` schedules relative delay in minutes.
//! • The task is executed in the background using system shell (`sh -c`) on
//!   Unix, `cmd /C` on Windows.
//! • Multiple `at` tasks can coexist; they are stored in-memory for the shell
//!   lifetime.
//!
//! Limitations:
//! – Persistence across shell restarts is not implemented yet.
//! – Timezone handling relies on system locale.

use anyhow::{anyhow, Result};
use chrono::{Local, NaiveTime, Timelike, Duration as ChronoDuration};
use tokio::time::{sleep_until, Instant, Duration};
use std::process::Command;

pub async fn at_cli(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        return Err(anyhow!("at: usage: at <time>|+<minutes> command [args...]"));
    }
    let spec = &args[0];
    let cmd = args[1..].join(" ");

    let delay = if let Some(minutes) = spec.strip_prefix('+') {
        let mins: u64 = minutes.parse()?;
        Duration::from_secs(mins * 60)
    } else {
        let time = NaiveTime::parse_from_str(spec, "%H:%M")?;
        let now = Local::now();
        let today = now.date_naive().and_time(time);
        let mut target = today;
        if target <= now.naive_local() {
            target = target + ChronoDuration::days(1);
        }
        let dur = (target - now.naive_local()).to_std()?;
        Duration::from_secs(dur.as_secs())
    };

    println!("at: job scheduled in {:.0} seconds", delay.as_secs_f64());
    tokio::spawn(async move {
        sleep_until(Instant::now() + delay).await;
        #[cfg(unix)]
        let status = Command::new("sh").arg("-c").arg(cmd).status();
        #[cfg(windows)]
        let status = Command::new("cmd").arg("/C").arg(cmd).status();
        match status {
            Ok(st) => println!("at: job finished with status {}", st),
            Err(e) => eprintln!("at: failed to run command: {}", e),
        }
    });
    Ok(())
} 