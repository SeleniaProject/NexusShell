use anyhow::{anyhow, Result};
use std::process::Command;
use std::time::{Duration, Instant};
use std::thread;
use which::which;

/// Entry point for the `timeout` builtin
pub fn timeout_cli(args: &[String]) -> Result<()> {
    // Try external binary first
    if let Ok(path) = which("timeout") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("timeout: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // Basic internal fallback
    if args.len() < 2 {
        eprintln!("timeout: missing operand");
        eprintln!("Usage: timeout DURATION COMMAND [ARG]...");
        std::process::exit(1);
    }

    let duration_str = &args[0];
    let timeout_duration = parse_duration(duration_str)?;
    let command = &args[1];
    let command_args = if args.len() > 2 { &args[2..] } else { &[] };

    // Start the command
    let mut child = Command::new(command)
        .args(command_args)
        .spawn()
        .map_err(|e| anyhow!("timeout: failed to start '{}': {e}", command))?;

    let start_time = Instant::now();
    
    // Wait for either completion or timeout
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Command completed
                std::process::exit(status.code().unwrap_or(0));
            }
            Ok(None) => {
                // Still running, check if timeout elapsed
                if start_time.elapsed() >= timeout_duration {
                    // Timeout - kill the process
                    let _ = child.kill();
                    let _ = child.wait();
                    eprintln!("timeout: sending signal TERM to command '{command}'");
                    std::process::exit(124); // Standard timeout exit code
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(anyhow!("timeout: error waiting for command: {e}"));
            }
        }
    }
}

/// Parse duration string (e.g., "5s", "2m", "1h")
fn parse_duration(duration_str: &str) -> Result<Duration> {
    let duration_str = duration_str.trim();
    
    if duration_str.is_empty() {
        return Err(anyhow!("timeout: invalid duration: empty string"));
    }

    // Extract number and unit
    let (number_part, unit_part) = if let Some(last_char) = duration_str.chars().last() {
        if last_char.is_alphabetic() {
            let split_pos = duration_str.len() - 1;
            duration_str.split_at(split_pos)
        } else {
            // No unit specified, assume seconds
            (duration_str, "s")
        }
    } else {
        return Err(anyhow!("timeout: invalid duration: empty string"));
    };

    let number: f64 = number_part.parse()
        .map_err(|_| anyhow!("timeout: invalid duration number: {}", number_part))?;

    if number < 0.0 {
        return Err(anyhow!("timeout: invalid duration: negative time"));
    }

    let seconds = match unit_part.to_lowercase().as_str() {
        "s" | "sec" | "secs" | "second" | "seconds" => number,
        "m" | "min" | "mins" | "minute" | "minutes" => number * 60.0,
        "h" | "hr" | "hrs" | "hour" | "hours" => number * 3600.0,
        "d" | "day" | "days" => number * 86400.0,
        _ => {
            return Err(anyhow!("timeout: invalid duration unit: {}", unit_part));
        }
    };

    Ok(Duration::from_secs_f64(seconds))
}


/// Execute function stub
pub fn execute(_args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}
