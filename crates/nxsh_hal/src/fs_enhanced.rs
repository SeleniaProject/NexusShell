use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
    path::Path,
    fs,
};
use anyhow::Result;

/// Advanced file system operations and monitoring
#[derive(Debug, Clone)]
pub struct FileSystemMonitor {
    stats: Arc<RwLock<FileSystemStats>>,
    watchers: Arc<RwLock<HashMap<String, FileWatcher>>>,
}

impl Default for FileSystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystemMonitor {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(FileSystemStats::default())),
            watchers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Monitor directory for changes
    pub fn watch_directory(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        
        if let Ok(mut watchers) = self.watchers.write() {
            let watcher = FileWatcher::new(path)?;
            watchers.insert(path_str, watcher);
        }
        
        Ok(())
    }

    /// Stop monitoring directory
    pub fn unwatch_directory(&self, path: &Path) {
        let path_str = path.to_string_lossy().to_string();
        
        if let Ok(mut watchers) = self.watchers.write() {
            watchers.remove(&path_str);
        }
    }

    /// Get file system statistics
    pub fn stats(&self) -> FileSystemStats {
        self.stats.read().unwrap().clone()
    }

    /// Record file operation statistics
    pub fn record_operation(&self, operation: FileOperation, duration: Duration, bytes: u64) {
        if let Ok(mut stats) = self.stats.write() {
            match operation {
                FileOperation::Read => {
                    stats.reads += 1;
                    stats.bytes_read += bytes;
                    stats.total_read_time += duration;
                }
                FileOperation::Write => {
                    stats.writes += 1;
                    stats.bytes_written += bytes;
                    stats.total_write_time += duration;
                }
                FileOperation::Delete => {
                    stats.deletes += 1;
                    stats.total_delete_time += duration;
                }
                FileOperation::Create => {
                    stats.creates += 1;
                    stats.total_create_time += duration;
                }
            }
        }
    }

    /// Analyze directory structure
    pub fn analyze_directory(&self, path: &Path) -> Result<DirectoryAnalysis> {
        let start = Instant::now();
        let mut analysis = DirectoryAnalysis::default();
        
        self.analyze_recursive(path, &mut analysis, 0)?;
        analysis.analysis_time = start.elapsed();
        
        Ok(analysis)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn analyze_recursive(&self, path: &Path, analysis: &mut DirectoryAnalysis, depth: usize) -> Result<()> {
        if depth > 100 {
            return Ok(()); // Prevent infinite recursion
        }

        let entries = fs::read_dir(path)?;
        
        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let path = entry.path();
            
            if metadata.is_dir() {
                analysis.directories += 1;
                self.analyze_recursive(&path, analysis, depth + 1)?;
            } else {
                analysis.files += 1;
                analysis.total_size += metadata.len();
                
                // Categorize by extension
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    *analysis.file_types.entry(ext).or_insert(0) += 1;
                }
            }
        }
        
        Ok(())
    }
}

/// File operation types for statistics
#[derive(Debug, Clone, Copy)]
pub enum FileOperation {
    Read,
    Write,
    Create,
    Delete,
}

/// File system operation statistics
#[derive(Debug, Clone, Default)]
pub struct FileSystemStats {
    pub reads: u64,
    pub writes: u64,
    pub creates: u64,
    pub deletes: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub total_read_time: Duration,
    pub total_write_time: Duration,
    pub total_create_time: Duration,
    pub total_delete_time: Duration,
}

impl FileSystemStats {
    pub fn read_throughput_mbps(&self) -> f64 {
        if self.total_read_time.as_secs_f64() > 0.0 {
            (self.bytes_read as f64 / (1024.0 * 1024.0)) / self.total_read_time.as_secs_f64()
        } else {
            0.0
        }
    }

    pub fn write_throughput_mbps(&self) -> f64 {
        if self.total_write_time.as_secs_f64() > 0.0 {
            (self.bytes_written as f64 / (1024.0 * 1024.0)) / self.total_write_time.as_secs_f64()
        } else {
            0.0
        }
    }

    pub fn avg_read_time_ms(&self) -> f64 {
        if self.reads > 0 {
            self.total_read_time.as_millis() as f64 / self.reads as f64
        } else {
            0.0
        }
    }

    pub fn avg_write_time_ms(&self) -> f64 {
        if self.writes > 0 {
            self.total_write_time.as_millis() as f64 / self.writes as f64
        } else {
            0.0
        }
    }
}

/// Directory analysis results
#[derive(Debug, Clone, Default)]
pub struct DirectoryAnalysis {
    pub files: u64,
    pub directories: u64,
    pub total_size: u64,
    pub file_types: HashMap<String, u64>,
    pub analysis_time: Duration,
}

impl DirectoryAnalysis {
    pub fn largest_file_types(&self, count: usize) -> Vec<(String, u64)> {
        let mut types: Vec<_> = self.file_types.iter()
            .map(|(ext, count)| (ext.clone(), *count))
            .collect();
        
        types.sort_by(|a, b| b.1.cmp(&a.1));
        types.into_iter().take(count).collect()
    }

    pub fn total_items(&self) -> u64 {
        self.files + self.directories
    }
}

/// File watcher for monitoring changes
#[derive(Debug)]
pub struct FileWatcher {
    path: std::path::PathBuf,
    last_check: Instant,
    file_states: HashMap<String, FileState>,
}

impl FileWatcher {
    pub fn new(path: &Path) -> Result<Self> {
        let mut watcher = Self {
            path: path.to_path_buf(),
            last_check: Instant::now(),
            file_states: HashMap::new(),
        };
        
        watcher.initialize_states()?;
        Ok(watcher)
    }

    pub fn check_changes(&mut self) -> Result<Vec<FileChange>> {
        let mut changes = Vec::new();
        let current_states = self.scan_directory()?;
        
        // Check for new/modified files
        for (path, new_state) in &current_states {
            match self.file_states.get(path) {
                Some(old_state) => {
                    if old_state.modified != new_state.modified {
                        changes.push(FileChange::Modified(path.clone()));
                    }
                    if old_state.size != new_state.size {
                        changes.push(FileChange::SizeChanged(path.clone(), old_state.size, new_state.size));
                    }
                }
                None => {
                    changes.push(FileChange::Created(path.clone()));
                }
            }
        }
        
        // Check for deleted files
        for path in self.file_states.keys() {
            if !current_states.contains_key(path) {
                changes.push(FileChange::Deleted(path.clone()));
            }
        }
        
        self.file_states = current_states;
        self.last_check = Instant::now();
        
        Ok(changes)
    }

    fn initialize_states(&mut self) -> Result<()> {
        self.file_states = self.scan_directory()?;
        Ok(())
    }

    fn scan_directory(&self) -> Result<HashMap<String, FileState>> {
        let mut states = HashMap::new();
        
        if self.path.is_dir() {
            for entry in fs::read_dir(&self.path)? {
                let entry = entry?;
                let path = entry.path();
                let metadata = entry.metadata()?;
                
                if let Some(path_str) = path.to_str() {
                    let state = FileState {
                        size: metadata.len(),
                        modified: metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                    };
                    states.insert(path_str.to_string(), state);
                }
            }
        }
        
        Ok(states)
    }
}

/// File state for change detection
#[derive(Debug, Clone)]
struct FileState {
    size: u64,
    modified: std::time::SystemTime,
}

/// Types of file changes
#[derive(Debug, Clone)]
pub enum FileChange {
    Created(String),
    Modified(String),
    Deleted(String),
    SizeChanged(String, u64, u64),
}

/// Disk usage analyzer
pub struct DiskUsageAnalyzer;

impl DiskUsageAnalyzer {
    /// Analyze disk usage for a directory
    pub fn analyze(path: &Path) -> Result<DiskUsage> {
        let start = Instant::now();
        let mut usage = DiskUsage::default();
        
        Self::scan_recursive(path, &mut usage)?;
        usage.scan_time = start.elapsed();
        
        Ok(usage)
    }

    /// Get disk usage for current directory
    pub fn current_directory() -> Result<DiskUsage> {
        let current_dir = std::env::current_dir()?;
        Self::analyze(&current_dir)
    }

    /// Find largest directories
    pub fn find_large_directories(path: &Path, min_size: u64) -> Result<Vec<(std::path::PathBuf, u64)>> {
        let mut large_dirs = Vec::new();
        Self::find_large_recursive(path, min_size, &mut large_dirs)?;
        
        large_dirs.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(large_dirs)
    }

    fn scan_recursive(path: &Path, usage: &mut DiskUsage) -> Result<u64> {
        let mut dir_size = 0;
        
        if !path.is_dir() {
            let metadata = fs::metadata(path)?;
            return Ok(metadata.len());
        }

        usage.directories += 1;
        
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let metadata = entry.metadata()?;
            
            if metadata.is_dir() {
                let subdir_size = Self::scan_recursive(&entry_path, usage)?;
                dir_size += subdir_size;
            } else {
                usage.files += 1;
                let file_size = metadata.len();
                dir_size += file_size;
                
                if file_size > 100 * 1024 * 1024 { // Files > 100MB
                    usage.large_files.push((entry_path, file_size));
                }
            }
        }
        
        usage.total_size += dir_size;
        Ok(dir_size)
    }

    fn find_large_recursive(path: &Path, min_size: u64, results: &mut Vec<(std::path::PathBuf, u64)>) -> Result<u64> {
        if !path.is_dir() {
            return Ok(0);
        }

        let mut dir_size = 0;
        
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                let subdir_size = Self::find_large_recursive(&entry_path, min_size, results)?;
                dir_size += subdir_size;
                
                if subdir_size >= min_size {
                    results.push((entry_path, subdir_size));
                }
            } else {
                let metadata = entry.metadata()?;
                dir_size += metadata.len();
            }
        }
        
        Ok(dir_size)
    }
}

/// Disk usage information
#[derive(Debug, Clone, Default)]
pub struct DiskUsage {
    pub total_size: u64,
    pub files: u64,
    pub directories: u64,
    pub large_files: Vec<(std::path::PathBuf, u64)>,
    pub scan_time: Duration,
}

impl DiskUsage {
    pub fn format_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit = 0;
        
        while size >= 1024.0 && unit < UNITS.len() - 1 {
            size /= 1024.0;
            unit += 1;
        }
        
        if unit == 0 {
            format!("{} {}", size as u64, UNITS[unit])
        } else {
            format!("{:.2} {}", size, UNITS[unit])
        }
    }

    pub fn avg_file_size(&self) -> u64 {
        if self.files > 0 {
            self.total_size / self.files
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_filesystem_monitor() {
        let monitor = FileSystemMonitor::new();
        
        monitor.record_operation(
            FileOperation::Read, 
            Duration::from_millis(10), 
            1024
        );
        
        let stats = monitor.stats();
        assert_eq!(stats.reads, 1);
        assert_eq!(stats.bytes_read, 1024);
        assert!(stats.read_throughput_mbps() > 0.0);
    }

    #[test]
    fn test_directory_analysis() {
        let dir = tempdir().unwrap();
        
        // Create test files
        let mut file1 = File::create(dir.path().join("test1.txt")).unwrap();
        file1.write_all(b"test content").unwrap();
    // Ensure metadata length is visible on all platforms before analysis
    let _ = file1.sync_all();
        
        let mut file2 = File::create(dir.path().join("test2.rs")).unwrap();
        file2.write_all(b"fn main() {}").unwrap();
    let _ = file2.sync_all();
        
        let monitor = FileSystemMonitor::new();
        let analysis = monitor.analyze_directory(dir.path()).unwrap();
        
        assert_eq!(analysis.files, 2);
        assert!(analysis.file_types.contains_key("txt"));
        assert!(analysis.file_types.contains_key("rs"));
        assert!(analysis.total_size > 0);
    }

    #[test]
    fn test_disk_usage_analyzer() {
        let dir = tempdir().unwrap();
        
        // Create test file
        let mut file = File::create(dir.path().join("test.txt")).unwrap();
        file.write_all(b"test content for disk usage").unwrap();
    let _ = file.sync_all();
        
        let usage = DiskUsageAnalyzer::analyze(dir.path()).unwrap();
        
        assert_eq!(usage.files, 1);
        assert!(usage.total_size > 0);
        assert!(usage.scan_time > Duration::ZERO);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(DiskUsage::format_size(1024), "1.00 KB");
        assert_eq!(DiskUsage::format_size(1024 * 1024), "1.00 MB");
        assert_eq!(DiskUsage::format_size(1536), "1.50 KB");
        assert_eq!(DiskUsage::format_size(512), "512 B");
    }
}
