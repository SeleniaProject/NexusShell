use std::{
    collections::HashMap,
    io::{BufReader, BufWriter, Read, Write, BufRead},
    fs::{File, OpenOptions},
    path::Path,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use crate::compat::Result;

/// Buffered I/O manager for optimized file operations
pub struct IoManager {
    read_buffers: Arc<RwLock<HashMap<String, BufReader<File>>>>,
    write_buffers: Arc<RwLock<HashMap<String, BufWriter<File>>>>,
    buffer_size: usize,
    stats: Arc<RwLock<IoStats>>,
}

impl IoManager {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            read_buffers: Arc::new(RwLock::new(HashMap::new())),
            write_buffers: Arc::new(RwLock::new(HashMap::new())),
            buffer_size,
            stats: Arc::new(RwLock::new(IoStats::default())),
        }
    }

    pub fn read_file_buffered(&self, path: &Path) -> Result<String> {
        let path_str = path.to_string_lossy().to_string();
        let start = Instant::now();
        
        let mut content = String::new();
        
        // Try to get existing buffered reader
        if let Ok(mut buffers) = self.read_buffers.write() {
            if let Some(reader) = buffers.get_mut(&path_str) {
                reader.read_to_string(&mut content)?;
                self.update_read_stats(content.len() as u64, start.elapsed());
                return Ok(content);
            }
        }

        // Create new buffered reader
        let file = File::open(path)?;
        let mut reader = BufReader::with_capacity(self.buffer_size, file);
        reader.read_to_string(&mut content)?;

        // Cache the reader for reuse
        if let Ok(mut buffers) = self.read_buffers.write() {
            // Reset reader position for next use
            let file = File::open(path)?;
            let new_reader = BufReader::with_capacity(self.buffer_size, file);
            buffers.insert(path_str, new_reader);
        }

        self.update_read_stats(content.len() as u64, start.elapsed());
        Ok(content)
    }

    pub fn read_file_lines(&self, path: &Path) -> Result<Vec<String>> {
        let path_str = path.to_string_lossy().to_string();
        let start = Instant::now();
        
        let mut lines = Vec::new();
        
        // Try to use existing buffered reader
        if let Ok(mut buffers) = self.read_buffers.write() {
            if let Some(reader) = buffers.get_mut(&path_str) {
                for line_result in reader.lines() {
                    lines.push(line_result?);
                }
                self.update_read_stats(lines.len() as u64, start.elapsed());
                return Ok(lines);
            }
        }

        // Create new buffered reader
        let file = File::open(path)?;
        let reader = BufReader::with_capacity(self.buffer_size, file);
        
        for line_result in reader.lines() {
            lines.push(line_result?);
        }

        // Cache new reader
        if let Ok(mut buffers) = self.read_buffers.write() {
            let file = File::open(path)?;
            let new_reader = BufReader::with_capacity(self.buffer_size, file);
            buffers.insert(path_str, new_reader);
        }

        self.update_read_stats(lines.len() as u64, start.elapsed());
        Ok(lines)
    }

    pub fn write_file_buffered(&self, path: &Path, content: &str) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        let start = Instant::now();
        
        // Try to use existing buffered writer
        if let Ok(mut buffers) = self.write_buffers.write() {
            if let Some(writer) = buffers.get_mut(&path_str) {
                writer.write_all(content.as_bytes())?;
                writer.flush()?;
                self.update_write_stats(content.len() as u64, start.elapsed());
                return Ok(());
            }
        }

        // Create new buffered writer
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        
        let mut writer = BufWriter::with_capacity(self.buffer_size, file);
        writer.write_all(content.as_bytes())?;
        writer.flush()?;

        // Cache writer for reuse
        if let Ok(mut buffers) = self.write_buffers.write() {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(path)?;
            let new_writer = BufWriter::with_capacity(self.buffer_size, file);
            buffers.insert(path_str, new_writer);
        }

        self.update_write_stats(content.len() as u64, start.elapsed());
        Ok(())
    }

    pub fn append_file_buffered(&self, path: &Path, content: &str) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        let start = Instant::now();
        
        // Try to use existing buffered writer
        if let Ok(mut buffers) = self.write_buffers.write() {
            if let Some(writer) = buffers.get_mut(&path_str) {
                writer.write_all(content.as_bytes())?;
                writer.flush()?;
                self.update_write_stats(content.len() as u64, start.elapsed());
                return Ok(());
            }
        }

        // Create new buffered writer for appending
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)?;
        
        let mut writer = BufWriter::with_capacity(self.buffer_size, file);
        writer.write_all(content.as_bytes())?;
        writer.flush()?;

        // Cache writer
        if let Ok(mut buffers) = self.write_buffers.write() {
            buffers.insert(path_str, writer);
        }

        self.update_write_stats(content.len() as u64, start.elapsed());
        Ok(())
    }

    pub fn flush_all(&self) -> Result<()> {
        if let Ok(mut buffers) = self.write_buffers.write() {
            for writer in buffers.values_mut() {
                writer.flush()?;
            }
        }
        Ok(())
    }

    pub fn close_file(&self, path: &Path) {
        let path_str = path.to_string_lossy().to_string();
        
        if let Ok(mut read_buffers) = self.read_buffers.write() {
            read_buffers.remove(&path_str);
        }
        
        if let Ok(mut write_buffers) = self.write_buffers.write() {
            if let Some(mut writer) = write_buffers.remove(&path_str) {
                let _ = writer.flush();
            }
        }
    }

    pub fn clear_cache(&self) {
        if let Ok(mut read_buffers) = self.read_buffers.write() {
            read_buffers.clear();
        }
        
        if let Ok(mut write_buffers) = self.write_buffers.write() {
            for writer in write_buffers.values_mut() {
                let _ = writer.flush();
            }
            write_buffers.clear();
        }
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
        use tokio::fs;
        use futures::future::join_all;
        
        let start = Instant::now();
        let chunks: Vec<_> = paths.chunks(self.concurrent_limit).collect();
        let mut all_results = Vec::new();
        
        for chunk in chunks {
            let tasks: Vec<_> = chunk.iter().map(|&path| {
                async move {
                    fs::read_to_string(path).await
                }
            }).collect();
            
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
        use tokio::fs;
        use futures::future::join_all;
        
        let start = Instant::now();
        let chunks: Vec<_> = data.chunks(self.concurrent_limit).collect();
        
        for chunk in chunks {
            let tasks: Vec<_> = chunk.iter().map(|&(path, content)| {
                async move {
                    fs::write(path, content).await
                }
            }).collect();
            
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
    GLOBAL_IO_MANAGER.get_or_init(|| {
        Arc::new(IoManager::new(8192)) // 8KB buffer by default
    }).clone()
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
        io_manager.append_file_buffered(&file_path, append_content).unwrap();
        
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
        let mut stats = IoStats::default();
        stats.bytes_read = 1024 * 1024; // 1MB
        stats.read_operations = 10;
        stats.total_read_time = Duration::from_millis(100);
        
        assert!(stats.read_throughput_mbps() > 0.0);
        assert_eq!(stats.avg_read_time_ms(), 10.0);
    }
}
