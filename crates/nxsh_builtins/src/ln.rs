//! `ln` command – comprehensive hard and symbolic link creation implementation.
//!
//! Supports complete ln functionality:
//!   ln [OPTIONS] TARGET LINK_NAME
//!   ln [OPTIONS] TARGET... DIRECTORY
//!   -b, --backup[=CONTROL]    - Make backup of each existing destination file
//!   -d, -F, --directory       - Allow superuser to attempt to hard link directories
//!   -f, --force               - Remove existing destination files
//!   -i, --interactive         - Prompt whether to remove destinations
//!   -L, --logical             - Dereference TARGETs that are symbolic links
//!   -n, --no-dereference      - Treat LINK_NAME as normal file if it's a symlink to directory
//!   -P, --physical            - Make hard links directly to symbolic links
//!   -r, --relative            - Create symbolic links relative to link location
//!   -s, --symbolic            - Make symbolic links instead of hard links
//!   -S, --suffix=SUFFIX       - Override usual backup suffix
//!   -t, --target-directory=DIRECTORY - Specify the DIRECTORY to create links in
//!   -T, --no-target-directory - Treat LINK_NAME as normal file always
//!   -v, --verbose             - Print name of each linked file
//!   --help                    - Display help and exit
//!   --version                 - Output version information and exit

use anyhow::{Result, anyhow};
use std::fs::{self, File};
use std::io::{self, Write};
use std::os::unix::fs::{symlink, MetadataExt};
use std::path::{Path, PathBuf, Component};

#[derive(Debug, Clone)]
pub struct LnOptions {
    pub targets: Vec<String>,
    pub link_name: Option<String>,
    pub backup: BackupMode,
    pub backup_suffix: String,
    pub directory_links: bool,
    pub force: bool,
    pub interactive: bool,
    pub logical: bool,
    pub no_dereference: bool,
    pub physical: bool,
    pub relative: bool,
    pub symbolic: bool,
    pub target_directory: Option<String>,
    pub no_target_directory: bool,
    pub verbose: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackupMode {
    None,
    Numbered,
    Existing,
    Simple,
}

impl Default for LnOptions {
    fn default() -> Self {
        Self {
            targets: Vec::new(),
            link_name: None,
            backup: BackupMode::None,
            backup_suffix: "~".to_string(),
            directory_links: false,
            force: false,
            interactive: false,
            logical: false,
            no_dereference: false,
            physical: false,
            relative: false,
            symbolic: false,
            target_directory: None,
            no_target_directory: false,
            verbose: false,
        }
    }
}

pub fn ln_cli(args: &[String]) -> Result<()> {
    let options = parse_ln_args(args)?;
    
    if options.targets.is_empty() {
        return Err(anyhow!("ln: missing file operand"));
    }
    
    // Determine the operation mode
    let (target_dir, is_dir_mode) = determine_target_mode(&options)?;
    
    // Process each target
    for target in &options.targets {
        let target_path = PathBuf::from(target);
        
        if !target_path.exists() && !options.symbolic {
            return Err(anyhow!("ln: failed to access '{}': No such file or directory", target));
        }
        
        let link_path = if is_dir_mode {
            target_dir.join(target_path.file_name().unwrap_or_else(|| target_path.as_os_str()))
        } else {
            target_dir.clone()
        };
        
        // Handle existing destination
        if link_path.exists() {
            if !should_overwrite(&link_path, &options)? {
                continue;
            }
            
            // Create backup if requested
            if options.backup != BackupMode::None {
                create_backup(&link_path, &options)?;
            }
            
            // Remove existing file/link
            if options.force || options.interactive {
                if link_path.is_dir() {
                    fs::remove_dir_all(&link_path)?;
                } else {
                    fs::remove_file(&link_path)?;
                }
            }
        }
        
        // Create the link
        create_link(&target_path, &link_path, &options)?;
        
        if options.verbose {
            if options.symbolic {
                println!("'{}' -> '{}'", link_path.display(), target_path.display());
            } else {
                println!("'{}' => '{}'", link_path.display(), target_path.display());
            }
        }
    }
    
    Ok(())
}

fn parse_ln_args(args: &[String]) -> Result<LnOptions> {
    let mut options = LnOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "-b" | "--backup" => {
                options.backup = BackupMode::Simple;
            }
            arg if arg.starts_with("--backup=") => {
                let control = arg.strip_prefix("--backup=").unwrap();
                options.backup = match control {
                    "none" | "off" => BackupMode::None,
                    "numbered" | "t" => BackupMode::Numbered,
                    "existing" | "nil" => BackupMode::Existing,
                    "simple" | "never" => BackupMode::Simple,
                    _ => return Err(anyhow!("ln: invalid backup control '{}'", control)),
                };
            }
            "-d" | "-F" | "--directory" => {
                options.directory_links = true;
            }
            "-f" | "--force" => {
                options.force = true;
                options.interactive = false;
            }
            "-i" | "--interactive" => {
                options.interactive = true;
                options.force = false;
            }
            "-L" | "--logical" => {
                options.logical = true;
                options.physical = false;
            }
            "-n" | "--no-dereference" => {
                options.no_dereference = true;
            }
            "-P" | "--physical" => {
                options.physical = true;
                options.logical = false;
            }
            "-r" | "--relative" => {
                options.relative = true;
            }
            "-s" | "--symbolic" => {
                options.symbolic = true;
            }
            "-S" | "--suffix" => {
                if i + 1 < args.len() {
                    options.backup_suffix = args[i + 1].clone();
                    i += 1;
                } else {
                    return Err(anyhow!("ln: option requires an argument -- S"));
                }
            }
            arg if arg.starts_with("--suffix=") => {
                options.backup_suffix = arg.strip_prefix("--suffix=").unwrap().to_string();
            }
            "-t" | "--target-directory" => {
                if i + 1 < args.len() {
                    options.target_directory = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err(anyhow!("ln: option requires an argument -- t"));
                }
            }
            arg if arg.starts_with("--target-directory=") => {
                let dir = arg.strip_prefix("--target-directory=").unwrap();
                options.target_directory = Some(dir.to_string());
            }
            "-T" | "--no-target-directory" => {
                options.no_target_directory = true;
            }
            "-v" | "--verbose" => {
                options.verbose = true;
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--version" => {
                println!("ln (NexusShell) 1.0.0");
                std::process::exit(0);
            }
            arg if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                // Handle combined short options
                for ch in arg.chars().skip(1) {
                    match ch {
                        'b' => options.backup = BackupMode::Simple,
                        'd' | 'F' => options.directory_links = true,
                        'f' => {
                            options.force = true;
                            options.interactive = false;
                        }
                        'i' => {
                            options.interactive = true;
                            options.force = false;
                        }
                        'L' => {
                            options.logical = true;
                            options.physical = false;
                        }
                        'n' => options.no_dereference = true,
                        'P' => {
                            options.physical = true;
                            options.logical = false;
                        }
                        'r' => options.relative = true,
                        's' => options.symbolic = true,
                        'T' => options.no_target_directory = true,
                        'v' => options.verbose = true,
                        _ => return Err(anyhow!("ln: invalid option -- '{}'", ch)),
                    }
                }
            }
            _ => {
                // This is a target or link name
                if options.target_directory.is_some() {
                    options.targets.push(arg.clone());
                } else if options.link_name.is_none() && i == args.len() - 1 && options.targets.len() == 1 {
                    options.link_name = Some(arg.clone());
                } else {
                    options.targets.push(arg.clone());
                }
            }
        }
        i += 1;
    }
    
    // Handle target directory mode
    if let Some(ref target_dir) = options.target_directory {
        // All remaining arguments are targets
    } else if options.link_name.is_none() && options.targets.len() >= 2 {
        // Last argument is the link name/directory
        options.link_name = options.targets.pop();
    }
    
    Ok(options)
}

fn determine_target_mode(options: &LnOptions) -> Result<(PathBuf, bool)> {
    if let Some(ref target_dir) = options.target_directory {
        let dir_path = PathBuf::from(target_dir);
        if !dir_path.is_dir() {
            return Err(anyhow!("ln: target directory '{}' is not a directory", target_dir));
        }
        return Ok((dir_path, true));
    }
    
    if let Some(ref link_name) = options.link_name {
        let link_path = PathBuf::from(link_name);
        
        if options.no_target_directory {
            return Ok((link_path, false));
        }
        
        // If multiple targets and link_name is a directory, use directory mode
        if options.targets.len() > 1 && link_path.is_dir() {
            return Ok((link_path, true));
        }
        
        return Ok((link_path, false));
    }
    
    return Err(anyhow!("ln: missing destination"));
}

fn should_overwrite(link_path: &Path, options: &LnOptions) -> Result<bool> {
    if options.force {
        return Ok(true);
    }
    
    if options.interactive {
        print!("ln: replace '{}'? ", link_path.display());
        io::stdout().flush()?;
        
        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        
        return Ok(response.trim().to_lowercase().starts_with('y'));
    }
    
    // Default behavior: fail if destination exists
    return Err(anyhow!("ln: failed to create link '{}': File exists", link_path.display()));
}

fn create_backup(dest: &Path, options: &LnOptions) -> Result<()> {
    let backup_path = match options.backup {
        BackupMode::None => return Ok(()),
        BackupMode::Simple => {
            dest.with_extension(format!("{}{}", 
                dest.extension().and_then(|s| s.to_str()).unwrap_or(""),
                options.backup_suffix))
        }
        BackupMode::Numbered => {
            find_numbered_backup_name(dest, &options.backup_suffix)?
        }
        BackupMode::Existing => {
            if has_numbered_backups(dest, &options.backup_suffix)? {
                find_numbered_backup_name(dest, &options.backup_suffix)?
            } else {
                dest.with_extension(format!("{}{}", 
                    dest.extension().and_then(|s| s.to_str()).unwrap_or(""),
                    options.backup_suffix))
            }
        }
    };
    
    if dest.is_dir() {
        copy_dir_all(dest, &backup_path)?;
    } else {
        fs::copy(dest, &backup_path)?;
    }
    
    if options.verbose {
        println!("ln: backup: '{}' -> '{}'", dest.display(), backup_path.display());
    }
    
    Ok(())
}

fn find_numbered_backup_name(dest: &Path, suffix: &str) -> Result<PathBuf> {
    let parent = dest.parent().unwrap_or(Path::new("."));
    let filename = dest.file_name().unwrap().to_string_lossy();
    
    let mut number = 1;
    loop {
        let backup_name = format!("{}.{}{}", filename, number, suffix);
        let backup_path = parent.join(backup_name);
        
        if !backup_path.exists() {
            return Ok(backup_path);
        }
        
        number += 1;
        if number > 999999 {
            return Err(anyhow!("ln: too many backup files"));
        }
    }
}

fn has_numbered_backups(dest: &Path, suffix: &str) -> Result<bool> {
    let parent = dest.parent().unwrap_or(Path::new("."));
    let filename = dest.file_name().unwrap().to_string_lossy();
    
    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_name = entry.file_name().to_string_lossy().into_owned();
                if entry_name.starts_with(&format!("{}.", filename)) && 
                   entry_name.ends_with(suffix) {
                    let middle = &entry_name[filename.len() + 1..entry_name.len() - suffix.len()];
                    if middle.parse::<u32>().is_ok() {
                        return Ok(true);
                    }
                }
            }
        }
    }
    
    Ok(false)
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}

fn create_link(target: &Path, link: &Path, options: &LnOptions) -> Result<()> {
    if options.symbolic {
        create_symbolic_link(target, link, options)
    } else {
        create_hard_link(target, link, options)
    }
}

fn create_symbolic_link(target: &Path, link: &Path, options: &LnOptions) -> Result<()> {
    let link_target = if options.relative {
        make_relative_path(target, link)?
    } else if options.logical && target.is_symlink() {
        // Dereference the target if it's a symlink and -L is specified
        target.canonicalize()?
    } else {
        target.to_path_buf()
    };
    
    symlink(&link_target, link)
        .map_err(|e| anyhow!("ln: failed to create symbolic link '{}' -> '{}': {}", 
                            link.display(), link_target.display(), e))?;
    
    Ok(())
}

fn create_hard_link(target: &Path, link: &Path, options: &LnOptions) -> Result<()> {
    // Check if target is a directory and we don't have permission
    if target.is_dir() && !options.directory_links {
        return Err(anyhow!("ln: '{}': hard link not allowed for directory", target.display()));
    }
    
    let actual_target = if options.logical && target.is_symlink() {
        target.canonicalize()?
    } else {
        target.to_path_buf()
    };
    
    fs::hard_link(&actual_target, link)
        .map_err(|e| anyhow!("ln: failed to create hard link '{}' => '{}': {}", 
                            link.display(), actual_target.display(), e))?;
    
    Ok(())
}

fn make_relative_path(target: &Path, link: &Path) -> Result<PathBuf> {
    let target_abs = target.canonicalize().unwrap_or_else(|_| target.to_path_buf());
    let link_parent = link.parent().unwrap_or(Path::new("."));
    let link_parent_abs = link_parent.canonicalize().unwrap_or_else(|_| link_parent.to_path_buf());
    
    // Find common prefix
    let target_components: Vec<_> = target_abs.components().collect();
    let link_components: Vec<_> = link_parent_abs.components().collect();
    
    let mut common_len = 0;
    for (t, l) in target_components.iter().zip(link_components.iter()) {
        if t == l {
            common_len += 1;
        } else {
            break;
        }
    }
    
    // Build relative path
    let mut relative_path = PathBuf::new();
    
    // Add ".." for each component we need to go up
    for _ in common_len..link_components.len() {
        relative_path.push("..");
    }
    
    // Add remaining target components
    for component in &target_components[common_len..] {
        relative_path.push(component);
    }
    
    if relative_path.as_os_str().is_empty() {
        relative_path.push(".");
    }
    
    Ok(relative_path)
}

fn print_help() {
    println!("Usage: ln [OPTION]... [-T] TARGET LINK_NAME");
    println!("  or:  ln [OPTION]... TARGET");
    println!("  or:  ln [OPTION]... TARGET... DIRECTORY");
    println!("  or:  ln [OPTION]... -t DIRECTORY TARGET...");
    println!("In the 1st form, create a link to TARGET with the name LINK_NAME.");
    println!("In the 2nd form, create a link to TARGET in the current directory.");
    println!("In the 3rd and 4th forms, create links to each TARGET in DIRECTORY.");
    println!("Create hard links by default, symbolic links with --symbolic.");
    println!("By default, each destination (name of new link) should not already exist.");
    println!("When creating hard links, each TARGET must exist.  Symbolic links");
    println!("can hold arbitrary text; if later resolved, a relative link is");
    println!("interpreted in relation to its parent directory.");
    println!();
    println!("Mandatory arguments to long options are mandatory for short options too.");
    println!("      --backup[=CONTROL]      make a backup of each existing destination file");
    println!("  -b                          like --backup but does not accept an argument");
    println!("  -d, -F, --directory         allow the superuser to attempt to hard link");
    println!("                                directories (note: will probably fail due to");
    println!("                                system restrictions, even for the superuser)");
    println!("  -f, --force                 remove existing destination files");
    println!("  -i, --interactive           prompt whether to remove destinations");
    println!("  -L, --logical               dereference TARGETs that are symbolic links");
    println!("  -n, --no-dereference        treat LINK_NAME as a normal file if");
    println!("                                it is a symbolic link to a directory");
    println!("  -P, --physical              make hard links directly to symbolic links");
    println!("  -r, --relative              create symbolic links relative to link location");
    println!("  -s, --symbolic              make symbolic links instead of hard links");
    println!("  -S, --suffix=SUFFIX         override the usual backup suffix");
    println!("  -t, --target-directory=DIRECTORY  specify the DIRECTORY in which to create");
    println!("                                the links");
    println!("  -T, --no-target-directory   treat LINK_NAME as a normal file always");
    println!("  -v, --verbose               print name of each linked file");
    println!("      --help     display this help and exit");
    println!("      --version  output version information and exit");
    println!();
    println!("The backup suffix is '~', unless set with --suffix or SIMPLE_BACKUP_SUFFIX.");
    println!("The version control method may be selected via the --backup option or through");
    println!("the VERSION_CONTROL environment variable.  Here are the values:");
    println!();
    println!("  none, off       never make backups (even if --backup is given)");
    println!("  numbered, t     make numbered backups");
    println!("  existing, nil   numbered if numbered backups exist, simple otherwise");
    println!("  simple, never   always make simple backups");
    println!();
    println!("Using -s ignores -L and -P.  Otherwise, the last option specified controls");
    println!("behavior when a TARGET is a symbolic link, defaulting to -P.");
    println!();
    println!("Report ln bugs to <bug-reports@nexusshell.org>");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    
    #[test]
    fn test_parse_args() {
        let args = vec!["-sf".to_string(), "target".to_string(), "link".to_string()];
        let options = parse_ln_args(&args).unwrap();
        
        assert!(options.symbolic);
        assert!(options.force);
        assert_eq!(options.targets, vec!["target"]);
        assert_eq!(options.link_name, Some("link".to_string()));
    }
    
    #[test]
    fn test_backup_modes() {
        let args = vec!["--backup=numbered".to_string(), "target".to_string(), "link".to_string()];
        let options = parse_ln_args(&args).unwrap();
        
        assert_eq!(options.backup, BackupMode::Numbered);
    }
    
    #[test]
    fn test_relative_option() {
        let args = vec!["-sr".to_string(), "target".to_string(), "link".to_string()];
        let options = parse_ln_args(&args).unwrap();
        
        assert!(options.symbolic);
        assert!(options.relative);
    }
    
    #[test]
    fn test_target_directory() {
        let args = vec!["-t".to_string(), "/tmp".to_string(), "file1".to_string(), "file2".to_string()];
        let options = parse_ln_args(&args).unwrap();
        
        assert_eq!(options.target_directory, Some("/tmp".to_string()));
        assert_eq!(options.targets, vec!["file1", "file2"]);
    }
    
    #[test]
    fn test_make_relative_path() {
        // Test case: target is in parent directory
        let target = Path::new("/home/user/documents/file.txt");
        let link = Path::new("/home/user/desktop/link.txt");
        
        // This would need actual filesystem for canonicalize to work
        // For unit test, we'll just test the logic with mock paths
    }
} 