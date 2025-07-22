//! `time` builtin — measure execution time of a command.
//!
//! Syntax: `time CMD [ARGS...]`
//! Reports real, user, and sys time similar to GNU time (brief mode).
//! Uses `Instant` for wall clock and `getrusage(RUSAGE_CHILDREN)` for CPU usage.

use anyhow::{anyhow, Result};
use std::process::Command;
use std::time::Instant;
#[cfg(unix)]
use libc::{getrusage, rusage, RUSAGE_CHILDREN};

pub fn time_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("time: missing command"));
    }

    #[cfg(unix)]
    unsafe {
        let mut before: rusage = std::mem::zeroed();
        getrusage(RUSAGE_CHILDREN, &mut before);
        let start = Instant::now();

        let status = Command::new(&args[0]).args(&args[1..]).status()?;

        let duration = start.elapsed();
        let mut after: rusage = std::mem::zeroed();
        getrusage(RUSAGE_CHILDREN, &mut after);

        let user_sec = ru_sec(after.ru_utime) - ru_sec(before.ru_utime);
        let sys_sec = ru_sec(after.ru_stime) - ru_sec(before.ru_stime);

        println!("real\t{:.3}s", sec_f64(duration));
        println!("user\t{:.3}s", user_sec);
        println!("sys \t{:.3}s", sys_sec);
        std::process::exit(status.code().unwrap_or(1));
    }

    #[cfg(windows)]
    {
        let start = Instant::now();
        let status = Command::new(&args[0]).args(&args[1..]).status()?;
        let duration = start.elapsed();
        println!("real\t{:.3}s", sec_f64(duration));
        println!("user\tN/A");
        println!("sys \tN/A");
        std::process::exit(status.code().unwrap_or(1));
    }
}

#[cfg(unix)]
fn ru_sec(tv: libc::timeval) -> f64 {
    tv.tv_sec as f64 + (tv.tv_usec as f64 / 1_000_000.0)
}

fn sec_f64(dur: std::time::Duration) -> f64 {
    dur.as_secs() as f64 + dur.subsec_nanos() as f64 / 1_000_000_000.0
} 