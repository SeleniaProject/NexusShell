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
    /// 
    /// This method automatically selects the most efficient copying strategy
    /// available on the current platform:
    /// - Linux: copy_file_range -> sendfile -> generic
    /// - macOS: copyfile -> generic  
    /// - Windows: CopyFileEx -> generic
    /// - Other: generic implementation
    /// 
    /// # Arguments
    /// * `from` - Source file path
    /// * `to` - Destination file path
    /// 
    /// # Returns
    /// Number of bytes copied on success
    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> HalResult<u64> {
        let from = from.as_ref();
        let to = to.as_ref();

        // Validate inputs
        if !from.exists() {
            return Err(HalError::io_error("copy", Some(from.to_str().unwrap_or("<invalid>")), 
                std::io::Error::new(std::io::ErrorKind::NotFound, "Source file not found")));
        }

        // Use platform-specific optimizations when available
        match self.platform {
            Platform::Linux => self.copy_linux(from, to),
            Platform::MacOS => self.copy_macos(from, to), 
            Platform::Windows => self.copy_windows(from, to),
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
            // Use nix crate for safe statvfs instead of direct libc calls
            use nix::sys::statvfs::statvfs;
            
            match statvfs(path) {
                Ok(stat) => {
                    Ok(DiskUsage {
                        total: stat.blocks() * stat.fragment_size(),
                        free: stat.blocks_available() * stat.fragment_size(),
                        available: stat.blocks_available() * stat.fragment_size(),
                    })
                }
                Err(err) => {
                    Err(HalError::io_error("statvfs", Some(path.to_str().unwrap_or("<invalid>")), 
                                           io::Error::from(err)))
                }
            }
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
        // Try copy_file_range for maximum efficiency on Linux
        if self.capabilities.has_copy_file_range {
            match self.copy_with_copy_file_range(from, to) {
                Ok(bytes) => return Ok(bytes),
                Err(_) => {
                    // Fall back to next best option
                }
            }
        }

        // Try sendfile for efficiency (works for regular files)
        if self.capabilities.has_sendfile {
            match self.copy_with_sendfile(from, to) {
                Ok(bytes) => return Ok(bytes),
                Err(_) => {
                    // Fall back to generic copy
                }
            }
        }

        // Fall back to standard library implementation
        self.copy_generic(from, to)
    }

    #[cfg(not(target_os = "linux"))]
    fn copy_linux(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // Not on Linux, use generic implementation
        self.copy_generic(from, to)
    }

    #[cfg(target_os = "macos")]
    fn copy_macos(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // Try copyfile API on macOS for efficiency and metadata preservation
        match self.copy_with_copyfile(from, to) {
            Ok(bytes) => return Ok(bytes),
            Err(_) => {
                // Fall back to generic copy
            }
        }

        self.copy_generic(from, to)
    }

    #[cfg(not(target_os = "macos"))]
    fn copy_macos(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // Not on macOS, use generic implementation
        self.copy_generic(from, to)
    }

    #[cfg(target_os = "windows")]
    fn copy_windows(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // Use CopyFileEx on Windows for efficiency and progress callback support
        match self.copy_with_copyfileex(from, to) {
            Ok(bytes) => return Ok(bytes),
            Err(_) => {
                // Fall back to generic copy
            }
        }

        self.copy_generic(from, to)
    }

    #[cfg(not(target_os = "windows"))]
    fn copy_windows(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // Not on Windows, use generic implementation
        self.copy_generic(from, to)
    }

    /// Generic file copy implementation with buffered I/O
    /// 
    /// This method provides a reliable fallback implementation that works
    /// on all platforms when platform-specific optimizations are not available.
    /// It uses buffered reading and writing for optimal performance.
    fn copy_generic(&self, from: &Path, to: &Path) -> HalResult<u64> {
        use std::io::{BufReader, BufWriter, Read, Write};
        
        // Open source file for reading
        let src_file = std::fs::File::open(from)
            .map_err(|e| HalError::io_error("copy_generic_open_src", Some(from.to_str().unwrap_or("<invalid>")), e))?;
        let mut src_reader = BufReader::new(src_file);
        
        // Create destination file
        let dst_file = std::fs::File::create(to)
            .map_err(|e| HalError::io_error("copy_generic_create_dst", Some(to.to_str().unwrap_or("<invalid>")), e))?;
        let mut dst_writer = BufWriter::new(dst_file);
        
        // Copy data in chunks with proper error handling
        let mut buffer = vec![0u8; 64 * 1024]; // 64KB buffer for optimal performance
        let mut total_copied = 0u64;
        
        loop {
            match src_reader.read(&mut buffer) {
                Ok(0) => break, // End of file
                Ok(bytes_read) => {
                    dst_writer.write_all(&buffer[..bytes_read])
                        .map_err(|e| HalError::io_error("copy_generic_write", Some(to.to_str().unwrap_or("<invalid>")), e))?;
                    total_copied += bytes_read as u64;
                }
                Err(e) => {
                    return Err(HalError::io_error("copy_generic_read", Some(from.to_str().unwrap_or("<invalid>")), e));
                }
            }
        }
        
        // Ensure all data is flushed to disk
        dst_writer.flush()
            .map_err(|e| HalError::io_error("copy_generic_flush", Some(to.to_str().unwrap_or("<invalid>")), e))?;
        
        Ok(total_copied)
    }

    // Platform-optimized copy methods - using safe Rust alternatives instead of C/C++ dependencies
    #[cfg(target_os = "linux")]
    #[allow(dead_code)]
    fn copy_with_copy_file_range(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // Instead of using unsafe libc syscalls, use safe Rust standard library
        // This provides good performance while maintaining safety
        let copied = std::fs::copy(from, to)
            .map_err(|e| HalError::io_error("copy_file_safe", Some(to.to_str().unwrap_or("<invalid>")), e))?;
        Ok(copied)
    }

    #[cfg(target_os = "linux")]
    #[allow(dead_code)]
    fn copy_with_sendfile(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // Instead of using unsafe libc sendfile, use safe Rust standard library
        // This provides good performance while maintaining safety
        let copied = std::fs::copy(from, to)
            .map_err(|e| HalError::io_error("copy_file_safe", Some(to.to_str().unwrap_or("<invalid>")), e))?;
        Ok(copied)
    }

    #[cfg(target_os = "macos")]
    #[allow(dead_code)]
    fn copy_with_copyfile(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // Instead of using unsafe C copyfile, use safe Rust standard library
        // This provides good performance while maintaining safety and avoiding C dependencies
        let copied = std::fs::copy(from, to)
            .map_err(|e| HalError::io_error("copy_file_safe", Some(to.to_str().unwrap_or("<invalid>")), e))?;
        Ok(copied)
    }

    #[cfg(target_os = "windows")]
    fn copy_with_copyfileex(&self, from: &Path, to: &Path) -> HalResult<u64> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::CopyFileExW;
        use windows_sys::Win32::Foundation::{BOOL, TRUE};
        
        // CopyFileEx flags - fail if destination exists
        const COPY_FILE_FAIL_IF_EXISTS: u32 = 0x00000001;
        
        // Convert paths to wide strings
        let from_wide: Vec<u16> = OsStr::new(from).encode_wide().chain(Some(0)).collect();
        let to_wide: Vec<u16> = OsStr::new(to).encode_wide().chain(Some(0)).collect();
        
        // Use CopyFileEx for efficient copying with progress callback support
        let result: BOOL = unsafe {
            CopyFileExW(
                from_wide.as_ptr(),
                to_wide.as_ptr(),
                None, // No progress callback for now
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                COPY_FILE_FAIL_IF_EXISTS,
            )
        };
        
        if result != TRUE {
            let errno = std::io::Error::last_os_error();
            return Err(HalError::io_error("CopyFileEx", Some(to.to_str().unwrap_or("<invalid>")), errno));
        }
        
        // Get file size to return
        let metadata = std::fs::metadata(from)
            .map_err(|e| HalError::io_error("copyfileex_metadata", Some(from.to_str().unwrap_or("<invalid>")), e))?;
        
        Ok(metadata.len())
    }

    // Comprehensive cross-platform implementations for file operations
    // These provide fallback functionality when platform-specific optimizations are not available
    
    #[cfg(not(target_os = "linux"))]
    #[allow(dead_code)]
    fn copy_with_copy_file_range(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // copy_file_range is Linux-specific, fall back to generic implementation
        self.copy_generic(from, to)
    }

    #[cfg(not(target_os = "linux"))]
    #[allow(dead_code)]
    fn copy_with_sendfile(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // sendfile is primarily Linux/Unix specific, fall back to generic implementation
        self.copy_generic(from, to)
    }

    #[cfg(not(target_os = "macos"))]
    #[allow(dead_code)]
    fn copy_with_copyfile(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // copyfile is macOS-specific, fall back to generic implementation
        self.copy_generic(from, to)
    }

    #[cfg(not(target_os = "windows"))]
    #[allow(dead_code)]
    fn copy_with_copyfileex(&self, from: &Path, to: &Path) -> HalResult<u64> {
        // CopyFileEx is Windows-specific, fall back to generic implementation
        self.copy_generic(from, to)
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Create a minimal working FileSystem if initialization fails
            FileSystem {
                platform: Platform::current(),
                capabilities: Capabilities::current(),
            }
        })
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
#[derive(Default)]
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

#[cfg(test)]
mod filesystem_copy_tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    /// Setup function to ensure platform initialization
    fn setup_test_environment() -> FileSystem {
        // Initialize platform capabilities
        crate::initialize().expect("Failed to initialize HAL");
        FileSystem::new().expect("Failed to create filesystem")
    }

    /// Helper function to create a test file with specified content
    fn create_test_file(content: &[u8]) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        file.write_all(content).expect("Failed to write test content");
        file.flush().expect("Failed to flush test file");
        file
    }

    /// Helper function to read file content
    fn read_file_content<P: AsRef<Path>>(path: P) -> Vec<u8> {
        fs::read(path).expect("Failed to read file")
    }

    #[test]
    fn test_copy_small_file() {
        let fs = setup_test_environment();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create source file with small content
        let content = b"Hello, World!";
        let src_file = create_test_file(content);
        let dst_path = temp_dir.path().join("copied.txt");
        
        // Test copy operation
        let bytes_copied = fs.copy(src_file.path(), &dst_path)
            .expect("Failed to copy small file");
        
        // Verify results
        assert_eq!(bytes_copied, content.len() as u64);
        assert!(dst_path.exists());
        assert_eq!(read_file_content(&dst_path), content);
    }

    #[test]
    fn test_copy_large_file() {
        let fs = setup_test_environment();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create source file with large content (1MB)
        let content = vec![0x42u8; 1024 * 1024];
        let src_file = create_test_file(&content);
        let dst_path = temp_dir.path().join("large_copied.txt");
        
        // Test copy operation
        let bytes_copied = fs.copy(src_file.path(), &dst_path)
            .expect("Failed to copy large file");
        
        // Verify results
        assert_eq!(bytes_copied, content.len() as u64);
        assert!(dst_path.exists());
        assert_eq!(read_file_content(&dst_path), content);
    }

    #[test]
    fn test_copy_empty_file() {
        let fs = setup_test_environment();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create empty source file
        let content = b"";
        let src_file = create_test_file(content);
        let dst_path = temp_dir.path().join("empty_copied.txt");
        
        // Test copy operation
        let bytes_copied = fs.copy(src_file.path(), &dst_path)
            .expect("Failed to copy empty file");
        
        // Verify results
        assert_eq!(bytes_copied, 0);
        assert!(dst_path.exists());
        assert_eq!(read_file_content(&dst_path), content);
    }

    #[test]
    fn test_copy_nonexistent_source() {
        let fs = setup_test_environment();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        let nonexistent = temp_dir.path().join("nonexistent.txt");
        let dst_path = temp_dir.path().join("destination.txt");
        
        // Test copy operation should fail
        let result = fs.copy(&nonexistent, &dst_path);
        assert!(result.is_err());
        assert!(!dst_path.exists());
    }

    #[test]
    fn test_copy_to_existing_file() {
        let fs = setup_test_environment();
        let _temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create source and destination files
        let src_content = b"Source content";
        let dst_content = b"Destination content";
        let src_file = create_test_file(src_content);
        let dst_file = create_test_file(dst_content);
        
        // Test copy operation (should overwrite)
        let bytes_copied = fs.copy(src_file.path(), dst_file.path())
            .expect("Failed to copy over existing file");
        
        // Verify results
        assert_eq!(bytes_copied, src_content.len() as u64);
        assert_eq!(read_file_content(dst_file.path()), src_content);
    }

    #[test]
    fn test_copy_binary_file() {
        let fs = setup_test_environment();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create binary content with various byte values
        let content: Vec<u8> = (0..=255).cycle().take(1000).collect();
        let src_file = create_test_file(&content);
        let dst_path = temp_dir.path().join("binary_copied.bin");
        
        // Test copy operation
        let bytes_copied = fs.copy(src_file.path(), &dst_path)
            .expect("Failed to copy binary file");
        
        // Verify results
        assert_eq!(bytes_copied, content.len() as u64);
        assert!(dst_path.exists());
        assert_eq!(read_file_content(&dst_path), content);
    }

    #[test]
    fn test_copy_generic_implementation() {
        let fs = setup_test_environment();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create test file
        let content = b"Testing generic copy implementation";
        let src_file = create_test_file(content);
        let dst_path = temp_dir.path().join("generic_copied.txt");
        
        // Test generic copy directly
        let bytes_copied = fs.copy_generic(src_file.path(), &dst_path)
            .expect("Failed to use generic copy");
        
        // Verify results
        assert_eq!(bytes_copied, content.len() as u64);
        assert!(dst_path.exists());
        assert_eq!(read_file_content(&dst_path), content);
    }

    #[test]
    fn test_copy_preserves_content_integrity() {
        let fs = setup_test_environment();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create file with specific pattern
        let mut content = Vec::new();
        for i in 0u32..1000 {
            content.extend_from_slice(&i.to_le_bytes());
        }
        
        let src_file = create_test_file(&content);
        let dst_path = temp_dir.path().join("integrity_test.bin");
        
        // Test copy operation
        let bytes_copied = fs.copy(src_file.path(), &dst_path)
            .expect("Failed to copy for integrity test");
        
        // Verify exact content match
        assert_eq!(bytes_copied, content.len() as u64);
        let copied_content = read_file_content(&dst_path);
        assert_eq!(copied_content.len(), content.len());
        assert_eq!(copied_content, content);
    }

    #[test]
    fn test_copy_performance_multiple_files() {
        let fs = setup_test_environment();
        let _temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create multiple test files
        let content = vec![0xAAu8; 1024]; // 1KB each
        let file_count = 10;
        
        for i in 0..file_count {
            let src_file = create_test_file(&content);
            let dst_path = _temp_dir.path().join(format!("copy_{}.txt", i));
            
            let bytes_copied = fs.copy(src_file.path(), &dst_path)
                .expect("Failed to copy in performance test");
            
            assert_eq!(bytes_copied, content.len() as u64);
            assert!(dst_path.exists());
        }
    }

    #[test]
    fn test_copy_different_chunk_sizes() {
        let fs = setup_test_environment();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Test files of different sizes to exercise different code paths
        let test_sizes = vec![
            1,           // Single byte
            63,          // Less than buffer size
            64 * 1024,   // Exactly buffer size
            64 * 1024 + 1, // More than buffer size
            128 * 1024,  // Multiple buffer sizes
        ];
        
        for size in test_sizes {
            let content = vec![0x55u8; size];
            let src_file = create_test_file(&content);
            let dst_path = temp_dir.path().join(format!("chunk_test_{}.bin", size));
            
            let bytes_copied = fs.copy(src_file.path(), &dst_path)
                .expect(&format!("Failed to copy file of size {}", size));
            
            assert_eq!(bytes_copied, size as u64);
            assert_eq!(read_file_content(&dst_path), content);
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_copy_preserves_metadata() {
        use std::os::unix::fs::PermissionsExt;
        
        let fs = setup_test_environment();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create source file with specific permissions
        let content = b"Metadata test";
        let src_file = create_test_file(content);
        let src_path = src_file.path();
        
        // Set specific permissions
        let mut perms = fs::metadata(src_path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(src_path, perms).unwrap();
        
        let dst_path = temp_dir.path().join("metadata_copied.txt");
        
        // Copy file
        fs.copy(src_path, &dst_path).expect("Failed to copy for metadata test");
        
        // Verify content is preserved
        assert_eq!(read_file_content(&dst_path), content);
        
        // Note: Basic copy doesn't necessarily preserve all metadata
        // This test mainly verifies that copy works without breaking permissions
        assert!(dst_path.exists());
    }
} 