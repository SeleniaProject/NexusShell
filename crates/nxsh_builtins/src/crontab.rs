use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;
use anyhow::Result;
use nxsh_core::{ErrorKind, ShellError};
use nxsh_core::error::RuntimeErrorKind;

pub fn crontab_cli(args: Vec<String>) -> Result<()> {
    if args.is_empty() || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let mut list_mode = false;
    let mut edit_mode = false;
    let mut remove_mode = false;
    let mut user = None;
    let mut file_input = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-l" => list_mode = true,
            "-e" => edit_mode = true,
            "-r" => remove_mode = true,
            "-u" => {
                i += 1;
                if i < args.len() {
                    user = Some(args[i].clone());
                }
            }
            arg if !arg.starts_with('-') => {
                file_input = Some(arg.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    if list_mode {
        list_cron_jobs(&user)
    } else if edit_mode {
        edit_cron_jobs(&user)
    } else if remove_mode {
        remove_cron_jobs(&user)
    } else if let Some(file) = file_input {
        install_cron_file(&file, &user)
    } else {
        // Default: read from stdin
        read_from_stdin(&user)
    }
}

fn print_help() {
    println!("Usage: crontab [options] [file]");
    println!();
    println!("Options:");
    println!("  -l              Display the current crontab");
    println!("  -e              Edit the current crontab");
    println!("  -r              Remove the current crontab");
    println!("  -u user         Specify the user whose crontab to manipulate");
    println!("  -h, --help      Show this help message");
    println!();
    println!("Examples:");
    println!("  crontab -l           # List current user's cron jobs");
    println!("  crontab -e           # Edit current user's cron jobs");
    println!("  crontab -r           # Remove all cron jobs");
    println!("  crontab mycron.txt   # Install cron jobs from file");
    println!("  crontab -u john -l   # List john's cron jobs (requires privileges)");
}

fn list_cron_jobs(user: &Option<String>) -> Result<()> {
    let cron_file = get_cron_file_path(user)?;
    
    match std::fs::read_to_string(&cron_file) {
        Ok(contents) => {
            if contents.trim().is_empty() {
                println!("no crontab for {}", user.as_deref().unwrap_or("current user"));
            } else {
                print!("{contents}");
            }
        }
        Err(_) => {
            println!("no crontab for {}", user.as_deref().unwrap_or("current user"));
        }
    }
    
    Ok(())
}

fn edit_cron_jobs(user: &Option<String>) -> Result<()> {
    let cron_file = get_cron_file_path(user)?;
    let editor = env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(windows) {
            "notepad".to_string()
        } else {
            "vi".to_string()
        }
    });

    // Create temp file for editing
    let temp_file = format!("{}.tmp", cron_file.display());
    
    // Copy existing crontab to temp file if it exists
    if let Ok(existing) = std::fs::read_to_string(&cron_file) {
        std::fs::write(&temp_file, existing)?;
    }

    // Launch editor
    let status = Command::new(&editor)
        .arg(&temp_file)
        .status()?;

    if status.success() {
        // Validate and install the edited crontab
        if validate_cron_file(&temp_file)? {
            std::fs::rename(&temp_file, &cron_file)?;
            println!("crontab: installing new crontab");
        } else {
            std::fs::remove_file(&temp_file)?;
            return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid crontab format").into());
        }
    } else {
        std::fs::remove_file(&temp_file)?;
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Editor failed").into());
    }

    Ok(())
}

fn remove_cron_jobs(user: &Option<String>) -> Result<()> {
    let cron_file = get_cron_file_path(user)?;
    
    if cron_file.exists() {
        std::fs::remove_file(&cron_file)?;
        println!("crontab: removing crontab for {}", user.as_deref().unwrap_or("current user"));
    } else {
        println!("no crontab for {}", user.as_deref().unwrap_or("current user"));
    }
    
    Ok(())
}

fn install_cron_file(file_path: &str, user: &Option<String>) -> Result<()> {
    let cron_file = get_cron_file_path(user)?;
    
    if !validate_cron_file(file_path)? {
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid crontab format").into());
    }
    
    std::fs::copy(file_path, &cron_file)?;
    println!("crontab: installing new crontab");
    
    Ok(())
}

fn read_from_stdin(user: &Option<String>) -> Result<()> {
    let cron_file = get_cron_file_path(user)?;
    
    let mut contents = String::new();
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        contents.push_str(&line?);
        contents.push('\n');
    }
    
    // Write to temp file first for validation
    let temp_file = format!("{}.tmp", cron_file.display());
    std::fs::write(&temp_file, &contents)?;
    
    if validate_cron_file(&temp_file)? {
        std::fs::rename(&temp_file, &cron_file)?;
        println!("crontab: installing new crontab");
    } else {
        std::fs::remove_file(&temp_file)?;
        return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "Invalid crontab format").into());
    }
    
    Ok(())
}

fn get_cron_file_path(user: &Option<String>) -> Result<std::path::PathBuf> {
    let default_user = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    let username = user.as_deref().unwrap_or(&default_user);

    if cfg!(windows) {
        let appdata = env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        Ok(std::path::PathBuf::from(appdata).join("nxsh").join("cron").join(username))
    } else {
        Ok(std::path::PathBuf::from("/var/spool/cron/crontabs").join(username))
    }
}

fn validate_cron_file(file_path: &str) -> Result<bool> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        
        // Basic cron format validation (5 or 6 fields + command)
        let fields: Vec<&str> = trimmed.split_whitespace().collect();
        if fields.len() < 6 {
            eprintln!("crontab: error: invalid line format: {line}");
            return Ok(false);
        }
        
        // Validate time fields
        for (i, field) in fields.iter().enumerate().take(5) {
            if !validate_time_field(field, i) {
                eprintln!("crontab: error: invalid time field {}: {}", i + 1, fields[i]);
                return Ok(false);
            }
        }
    }
    
    Ok(true)
}

fn validate_time_field(field: &str, index: usize) -> bool {
    if field == "*" {
        return true;
    }
    
    let ranges = match index {
        0 => (0, 59),   // minute
        1 => (0, 23),   // hour
        2 => (1, 31),   // day
        3 => (1, 12),   // month
        4 => (0, 7),    // weekday (0 and 7 are Sunday)
        _ => return false,
    };
    
    // Handle ranges, lists, and steps
    for part in field.split(',') {
        if part.contains('/') {
            let step_parts: Vec<&str> = part.split('/').collect();
            if step_parts.len() != 2 {
                return false;
            }
            if !validate_range_or_star(step_parts[0], ranges) {
                return false;
            }
            if step_parts[1].parse::<u32>().is_err() {
                return false;
            }
        } else if part.contains('-') {
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() != 2 {
                return false;
            }
            let start: Result<u32, _> = range_parts[0].parse();
            let end: Result<u32, _> = range_parts[1].parse();
            match (start, end) {
                (Ok(s), Ok(e)) => {
                    if s < ranges.0 || e > ranges.1 || s > e {
                        return false;
                    }
                }
                _ => return false,
            }
        } else if let Ok(num) = part.parse::<u32>() {
            if num < ranges.0 || num > ranges.1 {
                return false;
            }
        } else {
            return false;
        }
    }
    
    true
}

fn validate_range_or_star(field: &str, ranges: (u32, u32)) -> bool {
    if field == "*" {
        return true;
    }
    
    if let Ok(num) = field.parse::<u32>() {
        num >= ranges.0 && num <= ranges.1
    } else {
        false
    }
}

