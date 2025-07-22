//! `sleep` builtin – pause execution for specified duration.
//!
//! Usage examples:
//!     sleep 5          # 5 seconds
//!     sleep 1.5        # 1.5 seconds (fractional)
//!     sleep 2m         # 2 minutes (supports s, m, h suffix)
//!     sleep 500ms      # milliseconds suffix
//!
//! Parse rules:
//! * Number with optional fractional part and optional unit suffix.
//! * Supported units: s (seconds, default), ms, m (minutes), h (hours).
//!
//! The command returns successfully after the elapsed time.

use anyhow::{anyhow, Result};
use tokio::time::{sleep as async_sleep, Duration};

pub async fn sleep_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("sleep: missing operand"));
    }

    let duration = parse_duration(&args[0])?;
    async_sleep(duration).await;
    Ok(())
}

fn parse_duration(s: &str) -> Result<Duration> {
    let (num_str, unit) = if let Some(idx) = s.find(|c: char| c.is_alphabetic()) {
        (&s[..idx], &s[idx..])
    } else {
        (s, "s") // default seconds
    };

    let value: f64 = num_str.parse()?;
    if value < 0.0 {
        return Err(anyhow!("sleep: negative time not allowed"));
    }

    let secs = match unit {
        "s" => value,
        "ms" => value / 1000.0,
        "m" => value * 60.0,
        "h" => value * 3600.0,
        _ => return Err(anyhow!("sleep: invalid time suffix")),
    };

    Ok(Duration::from_secs_f64(secs))
} 