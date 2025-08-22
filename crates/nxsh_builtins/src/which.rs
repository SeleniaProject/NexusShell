use std::env;
use std::path::PathBuf;
use crate::common::{BuiltinResult, BuiltinContext};

/// Locate a command in the PATH
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    if args.is_empty() {
        eprintln!("which: missing command name");
        return Ok(1);
    }

    let mut show_all = false;
    let mut commands = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--all" => show_all = true,
            "-h" | "--help" => {
                print_help();
                return Ok(0);
            }
            arg if arg.starts_with('-') => {
                eprintln!("which: invalid option '{}'", arg);
                return Ok(1);
            }
            _ => commands.push(&args[i]),
        }
        i += 1;
    }

    if commands.is_empty() {
        eprintln!("which: missing command name");
        return Ok(1);
    }

    let mut exit_code = 0;
    for &command in &commands {
        let found_paths = find_command(command, show_all);
        
        if found_paths.is_empty() {
            eprintln!("which: no {} in PATH", command);
            exit_code = 1;
        } else {
            for path in found_paths {
                println!("{}", path.display());
                if !show_all {
                    break; // Only show first match unless -a is specified
                }
            }
        }
    }

    Ok(exit_code)
}

fn find_command(command: &str, find_all: bool) -> Vec<PathBuf> {
    let mut found_paths = Vec::new();

    // Get PATH environment variable
    let path_var = match env::var("PATH") {
        Ok(path) => path,
        Err(_) => return found_paths,
    };

    // Split PATH by the appropriate delimiter
    let path_separator = if cfg!(windows) { ';' } else { ':' };
    let paths: Vec<&str> = path_var.split(path_separator).collect();

    // Executable extensions on Windows
    let exe_extensions = if cfg!(windows) {
        env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string())
            .to_uppercase()
            .split(';')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    } else {
        vec!["".to_string()] // Unix systems don't need extensions
    };

    for path_dir in paths {
        if path_dir.is_empty() {
            continue;
        }

        let path_buf = PathBuf::from(path_dir);
        if !path_buf.exists() || !path_buf.is_dir() {
            continue;
        }

        // Check for the command with each possible extension
        for extension in &exe_extensions {
            let mut command_path = path_buf.clone();
            let command_with_ext = if extension.is_empty() {
                command.to_string()
            } else {
                format!("{}{}", command, extension)
            };
            
            command_path.push(&command_with_ext);

            if command_path.exists() && is_executable(&command_path) {
                found_paths.push(command_path);
                if !find_all {
                    return found_paths; // Return first match unless finding all
                }
            }
        }

        // Also check for exact command name (case-sensitive)
        let mut exact_path = path_buf.clone();
        exact_path.push(command);
        if exact_path.exists() && is_executable(&exact_path) {
            found_paths.push(exact_path);
            if !find_all {
                return found_paths;
            }
        }
    }

    found_paths
}

fn is_executable(path: &PathBuf) -> bool {
    if !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = path.metadata() {
            let permissions = metadata.permissions();
            // Check if the file has execute permission for owner, group, or others
            return permissions.mode() & 0o111 != 0;
        }
        false
    }

    #[cfg(windows)]
    {
        // On Windows, if we can read the file metadata, it's generally executable
        // if it has an appropriate extension or is a .exe file
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_uppercase();
            return matches!(ext.as_str(), "EXE" | "COM" | "BAT" | "CMD" | "MSI");
        }
        
        // Also check PATHEXT environment variable
        if let Ok(pathext) = env::var("PATHEXT") {
            if let Some(extension) = path.extension() {
                let ext = format!(".{}", extension.to_string_lossy().to_uppercase());
                return pathext.to_uppercase().contains(&ext);
            }
        }
        
        false
    }

    #[cfg(not(any(unix, windows)))]
    {
        // Fallback for other platforms
        path.metadata().is_ok()
    }
}

fn print_help() {
    println!("Usage: which [OPTIONS] COMMAND...");
    println!("Locate a command in the user's PATH.");
    println!();
    println!("Options:");
    println!("  -a, --all       print all matching pathnames of each argument");
    println!("  -h, --help      display this help and exit");
    println!();
    println!("Exit status:");
    println!("  0   if all specified commands are found and executable");
    println!("  1   if one or more specified commands is nonexistent or not executable");
    println!("  2   if an invalid option is specified");
    println!();
    println!("Examples:");
    println!("  which ls            Find location of 'ls' command");
    println!("  which -a python     Find all locations of 'python' command");
    println!("  which cmd1 cmd2     Find locations of multiple commands");
}
