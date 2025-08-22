use std::thread;
use std::time::Duration;
use crate::common::{BuiltinResult, BuiltinContext};

/// Delay for a specified amount of time
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    if args.is_empty() {
        eprintln!("sleep: missing operand");
        eprintln!("Try 'sleep --help' for more information.");
        return Ok(1);
    }

    let mut first_non_option_index = None;
    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(0);
            }
            "--version" => {
                println!("sleep (NexusShell builtins) 1.0.0");
                return Ok(0);
            }
            arg_str if arg_str.starts_with('-') => {
                eprintln!("sleep: invalid option '{arg_str}'");
                return Ok(1);
            }
            _ => {
                first_non_option_index = Some(i);
                break;
            }
        }
    }

    let start_index = match first_non_option_index {
        Some(idx) => idx,
        None => {
            eprintln!("sleep: missing operand");
            return Ok(1);
        }
    };

    let duration_str = &args[start_index];
    let duration = match parse_duration(duration_str) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("sleep: {e}");
            return Ok(1);
        }
    };

    // Check for additional arguments
    if start_index + 1 < args.len() {
        eprintln!("sleep: extra operand '{}'", args[start_index + 1]);
        return Ok(1);
    }

    // Perform the sleep
    thread::sleep(duration);
    Ok(0)
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    if s.is_empty() {
        return Err("invalid time interval".to_string());
    }

    // Handle suffixes
    let (number_str, suffix) = if let Some(stripped) = s.strip_suffix('s') {
        (stripped, "s")
    } else if let Some(stripped) = s.strip_suffix('m') {
        (stripped, "m")
    } else if let Some(stripped) = s.strip_suffix('h') {
        (stripped, "h")
    } else if let Some(stripped) = s.strip_suffix('d') {
        (stripped, "d")
    } else {
        (s, "s") // Default to seconds
    };

    // Parse the number part
    let number: f64 = number_str.parse()
        .map_err(|_| format!("invalid time interval '{s}'"))?;

    if number < 0.0 {
        return Err("invalid time interval".to_string());
    }

    // Convert to seconds based on suffix
    let seconds = match suffix {
        "s" => number,
        "m" => number * 60.0,
        "h" => number * 3600.0,
        "d" => number * 86400.0,
        _ => return Err(format!("invalid time interval '{s}'")),
    };

    // Convert to Duration
    let duration = Duration::from_secs_f64(seconds);
    
    // Check for reasonable limits (avoid overflow)
    if seconds > u64::MAX as f64 {
        return Err("time interval too large".to_string());
    }

    Ok(duration)
}

fn print_help() {
    println!("Usage: sleep NUMBER[SUFFIX]...");
    println!("Pause for NUMBER seconds. SUFFIX may be 's' for seconds (the default),");
    println!("'m' for minutes, 'h' for hours or 'd' for days.");
    println!();
    println!("NUMBER need not be an integer. Given two or more arguments, pause for");
    println!("the amount of time specified by the sum of their values.");
    println!();
    println!("Options:");
    println!("  -h, --help     display this help and exit");
    println!("      --version  output version information and exit");
    println!();
    println!("Examples:");
    println!("  sleep 0.5      Pause for half a second");
    println!("  sleep 2        Pause for 2 seconds");
    println!("  sleep 1m       Pause for 1 minute");
    println!("  sleep 2h       Pause for 2 hours");
    println!("  sleep 1d       Pause for 1 day");
    println!("  sleep 1.5m     Pause for 1.5 minutes (90 seconds)");
}
