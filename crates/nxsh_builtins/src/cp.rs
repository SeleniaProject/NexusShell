//! `cp` command  Ecopy files and directories.
//! Supported syntax:
//!   cp SRC DST
//!   cp -r SRC_DIR DST_DIR
//!   cp -p SRC DST (preserve permissions and timestamps)
//!   cp -v SRC DST (verbose output)

use anyhow::{Result, anyhow, Context};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::io::{self, Write};
use tracing::{info, debug, warn};
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt; // for .custom_flags()

// Progress tracking for large operations
struct ProgressTracker {
    total_files: u64,
    processed_files: u64,
    show_progress: bool,
}

impl ProgressTracker {
    fn new(total_files: u64, show_progress: bool) -> Self {
        Self {
            total_files,
            processed_files: 0,
            show_progress,
        }
    }

    fn increment(&mut self) {
        self.processed_files += 1;
        if self.show_progress && self.total_files > 0 {
            let percentage = (self.processed_files * 100) / self.total_files;
            print!("\rCopying files: {}/{} ({}%)", self.processed_files, self.total_files, percentage);
            io::stdout().flush().unwrap_or(());
        }
    }

    fn finish(&self) {
        if self.show_progress {
            println!("\nCopy completed: {} files processed", self.processed_files);
        }
    }
}

/// Copy options for controlling behavior
#[derive(Debug, Default)]
struct CopyOptions {
    recursive: bool,
    preserve: bool,
    verbose: bool,
    show_progress: bool,
}

// In super-min (size focused) build we compile a synchronous version to avoid pulling async runtime.
#[cfg(feature = "super-min")]
pub fn cp_cli(args: &[String]) -> Result<()> {
    cp_impl(args)
}

// Default (non super-min) build keeps async for potential future async optimizations;
// we keep the original signature but internally call the same sync implementation to
// simplify and allow gating out Tokio entirely when async-runtime feature is absent.
#[cfg(not(feature = "super-min"))]
pub async fn cp_cli(args: &[String]) -> Result<()> {
    cp_impl(args)
}

// Shared implementation (pure synchronous) used by both variants.
fn cp_impl(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("cp: missing operands"));
    }

    let mut options = CopyOptions::default();
    // First collect all non-flag operands, then validate count and split into sources/destination
    let mut operands: Vec<String> = Vec::new();

    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 {
            // Parse short flags possibly combined (e.g., -rpv)
            for ch in arg.chars().skip(1) {
                match ch {
                    'r' | 'R' => options.recursive = true,
                    'p' => options.preserve = true,
                    'v' => options.verbose = true,
                    _ => return Err(anyhow!("cp: invalid option -- '{}'", ch)),
                }
            }
        } else {
            operands.push(arg.clone());
        }
    }

    if operands.is_empty() {
        return Err(anyhow!("cp: missing file operand"));
    }

    if operands.len() == 1 {
        return Err(anyhow!("cp: missing destination file operand"));
    }

    // Split operands into sources and destination
    let destination = operands.last().cloned().unwrap();
    let sources = operands[..operands.len() - 1].to_vec();
    
    let dst_path = PathBuf::from(&destination);

    // Check if destination should be a directory when copying multiple sources
    if sources.len() > 1 && !dst_path.is_dir() {
        return Err(anyhow!("cp: target '{}' is not a directory", destination));
    }

    // Enable progress bar for large operations
    options.show_progress = should_show_progress(&sources, &options)?;

    // Process each source
    for source in sources {
        let src_path = Path::new(&source);
        
        if !src_path.exists() {
            return Err(anyhow!("cp: cannot stat '{}': No such file or directory", source));
        }

        let target_path = if dst_path.is_dir() {
            dst_path.join(src_path.file_name()
                .ok_or_else(|| anyhow!("cp: invalid source path '{}'", source))?)
        } else {
            dst_path.clone()
        };

        if src_path.is_dir() {
            if !options.recursive {
                return Err(anyhow!("cp: -r not specified; omitting directory '{}'", source));
            }
            copy_directory_with_progress(src_path, &target_path, &options)
                .with_context(|| format!("Failed to copy directory '{}' to '{}'", source, target_path.display()))?;
        } else {
            copy_file_with_metadata(src_path, &target_path, &options)
                .with_context(|| format!("Failed to copy file '{}' to '{}'", source, target_path.display()))?;
        }

        if options.verbose {
            info!("'{}' -> '{}'", source, target_path.display());
        }
    }

    Ok(())
}

/// Determine if progress bar should be shown based on operation size
fn should_show_progress(sources: &[String], options: &CopyOptions) -> Result<bool> {
    if !options.recursive {
        return Ok(false);
    }

    let mut total_files = 0;
    for source in sources {
        let src_path = Path::new(source);
        if src_path.is_dir() {
            total_files += count_files_recursively(src_path)?;
        } else {
            total_files += 1;
        }
    }

    // Show progress bar if copying more than 100 files
    Ok(total_files > 100)
}

/// Count files recursively in a directory
fn count_files_recursively(dir: &Path) -> Result<u64> {
    let mut count = 0;
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory '{}'", dir.display()))?;

    for entry in entries {
        let entry = entry
            .with_context(|| format!("Failed to read directory entry in '{}'", dir.display()))?;
        
        let file_type = entry.file_type()
            .with_context(|| format!("Failed to get file type for '{}'", entry.path().display()))?;
        
        if file_type.is_dir() {
            count += count_files_recursively(&entry.path())?;
        } else if file_type.is_file() {
            count += 1;
        }
    }

    Ok(count)
}

/// Copy a single file with metadata preservation if requested
fn copy_file_with_metadata(src: &Path, dst: &Path, options: &CopyOptions) -> Result<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory '{}'", parent.display()))?;
    }

    // Copy the file content
    fs::copy(src, dst)
        .with_context(|| format!("Failed to copy file content from '{}' to '{}'", src.display(), dst.display()))?;

    // Preserve metadata if requested
    if options.preserve {
        preserve_metadata(src, dst)
            .with_context(|| format!("Failed to preserve metadata for '{}'", dst.display()))?;
    }

    debug!("Copied file: {} -> {}", src.display(), dst.display());
    Ok(())
}

/// Copy directory with progress tracking
fn copy_directory_with_progress(src: &Path, dst: &Path, options: &CopyOptions) -> Result<()> {
    if options.show_progress {
        copy_dir_with_progress_bar(src, dst, options)
    } else {
        copy_dir_recursively(src, dst, options)
    }
}

/// Enhanced recursive directory copy with metadata preservation
fn copy_dir_recursively(src: &Path, dst: &Path, options: &CopyOptions) -> Result<()> {
    // Create destination directory
    fs::create_dir_all(dst)
        .with_context(|| format!("Failed to create directory '{}'", dst.display()))?;

    // Preserve directory metadata if requested
    if options.preserve {
        preserve_metadata(src, dst)
            .with_context(|| format!("Failed to preserve directory metadata for '{}'", dst.display()))?;
    }

    // Read directory entries
    let entries = fs::read_dir(src)
        .with_context(|| format!("Failed to read directory '{}'", src.display()))?;

    for entry in entries {
        let entry = entry
            .with_context(|| format!("Failed to read directory entry in '{}'", src.display()))?;
        
        let file_type = entry.file_type()
            .with_context(|| format!("Failed to get file type for '{}'", entry.path().display()))?;
        
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursively(&src_path, &dst_path, options)
                .with_context(|| format!("Failed to copy subdirectory '{}' to '{}'", src_path.display(), dst_path.display()))?;
        } else if file_type.is_file() {
            copy_file_with_metadata(&src_path, &dst_path, options)
                .with_context(|| format!("Failed to copy file '{}' to '{}'", src_path.display(), dst_path.display()))?;
        } else if file_type.is_symlink() {
            copy_symlink(&src_path, &dst_path)
                .with_context(|| format!("Failed to copy symlink '{}' to '{}'", src_path.display(), dst_path.display()))?;
        } else {
            warn!("Skipping special file: {}", src_path.display());
        }
    }

    debug!("Copied directory: {} -> {}", src.display(), dst.display());
    Ok(())
}

/// Copy directory with progress bar
fn copy_dir_with_progress_bar(src: &Path, dst: &Path, options: &CopyOptions) -> Result<()> {
    // Count total files first
    let total_files = count_files_recursively(src)?;

    if total_files == 0 {
        return copy_dir_recursively(src, dst, options);
    }

    // Create progress tracker
    let mut progress = ProgressTracker::new(total_files, true);

    // Copy with progress tracking
    copy_dir_with_progress_tracking(src, dst, options, &mut progress)?;
    
    progress.finish();
    Ok(())
}

/// Recursive copy with progress tracking
fn copy_dir_with_progress_tracking(src: &Path, dst: &Path, options: &CopyOptions, progress: &mut ProgressTracker) -> Result<()> {
    // Create destination directory
    fs::create_dir_all(dst)
        .with_context(|| format!("Failed to create directory '{}'", dst.display()))?;

    // Preserve directory metadata if requested
    if options.preserve {
        preserve_metadata(src, dst)
            .with_context(|| format!("Failed to preserve directory metadata for '{}'", dst.display()))?;
    }

    // Read directory entries
    let entries = fs::read_dir(src)
        .with_context(|| format!("Failed to read directory '{}'", src.display()))?;

    for entry in entries {
        let entry = entry
            .with_context(|| format!("Failed to read directory entry in '{}'", src.display()))?;
        
        let file_type = entry.file_type()
            .with_context(|| format!("Failed to get file type for '{}'", entry.path().display()))?;
        
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_with_progress_tracking(&src_path, &dst_path, options, progress)
                .with_context(|| format!("Failed to copy subdirectory '{}' to '{}'", src_path.display(), dst_path.display()))?;
        } else if file_type.is_file() {
            copy_file_with_metadata(&src_path, &dst_path, options)
                .with_context(|| format!("Failed to copy file '{}' to '{}'", src_path.display(), dst_path.display()))?;
            progress.increment();
        } else if file_type.is_symlink() {
            copy_symlink(&src_path, &dst_path)
                .with_context(|| format!("Failed to copy symlink '{}' to '{}'", src_path.display(), dst_path.display()))?;
        } else {
            eprintln!("Warning: Skipping special file: {}", src_path.display());
        }
    }

    Ok(())
}

/// Copy a symbolic link
fn copy_symlink(src: &Path, dst: &Path) -> Result<()> {
    let target = fs::read_link(src)
        .with_context(|| format!("Failed to read symlink '{}'", src.display()))?;
    
    // Remove destination if it exists
    if dst.exists() {
        fs::remove_file(dst)
            .with_context(|| format!("Failed to remove existing file '{}'", dst.display()))?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target, dst)
            .with_context(|| format!("Failed to create symlink '{}' -> '{}'", dst.display(), target.display()))?;
    }

    #[cfg(windows)]
    {
        if target.is_dir() {
            std::os::windows::fs::symlink_dir(&target, dst)
                .with_context(|| format!("Failed to create directory symlink '{}' -> '{}'", dst.display(), target.display()))?;
        } else {
            std::os::windows::fs::symlink_file(&target, dst)
                .with_context(|| format!("Failed to create file symlink '{}' -> '{}'", dst.display(), target.display()))?;
        }
    }

    debug!("Copied symlink: {} -> {} (target: {})", src.display(), dst.display(), target.display());
    Ok(())
}

/// Preserve file/directory metadata (permissions, timestamps)
fn preserve_metadata(src: &Path, dst: &Path) -> Result<()> {
    let metadata = fs::metadata(src)
        .with_context(|| format!("Failed to read metadata for '{}'", src.display()))?;

    // Preserve timestamps
    let accessed = metadata.accessed()
        .with_context(|| format!("Failed to get access time for '{}'", src.display()))?;
    let modified = metadata.modified()
        .with_context(|| format!("Failed to get modification time for '{}'", src.display()))?;

    // Set timestamps on destination
    set_file_times(dst, accessed, modified)
        .with_context(|| format!("Failed to set timestamps for '{}'", dst.display()))?;

    // Preserve permissions on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        
        let dst_permissions = std::fs::Permissions::from_mode(mode);
        fs::set_permissions(dst, dst_permissions)
            .with_context(|| format!("Failed to set permissions for '{}'", dst.display()))?;
    }

    debug!("Preserved metadata for: {}", dst.display());
    Ok(())
}

/// Set file access and modification times
fn set_file_times(path: &Path, accessed: SystemTime, modified: SystemTime) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        use nix::libc::{utimensat, timespec, AT_FDCWD};
        use std::ffi::CString;
        use std::time::UNIX_EPOCH;

        let path_cstr = CString::new(path.as_os_str().to_string_lossy().as_ref())
            .map_err(|e| anyhow!("Invalid path for timestamp setting: {}", e))?;

        let accessed_duration = accessed.duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("Invalid access time: {}", e))?;
        let modified_duration = modified.duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("Invalid modification time: {}", e))?;

        let times = [
            timespec {
                tv_sec: accessed_duration.as_secs() as i64,
                tv_nsec: accessed_duration.subsec_nanos() as i64,
            },
            timespec {
                tv_sec: modified_duration.as_secs() as i64,
                tv_nsec: modified_duration.subsec_nanos() as i64,
            },
        ];

        let result = unsafe {
            utimensat(AT_FDCWD, path_cstr.as_ptr(), times.as_ptr(), 0)
        };

        if result != 0 {
            return Err(anyhow!("Failed to set file times: {}", std::io::Error::last_os_error()));
        }
    }

    #[cfg(windows)]
    {
        // Windows implementation using SetFileTime
        use std::fs::OpenOptions;
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::{
            Foundation::FILETIME,
            Storage::FileSystem::SetFileTime,
        };
        use std::time::UNIX_EPOCH;

        fn to_filetime(t: SystemTime) -> FILETIME {
            let dur = t.duration_since(UNIX_EPOCH).unwrap_or_default();
            let mut intervals = dur.as_secs() * 10_000_000 + (dur.subsec_nanos() as u64) / 100;
            intervals += 11644473600u64 * 10_000_000; // Unix -> Windows epoch offset
            FILETIME { dwLowDateTime: intervals as u32, dwHighDateTime: (intervals >> 32) as u32 }
        }

        let at = to_filetime(accessed);
        let mt = to_filetime(modified);
        let file = OpenOptions::new()
            .write(true)
            // FILE_FLAG_BACKUP_SEMANTICS (0x02000000) is needed for directories; open without creation
            .custom_flags(0x02000000)
            .open(path)
            .with_context(|| format!("Failed to open file for timestamp setting: {}", path.display()))?;
        let handle = file.as_raw_handle();
        unsafe {
            let _ = SetFileTime(handle as _, std::ptr::null(), &at as *const FILETIME, &mt as *const FILETIME);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;
    use std::fs::File;

    // Helper to invoke cp_cli in both async (non super-min) and sync (super-min) modes
    #[cfg(not(feature = "super-min"))]
    fn run(args: &[String]) -> Result<()> { futures::executor::block_on(cp_cli(args)) }
    #[cfg(feature = "super-min")]
    fn run(args: &[String]) -> Result<()> { cp_cli(args) }

    #[tokio::test]
    async fn copy_single_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("a.txt");
        let dst = dir.path().join("b.txt");
        
        let mut f = File::create(&src).unwrap();
        writeln!(f, "hello world").unwrap();
        
    run(&[src.to_string_lossy().into(), dst.to_string_lossy().into()]).unwrap();
        
        assert!(dst.exists());
        let content = fs::read_to_string(&dst).unwrap();
        assert_eq!(content, "hello world\n");
    }

    #[tokio::test]
    async fn copy_file_with_preserve() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("dest.txt");
        
        let mut f = File::create(&src).unwrap();
        writeln!(f, "test content").unwrap();
        
        // Copy with preserve flag
    run(&["-p".to_string(), src.to_string_lossy().into(), dst.to_string_lossy().into()]).unwrap();
        
        assert!(dst.exists());
        let content = fs::read_to_string(&dst).unwrap();
        assert_eq!(content, "test content\n");
        
        // Check that metadata is preserved (at least modification time should be similar)
        let src_meta = fs::metadata(&src).unwrap();
        let dst_meta = fs::metadata(&dst).unwrap();
        
        // Allow for small differences in timestamps due to filesystem precision
        let src_modified = src_meta.modified().unwrap();
        let dst_modified = dst_meta.modified().unwrap();
        let diff = if src_modified > dst_modified {
            src_modified.duration_since(dst_modified).unwrap()
        } else {
            dst_modified.duration_since(src_modified).unwrap()
        };
        assert!(diff.as_secs() < 2, "Timestamps should be preserved within 2 seconds");
    }

    #[tokio::test]
    async fn copy_directory_recursive() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("source");
        let dst_dir = dir.path().join("destination");
        
        // Create source directory structure
        fs::create_dir_all(&src_dir).unwrap();
        fs::create_dir_all(src_dir.join("subdir")).unwrap();
        
        let mut f1 = File::create(src_dir.join("file1.txt")).unwrap();
        writeln!(f1, "content1").unwrap();
        
        let mut f2 = File::create(src_dir.join("subdir").join("file2.txt")).unwrap();
        writeln!(f2, "content2").unwrap();
        
        // Copy recursively
    run(&["-r".to_string(), src_dir.to_string_lossy().into(), dst_dir.to_string_lossy().into()]).unwrap();
        
        // Verify structure was copied
        assert!(dst_dir.exists());
        assert!(dst_dir.join("file1.txt").exists());
        assert!(dst_dir.join("subdir").exists());
        assert!(dst_dir.join("subdir").join("file2.txt").exists());
        
        // Verify content
        let content1 = fs::read_to_string(dst_dir.join("file1.txt")).unwrap();
        assert_eq!(content1, "content1\n");
        
        let content2 = fs::read_to_string(dst_dir.join("subdir").join("file2.txt")).unwrap();
        assert_eq!(content2, "content2\n");
    }

    #[tokio::test]
    async fn copy_directory_without_recursive_flag_fails() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("source");
        let dst_dir = dir.path().join("destination");
        
        fs::create_dir_all(&src_dir).unwrap();
        
        // Should fail without -r flag
    let result = run(&[src_dir.to_string_lossy().into(), dst_dir.to_string_lossy().into()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("-r not specified"));
    }

    #[tokio::test]
    async fn copy_multiple_files_to_directory() {
        let dir = tempdir().unwrap();
        let dst_dir = dir.path().join("destination");
        fs::create_dir_all(&dst_dir).unwrap();
        
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        
        let mut f1 = File::create(&file1).unwrap();
        writeln!(f1, "content1").unwrap();
        
        let mut f2 = File::create(&file2).unwrap();
        writeln!(f2, "content2").unwrap();
        
        // Copy multiple files to directory
    run(&[
            file1.to_string_lossy().into(),
            file2.to_string_lossy().into(),
            dst_dir.to_string_lossy().into()
    ]).unwrap();
        
        // Verify both files were copied
        assert!(dst_dir.join("file1.txt").exists());
        assert!(dst_dir.join("file2.txt").exists());
        
        let content1 = fs::read_to_string(dst_dir.join("file1.txt")).unwrap();
        assert_eq!(content1, "content1\n");
        
        let content2 = fs::read_to_string(dst_dir.join("file2.txt")).unwrap();
        assert_eq!(content2, "content2\n");
    }

    #[tokio::test]
    async fn copy_nonexistent_file_fails() {
        let dir = tempdir().unwrap();
        let nonexistent = dir.path().join("nonexistent.txt");
        let dst = dir.path().join("destination.txt");
        
    let result = run(&[nonexistent.to_string_lossy().into(), dst.to_string_lossy().into()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No such file or directory"));
    }

    #[tokio::test]
    async fn copy_with_verbose_flag() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("dest.txt");
        
        let mut f = File::create(&src).unwrap();
        writeln!(f, "test").unwrap();
        
        // This test mainly ensures the verbose flag is parsed correctly
        // In a real implementation, we'd capture log output to verify verbose messages
    run(&["-v".to_string(), src.to_string_lossy().into(), dst.to_string_lossy().into()]).unwrap();
        
        assert!(dst.exists());
    }

    #[tokio::test]
    async fn copy_with_combined_flags() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("source");
        let dst_dir = dir.path().join("destination");
        
        fs::create_dir_all(&src_dir).unwrap();
        let mut f = File::create(src_dir.join("test.txt")).unwrap();
        writeln!(f, "test content").unwrap();
        
        // Test combined flags -rpv
    run(&["-rpv".to_string(), src_dir.to_string_lossy().into(), dst_dir.to_string_lossy().into()]).unwrap();
        
        assert!(dst_dir.exists());
        assert!(dst_dir.join("test.txt").exists());
        
        let content = fs::read_to_string(dst_dir.join("test.txt")).unwrap();
        assert_eq!(content, "test content\n");
    }

    #[tokio::test]
    async fn invalid_option_fails() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("dest.txt");
        
        let mut f = File::create(&src).unwrap();
        writeln!(f, "test").unwrap();
        
    let result = run(&["-x".to_string(), src.to_string_lossy().into(), dst.to_string_lossy().into()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid option"));
    }

    #[tokio::test]
    async fn missing_operands_fails() {
    let result = run(&[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing operands"));
    }

    #[tokio::test]
    async fn missing_destination_fails() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let mut f = File::create(&src).unwrap();
        writeln!(f, "test").unwrap();
        
    let result = run(&[src.to_string_lossy().into()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing destination"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn copy_symlink() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("target.txt");
        let link = dir.path().join("link.txt");
        let dst_dir = dir.path().join("destination");
        
        // Create target file and symlink
        let mut f = File::create(&target).unwrap();
        writeln!(f, "target content").unwrap();
        
        std::os::unix::fs::symlink(&target, &link).unwrap();
        fs::create_dir_all(&dst_dir).unwrap();
        
        // Copy directory containing symlink
        let src_dir = dir.path().join("source");
        fs::create_dir_all(&src_dir).unwrap();
        
        let link_in_src = src_dir.join("link.txt");
        std::os::unix::fs::symlink(&target, &link_in_src).unwrap();
        
    run(&["-r".to_string(), src_dir.to_string_lossy().into(), dst_dir.to_string_lossy().into()]).unwrap();
        
        let copied_link = dst_dir.join("source").join("link.txt");
        assert!(copied_link.exists());
        assert!(copied_link.is_symlink());
    }
} 
