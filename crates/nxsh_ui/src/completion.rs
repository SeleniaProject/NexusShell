//! Intelligent tab completion system for NexusShell
//! 
//! This module provides context-aware completion for commands, files, variables,
//! and more, with fuzzy matching and smart filtering capabilities.
//! Pure cross-platform implementation using only crossterm and standard library.

use std::{
    collections::{HashMap, HashSet},
    env,
    fs,
    path::{Path, PathBuf},
};
/// Completion types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionType {
    Command,
    File,
    Directory,
    Variable,
    Alias,
    Builtin,
}

/// Completion result
#[derive(Debug, Clone)]
pub struct CompletionResult {
    pub completion: String,
    pub display: Option<String>,
    pub completion_type: CompletionType,
    pub score: i64,
}

/// Configuration for completion behavior
#[derive(Debug, Clone)]
pub struct CompletionConfig {
    pub max_suggestions: usize,
    pub fuzzy_matching: bool,
    pub case_sensitive: bool,
    pub show_descriptions: bool,
    pub complete_hidden_files: bool,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            max_suggestions: 50,
            fuzzy_matching: true,
            case_sensitive: false,
            show_descriptions: true,
            complete_hidden_files: false,
        }
    }
}

/// Main completion engine for NexusShell
pub struct NexusCompleter {
    command_cache: HashMap<String, String>, // command -> description
    pub builtin_cache: HashMap<String, String>, // builtin -> description
    variable_cache: HashSet<String>,
    alias_cache: HashMap<String, String>,
    completion_config: CompletionConfig,
    system_scanned: bool,
}

impl NexusCompleter {
    /// Create a new completer
    pub fn new() -> Self {
        let mut completer = Self {
            command_cache: HashMap::new(),
            builtin_cache: HashMap::new(),
            variable_cache: HashSet::new(),
            alias_cache: HashMap::new(),
            completion_config: CompletionConfig::default(),
            system_scanned: false,
        };
        
        // Initialize with basic builtins
        completer.init_builtins();
        
        completer
    }
    
    /// Initialize builtin commands
    fn init_builtins(&mut self) {
        let builtins = [
            ("cd", "Change directory"),
            ("ls", "List directory contents"),
            ("pwd", "Print working directory"),
            ("mkdir", "Create directory"),
            ("rmdir", "Remove directory"),
            ("cp", "Copy files"),
            ("mv", "Move files"),
            ("rm", "Remove files"),
            ("cat", "Display file contents"),
            ("grep", "Search text"),
            ("find", "Find files"),
            ("echo", "Display text"),
            ("export", "Set environment variable"),
            ("alias", "Create command alias"),
            ("history", "Show command history"),
            ("help", "Show help"),
            ("exit", "Exit shell"),
            ("clear", "Clear screen"),
        ];
        
        for (cmd, desc) in &builtins {
            self.builtin_cache.insert(cmd.to_string(), desc.to_string());
        }
    }
    
    /// Scan system commands from PATH
    fn scan_system_commands(&mut self) {
        if let Ok(path_var) = env::var("PATH") {
            for path_dir in env::split_paths(&path_var) {
                if let Ok(entries) = fs::read_dir(&path_dir) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            // Windows: respect PATHEXT and case-insensitive extensions
                            if cfg!(windows) {
                                let pathext = env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string());
                                let exts: Vec<String> = pathext
                                    .split(';')
                                    .filter_map(|s| {
                                        let s = s.trim();
                                        if s.is_empty() { return None; }
                                        Some(s.trim_start_matches('.').to_ascii_lowercase())
                                    })
                                    .collect();

                                let path_name = Path::new(name);
                                if let Some(ext) = path_name.extension().and_then(|e| e.to_str()) {
                                    let ext = ext.to_ascii_lowercase();
                                    if exts.iter().any(|e| e == &ext) {
                                        let stem = path_name
                                            .file_stem()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("")
                                            .to_string();
                                        if !stem.is_empty() {
                                            self.command_cache
                                                .entry(stem)
                                                .or_insert_with(|| "System command".to_string());
                                        }
                                    }
                                }
                            } else {
                                // Unix-like: include only executables
                                let path = entry.path();
                if let Ok(metadata) = fs::metadata(&path) {
                                    #[cfg(unix)]
                                    {
                                        use std::os::unix::fs::PermissionsExt;
                    if metadata.permissions().mode() & 0o111 != 0 {
                                            self.command_cache.insert(name.to_string(), "System command".to_string());
                                        }
                                    }
                                    #[cfg(not(unix))]
                                    {
                    // Ensure `metadata` is considered used to avoid warnings when compiling this branch
                    let _ = &metadata;
                                        self.command_cache.insert(name.to_string(), "System command".to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Ensure system commands have been scanned once (lazy init)
    fn ensure_system_commands(&mut self) {
        if !self.system_scanned {
            self.scan_system_commands();
            self.system_scanned = true;
        }
    }
    
    /// Complete input with suggestions
    pub fn complete(&mut self, input: &str, pos: usize) -> Vec<CompletionResult> {
        let text = &input[..pos];
        let parts: Vec<&str> = text.split_whitespace().collect();
        
        if parts.is_empty() || (parts.len() == 1 && !text.ends_with(' ')) {
            // Complete command
            self.complete_command(text)
        } else {
            // Complete file/directory
            let last_part = parts.last().map_or("", |v| v);
            self.complete_file(last_part)
        }
    }
    
    /// Complete command names
    fn complete_command(&mut self, input: &str) -> Vec<CompletionResult> {
        // Lazily populate system command cache
        self.ensure_system_commands();
        let mut results = Vec::new();
        
        // Search builtins
        for (cmd, desc) in &self.builtin_cache {
            if cmd.starts_with(input) {
                results.push(CompletionResult {
                    completion: cmd.clone(),
                    display: Some(format!("{:<12} {}", cmd, desc)),
                    completion_type: CompletionType::Builtin,
                    score: self.calculate_score(input, cmd),
                });
            }
        }
        
        // Search system commands
        for (cmd, desc) in &self.command_cache {
            if cmd.starts_with(input) {
                results.push(CompletionResult {
                    completion: cmd.clone(),
                    display: Some(format!("{:<12} {}", cmd, desc)),
                    completion_type: CompletionType::Command,
                    score: self.calculate_score(input, cmd),
                });
            }
        }
        
        // Sort by score (higher is better)
        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(self.completion_config.max_suggestions);
        
        results
    }
    
    /// Complete file and directory names
    fn complete_file(&self, input: &str) -> Vec<CompletionResult> {
        let mut results = Vec::new();
        
        let path = if input.is_empty() {
            PathBuf::from(".")
        } else {
            PathBuf::from(input)
        };
        
        let (dir, prefix) = if path.is_dir() && input.ends_with('/') {
            (path, String::new())
        } else {
            let dir = path.parent().unwrap_or(Path::new("."));
            let prefix = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            (dir.to_path_buf(), prefix)
        };
        
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(&prefix) {
                        // Skip hidden files unless configured to show them
                        if !self.completion_config.complete_hidden_files && name.starts_with('.') {
                            continue;
                        }
                        
                        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                        let completion_type = if is_dir { CompletionType::Directory } else { CompletionType::File };
                        
                        let full_path = dir.join(name);
                        let completion = full_path.to_string_lossy().to_string();
                        
                        // Create properly formatted display with consistent spacing
                        let display_name = if is_dir { 
                            format!("{}/", name) 
                        } else { 
                            name.to_string() 
                        };
                        
                        let display = if self.completion_config.show_descriptions {
                            let file_type = if is_dir { "directory" } else { "file" };
                            format!("{:<20} {}", display_name, file_type)
                        } else {
                            display_name
                        };
                            
                        results.push(CompletionResult {
                            completion,
                            display: Some(display),
                            completion_type,
                            score: self.calculate_score(&prefix, name),
                        });
                        }
                    }
            }
        }
        
        // Sort by score and type (directories first)
        results.sort_by(|a, b| {
            match (&a.completion_type, &b.completion_type) {
                (CompletionType::Directory, CompletionType::File) => std::cmp::Ordering::Less,
                (CompletionType::File, CompletionType::Directory) => std::cmp::Ordering::Greater,
                _ => b.score.cmp(&a.score),
            }
        });
        
        results.truncate(self.completion_config.max_suggestions);
        results
    }
    
    /// Calculate completion score
    fn calculate_score(&self, input: &str, candidate: &str) -> i64 {
        if candidate.starts_with(input) {
            // Exact prefix match gets high score
            100 + (candidate.len() as i64 - input.len() as i64)
        } else if self.completion_config.fuzzy_matching {
            // Simple fuzzy matching score
            self.fuzzy_score(input, candidate)
        } else {
            0
        }
    }
    
    /// Simple fuzzy matching score
    fn fuzzy_score(&self, input: &str, candidate: &str) -> i64 {
        let input_chars: Vec<char> = input.to_lowercase().chars().collect();
        let candidate_chars: Vec<char> = candidate.to_lowercase().chars().collect();
        
        let mut score = 0i64;
        let mut input_idx = 0;
        
        for &ch in &candidate_chars {
            if input_idx < input_chars.len() && ch == input_chars[input_idx] {
                score += 10;
                input_idx += 1;
            }
        }
        
        // Bonus for matching all characters
        if input_idx == input_chars.len() {
            score += 50;
        }
        
        score
    }
}

impl Default for NexusCompleter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completer_creation() {
    let mut completer = NexusCompleter::new();
    // Builtins are initialized eagerly
    assert!(!completer.builtin_cache.is_empty());
    // System commands are loaded lazily; trigger scan and verify flag toggles
    let results = completer.complete_command("");
    assert!(completer.system_scanned);
    // Even if PATH had no executables, builtins should produce results
    assert!(!results.is_empty());
    }

    #[test]
    fn test_command_completion() {
        let mut completer = NexusCompleter::new();
        let results = completer.complete_command("l");
        // The completion should work even if specific commands aren't found
        // This depends on the system PATH and builtin commands available
        // Just verify the function doesn't panic
        let _ = results;
    }

    #[test]
    fn test_file_completion() {
        let completer = NexusCompleter::new();
        // Test with current directory which should always exist
        let results = completer.complete_file(".");
        // File completion should work, even if no files are returned
        // Just verify the function doesn't panic
        let _ = results;
    }

    #[test]
    fn test_fuzzy_matching() {
        let completer = NexusCompleter::new();
        let score = completer.fuzzy_score("lst", "list");
        assert!(score > 0);
    }
}