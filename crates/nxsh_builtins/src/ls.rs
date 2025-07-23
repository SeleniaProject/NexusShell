//! `ls` command – comprehensive directory listing implementation.
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
use std::collections::HashMap;
use std::fs::{self, DirEntry, Metadata};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Local, TimeZone};
use users::{Users, UsersCache, Groups, get_user_by_uid, get_group_by_gid};
use unicode_width::UnicodeWidthStr;
use ansi_term::{Colour, Style, ANSIString};
use humansize::{format_size, DECIMAL, BINARY};
use tabled::{Table, Tabled, settings::{Style as TableStyle, Alignment, Width}};
use std::pin::Pin;
use std::future::Future;

#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;

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
    Untracked,
    Modified,
    Added,
    Deleted,
    Renamed,
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

pub async fn ls_async(dir: Option<&str>) -> Result<()> {
    let args = if let Some(dir) = dir {
        vec![dir.to_string()]
    } else {
        vec![]
    };
    ls_cli(&args).await
}

pub async fn ls_cli(args: &[String]) -> Result<()> {
    let (options, paths) = parse_ls_args(args)?;
    
    let paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.into_iter().map(PathBuf::from).collect()
    };
    
    // Check if we should use colors
    let use_colors = should_use_colors(&options.color);
    
    // Initialize git repository if needed
    let git_repo = if options.git_status {
        git2::Repository::discover(".").ok()
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
        
        list_directory(path, &options, use_colors, &git_repo).await?;
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
        ColorOption::Auto => atty::is(atty::Stream::Stdout),
    }
}

async fn list_directory(
    path: &Path,
    options: &LsOptions,
    use_colors: bool,
    git_repo: &Option<git2::Repository>,
) -> Result<()> {
    list_directory_boxed(path, options, use_colors, git_repo).await
}

fn list_directory_boxed(
    path: &Path,
    options: &LsOptions,
    use_colors: bool,
    git_repo: Option<&git2::Repository>,
) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
    Box::pin(async move {
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
        
        let entries = read_directory(path, options, git_repo).await?;
        
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
        
        // Handle recursive listing
        if options.recursive {
            for entry in &sorted_entries {
                if entry.metadata.is_dir() && entry.name != "." && entry.name != ".." {
                    println!("\n{}:", entry.path.display());
                    list_directory_boxed(&entry.path, options, use_colors, git_repo).await?;
                }
            }
        }
        
        Ok(())
    })
}

async fn read_directory(
    path: &Path,
    options: &LsOptions,
    git_repo: &Option<git2::Repository>,
) -> Result<Vec<FileInfo>> {
    let mut entries = Vec::new();
    
    let mut dir_entries = tokio::fs::read_dir(path).await?;
    
    while let Some(entry) = dir_entries.next_entry().await? {
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

fn get_file_info(path: &Path, git_repo: &Option<git2::Repository>) -> Result<FileInfo> {
    let metadata = fs::symlink_metadata(path)?;
    let is_symlink = metadata.file_type().is_symlink();
    let name = path.file_name()
        .unwrap_or_else(|| path.as_os_str())
        .to_string_lossy()
        .to_string();
    
    let symlink_target = if is_symlink {
        fs::read_link(path).ok().map(|p| p.to_string_lossy().to_string())
    } else {
        None
    };
    
    let git_status = if let Some(repo) = git_repo {
        get_git_status(repo, path).ok()
    } else {
        None
    };
    
    Ok(FileInfo {
        name,
        path: path.to_path_buf(),
        metadata,
        is_symlink,
        symlink_target,
        git_status,
    })
}

fn get_git_status(repo: &git2::Repository, path: &Path) -> Result<GitStatus, git2::Error> {
    let mut status_opts = git2::StatusOptions::new();
    status_opts.include_untracked(true);
    status_opts.include_ignored(false);
    
    let statuses = repo.statuses(Some(&mut status_opts))?;
    
    let relative_path = path.strip_prefix(repo.workdir().unwrap_or(Path::new(".")))
        .unwrap_or(path);
    
    for entry in statuses.iter() {
        if Path::new(entry.path().unwrap_or("")) == relative_path {
            let flags = entry.status();
            
            if flags.contains(git2::Status::CONFLICTED) {
                return Ok(GitStatus::Conflicted);
            }
            if flags.contains(git2::Status::WT_NEW) || flags.contains(git2::Status::INDEX_NEW) {
                return Ok(GitStatus::Added);
            }
            if flags.contains(git2::Status::WT_MODIFIED) || flags.contains(git2::Status::INDEX_MODIFIED) {
                return Ok(GitStatus::Modified);
            }
            if flags.contains(git2::Status::WT_DELETED) || flags.contains(git2::Status::INDEX_DELETED) {
                return Ok(GitStatus::Deleted);
            }
            if flags.contains(git2::Status::WT_RENAMED) || flags.contains(git2::Status::INDEX_RENAMED) {
                return Ok(GitStatus::Renamed);
            }
            if flags.contains(git2::Status::WT_TYPECHANGE) || flags.contains(git2::Status::INDEX_TYPECHANGE) {
                return Ok(GitStatus::TypeChange);
            }
            if flags.contains(git2::Status::IGNORED) {
                return Ok(GitStatus::Ignored);
            }
        }
    }
    
    Ok(GitStatus::Untracked)
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
            let a_ctime = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(a.metadata.ctime() as u64);
            let b_ctime = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(b.metadata.ctime() as u64);
            b_ctime.cmp(&a_ctime)
        } else if options.sort_by_atime {
            let a_atime = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(a.metadata.atime() as u64);
            let b_atime = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(b.metadata.atime() as u64);
            b_atime.cmp(&a_atime)
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
    let users_cache = UsersCache::new();
    
    // Calculate column widths
    let mut max_links = 0;
    let mut max_size = 0;
    let mut max_user = 0;
    let mut max_group = 0;
    
    for entry in entries {
        max_links = max_links.max(entry.metadata.nlink().to_string().len());
        max_size = max_size.max(format_file_size(entry.metadata.len(), options.human_readable).len());
        
        if !options.long_no_owner && !options.numeric_ids {
            let user_name = get_user_name(entry.metadata.uid(), &users_cache);
            max_user = max_user.max(user_name.len());
        }
        
        if !options.long_no_group && !options.no_group && !options.numeric_ids {
            let group_name = get_group_name(entry.metadata.gid(), &users_cache);
            max_group = max_group.max(group_name.len());
        }
    }
    
    for entry in entries {
        print_long_entry(entry, options, use_colors, &users_cache, max_links, max_size, max_user, max_group)?;
    }
    
    Ok(())
}

fn print_long_entry(
    entry: &FileInfo,
    options: &LsOptions,
    use_colors: bool,
    users_cache: &UsersCache,
    max_links: usize,
    max_size: usize,
    max_user: usize,
    max_group: usize,
) -> Result<()> {
    let mut line = String::new();
    
    // Inode number
    if options.show_inode {
        line.push_str(&format!("{:8} ", entry.metadata.ino()));
    }
    
    // Block size
    if options.show_size_blocks {
        let blocks = entry.metadata.blocks();
        line.push_str(&format!("{:6} ", blocks));
    }
    
    // File type and permissions
    line.push_str(&format_permissions(&entry.metadata));
    
    // Number of links
    line.push_str(&format!(" {:width$}", entry.metadata.nlink(), width = max_links));
    
    // Owner
    if !options.long_no_owner {
        let owner = if options.numeric_ids {
            entry.metadata.uid().to_string()
        } else {
            get_user_name(entry.metadata.uid(), users_cache)
        };
        line.push_str(&format!(" {:width$}", owner, width = max_user));
    }
    
    // Group
    if !options.long_no_group && !options.no_group {
        let group = if options.numeric_ids {
            entry.metadata.gid().to_string()
        } else {
            get_group_name(entry.metadata.gid(), users_cache)
        };
        line.push_str(&format!(" {:width$}", group, width = max_group));
    }
    
    // File size
    let size_str = format_file_size(entry.metadata.len(), options.human_readable);
    line.push_str(&format!(" {:>width$}", size_str, width = max_size));
    
    // Modification time
    let time_str = format_time(&entry.metadata, &options.time_style, options.full_time);
    line.push_str(&format!(" {}", time_str));
    
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
    
    println!("{}", line);
    
    Ok(())
}

fn print_short_format(entries: &[FileInfo], options: &LsOptions, use_colors: bool) -> Result<()> {
    if options.one_per_line {
        for entry in entries {
            let mut line = String::new();
            
            if options.show_inode {
                line.push_str(&format!("{:8} ", entry.metadata.ino()));
            }
            
            if options.show_size_blocks {
                let blocks = entry.metadata.blocks();
                line.push_str(&format!("{:6} ", blocks));
            }
            
            let colored_name = format_file_name(entry, use_colors, options.classify);
            line.push_str(&colored_name);
            
            println!("{}", line);
        }
    } else {
        // Multi-column output
        let term_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
        print_columns(entries, options, use_colors, term_width)?;
    }
    
    Ok(())
}

fn print_columns(entries: &[FileInfo], options: &LsOptions, use_colors: bool, term_width: usize) -> Result<()> {
    let mut names = Vec::new();
    let mut max_width = 0;
    
    for entry in entries {
        let mut name = String::new();
        
        if options.show_inode {
            name.push_str(&format!("{:8} ", entry.metadata.ino()));
        }
        
        if options.show_size_blocks {
            let blocks = entry.metadata.blocks();
            name.push_str(&format!("{:6} ", blocks));
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
    let rows = (names.len() + cols - 1) / cols;
    
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
        println!("{}", line);
    }
    
    Ok(())
}

fn format_permissions(metadata: &Metadata) -> String {
    let mode = metadata.permissions().mode();
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
        style = style.fg(Colour::Blue).bold();
    } else if entry.is_symlink {
        style = style.fg(Colour::Cyan);
    } else if is_executable(&entry.metadata) {
        style = style.fg(Colour::Green);
    } else {
        // Color by extension
        if let Some(ext) = entry.path.extension() {
            match ext.to_string_lossy().to_lowercase().as_str() {
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "ico" => {
                    style = style.fg(Colour::Purple);
                }
                "mp3" | "wav" | "flac" | "ogg" | "m4a" => {
                    style = style.fg(Colour::Cyan);
                }
                "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" => {
                    style = style.fg(Colour::Purple);
                }
                "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => {
                    style = style.fg(Colour::Red);
                }
                "txt" | "md" | "rst" | "doc" | "pdf" => {
                    style = style.fg(Colour::Yellow);
                }
                _ => {}
            }
        }
    }
    
    // Add git status indicator
    if let Some(ref git_status) = entry.git_status {
        let git_indicator = match git_status {
            GitStatus::Untracked => "?",
            GitStatus::Modified => "M",
            GitStatus::Added => "A",
            GitStatus::Deleted => "D",
            GitStatus::Renamed => "R",
            GitStatus::TypeChange => "T",
            GitStatus::Ignored => "!",
            GitStatus::Conflicted => "U",
        };
        
        let git_color = match git_status {
            GitStatus::Untracked => Colour::Red,
            GitStatus::Modified => Colour::Yellow,
            GitStatus::Added => Colour::Green,
            GitStatus::Deleted => Colour::Red,
            GitStatus::Renamed => Colour::Blue,
            GitStatus::TypeChange => Colour::Purple,
            GitStatus::Ignored => Colour::Fixed(8), // Dark gray
            GitStatus::Conflicted => Colour::Red,
        };
        
        return format!("{} {}", 
            git_color.paint(git_indicator),
            style.paint(name)
        );
    }
    
    style.paint(name).to_string()
}

fn get_user_name(uid: u32, users_cache: &UsersCache) -> String {
    users_cache.get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().to_string())
        .unwrap_or_else(|| uid.to_string())
}

fn get_group_name(gid: u32, users_cache: &UsersCache) -> String {
    users_cache.get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().to_string())
        .unwrap_or_else(|| gid.to_string())
}

fn is_executable(metadata: &Metadata) -> bool {
    #[cfg(unix)]
    {
        metadata.permissions().mode() & 0o111 != 0
    }
    
    #[cfg(not(unix))]
    {
        false // Windows doesn't have the same concept
    }
}