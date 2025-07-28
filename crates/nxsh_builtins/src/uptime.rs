//! `uptime` command - show how long the system has been running
//!
//! Full uptime implementation with load averages and user count

use crate::common::{i18n::*, logging::*};
use std::io::Write;
use std::collections::HashMap;
use nxsh_core::{Builtin, Context, ExecutionResult, ShellResult};
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct UptimeBuiltin;

#[derive(Debug, Clone)]
pub struct UptimeOptions {
    pub pretty: bool,
    pub since: bool,
    pub help: bool,
}

#[derive(Debug, Clone)]
pub struct UptimeInfo {
    pub uptime: Duration,
    pub boot_time: SystemTime,
    pub load_avg: (f64, f64, f64),
    pub users: u32,
}

impl Builtin for UptimeBuiltin {
    fn name(&self) -> &str {
        "uptime"
    }

    fn execute(&self, context: &mut Context, args: Vec<String>) -> ShellResult<i32> {
        let options = parse_uptime_args(&args)?;
        
        let uptime_info = collect_uptime_info()?;
        
        if options.since {
            display_since(&uptime_info);
        } else if options.pretty {
            display_pretty(&uptime_info);
        } else {
            display_standard(&uptime_info);
        }
        
        Ok(0)
    }

    fn help(&self) -> &str {
        "uptime - show how long the system has been running

USAGE:
    uptime [OPTIONS]

OPTIONS:
    -p, --pretty    Show uptime in pretty format
    -s, --since     Show when the system was booted
    --help          Display this help and exit

OUTPUT:
    The default output shows:
    - Current time
    - How long the system has been running
    - Number of users currently logged on
    - System load averages for 1, 5, and 15 minutes

EXAMPLES:
    uptime          Show standard uptime information
    uptime -p       Show uptime in human-readable format
    uptime -s       Show boot time"
    }
}

fn parse_uptime_args(args: &[String]) -> ShellResult<UptimeOptions> {
    let mut options = UptimeOptions {
        pretty: false,
        since: false,
        help: false,
    };

    for arg in args {
        match arg.as_str() {
            "-p" | "--pretty" => options.pretty = true,
            "-s" | "--since" => options.since = true,
            "--help" => return Err(ShellError::runtime("Help requested")),
            _ if arg.starts_with("-") => {
                return Err(ShellError::runtime(format!("Unknown option: {}", arg)));
            }
            _ => return Err(ShellError::runtime(format!("Unknown argument: {}", arg))),
        }
    }

    Ok(options)
}

fn collect_uptime_info() -> ShellResult<UptimeInfo> {
    #[cfg(target_os = "linux")]
    {
        collect_linux_uptime_info()
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        // Simplified uptime info for other platforms
        Ok(UptimeInfo {
            uptime: Duration::from_secs(0),
            boot_time: UNIX_EPOCH,
            load_avg: (0.0, 0.0, 0.0),
            users: 0,
        })
    }
}

#[cfg(target_os = "linux")]
fn collect_linux_uptime_info() -> ShellResult<UptimeInfo> {
    // Read uptime from /proc/uptime
    let uptime = read_proc_uptime()?;
    
    // Read load averages from /proc/loadavg
    let load_avg = read_proc_loadavg()?;
    
    // Calculate boot time
    let boot_time = SystemTime::now() - uptime;
    
    // Count logged in users from /var/run/utmp or /proc
    let users = count_logged_in_users();
    
    Ok(UptimeInfo {
        uptime,
        boot_time,
        load_avg,
        users,
    })
}

#[cfg(target_os = "linux")]
fn read_proc_uptime() -> ShellResult<Duration> {
    let content = fs::read_to_string("/proc/uptime")
        .map_err(|e| ShellError::io(format!("Cannot read /proc/uptime: {}", e)))?;
    
    let parts: Vec<&str> = content.split_whitespace().collect();
    if parts.is_empty() {
        return Err(ShellError::runtime("Invalid /proc/uptime format"));
    }
    
    let uptime_secs = parts[0].parse::<f64>()
        .map_err(|_| ShellError::runtime("Invalid uptime value"))?;
    
    Ok(Duration::from_secs_f64(uptime_secs))
}

#[cfg(target_os = "linux")]
fn read_proc_loadavg() -> ShellResult<(f64, f64, f64)> {
    let content = fs::read_to_string("/proc/loadavg")
        .map_err(|e| ShellError::io(format!("Cannot read /proc/loadavg: {}", e)))?;
    
    let parts: Vec<&str> = content.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(ShellError::runtime("Invalid /proc/loadavg format"));
    }
    
    let load1 = parts[0].parse::<f64>()
        .map_err(|_| ShellError::runtime("Invalid load average"))?;
    let load5 = parts[1].parse::<f64>()
        .map_err(|_| ShellError::runtime("Invalid load average"))?;
    let load15 = parts[2].parse::<f64>()
        .map_err(|_| ShellError::runtime("Invalid load average"))?;
    
    Ok((load1, load5, load15))
}

#[cfg(target_os = "linux")]
fn count_logged_in_users() -> u32 {
    // Try to read from /var/run/utmp first, then fall back to /proc
    if let Ok(count) = count_users_from_utmp() {
        return count;
    }
    
    // Fallback: count unique users from /proc/*/stat
    count_users_from_proc().unwrap_or(0)
}

#[cfg(target_os = "linux")]
fn count_users_from_utmp() -> Result<u32, Box<dyn std::error::Error>> {
    // This is a simplified implementation
    // In a real implementation, we would parse the utmp binary format
    // For now, we'll try to count login sessions from who command output
    use std::process::Command;
    
    let output = Command::new("who")
        .output()?;
    
    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let user_count = output_str.lines().count() as u32;
        Ok(user_count)
    } else {
        Err("Failed to run who command".into())
    }
}

#[cfg(target_os = "linux")]
fn count_users_from_proc() -> Result<u32, Box<dyn std::error::Error>> {
    use std::collections::HashSet;
    
    let mut users = HashSet::new();
    let proc_dir = fs::read_dir("/proc")?;
    
    for entry in proc_dir {
        let entry = entry?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        
        if let Ok(_pid) = name_str.parse::<u32>() {
            let status_path = format!("/proc/{}/status", name_str);
            if let Ok(content) = fs::read_to_string(&status_path) {
                for line in content.lines() {
                    if line.starts_with("Uid:") {
                        if let Some(uid_str) = line.split_whitespace().nth(1) {
                            if let Ok(uid) = uid_str.parse::<u32>() {
                                // Only count regular users (UID >= 1000)
                                if uid >= 1000 {
                                    users.insert(uid);
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
    
    Ok(users.len() as u32)
}

fn display_standard(uptime_info: &UptimeInfo) {
    let current_time = format_current_time();
    let uptime_str = format_uptime_duration(uptime_info.uptime);
    let users_str = if uptime_info.users == 1 {
        "1 user".to_string()
    } else {
        format!("{} users", uptime_info.users)
    };
    
    println!(" {} up {}, {}, load average: {:.2}, {:.2}, {:.2}",
        current_time,
        uptime_str,
        users_str,
        uptime_info.load_avg.0,
        uptime_info.load_avg.1,
        uptime_info.load_avg.2
    );
}

fn display_pretty(uptime_info: &UptimeInfo) {
    let uptime_str = format_uptime_pretty(uptime_info.uptime);
    println!("up {}", uptime_str);
}

fn display_since(uptime_info: &UptimeInfo) {
    let boot_time_str = format_boot_time(uptime_info.boot_time);
    println!("{}", boot_time_str);
}

fn format_current_time() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let timestamp = duration.as_secs();
            let hours = (timestamp % 86400) / 3600;
            let minutes = (timestamp % 3600) / 60;
            format!("{:02}:{:02}", hours, minutes)
        }
        Err(_) => "??:??".to_string(),
    }
}

fn format_uptime_duration(uptime: Duration) -> String {
    let total_seconds = uptime.as_secs();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    
    if days > 0 {
        if days == 1 {
            if hours > 0 {
                format!("{} day, {}:{:02}", days, hours, minutes)
            } else {
                format!("{} day, {} min", days, minutes)
            }
        } else {
            if hours > 0 {
                format!("{} days, {}:{:02}", days, hours, minutes)
            } else {
                format!("{} days, {} min", days, minutes)
            }
        }
    } else if hours > 0 {
        format!("{}:{:02}", hours, minutes)
    } else {
        format!("{} min", minutes)
    }
}

fn format_uptime_pretty(uptime: Duration) -> String {
    let total_seconds = uptime.as_secs();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    
    let mut parts = Vec::new();
    
    if days > 0 {
        if days == 1 {
            parts.push("1 day".to_string());
        } else {
            parts.push(format!("{} days", days));
        }
    }
    
    if hours > 0 {
        if hours == 1 {
            parts.push("1 hour".to_string());
        } else {
            parts.push(format!("{} hours", hours));
        }
    }
    
    if minutes > 0 {
        if minutes == 1 {
            parts.push("1 minute".to_string());
        } else {
            parts.push(format!("{} minutes", minutes));
        }
    }
    
    if parts.is_empty() {
        "less than a minute".to_string()
    } else if parts.len() == 1 {
        parts[0].clone()
    } else if parts.len() == 2 {
        format!("{} and {}", parts[0], parts[1])
    } else {
        let last = parts.pop().unwrap();
        format!("{}, and {}", parts.join(", "), last)
    }
}

fn format_boot_time(boot_time: SystemTime) -> String {
    match boot_time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let timestamp = duration.as_secs();
            
            // Simple date formatting (in a real implementation, we'd use a proper date library)
            let days_since_epoch = timestamp / 86400;
            let year = 1970 + days_since_epoch / 365; // Approximation
            let day_of_year = days_since_epoch % 365;
            let month = (day_of_year / 30) + 1; // Approximation
            let day = (day_of_year % 30) + 1;
            
            let hours = (timestamp % 86400) / 3600;
            let minutes = (timestamp % 3600) / 60;
            let seconds = timestamp % 60;
            
            format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, hours, minutes, seconds)
        }
        Err(_) => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime_duration() {
        assert_eq!(format_uptime_duration(Duration::from_secs(30)), "0 min");
        assert_eq!(format_uptime_duration(Duration::from_secs(60)), "1 min");
        assert_eq!(format_uptime_duration(Duration::from_secs(3600)), "1:00");
        assert_eq!(format_uptime_duration(Duration::from_secs(3660)), "1:01");
        assert_eq!(format_uptime_duration(Duration::from_secs(86400)), "1 day, 0 min");
        assert_eq!(format_uptime_duration(Duration::from_secs(90000)), "1 day, 1:00");
    }

    #[test]
    fn test_format_uptime_pretty() {
        assert_eq!(format_uptime_pretty(Duration::from_secs(30)), "less than a minute");
        assert_eq!(format_uptime_pretty(Duration::from_secs(60)), "1 minute");
        assert_eq!(format_uptime_pretty(Duration::from_secs(120)), "2 minutes");
        assert_eq!(format_uptime_pretty(Duration::from_secs(3600)), "1 hour");
        assert_eq!(format_uptime_pretty(Duration::from_secs(3660)), "1 hour and 1 minute");
        assert_eq!(format_uptime_pretty(Duration::from_secs(86400)), "1 day");
        assert_eq!(format_uptime_pretty(Duration::from_secs(90060)), "1 day, 1 hour, and 1 minute");
    }
} 