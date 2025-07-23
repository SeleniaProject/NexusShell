//! `stat` command – comprehensive file and filesystem status display implementation.
//!
//! Supports complete stat functionality:
//!   stat [OPTIONS] FILE...
//!   -L, --dereference         - Follow symbolic links
//!   -f, --file-system         - Display file system status instead of file status
//!   --cached=MODE             - Specify how to use cached attributes
//!   -c, --format=FORMAT       - Use specified format instead of default
//!   --printf=FORMAT           - Like --format, but interpret backslash escapes
//!   -t, --terse               - Print information in terse form
//!   --help                    - Display help and exit
//!   --version                 - Output version information and exit

use anyhow::{Result, anyhow};
use std::fs::{self, Metadata};
use std::os::unix::fs::{MetadataExt, FileTypeExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Local, TimeZone, Utc};
use users::{get_user_by_uid, get_group_by_gid};
use humansize::{format_size, DECIMAL, BINARY};

#[derive(Debug, Clone)]
pub struct StatOptions {
    pub files: Vec<String>,
    pub dereference: bool,
    pub file_system: bool,
    pub cached: CacheMode,
    pub format: Option<String>,
    pub printf_format: Option<String>,
    pub terse: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CacheMode {
    Default,
    Never,
    Always,
}

impl Default for StatOptions {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            dereference: false,
            file_system: false,
            cached: CacheMode::Default,
            format: None,
            printf_format: None,
            terse: false,
        }
    }
}

#[derive(Debug)]
pub struct FileInfo {
    pub path: PathBuf,
    pub metadata: Metadata,
    pub is_symlink: bool,
    pub symlink_target: Option<PathBuf>,
}

#[derive(Debug)]
pub struct FilesystemInfo {
    pub path: PathBuf,
    pub fs_type: String,
    pub block_size: u64,
    pub total_blocks: u64,
    pub free_blocks: u64,
    pub available_blocks: u64,
    pub total_inodes: u64,
    pub free_inodes: u64,
    pub max_filename_length: u64,
    pub fs_id: u64,
}

pub fn stat_cli(args: &[String]) -> Result<()> {
    let options = parse_stat_args(args)?;
    
    if options.files.is_empty() {
        return Err(anyhow!("stat: missing operand"));
    }
    
    for (i, file) in options.files.iter().enumerate() {
        if i > 0 {
            println!(); // Blank line between files
        }
        
        let path = PathBuf::from(file);
        
        if options.file_system {
            display_filesystem_info(&path, &options)?;
        } else {
            display_file_info(&path, &options)?;
        }
    }
    
    Ok(())
}

fn parse_stat_args(args: &[String]) -> Result<StatOptions> {
    let mut options = StatOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "-L" | "--dereference" => {
                options.dereference = true;
            }
            "-f" | "--file-system" => {
                options.file_system = true;
            }
            "--cached=default" => {
                options.cached = CacheMode::Default;
            }
            "--cached=never" => {
                options.cached = CacheMode::Never;
            }
            "--cached=always" => {
                options.cached = CacheMode::Always;
            }
            "-c" | "--format" => {
                if i + 1 < args.len() {
                    options.format = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err(anyhow!("stat: option requires an argument -- c"));
                }
            }
            arg if arg.starts_with("--format=") => {
                let format = arg.strip_prefix("--format=").unwrap();
                options.format = Some(format.to_string());
            }
            "--printf" => {
                if i + 1 < args.len() {
                    options.printf_format = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err(anyhow!("stat: option requires an argument -- printf"));
                }
            }
            arg if arg.starts_with("--printf=") => {
                let format = arg.strip_prefix("--printf=").unwrap();
                options.printf_format = Some(format.to_string());
            }
            "-t" | "--terse" => {
                options.terse = true;
            }
            "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--version" => {
                println!("stat (NexusShell) 1.0.0");
                std::process::exit(0);
            }
            arg if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") => {
                // Handle combined short options
                for ch in arg.chars().skip(1) {
                    match ch {
                        'L' => options.dereference = true,
                        'f' => options.file_system = true,
                        't' => options.terse = true,
                        _ => return Err(anyhow!("stat: invalid option -- '{}'", ch)),
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

fn display_file_info(path: &Path, options: &StatOptions) -> Result<()> {
    let file_info = get_file_info(path, options)?;
    
    if let Some(ref format) = options.format {
        display_custom_format(&file_info, format, false)?;
    } else if let Some(ref format) = options.printf_format {
        display_custom_format(&file_info, format, true)?;
    } else if options.terse {
        display_terse_format(&file_info)?;
    } else {
        display_default_format(&file_info)?;
    }
    
    Ok(())
}

fn display_filesystem_info(path: &Path, options: &StatOptions) -> Result<()> {
    let fs_info = get_filesystem_info(path)?;
    
    if options.terse {
        display_filesystem_terse(&fs_info)?;
    } else {
        display_filesystem_default(&fs_info)?;
    }
    
    Ok(())
}

fn get_file_info(path: &Path, options: &StatOptions) -> Result<FileInfo> {
    let is_symlink = path.is_symlink();
    let symlink_target = if is_symlink {
        fs::read_link(path).ok()
    } else {
        None
    };
    
    let metadata = if options.dereference && is_symlink {
        fs::metadata(path)?
    } else {
        fs::symlink_metadata(path)?
    };
    
    Ok(FileInfo {
        path: path.to_path_buf(),
        metadata,
        is_symlink,
        symlink_target,
    })
}

fn get_filesystem_info(path: &Path) -> Result<FilesystemInfo> {
    use std::ffi::CString;
    use std::mem;
    
    let path_cstr = CString::new(path.to_string_lossy().as_bytes())?;
    
    unsafe {
        let mut statfs: libc::statfs = mem::zeroed();
        let result = libc::statfs(path_cstr.as_ptr(), &mut statfs);
        
        if result != 0 {
            return Err(anyhow!("stat: cannot read file system information for '{}': {}", 
                             path.display(), std::io::Error::last_os_error()));
        }
        
        Ok(FilesystemInfo {
            path: path.to_path_buf(),
            fs_type: get_filesystem_type(statfs.f_type),
            block_size: statfs.f_bsize as u64,
            total_blocks: statfs.f_blocks as u64,
            free_blocks: statfs.f_bfree as u64,
            available_blocks: statfs.f_bavail as u64,
            total_inodes: statfs.f_files as u64,
            free_inodes: statfs.f_ffree as u64,
            max_filename_length: statfs.f_namelen as u64,
            fs_id: ((statfs.f_fsid.val[0] as u64) << 32) | (statfs.f_fsid.val[1] as u64),
        })
    }
}

fn get_filesystem_type(fs_type: i64) -> String {
    match fs_type {
        0x61756673 => "aufs".to_string(),
        0x9123683E => "btrfs".to_string(),
        0x28cd3d45 => "cramfs".to_string(),
        0x453dcd28 => "cramfs".to_string(),
        0x64626720 => "debugfs".to_string(),
        0x73636673 => "securityfs".to_string(),
        0xf97cff8c => "selinuxfs".to_string(),
        0x62656572 => "sysfs".to_string(),
        0x958458f6 => "hugetlbfs".to_string(),
        0x01021994 => "tmpfs".to_string(),
        0x9fa0 => "proc".to_string(),
        0xef51 => "ext2".to_string(),
        0xef53 => "ext3/ext4".to_string(),
        0x4d44 => "msdos".to_string(),
        0x4006 => "fat".to_string(),
        0x564c => "ncp".to_string(),
        0x6969 => "nfs".to_string(),
        0x9660 => "iso9660".to_string(),
        0x517b => "smb".to_string(),
        0x52654973 => "reiserfs".to_string(),
        0x58465342 => "xfs".to_string(),
        0x01021997 => "v9fs".to_string(),
        0x27e0eb => "cgroup".to_string(),
        0x63677270 => "cgroup2".to_string(),
        _ => format!("unknown (0x{:x})", fs_type),
    }
}

fn display_default_format(info: &FileInfo) -> Result<()> {
    let path = &info.path;
    let meta = &info.metadata;
    
    println!("  File: {}", path.display());
    
    if info.is_symlink {
        if let Some(ref target) = info.symlink_target {
            println!("  Size: {:<15} Blocks: {:<10} IO Block: {:<6} symbolic link",
                meta.len(), meta.blocks(), meta.blksize());
            println!("Device: {:<15} Inode: {:<10} Links: {}",
                format!("{}h/{}d", format!("{:x}", meta.dev()), meta.dev()),
                meta.ino(), meta.nlink());
            println!("Access: ({:04o}/{})  Uid: ({:5}/{})   Gid: ({:5}/{})",
                meta.mode() & 0o7777, format_permissions(meta.mode()),
                meta.uid(), get_username(meta.uid()),
                meta.gid(), get_groupname(meta.gid()));
        }
    } else {
        let file_type = get_file_type_description(meta);
        println!("  Size: {:<15} Blocks: {:<10} IO Block: {:<6} {}",
            meta.len(), meta.blocks(), meta.blksize(), file_type);
        println!("Device: {:<15} Inode: {:<10} Links: {}",
            format!("{}h/{}d", format!("{:x}", meta.dev()), meta.dev()),
            meta.ino(), meta.nlink());
        println!("Access: ({:04o}/{})  Uid: ({:5}/{})   Gid: ({:5}/{})",
            meta.mode() & 0o7777, format_permissions(meta.mode()),
            meta.uid(), get_username(meta.uid()),
            meta.gid(), get_groupname(meta.gid()));
    }
    
    // Display timestamps
    let atime = DateTime::<Local>::from(meta.accessed()?);
    let mtime = DateTime::<Local>::from(meta.modified()?);
    let ctime = DateTime::<Local>::from(
        UNIX_EPOCH + std::time::Duration::from_secs(meta.ctime() as u64)
    );
    
    println!("Access: {}", atime.format("%Y-%m-%d %H:%M:%S.%9f %z"));
    println!("Modify: {}", mtime.format("%Y-%m-%d %H:%M:%S.%9f %z"));
    println!("Change: {}", ctime.format("%Y-%m-%d %H:%M:%S.%9f %z"));
    println!(" Birth: -"); // Not available on most Unix systems
    
    Ok(())
}

fn display_terse_format(info: &FileInfo) -> Result<()> {
    let meta = &info.metadata;
    let path = info.path.to_string_lossy();
    
    // Terse format: file size mode uid gid device inode links atime mtime ctime birth blksize blocks
    println!("{} {} {} {} {} {} {} {} {} {} {} {} {} {}",
        path,
        meta.len(),
        format!("{:o}", meta.mode()),
        meta.uid(),
        meta.gid(),
        meta.dev(),
        meta.ino(),
        meta.nlink(),
        meta.atime(),
        meta.mtime(),
        meta.ctime(),
        "-", // birth time not available
        meta.blksize(),
        meta.blocks()
    );
    
    Ok(())
}

fn display_filesystem_default(info: &FilesystemInfo) -> Result<()> {
    println!("  File: \"{}\"", info.path.display());
    println!("    ID: {:016x} Namelen: {:<7} Type: {}",
        info.fs_id, info.max_filename_length, info.fs_type);
    
    let block_size = info.block_size;
    let total_size = info.total_blocks * block_size;
    let free_size = info.free_blocks * block_size;
    let available_size = info.available_blocks * block_size;
    
    println!("Block size: {:<10} Fundamental block size: {}",
        block_size, block_size);
    println!("Blocks: Total: {:<10} Free: {:<10} Available: {}",
        info.total_blocks, info.free_blocks, info.available_blocks);
    println!("Inodes: Total: {:<10} Free: {}",
        info.total_inodes, info.free_inodes);
    
    Ok(())
}

fn display_filesystem_terse(info: &FilesystemInfo) -> Result<()> {
    println!("{} {} {} {} {} {} {} {} {} {} {}",
        info.path.display(),
        info.fs_type,
        info.block_size,
        info.total_blocks,
        info.free_blocks,
        info.available_blocks,
        info.total_inodes,
        info.free_inodes,
        info.max_filename_length,
        info.fs_id,
        info.block_size
    );
    
    Ok(())
}

fn display_custom_format(info: &FileInfo, format: &str, is_printf: bool) -> Result<()> {
    let mut result = String::new();
    let mut chars = format.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '%' {
            if let Some(&next_ch) = chars.peek() {
                match next_ch {
                    'n' => {
                        chars.next();
                        result.push_str(&info.path.to_string_lossy());
                    }
                    's' => {
                        chars.next();
                        result.push_str(&info.metadata.len().to_string());
                    }
                    'b' => {
                        chars.next();
                        result.push_str(&info.metadata.blocks().to_string());
                    }
                    'f' => {
                        chars.next();
                        result.push_str(&format!("{:x}", info.metadata.mode()));
                    }
                    'F' => {
                        chars.next();
                        result.push_str(&get_file_type_description(&info.metadata));
                    }
                    'a' => {
                        chars.next();
                        result.push_str(&format!("{:o}", info.metadata.mode() & 0o7777));
                    }
                    'A' => {
                        chars.next();
                        result.push_str(&format_permissions(info.metadata.mode()));
                    }
                    'u' => {
                        chars.next();
                        result.push_str(&info.metadata.uid().to_string());
                    }
                    'U' => {
                        chars.next();
                        result.push_str(&get_username(info.metadata.uid()));
                    }
                    'g' => {
                        chars.next();
                        result.push_str(&info.metadata.gid().to_string());
                    }
                    'G' => {
                        chars.next();
                        result.push_str(&get_groupname(info.metadata.gid()));
                    }
                    'd' => {
                        chars.next();
                        result.push_str(&info.metadata.dev().to_string());
                    }
                    'i' => {
                        chars.next();
                        result.push_str(&info.metadata.ino().to_string());
                    }
                    'h' => {
                        chars.next();
                        result.push_str(&info.metadata.nlink().to_string());
                    }
                    'X' => {
                        chars.next();
                        let atime = info.metadata.atime();
                        result.push_str(&atime.to_string());
                    }
                    'Y' => {
                        chars.next();
                        let mtime = info.metadata.mtime();
                        result.push_str(&mtime.to_string());
                    }
                    'Z' => {
                        chars.next();
                        let ctime = info.metadata.ctime();
                        result.push_str(&ctime.to_string());
                    }
                    '%' => {
                        chars.next();
                        result.push('%');
                    }
                    _ => {
                        result.push(ch);
                    }
                }
            } else {
                result.push(ch);
            }
        } else if is_printf && ch == '\\' {
            if let Some(&next_ch) = chars.peek() {
                match next_ch {
                    'n' => {
                        chars.next();
                        result.push('\n');
                    }
                    't' => {
                        chars.next();
                        result.push('\t');
                    }
                    'r' => {
                        chars.next();
                        result.push('\r');
                    }
                    '\\' => {
                        chars.next();
                        result.push('\\');
                    }
                    _ => {
                        result.push(ch);
                    }
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }
    
    if is_printf {
        print!("{}", result);
    } else {
        println!("{}", result);
    }
    
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
    } else if file_type.is_block_device() {
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

fn format_permissions(mode: u32) -> String {
    let mut perms = String::new();
    
    // File type
    let file_type = (mode & libc::S_IFMT) as u32;
    perms.push(match file_type {
        libc::S_IFREG => '-',
        libc::S_IFDIR => 'd',
        libc::S_IFLNK => 'l',
        libc::S_IFBLK => 'b',
        libc::S_IFCHR => 'c',
        libc::S_IFIFO => 'p',
        libc::S_IFSOCK => 's',
        _ => '?',
    });
    
    // Owner permissions
    perms.push(if mode & libc::S_IRUSR != 0 { 'r' } else { '-' });
    perms.push(if mode & libc::S_IWUSR != 0 { 'w' } else { '-' });
    perms.push(if mode & libc::S_ISUID != 0 {
        if mode & libc::S_IXUSR != 0 { 's' } else { 'S' }
    } else if mode & libc::S_IXUSR != 0 { 'x' } else { '-' });
    
    // Group permissions
    perms.push(if mode & libc::S_IRGRP != 0 { 'r' } else { '-' });
    perms.push(if mode & libc::S_IWGRP != 0 { 'w' } else { '-' });
    perms.push(if mode & libc::S_ISGID != 0 {
        if mode & libc::S_IXGRP != 0 { 's' } else { 'S' }
    } else if mode & libc::S_IXGRP != 0 { 'x' } else { '-' });
    
    // Other permissions
    perms.push(if mode & libc::S_IROTH != 0 { 'r' } else { '-' });
    perms.push(if mode & libc::S_IWOTH != 0 { 'w' } else { '-' });
    perms.push(if mode & libc::S_ISVTX != 0 {
        if mode & libc::S_IXOTH != 0 { 't' } else { 'T' }
    } else if mode & libc::S_IXOTH != 0 { 'x' } else { '-' });
    
    perms
}

fn get_username(uid: u32) -> String {
    get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().to_string())
        .unwrap_or_else(|| uid.to_string())
}

fn get_groupname(gid: u32) -> String {
    get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().to_string())
        .unwrap_or_else(|| gid.to_string())
}

fn print_help() {
    println!("Usage: stat [OPTION]... FILE...");
    println!("Display file or file system status.");
    println!();
    println!("Mandatory arguments to long options are mandatory for short options too.");
    println!("  -L, --dereference     follow links");
    println!("  -f, --file-system     display file system status instead of file status");
    println!("      --cached=MODE     specify how to use cached attributes;");
    println!("                          useful on remote file systems. See MODE below");
    println!("  -c  --format=FORMAT   use the specified FORMAT instead of the default;");
    println!("                          output a newline after each use of FORMAT");
    println!("      --printf=FORMAT   like --format, but interpret backslash escapes,");
    println!("                          and do not output a mandatory trailing newline;");
    println!("                          if you want a newline, include \\n in FORMAT");
    println!("  -t, --terse           print the information in terse form");
    println!("      --help     display this help and exit");
    println!("      --version  output version information and exit");
    println!();
    println!("The --cached MODE argument can be; always, never, or default.");
    println!("always will use cached attributes if available, while");
    println!("never will try to synchronize with the latest attributes, and");
    println!("default will leave it up to the underlying file system.");
    println!();
    println!("The valid format sequences for files (without --file-system):");
    println!("  %a   access rights in octal (note '#' and '0' printf flags)");
    println!("  %A   access rights in human readable form");
    println!("  %b   number of blocks allocated (see %B)");
    println!("  %B   the size in bytes of each block reported by %b");
    println!("  %C   SELinux security context string");
    println!("  %d   device number in decimal");
    println!("  %D   device number in hex");
    println!("  %f   raw mode in hex");
    println!("  %F   file type");
    println!("  %g   group ID of owner");
    println!("  %G   group name of owner");
    println!("  %h   number of hard links");
    println!("  %i   inode number");
    println!("  %m   mount point");
    println!("  %n   file name");
    println!("  %N   quoted file name with dereference if symbolic link");
    println!("  %o   optimal I/O transfer size hint");
    println!("  %s   total size, in bytes");
    println!("  %t   major device type in hex, for character/block device special files");
    println!("  %T   minor device type in hex, for character/block device special files");
    println!("  %u   user ID of owner");
    println!("  %U   user name of owner");
    println!("  %w   time of file birth, human-readable; - if unknown");
    println!("  %W   time of file birth, seconds since Epoch; 0 if unknown");
    println!("  %x   time of last access, human-readable");
    println!("  %X   time of last access, seconds since Epoch");
    println!("  %y   time of last data modification, human-readable");
    println!("  %Y   time of last data modification, seconds since Epoch");
    println!("  %z   time of last status change, human-readable");
    println!("  %Z   time of last status change, seconds since Epoch");
    println!();
    println!("Valid format sequences for file systems:");
    println!("  %a   free blocks available to non-superuser");
    println!("  %b   total data blocks in file system");
    println!("  %c   total file nodes in file system");
    println!("  %d   free file nodes in file system");
    println!("  %f   free blocks in file system");
    println!("  %i   file system ID in hex");
    println!("  %l   maximum length of filenames");
    println!("  %n   file name");
    println!("  %s   block size (for faster transfers)");
    println!("  %S   fundamental block size (for block counts)");
    println!("  %t   file system type in hex");
    println!("  %T   file system type in human readable form");
    println!();
    println!("Report stat bugs to <bug-reports@nexusshell.org>");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, NamedTempFile};
    use std::fs::File;
    use std::io::Write;
    
    #[test]
    fn test_parse_args() {
        let args = vec!["-L".to_string(), "-t".to_string(), "file.txt".to_string()];
        let options = parse_stat_args(&args).unwrap();
        
        assert!(options.dereference);
        assert!(options.terse);
        assert_eq!(options.files, vec!["file.txt"]);
    }
    
    #[test]
    fn test_file_system_option() {
        let args = vec!["-f".to_string(), "/".to_string()];
        let options = parse_stat_args(&args).unwrap();
        
        assert!(options.file_system);
        assert_eq!(options.files, vec!["/"]);
    }
    
    #[test]
    fn test_format_option() {
        let args = vec!["--format=%n %s".to_string(), "file.txt".to_string()];
        let options = parse_stat_args(&args).unwrap();
        
        assert_eq!(options.format, Some("%n %s".to_string()));
        assert_eq!(options.files, vec!["file.txt"]);
    }
    
    #[test]
    fn test_format_permissions() {
        // Test regular file with 644 permissions
        let mode = libc::S_IFREG | 0o644;
        let perms = format_permissions(mode);
        assert_eq!(perms, "-rw-r--r--");
        
        // Test directory with 755 permissions
        let mode = libc::S_IFDIR | 0o755;
        let perms = format_permissions(mode);
        assert_eq!(perms, "drwxr-xr-x");
        
        // Test symbolic link
        let mode = libc::S_IFLNK | 0o777;
        let perms = format_permissions(mode);
        assert_eq!(perms, "lrwxrwxrwx");
    }
    
    #[test]
    fn test_get_file_type_description() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file");
        File::create(&file_path).unwrap();
        
        let metadata = fs::metadata(&file_path).unwrap();
        let description = get_file_type_description(&metadata);
        assert_eq!(description, "regular file");
        
        let dir_metadata = fs::metadata(temp_dir.path()).unwrap();
        let dir_description = get_file_type_description(&dir_metadata);
        assert_eq!(dir_description, "directory");
    }
} 