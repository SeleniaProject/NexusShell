//! `rmdir` command  Ecomprehensive empty directory removal implementation.
//!
//! Supports complete rmdir functionality:
//!   rmdir [OPTIONS] DIRECTORY...
//!   --ignore-fail-on-non-empty - Ignore each failure that is solely because a directory
//!                                 is non-empty
//!   -p, --parents               - Remove DIRECTORY and its ancestors; e.g., 'rmdir -p a/b/c'
//!                                 is similar to 'rmdir a/b/c a/b a'
//!   -v, --verbose               - Output a diagnostic for every directory processed
//!   --help                      - Display help and exit
//!   --version                   - Output version information and exit

use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use super::ui_design::{Colorize, TableFormatter, ColorPalette, Icons};

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct RmdirOptions {
    pub directories: Vec<String>,
    pub ignore_fail_on_non_empty: bool,
    pub parents: bool,
    pub verbose: bool,
}


pub fn rmdir_cli(args: &[String]) -> Result<()> {
    let options = parse_rmdir_args(args)?;
    
    if options.directories.is_empty() {
        return Err(anyhow!("rmdir: missing operand"));
    }
    
    let mut exit_code = 0;
    
    for directory in &options.directories {
        let path = PathBuf::from(directory);
        
        if let Err(e) = remove_directory(&path, &options) {
            if !options.ignore_fail_on_non_empty || !is_non_empty_error(&e) {
                eprintln!("rmdir: {e}");
                exit_code = 1;
            }
        }
    }
    
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    
    Ok(())
}

fn parse_rmdir_args(args: &[String]) -> Result<RmdirOptions> {
    let mut options = RmdirOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "--ignore-fail-on-non-empty" => {
                options.ignore_fail_on_non_empty = true;
            }
            "-p" | "--parents" => {
                options.parents = true;
            }
            "-v" | "--verbose" => {
                options.verbose = true;
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--version" => {
                println!("rmdir (NexusShell) 1.0.0");
                std::process::exit(0);
            }
            arg if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                // Handle combined short options
                for ch in arg.chars().skip(1) {
                    match ch {
                        'p' => options.parents = true,
                        'v' => options.verbose = true,
                        _ => return Err(anyhow!("rmdir: invalid option -- '{}'", ch)),
                    }
                }
            }
            _ => {
                // This is a directory name
                options.directories.push(arg.clone());
            }
        }
        i += 1;
    }
    
    Ok(options)
}

fn remove_directory(path: &Path, options: &RmdirOptions) -> Result<()> {
    if options.parents {
        remove_directory_with_parents(path, options)
    } else {
        remove_single_directory(path, options)
    }
}

fn remove_single_directory(path: &Path, options: &RmdirOptions) -> Result<()> {
    // Check if path exists
    if !path.exists() {
        return Err(anyhow!("failed to remove '{}': No such file or directory", path.display()));
    }
    
    // Check if it's actually a directory
    if !path.is_dir() {
        return Err(anyhow!("failed to remove '{}': Not a directory", path.display()));
    }
    
    // Attempt to remove the directory
    match fs::remove_dir(path) {
        Ok(()) => {
            if options.verbose {
                println!("{} {} {}", 
                    Icons::FOLDER_MINUS, 
                    "Removed directory:".colorize(&ColorPalette::WARNING),
                    path.display().to_string().colorize(&ColorPalette::ACCENT)
                );
            }
            Ok(())
        }
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::DirectoryNotEmpty => {
                    Err(anyhow!("failed to remove '{}': Directory not empty", path.display()))
                }
                std::io::ErrorKind::PermissionDenied => {
                    Err(anyhow!("failed to remove '{}': Permission denied", path.display()))
                }
                _ => {
                    Err(anyhow!("failed to remove '{}': {}", path.display(), e))
                }
            }
        }
    }
}

fn remove_directory_with_parents(path: &Path, options: &RmdirOptions) -> Result<()> {
    let mut current_path = path.to_path_buf();
    let mut directories_to_remove = Vec::new();
    
    // Collect all directories from the specified path up to the root
    loop {
        if current_path.exists() && current_path.is_dir() {
            directories_to_remove.push(current_path.clone());
        }
        
        match current_path.parent() {
            Some(parent) => {
                // Don't try to remove root directory or current directory
                if parent == Path::new("/") || parent == Path::new(".") || parent == Path::new("") {
                    break;
                }
                current_path = parent.to_path_buf();
            }
            None => break,
        }
    }
    
    // Remove directories from deepest to shallowest
    for dir_path in directories_to_remove {
        match remove_single_directory(&dir_path, options) {
            Ok(()) => {
                // Continue removing parent directories
            }
            Err(e) => {
                // If we fail to remove a directory, we should stop trying to remove its parents
                // unless it's a non-empty error and we're ignoring those
                if options.ignore_fail_on_non_empty && is_non_empty_error(&e) {
                    // Stop here but don't report the error
                    break;
                } else {
                    return Err(e);
                }
            }
        }
    }
    
    Ok(())
}

fn is_non_empty_error(error: &anyhow::Error) -> bool {
    error.to_string().contains("Directory not empty")
}

fn print_help() {
    println!("Usage: rmdir [OPTION]... DIRECTORY...");
    println!("Remove the DIRECTORY(ies), if they are empty.");
    println!();
    println!("      --ignore-fail-on-non-empty");
    println!("                  ignore each failure that is solely because a directory");
    println!("                    is non-empty");
    println!("  -p, --parents   remove DIRECTORY and its ancestors; e.g., 'rmdir -p a/b/c' is");
    println!("                    similar to 'rmdir a/b/c a/b a'");
    println!("  -v, --verbose   output a diagnostic for every directory processed");
    println!("      --help     display this help and exit");
    println!("      --version  output version information and exit");
    println!();
    println!("Examples:");
    println!("  rmdir empty_dir           Remove a single empty directory");
    println!("  rmdir -p a/b/c            Remove directory c and its parents a/b and a if empty");
    println!("  rmdir -v dir1 dir2        Remove directories with verbose output");
    println!("  rmdir --ignore-fail-on-non-empty dir");
    println!("                            Try to remove dir, ignore if not empty");
    println!();
    println!("Note: rmdir will only remove empty directories. To remove directories");
    println!("and their contents, use 'rm -r' instead.");
    println!();
    println!("Report rmdir bugs to <bug-reports@nexusshell.org>");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_parse_args() {
        let args = vec!["-pv".to_string(), "dir1".to_string(), "dir2".to_string()];
        let options = parse_rmdir_args(&args).unwrap();
        
        assert!(options.parents);
        assert!(options.verbose);
        assert_eq!(options.directories, vec!["dir1", "dir2"]);
    }
    
    #[test]
    fn test_ignore_fail_on_non_empty() {
        let args = vec!["--ignore-fail-on-non-empty".to_string(), "dir".to_string()];
        let options = parse_rmdir_args(&args).unwrap();
        
        assert!(options.ignore_fail_on_non_empty);
        assert_eq!(options.directories, vec!["dir"]);
    }
    
    #[test]
    fn test_combined_options() {
        let args = vec!["-pv".to_string(), "test/dir".to_string()];
        let options = parse_rmdir_args(&args).unwrap();
        
        assert!(options.parents);
        assert!(options.verbose);
        assert_eq!(options.directories, vec!["test/dir"]);
    }
    
    #[test]
    fn test_remove_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("empty_dir");
        fs::create_dir(&test_dir).unwrap();
        
        let options = RmdirOptions::default();
        remove_single_directory(&test_dir, &options).unwrap();
        
        assert!(!test_dir.exists());
    }
    
    #[test]
    fn test_remove_non_empty_directory_fails() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("non_empty_dir");
        fs::create_dir(&test_dir).unwrap();
        
        // Create a file inside the directory
        let file_path = test_dir.join("file.txt");
        fs::write(&file_path, "content").unwrap();
        
        let options = RmdirOptions::default();
        let result = remove_single_directory(&test_dir, &options);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Directory not empty"));
    }
    
    #[test]
    fn test_is_non_empty_error() {
        let error = anyhow!("failed to remove 'test': Directory not empty");
        assert!(is_non_empty_error(&error));
        
        let error = anyhow!("failed to remove 'test': Permission denied");
        assert!(!is_non_empty_error(&error));
    }
} 
