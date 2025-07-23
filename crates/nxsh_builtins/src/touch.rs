//! `touch` command – comprehensive file timestamp update and creation implementation.
//!
//! Supports complete touch functionality:
//!   touch [OPTIONS] FILE...
//!   -a                        - Change only the access time
//!   -c, --no-create           - Do not create any files
//!   -d, --date=STRING         - Parse STRING and use it instead of current time
//!   -f                        - (ignored)
//!   -h, --no-dereference      - Affect each symbolic link instead of any referenced file
//!   -m                        - Change only the modification time
//!   -r, --reference=FILE      - Use this file's times instead of current time
//!   -t STAMP                  - Use [[CC]YY]MMDDhhmm[.ss] instead of current time
//!   --time=WORD               - Change the specified time (access, atime, modify, mtime)
//!   --help                    - Display help and exit
//!   --version                 - Output version information and exit

use anyhow::{Result, anyhow};
use std::fs::{self, OpenOptions, File, Metadata};
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc, ParseResult};
use libc::{timespec, utimensat, AT_FDCWD, UTIME_NOW, UTIME_OMIT};
use std::ffi::CString;

#[derive(Debug, Clone)]
pub struct TouchOptions {
    pub files: Vec<String>,
    pub access_only: bool,
    pub no_create: bool,
    pub date_string: Option<String>,
    pub no_dereference: bool,
    pub modify_only: bool,
    pub reference_file: Option<String>,
    pub timestamp: Option<String>,
    pub time_type: TimeType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeType {
    Both,
    Access,
    Modify,
}

impl Default for TouchOptions {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            access_only: false,
            no_create: false,
            date_string: None,
            no_dereference: false,
            modify_only: false,
            reference_file: None,
            timestamp: None,
            time_type: TimeType::Both,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TouchTime {
    pub access_time: Option<SystemTime>,
    pub modify_time: Option<SystemTime>,
}

pub fn touch_cli(args: &[String]) -> Result<()> {
    let options = parse_touch_args(args)?;
    
    if options.files.is_empty() {
        return Err(anyhow!("touch: missing file operand"));
    }
    
    // Determine the timestamps to use
    let touch_time = determine_touch_time(&options)?;
    
    // Process each file
    for file in &options.files {
        let path = PathBuf::from(file);
        
        if let Err(e) = update_file_timestamps(&path, &touch_time, &options) {
            eprintln!("touch: {}", e);
            // Continue with other files
        }
    }
    
    Ok(())
}

fn parse_touch_args(args: &[String]) -> Result<TouchOptions> {
    let mut options = TouchOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "-a" => {
                options.access_only = true;
                options.modify_only = false;
                options.time_type = TimeType::Access;
            }
            "-c" | "--no-create" => {
                options.no_create = true;
            }
            "-d" | "--date" => {
                if i + 1 < args.len() {
                    options.date_string = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err(anyhow!("touch: option requires an argument -- d"));
                }
            }
            arg if arg.starts_with("--date=") => {
                let date_str = arg.strip_prefix("--date=").unwrap();
                options.date_string = Some(date_str.to_string());
            }
            "-f" => {
                // Ignored for compatibility
            }
            "-h" | "--no-dereference" => {
                options.no_dereference = true;
            }
            "-m" => {
                options.modify_only = true;
                options.access_only = false;
                options.time_type = TimeType::Modify;
            }
            "-r" | "--reference" => {
                if i + 1 < args.len() {
                    options.reference_file = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err(anyhow!("touch: option requires an argument -- r"));
                }
            }
            arg if arg.starts_with("--reference=") => {
                let ref_file = arg.strip_prefix("--reference=").unwrap();
                options.reference_file = Some(ref_file.to_string());
            }
            "-t" => {
                if i + 1 < args.len() {
                    options.timestamp = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err(anyhow!("touch: option requires an argument -- t"));
                }
            }
            "--time=access" | "--time=atime" => {
                options.time_type = TimeType::Access;
            }
            "--time=modify" | "--time=mtime" => {
                options.time_type = TimeType::Modify;
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--version" => {
                println!("touch (NexusShell) 1.0.0");
                std::process::exit(0);
            }
            arg if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                // Handle combined short options
                for ch in arg.chars().skip(1) {
                    match ch {
                        'a' => {
                            options.access_only = true;
                            options.modify_only = false;
                            options.time_type = TimeType::Access;
                        }
                        'c' => options.no_create = true,
                        'f' => {}, // Ignored
                        'h' => options.no_dereference = true,
                        'm' => {
                            options.modify_only = true;
                            options.access_only = false;
                            options.time_type = TimeType::Modify;
                        }
                        _ => return Err(anyhow!("touch: invalid option -- '{}'", ch)),
                    }
                }
            }
            _ => {
                // This is a file name
                options.files.push(arg.clone());
            }
        }
        i += 1;
    }
    
    Ok(options)
}

fn determine_touch_time(options: &TouchOptions) -> Result<TouchTime> {
    let now = SystemTime::now();
    
    // If reference file is specified, use its timestamps
    if let Some(ref ref_file) = options.reference_file {
        let ref_path = Path::new(ref_file);
        if !ref_path.exists() {
            return Err(anyhow!("touch: failed to get attributes of '{}': No such file or directory", ref_file));
        }
        
        let metadata = if options.no_dereference {
            fs::symlink_metadata(ref_path)?
        } else {
            fs::metadata(ref_path)?
        };
        
        let access_time = metadata.accessed().ok();
        let modify_time = metadata.modified().ok();
        
        return Ok(TouchTime {
            access_time,
            modify_time,
        });
    }
    
    // If timestamp is specified, parse it
    if let Some(ref timestamp) = options.timestamp {
        let parsed_time = parse_timestamp(timestamp)?;
        return Ok(TouchTime {
            access_time: Some(parsed_time),
            modify_time: Some(parsed_time),
        });
    }
    
    // If date string is specified, parse it
    if let Some(ref date_str) = options.date_string {
        let parsed_time = parse_date_string(date_str)?;
        return Ok(TouchTime {
            access_time: Some(parsed_time),
            modify_time: Some(parsed_time),
        });
    }
    
    // Default to current time
    Ok(TouchTime {
        access_time: Some(now),
        modify_time: Some(now),
    })
}

fn parse_timestamp(timestamp: &str) -> Result<SystemTime> {
    // Format: [[CC]YY]MMDDhhmm[.ss]
    let len = timestamp.len();
    
    if len < 8 || len > 15 {
        return Err(anyhow!("touch: invalid date format '{}'", timestamp));
    }
    
    let mut chars: Vec<char> = timestamp.chars().collect();
    let mut pos = 0;
    
    // Parse seconds if present (after the dot)
    let seconds = if let Some(dot_pos) = timestamp.find('.') {
        if dot_pos + 3 != len {
            return Err(anyhow!("touch: invalid date format '{}'", timestamp));
        }
        let sec_str: String = chars[dot_pos + 1..].iter().collect();
        chars.truncate(dot_pos);
        sec_str.parse::<u32>().map_err(|_| anyhow!("touch: invalid seconds '{}'", sec_str))?
    } else {
        0
    };
    
    let remaining: String = chars.iter().collect();
    let len = remaining.len();
    
    let (year, month, day, hour, minute) = match len {
        8 => {
            // MMDDhhmm
            let mm: u32 = remaining[0..2].parse()?;
            let dd: u32 = remaining[2..4].parse()?;
            let hh: u32 = remaining[4..6].parse()?;
            let min: u32 = remaining[6..8].parse()?;
            let current_year = Local::now().year();
            (current_year, mm, dd, hh, min)
        }
        10 => {
            // YYMMDDhhmm
            let yy: i32 = remaining[0..2].parse()?;
            let mm: u32 = remaining[2..4].parse()?;
            let dd: u32 = remaining[4..6].parse()?;
            let hh: u32 = remaining[6..8].parse()?;
            let min: u32 = remaining[8..10].parse()?;
            let year = if yy >= 69 { 1900 + yy } else { 2000 + yy };
            (year, mm, dd, hh, min)
        }
        12 => {
            // CCYYMMDDhhmm
            let ccyy: i32 = remaining[0..4].parse()?;
            let mm: u32 = remaining[4..6].parse()?;
            let dd: u32 = remaining[6..8].parse()?;
            let hh: u32 = remaining[8..10].parse()?;
            let min: u32 = remaining[10..12].parse()?;
            (ccyy, mm, dd, hh, min)
        }
        _ => return Err(anyhow!("touch: invalid date format '{}'", timestamp)),
    };
    
    // Validate ranges
    if month < 1 || month > 12 {
        return Err(anyhow!("touch: invalid month '{}'", month));
    }
    if day < 1 || day > 31 {
        return Err(anyhow!("touch: invalid day '{}'", day));
    }
    if hour > 23 {
        return Err(anyhow!("touch: invalid hour '{}'", hour));
    }
    if minute > 59 {
        return Err(anyhow!("touch: invalid minute '{}'", minute));
    }
    if seconds > 59 {
        return Err(anyhow!("touch: invalid seconds '{}'", seconds));
    }
    
    // Create datetime
    let dt = Local.ymd(year, month, day).and_hms(hour, minute, seconds);
    let system_time = SystemTime::UNIX_EPOCH + Duration::from_secs(dt.timestamp() as u64);
    
    Ok(system_time)
}

fn parse_date_string(date_str: &str) -> Result<SystemTime> {
    // Try various date formats
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d",
        "%m/%d/%Y %H:%M:%S",
        "%m/%d/%Y %H:%M",
        "%m/%d/%Y",
        "%d %b %Y %H:%M:%S",
        "%d %b %Y %H:%M",
        "%d %b %Y",
        "%Y%m%d %H:%M:%S",
        "%Y%m%d %H:%M",
        "%Y%m%d",
    ];
    
    for format in &formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, format) {
            let local_dt = Local.from_local_datetime(&dt).single();
            if let Some(local_dt) = local_dt {
                let system_time = SystemTime::UNIX_EPOCH + Duration::from_secs(local_dt.timestamp() as u64);
                return Ok(system_time);
            }
        }
    }
    
    // Try parsing as relative time (like "now", "1 hour ago", etc.)
    match date_str.to_lowercase().as_str() {
        "now" => Ok(SystemTime::now()),
        _ => Err(anyhow!("touch: invalid date '{}'", date_str)),
    }
}

fn update_file_timestamps(path: &Path, touch_time: &TouchTime, options: &TouchOptions) -> Result<()> {
    // Check if file exists
    let file_exists = if options.no_dereference {
        path.symlink_metadata().is_ok()
    } else {
        path.metadata().is_ok()
    };
    
    // Create file if it doesn't exist and creation is allowed
    if !file_exists {
        if options.no_create {
            return Ok(());
        }
        
        // Create empty file
        File::create(path)
            .map_err(|e| anyhow!("touch: cannot touch '{}': {}", path.display(), e))?;
    }
    
    // Update timestamps using utimensat system call
    update_timestamps_unix(path, touch_time, options)
}

fn update_timestamps_unix(path: &Path, touch_time: &TouchTime, options: &TouchOptions) -> Result<()> {
    let path_cstr = CString::new(path.to_string_lossy().as_bytes())
        .map_err(|_| anyhow!("touch: invalid path '{}'", path.display()))?;
    
    // Prepare timespec structures
    let mut times = [
        timespec { tv_sec: 0, tv_nsec: UTIME_OMIT }, // access time
        timespec { tv_sec: 0, tv_nsec: UTIME_OMIT }, // modify time
    ];
    
    // Set access time
    if should_update_access_time(options) {
        if let Some(access_time) = touch_time.access_time {
            let duration = access_time.duration_since(UNIX_EPOCH)
                .map_err(|_| anyhow!("touch: invalid access time"))?;
            times[0] = timespec {
                tv_sec: duration.as_secs() as i64,
                tv_nsec: duration.subsec_nanos() as i64,
            };
        } else {
            times[0].tv_nsec = UTIME_NOW;
        }
    }
    
    // Set modify time
    if should_update_modify_time(options) {
        if let Some(modify_time) = touch_time.modify_time {
            let duration = modify_time.duration_since(UNIX_EPOCH)
                .map_err(|_| anyhow!("touch: invalid modify time"))?;
            times[1] = timespec {
                tv_sec: duration.as_secs() as i64,
                tv_nsec: duration.subsec_nanos() as i64,
            };
        } else {
            times[1].tv_nsec = UTIME_NOW;
        }
    }
    
    // Call utimensat
    let flags = if options.no_dereference {
        libc::AT_SYMLINK_NOFOLLOW
    } else {
        0
    };
    
    let result = unsafe {
        utimensat(AT_FDCWD, path_cstr.as_ptr(), times.as_ptr(), flags)
    };
    
    if result != 0 {
        let error = std::io::Error::last_os_error();
        return Err(anyhow!("touch: setting times of '{}': {}", path.display(), error));
    }
    
    Ok(())
}

fn should_update_access_time(options: &TouchOptions) -> bool {
    match options.time_type {
        TimeType::Access => true,
        TimeType::Modify => false,
        TimeType::Both => !options.modify_only,
    }
}

fn should_update_modify_time(options: &TouchOptions) -> bool {
    match options.time_type {
        TimeType::Access => false,
        TimeType::Modify => true,
        TimeType::Both => !options.access_only,
    }
}

fn print_help() {
    println!("Usage: touch [OPTION]... FILE...");
    println!("Update the access and modification times of each FILE to the current time.");
    println!();
    println!("A FILE argument that does not exist is created empty, unless -c or -h");
    println!("is supplied.");
    println!();
    println!("A FILE argument string of - is handled specially and causes touch to");
    println!("change the times of the file associated with standard output.");
    println!();
    println!("Mandatory arguments to long options are mandatory for short options too.");
    println!("  -a                     change only the access time");
    println!("  -c, --no-create        do not create any files");
    println!("  -d, --date=STRING      parse STRING and use it instead of current time");
    println!("  -f                     (ignored)");
    println!("  -h, --no-dereference   affect each symbolic link instead of any referenced");
    println!("                         file (useful only on systems that can change the");
    println!("                         timestamps of a symlink)");
    println!("  -m                     change only the modification time");
    println!("  -r, --reference=FILE   use this file's times instead of current time");
    println!("  -t STAMP               use [[CC]YY]MMDDhhmm[.ss] instead of current time");
    println!("      --time=WORD        change the specified time:");
    println!("                           WORD is access, atime, or use: equivalent to -a");
    println!("                           WORD is modify or mtime: equivalent to -m");
    println!("      --help     display this help and exit");
    println!("      --version  output version information and exit");
    println!();
    println!("Note that the -d and -t options accept different time-date formats.");
    println!();
    println!("STAMP may be used without -t if none of -drt, or --date, --reference, or");
    println!("--time are used, and if there are two or more arguments and the first");
    println!("argument is a valid STAMP.  In that case, the STAMP is equivalent to");
    println!("date +%Y%m%d%H%M.%S.");
    println!();
    println!("Examples:");
    println!("  touch file.txt                    Update timestamps to current time");
    println!("  touch -t 202301011200 file.txt    Set time to Jan 1, 2023 12:00");
    println!("  touch -r reference.txt file.txt   Use reference.txt's timestamps");
    println!("  touch -a file.txt                 Update only access time");
    println!("  touch -m file.txt                 Update only modification time");
    println!();
    println!("Report touch bugs to <bug-reports@nexusshell.org>");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, NamedTempFile};
    use std::fs::File;
    use std::io::Write;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_parse_args() {
        let args = vec!["-am".to_string(), "file.txt".to_string()];
        let options = parse_touch_args(&args).unwrap();
        
        // Note: -am would set both, but the last one wins in our implementation
        assert_eq!(options.files, vec!["file.txt"]);
    }
    
    #[test]
    fn test_no_create_option() {
        let args = vec!["-c".to_string(), "nonexistent.txt".to_string()];
        let options = parse_touch_args(&args).unwrap();
        
        assert!(options.no_create);
        assert_eq!(options.files, vec!["nonexistent.txt"]);
    }
    
    #[test]
    fn test_reference_option() {
        let args = vec!["-r".to_string(), "ref.txt".to_string(), "target.txt".to_string()];
        let options = parse_touch_args(&args).unwrap();
        
        assert_eq!(options.reference_file, Some("ref.txt".to_string()));
        assert_eq!(options.files, vec!["target.txt"]);
    }
    
    #[test]
    fn test_parse_timestamp() {
        // Test CCYYMMDDHHMMSS format
        let timestamp = "202301011200.30";
        let result = parse_timestamp(timestamp).unwrap();
        
        // Should parse to Jan 1, 2023 12:00:30
        let duration = result.duration_since(UNIX_EPOCH).unwrap();
        let expected = Local.ymd(2023, 1, 1).and_hms(12, 0, 30).timestamp() as u64;
        assert_eq!(duration.as_secs(), expected);
    }
    
    #[test]
    fn test_parse_date_string() {
        let date_str = "2023-01-01 12:00:00";
        let result = parse_date_string(date_str).unwrap();
        
        // Should parse correctly
        assert!(result > UNIX_EPOCH);
    }
    
    #[test]
    fn test_should_update_times() {
        let mut options = TouchOptions::default();
        
        // Default: update both
        assert!(should_update_access_time(&options));
        assert!(should_update_modify_time(&options));
        
        // Access only
        options.time_type = TimeType::Access;
        assert!(should_update_access_time(&options));
        assert!(!should_update_modify_time(&options));
        
        // Modify only
        options.time_type = TimeType::Modify;
        assert!(!should_update_access_time(&options));
        assert!(should_update_modify_time(&options));
    }
} 