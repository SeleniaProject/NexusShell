//! `ls` command  Ecomprehensive directory listing implementation.
//!
//! Supports most standard ls options:
//!   ls [OPTIONS] [FILES...]
//!   -a, --all              - Show hidden files
//!   -A, --almost-all       - Show all except . and ..
//!   -l                     - Long format listing
//!   -h, --human-readable   - Human readable sizes
//!   -r, --reverse          - Reverse sort order
//!   -t                     - Sort by modification time
//!   -S                     - Sort by file size
//!   -R, --recursive        - List subdirectories recursively
//!   -d, --directory        - List directories themselves, not contents
//!   -1                     - One file per line
//!   --color[=WHEN]         - Colorize output (always, never, auto)
//!   -i, --inode            - Show inode numbers
//!   -s, --size             - Show allocated size in blocks
//!   -F, --classify         - Append indicator to entries
//!   -G, --no-group         - Don't show group names in long format
//!   -n, --numeric-uid-gid  - Show numeric UIDs/GIDs instead of names
//!   -o                     - Long format without group info
//!   -g                     - Long format without owner info
//!   --time-style=STYLE     - Time display style
//!   --full-time            - Show full timestamp
//!   -c                     - Sort by change time
//!   -u                     - Sort by access time
//!   --group-directories-first - Group directories before files

use anyhow::{Result, anyhow};
use std::fs::{self, Metadata};
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::{DateTime, Local};
use nu_ansi_term::{Color as NuColor, Style};
use humansize::{format_size, BINARY};
use crate::ui_design::{TableFormatter, Colorize, TableOptions, BorderStyle, TextAlignment, Theme, set_theme, Animation, ProgressBar, ProgressStyle, Notification};
#[cfg(windows)]
use windows_sys::Win32::{
    Security::{
        LookupAccountSidW, GROUP_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION, SID_NAME_USE,
        GetFileSecurityW, GetSecurityDescriptorOwner, GetSecurityDescriptorGroup,
    },
};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

// Git repository integration
#[derive(Debug, Clone)]
pub struct GitRepository {
    pub root_path: PathBuf,
    pub is_initialized: bool,
}

impl GitRepository {
    /// Create a Git repository instance
    pub fn new(path: &Path) -> Option<Self> {
        Self::find_git_root(path).map(|git_root| GitRepository {
                root_path: git_root,
                is_initialized: true,
            })
    }

    /// Find the Git repository root by walking up directories
    fn find_git_root(start_path: &Path) -> Option<PathBuf> {
        let mut current = start_path;
        loop {
            let git_dir = current.join(".git");
            if git_dir.exists() {
                return Some(current.to_path_buf());
            }
            match current.parent() {
                Some(parent) => current = parent,
                None => return None,
            }
        }
    }

    /// Get Git status for a specific file
    pub fn get_file_status(&self, file_path: &Path) -> GitStatus {
        if !self.is_initialized {
            return GitStatus::None;
        }

        // Convert to relative path from repo root
        let relative_path = match file_path.strip_prefix(&self.root_path) {
            Ok(rel) => rel,
            Err(_) => return GitStatus::None,
        };

        // Use git command to get status
        if let Ok(output) = std::process::Command::new("git")
            .args(["status", "--porcelain", "--"])
            .arg(relative_path)
            .current_dir(&self.root_path)
            .output()
        {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = output_str.lines().next() {
                    if line.len() >= 2 {
                        return match &line[..2] {
                            "??" => GitStatus::Untracked,
                            "A " | " A" => GitStatus::Added,
                            "M " | " M" => GitStatus::Modified,
                            "D " | " D" => GitStatus::Deleted,
                            "R " | " R" => GitStatus::Renamed,
                            "C " | " C" => GitStatus::Copied,
                            "UU" | "AA" | "DD" => GitStatus::Conflicted,
                            _ => GitStatus::None,
                        };
                    }
                }
            }
        }

        GitStatus::None
    }
}

// Pure Rust user/group name resolution system
lazy_static::lazy_static! {
    static ref USER_CACHE: Mutex<HashMap<u32, String>> = Mutex::new(HashMap::new());
    static ref GROUP_CACHE: Mutex<HashMap<u32, String>> = Mutex::new(HashMap::new());
    #[cfg(windows)]
    static ref OWNER_GROUP_CACHE: Mutex<HashMap<PathBuf, (String, String)>> = Mutex::new(HashMap::new());
}

/// Get user name from UID with caching
fn get_user_name(uid: u32) -> String {
    // Check cache first
    if let Ok(cache) = USER_CACHE.lock() {
        if let Some(name) = cache.get(&uid) {
            return name.clone();
        }
    }

    // Try to resolve from system
    let name = resolve_user_name(uid).unwrap_or_else(|| uid.to_string());
    
    // Cache the result
    if let Ok(mut cache) = USER_CACHE.lock() {
        cache.insert(uid, name.clone());
    }
    
    name
}

/// Get group name from GID with caching
fn get_group_name(gid: u32) -> String {
    // Check cache first
    if let Ok(cache) = GROUP_CACHE.lock() {
        if let Some(name) = cache.get(&gid) {
            return name.clone();
        }
    }

    // Try to resolve from system
    let name = resolve_group_name(gid).unwrap_or_else(|| gid.to_string());
    
    // Cache the result
    if let Ok(mut cache) = GROUP_CACHE.lock() {
        cache.insert(gid, name.clone());
    }
    
    name
}

/// Pure Rust user/group resolution cache
use std::sync::OnceLock;

#[derive(Debug, Clone)]
struct UserGroupCache {
    users: HashMap<u32, String>,
    groups: HashMap<u32, String>,
}

impl UserGroupCache {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
            groups: HashMap::new(),
        }
    }
    
    fn get_user_name(&mut self, uid: u32) -> String {
        if let Some(name) = self.users.get(&uid) {
            return name.clone();
        }
        
        let name = Self::resolve_user_name_system(uid);
        self.users.insert(uid, name.clone());
        name
    }
    
    fn get_group_name(&mut self, gid: u32) -> String {
        if let Some(name) = self.groups.get(&gid) {
            return name.clone();
        }
        
        let name = Self::resolve_group_name_system(gid);
        self.groups.insert(gid, name.clone());
        name
    }
    
    #[cfg(unix)]
    fn resolve_user_name_system(uid: u32) -> String {
        // Try reading /etc/passwd directly
        if let Ok(passwd_content) = std::fs::read_to_string("/etc/passwd") {
            for line in passwd_content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    if let Ok(file_uid) = parts[2].parse::<u32>() {
                        if file_uid == uid {
                            return parts[0].to_string();
                        }
                    }
                }
            }
        }
        
        // Fallback to getent if /etc/passwd fails
        if let Ok(output) = std::process::Command::new("getent")
            .args(["passwd", &uid.to_string()])
            .output()
        {
            if output.status.success() {
                let passwd_line = String::from_utf8_lossy(&output.stdout);
                if let Some(name) = passwd_line.split(':').next() {
                    if !name.is_empty() {
                        return name.to_string();
                    }
                }
            }
        }
        
        // Final fallback to numeric ID
        uid.to_string()
    }
    
    #[cfg(unix)]
    fn resolve_group_name_system(gid: u32) -> String {
        // Try reading /etc/group directly
        if let Ok(group_content) = std::fs::read_to_string("/etc/group") {
            for line in group_content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    if let Ok(file_gid) = parts[2].parse::<u32>() {
                        if file_gid == gid {
                            return parts[0].to_string();
                        }
                    }
                }
            }
        }
        
        // Fallback to getent if /etc/group fails
        if let Ok(output) = std::process::Command::new("getent")
            .args(["group", &gid.to_string()])
            .output()
        {
            if output.status.success() {
                let group_line = String::from_utf8_lossy(&output.stdout);
                if let Some(name) = group_line.split(':').next() {
                    if !name.is_empty() {
                        return name.to_string();
                    }
                }
            }
        }
        
        // Final fallback to numeric ID
        gid.to_string()
    }
    
    #[cfg(windows)]
    fn resolve_user_name_system(uid: u32) -> String {
        // Try to get Windows user information
        if let Some(username) = Self::get_windows_username_from_sid(uid) {
            return username;
        }
        
        // Fallback to whoami command
        if let Ok(output) = std::process::Command::new("whoami").output() {
            if output.status.success() {
                let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !username.is_empty() {
                    return username;
                }
            }
        }
        
        format!("user{}", uid)
    }
    
    #[cfg(windows)]
    fn resolve_group_name_system(gid: u32) -> String {
        // Windows doesn't have traditional Unix-style groups
        // Try to resolve via Windows Security APIs
        if let Some(groupname) = Self::get_windows_groupname_from_sid(gid) {
            return groupname;
        }
        
        format!("group{}", gid)
    }
    
    #[cfg(windows)]
    fn get_windows_username_from_sid(_uid: u32) -> Option<String> {
        // This would use Windows Security APIs like LookupAccountSid
        // For now, return None to fall back to whoami
        None
    }
    
    #[cfg(windows)]
    fn get_windows_groupname_from_sid(_gid: u32) -> Option<String> {
        // This would use Windows Security APIs
        None
    }
}

static USER_GROUP_CACHE: OnceLock<Mutex<UserGroupCache>> = OnceLock::new();

fn get_cache() -> &'static Mutex<UserGroupCache> {
    USER_GROUP_CACHE.get_or_init(|| Mutex::new(UserGroupCache::new()))
}

/// Resolve user name from UID using pure Rust implementation
#[cfg(unix)]
fn resolve_user_name(uid: u32) -> Option<String> {
    let cache = get_cache();
    if let Ok(mut cache_guard) = cache.lock() {
        Some(cache_guard.get_user_name(uid))
    } else {
        // Fallback without cache
        Some(UserGroupCache::resolve_user_name_system(uid))
    }
}

/// Resolve group name from GID using pure Rust implementation
#[cfg(unix)]
fn resolve_group_name(gid: u32) -> Option<String> {
    let cache = get_cache();
    if let Ok(mut cache_guard) = cache.lock() {
        Some(cache_guard.get_group_name(gid))
    } else {
        // Fallback without cache
        Some(UserGroupCache::resolve_group_name_system(gid))
    }
}

/// Windows user/group resolution implementations
#[cfg(windows)]
fn resolve_user_name(uid: u32) -> Option<String> {
    let cache = get_cache();
    if let Ok(mut cache_guard) = cache.lock() {
        Some(cache_guard.get_user_name(uid))
    } else {
        // Fallback without cache
        Some(UserGroupCache::resolve_user_name_system(uid))
    }
}

#[cfg(windows)]
fn resolve_group_name(gid: u32) -> Option<String> {
    let cache = get_cache();
    if let Ok(mut cache_guard) = cache.lock() {
        Some(cache_guard.get_group_name(gid))
    } else {
        // Fallback without cache
        Some(UserGroupCache::resolve_group_name_system(gid))
    }
}


#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;

// Cross-platform metadata helpers
#[cfg(unix)]
fn get_uid(metadata: &Metadata) -> u32 {
    metadata.uid()
}

#[cfg(not(unix))]
fn get_uid(_metadata: &Metadata) -> u32 {
    0 // Default for Windows
}

#[cfg(unix)]
fn get_gid(metadata: &Metadata) -> u32 {
    metadata.gid()
}

#[cfg(not(unix))]
fn get_gid(_metadata: &Metadata) -> u32 {
    0 // Default for Windows
}

#[cfg(unix)]
fn get_nlink(metadata: &Metadata) -> u64 {
    metadata.nlink()
}

#[cfg(not(unix))]
fn get_nlink(_metadata: &Metadata) -> u64 {
    1 // Default for Windows
}

#[cfg(unix)]
fn get_ino(metadata: &Metadata) -> u64 {
    metadata.ino()
}

#[cfg(not(unix))]
fn get_ino(_metadata: &Metadata) -> u64 {
    0 // Default for Windows
}

#[cfg(unix)]
fn get_blocks(metadata: &Metadata) -> u64 {
    metadata.blocks()
}

#[cfg(not(unix))]
fn get_blocks(metadata: &Metadata) -> u64 {
    // Approximate blocks from file size
    metadata.len().div_ceil(512)
}

#[cfg(unix)]
fn get_mode(permissions: &std::fs::Permissions) -> u32 {
    permissions.mode()
}

#[cfg(not(unix))]
fn get_mode(_permissions: &std::fs::Permissions) -> u32 {
    0o644 // Default for Windows
}

// Cross-platform ctime/atime helpers with improved Windows support
#[cfg(unix)]
fn get_ctime(metadata: &Metadata) -> SystemTime {
    use std::os::unix::fs::MetadataExt as _;
    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(metadata.ctime() as u64) 
        + std::time::Duration::from_nanos(metadata.ctime_nsec() as u64)
}

#[cfg(windows)]
fn get_ctime(metadata: &Metadata) -> SystemTime {
    // On Windows, creation time is the closest equivalent to Unix ctime
    // But Windows also provides change time through file attributes
    if let Some(creation_time) = filetime::FileTime::from_creation_time(metadata) {
        filetime_to_system_time(creation_time)
    } else {
        // Fallback to modified time
        metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH)
    }
}

#[cfg(unix)]
fn get_atime(metadata: &Metadata) -> SystemTime {
    use std::os::unix::fs::MetadataExt as _;
    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(metadata.atime() as u64)
        + std::time::Duration::from_nanos(metadata.atime_nsec() as u64)
}

#[cfg(windows)]
fn get_atime(metadata: &Metadata) -> SystemTime {
    let ft = filetime::FileTime::from_last_access_time(metadata);
    filetime_to_system_time(ft)
}

#[cfg(windows)]
fn filetime_to_system_time(ft: filetime::FileTime) -> SystemTime {
    // filetime::FileTime::seconds() returns i64 (seconds since UNIX_EPOCH), can be negative.
    let secs = ft.seconds();
    let nanos = ft.nanoseconds();
    
    if secs >= 0 {
        let mut time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs as u64);
        if nanos > 0 {
            time += std::time::Duration::from_nanos(nanos as u64);
        }
        time
    } else {
        // For pre-1970 timestamps, subtract from UNIX_EPOCH safely
        let duration = std::time::Duration::from_secs((-secs) as u64);
        if let Some(time) = SystemTime::UNIX_EPOCH.checked_sub(duration) {
            if nanos > 0 {
                // When going backwards, adjust nanoseconds with checked_sub
                time.checked_sub(std::time::Duration::from_nanos(nanos as u64))
                    .unwrap_or(SystemTime::UNIX_EPOCH)
            } else {
                time
            }
        } else {
            SystemTime::UNIX_EPOCH
        }
    }
}

#[derive(Debug, Clone)]
pub struct LsOptions {
    pub show_hidden: bool,
    pub show_almost_all: bool,
    pub long_format: bool,
    pub human_readable: bool,
    pub reverse_sort: bool,
    pub sort_by_time: bool,
    pub sort_by_size: bool,
    pub sort_by_ctime: bool,
    pub sort_by_atime: bool,
    pub recursive: bool,
    pub directory_only: bool,
    pub one_per_line: bool,
    pub color: ColorOption,
    pub show_inode: bool,
    pub show_size_blocks: bool,
    pub classify: bool,
    pub no_group: bool,
    pub numeric_ids: bool,
    pub long_no_group: bool,
    pub long_no_owner: bool,
    pub time_style: TimeStyle,
    pub full_time: bool,
    pub group_dirs_first: bool,
    pub git_status: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorOption {
    Always,
    Never,
    Auto,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeStyle {
    Default,
    Iso,
    LongIso,
    Full,
    Locale,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: PathBuf,
    pub metadata: Metadata,
    pub is_symlink: bool,
    pub symlink_target: Option<String>,
    pub git_status: Option<GitStatus>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GitStatus {
    None,
    Clean,
    Untracked,
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    TypeChange,
    Ignored,
    Conflicted,
}

impl Default for LsOptions {
    fn default() -> Self {
        Self {
            show_hidden: false,
            show_almost_all: false,
            long_format: false,
            human_readable: false,
            reverse_sort: false,
            sort_by_time: false,
            sort_by_size: false,
            sort_by_ctime: false,
            sort_by_atime: false,
            recursive: false,
            directory_only: false,
            one_per_line: false,
            color: ColorOption::Auto,
            show_inode: false,
            show_size_blocks: false,
            classify: false,
            no_group: false,
            numeric_ids: false,
            long_no_group: false,
            long_no_owner: false,
            time_style: TimeStyle::Default,
            full_time: false,
            group_dirs_first: false,
            git_status: true,
        }
    }
}

pub fn ls_async(dir: Option<&str>) -> Result<()> {
    let args = if let Some(dir) = dir {
        vec![dir.to_string()]
    } else {
        vec![]
    };
    ls_cli(&args)
}

pub fn ls_cli(args: &[String]) -> Result<()> {
    let (options, paths) = parse_ls_args(args)?;
    
    let paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.into_iter().map(PathBuf::from).collect()
    };
    
    // Check if we should use colors
    let use_colors = should_use_colors(&options.color);
    
    // Initialize git repository if needed
    let default_path = PathBuf::from(".");
    let git_repo = if options.git_status {
        // Try to find Git repository starting from first path
        let start_path = paths.first().unwrap_or(&default_path);
        GitRepository::new(start_path)
    } else {
        None
    };
    
    for (i, path) in paths.iter().enumerate() {
        if i > 0 {
            println!();
        }
        
        if paths.len() > 1 {
            println!("{}:", path.display());
        }
        
        list_directory(path, &options, use_colors, git_repo.as_ref())?;
    }
    
    Ok(())
}

fn parse_ls_args(args: &[String]) -> Result<(LsOptions, Vec<String>)> {
    let mut options = LsOptions::default();
    let mut paths = Vec::new();
    let mut i = 0;
    
    while i < args.len() {
        let arg = &args[i];
        
        if arg.starts_with('-') && arg.len() > 1 {
            if arg.starts_with("--") {
                // Long options
                match arg.as_str() {
                    "--all" => options.show_hidden = true,
                    "--almost-all" => options.show_almost_all = true,
                    "--human-readable" => options.human_readable = true,
                    "--reverse" => options.reverse_sort = true,
                    "--recursive" => options.recursive = true,
                    "--directory" => options.directory_only = true,
                    "--inode" => options.show_inode = true,
                    "--size" => options.show_size_blocks = true,
                    "--classify" => options.classify = true,
                    "--no-group" => options.no_group = true,
                    "--numeric-uid-gid" => options.numeric_ids = true,
                    "--full-time" => {
                        options.full_time = true;
                        options.long_format = true;
                    }
                    "--group-directories-first" => options.group_dirs_first = true,
                    "--color" => options.color = ColorOption::Always,
                    "--color=always" => options.color = ColorOption::Always,
                    "--color=never" => options.color = ColorOption::Never,
                    "--color=auto" => options.color = ColorOption::Auto,
                    arg if arg.starts_with("--time-style=") => {
                        let style = arg.strip_prefix("--time-style=").unwrap();
                        options.time_style = match style {
                            "iso" => TimeStyle::Iso,
                            "long-iso" => TimeStyle::LongIso,
                            "full" => TimeStyle::Full,
                            "locale" => TimeStyle::Locale,
                            _ => return Err(anyhow!("ls: invalid time style '{}'", style)),
                        };
                    }
                    _ => return Err(anyhow!("ls: unknown option '{}'", arg)),
                }
            } else {
                // Short options
                let chars: Vec<char> = arg.chars().skip(1).collect();
                for ch in chars {
                    match ch {
                        'a' => options.show_hidden = true,
                        'A' => options.show_almost_all = true,
                        'l' => options.long_format = true,
                        'h' => options.human_readable = true,
                        'r' => options.reverse_sort = true,
                        't' => options.sort_by_time = true,
                        'S' => options.sort_by_size = true,
                        'R' => options.recursive = true,
                        'd' => options.directory_only = true,
                        '1' => options.one_per_line = true,
                        'i' => options.show_inode = true,
                        's' => options.show_size_blocks = true,
                        'F' => options.classify = true,
                        'G' => options.no_group = true,
                        'n' => options.numeric_ids = true,
                        'o' => {
                            options.long_format = true;
                            options.long_no_group = true;
                        }
                        'g' => {
                            options.long_format = true;
                            options.long_no_owner = true;
                        }
                        'c' => options.sort_by_ctime = true,
                        'u' => options.sort_by_atime = true,
                        _ => return Err(anyhow!("ls: unknown option '-{}'", ch)),
                    }
                }
            }
        } else {
            paths.push(arg.clone());
        }
        
        i += 1;
    }
    
    Ok((options, paths))
}

fn should_use_colors(color_option: &ColorOption) -> bool {
    match color_option {
        ColorOption::Always => true,
        ColorOption::Never => false,
        ColorOption::Auto => std::io::IsTerminal::is_terminal(&std::io::stdout()),
    }
}

fn list_directory(
    path: &Path,
    options: &LsOptions,
    use_colors: bool,
    git_repo: Option<&GitRepository>,
) -> Result<()> {
    if options.directory_only {
        // Just list the directory itself
        let file_info = get_file_info(path, git_repo)?;
        if options.long_format {
            print_long_format(&[file_info], options, use_colors)?;
        } else {
            print_short_format(&[file_info], options, use_colors)?;
        }
        return Ok(());
    }
    
    let entries = read_directory_sync(path, options, git_repo)?;
    
    if entries.is_empty() {
        return Ok(());
    }
    
    let mut sorted_entries = entries;
    sort_entries(&mut sorted_entries, options);
    
    if options.long_format {
        print_long_format(&sorted_entries, options, use_colors)?;
    } else {
        print_short_format(&sorted_entries, options, use_colors)?;
    }
    
    Ok(())
}



fn read_directory_sync(
    path: &Path,
    options: &LsOptions,
    git_repo: Option<&GitRepository>, // Reference to Git repository
) -> Result<Vec<FileInfo>> {
    let mut entries = Vec::new();
    
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        
        // Skip hidden files unless requested
        if file_name.starts_with('.') {
            if !options.show_hidden && !options.show_almost_all {
                continue;
            }
            if options.show_almost_all && (file_name == "." || file_name == "..") {
                continue;
            }
        }
        
        let file_info = get_file_info(&entry.path(), git_repo)?;
        entries.push(file_info);
    }
    
    Ok(entries)
}

fn get_file_info(path: &Path, git_repo: Option<&GitRepository>) -> Result<FileInfo> {
    let metadata = fs::symlink_metadata(path)?;
    let is_symlink = metadata.file_type().is_symlink();
    let name = path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .to_string();
    
    let symlink_target = if is_symlink {
        fs::read_link(path).ok().map(|p| p.to_string_lossy().to_string())
    } else {
        None
    };
    
    let git_status = git_repo.map(|repo| get_git_status(repo, path));
    
    Ok(FileInfo {
        name,
        path: path.to_path_buf(),
        metadata,
        is_symlink,
        symlink_target,
        git_status,
    })
}

// Git status checking implementation using pure Rust
fn get_git_status(git_repo: &GitRepository, path: &Path) -> GitStatus {
    git_repo.get_file_status(path)
}

fn sort_entries(entries: &mut [FileInfo], options: &LsOptions) {
    entries.sort_by(|a, b| {
        // Group directories first if requested
        if options.group_dirs_first {
            let a_is_dir = a.metadata.is_dir();
            let b_is_dir = b.metadata.is_dir();
            if a_is_dir != b_is_dir {
                return b_is_dir.cmp(&a_is_dir);
            }
        }
        
        let cmp = if options.sort_by_time {
            b.metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH)
                .cmp(&a.metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH))
        } else if options.sort_by_size {
            b.metadata.len().cmp(&a.metadata.len())
        } else if options.sort_by_ctime {
            let a_time = get_ctime(&a.metadata);
            let b_time = get_ctime(&b.metadata);
            b_time.cmp(&a_time)
        } else if options.sort_by_atime {
            let a_time = get_atime(&a.metadata);
            let b_time = get_atime(&b.metadata);
            b_time.cmp(&a_time)
        } else {
            // Default: sort by name
            a.name.cmp(&b.name)
        };
        
        if options.reverse_sort {
            cmp.reverse()
        } else {
            cmp
        }
    });
}

fn print_long_format(entries: &[FileInfo], options: &LsOptions, use_colors: bool) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }

    let formatter = TableFormatter::new();
    
    // Show loading animation for large directories
    if entries.len() > 100 {
        Animation::spinner(500, "Preparing file listing...");
    }
    
    // Configure advanced table options
    let table_options = TableOptions {
        border_style: BorderStyle::rounded(),
        show_header: true,
        alternating_rows: entries.len() > 10,
        alignment: TextAlignment::Left,
        max_width: Some(120),
    };
    
    // Define table headers
    let mut headers = vec![];
    
    if options.show_inode {
        headers.push("Inode".to_string());
    }
    
    headers.push("Permissions".to_string());
    headers.push("Links".to_string());
    
    if !options.long_no_owner {
        headers.push("Owner".to_string());
    }
    
    if !options.long_no_group && !options.no_group {
        headers.push("Group");
    }
    
    headers.push("Size");
    headers.push("Modified");
    headers.push("Name");
    
    // Prepare table rows
    let mut rows = Vec::new();
    
    for entry in entries {
        let mut row = Vec::new();
        
        // Inode
        if options.show_inode {
            row.push(get_ino(&entry.metadata).to_string().muted());
        }
        
        // Permissions
        #[cfg(unix)]
        let mode = entry.metadata.mode();
        #[cfg(windows)]
        let mode = 0o755; // Default for Windows
        
        row.push(formatter.format_permissions(mode));
        
        // Links
        row.push(get_nlink(&entry.metadata).to_string().info());
        
        // Owner
        if !options.long_no_owner {
            let owner = if options.numeric_ids {
                get_uid(&entry.metadata).to_string()
            } else {
                #[cfg(windows)]
                {
                    if let Some((owner, _)) = get_windows_owner_group(&entry.path) {
                        owner
                    } else {
                        get_uid(&entry.metadata).to_string()
                    }
                }
                #[cfg(not(windows))]
                {
                    get_user_name(get_uid(&entry.metadata))
                }
            };
            row.push(owner.primary());
        }
        
        // Group
        if !options.long_no_group && !options.no_group {
            let group = if options.numeric_ids {
                get_gid(&entry.metadata).to_string()
            } else {
                #[cfg(windows)]
                {
                    if let Some((_, group)) = get_windows_owner_group(&entry.path) {
                        group
                    } else {
                        get_gid(&entry.metadata).to_string()
                    }
                }
                #[cfg(not(windows))]
                {
                    get_group_name(get_gid(&entry.metadata))
                }
            };
            row.push(group.secondary());
        }
        
        // Size
        let size_str = if entry.metadata.is_dir() {
            "-".muted()
        } else {
            formatter.format_size(entry.metadata.len())
        };
        row.push(size_str);
        
        // Modified time
        let modified_time = entry.metadata.modified()
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let datetime: DateTime<Local> = DateTime::from(modified_time);
        let time_str = if options.full_time {
            datetime.format("%Y-%m-%d %H:%M:%S %.3f %z").to_string()
        } else {
            datetime.format("%b %d %H:%M").to_string()
        };
        row.push(time_str.dim());
        
        // Name with icon
        let icon = formatter.get_file_icon(&entry.path, entry.metadata.is_dir(), 
                                         is_executable(&entry.metadata));
        let name_with_icon = if use_colors {
            let colored_name = format_file_name(entry, use_colors, false);
            format!("{} {}", icon, colored_name)
        } else {
            format!("{} {}", icon, entry.path.file_name().unwrap_or_default().to_string_lossy())
        };
        
        // Add classification suffix if requested
        let final_name = if options.classify {
            let suffix = if entry.metadata.is_dir() {
                "/".primary()
            } else if entry.is_symlink {
                "@".info()
            } else if is_executable(&entry.metadata) {
                "*".success()
            } else {
                "".to_string()
            };
            format!("{}{}", name_with_icon, suffix)
        } else {
            name_with_icon
    }
    
    // Print the beautiful advanced table
    print!("{}", formatter.create_advanced_table(headers, rows, table_options));
    
    // Show summary for large directories
    if entries.len() > 50 {
        Notification::info(&format!("Listed {} items", entries.len()));
    }
    
    Ok(())
}

fn print_long_entry(
    entry: &FileInfo,
    options: &LsOptions,
    use_colors: bool,
    max_links: usize,
    max_size: usize,
    max_user: usize,
    max_group: usize,
) -> Result<()> {
    let mut line = String::new();
    
    // Inode number
    if options.show_inode {
        line.push_str(&format!("{:8} ", get_ino(&entry.metadata)));
    }
    
    // Block size
    if options.show_size_blocks {
        let blocks = get_blocks(&entry.metadata);
        line.push_str(&format!("{blocks:6} "));
    }
    
    // File type and permissions
    line.push_str(&format_permissions(&entry.metadata));
    
    // Number of links
    line.push_str(&format!(" {:width$}", get_nlink(&entry.metadata), width = max_links));
    
    // Owner
    if !options.long_no_owner {
        let owner = if options.numeric_ids {
            get_uid(&entry.metadata).to_string()
        } else {
            #[cfg(windows)]
            {
                if let Some((owner, _group)) = get_windows_owner_group(&entry.path) {
                    owner
                } else {
                    get_user_name(get_uid(&entry.metadata))
                }
            }
            #[cfg(not(windows))]
            {
                get_user_name(get_uid(&entry.metadata))
            }
        };
        line.push_str(&format!(" {owner:max_user$}"));
    }
    
    // Group
    if !options.long_no_group && !options.no_group {
        let group = if options.numeric_ids {
            get_gid(&entry.metadata).to_string()
        } else {
            #[cfg(windows)]
            {
                if let Some((_owner, group)) = get_windows_owner_group(&entry.path) {
                    group
                } else {
                    get_group_name(get_gid(&entry.metadata))
                }
            }
            #[cfg(not(windows))]
            {
                get_group_name(get_gid(&entry.metadata))
            }
        };
        line.push_str(&format!(" {group:max_group$}"));
    }
    
    // File size
    let size_str = format_file_size(entry.metadata.len(), options.human_readable);
    line.push_str(&format!(" {size_str:>max_size$}"));
    
    // Modification time
    let time_str = format_time(&entry.metadata, &options.time_style, options.full_time);
    line.push_str(&format!(" {time_str}"));
    
    // File name with colors and git status
    line.push(' ');
    let colored_name = format_file_name(entry, use_colors, options.classify);
    line.push_str(&colored_name);
    
    // Symlink target
    if entry.is_symlink {
        if let Some(ref target) = entry.symlink_target {
            line.push_str(" -> ");
            line.push_str(target);
        }
    }
    
    println!("{line}");
    
    Ok(())
}

fn print_short_format(entries: &[FileInfo], options: &LsOptions, use_colors: bool) -> Result<()> {
    let formatter = TableFormatter::new();
    
    if options.one_per_line {
        for entry in entries {
            let mut line = String::new();
            
            if options.show_inode {
                line.push_str(&format!("{} ", get_ino(&entry.metadata).to_string().muted()));
            }
            
            if options.show_size_blocks {
                let blocks = get_blocks(&entry.metadata);
                line.push_str(&format!("{} ", blocks.to_string().dim()));
            }
            
            // Add file icon
            let icon = formatter.get_file_icon(&entry.path, entry.metadata.is_dir(), 
                                             is_executable(&entry.metadata));
            line.push_str(&format!("{} ", icon));
            
            // Add colored file name
            let colored_name = format_file_name(entry, use_colors, options.classify);
            line.push_str(&colored_name);
            
            println!("{}", line);
        }
    } else {
        // Multi-column output with beautiful formatting
        let term_width = terminal_size::terminal_size()
            .map(|(w, _)| w.0 as usize)
            .unwrap_or(80);
        print_beautiful_columns(entries, options, use_colors, term_width)?;
    }
    
    Ok(())
}

fn print_columns(entries: &[FileInfo], options: &LsOptions, use_colors: bool, term_width: usize) -> Result<()> {
    let mut names = Vec::new();
    let mut max_width = 0;
    
    for entry in entries {
        let mut name = String::new();
        
        if options.show_inode {
            name.push_str(&format!("{:8} ", get_ino(&entry.metadata)));
        }
        
        if options.show_size_blocks {
            let blocks = get_blocks(&entry.metadata);
            name.push_str(&format!("{blocks:6} "));
        }
        
        let colored_name = format_file_name(entry, use_colors, options.classify);
        name.push_str(&colored_name);
        
        let display_width = unicode_width::UnicodeWidthStr::width(name.as_str());
        max_width = max_width.max(display_width);
        names.push(name);
    }
    
    if max_width == 0 {
        return Ok(());
    }
    
    let cols = (term_width / (max_width + 2)).max(1);
    let rows = names.len().div_ceil(cols);
    
    for row in 0..rows {
        let mut line = String::new();
        for col in 0..cols {
            let idx = row + col * rows;
            if idx < names.len() {
                let name = &names[idx];
                let width = unicode_width::UnicodeWidthStr::width(name.as_str());
                line.push_str(name);
                
                if col < cols - 1 && idx + rows < names.len() {
                    for _ in width..max_width + 2 {
                        line.push(' ');
                    }
                }
            }
        }
        println!("{line}");
    }
    
    Ok(())
}

fn print_beautiful_columns(entries: &[FileInfo], options: &LsOptions, use_colors: bool, term_width: usize) -> Result<()> {
    let formatter = TableFormatter::new();
    let mut items = Vec::new();
    let mut max_width = 0;
    
    for entry in entries {
        let mut item = String::new();
        
        if options.show_inode {
            item.push_str(&format!("{} ", get_ino(&entry.metadata).to_string().muted()));
        }
        
        if options.show_size_blocks {
            let blocks = get_blocks(&entry.metadata);
            item.push_str(&format!("{} ", blocks.to_string().dim()));
        }
        
        // Add beautiful icon
        let icon = formatter.get_file_icon(&entry.path, entry.metadata.is_dir(), 
                                         is_executable(&entry.metadata));
        item.push_str(&format!("{} ", icon));
        
        let colored_name = format_file_name(entry, use_colors, options.classify);
        item.push_str(&colored_name);
        
        let display_width = formatter.display_width(&item);
        max_width = max_width.max(display_width);
        items.push(item);
    }
    
    if max_width == 0 {
        return Ok(());
    }
    
    // Calculate optimal columns with padding
    let padding = 3; // Space between columns
    let cols = ((term_width + padding) / (max_width + padding)).max(1);
    let rows = items.len().div_ceil(cols);
    
    // Print header if many files
    if items.len() > 20 {
        println!("{}", "Files".bright());
        println!("{}", "─".repeat(term_width.min(40)).muted());
    }
    
    for row in 0..rows {
        let mut line = String::new();
        for col in 0..cols {
            let idx = row + col * rows;
            if idx < items.len() {
                let item = &items[idx];
                let width = formatter.display_width(item);
                line.push_str(item);
                
                // Add padding for alignment
                if col < cols - 1 && idx + rows < items.len() {
                    let spaces_needed = max_width + padding - width;
                    line.push_str(&" ".repeat(spaces_needed));
                }
            }
        }
        println!("{}", line);
    }
    
    Ok(())
}

fn format_permissions(metadata: &Metadata) -> String {
    let mode = get_mode(&metadata.permissions());
    let mut perms = String::with_capacity(10);
    
    // File type
    #[cfg(unix)]
    {
        if metadata.file_type().is_symlink() {
            perms.push('l');
        } else if metadata.file_type().is_dir() {
            perms.push('d');
        } else if metadata.file_type().is_block_device() {
            perms.push('b');
        } else if metadata.file_type().is_char_device() {
            perms.push('c');
        } else if metadata.file_type().is_fifo() {
            perms.push('p');
        } else if metadata.file_type().is_socket() {
            perms.push('s');
        } else {
            perms.push('-');
        }
    }
    
    #[cfg(not(unix))]
    {
        if metadata.is_dir() {
            perms.push('d');
        } else {
            perms.push('-');
        }
    }
    
    // Owner permissions
    perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o100 != 0 { 'x' } else { '-' });
    
    // Group permissions
    perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o010 != 0 { 'x' } else { '-' });
    
    // Other permissions
    perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    
    perms
}

fn format_file_size(size: u64, human_readable: bool) -> String {
    if human_readable {
        format_size(size, BINARY)
    } else {
        size.to_string()
    }
}

fn format_time(metadata: &Metadata, time_style: &TimeStyle, full_time: bool) -> String {
    let mtime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let datetime = DateTime::<Local>::from(mtime);
    
    if full_time {
        return datetime.format("%Y-%m-%d %H:%M:%S.%9f %z").to_string();
    }
    
    match time_style {
        TimeStyle::Iso => datetime.format("%m-%d %H:%M").to_string(),
        TimeStyle::LongIso => datetime.format("%Y-%m-%d %H:%M").to_string(),
        TimeStyle::Full => datetime.format("%a %b %e %H:%M:%S %Y").to_string(),
        TimeStyle::Locale => datetime.format("%b %e %H:%M").to_string(),
        TimeStyle::Default => {
            let now = Local::now();
            let six_months_ago = now - chrono::Duration::days(180);
            
            if datetime > six_months_ago && datetime <= now {
                datetime.format("%b %e %H:%M").to_string()
            } else {
                datetime.format("%b %e  %Y").to_string()
            }
        }
    }
}

fn format_file_name(entry: &FileInfo, use_colors: bool, classify: bool) -> String {
    let mut name = entry.name.clone();
    
    // Add classification suffix
    if classify {
        if entry.metadata.is_dir() {
            name.push('/');
        } else if entry.is_symlink {
            name.push('@');
        } else if is_executable(&entry.metadata) {
            name.push('*');
        }
    }
    
    if !use_colors {
        return name;
    }
    
    // Apply colors based on file type and git status
    let mut style = Style::new();
    
    if entry.metadata.is_dir() {
        style = style.fg(NuColor::Blue).bold();
    } else if entry.is_symlink {
        style = style.fg(NuColor::Cyan);
    } else if is_executable(&entry.metadata) {
        style = style.fg(NuColor::Green);
    } else {
        // Color by extension
        if let Some(ext) = entry.path.extension() {
            match ext.to_string_lossy().to_lowercase().as_str() {
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "ico" => {
                    style = style.fg(NuColor::Purple);
                }
                "mp3" | "wav" | "flac" | "ogg" | "m4a" => {
                    style = style.fg(NuColor::Cyan);
                }
                "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" => {
                    style = style.fg(NuColor::Purple);
                }
                "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => {
                    style = style.fg(NuColor::Red);
                }
                "txt" | "md" | "rst" | "doc" | "pdf" => {
                    style = style.fg(NuColor::Yellow);
                }
                _ => {}
            }
        }
    }
    
    // Add git status indicator
    if let Some(ref git_status) = entry.git_status {
        let git_indicator = match git_status {
            GitStatus::None => "",
            GitStatus::Clean => "",
            GitStatus::Untracked => "?",
            GitStatus::Modified => "M",
            GitStatus::Added => "A",
            GitStatus::Deleted => "D",
            GitStatus::Renamed => "R",
            GitStatus::Copied => "C",
            GitStatus::TypeChange => "T",
            GitStatus::Ignored => "!",
            GitStatus::Conflicted => "U",
        };
        
        let git_color = match git_status {
            GitStatus::None => NuColor::White,
            GitStatus::Clean => NuColor::Green,
            GitStatus::Untracked => NuColor::Red,
            GitStatus::Modified => NuColor::Yellow,
            GitStatus::Added => NuColor::Green,
            GitStatus::Deleted => NuColor::Red,
            GitStatus::Renamed => NuColor::Blue,
            GitStatus::Copied => NuColor::Blue,
            GitStatus::TypeChange => NuColor::Purple,
            GitStatus::Ignored => NuColor::Fixed(8), // Dark gray
            GitStatus::Conflicted => NuColor::Red,
        };
        
        return format!("{} {}", 
            if git_indicator.is_empty() { "".to_string() } else { git_color.paint(git_indicator).to_string() },
            style.paint(name)
        );
    }
    
    style.paint(name).to_string()
}

// (removed duplicate generic helpers; platform-specific versions above are used)

fn is_executable(_metadata: &Metadata) -> bool {
    #[cfg(unix)]
    {
        get_mode(&metadata.permissions()) & 0o111 != 0
    }
    
    #[cfg(not(unix))]
    {
        false // Windows doesn't have the same concept
    }
}

// Windows: retrieve file owner and primary group via WinAPI (GetFileSecurityW + LookupAccountSidW)
#[cfg(windows)]
fn get_windows_owner_group(path: &Path) -> Option<(String, String)> {
    // Cache fast‑path
    if let Ok(cache) = OWNER_GROUP_CACHE.lock() {
        if let Some(v) = cache.get(path) { return Some(v.clone()); }
    }
    unsafe {
        // Convert path to wide string
        let widestr: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();

        // First call to get required security descriptor size
        let mut needed: u32 = 0;
        let _ = GetFileSecurityW(
            widestr.as_ptr(),
            OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            0,
            &mut needed,
        );
        if needed == 0 {
            return None;
        }

        // Allocate buffer and retrieve security descriptor
        let mut sd_buf = vec![0u8; needed as usize];
        if GetFileSecurityW(
            widestr.as_ptr(),
            OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION,
            sd_buf.as_mut_ptr() as *mut core::ffi::c_void,
            needed,
            &mut needed,
        ) == 0
        {
            return None;
        }

        let mut p_owner_sid: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut owner_defaulted: i32 = 0;
        if GetSecurityDescriptorOwner(
            sd_buf.as_mut_ptr() as *mut core::ffi::c_void,
            &mut p_owner_sid,
            &mut owner_defaulted,
        ) == 0
        {
            return None;
        }

        let mut p_group_sid: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut group_defaulted: i32 = 0;
        if GetSecurityDescriptorGroup(
            sd_buf.as_mut_ptr() as *mut core::ffi::c_void,
            &mut p_group_sid,
            &mut group_defaulted,
        ) == 0
        {
            return None;
        }

        // Helper to resolve SID to Domain\\Name
        unsafe fn sid_to_name(sid: *mut core::ffi::c_void) -> Option<String> {
            if sid.is_null() {
                return None;
            }
            let mut name_len: u32 = 0;
            let mut domain_len: u32 = 0;
            let mut use_type: SID_NAME_USE = 0;
            // First call to get required lengths
            let _ = LookupAccountSidW(
                std::ptr::null(),
                sid,
                std::ptr::null_mut(),
                &mut name_len,
                std::ptr::null_mut(),
                &mut domain_len,
                &mut use_type,
            );
            if name_len == 0 { return None; }
            let mut name_buf = vec![0u16; name_len as usize];
            let mut domain_buf = if domain_len > 0 { vec![0u16; domain_len as usize] } else { Vec::new() };
            if LookupAccountSidW(
                std::ptr::null(),
                sid,
                name_buf.as_mut_ptr(),
                &mut name_len,
                if domain_len > 0 { domain_buf.as_mut_ptr() } else { std::ptr::null_mut() },
                &mut domain_len,
                &mut use_type,
            ) == 0
            {
                return None;
            }
            // Trim trailing NULs
            while let Some(&0) = name_buf.last() { name_buf.pop(); }
            while let Some(&0) = domain_buf.last() { domain_buf.pop(); }
            let name = String::from_utf16(&name_buf).ok()?;
            let domain = if !domain_buf.is_empty() { Some(String::from_utf16(&domain_buf).ok()?) } else { None };
            Some(match domain {
                Some(d) if !d.is_empty() => format!("{d}\\{name}"),
                _ => name,
            })
        }

        let owner = sid_to_name(p_owner_sid).unwrap_or_else(|| "unknown".to_string());
        let group = sid_to_name(p_group_sid).unwrap_or_else(|| "unknown".to_string());

        let pair = (owner, group);
        if let Ok(mut cache) = OWNER_GROUP_CACHE.lock() {
            cache.insert(path.to_path_buf(), pair.clone());
        }
        Some(pair)
    }
}
