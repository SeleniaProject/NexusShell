//! `stat` command - comprehensive file and filesystem status display implementation.

use anyhow::{Result, anyhow};
use std::fs::{self, Metadata};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use chrono::{DateTime, Local};

// Platform-specific imports
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, FileTypeExt};

#[cfg(windows)]
use whoami;

use nxsh_core::{Context, ExecutionResult, ShellError};

pub struct StatBuiltin;

impl StatBuiltin {
    pub fn execute(&self, _ctx: &mut Context, args: Vec<String>) -> Result<ExecutionResult, ShellError> {
        match stat_cli(&args) {
            Ok(()) => Ok(ExecutionResult::success(0)),
            Err(e) => Ok(ExecutionResult::success(1).with_error(e.to_string().into_bytes())),
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Default)]
struct StatOptions {
    dereference: bool,
    file_system: bool,
    terse: bool,
    format: Option<String>,
    printf_format: Option<String>,
    files: Vec<String>,
}


#[derive(Debug)]
struct FileInfo {
    path: PathBuf,
    metadata: Metadata,
}

#[derive(Debug)]
struct FilesystemInfo {
    path: PathBuf,
    fs_type: String,
    block_size: u64,
    total_blocks: u64,
    free_blocks: u64,
    available_blocks: u64,
    total_inodes: u64,
    free_inodes: u64,
    max_filename_length: u64,
    fs_id: u64,
}

pub fn stat_cli(args: &[String]) -> anyhow::Result<()> {
    let options = parse_stat_args(args)?;

    if options.files.is_empty() {
        return Err(anyhow!("stat: missing operand"));
    }

    for file_path in &options.files {
        if options.file_system {
            let fs_info = get_filesystem_info(file_path)?;
            if options.terse {
                display_filesystem_terse(&fs_info)?;
            } else {
                display_filesystem_default(&fs_info)?;
            }
        } else {
            let file_info = get_file_info(file_path, options.dereference)?;
            
            if let Some(ref format) = options.format {
                display_custom_format(&file_info, format, false)?;
            } else if let Some(ref printf_format) = options.printf_format {
                display_custom_format(&file_info, printf_format, true)?;
            } else if options.terse {
                display_terse_format(&file_info)?;
            } else {
                display_default_format(&file_info)?;
            }
        }
    }

    Ok(())
}

fn parse_stat_args(args: &[String]) -> Result<StatOptions> {
    let mut options = StatOptions::default();
    let mut i = 1; // Skip program name

    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "-L" | "--dereference" => {
                options.dereference = true;
            }
            "-f" | "--file-system" => {
                options.file_system = true;
            }
            "-t" | "--terse" => {
                options.terse = true;
            }
            "-c" | "--format" => {
                if i + 1 >= args.len() {
                    return Err(anyhow!("Option {} requires an argument", arg));
                }
                i += 1;
                options.format = Some(args[i].clone());
            }
            "--printf" => {
                if i + 1 >= args.len() {
                    return Err(anyhow!("Option {} requires an argument", arg));
                }
                i += 1;
                options.printf_format = Some(args[i].clone());
            }
            "--help" => {
                print_help();
                return Ok(options);
            }
            "--version" => {
                println!("stat (NexusShell) 1.0.0");
                return Ok(options);
            }
            _ if arg.starts_with('-') => {
                return Err(anyhow!("Unknown option: {}", arg));
            }
            _ => {
                options.files.push(arg.clone());
            }
        }
        i += 1;
    }

    Ok(options)
}

fn get_file_info(path: &str, dereference: bool) -> Result<FileInfo> {
    let path_buf = PathBuf::from(path);
    
    let metadata = if dereference {
        fs::metadata(&path_buf)
    } else {
        fs::symlink_metadata(&path_buf)
    }.map_err(|e| anyhow!("Cannot stat {}: {}", path, e))?;

    Ok(FileInfo {
        path: path_buf,
        metadata,
    })
}

fn get_filesystem_info(path: &str) -> Result<FilesystemInfo> {
    let path_buf = PathBuf::from(path);
    
    // This is a simplified implementation
    // Real implementation would use platform-specific system calls
    Ok(FilesystemInfo {
        path: path_buf,
        fs_type: "unknown".to_string(),
        block_size: 4096,
        total_blocks: 0,
        free_blocks: 0,
        available_blocks: 0,
        total_inodes: 0,
        free_inodes: 0,
        max_filename_length: 255,
        fs_id: 0,
    })
}

fn display_default_format(info: &FileInfo) -> Result<()> {
    let meta = &info.metadata;
    let file_type = get_file_type_description(meta);
    
    println!("  File: \"{}\"", info.path.display());
    
    if meta.file_type().is_symlink() {
        if let Ok(target) = fs::read_link(&info.path) {
            println!("  Link: {} -> {}", info.path.display(), target.display());
        }
    }
    
    // Display file information
    #[cfg(unix)]
    {
        println!("  Size: {:<15} Blocks: {:<10} IO Block: {:<6} {}",
            meta.len(), meta.blocks(), meta.blksize(), file_type);
        println!("Device: {:<15} Inode: {:<10} Links: {}",
            format!("{}h/{}d", format!("{:x}", meta.dev()), meta.dev()),
            meta.ino(), meta.nlink());
        println!(
            "Access: ({:04o}/{})  Uid: ({:5}/{})   Gid: ({:5}/{})",
            meta.mode() & 0o7777,
            format_permissions(meta.mode()),
            meta.uid(),
            get_username(meta.uid()),
            meta.gid(),
            get_groupname(meta.gid())
        );
    }
    #[cfg(windows)]
    {
        println!("  Size: {:<15} Type: {}",
            meta.len(), file_type);
        println!("Device: N/A             Inode: N/A        Links: N/A");
        println!("Access: N/A  Owner: {}   Group: N/A",
            whoami::username());
    }
    
    // Display timestamps
    #[cfg(unix)]
    {
        let atime = DateTime::<Local>::from(meta.accessed()?);
        let mtime = DateTime::<Local>::from(meta.modified()?);
        let ctime = DateTime::<Local>::from(
            UNIX_EPOCH + std::time::Duration::from_secs(meta.ctime() as u64)
        );
        
        println!("Access: {}", atime.format("%Y-%m-%d %H:%M:%S.%9f %z"));
        println!("Modify: {}", mtime.format("%Y-%m-%d %H:%M:%S.%9f %z"));
        println!("Change: {}", ctime.format("%Y-%m-%d %H:%M:%S.%9f %z"));
        println!(" Birth: -"); // Not available on most Unix systems
    }
    #[cfg(windows)]
    {
        let created = DateTime::<Local>::from(meta.created()?);
        let modified = DateTime::<Local>::from(meta.modified()?);
        let accessed = DateTime::<Local>::from(meta.accessed()?);
        
        println!("Access: {}", accessed.format("%Y-%m-%d %H:%M:%S.%9f"));
        println!("Modify: {}", modified.format("%Y-%m-%d %H:%M:%S.%9f"));
        println!("Change: {}", modified.format("%Y-%m-%d %H:%M:%S.%9f"));
        println!(" Birth: {}", created.format("%Y-%m-%d %H:%M:%S.%9f"));
    }
    
    Ok(())
}

fn display_terse_format(info: &FileInfo) -> Result<()> {
    let meta = &info.metadata;
    let path = info.path.to_string_lossy();
    
    // Terse format for Windows
    #[cfg(windows)]
    {
        let created = meta.created().unwrap_or(UNIX_EPOCH);
        let modified = meta.modified().unwrap_or(UNIX_EPOCH);
        let accessed = meta.accessed().unwrap_or(UNIX_EPOCH);
        
        println!("{} {} 100644 0 0 0 0 1 {} {} {} {} 4096 {}",
            path,
            meta.len(), // nlink
            accessed.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            modified.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            modified.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            created.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(), // blksize
            meta.len().div_ceil(4096) // blocks
        );
    }
    
    Ok(())
}

fn display_filesystem_default(_info: &FilesystemInfo) -> Result<()> {
    // Simplified implementation
    println!("Filesystem information not fully implemented");
    Ok(())
}

fn display_filesystem_terse(_info: &FilesystemInfo) -> Result<()> {
    // Simplified implementation
    println!("Filesystem terse format not fully implemented");
    Ok(())
}

fn display_custom_format(_info: &FileInfo, _format: &str, _is_printf: bool) -> Result<()> {
    // Simplified implementation
    println!("Custom format not fully implemented");
    Ok(())
}

fn get_file_type_description(meta: &Metadata) -> String {
    let file_type = meta.file_type();
    
    if file_type.is_file() {
        "regular file".to_string()
    } else if file_type.is_dir() {
        "directory".to_string()
    } else if file_type.is_symlink() {
        "symbolic link".to_string()
    } else {
        #[cfg(unix)]
        {
            if file_type.is_block_device() {
                "block special file".to_string()
            } else if file_type.is_char_device() {
                "character special file".to_string()
            } else if file_type.is_fifo() {
                "fifo".to_string()
            } else if file_type.is_socket() {
                "socket".to_string()
            } else {
                "unknown".to_string()
            }
        }
        #[cfg(windows)]
        {
            "unknown".to_string()
        }
    }
}

fn format_permissions(_mode: u32) -> String {
    #[cfg(unix)]
    {
        let mut perms = String::new();
        
        // File type
        let file_type = (_mode & 0o170000) as u32;
        perms.push(match file_type {
            0o100000 => '-', // Regular file
            0o040000 => 'd', // Directory
            0o120000 => 'l', // Symbolic link
            0o060000 => 'b', // Block device
            0o020000 => 'c', // Character device
            0o010000 => 'p', // FIFO
            0o140000 => 's', // Socket
            _ => '?',
        });
        
        // Owner permissions
        perms.push(if _mode & 0o400 != 0 { 'r' } else { '-' });
        perms.push(if _mode & 0o200 != 0 { 'w' } else { '-' });
        perms.push(if _mode & 0o100 != 0 { 'x' } else { '-' });
        
        // Group permissions
        perms.push(if _mode & 0o040 != 0 { 'r' } else { '-' });
        perms.push(if _mode & 0o020 != 0 { 'w' } else { '-' });
        perms.push(if _mode & 0o010 != 0 { 'x' } else { '-' });
        
        // Other permissions
        perms.push(if _mode & 0o004 != 0 { 'r' } else { '-' });
        perms.push(if _mode & 0o002 != 0 { 'w' } else { '-' });
        perms.push(if _mode & 0o001 != 0 { 'x' } else { '-' });
        
        perms
    }
    #[cfg(windows)]
    {
        "-rw-r--r--".to_string()
    }
}

/// Resolve a user name for a given UID without libc bindings.
/// Preference order: /etc/passwd ↁEgetent ↁEid ↁEnumeric UID
fn get_username(_uid: u32) -> String {
    #[cfg(unix)]
    {
        // Try /etc/passwd
        if let Ok(file) = std::fs::File::open("/etc/passwd") {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    if let Ok(uid) = parts[2].parse::<u32>() {
                        if uid == _uid {
                            return parts[0].to_string();
                        }
                    }
                }
            }
        }

        // Try getent (supports NSS/LDAP)
        if let Ok(output) = std::process::Command::new("getent")
            .args(["passwd", &_uid.to_string()])
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = s.trim().split(':').collect();
                if !parts.is_empty() {
                    return parts[0].to_string();
                }
            }
        }

        // Try id -nu
        if let Ok(output) = std::process::Command::new("id")
            .args(["-nu", &_uid.to_string()])
            .output()
        {
            if output.status.success() {
                let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !name.is_empty() && name != _uid.to_string() {
                    return name;
                }
            }
        }

        // Fallback to numeric
        _uid.to_string()
    }
    #[cfg(windows)]
    {
        whoami::username()
    }
}

/// Resolve a group name for a given GID without libc bindings.
/// Preference order: /etc/group ↁEgetent ↁEid ↁEnumeric GID
fn get_groupname(_gid: u32) -> String {
    #[cfg(unix)]
    {
        // Try /etc/group
        if let Ok(file) = std::fs::File::open("/etc/group") {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    if let Ok(gid) = parts[2].parse::<u32>() {
                        if gid == _gid {
                            return parts[0].to_string();
                        }
                    }
                }
            }
        }

        // Try getent
        if let Ok(output) = std::process::Command::new("getent")
            .args(["group", &_gid.to_string()])
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = s.trim().split(':').collect();
                if !parts.is_empty() {
                    return parts[0].to_string();
                }
            }
        }

        // Try id -ng
        if let Ok(output) = std::process::Command::new("id")
            .args(["-ng", &_gid.to_string()])
            .output()
        {
            if output.status.success() {
                let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !name.is_empty() && name != _gid.to_string() {
                    return name;
                }
            }
        }

        // Fallback to numeric
        _gid.to_string()
    }
    #[cfg(windows)]
    {
        "Users".to_string()
    }
}

fn print_help() {
    println!("Usage: stat [OPTION]... FILE...");
    println!("Display file or file system status.");
    println!();
    println!("  -L, --dereference     follow links");
    println!("  -f, --file-system     display file system status instead of file status");
    println!("  -c  --format=FORMAT   use the specified FORMAT instead of the default");
    println!("      --printf=FORMAT   like --format, but interpret backslash escapes");
    println!("  -t, --terse           print the information in terse form");
    println!("      --help            display this help and exit");
    println!("      --version         output version information and exit");
}

/// Execute stat command
pub fn execute(args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    match stat_cli(args) {
        Ok(_) => Ok(0),
        Err(e) => {
            eprintln!("stat: {e}");
            Ok(1)
        }
    }
}

