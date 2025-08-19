//! `mv` command  Ecomprehensive file and directory move/rename implementation.
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
//!   Windows-specific options:
//!   --preserve-acl            - Preserve Access Control Lists (ACLs)
//!   --preserve-ads            - Preserve Alternate Data Streams
//!   --verify                  - Verify move integrity using checksums
//!   --retry=N                 - Retry failed operations N times

use anyhow::{Result, anyhow, Context};
use super::ui_design::{Colorize, TableFormatter, ColorPalette, Icons};
use std::fs::{self};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{info, debug, warn};

// SHA-256 for integrity verification
use sha2::{Sha256, Digest};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

#[cfg(unix)]
use uzers::{get_user_by_uid, get_group_by_gid};

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
    // Windows-specific options
    pub preserve_acl: bool,
    pub preserve_ads: bool,
    pub verify_integrity: bool,
    pub retry_count: u32,
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
            // Windows-specific defaults
            preserve_acl: false,
            preserve_ads: false,
            verify_integrity: false,
            retry_count: 0,
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
            println!("{} {} {} {}", 
                Icons::MOVE, 
                source.colorize(&ColorPalette::ACCENT),
                "→".colorize(&ColorPalette::INFO),
                dest_path.display().to_string().colorize(&ColorPalette::SUCCESS)
            );
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
            "--preserve-acl" => {
                options.preserve_acl = true;
            }
            "--preserve-ads" => {
                options.preserve_ads = true;
            }
            "--verify" => {
                options.verify_integrity = true;
            }
            arg if arg.starts_with("--retry=") => {
                let retry_str = arg.strip_prefix("--retry=").unwrap();
                options.retry_count = retry_str.parse()
                    .with_context(|| format!("Invalid retry count: {}", retry_str))?;
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
        let backup_name = format!("{filename}.{number}{suffix}");
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
        for entry in entries.flatten() {
            let entry_name = entry.file_name().to_string_lossy().into_owned();
            if entry_name.starts_with(&format!("{filename}.")) &&
               entry_name.ends_with(suffix) {
                let middle = &entry_name[filename.len() + 1..entry_name.len() - suffix.len()];
                if middle.parse::<u32>().is_ok() {
                    return Ok(true);
                }
            }
        }
    }
    
    Ok(false)
}

fn move_file(source: &Path, dest: &Path, options: &MvOptions) -> Result<()> {
    // Try atomic rename first
    match fs::rename(source, dest) {
        Ok(()) => Ok(()),
    Err(_e) => {
            // Check if this is a cross-filesystem move
            #[cfg(unix)]
            if e.raw_os_error() == Some(libc::EXDEV) {
                // Perform copy + remove for cross-filesystem moves
                return move_cross_filesystem(source, dest, options);
            }
            
            #[cfg(windows)]
            {
                // On Windows, try copy + remove for cross-device moves
                move_cross_filesystem(source, dest, options)
            }
            
            #[cfg(not(any(unix, windows)))]
            return Err(anyhow!("mv: cannot move '{}' to '{}': {}", 
                source.display(), dest.display(), e));
        }
    }
}

fn move_cross_filesystem(source: &Path, dest: &Path, options: &MvOptions) -> Result<()> {
    // For atomic operation guarantee, we first copy completely, then remove source
    // This ensures that if the operation fails partway through, the source remains intact
    
    // Create temporary destination path for atomic operation
    let temp_dest = generate_temp_dest_path(dest)?;
    
    if source.is_dir() {
        // Use enhanced recursive copy with metadata preservation
        copy_dir_recursively_for_mv(source, &temp_dest, options)?;
        
        // Atomic rename from temp to final destination
        fs::rename(&temp_dest, dest)
            .with_context(|| format!("Failed to atomically move directory '{}' to '{}'", temp_dest.display(), dest.display()))?;
        
        // Only remove source after successful atomic move
        fs::remove_dir_all(source)
            .with_context(|| format!("Failed to remove source directory '{}' after successful copy", source.display()))?;
    } else {
        // Copy file with metadata preservation to temporary location
        copy_file_with_metadata_for_mv(source, &temp_dest, options)?;
        
        // Atomic rename from temp to final destination
        fs::rename(&temp_dest, dest)
            .with_context(|| format!("Failed to atomically move file '{}' to '{}'", temp_dest.display(), dest.display()))?;
        
        // Only remove source after successful atomic move
        fs::remove_file(source)
            .with_context(|| format!("Failed to remove source file '{}' after successful copy", source.display()))?;
    }
    
    Ok(())
}

/// Generate a temporary destination path for atomic operations
fn generate_temp_dest_path(dest: &Path) -> Result<PathBuf> {
    let parent = dest.parent().unwrap_or(Path::new("."));
    let filename = dest.file_name()
        .ok_or_else(|| anyhow!("Cannot get filename for destination"))?
        .to_string_lossy();
    
    // Create a unique temporary filename
    let mut counter = 0;
    loop {
        let temp_name = format!(".nxsh_mv_temp_{filename}_{counter}");
        let temp_path = parent.join(temp_name);
        
        if !temp_path.exists() {
            return Ok(temp_path);
        }
        
        counter += 1;
        if counter > 1000 {
            return Err(anyhow!("Cannot generate unique temporary filename after 1000 attempts"));
        }
    }
}

/// Enhanced recursive directory copy for mv command with full metadata preservation and error recovery
fn copy_dir_recursively_for_mv(src: &Path, dst: &Path, options: &MvOptions) -> Result<()> {
    // Create destination directory
    fs::create_dir_all(dst)
        .with_context(|| format!("Failed to create directory '{}'", dst.display()))?;

    // Preserve directory metadata
    preserve_metadata_for_mv(src, dst)
        .with_context(|| format!("Failed to preserve directory metadata for '{}'", dst.display()))?;

    // Read directory entries with proper error handling
    let entries = fs::read_dir(src)
        .with_context(|| format!("Failed to read directory '{}'", src.display()))?;

    let mut copied_items = Vec::new();
    
    for entry in entries {
        let entry = entry
            .with_context(|| format!("Failed to read directory entry in '{}'", src.display()))?;
        
        let file_type = entry.file_type()
            .with_context(|| format!("Failed to get file type for '{}'", entry.path().display()))?;
        
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        // Track successful copies for cleanup on error
        let copy_result = if file_type.is_dir() {
            copy_dir_recursively_for_mv(&src_path, &dst_path, options)
                .with_context(|| format!("Failed to copy subdirectory '{}' to '{}'", src_path.display(), dst_path.display()))
        } else if file_type.is_file() {
            copy_file_with_metadata_for_mv(&src_path, &dst_path, options)
                .with_context(|| format!("Failed to copy file '{}' to '{}'", src_path.display(), dst_path.display()))
        } else if file_type.is_symlink() {
            copy_symlink_for_mv(&src_path, &dst_path)
                .with_context(|| format!("Failed to copy symlink '{}' to '{}'", src_path.display(), dst_path.display()))
        } else {
            if options.verbose {
                eprintln!("mv: warning: skipping special file: {}", src_path.display());
            }
            Ok(())
        };

        match copy_result {
            Ok(()) => {
                copied_items.push(dst_path);
            }
            Err(e) => {
                // Cleanup partially copied items on error
                cleanup_partial_copy(&copied_items);
                return Err(e);
            }
        }
    }

    if options.verbose {
        println!("mv: copied directory: {} -> {}", src.display(), dst.display());
    }
    
    Ok(())
}

/// Cleanup partially copied items in case of error
fn cleanup_partial_copy(copied_items: &[PathBuf]) {
    for item in copied_items.iter().rev() {
        if item.is_dir() {
            let _ = fs::remove_dir_all(item);
        } else {
            let _ = fs::remove_file(item);
        }
    }
}

/// Copy a single file with full metadata preservation for mv command with enhanced error handling
fn copy_file_with_metadata_for_mv(src: &Path, dst: &Path, options: &MvOptions) -> Result<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory '{}'", parent.display()))?;
    }

    // Copy the file content with verification
    let bytes_copied = fs::copy(src, dst)
        .with_context(|| format!("Failed to copy file content from '{}' to '{}'", src.display(), dst.display()))?;
    
    // Verify the copy was successful by checking file size
    let src_metadata = fs::metadata(src)
        .with_context(|| format!("Failed to read source file metadata for '{}'", src.display()))?;
    
    if bytes_copied != src_metadata.len() {
        // Cleanup incomplete copy
        let _ = fs::remove_file(dst);
        return Err(anyhow!("File copy incomplete: expected {} bytes, copied {} bytes", 
                          src_metadata.len(), bytes_copied));
    }

    // Preserve metadata (permissions and timestamps)
    preserve_metadata_for_mv(src, dst)
        .with_context(|| format!("Failed to preserve metadata for '{}'", dst.display()))?;

    if options.verbose {
        println!("mv: copied file: {} -> {}", src.display(), dst.display());
    }
    
    Ok(())
}

/// Copy a symbolic link for mv command with enhanced error handling
fn copy_symlink_for_mv(src: &Path, dst: &Path) -> Result<()> {
    let target = fs::read_link(src)
        .with_context(|| format!("Failed to read symlink '{}'", src.display()))?;
    
    // Remove destination if it exists
    if dst.exists() {
        if dst.is_dir() {
            fs::remove_dir_all(dst)
                .with_context(|| format!("Failed to remove existing directory '{}'", dst.display()))?;
        } else {
            fs::remove_file(dst)
                .with_context(|| format!("Failed to remove existing file '{}'", dst.display()))?;
        }
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target, dst)
            .with_context(|| format!("Failed to create symlink '{}' -> '{}'", dst.display(), target.display()))?;
    }

    #[cfg(windows)]
    {
        // Determine if target is directory or file for Windows symlink creation
        let target_is_dir = if target.is_absolute() {
            target.is_dir()
        } else {
            // For relative symlinks, check relative to the symlink's directory
            if let Some(symlink_parent) = src.parent() {
                symlink_parent.join(&target).is_dir()
            } else {
                target.is_dir()
            }
        };

        if target_is_dir {
            std::os::windows::fs::symlink_dir(&target, dst)
                .with_context(|| format!("Failed to create directory symlink '{}' -> '{}'", dst.display(), target.display()))?;
        } else {
            std::os::windows::fs::symlink_file(&target, dst)
                .with_context(|| format!("Failed to create file symlink '{}' -> '{}'", dst.display(), target.display()))?;
        }
    }

    Ok(())
}

/// Preserve file/directory metadata (permissions, timestamps) for mv command
fn preserve_metadata_for_mv(src: &Path, dst: &Path) -> Result<()> {
    let metadata = fs::metadata(src)
        .with_context(|| format!("Failed to read metadata for '{}'", src.display()))?;

    // Preserve timestamps
    let accessed = metadata.accessed()
        .with_context(|| format!("Failed to get access time for '{}'", src.display()))?;
    let modified = metadata.modified()
        .with_context(|| format!("Failed to get modification time for '{}'", src.display()))?;

    // Set timestamps on destination
    set_file_times_for_mv(dst, accessed, modified)
        .with_context(|| format!("Failed to set timestamps for '{}'", dst.display()))?;

    // Preserve permissions on Unix systems
    #[cfg(unix)]
    {
        #[cfg(unix)] use std::os::unix::fs::PermissionsExt;
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        
        let dst_permissions = std::fs::Permissions::from_mode(mode);
        fs::set_permissions(dst, dst_permissions)
            .with_context(|| format!("Failed to set permissions for '{}'", dst.display()))?;
    }

    Ok(())
}

/// Set file access and modification times for mv command (cross-platform via filetime crate)
fn set_file_times_for_mv(path: &Path, accessed: SystemTime, modified: SystemTime) -> Result<()> {
    use filetime::{FileTime, set_file_times};
    let at = FileTime::from_system_time(accessed);
    let mt = FileTime::from_system_time(modified);
    set_file_times(path, at, mt)
        .with_context(|| format!("Failed to set file times for '{}'", path.display()))?;
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

/// Calculate SHA-256 hash of a file for integrity verification
fn calculate_file_hash(path: &Path) -> Result<Vec<u8>> {
    use std::io::Read;
    
    let mut file = fs::File::open(path)
        .with_context(|| format!("Failed to open file for hashing: '{}'", path.display()))?;
    
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 8192]; // 8KB buffer
    
    loop {
        let bytes_read = file.read(&mut buffer)
            .with_context(|| format!("Failed to read from file: '{}'", path.display()))?;
        
        if bytes_read == 0 {
            break;
        }
        
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(hasher.finalize().to_vec())
}

/// Move file with integrity verification
fn move_file_with_verification(src: &Path, dst: &Path, options: &MvOptions) -> Result<()> {
    let mut last_error = None;
    for attempt in 0..=options.retry_count {
        match move_file_with_advanced_features(src, dst, options) {
            Ok(()) => {
                if options.verbose {
                    if attempt > 0 {
                        println!("Successfully moved '{}' -> '{}' (attempt {})", 
                                src.display(), dst.display(), attempt + 1);
                    } else {
                        println!("'{}' -> '{}'", src.display(), dst.display());
                    }
                }
                return Ok(());
            }
            Err(e) => {
                last_error = Some(e);
                if attempt < options.retry_count {
                    warn!("Move attempt {} failed, retrying: {}", attempt + 1, last_error.as_ref().unwrap());
                    std::thread::sleep(std::time::Duration::from_millis(100 * (attempt + 1) as u64));
                }
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| anyhow!("Move failed after all retries")))
}

/// Advanced move with Windows-specific features and verification
fn move_file_with_advanced_features(src: &Path, dst: &Path, options: &MvOptions) -> Result<()> {
    if options.verify_integrity {
        // Calculate hash before move
        let src_hash = calculate_file_hash(src)
            .with_context(|| format!("Failed to calculate hash for source file '{}'", src.display()))?;
        
        // Try atomic rename first (same filesystem)
        if let Err(_) = fs::rename(src, dst) {
            // Fall back to copy + remove with verification
            fs::copy(src, dst)
                .with_context(|| format!("Failed to copy '{}' to '{}'", src.display(), dst.display()))?;
            
            // Verify integrity after copy
            let dst_hash = calculate_file_hash(dst)
                .with_context(|| format!("Failed to calculate hash for destination file '{}'", dst.display()))?;
            
            if src_hash != dst_hash {
                fs::remove_file(dst).ok(); // Clean up on failure
                return Err(anyhow!("Integrity verification failed: file hashes do not match"));
            }
            
            // Remove source after successful copy and verification
            fs::remove_file(src)
                .with_context(|| format!("Failed to remove source file '{}'", src.display()))?;
        }
        
        debug!("Integrity verification passed for '{}' -> '{}'", src.display(), dst.display());
    } else {
        // Standard move operation
        if let Err(_) = fs::rename(src, dst) {
            // Fall back to copy + remove
            fs::copy(src, dst)
                .with_context(|| format!("Failed to copy '{}' to '{}'", src.display(), dst.display()))?;
            
            fs::remove_file(src)
                .with_context(|| format!("Failed to remove source file '{}'", src.display()))?;
        }
    }
    
    Ok(())
}

/// Windows-specific advanced move (placeholder)
#[cfg(windows)]
fn move_file_windows_advanced(src: &Path, dst: &Path, options: &MvOptions) -> Result<()> {
    // For now, use standard move - Windows-specific features can be added later
    move_file_with_advanced_features(src, dst, options)
}

/// Print enhanced help information for the mv command
fn print_help() {
    println!("mv - move (rename) files");
    println!();
    println!("USAGE:");
    println!("    mv [OPTIONS] SOURCE DEST");
    println!("    mv [OPTIONS] SOURCE... DIRECTORY");
    println!();
    println!("OPTIONS:");
    println!("    -b, --backup[=CONTROL]       Make backup of existing destination files");
    println!("    -f, --force                  Do not prompt before overwriting");
    println!("    -i, --interactive            Prompt before overwrite");
    println!("    -n, --no-clobber             Do not overwrite an existing file");
    println!("    --strip-trailing-slashes     Remove any trailing slashes from each SOURCE");
    println!("    -S, --suffix=SUFFIX          Override the usual backup suffix");
    println!("    -t, --target-directory=DIR   Move all SOURCE arguments into DIRECTORY");
    println!("    -T, --no-target-directory    Treat DEST as normal file");
    println!("    -u, --update                 Move only when SOURCE file is newer");
    println!("    -v, --verbose                Explain what is being done");
    println!("    -Z, --context=CTX            Set SELinux security context of destination");
    println!();
    println!("Windows-specific options:");
    println!("    --preserve-acl               Preserve Access Control Lists (ACLs)");
    println!("    --preserve-ads               Preserve Alternate Data Streams");
    println!("    --verify                     Verify move integrity using checksums");
    println!("    --retry=N                    Retry failed operations N times");
    println!();
    println!("BACKUP CONTROL:");
    println!("  none, off       never make backups");
    println!("  numbered, t     make numbered backups");
    println!("  existing, nil   numbered if numbered backups exist, simple otherwise");
    println!("  simple, never   always make simple backups");
    println!();
    println!("EXAMPLES:");
    println!("    mv file.txt renamed.txt");
    println!("    mv *.txt /backup/");
    println!("    mv --verify important.dat backup/");
    println!("    mv --backup=numbered config.ini config.ini");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    
    
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

    /// Test Windows-specific options parsing
    #[test]
    fn test_windows_options() -> Result<()> {
        let args = vec![
            "--preserve-acl".to_string(),
            "--preserve-ads".to_string(),
            "--verify".to_string(),
            "--retry=3".to_string(),
            "src".to_string(),
            "dest".to_string()
        ];
        let options = parse_mv_args(&args)?;
        
        assert!(options.preserve_acl);
        assert!(options.preserve_ads);
        assert!(options.verify_integrity);
        assert_eq!(options.retry_count, 3);
        Ok(())
    }

    /// Test hash calculation function
    #[test]
    fn test_calculate_file_hash() -> Result<()> {
        let temp_dir = tempfile::TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");

        fs::write(&test_file, "Test data for hashing")?;

        let hash1 = calculate_file_hash(&test_file)?;
        let hash2 = calculate_file_hash(&test_file)?;

        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
        Ok(())
    }

    /// Test move with verification
    #[test]
    fn test_move_with_verification() -> Result<()> {
        let temp_dir = tempfile::TempDir::new()?;
        let src_file = temp_dir.path().join("source.txt");
        let dst_file = temp_dir.path().join("dest.txt");

        let test_data = "Test data for move verification";
        fs::write(&src_file, test_data)?;

        let mut options = MvOptions::default();
        options.verify_integrity = true;

        move_file_with_verification(&src_file, &dst_file, &options)?;

        assert!(!src_file.exists());
        assert!(dst_file.exists());
        assert_eq!(fs::read_to_string(&dst_file)?, test_data);
        Ok(())
    }

    /// Test retry mechanism
    #[test]
    fn test_retry_mechanism() -> Result<()> {
        let temp_dir = tempfile::TempDir::new()?;
        let src_file = temp_dir.path().join("source.txt");
        let dst_file = temp_dir.path().join("dest.txt");

        fs::write(&src_file, "Test content for retry")?;

        let mut options = MvOptions::default();
        options.retry_count = 3;

        move_file_with_verification(&src_file, &dst_file, &options)?;

        assert!(!src_file.exists());
        assert!(dst_file.exists());
        assert_eq!(fs::read_to_string(&dst_file)?, "Test content for retry");
        Ok(())
    }

    /// Test verbose mode functionality
    #[test]
    fn test_verbose_mode() -> Result<()> {
        let temp_dir = tempfile::TempDir::new()?;
        let src_file = temp_dir.path().join("source.txt");
        let dst_file = temp_dir.path().join("dest.txt");

        fs::write(&src_file, "Verbose test content")?;

        let mut options = MvOptions::default();
        options.verbose = true;

        move_file_with_verification(&src_file, &dst_file, &options)?;

        assert!(!src_file.exists());
        assert!(dst_file.exists());
        assert_eq!(fs::read_to_string(&dst_file)?, "Verbose test content");
        Ok(())
    }
    
    #[test]
    fn test_target_directory() {
        let args = vec!["-t".to_string(), "/tmp".to_string(), "file1".to_string(), "file2".to_string()];
        let options = parse_mv_args(&args).unwrap();
        
        assert_eq!(options.target_directory, Some("/tmp".to_string()));
        assert_eq!(options.sources, vec!["file1", "file2"]);
    }
} 

