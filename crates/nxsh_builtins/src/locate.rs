use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::Result;
use nxsh_core::{ErrorKind, ShellError};
use nxsh_core::error::{RuntimeErrorKind, SystemErrorKind};

pub fn locate_cli(args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        print_help();
        return Ok(());
    }

    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let mut case_insensitive = false;
    let mut limit = None;
    let mut basename_only = false;
    let mut existing_only = true;
    let mut regex_mode = false;
    let mut database_path = None;
    let mut patterns = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-i" | "--ignore-case" => case_insensitive = true,
            "-l" | "--limit" => {
                i += 1;
                if i < args.len() {
                    limit = args[i].parse().ok();
                }
            }
            "-b" | "--basename" => basename_only = true,
            "-e" | "--existing" => existing_only = true,
            "-A" | "--all" => existing_only = false,
            "-r" | "--regexp" => regex_mode = true,
            "-d" | "--database" => {
                i += 1;
                if i < args.len() {
                    database_path = Some(args[i].clone());
                }
            }
            "-V" | "--version" => {
                print_version();
                return Ok(());
            }
            "-S" | "--statistics" => {
                print_statistics(database_path.as_deref())?;
                return Ok(());
            }
            "-u" | "--update" => {
                update_database(database_path.as_deref())?;
                return Ok(());
            }
            pattern if !pattern.starts_with('-') => {
                patterns.push(pattern.to_string());
            }
            _ => {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("Unknown option: {}", args[i])
                ).into());
            }
        }
        i += 1;
    }

    if patterns.is_empty() {
        return Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            "No patterns specified"
        ).into());
    }

    // Try to use system locate if available
    if let Ok(results) = use_system_locate(&patterns, case_insensitive, limit, basename_only, existing_only, regex_mode) {
        for result in results {
            println!("{result}");
        }
    } else {
        // Fallback to our own implementation
        for pattern in patterns {
            let results = search_pattern(&pattern, case_insensitive, limit, basename_only, existing_only, regex_mode)?;
            for result in results {
                println!("{result}");
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!("Usage: locate [options] pattern...");
    println!();
    println!("Find files by name using a prebuilt database.");
    println!();
    println!("Options:");
    println!("  -i, --ignore-case     Ignore case distinctions");
    println!("  -l, --limit N         Limit output to N entries");
    println!("  -b, --basename        Match only the base name of path names");
    println!("  -e, --existing        Print only entries that refer to files existing at");
    println!("                        the time locate is run");
    println!("  -A, --all             Print entries even if they don't exist");
    println!("  -r, --regexp          Interpret pattern as regular expression");
    println!("  -d, --database PATH   Use PATH instead of default database");
    println!("  -S, --statistics      Display database statistics and exit");
    println!("  -u, --update          Update the database");
    println!("  -V, --version         Display version information and exit");
    println!("  -h, --help            Display this help and exit");
    println!();
    println!("Examples:");
    println!("  locate passwd         # Find files containing 'passwd'");
    println!("  locate -i README      # Case-insensitive search for README");
    println!("  locate -l 10 '*.txt'  # Limit to 10 .txt files");
    println!("  locate -b python      # Match basename only");
}

fn print_version() {
    println!("locate (NexusShell implementation) 1.0.0");
}

fn print_statistics(database_path: Option<&str>) -> Result<()> {
    let db_path = get_database_path(database_path);
    
    if !db_path.exists() {
        println!("Database {} does not exist", db_path.display());
        println!("Run 'locate -u' to create the database");
        return Ok(());
    }

    let metadata = fs::metadata(&db_path)?;
    let file_size = metadata.len();
    
    // Try to count entries (simplified)
    let content = fs::read_to_string(&db_path).unwrap_or_default();
    let entry_count = content.lines().count();
    
    println!("Database: {}", db_path.display());
    println!("File size: {file_size} bytes");
    println!("Entries: {entry_count}");
    println!("Last modified: {:?}", metadata.modified().unwrap_or(std::time::UNIX_EPOCH));
    
    Ok(())
}

fn update_database(database_path: Option<&str>) -> Result<()> {
    let db_path = get_database_path(database_path);
    
    println!("Updating database: {}", db_path.display());
    
    // Create database directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Try to use system updatedb if available
    if Command::new("updatedb")
        .arg("--localpaths=.")
        .arg(format!("--output={}", db_path.display()))
        .status()
        .is_ok() {
        println!("Database updated successfully");
        return Ok(());
    }

    // Fallback to our own database creation
    create_database(&db_path)?;
    println!("Database created successfully");
    
    Ok(())
}

fn use_system_locate(
    patterns: &[String],
    case_insensitive: bool,
    limit: Option<u32>,
    basename_only: bool,
    existing_only: bool,
    regex_mode: bool,
) -> Result<Vec<String>> {
    let mut cmd = Command::new("locate");
    
    if case_insensitive {
        cmd.arg("-i");
    }
    if let Some(limit_val) = limit {
        cmd.arg("-l").arg(limit_val.to_string());
    }
    if basename_only {
        cmd.arg("-b");
    }
    if !existing_only {
        cmd.arg("-A");
    }
    if regex_mode {
        cmd.arg("-r");
    }
    
    for pattern in patterns {
        cmd.arg(pattern);
    }
    
    let output = cmd.output()?;
    
    if !output.status.success() {
        return Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            "System locate command failed"
        ).into());
    }
    
    let results: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();
    
    Ok(results)
}

fn search_pattern(
    pattern: &str,
    case_insensitive: bool,
    limit: Option<u32>,
    basename_only: bool,
    existing_only: bool,
    regex_mode: bool,
) -> Result<Vec<String>> {
    let db_path = get_database_path(None);
    
    if !db_path.exists() {
        return Err(ShellError::new(
            ErrorKind::SystemError(SystemErrorKind::SystemCallError),
            format!("Database {} does not exist. Run 'locate -u' to create it.", db_path.display())
        ).into());
    }

    let content = fs::read_to_string(&db_path)?;
    let mut results = Vec::new();
    let mut count = 0;
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Check if file exists if existing_only is true
        if existing_only && !PathBuf::from(line).exists() {
            continue;
        }
        
        let match_text = if basename_only {
            PathBuf::from(line)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(line)
                .to_string()
        } else {
            line.to_string()
        };
        
        let pattern_to_match = if case_insensitive {
            pattern.to_lowercase()
        } else {
            pattern.to_string()
        };
        
        let text_to_match = if case_insensitive {
            match_text.to_lowercase()
        } else {
            match_text.to_string()
        };
        
        let matches = if regex_mode {
            // Simple regex matching (basic implementation)
            text_to_match.contains(&pattern_to_match)
        } else {
            // Glob pattern matching
            glob_match(&pattern_to_match, &text_to_match)
        };
        
        if matches {
            results.push(line.to_string());
            count += 1;
            
            if let Some(limit_val) = limit {
                if count >= limit_val {
                    break;
                }
            }
        }
    }
    
    Ok(results)
}

fn glob_match(pattern: &str, text: &str) -> bool {
    // Simple glob matching with * and ?
    if pattern == "*" {
        return true;
    }
    
    if !pattern.contains('*') && !pattern.contains('?') {
        return text.contains(pattern);
    }
    
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    
    glob_match_recursive(&pattern_chars, &text_chars, 0, 0)
}

fn glob_match_recursive(pattern: &[char], text: &[char], p: usize, t: usize) -> bool {
    if p >= pattern.len() {
        return t >= text.len();
    }
    
    if t >= text.len() && pattern[p] != '*' {
        return p >= pattern.len();
    }
    
    match pattern[p] {
        '*' => {
            // Try matching zero or more characters
            for i in t..=text.len() {
                if glob_match_recursive(pattern, text, p + 1, i) {
                    return true;
                }
            }
            false
        }
        '?' => {
            // Match exactly one character
            if t < text.len() {
                glob_match_recursive(pattern, text, p + 1, t + 1)
            } else {
                false
            }
        }
        c => {
            // Match exact character
            if t < text.len() && text[t] == c {
                glob_match_recursive(pattern, text, p + 1, t + 1)
            } else {
                false
            }
        }
    }
}

fn get_database_path(custom_path: Option<&str>) -> PathBuf {
    if let Some(path) = custom_path {
        PathBuf::from(path)
    } else if cfg!(windows) {
        let appdata = env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata).join("nxsh").join("locate.db")
    } else {
        PathBuf::from("/var/lib/mlocate/mlocate.db")
            .if_exists()
            .unwrap_or_else(|| {
                let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".cache").join("nxsh").join("locate.db")
            })
    }
}

fn create_database(db_path: &Path) -> Result<()> {
    let mut entries = Vec::new();
    
    // Get search roots
    let search_roots = get_search_roots();
    
    for root in search_roots {
        collect_files(&root, &mut entries)?;
    }
    
    // Sort entries
    entries.sort();
    
    // Write to database
    let content = entries.join("\n");
    fs::write(db_path, content)?;
    
    Ok(())
}

fn get_search_roots() -> Vec<PathBuf> {
    if cfg!(windows) {
        vec![
            PathBuf::from("C:\\"),
            // Add other drives if they exist
        ]
    } else {
        vec![
            PathBuf::from("/"),
        ]
    }
}

fn collect_files(root: &Path, entries: &mut Vec<String>) -> Result<()> {
    let max_entries = 100000; // Limit to prevent excessive memory usage
    
    if entries.len() >= max_entries {
        return Ok(());
    }
    
    if !root.is_dir() {
        return Ok(());
    }
    
    // Skip certain directories
    let root_str = root.to_string_lossy();
    if root_str.contains("/proc") ||
       root_str.contains("/dev") ||
       root_str.contains("/sys") ||
       root_str.contains("/.git") ||
       root_str.contains("/node_modules") {
        return Ok(());
    }
    
    if let Ok(dir_entries) = fs::read_dir(root) {
        for entry in dir_entries.flatten() {
            if entries.len() >= max_entries {
                break;
            }
            
            let path = entry.path();
            entries.push(path.to_string_lossy().to_string());
            
            if path.is_dir() {
                // Recursively collect from subdirectories
                collect_files(path.as_path(), entries)?;
            }
        }
    }
    
    Ok(())
}

// Utility trait for path existence check
trait PathExtension {
    fn if_exists(self) -> Option<PathBuf>;
}

impl PathExtension for PathBuf {
    fn if_exists(self) -> Option<PathBuf> {
        if self.exists() {
            Some(self)
        } else {
            None
        }
    }
}

// Additional utility functions

pub fn quick_locate(pattern: &str) -> Vec<String> {
    search_pattern(pattern, false, None, false, true, false)
        .unwrap_or_default()
}

pub fn locate_exact(filename: &str) -> Vec<String> {
    search_pattern(filename, false, None, true, true, false)
        .unwrap_or_default()
}

pub fn locate_with_limit(pattern: &str, limit: u32) -> Vec<String> {
    search_pattern(pattern, false, Some(limit), false, true, false)
        .unwrap_or_default()
}

pub fn database_exists() -> bool {
    get_database_path(None).exists()
}

pub fn get_database_info() -> Result<(PathBuf, u64, usize)> {
    let db_path = get_database_path(None);
    
    if !db_path.exists() {
        return Err(ShellError::new(
            ErrorKind::SystemError(SystemErrorKind::SystemCallError),
            "Database does not exist"
        ).into());
    }
    
    let metadata = fs::metadata(&db_path)?;
    let content = fs::read_to_string(&db_path)?;
    let entry_count = content.lines().count();
    
    Ok((db_path, metadata.len(), entry_count))
}
