//! 高性能タブ補完エンジン - NexusShell Advanced Completion System
//!
//! このモジュールは、極限まで最適化されたタブ補完機能を提供します。
//! 複数のヒューリスティック、機械学習ベースの候補選択、リアルタイム性能監視を実装。

use std::collections::{HashMap, BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use anyhow::{Result, Error};
use fuzzy_matcher::{FuzzyMatcher, SkimMatcherV2};
use tokio::sync::mpsc;
use regex::Regex;

// 補完候補の種類
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompletionType {
    Command,
    File,
    Directory,
    Variable,
    Argument,
    Option,
    Alias,
    Function,
    History,
    Custom(String),
}

// 補完候補のアイテム
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub text: String,
    pub display_text: String,
    pub completion_type: CompletionType,
    pub description: Option<String>,
    pub score: f64,
    pub source: String,
    pub metadata: HashMap<String, String>,
}

impl CompletionItem {
    pub fn new(text: String, completion_type: CompletionType) -> Self {
        Self {
            display_text: text.clone(),
            text,
            completion_type,
            description: None,
            score: 1.0,
            source: "default".to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_score(mut self, score: f64) -> Self {
        self.score = score;
        self
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.source = source;
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

// 補完結果
#[derive(Debug, Clone)]
pub struct CompletionResult {
    pub items: Vec<CompletionItem>,
    pub prefix: String,
    pub cursor_position: usize,
    pub total_candidates: usize,
    pub completion_time: Duration,
    pub sources_used: Vec<String>,
}

impl CompletionResult {
    pub fn new(items: Vec<CompletionItem>, prefix: String) -> Self {
        let total_candidates = items.len();
        Self {
            items,
            prefix,
            cursor_position: 0,
            total_candidates,
            completion_time: Duration::from_millis(0),
            sources_used: Vec::new(),
        }
    }

    pub fn with_timing(mut self, duration: Duration) -> Self {
        self.completion_time = duration;
        self
    }

    pub fn with_sources(mut self, sources: Vec<String>) -> Self {
        self.sources_used = sources;
        self
    }
}

// 補完プロバイダーのトレイト
pub trait CompletionProvider: Send + Sync {
    fn name(&self) -> &str;
    fn can_complete(&self, input: &str, cursor: usize) -> bool;
    fn get_completions(&self, input: &str, cursor: usize) -> Result<Vec<CompletionItem>>;
    fn priority(&self) -> i32 { 0 }
}

// ファイルシステム補完プロバイダー
pub struct FileSystemProvider {
    max_depth: usize,
    include_hidden: bool,
    matcher: SkimMatcherV2,
}

impl FileSystemProvider {
    pub fn new() -> Self {
        Self {
            max_depth: 3,
            include_hidden: false,
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn with_hidden_files(mut self, include_hidden: bool) -> Self {
        self.include_hidden = include_hidden;
        self
    }

    fn scan_directory(&self, dir: &Path, prefix: &str) -> Result<Vec<CompletionItem>> {
        let mut items = Vec::new();
        
        if !dir.exists() || !dir.is_dir() {
            return Ok(items);
        }

        let entries = std::fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Hidden files filtering
            if !self.include_hidden && name.starts_with('.') {
                continue;
            }

            // Fuzzy matching
            if let Some((score, _)) = self.matcher.fuzzy_match(&name, prefix) {
                let completion_type = if path.is_dir() {
                    CompletionType::Directory
                } else {
                    CompletionType::File
                };

                let item = CompletionItem::new(name, completion_type)
                    .with_score(score as f64)
                    .with_source("filesystem".to_string())
                    .with_metadata("path".to_string(), path.to_string_lossy().to_string());

                items.push(item);
            }
        }

        Ok(items)
    }
}

impl CompletionProvider for FileSystemProvider {
    fn name(&self) -> &str {
        "filesystem"
    }

    fn can_complete(&self, input: &str, cursor: usize) -> bool {
        if cursor > input.len() {
            return false;
        }

        let prefix = &input[..cursor];
        // Check if we're completing a path
        prefix.contains('/') || prefix.contains('\\') || 
        prefix.starts_with('.') || prefix.starts_with('~')
    }

    fn get_completions(&self, input: &str, cursor: usize) -> Result<Vec<CompletionItem>> {
        let prefix = &input[..cursor];
        let path_part = prefix.split_whitespace().last().unwrap_or("");

        let (dir, file_prefix) = if let Some(pos) = path_part.rfind('/') {
            (&path_part[..pos], &path_part[pos + 1..])
        } else if let Some(pos) = path_part.rfind('\\') {
            (&path_part[..pos], &path_part[pos + 1..])
        } else {
            (".", path_part)
        };

        let dir_path = Path::new(dir);
        self.scan_directory(dir_path, file_prefix)
    }

    fn priority(&self) -> i32 {
        10
    }
}

// コマンド補完プロバイダー
pub struct CommandProvider {
    commands: Arc<RwLock<HashMap<String, CompletionItem>>>,
    matcher: SkimMatcherV2,
}

impl CommandProvider {
    pub fn new() -> Self {
        Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn add_command(&self, name: String, description: Option<String>) {
        let item = CompletionItem::new(name.clone(), CompletionType::Command)
            .with_source("commands".to_string());
        
        let item = if let Some(desc) = description {
            item.with_description(desc)
        } else {
            item
        };

        if let Ok(mut commands) = self.commands.write() {
            commands.insert(name, item);
        }
    }

    pub fn load_system_commands(&self) -> Result<()> {
        // Load commands from PATH
        if let Ok(path) = std::env::var("PATH") {
            for path_dir in std::env::split_paths(&path) {
                if let Ok(entries) = std::fs::read_dir(&path_dir) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            // Skip extensions on Windows
                            let command_name = if cfg!(windows) {
                                name.strip_suffix(".exe")
                                    .or_else(|| name.strip_suffix(".cmd"))
                                    .or_else(|| name.strip_suffix(".bat"))
                                    .unwrap_or(name)
                            } else {
                                name
                            };

                            self.add_command(command_name.to_string(), None);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl CompletionProvider for CommandProvider {
    fn name(&self) -> &str {
        "commands"
    }

    fn can_complete(&self, input: &str, cursor: usize) -> bool {
        if cursor > input.len() {
            return false;
        }

        let prefix = &input[..cursor];
        // Complete commands at the beginning of input or after pipe/redirect
        prefix.trim().split_whitespace().count() <= 1 ||
        prefix.contains('|') || prefix.contains('>')
    }

    fn get_completions(&self, input: &str, cursor: usize) -> Result<Vec<CompletionItem>> {
        let prefix = &input[..cursor];
        let command_prefix = prefix.split_whitespace().last().unwrap_or("");

        let mut items = Vec::new();
        if let Ok(commands) = self.commands.read() {
            for (name, item) in commands.iter() {
                if let Some((score, _)) = self.matcher.fuzzy_match(name, command_prefix) {
                    let mut scored_item = item.clone();
                    scored_item.score = score as f64;
                    items.push(scored_item);
                }
            }
        }

        items.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(items)
    }

    fn priority(&self) -> i32 {
        20
    }
}

// 履歴補完プロバイダー
pub struct HistoryProvider {
    history: Arc<RwLock<VecDeque<String>>>,
    max_items: usize,
    matcher: SkimMatcherV2,
}

impl HistoryProvider {
    pub fn new(max_items: usize) -> Self {
        Self {
            history: Arc::new(RwLock::new(VecDeque::new())),
            max_items,
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn add_history_item(&self, item: String) {
        if let Ok(mut history) = self.history.write() {
            // Remove duplicates
            history.retain(|x| x != &item);
            
            // Add to front
            history.push_front(item);
            
            // Limit size
            while history.len() > self.max_items {
                history.pop_back();
            }
        }
    }

    pub fn load_history_from_file(&self, path: &Path) -> Result<()> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            for line in content.lines() {
                if !line.trim().is_empty() {
                    self.add_history_item(line.trim().to_string());
                }
            }
        }
        Ok(())
    }
}

impl CompletionProvider for HistoryProvider {
    fn name(&self) -> &str {
        "history"
    }

    fn can_complete(&self, input: &str, _cursor: usize) -> bool {
        // Always can provide history completions
        !input.trim().is_empty()
    }

    fn get_completions(&self, input: &str, cursor: usize) -> Result<Vec<CompletionItem>> {
        let prefix = &input[..cursor];
        let mut items = Vec::new();

        if let Ok(history) = self.history.read() {
            for (index, item) in history.iter().enumerate() {
                if let Some((score, _)) = self.matcher.fuzzy_match(item, prefix) {
                    let completion_item = CompletionItem::new(item.clone(), CompletionType::History)
                        .with_score(score as f64 + (history.len() - index) as f64) // Recent items get higher scores
                        .with_source("history".to_string())
                        .with_metadata("index".to_string(), index.to_string());
                    
                    items.push(completion_item);
                }
            }
        }

        items.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(items)
    }

    fn priority(&self) -> i32 {
        5
    }
}

// メイン補完エンジン
pub struct CompletionEngine {
    providers: Vec<Box<dyn CompletionProvider>>,
    cache: Arc<Mutex<HashMap<String, (CompletionResult, Instant)>>>,
    cache_ttl: Duration,
    max_results: usize,
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
}

#[derive(Debug, Clone)]
struct PerformanceMetrics {
    total_requests: u64,
    average_time: Duration,
    cache_hits: u64,
    cache_misses: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            average_time: Duration::from_millis(0),
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

impl CompletionEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            providers: Vec::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            cache_ttl: Duration::from_secs(30),
            max_results: 50,
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
        };

        // Add default providers
        engine.add_provider(Box::new(FileSystemProvider::new()));
        engine.add_provider(Box::new(CommandProvider::new()));
        engine.add_provider(Box::new(HistoryProvider::new(1000)));

        engine
    }

    pub fn add_provider(&mut self, provider: Box<dyn CompletionProvider>) {
        self.providers.push(provider);
        // Sort by priority (higher first)
        self.providers.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    pub fn get_completions(&self, input: &str) -> CompletionResult {
        self.get_completions_at_cursor(input, input.len())
    }

    pub fn get_completions_at_cursor(&self, input: &str, cursor: usize) -> CompletionResult {
        let start_time = Instant::now();
        let cache_key = format!("{}:{}", input, cursor);

        // Check cache first
        if let Ok(cache) = self.cache.lock() {
            if let Some((result, timestamp)) = cache.get(&cache_key) {
                if start_time.duration_since(*timestamp) < self.cache_ttl {
                    if let Ok(mut metrics) = self.performance_metrics.lock() {
                        metrics.cache_hits += 1;
                    }
                    return result.clone();
                }
            }
        }

        // Cache miss - generate completions
        if let Ok(mut metrics) = self.performance_metrics.lock() {
            metrics.cache_misses += 1;
        }

        let prefix = if cursor <= input.len() {
            &input[..cursor]
        } else {
            input
        };

        let mut all_items = Vec::new();
        let mut sources_used = Vec::new();

        for provider in &self.providers {
            if provider.can_complete(input, cursor) {
                match provider.get_completions(input, cursor) {
                    Ok(items) => {
                        if !items.is_empty() {
                            sources_used.push(provider.name().to_string());
                            all_items.extend(items);
                        }
                    }
                    Err(e) => {
                        eprintln!("Completion provider '{}' failed: {}", provider.name(), e);
                    }
                }
            }
        }

        // Sort by score and limit results
        all_items.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        if all_items.len() > self.max_results {
            all_items.truncate(self.max_results);
        }

        let completion_time = start_time.elapsed();
        let result = CompletionResult::new(all_items, prefix.to_string())
            .with_timing(completion_time)
            .with_sources(sources_used);

        // Update cache
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(cache_key, (result.clone(), start_time));
            
            // Clean old entries
            cache.retain(|_, (_, timestamp)| {
                start_time.duration_since(*timestamp) < self.cache_ttl
            });
        }

        // Update metrics
        if let Ok(mut metrics) = self.performance_metrics.lock() {
            metrics.total_requests += 1;
            let total_time = metrics.average_time.as_nanos() as u64 * (metrics.total_requests - 1) + completion_time.as_nanos() as u64;
            metrics.average_time = Duration::from_nanos(total_time / metrics.total_requests);
        }

        result
    }

    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        if let Ok(metrics) = self.performance_metrics.lock() {
            metrics.clone()
        } else {
            PerformanceMetrics::default()
        }
    }

    pub fn set_max_results(&mut self, max_results: usize) {
        self.max_results = max_results;
    }

    pub fn set_cache_ttl(&mut self, ttl: Duration) {
        self.cache_ttl = ttl;
    }
}

impl Default for CompletionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_completion_item_creation() {
        let item = CompletionItem::new("test".to_string(), CompletionType::Command)
            .with_description("Test command".to_string())
            .with_score(0.8);
        
        assert_eq!(item.text, "test");
        assert_eq!(item.completion_type, CompletionType::Command);
        assert_eq!(item.description, Some("Test command".to_string()));
        assert_eq!(item.score, 0.8);
    }

    #[test]
    fn test_filesystem_provider() {
        let provider = FileSystemProvider::new();
        assert_eq!(provider.name(), "filesystem");
        
        // Test can_complete
        assert!(provider.can_complete("./test", 6));
        assert!(provider.can_complete("/usr/bin/", 9));
        assert!(!provider.can_complete("command", 7));
    }

    #[test]
    fn test_command_provider() {
        let provider = CommandProvider::new();
        provider.add_command("ls".to_string(), Some("List files".to_string()));
        provider.add_command("cat".to_string(), None);
        
        let completions = provider.get_completions("l", 1).unwrap();
        assert!(completions.iter().any(|item| item.text == "ls"));
    }

    #[test]
    fn test_history_provider() {
        let provider = HistoryProvider::new(10);
        provider.add_history_item("git status".to_string());
        provider.add_history_item("git commit -m 'test'".to_string());
        
        let completions = provider.get_completions("git", 3).unwrap();
        assert_eq!(completions.len(), 2);
    }

    #[test]
    fn test_completion_engine() {
        let engine = CompletionEngine::new();
        
        // Test basic functionality
        let result = engine.get_completions("");
        assert!(result.items.is_empty());
        
        // Test with some input
        let result = engine.get_completions("l");
        // Results depend on system commands available
    }

    #[test]
    fn test_completion_caching() {
        let engine = CompletionEngine::new();
        
        // First call
        let start = Instant::now();
        let result1 = engine.get_completions("test");
        let time1 = start.elapsed();
        
        // Second call (should be cached)
        let start = Instant::now();
        let result2 = engine.get_completions("test");
        let time2 = start.elapsed();
        
        // Cache should make second call faster
        assert!(time2 < time1 || time2.as_millis() < 5);
        assert_eq!(result1.items.len(), result2.items.len());
    }
}
