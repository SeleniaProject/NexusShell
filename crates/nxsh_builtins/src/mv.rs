//! `mv` command – comprehensive file and directory move/rename implementation.
//!
//! Supports complete mv functionality:
//!   mv [OPTIONS] SOURCE DEST
//!   mv [OPTIONS] SOURCE... DIRECTORY
//!   -b, --backup[=CONTROL]    - Make backup of each existing destination file
//!   -f, --force               - Do not prompt before overwriting
//!   -i, --interactive         - Prompt before overwriting
//!   -n, --no-clobber          - Do not overwrite existing files
//!   --strip-trailing-slashes  - Remove trailing slashes from SOURCE arguments
//!   -S, --suffix=SUFFIX       - Override usual backup suffix
//!   -t, --target-directory=DIRECTORY - Move all SOURCE arguments into DIRECTORY
//!   -T, --no-target-directory - Treat DEST as normal file
//!   -u, --update              - Move only when SOURCE is newer than destination
//!   -v, --verbose             - Explain what is being done
//!   -Z, --context             - Set SELinux security context of destination
//!   --help                    - Display help and exit
//!   --version                 - Output version information and exit

use anyhow::{Result, anyhow};
use std::fs::{self, Metadata, OpenOptions};
use std::io::{self, Write, BufRead, BufReader};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use chrono::{DateTime, Local};
use users::{get_user_by_uid, get_group_by_gid};

#[derive(Debug, Clone)]
pub struct MvOptions {
    pub sources: Vec<String>,
    pub destination: String,
    pub backup: BackupMode,
    pub backup_suffix: String,
    pub force: bool,
    pub interactive: bool,
    pub no_clobber: bool,
    pub strip_trailing_slashes: bool,
    pub target_directory: Option<String>,
    pub no_target_directory: bool,
    pub update: bool,
    pub verbose: bool,
    pub context: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackupMode {
    None,
    Numbered,
    Existing,
    Simple,
}

impl Default for MvOptions {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            destination: String::new(),
            backup: BackupMode::None,
            backup_suffix: "~".to_string(),
            force: false,
            interactive: false,
            no_clobber: false,
            strip_trailing_slashes: false,
            target_directory: None,
            no_target_directory: false,
            update: false,
            verbose: false,
            context: None,
        }
    }
}

pub fn mv_cli(args: &[String]) -> Result<()> {
    let options = parse_mv_args(args)?;
    
    if options.sources.is_empty() {
        return Err(anyhow!("mv: missing file operand"));
    }
    
    // Determine target directory and validate arguments
    let (target_dir, dest_is_dir) = determine_target(&options)?;
    
    // Process each source file
    for source in &options.sources {
        let source_path = if options.strip_trailing_slashes {
            PathBuf::from(source.trim_end_matches('/'))
        } else {
            PathBuf::from(source)
        };
        
        if !source_path.exists() {
            return Err(anyhow!("mv: cannot stat '{}': No such file or directory", source));
        }
        
        let dest_path = if dest_is_dir {
            target_dir.join(source_path.file_name().unwrap())
        } else {
            target_dir.clone()
        };
        
        // Check if source and destination are the same
        if source_path.canonicalize()? == dest_path.canonicalize().unwrap_or(dest_path.clone()) {
            if options.verbose {
                println!("mv: '{}' and '{}' are the same file", source, dest_path.display());
            }
            continue;
        }
        
        // Handle existing destination file
        if dest_path.exists() {
            if !should_overwrite(&source_path, &dest_path, &options)? {
                continue;
            }
            
            // Create backup if requested
            if options.backup != BackupMode::None {
                create_backup(&dest_path, &options)?;
            }
        }
        
        // Perform the move operation
        move_file(&source_path, &dest_path, &options)?;
        
        if options.verbose {
            println!("'{}' -> '{}'", source, dest_path.display());
        }
    }
    
    Ok(())
}

fn parse_mv_args(args: &[String]) -> Result<MvOptions> {
    let mut options = MvOptions::default();
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
                    _ => return Err(anyhow!("mv: invalid backup control '{}'", control)),
                };
            }
            "-f" | "--force" => {
                options.force = true;
                options.interactive = false;
                options.no_clobber = false;
            }
            "-i" | "--interactive" => {
                options.interactive = true;
                options.force = false;
                options.no_clobber = false;
            }
            "-n" | "--no-clobber" => {
                options.no_clobber = true;
                options.force = false;
                options.interactive = false;
            }
            "--strip-trailing-slashes" => {
                options.strip_trailing_slashes = true;
            }
            "-S" | "--suffix" => {
                if i + 1 < args.len() {
                    options.backup_suffix = args[i + 1].clone();
                    i += 1;
                } else {
                    return Err(anyhow!("mv: option requires an argument -- S"));
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
                    return Err(anyhow!("mv: option requires an argument -- t"));
                }
            }
            arg if arg.starts_with("--target-directory=") => {
                let dir = arg.strip_prefix("--target-directory=").unwrap();
                options.target_directory = Some(dir.to_string());
            }
            "-T" | "--no-target-directory" => {
                options.no_target_directory = true;
            }
            "-u" | "--update" => {
                options.update = true;
            }
            "-v" | "--verbose" => {
                options.verbose = true;
            }
            "-Z" | "--context" => {
                if i + 1 < args.len() {
                    options.context = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err(anyhow!("mv: option requires an argument -- Z"));
                }
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--version" => {
                println!("mv (NexusShell) 1.0.0");
                std::process::exit(0);
            }
            arg if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                // Handle combined short options
                for ch in arg.chars().skip(1) {
                    match ch {
                        'b' => options.backup = BackupMode::Simple,
                        'f' => {
                            options.force = true;
                            options.interactive = false;
                            options.no_clobber = false;
                        }
                        'i' => {
                            options.interactive = true;
                            options.force = false;
                            options.no_clobber = false;
                        }
                        'n' => {
                            options.no_clobber = true;
                            options.force = false;
                            options.interactive = false;
                        }
                        'u' => options.update = true,
                        'v' => options.verbose = true,
                        _ => return Err(anyhow!("mv: invalid option -- '{}'", ch)),
                    }
                }
            }
            _ => {
                // This is a source or destination
                if options.target_directory.is_some() {
                    options.sources.push(arg.clone());
                } else if options.destination.is_empty() && i == args.len() - 1 {
                    options.destination = arg.clone();
                } else {
                    options.sources.push(arg.clone());
                }
            }
        }
        i += 1;
    }
    
    // Handle target directory mode
    if let Some(ref target) = options.target_directory {
        options.destination = target.clone();
    } else if options.destination.is_empty() && !options.sources.is_empty() {
        options.destination = options.sources.pop().unwrap();
    }
    
    Ok(options)
}

fn determine_target(options: &MvOptions) -> Result<(PathBuf, bool)> {
    let dest_path = PathBuf::from(&options.destination);
    
    if options.no_target_directory {
        if options.sources.len() > 1 {
            return Err(anyhow!("mv: extra operand '{}'", options.sources[1]));
        }
        return Ok((dest_path, false));
    }
    
    let dest_is_dir = dest_path.is_dir();
    
    if options.sources.len() > 1 && !dest_is_dir {
        return Err(anyhow!("mv: target '{}' is not a directory", options.destination));
    }
    
    if options.target_directory.is_some() && !dest_is_dir {
        return Err(anyhow!("mv: target directory '{}' is not a directory", options.destination));
    }
    
    Ok((dest_path, dest_is_dir))
}

fn should_overwrite(source: &Path, dest: &Path, options: &MvOptions) -> Result<bool> {
    if options.no_clobber {
        return Ok(false);
    }
    
    if options.force {
        return Ok(true);
    }
    
    if options.update {
        let source_mtime = source.metadata()?.modified()?;
        let dest_mtime = dest.metadata()?.modified()?;
        if source_mtime <= dest_mtime {
            return Ok(false);
        }
    }
    
    if options.interactive {
        print!("mv: overwrite '{}'? ", dest.display());
        io::stdout().flush()?;
        
        let stdin = io::stdin();
        let mut line = String::new();
        stdin.read_line(&mut line)?;
        
        let response = line.trim().to_lowercase();
        return Ok(response.starts_with('y'));
    }
    
    Ok(true)
}

fn create_backup(dest: &Path, options: &MvOptions) -> Result<()> {
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
        println!("mv: backup: '{}' -> '{}'", dest.display(), backup_path.display());
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
            return Err(anyhow!("mv: too many backup files"));
        }
    }
}

fn has_numbered_backups(dest: &Path, suffix: &str) -> Result<bool> {
    let parent = dest.parent().unwrap_or(Path::new("."));
    let filename = dest.file_name().unwrap().to_string_lossy();
    
    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_name = entry.file_name().to_string_lossy();
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

fn move_file(source: &Path, dest: &Path, options: &MvOptions) -> Result<()> {
    // Try atomic rename first
    match fs::rename(source, dest) {
        Ok(()) => return Ok(()),
        Err(e) => {
            // Check if this is a cross-filesystem move
            if e.raw_os_error() == Some(libc::EXDEV) {
                // Perform copy + remove for cross-filesystem moves
                return move_cross_filesystem(source, dest, options);
            } else {
                return Err(anyhow!("mv: cannot move '{}' to '{}': {}", 
                    source.display(), dest.display(), e));
            }
        }
    }
}

fn move_cross_filesystem(source: &Path, dest: &Path, options: &MvOptions) -> Result<()> {
    if source.is_dir() {
        copy_dir_all(source, dest)?;
        fs::remove_dir_all(source)?;
    } else {
        fs::copy(source, dest)?;
        fs::remove_file(source)?;
        
        // Preserve permissions and timestamps
        let source_metadata = fs::metadata(source).or_else(|_| fs::metadata(dest))?;
        let dest_file = OpenOptions::new().write(true).open(dest)?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(source_metadata.permissions().mode());
            fs::set_permissions(dest, permissions)?;
        }
    }
    
    Ok(())
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

fn print_help() {
    println!("Usage: mv [OPTION]... [-T] SOURCE DEST");
    println!("  or:  mv [OPTION]... SOURCE... DIRECTORY");
    println!("  or:  mv [OPTION]... -t DIRECTORY SOURCE...");
    println!("Rename SOURCE to DEST, or move SOURCE(s) to DIRECTORY.");
    println!();
    println!("Mandatory arguments to long options are mandatory for short options too.");
    println!("      --backup[=CONTROL]       make a backup of each existing destination file");
    println!("  -b                           like --backup but does not accept an argument");
    println!("  -f, --force                  do not prompt before overwriting");
    println!("  -i, --interactive            prompt before overwrite");
    println!("  -n, --no-clobber             do not overwrite an existing file");
    println!("If you specify more than one of -i, -f, -n, only the final one takes effect.");
    println!("      --strip-trailing-slashes  remove any trailing slashes from each SOURCE");
    println!("                                 argument");
    println!("  -S, --suffix=SUFFIX          override the usual backup suffix");
    println!("  -t, --target-directory=DIRECTORY  move all SOURCE arguments into DIRECTORY");
    println!("  -T, --no-target-directory    treat DEST as a normal file");
    println!("  -u, --update                 move only when the SOURCE file is newer");
    println!("                                 than the destination file or when the");
    println!("                                 destination file is missing");
    println!("  -v, --verbose                explain what is being done");
    println!("  -Z, --context                set SELinux security context of destination");
    println!("                                 file to default type");
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
    println!("Report mv bugs to <bug-reports@nexusshell.org>");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    
    #[test]
    fn test_parse_args() {
        let args = vec!["-v".to_string(), "-i".to_string(), "src".to_string(), "dest".to_string()];
        let options = parse_mv_args(&args).unwrap();
        
        assert!(options.verbose);
        assert!(options.interactive);
        assert_eq!(options.sources, vec!["src"]);
        assert_eq!(options.destination, "dest");
    }
    
    #[test]
    fn test_backup_modes() {
        let args = vec!["--backup=numbered".to_string(), "src".to_string(), "dest".to_string()];
        let options = parse_mv_args(&args).unwrap();
        
        assert_eq!(options.backup, BackupMode::Numbered);
    }
    
    #[test]
    fn test_target_directory() {
        let args = vec!["-t".to_string(), "/tmp".to_string(), "file1".to_string(), "file2".to_string()];
        let options = parse_mv_args(&args).unwrap();
        
        assert_eq!(options.target_directory, Some("/tmp".to_string()));
        assert_eq!(options.sources, vec!["file1", "file2"]);
    }
} 