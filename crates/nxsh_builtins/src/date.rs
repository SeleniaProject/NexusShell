//! `date` builtin – display or set system date/time.
//!
//! Supported (read-only) usage:
//!     date                # local time, default format
//!     date -u             # UTC
//!     date +"%Y-%m-%d"    # custom strftime format
//!
//! Setting system clock is **not** implemented for safety; attempting to pass a
//! date string will produce an error.

use anyhow::{anyhow, Result};
use chrono::{Local, Utc, DateTime};

pub async fn date_cli(args: &[String]) -> Result<()> {
    let mut use_utc = false;
    let mut format: Option<String> = None;

    for arg in args {
        if arg == "-u" || arg == "--utc" || arg == "--universal" {
            use_utc = true;
        } else if arg.starts_with('+') {
            format = Some(arg[1..].to_string());
        } else {
            return Err(anyhow!("date: setting system time is not supported"));
        }
    }

    let now: DateTime<Utc> = Utc::now();
    let output = if use_utc {
        if let Some(fmt) = format {
            now.format(&fmt).to_string()
        } else {
            now.to_rfc2822()
        }
    } else {
        let local: DateTime<Local> = DateTime::from(now);
        if let Some(fmt) = format {
            local.format(&fmt).to_string()
        } else {
            local.format("%a %b %e %T %Z %Y").to_string() // same as GNU date default
        }
    };

    println!("{}", output);
    Ok(())
} 