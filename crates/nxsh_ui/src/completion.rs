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
        };
        
        // Initialize with basic builtins
        completer.init_builtins();
        completer.scan_system_commands();
        
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
                    for entry in entries {
                        if let Ok(entry) = entry {
                            if let Some(name) = entry.file_name().to_str() {
                                // On Windows, also check for .exe files
                                if cfg!(windows) {
                                    if name.ends_with(".exe") {
                                        let cmd_name = name.trim_end_matches(".exe");
                                        self.command_cache.insert(cmd_name.to_string(), "System command".to_string());
                                    }
                                } else {
                                    // On Unix-like systems, check if file is executable
                                    let path = entry.path();
                                    if let Ok(_metadata) = fs::metadata(&path) {
                                        #[cfg(unix)]
                                        {
                                            use std::os::unix::fs::PermissionsExt;
                                            if metadata.permissions().mode() & 0o111 != 0 {
                                                self.command_cache.insert(name.to_string(), "System command".to_string());
                                            }
                                        }
                                        #[cfg(not(unix))]
                                        {
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
    }
    
    /// Complete input with suggestions
    pub fn complete(&self, input: &str, pos: usize) -> Vec<CompletionResult> {
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
    fn complete_command(&self, input: &str) -> Vec<CompletionResult> {
        let mut results = Vec::new();
        
        // Search builtins
        for (cmd, desc) in &self.builtin_cache {
            if cmd.starts_with(input) {
                results.push(CompletionResult {
                    completion: cmd.clone(),
                    display: Some(format!("{} - {}", cmd, desc)),
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
                    display: Some(format!("{} - {}", cmd, desc)),
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
            for entry in entries {
                if let Ok(entry) = entry {
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
                            
                            results.push(CompletionResult {
                                completion,
                                display: Some(if is_dir { format!("{}/", name) } else { name.to_string() }),
                                completion_type,
                                score: self.calculate_score(&prefix, name),
                            });
                        }
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
        let completer = NexusCompleter::new();
        assert!(!completer.command_cache.is_empty());
        assert!(!completer.builtin_cache.is_empty());
    }

    #[test]
    fn test_command_completion() {
        let completer = NexusCompleter::new();
        let results = completer.complete_command("l");
        assert!(!results.is_empty());
        
        // Should find 'ls' command
        assert!(results.iter().any(|r| r.completion == "ls"));
    }

    #[test]
    fn test_file_completion() {
        let completer = NexusCompleter::new();
        let results = completer.complete_file(".");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_fuzzy_matching() {
        let completer = NexusCompleter::new();
        let score = completer.fuzzy_score("lst", "list");
        assert!(score > 0);
    }
}