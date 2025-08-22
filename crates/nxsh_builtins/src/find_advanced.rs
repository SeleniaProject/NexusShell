//! Advanced find command with smart search capabilities

use anyhow::Result;
use crate::ui_design::Colorize;
use std::path::{Path, PathBuf};
use std::fs;

/// Advanced find command
pub struct FindAdvanced;

impl FindAdvanced {
    /// Smart find with various filters
    pub fn find_smart(
        path: &str, 
        name_pattern: Option<&str>,
        file_type: Option<&str>,
        size_filter: Option<&str>,
        modified_days: Option<i32>
    ) -> Result<()> {
        
        println!("{}", format!("🔍 Searching in '{}'", path).primary().bold());
        if let Some(pattern) = name_pattern {
            println!("{}", format!("📝 Name pattern: {}", pattern).info());
        }
        if let Some(ftype) = file_type {
            println!("{}", format!("📁 Type: {}", ftype).info());
        }
        println!("{}", "─".repeat(50).muted());
        
        let mut results = Vec::new();
        Self::search_recursive(Path::new(path), &mut results, name_pattern, file_type, size_filter, modified_days)?;
        
        if results.is_empty() {
            println!("{}", "No files found matching criteria".warning());
        } else {
            for (i, result) in results.iter().enumerate() {
                Self::display_result(i + 1, result)?;
            }
            println!("\n{}", format!("Found {} file(s)", results.len()).success().bold());
        }
        
        Ok(())
    }
    
    /// Recursive search implementation
    fn search_recursive(
        dir: &Path,
        results: &mut Vec<PathBuf>,
        name_pattern: Option<&str>,
        file_type: Option<&str>,
        _size_filter: Option<&str>,
        _modified_days: Option<i32>
    ) -> Result<()> {
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Check name pattern
                if let Some(pattern) = name_pattern {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !Self::matches_pattern(name, pattern) {
                            continue;
                        }
                    }
                }
                
                // Check file type
                if let Some(ftype) = file_type {
                    match ftype {
                        "f" | "file" => if !path.is_file() { continue; },
                        "d" | "dir" | "directory" => if !path.is_dir() { continue; },
                        _ => {}
                    }
                }
                
                results.push(path.clone());
                
                // Recurse into directories
                if path.is_dir() && results.len() < 1000 { // Limit results
                    let _ = Self::search_recursive(&path, results, name_pattern, file_type, _size_filter, _modified_days);
                }
            }
        }
        
        Ok(())
    }
    
    /// Simple pattern matching (supports * wildcard)
    fn matches_pattern(name: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        if pattern.contains('*') {
            // Simple wildcard matching
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return name.starts_with(prefix) && name.ends_with(suffix);
            }
        }
        
        name.contains(pattern)
    }
    
    /// Display search result with formatting
    fn display_result(index: usize, path: &PathBuf) -> Result<()> {
        let metadata = fs::metadata(path).ok();
        
        let (size_str, type_str, color_fn): (String, String, fn(&str) -> String) = if path.is_dir() {
            ("DIR".to_string(), "📁".to_string(), |s: &str| s.cyan().to_string())
        } else {
            let size = metadata.map(|m| m.len()).unwrap_or(0);
            let size_str = Self::format_size(size);
            
            let type_str = if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext {
                    "txt" | "md" | "rst" => "📄",
                    "rs" | "py" | "js" | "cpp" | "c" | "h" => "💻",
                    "jpg" | "png" | "gif" | "bmp" | "svg" => "🖼️",
                    "mp3" | "wav" | "flac" | "ogg" => "🎵",
                    "mp4" | "avi" | "mkv" | "mov" => "🎬",
                    "zip" | "tar" | "gz" | "rar" => "📦",
                    _ => "📄"
                }
            } else {
                "📄"
            }.to_string();
            
            (size_str, type_str, |s: &str| s.green().to_string())
        };
        
        let display_path = color_fn(&path.display().to_string());
        let index_str = format!("{:3}.", index).secondary();
        let size_part = format!("({})", size_str).muted();
        
        println!("{} {} {} {}", index_str, type_str, display_path, size_part);
        
        Ok(())
    }
    
    /// Format file size in human readable format
    fn format_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
    
    /// Find files by extension
    pub fn find_by_extension(path: &str, extension: &str) -> Result<()> {
        let pattern = format!("*.{}", extension.trim_start_matches('.'));
        Self::find_smart(path, Some(&pattern), Some("file"), None, None)
    }
    
    /// Find large files
    pub fn find_large_files(path: &str, min_size_mb: u64) -> Result<()> {
        println!("{}", format!("🔍 Finding files larger than {} MB", min_size_mb).primary().bold());
        
        let mut large_files = Vec::new();
        Self::find_large_recursive(Path::new(path), &mut large_files, min_size_mb * 1024 * 1024)?;
        
        if large_files.is_empty() {
            println!("{}", "No large files found".info());
        } else {
            // Sort by size (largest first)
            large_files.sort_by(|a, b| b.1.cmp(&a.1));
            
            for (i, (path, size)) in large_files.iter().enumerate() {
                let size_str = Self::format_size(*size);
                println!("{:3}. {} {}", 
                    (i + 1).to_string().secondary(),
                    path.display().to_string().primary(),
                    format!("({})", size_str).success().bold()
                );
            }
        }
        
        Ok(())
    }
    
    /// Recursive search for large files
    fn find_large_recursive(dir: &Path, results: &mut Vec<(PathBuf, u64)>, min_size: u64) -> Result<()> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path) {
                        let size = metadata.len();
                        if size >= min_size {
                            results.push((path, size));
                        }
                    }
                } else if path.is_dir() && results.len() < 1000 {
                    let _ = Self::find_large_recursive(&path, results, min_size);
                }
            }
        }
        
        Ok(())
    }
}

/// CLI function for advanced find
pub fn find_advanced_cli(args: &[String]) -> Result<()> {
    match args.get(0).map(|s| s.as_str()) {
        Some("ext") => {
            let path = args.get(1).unwrap_or(&".".to_string());
            if let Some(extension) = args.get(2) {
                FindAdvanced::find_by_extension(path, extension)?;
            } else {
                println!("{}", "Usage: find ext <path> <extension>".warning());
            }
        },
        Some("large") => {
            let path = args.get(1).unwrap_or(&".".to_string());
            let size_mb = args.get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(100);
            FindAdvanced::find_large_files(path, size_mb)?;
        },
        Some("name") => {
            let path = args.get(1).unwrap_or(&".".to_string());
            let pattern = args.get(2);
            FindAdvanced::find_smart(path, pattern.map(|s| s.as_str()), None, None, None)?;
        },
        Some(path) => {
            // Default search in specified path
            FindAdvanced::find_smart(path, None, None, None, None)?;
        },
        None => {
            println!("{}", "Advanced Find Command".primary().bold());
            println!("{}", "  find <path>              - List all files in path".info());
            println!("{}", "  find name <path> <pattern> - Find by name pattern".info());
            println!("{}", "  find ext <path> <ext>    - Find by extension".info());
            println!("{}", "  find large <path> [MB]   - Find large files".info());
        }
    }
    
    Ok(())
}
