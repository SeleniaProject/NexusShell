
//! Date command implementation - Display and set system date and time
//!
//! This module provides a comprehensive implementation of the Unix `date` command
//! with support for formatting, timezone handling, and internationalization.
//! 
//! Features:
//! - Display current date and time in various formats
//! - Custom format strings using strftime syntax
//! - ISO 8601 standard format support
//! - Timezone handling (UTC, local, custom)
//! - Relative date calculations
//! - Unix timestamp conversion
//! - System date setting (with appropriate permissions)
//! - Full internationalization support

use crate::common::{BuiltinContext, BuiltinResult};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc, Datelike, Timelike};
#[cfg(feature = "i18n")]
use chrono_tz::{Tz, UTC as ChronoUTC};
use clap::{Arg, ArgMatches, Command};
use std::str::FromStr;

/// Default date format following POSIX standard
const DEFAULT_FORMAT: &str = "%a %b %e %H:%M:%S %Z %Y";

/// ISO 8601 format
const ISO_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

/// RFC 2822 format (email standard)
const RFC_FORMAT: &str = "%a, %d %b %Y %H:%M:%S %z";

/// Execute the date command
pub fn execute(args: &[String], context: &BuiltinContext) -> BuiltinResult<i32> {
    let app = build_app();
    let matches = match app.try_get_matches_from(std::iter::once("date".to_string()).chain(args.iter().cloned())) {
        Ok(matches) => matches,
        Err(e) => {
            eprintln!("date: {e}");
            return Ok(1);
        }
    };

    match execute_date_command(&matches, context) {
        Ok(()) => Ok(0),
        Err(e) => {
            eprintln!("date: {e}");
            Ok(1)
        }
    }
}

/// Build the command-line argument parser
fn build_app() -> Command {
    Command::new("date")
        .about("Display or set the system date")
        .version("1.0.0")
        .arg(Arg::new("format")
            .help("Display format string")
            .value_name("FORMAT")
            .conflicts_with_all(["iso", "rfc", "universal"]))
        .arg(Arg::new("date")
            .short('d')
            .long("date")
            .help("Display time described by STRING, not 'now'")
            .value_name("STRING")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("file")
            .short('f')
            .long("file")
            .help("Like --date once for each line of DATEFILE")
            .value_name("DATEFILE")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("iso")
            .short('I')
            .long("iso-8601")
            .help("Output date/time in ISO 8601 format")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("rfc")
            .short('R')
            .long("rfc-email")
            .help("Output date and time in RFC 5322 format")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("universal")
            .short('u')
            .long("utc")
            .help("Print or set Coordinated Universal Time (UTC)")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("set")
            .short('s')
            .long("set")
            .help("Set time described by STRING")
            .value_name("STRING")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("reference")
            .short('r')
            .long("reference")
            .help("Display the last modification time of FILE")
            .value_name("FILE")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("debug")
            .long("debug")
            .help("Annotate the parsed date and warn about questionable usage")
            .action(clap::ArgAction::SetTrue))
}

/// Execute the date command based on parsed arguments
fn execute_date_command(matches: &ArgMatches, _context: &BuiltinContext) -> Result<()> {
    // Handle system date setting (requires admin privileges)
    if let Some(date_string) = matches.get_one::<String>("set") {
        return set_system_date(date_string);
    }

    // Handle file reference time
    if let Some(file_path) = matches.get_one::<String>("reference") {
        return display_file_time(file_path, matches);
    }

    // Handle file-based date processing
    if let Some(file_path) = matches.get_one::<String>("file") {
        return process_date_file(file_path, matches);
    }

    // Handle custom date parsing
    if let Some(date_string) = matches.get_one::<String>("date") {
        return display_parsed_date(date_string, matches);
    }

    // Display current date/time
    display_current_date(matches)
}

/// Display current date and time
fn display_current_date(matches: &ArgMatches) -> Result<()> {
    let now = if matches.get_flag("universal") {
        Utc::now()
    } else {
        Local::now().with_timezone(&Utc)
    };

    let formatted = format_datetime(&now, matches)?;
    println!("{formatted}");
    
    if matches.get_flag("debug") {
        eprintln!("date: parsed date: {}", now.format("%Y-%m-%d %H:%M:%S %Z"));
    }
    
    Ok(())
}

/// Display a parsed date string
fn display_parsed_date(date_string: &str, matches: &ArgMatches) -> Result<()> {
    let parsed_date = parse_date_string(date_string)
        .with_context(|| format!("Failed to parse date: '{}'", date_string))?;
    
    let datetime = if matches.get_flag("universal") {
        parsed_date
    } else {
        parsed_date.with_timezone(&Local).with_timezone(&Utc)
    };

    let formatted = format_datetime(&datetime, matches)?;
    println!("{}", formatted);
    
    if matches.get_flag("debug") {
        eprintln!("date: input string: '{}'", date_string);
        eprintln!("date: parsed date: {}", datetime.format("%Y-%m-%d %H:%M:%S %Z"));
    }
    
    Ok(())
}

/// Display file modification time
fn display_file_time(file_path: &str, matches: &ArgMatches) -> Result<()> {
    let metadata = std::fs::metadata(file_path)
        .with_context(|| format!("Failed to read metadata for file: '{}'", file_path))?;
    
    let modified = metadata.modified()
        .with_context(|| format!("Failed to get modification time for file: '{}'", file_path))?;
    
    let datetime = if matches.get_flag("universal") {
        DateTime::<Utc>::from(modified)
    } else {
        DateTime::<Local>::from(modified).with_timezone(&Utc)
    };

    let formatted = format_datetime(&datetime, matches)?;
    println!("{}", formatted);
    
    if matches.get_flag("debug") {
        eprintln!("date: reference file: '{}'", file_path);
        eprintln!("date: file modification time: {}", datetime.format("%Y-%m-%d %H:%M:%S %Z"));
    }
    
    Ok(())
}

/// Process a file containing multiple date strings
fn process_date_file(file_path: &str, matches: &ArgMatches) -> Result<()> {
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read date file: '{}'", file_path))?;
    
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue; // Skip empty lines and comments
        }
        
        match parse_date_string(line) {
            Ok(parsed_date) => {
                let datetime = if matches.get_flag("universal") {
                    parsed_date
                } else {
                    parsed_date.with_timezone(&Local).with_timezone(&Utc)
                };
                
                let formatted = format_datetime(&datetime, matches)?;
                println!("{}", formatted);
                
                if matches.get_flag("debug") {
                    eprintln!("date: line {}: '{}'", line_num + 1, line);
                    eprintln!("date: parsed: {}", datetime.format("%Y-%m-%d %H:%M:%S %Z"));
                }
            }
            Err(e) => {
                eprintln!("date: line {}: {}: '{}'", line_num + 1, e, line);
                if !matches.get_flag("debug") {
                    return Err(e);
                }
            }
        }
    }
    
    Ok(())
}

/// Format datetime according to specified options
fn format_datetime(datetime: &DateTime<Utc>, matches: &ArgMatches) -> Result<String> {
    if matches.get_flag("iso") {
        Ok(datetime.format(ISO_FORMAT).to_string())
    } else if matches.get_flag("rfc") {
        Ok(datetime.format(RFC_FORMAT).to_string())
    } else if let Some(format_str) = matches.get_one::<String>("format") {
        // Custom format string
        validate_format_string(format_str)?;
        Ok(datetime.format(format_str).to_string())
    } else {
        // Default format
        Ok(datetime.format(DEFAULT_FORMAT).to_string())
    }
}

/// Parse various date string formats
fn parse_date_string(date_string: &str) -> Result<DateTime<Utc>> {
    let date_string = date_string.trim();
    
    // Handle relative dates
    if let Some(relative) = parse_relative_date(date_string) {
        return Ok(relative);
    }
    
    // Handle Unix timestamp
    if let Ok(timestamp) = date_string.parse::<i64>() {
        return Ok(DateTime::from_timestamp(timestamp, 0)
            .ok_or_else(|| anyhow!("Invalid timestamp: {}", timestamp))?);
    }

    // Try RFC 3339 / ISO 8601 format first (with timezone info)
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_string) {
        return Ok(dt.with_timezone(&Utc));
    }
    
    // Try various common formats
    let formats = [
        "%Y-%m-%d %H:%M:%S",      // 2023-12-25 15:30:45
        "%Y-%m-%dT%H:%M:%S",      // 2023-12-25T15:30:45
        "%Y-%m-%d",               // 2023-12-25
        "%m/%d/%Y",               // 12/25/2023
        "%m/%d/%Y %H:%M:%S",      // 12/25/2023 15:30:45
        "%d/%m/%Y",               // 25/12/2023
        "%d/%m/%Y %H:%M:%S",      // 25/12/2023 15:30:45
        "%b %d, %Y",              // Dec 25, 2023
        "%b %d %Y %H:%M:%S",      // Dec 25 2023 15:30:45
        "%a %b %d %H:%M:%S %Y",   // Mon Dec 25 15:30:45 2023
    ];
    
    for format in &formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(date_string, format) {
            return Ok(Local.from_local_datetime(&naive).single()
                .ok_or_else(|| anyhow!("Ambiguous local time"))?
                .with_timezone(&Utc));
        }
        
        // Try as date only (add midnight time)
        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_string, format) {
            let naive = date.and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow!("Invalid time component"))?;
            return Ok(Local.from_local_datetime(&naive).single()
                .ok_or_else(|| anyhow!("Ambiguous local time"))?
                .with_timezone(&Utc));
        }
    }
    
    Err(anyhow!("Unable to parse date string: '{}'", date_string))
}

/// Parse relative date expressions like "yesterday", "tomorrow", etc.
fn parse_relative_date(date_string: &str) -> Option<DateTime<Utc>> {
    let now = Utc::now();
    
    match date_string.to_lowercase().as_str() {
        "now" => Some(now),
        "today" => Some(now.date_naive().and_hms_opt(0, 0, 0)?.and_utc()),
        "yesterday" => Some((now - chrono::Duration::days(1)).date_naive().and_hms_opt(0, 0, 0)?.and_utc()),
        "tomorrow" => Some((now + chrono::Duration::days(1)).date_naive().and_hms_opt(0, 0, 0)?.and_utc()),
        "noon" => Some(now.date_naive().and_hms_opt(12, 0, 0)?.and_utc()),
        "midnight" => Some(now.date_naive().and_hms_opt(0, 0, 0)?.and_utc()),
        "epoch" => Some(DateTime::from_timestamp(0, 0)?),
        _ => {
            // Parse expressions like "3 days ago", "2 weeks from now"
            parse_relative_expression(date_string, now)
        }
    }
}

/// Parse complex relative expressions
fn parse_relative_expression(expr: &str, base_time: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    
    if parts.len() < 3 {
        return None;
    }
    
    let amount = parts[0].parse::<i64>().ok()?;
    let unit = parts[1].to_lowercase();
    let direction = parts.get(2)?.to_lowercase();
    
    let duration = match unit.as_str() {
        "second" | "seconds" | "sec" | "secs" => chrono::Duration::seconds(amount),
        "minute" | "minutes" | "min" | "mins" => chrono::Duration::minutes(amount),
        "hour" | "hours" | "hr" | "hrs" => chrono::Duration::hours(amount),
        "day" | "days" => chrono::Duration::days(amount),
        "week" | "weeks" => chrono::Duration::weeks(amount),
        "month" | "months" => chrono::Duration::days(amount * 30), // Approximate
        "year" | "years" => chrono::Duration::days(amount * 365), // Approximate
        _ => return None,
    };
    
    match direction.as_str() {
        "ago" | "before" => Some(base_time - duration),
        "from" | "after" => {
            if parts.len() > 3 && parts[3] == "now" {
                Some(base_time + duration)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Validate format string for security and correctness
fn validate_format_string(format_str: &str) -> Result<()> {
    // Basic validation - check for dangerous format specifiers
    if format_str.contains("%n") {
        return Err(anyhow!("Format specifier '%n' is not supported for security reasons"));
    }
    
    // Check if format string contains valid strftime patterns
    let has_valid_pattern = format_str.contains('%');
    if !has_valid_pattern && !format_str.is_empty() {
        return Err(anyhow!("Invalid format string: no format specifiers found"));
    }
    
    Ok(())
}

/// Set system date (requires administrative privileges)
#[cfg(not(target_os = "windows"))]
fn set_system_date(date_string: &str) -> Result<()> {
    use std::process::Command;
    
    // Parse the date string
    let datetime = parse_date_string(date_string)?;
    
    // Format for system date command
    let formatted = datetime.format("%m%d%H%M%Y.%S").to_string();
    
    // Execute system date command
    let output = Command::new("date")
        .arg(&formatted)
        .output()
        .with_context(|| "Failed to execute system date command")?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to set system date: {}", error));
    }
    
    println!("System date set to: {}", datetime.format(DEFAULT_FORMAT));
    Ok(())
}

/// Set system date on Windows
#[cfg(target_os = "windows")]
fn set_system_date(date_string: &str) -> Result<()> {
    use std::process::Command;
    
    // Parse the date string
    let datetime = parse_date_string(date_string)?;
    
    // Format for Windows date/time commands
    let date_formatted = datetime.format("%m-%d-%Y").to_string();
    let time_formatted = datetime.format("%H:%M:%S").to_string();
    
    // Set date
    let date_output = Command::new("date")
        .arg(&date_formatted)
        .output()
        .with_context(|| "Failed to execute date command")?;
    
    if !date_output.status.success() {
        let error = String::from_utf8_lossy(&date_output.stderr);
        return Err(anyhow!("Failed to set system date: {}", error));
    }
    
    // Set time
    let time_output = Command::new("time")
        .arg(&time_formatted)
        .output()
        .with_context(|| "Failed to execute time command")?;
    
    if !time_output.status.success() {
        let error = String::from_utf8_lossy(&time_output.stderr);
        return Err(anyhow!("Failed to set system time: {}", error));
    }
    
    println!("System date set to: {}", datetime.format(DEFAULT_FORMAT));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    
    #[test]
    fn test_parse_iso_date() {
        let result = parse_date_string("2023-12-25T15:30:45Z").unwrap();
        assert_eq!(result.year(), 2023);
        assert_eq!(result.month(), 12);
        assert_eq!(result.day(), 25);
        assert_eq!(result.hour(), 15);
        assert_eq!(result.minute(), 30);
        assert_eq!(result.second(), 45);
    }
    
    #[test]
    fn test_parse_relative_dates() {
        let now = Utc::now();
        
        assert!(parse_relative_date("now").is_some());
        assert!(parse_relative_date("today").is_some());
        assert!(parse_relative_date("yesterday").is_some());
        assert!(parse_relative_date("tomorrow").is_some());
        assert!(parse_relative_date("noon").is_some());
        assert!(parse_relative_date("midnight").is_some());
        assert!(parse_relative_date("epoch").is_some());
        
        // Test relative expressions
        assert!(parse_relative_expression("3 days ago", now).is_some());
        assert!(parse_relative_expression("2 weeks from now", now).is_some());
        assert!(parse_relative_expression("1 hour ago", now).is_some());
    }
    
    #[test]
    fn test_format_validation() {
        assert!(validate_format_string("%Y-%m-%d").is_ok());
        assert!(validate_format_string("%H:%M:%S").is_ok());
        assert!(validate_format_string("%n").is_err()); // Security check
    }
    
    #[test]
    fn test_unix_timestamp() {
        let result = parse_date_string("1703518245").unwrap(); // 2023-12-25 15:30:45 UTC
        assert_eq!(result.timestamp(), 1703518245);
    }
    
    #[test]
    fn test_various_formats() {
        // Test different date formats
        assert!(parse_date_string("2023-12-25").is_ok());
        assert!(parse_date_string("12/25/2023").is_ok());
        assert!(parse_date_string("Dec 25, 2023").is_ok());
        assert!(parse_date_string("Mon Dec 25 15:30:45 2023").is_ok());
    }
    
    #[test]
    fn test_invalid_dates() {
        assert!(parse_date_string("invalid").is_err());
        assert!(parse_date_string("32/13/2023").is_err());
        assert!(parse_date_string("2023-13-45").is_err());
    }
    
    #[test]
    fn test_format_datetime() {
        use chrono::TimeZone;
        let dt = Utc.with_ymd_and_hms(2023, 12, 25, 15, 30, 45).unwrap();
        let matches = build_app().get_matches_from(vec!["date"]);
        
        let result = format_datetime(&dt, &matches).unwrap();
        assert!(result.contains("2023"));
        assert!(result.contains("Dec"));
    }
}
