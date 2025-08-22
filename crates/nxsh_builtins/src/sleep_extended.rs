use anyhow::{anyhow, Result};
use std::process::Command;
use std::time::{Duration, Instant};
use std::thread;
use which::which;

/// Entry point for the `sleep` extended functionality
pub fn sleep_extended_cli(args: &[String]) -> Result<()> {
    // Try external binary first
    if let Ok(path) = which("sleep") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("sleep: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // Extended internal implementation
    if args.is_empty() {
        eprintln!("sleep: missing operand");
        eprintln!("Usage: sleep DURATION [DURATION]...");
        eprintln!("Examples:");
        eprintln!("  sleep 5         # Sleep 5 seconds");
        eprintln!("  sleep 2.5       # Sleep 2.5 seconds");
        eprintln!("  sleep 1m        # Sleep 1 minute");
        eprintln!("  sleep 1h30m     # Sleep 1.5 hours");
        std::process::exit(1);
    }

    let mut total_duration = Duration::new(0, 0);
    
    // Parse all duration arguments
    for arg in args {
        let duration = parse_sleep_duration(arg)?;
        total_duration += duration;
    }

    if total_duration.as_millis() == 0 {
        return Ok(());
    }

    // Show progress for long sleeps
    if total_duration.as_secs() > 10 {
        println!("Sleeping for {} seconds...", total_duration.as_secs());
        let start_time = Instant::now();
        let mut last_update = Instant::now();
        
        loop {
            let elapsed = start_time.elapsed();
            
            if elapsed >= total_duration {
                break;
            }
            
            // Update progress every 5 seconds for long sleeps
            if last_update.elapsed() >= Duration::from_secs(5) {
                let remaining = total_duration - elapsed;
                let remaining_secs = remaining.as_secs();
                
                if remaining_secs > 60 {
                    let minutes = remaining_secs / 60;
                    let seconds = remaining_secs % 60;
                    println!("Time remaining: {}m {}s", minutes, seconds);
                } else {
                    println!("Time remaining: {}s", remaining_secs);
                }
                
                last_update = Instant::now();
            }
            
            thread::sleep(Duration::from_millis(500));
        }
    } else {
        // Just sleep for short durations
        thread::sleep(total_duration);
    }

    Ok(())
}

/// Parse sleep duration with extended format support
fn parse_sleep_duration(duration_str: &str) -> Result<Duration> {
    let duration_str = duration_str.trim();
    
    if duration_str.is_empty() {
        return Err(anyhow!("sleep: invalid duration: empty string"));
    }

    // Handle pure numbers (seconds)
    if let Ok(seconds) = duration_str.parse::<f64>() {
        if seconds < 0.0 {
            return Err(anyhow!("sleep: invalid duration: negative time"));
        }
        return Ok(Duration::from_secs_f64(seconds));
    }

    // Handle compound duration format (e.g., "1h30m5s")
    let mut total_seconds = 0.0;
    let mut current_number = String::new();
    
    for ch in duration_str.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            current_number.push(ch);
        } else if ch.is_alphabetic() {
            if current_number.is_empty() {
                return Err(anyhow!("sleep: invalid duration format"));
            }
            
            let number: f64 = current_number.parse()
                .map_err(|_| anyhow!("sleep: invalid number: {}", current_number))?;
            
            if number < 0.0 {
                return Err(anyhow!("sleep: invalid duration: negative time"));
            }
            
            let unit = ch.to_lowercase().to_string();
            let seconds = match unit.as_str() {
                "s" => number,
                "m" => number * 60.0,
                "h" => number * 3600.0,
                "d" => number * 86400.0,
                _ => {
                    return Err(anyhow!("sleep: invalid duration unit: {}", unit));
                }
            };
            
            total_seconds += seconds;
            current_number.clear();
        } else {
            return Err(anyhow!("sleep: invalid character in duration: {}", ch));
        }
    }
    
    if total_seconds <= 0.0 {
        return Err(anyhow!("sleep: duration must be positive"));
    }

    Ok(Duration::from_secs_f64(total_seconds))
}

