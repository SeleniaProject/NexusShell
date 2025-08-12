use anyhow::{anyhow, Result};
use std::process::Command;
use std::time::{Duration, Instant};
use std::thread;
use which::which;

/// Entry point for the `timer` builtin
pub fn timer_cli(args: &[String]) -> Result<()> {
    // Try external binary first
    if let Ok(path) = which("timer") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("timer: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // Basic internal implementation
    if args.is_empty() {
        eprintln!("timer: missing duration");
        eprintln!("Usage: timer DURATION [MESSAGE]");
        eprintln!("Examples:");
        eprintln!("  timer 5s");
        eprintln!("  timer 2m 'Break time!'");
        eprintln!("  timer 1h30m 'Meeting time'");
        std::process::exit(1);
    }

    let duration_str = &args[0];
    let message = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "Timer finished!".to_string()
    };

    let timer_duration = parse_duration(duration_str)?;
    
    println!("Timer started for {} seconds", timer_duration.as_secs());
    println!("Message: {}", message);
    
    let start_time = Instant::now();
    let mut last_update = Instant::now();
    
    loop {
        let elapsed = start_time.elapsed();
        
        if elapsed >= timer_duration {
            // Timer finished
            println!("\n🔔 Timer finished! 🔔");
            println!("Message: {}", message);
            
            // Try to make a beep sound (system dependent)
            #[cfg(windows)]
            {
                let _ = Command::new("cmd")
                    .args(&["/c", "echo", "\x07"])
                    .output();
            }
            #[cfg(unix)]
            {
                let _ = Command::new("tput")
                    .arg("bel")
                    .output();
            }
            
            break;
        }
        
        // Update display every second
        if last_update.elapsed() >= Duration::from_secs(1) {
            let remaining = timer_duration - elapsed;
            let remaining_secs = remaining.as_secs();
            let hours = remaining_secs / 3600;
            let minutes = (remaining_secs % 3600) / 60;
            let seconds = remaining_secs % 60;
            
            if hours > 0 {
                print!("\rTime remaining: {}h {}m {}s   ", hours, minutes, seconds);
            } else if minutes > 0 {
                print!("\rTime remaining: {}m {}s   ", minutes, seconds);
            } else {
                print!("\rTime remaining: {}s   ", seconds);
            }
            
            use std::io::{self, Write};
            if let Err(_) = io::stdout().flush() {
                // Ignore flush errors - they're not critical for timer functionality
            }
            last_update = Instant::now();
        }
        
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}

/// Parse duration string with support for compound durations (e.g., "1h30m5s")
fn parse_duration(duration_str: &str) -> Result<Duration> {
    let duration_str = duration_str.trim();
    
    if duration_str.is_empty() {
        return Err(anyhow!("timer: invalid duration: empty string"));
    }

    let mut total_seconds = 0.0;
    let mut current_number = String::new();
    
    for ch in duration_str.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            current_number.push(ch);
        } else if ch.is_alphabetic() {
            if current_number.is_empty() {
                return Err(anyhow!("timer: invalid duration format"));
            }
            
            let number: f64 = current_number.parse()
                .map_err(|_| anyhow!("timer: invalid number: {}", current_number))?;
            
            if number < 0.0 {
                return Err(anyhow!("timer: invalid duration: negative time"));
            }
            
            let unit = ch.to_lowercase().to_string();
            let seconds = match unit.as_str() {
                "s" => number,
                "m" => number * 60.0,
                "h" => number * 3600.0,
                "d" => number * 86400.0,
                _ => {
                    return Err(anyhow!("timer: invalid duration unit: {}", unit));
                }
            };
            
            total_seconds += seconds;
            current_number.clear();
        } else {
            return Err(anyhow!("timer: invalid character in duration: {}", ch));
        }
    }
    
    // Handle case where no unit is specified (assume seconds)
    if !current_number.is_empty() {
        let number: f64 = current_number.parse()
            .map_err(|_| anyhow!("timer: invalid number: {}", current_number))?;
        total_seconds += number;
    }
    
    if total_seconds <= 0.0 {
        return Err(anyhow!("timer: duration must be positive"));
    }

    Ok(Duration::from_secs_f64(total_seconds))
}
