use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
    path::Path,
};
use anyhow::Result;
// removed unused import

/// High-performance completion system with caching
#[derive(Debug)]
pub struct CompletionEngine {
    cache: Arc<RwLock<CompletionCache>>,
    stats: Arc<RwLock<CompletionStats>>,
    config: CompletionConfig,
}

impl Default for CompletionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionEngine {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(CompletionCache::new())),
            stats: Arc::new(RwLock::new(CompletionStats::default())),
            config: CompletionConfig::default(),
        }
    }

    pub fn with_config(config: CompletionConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(CompletionCache::new())),
            stats: Arc::new(RwLock::new(CompletionStats::default())),
            config,
        }
    }

    /// Get completions for input (target: <1ms)
    pub fn get_completions(&self, input: &str, context: &CompletionContext) -> Result<Vec<Completion>> {
        let start = Instant::now();
        
        // Check cache first
        if let Some(cached) = self.get_cached_completions(input, context) {
            self.record_completion(start.elapsed(), cached.len(), true);
            return Ok(cached);
        }

        // Generate new completions
        let mut completions = self.generate_completions(input, context)?;

        // Lightweight command-specific suggestions
        completions.extend(self.complete_builtin_flags(&context.command_line));
        
        // Cache results
        self.cache_completions(input, context, &completions);
        
        let duration = start.elapsed();
        self.record_completion(duration, completions.len(), false);
        
        Ok(completions)
    }

    /// Clear completion cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Get completion statistics
    pub fn stats(&self) -> CompletionStats {
        self.stats.read().unwrap().clone()
    }

    fn get_cached_completions(&self, input: &str, context: &CompletionContext) -> Option<Vec<Completion>> {
        if !self.config.enable_cache {
            return None;
        }

        if let Ok(cache) = self.cache.read() {
            let cache_key = self.create_cache_key(input, context);
            if let Some(entry) = cache.get(&cache_key) {
                if entry.is_valid() {
                    return Some(entry.completions.clone());
                }
            }
        }
        None
    }

    fn cache_completions(&self, input: &str, context: &CompletionContext, completions: &[Completion]) {
        if !self.config.enable_cache || completions.len() > self.config.max_cache_items {
            return;
        }

        if let Ok(mut cache) = self.cache.write() {
            let cache_key = self.create_cache_key(input, context);
            let entry = CacheEntry::new(completions.to_vec(), self.config.cache_ttl);
            cache.insert(cache_key, entry);
            
            // Cleanup old entries if cache is too large
            if cache.len() > self.config.max_cache_size {
                cache.cleanup_old_entries(self.config.max_cache_size / 2);
            }
        }
    }

    fn create_cache_key(&self, input: &str, context: &CompletionContext) -> String {
        format!("{}:{}:{:?}", input, context.working_dir.to_string_lossy(), context.completion_type)
    }

    fn generate_completions(&self, input: &str, context: &CompletionContext) -> Result<Vec<Completion>> {
        match context.completion_type {
            CompletionType::Command => self.complete_commands(input),
            CompletionType::File => self.complete_files(input, &context.working_dir),
            CompletionType::Directory => self.complete_directories(input, &context.working_dir),
            CompletionType::Variable => self.complete_variables(input),
            CompletionType::Alias => self.complete_aliases(input),
            CompletionType::History => self.complete_history(input),
        }
    }

    /// Provide simple flag/subcommand suggestions for certain builtins based on prefix heuristics.
    /// This is intentionally lightweight to keep completion fast and dependency-free.
    fn complete_builtin_flags(&self, input: &str) -> Vec<Completion> {
        let trimmed = input.trim_start();
        // Match patterns like: "timedatectl ", "timedatectl t", etc.
        const TDCT_SUBCMDS: &[&str] = &[
            "status", "show", "timesync-status", "show-timesync", "statistics",
            "add-ntp-server", "remove-ntp-server",
        ];
        const TDCT_FLAGS: &[&str] = &[
            "--json", "-J", "--monitor", "-h", "--help",
        ];

        let mut out: Vec<Completion> = Vec::new();
        if trimmed.starts_with("timedatectl ") {
            let after = trimmed.strip_prefix("timedatectl ").unwrap_or("");
            let needle = after.trim_start();
            for s in TDCT_SUBCMDS {
                if s.starts_with(needle) {
                    out.push(Completion {
                        text: s.to_string(),
                        display: s.to_string(),
                        completion_type: CompletionType::Command,
                        description: Some("timedatectl subcommand".to_string()),
                        score: self.calculate_score(needle, s),
                    });
                }
            }
            for f in TDCT_FLAGS {
                if f.starts_with(needle) {
                    out.push(Completion {
                        text: f.to_string(),
                        display: f.to_string(),
                        completion_type: CompletionType::Command,
                        description: Some("timedatectl flag".to_string()),
                        score: self.calculate_score(needle, f),
                    });
                }
            }
        }

        // zstd flags (compression via external binary; decompression via internal path)
        if trimmed.starts_with("zstd ") {
            let after = trimmed.strip_prefix("zstd ").unwrap_or("");
            let needle = after.trim_start();
            const ZSTD_FLAGS: &[&str] = &[
                "-d", "-z", "-c", "-t", "-q", "-v", "-T", "--help", "--version",
            ];
            for f in ZSTD_FLAGS {
                if f.starts_with(needle) {
                    out.push(Completion {
                        text: f.to_string(),
                        display: f.to_string(),
                        completion_type: CompletionType::Command,
                        description: Some("zstd flag".to_string()),
                        score: self.calculate_score(needle, f),
                    });
                }
            }
        }

        // unzstd flags (pure-Rust decompression)
        if trimmed.starts_with("unzstd ") {
            let after = trimmed.strip_prefix("unzstd ").unwrap_or("");
            let needle = after.trim_start();
            const UNZSTD_FLAGS: &[&str] = &[
                "-k", "--keep", "-f", "--force", "-c", "--stdout", "-t", "--test", "-q", "--quiet", "-v", "--verbose",
            ];
            for f in UNZSTD_FLAGS {
                if f.starts_with(needle) {
                    out.push(Completion {
                        text: f.to_string(),
                        display: f.to_string(),
                        completion_type: CompletionType::Command,
                        description: Some("unzstd flag".to_string()),
                        score: self.calculate_score(needle, f),
                    });
                }
            }
        }
        out
    }

    fn complete_commands(&self, input: &str) -> Result<Vec<Completion>> {
        let mut completions = Vec::new();
        
        // Built-in commands
        let builtins = [
            "cd", "ls", "pwd", "echo", "cat", "grep", "find", "ps", "kill", 
            "cp", "mv", "rm", "mkdir", "rmdir", "touch", "chmod", "chown",
            "tar", "gzip", "gunzip", "curl", "wget", "git", "ssh", "scp",
            // Extended builtins frequently used in NexusShell
            "zstd", "unzstd", "zip", "unzip", "bzip2", "bunzip2", "xz", "unxz",
            "timedatectl",
        ];
        
        for builtin in &builtins {
            if builtin.starts_with(input) {
                completions.push(Completion {
                    text: builtin.to_string(),
                    display: builtin.to_string(),
                    completion_type: CompletionType::Command,
                    description: Some(format!("Built-in command: {builtin}")),
                    score: self.calculate_score(input, builtin),
                });
            }
        }
        
        // System commands from PATH
        if input.len() >= 2 {  // Only search PATH for inputs >= 2 chars
            completions.extend(self.complete_path_commands(input)?);
        }
        
        // Sort by score (higher is better)
        completions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(completions)
    }

    /// Truncate a long string for display safely
    fn truncate_display(&self, s: &str, max_len: usize) -> String {
        if s.len() <= max_len { return s.to_string(); }
        let keep = max_len.saturating_sub(3);
        format!("{}...", &s[..keep])
    }

    fn complete_path_commands(&self, input: &str) -> Result<Vec<Completion>> {
        let mut completions = Vec::new();
        
        if let Ok(path) = std::env::var("PATH") {
            let paths: Vec<&str> = path.split(if cfg!(windows) { ';' } else { ':' }).collect();
            
            // Ultra-fast mode: only search first 3 PATH entries and limit results
            for path_dir in paths.iter().take(3) {  
                if let Ok(entries) = std::fs::read_dir(path_dir) {
                    for entry in entries.flatten().take(20) {  // Limit to 20 entries per directory
                        if let Ok(file_name) = entry.file_name().into_string() {
                            if file_name.starts_with(input) {
                                // Skip expensive metadata check - assume executable
                                completions.push(Completion {
                                    text: file_name.clone(),
                                    display: file_name.clone(),
                                    completion_type: CompletionType::Command,
                                    description: Some(format!("Command from {path_dir}")),
                                    score: self.calculate_score(input, &file_name),
                                });
                                
                                if completions.len() >= 10 {  // Limit total results for speed
                                    return Ok(completions);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(completions)
    }

    fn complete_files(&self, input: &str, working_dir: &Path) -> Result<Vec<Completion>> {
        let mut completions = Vec::new();
        
        let (dir_path, file_prefix) = if let Some(last_slash) = input.rfind('/') {
            let dir_part = &input[..last_slash + 1];
            let file_part = &input[last_slash + 1..];
            (working_dir.join(dir_part), file_part)
        } else {
            (working_dir.to_path_buf(), input)
        };
        
        if let Ok(entries) = std::fs::read_dir(&dir_path) {
            for entry in entries.flatten().take(25) {  // Limit to 25 entries for speed
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.starts_with(file_prefix) && !file_name.starts_with('.') {
                        // Skip expensive is_dir() check for speed - infer from name
                        let is_dir = entry.path().is_dir();
                        let display_name = if is_dir {
                            format!("{file_name}/")
                        } else {
                            file_name.clone()
                        };
                        
                        completions.push(Completion {
                            text: file_name.clone(),
                            display: display_name,
                            completion_type: if is_dir { CompletionType::Directory } else { CompletionType::File },
                            description: if is_dir { Some("Directory".to_string()) } else { Some("File".to_string()) },
                            score: self.calculate_score(file_prefix, &file_name),
                        });
                        
                        if completions.len() >= 15 {  // Reduced limit for speed
                            break;
                        }
                    }
                }
            }
        }
        
        completions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(completions)
    }

    fn complete_directories(&self, input: &str, working_dir: &Path) -> Result<Vec<Completion>> {
        let completions = self.complete_files(input, working_dir)?;
        Ok(completions.into_iter()
           .filter(|c| c.completion_type == CompletionType::Directory)
           .collect())
    }

    fn complete_variables(&self, input: &str) -> Result<Vec<Completion>> {
        let mut completions = Vec::new();
        
        // Environment variables
        for (key, value) in std::env::vars() {
            if key.starts_with(input.trim_start_matches('$')) {
                completions.push(Completion {
                    text: format!("${key}"),
                    display: format!("${} = {}", key, if value.len() > 30 { 
                        format!("{}...", &value[..27]) 
                    } else { 
                        value 
                    }),
                    completion_type: CompletionType::Variable,
                    description: Some("Environment variable".to_string()),
                    score: self.calculate_score(input, &key),
                });
                
                if completions.len() >= self.config.max_results {
                    break;
                }
            }
        }
        
        completions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(completions)
    }

    fn complete_aliases(&self, input: &str) -> Result<Vec<Completion>> {
        // Integrate with shell alias system via environment snapshot for HAL context.
        // Aliases are exposed to subprocesses as NXSH_ALIAS_*=name=value pairs optionally,
        // and also used by interactive UI. Here we support both sources:
        // 1) Environment variables NXSH_ALIAS_* (cheap, no cross-crate deps)
        // 2) Fallback: parse common alias export file if configured (NXSH_ALIAS_FILE)

        let mut completions: Vec<Completion> = Vec::new();
        let needle = input.trim();

        // Source 1: Environment variables
        for (key, value) in std::env::vars() {
            if let Some(rest) = key.strip_prefix("NXSH_ALIAS_") {
                // Expected form: NXSH_ALIAS_<NAME>=<value>
                let alias_name = rest.to_lowercase();
                if alias_name.starts_with(needle) {
                    completions.push(Completion {
                        text: alias_name.clone(),
                        display: format!("{} -> {}", alias_name, self.truncate_display(&value, 40)),
                        completion_type: CompletionType::Alias,
                        description: Some("Alias".to_string()),
                        score: self.calculate_score(needle, &alias_name),
                    });
                }
            }
        }

        // Source 2: Optional alias file (line format: name=value)
        if let Ok(path) = std::env::var("NXSH_ALIAS_FILE") {
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines() {
                    if let Some((name, val)) = line.split_once('=') {
                        let name = name.trim();
                        if !name.is_empty() && name.starts_with(needle) {
                            completions.push(Completion {
                                text: name.to_string(),
                                display: format!("{} -> {}", name, self.truncate_display(val.trim(), 40)),
                                completion_type: CompletionType::Alias,
                                description: Some("Alias (file)".to_string()),
                                score: self.calculate_score(needle, name),
                            });
                        }
                    }
                }
            }
        }

        // Sort by score descending
        completions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        // Respect max_results
        completions.truncate(self.config.max_results);

        Ok(completions)
    }

    fn complete_history(&self, input: &str) -> Result<Vec<Completion>> {
        let needle = input.trim();
        let mut completions: Vec<Completion> = Vec::new();

        // Integration path A: direct NXSH_HISTORY environment (newline-separated recent commands)
        if let Ok(hist_env) = std::env::var("NXSH_HISTORY") {
            for line in hist_env.lines().rev().take(self.config.max_results * 2) {
                let cmd = line.trim();
                if cmd.is_empty() { continue; }
                if needle.is_empty() || cmd.starts_with(needle) || cmd.contains(needle) {
                    completions.push(Completion {
                        text: cmd.to_string(),
                        display: self.truncate_display(cmd, 60),
                        completion_type: CompletionType::History,
                        description: Some("History".to_string()),
                        score: self.calculate_score(needle, cmd),
                    });
                    if completions.len() >= self.config.max_results { break; }
                }
            }
        } else if let Ok(path) = std::env::var("NXSH_HISTORY_FILE") {
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines().rev() {
                    let cmd = line.trim();
                    if cmd.is_empty() { continue; }
                    if needle.is_empty() || cmd.starts_with(needle) || cmd.contains(needle) {
                        completions.push(Completion {
                            text: cmd.to_string(),
                            display: self.truncate_display(cmd, 60),
                            completion_type: CompletionType::History,
                            description: Some("History (file)".to_string()),
                            score: self.calculate_score(needle, cmd),
                        });
                        if completions.len() >= self.config.max_results { break; }
                    }
                }
            }
        }

        completions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        completions.truncate(self.config.max_results);
        Ok(completions)
    }

    fn calculate_score(&self, input: &str, candidate: &str) -> f64 {
        if candidate == input {
            return 100.0;
        }
        
        if candidate.starts_with(input) {
            let prefix_score = 50.0;
            let length_bonus = (input.len() as f64 / candidate.len() as f64) * 20.0;
            return prefix_score + length_bonus;
        }
        
        if candidate.contains(input) {
            return 25.0;
        }
        
        // Fuzzy matching score
        let mut score = 0.0;
        let mut input_chars = input.chars();
        let mut current_char = input_chars.next();
        
        for candidate_char in candidate.chars() {
            if let Some(ch) = current_char {
                if ch == candidate_char {
                    score += 10.0;
                    current_char = input_chars.next();
                }
            }
        }
        
        score
    }

    fn record_completion(&self, duration: Duration, count: usize, from_cache: bool) {
        if let Ok(mut stats) = self.stats.write() {
            stats.total_requests += 1;
            stats.total_time += duration;
            stats.total_completions += count;
            
            if from_cache {
                stats.cache_hits += 1;
            } else {
                stats.cache_misses += 1;
            }
            
            if duration < stats.fastest_completion || stats.fastest_completion == Duration::ZERO {
                stats.fastest_completion = duration;
            }
            
            if duration > stats.slowest_completion {
                stats.slowest_completion = duration;
            }
        }
    }
}

/// Configuration for completion engine
#[derive(Debug, Clone)]
pub struct CompletionConfig {
    pub enable_cache: bool,
    pub cache_ttl: Duration,
    pub max_cache_size: usize,
    pub max_cache_items: usize,
    pub max_results: usize,
    pub min_chars_for_path_search: usize,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_ttl: Duration::from_secs(300),  // 5 minutes
            max_cache_size: 1000,
            max_cache_items: 100,
            max_results: 15,  // Reduced for speed
            min_chars_for_path_search: 2,
        }
    }
}

/// Completion context information
#[derive(Debug, Clone)]
pub struct CompletionContext {
    pub completion_type: CompletionType,
    pub working_dir: std::path::PathBuf,
    pub command_line: String,
    pub cursor_position: usize,
}

/// Types of completions
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionType {
    Command,
    File,
    Directory,
    Variable,
    Alias,
    History,
}

/// Individual completion result
#[derive(Debug, Clone)]
pub struct Completion {
    pub text: String,
    pub display: String,
    pub completion_type: CompletionType,
    pub description: Option<String>,
    pub score: f64,
}

/// Completion cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    completions: Vec<Completion>,
    created: Instant,
    ttl: Duration,
}

impl CacheEntry {
    fn new(completions: Vec<Completion>, ttl: Duration) -> Self {
        Self {
            completions,
            created: Instant::now(),
            ttl,
        }
    }

    fn is_valid(&self) -> bool {
        self.created.elapsed() < self.ttl
    }
}

/// Completion cache
#[derive(Debug)]
struct CompletionCache {
    entries: HashMap<String, CacheEntry>,
}

impl CompletionCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn get(&self, key: &str) -> Option<&CacheEntry> {
        self.entries.get(key)
    }

    fn insert(&mut self, key: String, entry: CacheEntry) {
        self.entries.insert(key, entry);
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn clear(&mut self) {
        self.entries.clear();
    }

    fn cleanup_old_entries(&mut self, target_size: usize) {
        if self.entries.len() <= target_size {
            return;
        }

        let mut entries_with_age: Vec<_> = self.entries.iter()
            .map(|(k, v)| (k.clone(), v.created.elapsed()))
            .collect();
        
        entries_with_age.sort_by(|a, b| b.1.cmp(&a.1));
        
        let to_remove = self.entries.len() - target_size;
        for (key, _) in entries_with_age.into_iter().take(to_remove) {
            self.entries.remove(&key);
        }
    }
}

/// Completion engine statistics
#[derive(Debug, Clone, Default)]
pub struct CompletionStats {
    pub total_requests: u64,
    pub total_completions: usize,
    pub total_time: Duration,
    pub fastest_completion: Duration,
    pub slowest_completion: Duration,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl CompletionStats {
    pub fn avg_completion_time(&self) -> Duration {
        if self.total_requests > 0 {
            self.total_time / self.total_requests as u32
        } else {
            Duration::ZERO
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn avg_completions_per_request(&self) -> f64 {
        if self.total_requests > 0 {
            self.total_completions as f64 / self.total_requests as f64
        } else {
            0.0
        }
    }

    pub fn performance_target_met(&self) -> bool {
        self.avg_completion_time() < Duration::from_millis(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_engine() {
        let engine = CompletionEngine::new();
        let context = CompletionContext {
            completion_type: CompletionType::Command,
            working_dir: std::env::current_dir().unwrap(),
            command_line: "l".to_string(),
            cursor_position: 1,
        };

        let completions = engine.get_completions("l", &context).unwrap();
        assert!(!completions.is_empty());
        
        // Should contain "ls"
        let ls_completion = completions.iter().find(|c| c.text == "ls");
        assert!(ls_completion.is_some());
    }

    #[test]
    fn test_completion_cache() {
        let config = CompletionConfig {
            enable_cache: true,
            ..Default::default()
        };
        let engine = CompletionEngine::with_config(config);
        let context = CompletionContext {
            completion_type: CompletionType::Command,
            working_dir: std::env::current_dir().unwrap(),
            command_line: "e".to_string(),
            cursor_position: 1,
        };

        // First request - should miss cache
        let completions1 = engine.get_completions("e", &context).unwrap();
        
        // Second request - should hit cache
        let completions2 = engine.get_completions("e", &context).unwrap();
        
        assert_eq!(completions1.len(), completions2.len());
        
        let stats = engine.stats();
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
    }

    #[test]
    fn test_completion_scoring() {
        let engine = CompletionEngine::new();
        
        // Exact match should have highest score
        assert_eq!(engine.calculate_score("ls", "ls"), 100.0);
        
        // Prefix match should have high score
        let prefix_score = engine.calculate_score("l", "ls");
        assert!(prefix_score > 50.0);
        assert!(prefix_score < 100.0);
        
        // Contains match should have medium score
        let contains_score = engine.calculate_score("s", "ls");
        assert!(contains_score > 0.0);
        assert!(contains_score < prefix_score);
    }

    #[test]
    fn test_file_completion() {
        let engine = CompletionEngine::new();
        let temp_dir = std::env::temp_dir();
        
        let context = CompletionContext {
            completion_type: CompletionType::File,
            working_dir: temp_dir,
            command_line: "".to_string(),
            cursor_position: 0,
        };

        let completions = engine.get_completions("", &context).unwrap();
        // Should return some files from temp directory
        // This test might be environment-specific
    }
}
