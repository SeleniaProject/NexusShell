use crate::compat::Result;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::Path,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

/// High-performance I/O manager with smart optimizations
pub struct IoManager {
    buffer_size: usize,
    stats: Arc<RwLock<IoStats>>,
}

impl IoManager {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer_size: buffer_size.max(4096), // Ensure minimum efficient buffer size
            stats: Arc::new(RwLock::new(IoStats::default())),
        }
    }

    /// Optimized file reading with smart buffer sizing  
    pub fn read_file_buffered(&self, path: &Path) -> Result<String> {
        let start = Instant::now();

        // For small files, use pre-allocated capacity optimization
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len() as usize;

        // Optimize based on file size
        let mut content = if file_size > 0 && file_size <= 64 * 1024 {
            // For small to medium files, pre-allocate exact capacity
            String::with_capacity(file_size)
        } else {
            // For large files or unknown size, use default
            String::new()
        };

        // Use optimal buffer size based on file size
        let optimal_buffer_size = if file_size <= 4096 {
            file_size.max(512) // Minimum 512 bytes
        } else if file_size <= 64 * 1024 {
            8192 // 8KB for medium files
        } else {
            self.buffer_size // Use configured size for large files
        };

        let mut reader = BufReader::with_capacity(optimal_buffer_size, file);
        reader.read_to_string(&mut content)?;

        self.update_read_stats(content.len() as u64, start.elapsed());
        Ok(content)
    }

    /// Optimized line reading with efficient buffer sizing
    pub fn read_file_lines(&self, path: &Path) -> Result<Vec<String>> {
        let start = Instant::now();

        // For line reading, use a simple direct read approach for consistency
        let file = File::open(path)?;
        let reader = BufReader::with_capacity(self.buffer_size, file);
        let lines: Vec<String> = reader
            .lines()
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(crate::error::ShellError::io)?;

        self.update_read_stats(lines.len() as u64, start.elapsed());
        Ok(lines)
    }

    /// Optimized file writing with smart buffer sizing
    pub fn write_file_buffered(&self, path: &Path, content: &str) -> Result<()> {
        let start = Instant::now();

        // Use optimal buffer size based on content size
        let optimal_buffer_size = if content.len() <= 4096 {
            content.len().max(512) // Minimum 512 bytes
        } else if content.len() <= 64 * 1024 {
            8192 // 8KB for medium content
        } else {
            self.buffer_size // Use configured size for large content
        };

        let file = File::create(path)?;
        let mut writer = BufWriter::with_capacity(optimal_buffer_size, file);
        writer.write_all(content.as_bytes())?;
        writer.flush()?;

        self.update_write_stats(content.len() as u64, start.elapsed());
        Ok(())
    }

    /// Optimized append operation with smart buffering
    pub fn append_file_buffered(&self, path: &Path, content: &str) -> Result<()> {
        let start = Instant::now();

        let optimal_buffer_size = if content.len() <= 4096 {
            content.len().max(512)
        } else {
            8192
        };

        let file = OpenOptions::new().create(true).append(true).open(path)?;

        let mut writer = BufWriter::with_capacity(optimal_buffer_size, file);
        writer.write_all(content.as_bytes())?;
        writer.flush()?;

        self.update_write_stats(content.len() as u64, start.elapsed());
        Ok(())
    }

    pub fn stats(&self) -> IoStats {
        self.stats.read().map(|s| s.clone()).unwrap_or_default()
    }

    fn update_read_stats(&self, bytes: u64, duration: Duration) {
        if let Ok(mut stats) = self.stats.write() {
            stats.bytes_read += bytes;
            stats.read_operations += 1;
            stats.total_read_time += duration;
        }
    }

    fn update_write_stats(&self, bytes: u64, duration: Duration) {
        if let Ok(mut stats) = self.stats.write() {
            stats.bytes_written += bytes;
            stats.write_operations += 1;
            stats.total_write_time += duration;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct IoStats {
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub read_operations: u64,
    pub write_operations: u64,
    pub total_read_time: Duration,
    pub total_write_time: Duration,
}

impl IoStats {
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
        if self.read_operations > 0 {
            self.total_read_time.as_millis() as f64 / self.read_operations as f64
        } else {
            0.0
        }
    }

    pub fn avg_write_time_ms(&self) -> f64 {
        if self.write_operations > 0 {
            self.total_write_time.as_millis() as f64 / self.write_operations as f64
        } else {
            0.0
        }
    }
}

/// Async I/O operations with optimization
#[allow(dead_code)]
pub struct AsyncIoManager {
    buffer_size: usize,
    concurrent_limit: usize,
    stats: Arc<RwLock<IoStats>>,
}

impl AsyncIoManager {
    pub fn new(buffer_size: usize, concurrent_limit: usize) -> Self {
        Self {
            buffer_size,
            concurrent_limit,
            stats: Arc::new(RwLock::new(IoStats::default())),
        }
    }

    pub async fn read_multiple_files(&self, paths: Vec<&Path>) -> Result<Vec<String>> {
        use futures::future::join_all;
        use tokio::fs;

        let start = Instant::now();
        let chunks: Vec<_> = paths.chunks(self.concurrent_limit).collect();
        let mut all_results = Vec::new();

        for chunk in chunks {
            let tasks: Vec<_> = chunk
                .iter()
                .map(|&path| async move { fs::read_to_string(path).await })
                .collect();

            let results = join_all(tasks).await;
            for result in results {
                all_results.push(result?);
            }
        }

        let total_bytes: u64 = all_results.iter().map(|s| s.len() as u64).sum();
        self.update_read_stats(total_bytes, start.elapsed());

        Ok(all_results)
    }

    pub async fn write_multiple_files(&self, data: Vec<(&Path, &str)>) -> Result<()> {
        use futures::future::join_all;
        use tokio::fs;

        let start = Instant::now();
        let chunks: Vec<_> = data.chunks(self.concurrent_limit).collect();

        for chunk in chunks {
            let tasks: Vec<_> = chunk
                .iter()
                .map(|&(path, content)| async move { fs::write(path, content).await })
                .collect();

            let results = join_all(tasks).await;
            for result in results {
                result?;
            }
        }

        let total_bytes: u64 = data.iter().map(|(_, content)| content.len() as u64).sum();
        self.update_write_stats(total_bytes, start.elapsed());

        Ok(())
    }

    pub fn stats(&self) -> IoStats {
        self.stats.read().map(|s| s.clone()).unwrap_or_default()
    }

    fn update_read_stats(&self, bytes: u64, duration: Duration) {
        if let Ok(mut stats) = self.stats.write() {
            stats.bytes_read += bytes;
            stats.read_operations += 1;
            stats.total_read_time += duration;
        }
    }

    fn update_write_stats(&self, bytes: u64, duration: Duration) {
        if let Ok(mut stats) = self.stats.write() {
            stats.bytes_written += bytes;
            stats.write_operations += 1;
            stats.total_write_time += duration;
        }
    }
}

/// Global I/O manager instance
use std::sync::OnceLock;
static GLOBAL_IO_MANAGER: OnceLock<Arc<IoManager>> = OnceLock::new();

pub fn global_io_manager() -> Arc<IoManager> {
    GLOBAL_IO_MANAGER
        .get_or_init(|| {
            Arc::new(IoManager::new(8192)) // 8KB buffer by default
        })
        .clone()
}

/// Convenient functions for global I/O operations
pub fn read_file_fast(path: &Path) -> Result<String> {
    global_io_manager().read_file_buffered(path)
}

pub fn write_file_fast(path: &Path, content: &str) -> Result<()> {
    global_io_manager().write_file_buffered(path, content)
}

pub fn append_file_fast(path: &Path, content: &str) -> Result<()> {
    global_io_manager().append_file_buffered(path, content)
}

pub fn read_lines_fast(path: &Path) -> Result<Vec<String>> {
    global_io_manager().read_file_lines(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_io_manager_buffered_operations() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let io_manager = IoManager::new(1024);

        // Test write
        let content = "Hello, World!";
        io_manager.write_file_buffered(&file_path, content).unwrap();

        // Test read
        let read_content = io_manager.read_file_buffered(&file_path).unwrap();
        assert_eq!(content, read_content);

        // Test append
        let append_content = "\nAppended line";
        io_manager
            .append_file_buffered(&file_path, append_content)
            .unwrap();

        let final_content = io_manager.read_file_buffered(&file_path).unwrap();
        assert!(final_content.contains("Hello, World!"));
        assert!(final_content.contains("Appended line"));

        let stats = io_manager.stats();
        assert!(stats.bytes_read > 0);
        assert!(stats.bytes_written > 0);
        assert!(stats.read_operations >= 2);
        assert!(stats.write_operations >= 2);
    }

    #[test]
    fn test_io_stats_calculations() {
        let stats = IoStats {
            bytes_read: 1024 * 1024, // 1MB
            read_operations: 10,
            total_read_time: Duration::from_millis(100),
            ..Default::default()
        };

        assert!(stats.read_throughput_mbps() > 0.0);
        assert_eq!(stats.avg_read_time_ms(), 10.0);
    }
}
