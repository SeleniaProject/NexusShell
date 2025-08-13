//! `mkdir` command  Ecomprehensive directory creation implementation.
//!
//! Supports complete mkdir functionality:
//!   mkdir [OPTIONS] DIRECTORY...
//!   -m, --mode=MODE           - Set file mode (as in chmod), not a=rwx - umask
//!   -p, --parents             - No error if existing, make parent directories as needed
//!   -v, --verbose             - Print a message for each created directory
//!   -Z, --context=CTX         - Set the SELinux security context of each created directory
//!   --help                    - Display help and exit
//!   --version                 - Output version information and exit

use anyhow::{Result, anyhow};
use std::fs::{self};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct MkdirOptions {
    pub directories: Vec<String>,
    pub mode: Option<u32>,
    pub parents: bool,
    pub verbose: bool,
    pub context: Option<String>,
}


pub fn mkdir_cli(args: &[String]) -> Result<()> {
    let options = parse_mkdir_args(args)?;
    
    if options.directories.is_empty() {
        return Err(anyhow!("mkdir: missing operand"));
    }
    
    for directory in &options.directories {
        let path = PathBuf::from(directory);
        
        if let Err(e) = create_directory(&path, &options) {
            eprintln!("mkdir: {e}");
            // Continue with other directories instead of exiting
        }
    }
    
    Ok(())
}

fn parse_mkdir_args(args: &[String]) -> Result<MkdirOptions> {
    let mut options = MkdirOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "-m" | "--mode" => {
                if i + 1 < args.len() {
                    let mode_str = &args[i + 1];
                    options.mode = Some(parse_mode(mode_str)?);
                    i += 1;
                } else {
                    return Err(anyhow!("mkdir: option requires an argument -- m"));
                }
            }
            arg if arg.starts_with("--mode=") => {
                let mode_str = arg.strip_prefix("--mode=").unwrap();
                options.mode = Some(parse_mode(mode_str)?);
            }
            "-p" | "--parents" => {
                options.parents = true;
            }
            "-v" | "--verbose" => {
                options.verbose = true;
            }
            "-Z" | "--context" => {
                if i + 1 < args.len() {
                    options.context = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err(anyhow!("mkdir: option requires an argument -- Z"));
                }
            }
            arg if arg.starts_with("--context=") => {
                let context = arg.strip_prefix("--context=").unwrap();
                options.context = Some(context.to_string());
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--version" => {
                println!("mkdir (NexusShell) 1.0.0");
                std::process::exit(0);
            }
            arg if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                // Handle combined short options
                let mut chars = arg.chars().skip(1);
                while let Some(ch) = chars.next() {
                    match ch {
                        'm' => {
                            // Mode might be attached or separate
                            let rest: String = chars.collect();
                            if !rest.is_empty() {
                                options.mode = Some(parse_mode(&rest)?);
                                break;
                            } else if i + 1 < args.len() {
                                options.mode = Some(parse_mode(&args[i + 1])?);
                                i += 1;
                                break;
                            } else {
                                return Err(anyhow!("mkdir: option requires an argument -- m"));
                            }
                        }
                        'p' => options.parents = true,
                        'v' => options.verbose = true,
                        _ => return Err(anyhow!("mkdir: invalid option -- '{}'", ch)),
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

fn parse_mode(mode_str: &str) -> Result<u32> {
    // Handle octal mode (e.g., "755", "0755")
    if mode_str.chars().all(|c| c.is_digit(8)) {
        let mode = u32::from_str_radix(mode_str, 8)
            .map_err(|_| anyhow!("mkdir: invalid mode '{}'", mode_str))?;
        if mode > 0o7777 {
            return Err(anyhow!("mkdir: invalid mode '{}'", mode_str));
        }
        return Ok(mode);
    }
    
    // Handle symbolic mode (e.g., "u=rwx,g=rx,o=rx", "a+x", etc.)
    parse_symbolic_mode(mode_str)
}

fn parse_symbolic_mode(mode_str: &str) -> Result<u32> {
    let mut mode = 0o755; // Default mode for directories
    
    // Split by commas to handle multiple clauses
    for clause in mode_str.split(',') {
        mode = apply_symbolic_clause(mode, clause)?;
    }
    
    Ok(mode)
}

fn apply_symbolic_clause(mut mode: u32, clause: &str) -> Result<u32> {
    if clause.is_empty() {
        return Ok(mode);
    }
    
    // Parse who (u, g, o, a)
    let mut chars = clause.chars().peekable();
    let mut who_mask = 0u32;
    
    while let Some(&ch) = chars.peek() {
        match ch {
            'u' => { who_mask |= 0o700; chars.next(); }
            'g' => { who_mask |= 0o070; chars.next(); }
            'o' => { who_mask |= 0o007; chars.next(); }
            'a' => { who_mask |= 0o777; chars.next(); }
            _ => break,
        }
    }
    
    if who_mask == 0 {
        who_mask = 0o777; // Default to all if no who specified
    }
    
    // Parse operation (+, -, =)
    let operation = chars.next()
        .ok_or_else(|| anyhow!("mkdir: invalid mode clause '{}'", clause))?;
    
    // Parse permissions (r, w, x, X, s, t)
    let mut perm_mask = 0u32;
    
    for ch in chars {
        match ch {
            'r' => perm_mask |= 0o444,
            'w' => perm_mask |= 0o222,
            'x' => perm_mask |= 0o111,
            'X' => {
                // Execute only if directory or already has execute permission
                perm_mask |= 0o111;
            }
            's' => {
                // Set-user-ID and set-group-ID
                if who_mask & 0o700 != 0 { perm_mask |= 0o4000; }
                if who_mask & 0o070 != 0 { perm_mask |= 0o2000; }
            }
            't' => {
                // Sticky bit
                if who_mask & 0o007 != 0 { perm_mask |= 0o1000; }
            }
            _ => return Err(anyhow!("mkdir: invalid permission '{}'", ch)),
        }
    }
    
    // Apply the operation
    match operation {
        '+' => mode |= perm_mask & who_mask,
        '-' => mode &= !(perm_mask & who_mask),
        '=' => {
            mode &= !who_mask;
            mode |= perm_mask & who_mask;
        }
        _ => return Err(anyhow!("mkdir: invalid operation '{}'", operation)),
    }
    
    Ok(mode)
}

fn create_directory(path: &Path, options: &MkdirOptions) -> Result<()> {
    if options.parents {
        create_directory_with_parents(path, options)
    } else {
        create_single_directory(path, options)
    }
}

fn create_single_directory(path: &Path, options: &MkdirOptions) -> Result<()> {
    if path.exists() {
        return Err(anyhow!("cannot create directory '{}': File exists", path.display()));
    }
    
    // Create the directory
    fs::create_dir(path)
        .map_err(|e| anyhow!("cannot create directory '{}': {}", path.display(), e))?;
    
    // Set permissions if specified
    if let Some(mode) = options.mode {
        set_directory_permissions(path, mode)?;
    }
    
    // Set SELinux context if specified
    if let Some(ref context) = options.context {
        set_selinux_context(path, context)?;
    }
    
    if options.verbose {
        println!("mkdir: created directory '{}'", path.display());
    }
    
    Ok(())
}

fn create_directory_with_parents(path: &Path, options: &MkdirOptions) -> Result<()> {
    let mut components = Vec::new();
    let mut current = path;
    
    // Collect all components that need to be created
    while !current.exists() {
        components.push(current.to_path_buf());
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }
    
    // Create directories from parent to child
    components.reverse();
    
    for component in components {
        if !component.exists() {
            fs::create_dir(&component)
                .map_err(|e| anyhow!("cannot create directory '{}': {}", component.display(), e))?;
            
            // Set permissions if specified
            if let Some(mode) = options.mode {
                set_directory_permissions(&component, mode)?;
            }
            
            // Set SELinux context if specified
            if let Some(ref context) = options.context {
                set_selinux_context(&component, context)?;
            }
            
            if options.verbose {
                println!("mkdir: created directory '{}'", component.display());
            }
        }
    }
    
    Ok(())
}

fn set_directory_permissions(path: &Path, mode: u32) -> Result<()> {
    #[cfg(unix)]
    {
        #[cfg(unix)] use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(mode);
        fs::set_permissions(path, permissions)
            .map_err(|e| anyhow!("cannot set permissions for '{}': {}", path.display(), e))?;
    }
    #[cfg(not(unix))]
    {
        eprintln!("mkdir: warning: setting file permissions not supported on this platform");
    }
    Ok(())
}

fn set_selinux_context(path: &Path, context: &str) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        // Try to set SELinux context via xattr "security.selinux"
        // This requires appropriate privileges and SELinux enabled.
        use nix::sys::xattr;
        let name = "security.selinux";
        xattr::set(path, name, context.as_bytes(), xattr::XattrFlags::empty())
            .map_err(|e| anyhow!("mkdir: failed to set SELinux context on '{}': {}", path.display(), e))?;
        return Ok(());
    }
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("mkdir: warning: SELinux context setting not supported on this platform");
        Ok(())
    }
}

fn print_help() {
    println!("Usage: mkdir [OPTION]... DIRECTORY...");
    println!("Create the DIRECTORY(ies), if they do not already exist.");
    println!();
    println!("Mandatory arguments to long options are mandatory for short options too.");
    println!("  -m, --mode=MODE   set file mode (as in chmod), not a=rwx - umask");
    println!("  -p, --parents     no error if existing, make parent directories as needed");
    println!("  -v, --verbose     print a message for each created directory");
    println!("  -Z, --context=CTX  set the SELinux security context of each created");
    println!("                      directory to CTX");
    println!("      --help     display this help and exit");
    println!("      --version  output version information and exit");
    println!();
    println!("MODE may be specified in octal (e.g. 755) or symbolic notation (e.g. u=rwx,go=rx).");
    println!();
    println!("Examples:");
    println!("  mkdir newdir              Create directory 'newdir'");
    println!("  mkdir -p path/to/newdir   Create directory and any necessary parent directories");
    println!("  mkdir -m 755 newdir       Create directory with specific permissions");
    println!("  mkdir -v dir1 dir2 dir3   Create multiple directories with verbose output");
    println!();
    println!("Report mkdir bugs to <bug-reports@nexusshell.org>");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(unix)]
    #[cfg(unix)] use std::os::unix::fs::PermissionsExt;
    
    #[test]
    fn test_parse_args() {
        let args = vec!["-pv".to_string(), "dir1".to_string(), "dir2".to_string()];
        let options = parse_mkdir_args(&args).unwrap();
        
        assert!(options.parents);
        assert!(options.verbose);
        assert_eq!(options.directories, vec!["dir1", "dir2"]);
    }
    
    #[test]
    fn test_parse_mode_octal() {
        assert_eq!(parse_mode("755").unwrap(), 0o755);
        assert_eq!(parse_mode("0644").unwrap(), 0o644);
    }
    
    #[test]
    fn test_parse_mode_symbolic() {
        // Basic symbolic modes
        assert_eq!(parse_mode("u=rwx,g=rx,o=rx").unwrap(), 0o755);
        assert_eq!(parse_mode("u=rw,g=r,o=r").unwrap(), 0o644);
    }
    
    #[test]
    fn test_mode_with_arg() {
        let args = vec!["-m".to_string(), "755".to_string(), "testdir".to_string()];
        let options = parse_mkdir_args(&args).unwrap();
        
        assert_eq!(options.mode, Some(0o755));
        assert_eq!(options.directories, vec!["testdir"]);
    }
    
    #[test]
    fn test_combined_options() {
        let args = vec!["-pvm755".to_string(), "testdir".to_string()];
        let options = parse_mkdir_args(&args).unwrap();
        
        assert!(options.parents);
        assert!(options.verbose);
        assert_eq!(options.mode, Some(0o755));
        assert_eq!(options.directories, vec!["testdir"]);
    }
    
    #[test]
    fn test_apply_symbolic_clause() {
        let mut mode = 0o644;
        
        // Add execute permission for user
        mode = apply_symbolic_clause(mode, "u+x").unwrap();
        assert_eq!(mode, 0o744);
        
        // Remove write permission for group and others
        mode = apply_symbolic_clause(mode, "go-w").unwrap();
        assert_eq!(mode, 0o744);
        
        // Set all permissions for all
        mode = apply_symbolic_clause(mode, "a=rwx").unwrap();
        assert_eq!(mode, 0o777);
    }
} 


