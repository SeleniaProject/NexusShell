//! File system abstraction layer
//!
//! This module provides a comprehensive, platform-agnostic interface to
//! file system operations with optimizations for each supported platform.

use std::fs::{self, File, Permissions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::ffi::OsString;
use std::time::SystemTime;

use crate::error::{HalError, HalResult};
use crate::platform::{Platform, Capabilities};

/// High-level file system interface
#[derive(Debug)]
pub struct FileSystem {
    platform: Platform,
    capabilities: Capabilities,
}

impl FileSystem {
    /// Create a new file system interface
    pub fn new() -> HalResult<Self> {
        Ok(Self {
            platform: Platform::current(),
            capabilities: Capabilities::current(),
        })
    }

    /// Open a file with specified options
    pub fn open<P: AsRef<Path>>(&self, path: P, options: &HalOpenOptions) -> HalResult<FileHandle> {
        let path = path.as_ref();
        
        // Validate path length
        if path.as_os_str().len() > self.capabilities.max_path_length {
            return Err(HalError::invalid(&format!(
                "Path too long: {} > {}", 
                path.as_os_str().len(), 
                self.capabilities.max_path_length
            )));
        }

        let file = options.open(path)
            .map_err(|e| HalError::io_error("open", Some(path.to_str().unwrap_or("<invalid>")), e))?;

        Ok(FileHandle::new(file, path.to_path_buf(), options.clone()))
    }

    /// Create a directory with all parent directories
    pub fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> HalResult<()> {
        let path = path.as_ref();
        fs::create_dir_all(path)
            .map_err(|e| HalError::io_error("create_dir_all", Some(path.to_str().unwrap_or("<invalid>")), e))?;
        Ok(())
    }

    /// Remove a file
    pub fn remove_file<P: AsRef<Path>>(&self, path: P) -> HalResult<()> {
        let path = path.as_ref();
        fs::remove_file(path)
            .map_err(|e| HalError::io_error("remove_file", Some(path.to_str().unwrap_or("<invalid>")), e))?;
        Ok(())
    }

    /// Remove a directory and all its contents
    pub fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> HalResult<()> {
        let path = path.as_ref();
        fs::remove_dir_all(path)
            .map_err(|e| HalError::io_error("remove_dir_all", Some(path.to_str().unwrap_or("<invalid>")), e))?;
        Ok(())
    }

    /// Copy a file with platform-specific optimizations
    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> HalResult<u64> {
        let from = from.as_ref();
        let to = to.as_ref();

        // Use platform-specific optimizations when available
        match self.platform {
            Platform::Linux => self.copy_generic(from, to), // TODO: Implement copy_linux
            Platform::MacOS => self.copy_generic(from, to), // TODO: Implement copy_macos
            Platform::Windows => self.copy_generic(from, to), // TODO: Implement copy_windows
            _ => self.copy_generic(from, to),
        }
    }

    /// Get file metadata
    pub fn metadata<P: AsRef<Path>>(&self, path: P) -> HalResult<FileMetadata> {
        let path = path.as_ref();
        let metadata = fs::metadata(path)
            .map_err(|e| HalError::io_error("metadata", Some(path.to_str().unwrap_or("<invalid>")), e))?;
        
        Ok(FileMetadata::from_std(metadata, path))
    }

    /// Check if a path exists
    pub fn exists<P: AsRef<Path>>(&self, path: P) -> HalResult<bool> {
        let path = path.as_ref();
        Ok(path.exists())
    }

    /// Get the canonical absolute path
    pub fn canonicalize<P: AsRef<Path>>(&self, path: P) -> HalResult<PathBuf> {
        let path = path.as_ref();
        path.canonicalize()
            .map_err(|e| HalError::io_error("canonicalize", Some(path.to_str().unwrap_or("<invalid>")), e))
    }

    /// Create a hard link
    pub fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(&self, original: P, link: Q) -> HalResult<()> {
        let original = original.as_ref();
        let link = link.as_ref();

        if !self.capabilities.filesystem_feature("hard_links") {
            return Err(HalError::unsupported("Hard links not supported on this platform"));
        }

        fs::hard_link(original, link)
            .map_err(|e| HalError::io_error("hard_link", Some(link.to_str().unwrap_or("<invalid>")), e))?;
        Ok(())
    }

    /// Create a symbolic link
    pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(&self, original: P, link: Q) -> HalResult<()> {
        let original = original.as_ref();
        let link = link.as_ref();

        if !self.capabilities.filesystem_feature("symbolic_links") {
            return Err(HalError::unsupported("Symbolic links not supported on this platform"));
        }

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(original, link)
                .map_err(|e| HalError::io_error("symlink", Some(link.to_str().unwrap_or("<invalid>")), e))?;
        }

        #[cfg(windows)]
        {
            if original.is_dir() {
                std::os::windows::fs::symlink_dir(original, link)
                    .map_err(|e| HalError::io_error("symlink_dir", Some(link.to_str().unwrap_or("<invalid>")), e))?;
            } else {
                std::os::windows::fs::symlink_file(original, link)
                    .map_err(|e| HalError::io_error("symlink_file", Some(link.to_str().unwrap_or("<invalid>")), e))?;
            }
        }

        Ok(())
    }

    /// Read a symbolic link
    pub fn read_link<P: AsRef<Path>>(&self, path: P) -> HalResult<PathBuf> {
        let path = path.as_ref();
        fs::read_link(path)
            .map_err(|e| HalError::io_error("read_link", Some(path.to_str().unwrap_or("<invalid>")), e))
    }

    /// Set file permissions
    pub fn set_permissions<P: AsRef<Path>>(&self, path: P, permissions: Permissions) -> HalResult<()> {
        let path = path.as_ref();
        fs::set_permissions(path, permissions)
            .map_err(|e| HalError::io_error("set_permissions", Some(path.to_str().unwrap_or("<invalid>")), e))?;
        Ok(())
    }

    /// Rename/move a file or directory
    pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> HalResult<()> {
        let from = from.as_ref();
        let to = to.as_ref();
        fs::rename(from, to)
            .map_err(|e| HalError::io_error("rename", Some(to.to_str().unwrap_or("<invalid>")), e))?;
        Ok(())
    }

    /// Get available disk space
    pub fn disk_usage<P: AsRef<Path>>(&self, path: P) -> HalResult<DiskUsage> {
        let path = path.as_ref();
        
        #[cfg(unix)]
        {
            use std::ffi::CString;
            use std::mem;

            let path_cstr = CString::new(path.as_os_str().to_string_lossy().as_bytes())
                .map_err(|_| HalError::invalid("Invalid path"))?;
            let mut statfs: libc::statvfs = unsafe { mem::zeroed() };
            
            let result = unsafe { libc::statvfs(path_cstr.as_ptr(), &mut statfs) };
            if result != 0 {
                return Err(HalError::io_error("statvfs", Some(path.to_str().unwrap_or("<invalid>")), 
                                             io::Error::last_os_error()));
            }

            Ok(DiskUsage {
                total: statfs.f_blocks * statfs.f_frsize,
                free: statfs.f_bavail * statfs.f_frsize,
                available: statfs.f_bavail * statfs.f_frsize,
            })
        }

        #[cfg(windows)]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

            let path_wide: Vec<u16> = OsStr::new(path).encode_wide().chain(Some(0)).collect();
            let mut free_bytes = 0u64;
            let mut total_bytes = 0u64;
            let mut available_bytes = 0u64;

            let result = unsafe {
                GetDiskFreeSpaceExW(
                    path_wide.as_ptr(),
                    &mut available_bytes,
                    &mut total_bytes,
                    &mut free_bytes,
                )
            };

            if result == 0 {
                return Err(HalError::io_error("GetDiskFreeSpaceExW", Some(path.to_str().unwrap_or("<invalid>")), 
                                             io::Error::last_os_error()));
            }

            Ok(DiskUsage {
                total: total_bytes,
                free: free_bytes,
                available: available_bytes,
            })
        }

        #[cfg(not(any(unix, windows)))]
        {
            Err(HalError::unsupported("Disk usage not supported on this platform"))
        }
    }

    // Platform-specific copy implementations
    #[cfg(target_os = "linux")]
    fn copy_linux(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // Try copy_file_range for efficiency on Linux
        if self.capabilities.has_copy_file_range {
            match self.copy_with_copy_file_range(from, to) {
                Ok(bytes) => return Ok(bytes),
                Err(_) => {
                    // Fall back to sendfile or generic copy
                }
            }
        }

        if self.capabilities.has_sendfile {
            match self.copy_with_sendfile(from, to) {
                Ok(bytes) => return Ok(bytes),
                Err(_) => {
                    // Fall back to generic copy
                }
            }
        }

        self.copy_generic(from, to)
    }

    #[cfg(target_os = "macos")]
    fn copy_macos(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // Try copyfile API on macOS for efficiency
        match self.copy_with_copyfile(from, to) {
            Ok(bytes) => return Ok(bytes),
            Err(_) => {
                // Fall back to generic copy
            }
        }

        self.copy_generic(from, to)
    }

    #[cfg(target_os = "windows")]
    #[allow(dead_code)]
    fn copy_windows(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // Use CopyFileEx on Windows for efficiency
        match self.copy_with_copyfileex(from, to) {
            Ok(bytes) => return Ok(bytes),
            Err(_) => {
                // Fall back to generic copy
            }
        }

        self.copy_generic(from, to)
    }

    fn copy_generic(&self, from: &Path, to: &Path) -> HalResult<u64> {
        fs::copy(from, to)
            .map_err(|e| HalError::io_error("copy", Some(to.to_str().unwrap_or("<invalid>")), e))
    }

    // Platform-specific optimized copy methods would be implemented here
    #[cfg(target_os = "linux")]
    fn copy_with_copy_file_range(&self, _from: &Path, _to: &Path) -> HalResult<u64> {
        // Implementation would use copy_file_range system call
        Err(HalError::unsupported("copy_file_range not yet implemented"))
    }

    #[cfg(target_os = "linux")]
    fn copy_with_sendfile(&self, _from: &Path, _to: &Path) -> HalResult<u64> {
        // Implementation would use sendfile system call
        Err(HalError::unsupported("sendfile not yet implemented"))
    }

    #[cfg(target_os = "macos")]
    fn copy_with_copyfile(&self, _from: &Path, _to: &Path) -> HalResult<u64> {
        // Implementation would use copyfile API
        Err(HalError::unsupported("copyfile not yet implemented"))
    }

    #[cfg(target_os = "windows")]
    #[allow(dead_code)]
    fn copy_with_copyfileex(&self, _from: &Path, _to: &Path) -> HalResult<u64> {
        // Implementation would use CopyFileEx API
        Err(HalError::unsupported("CopyFileEx not yet implemented"))
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new().unwrap() // Changed to unwrap() as new() returns HalResult
    }
}

/// File handle with enhanced capabilities
pub struct FileHandle {
    file: fs::File,
    path: PathBuf,
    options: HalOpenOptions,
}

impl FileHandle {
    fn new(file: fs::File, path: PathBuf, options: HalOpenOptions) -> Self {
        Self { file, path, options }
    }

    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get file metadata
    pub fn metadata(&self) -> HalResult<FileMetadata> {
        let metadata = self.file.metadata()
            .map_err(|e| HalError::io_error("metadata", Some(self.path.to_str().unwrap_or("<invalid>")), e))?;
        Ok(FileMetadata::from_std(metadata, &self.path))
    }

    /// Sync all data to disk
    pub fn sync_all(&mut self) -> HalResult<()> {
        self.file.sync_all()
            .map_err(|e| HalError::io_error("sync_all", Some(self.path.to_str().unwrap_or("<invalid>")), e))?;
        Ok(())
    }

    /// Sync data (but not metadata) to disk
    pub fn sync_data(&mut self) -> HalResult<()> {
        self.file.sync_data()
            .map_err(|e| HalError::io_error("sync_data", Some(self.path.to_str().unwrap_or("<invalid>")), e))?;
        Ok(())
    }

    /// Set file length
    pub fn set_len(&mut self, size: u64) -> HalResult<()> {
        self.file.set_len(size)
            .map_err(|e| HalError::io_error("set_len", Some(self.path.to_str().unwrap_or("<invalid>")), e))?;
        Ok(())
    }

    /// Try to clone the file handle
    pub fn try_clone(&self) -> HalResult<FileHandle> {
        let cloned_file = self.file.try_clone()
            .map_err(|e| HalError::io_error("try_clone", Some(self.path.to_str().unwrap_or("<invalid>")), e))?;
        Ok(FileHandle::new(cloned_file, self.path.clone(), self.options.clone()))
    }
}

impl Read for FileHandle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Write for FileHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Seek for FileHandle {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}

/// Directory handle for efficient directory operations
pub struct DirectoryHandle {
    path: PathBuf,
}

impl DirectoryHandle {
    pub fn open<P: AsRef<Path>>(path: P) -> HalResult<Self> {
        let path = path.as_ref().to_path_buf();
        if !path.is_dir() {
            return Err(HalError::invalid("Path is not a directory"));
        }
        Ok(Self { path })
    }

    pub fn read_dir(&self) -> HalResult<Vec<DirEntry>> {
        let entries = fs::read_dir(&self.path)
            .map_err(|e| HalError::io_error("read_dir", Some(self.path.to_str().unwrap_or("<invalid>")), e))?;

        let mut result = Vec::new();
        for entry in entries {
            let entry = entry
                .map_err(|e| HalError::io_error("read_dir_entry", Some(self.path.to_str().unwrap_or("<invalid>")), e))?;
            result.push(DirEntry::from_std(entry)?);
        }

        Ok(result)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Enhanced file metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub permissions: Permissions,
    pub modified: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub path: PathBuf,
    #[cfg(unix)]
    pub mode: u32,
    #[cfg(unix)]
    pub uid: u32,
    #[cfg(unix)]
    pub gid: u32,
    #[cfg(unix)]
    pub inode: u64,
    #[cfg(unix)]
    pub device: u64,
    #[cfg(unix)]
    pub nlink: u64,
    #[cfg(unix)]
    pub blocks: u64,
    #[cfg(unix)]
    pub block_size: u64,
}

impl FileMetadata {
    fn from_std(metadata: fs::Metadata, path: &Path) -> Self {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            Self {
                size: metadata.len(),
                is_file: metadata.is_file(),
                is_dir: metadata.is_dir(),
                is_symlink: metadata.is_symlink(),
                permissions: metadata.permissions(),
                modified: metadata.modified().ok(),
                accessed: metadata.accessed().ok(),
                created: metadata.created().ok(),
                path: path.to_path_buf(),
                mode: metadata.mode(),
                uid: metadata.uid(),
                gid: metadata.gid(),
                inode: metadata.ino(),
                device: metadata.dev(),
                nlink: metadata.nlink(),
                blocks: metadata.blocks(),
                block_size: metadata.blksize(),
            }
        }

        #[cfg(not(unix))]
        {
            Self {
                size: metadata.len(),
                is_file: metadata.is_file(),
                is_dir: metadata.is_dir(),
                is_symlink: metadata.is_symlink(),
                permissions: metadata.permissions(),
                modified: metadata.modified().ok(),
                accessed: metadata.accessed().ok(),
                created: metadata.created().ok(),
                path: path.to_path_buf(),
            }
        }
    }
}

/// Directory entry information
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub path: PathBuf,
    pub file_name: OsString,
    pub file_type: Option<FileType>,
}

impl DirEntry {
    fn from_std(entry: fs::DirEntry) -> HalResult<Self> {
        Ok(Self {
            path: entry.path(),
            file_name: entry.file_name(),
            file_type: entry.file_type().ok().map(FileType::from_std),
        })
    }
}

/// File type information
#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    File,
    Directory,
    Symlink,
    #[cfg(unix)]
    BlockDevice,
    #[cfg(unix)]
    CharDevice,
    #[cfg(unix)]
    Fifo,
    #[cfg(unix)]
    Socket,
    Unknown,
}

impl FileType {
    fn from_std(file_type: fs::FileType) -> Self {
        if file_type.is_file() {
            FileType::File
        } else if file_type.is_dir() {
            FileType::Directory
        } else if file_type.is_symlink() {
            FileType::Symlink
        } else {
            #[cfg(unix)]
            {
                use std::os::unix::fs::FileTypeExt;
                if file_type.is_block_device() {
                    FileType::BlockDevice
                } else if file_type.is_char_device() {
                    FileType::CharDevice
                } else if file_type.is_fifo() {
                    FileType::Fifo
                } else if file_type.is_socket() {
                    FileType::Socket
                } else {
                    FileType::Unknown
                }
            }
            #[cfg(not(unix))]
            {
                FileType::Unknown
            }
        }
    }
}

/// File opening options (renamed to avoid conflict with std::fs::OpenOptions)
#[derive(Debug, Clone)]
pub struct HalOpenOptions {
    pub read: bool,
    pub write: bool,
    pub append: bool,
    pub truncate: bool,
    pub create: bool,
    pub create_new: bool,
    #[cfg(unix)]
    pub mode: Option<u32>,
    #[cfg(unix)]
    pub custom_flags: i32,
}

impl Default for HalOpenOptions {
    fn default() -> Self {
        Self {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            #[cfg(unix)]
            mode: None,
            #[cfg(unix)]
            custom_flags: 0,
        }
    }
}

impl HalOpenOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    #[cfg(unix)]
    pub fn mode(&mut self, mode: u32) -> &mut Self {
        self.mode = Some(mode);
        self
    }

    #[cfg(unix)]
    pub fn custom_flags(&mut self, flags: i32) -> &mut Self {
        self.custom_flags = flags;
        self
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        let mut options = fs::OpenOptions::new();
        options.read(self.read);
        options.write(self.write);
        options.append(self.append);
        options.truncate(self.truncate);
        options.create(self.create);
        options.create_new(self.create_new);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            if let Some(mode) = self.mode {
                options.mode(mode);
            }
            if self.custom_flags != 0 {
                options.custom_flags(self.custom_flags);
            }
        }

        options.open(path)
    }
}

/// Disk usage information
#[derive(Debug, Clone)]
pub struct DiskUsage {
    pub total: u64,
    pub free: u64,
    pub available: u64,
}

impl DiskUsage {
    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.free)
    }

    pub fn usage_percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.used() as f64 / self.total as f64) * 100.0
        }
    }
}

/// Check whether a path exists on the filesystem.
pub fn exists<P: AsRef<Path>>(path: P) -> HalResult<bool> {
    let path = path.as_ref();
    Ok(path.exists())
} 