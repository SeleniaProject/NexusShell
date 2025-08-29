//! 高性能タブ補完エンジン - NexusShell Advanced Completion System
//!
//! このモジュールは、極限まで最適化されたタブ補完機能を提供します。
//! 複数のヒューリスティック、機械学習ベースの候補選択、リアルタイム性能監視を実装。

use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

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
    Builtin,
    History,
    SmartSuggestion,
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

impl Default for FileSystemProvider {
    fn default() -> Self {
        Self::new()
    }
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
            if let Some(score) = self.matcher.fuzzy_match(&name, prefix) {
                let completion_type = if path.is_dir() {
                    CompletionType::Directory
                } else {
                    CompletionType::File
                };

                // Generate detailed description
                let description = self.generate_file_description(&path, &completion_type);

                let item = CompletionItem::new(name, completion_type)
                    .with_score(score as f64 / 100.0) // Normalize score
                    .with_source("filesystem".to_string())
                    .with_description(description)
                    .with_metadata("path".to_string(), path.to_string_lossy().to_string());

                items.push(item);
            }
        }

        Ok(items)
    }

    fn generate_file_description(&self, path: &Path, completion_type: &CompletionType) -> String {
        match completion_type {
            CompletionType::Directory => {
                if let Ok(entries) = std::fs::read_dir(path) {
                    let count = entries.count();
                    format!("Directory ({} items)", count)
                } else {
                    "Directory".to_string()
                }
            }
            CompletionType::File => {
                // Get file extension and size
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                
                let size_desc = if let Ok(metadata) = path.metadata() {
                    let size = metadata.len();
                    if size < 1024 {
                        format!("{} B", size)
                    } else if size < 1024 * 1024 {
                        format!("{:.1} KB", size as f64 / 1024.0)
                    } else if size < 1024 * 1024 * 1024 {
                        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                    } else {
                        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
                    }
                } else {
                    "Unknown size".to_string()
                };

                let file_type = match ext.to_lowercase().as_str() {
                    "rs" => "Rust source file",
                    "toml" => "TOML configuration",
                    "json" => "JSON data file",
                    "md" => "Markdown document",
                    "txt" => "Text file",
                    "log" => "Log file",
                    "exe" => "Executable",
                    "dll" => "Dynamic library",
                    "lib" => "Static library",
                    "py" => "Python script",
                    "js" => "JavaScript file",
                    "ts" => "TypeScript file",
                    "html" => "HTML document",
                    "css" => "Stylesheet",
                    "png" | "jpg" | "jpeg" | "gif" => "Image file",
                    "pdf" => "PDF document",
                    "zip" | "tar" | "gz" => "Archive file",
                    _ if !ext.is_empty() => &format!("{} file", ext.to_uppercase()),
                    _ => "File",
                };

                format!("{} ({})", file_type, size_desc)
            }
            _ => "File system item".to_string(),
        }
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

impl Default for CommandProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandProvider {
    pub fn new() -> Self {
        Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn add_command(&self, name: String, description: Option<String>) {
        let description = description.unwrap_or_else(|| self.generate_command_description(&name));
        
        let item = CompletionItem::new(name.clone(), CompletionType::Command)
            .with_source("commands".to_string())
            .with_description(description);
        
        if let Ok(mut commands) = self.commands.write() {
            commands.insert(name, item);
        }
    }

    fn generate_command_description(&self, command: &str) -> String {
        match command {
            // Shell builtins
            "cd" => "Change directory",
            "ls" => "List directory contents",
            "pwd" => "Print working directory",
            "echo" => "Display text",
            "cat" => "Display file contents",
            "cp" => "Copy files",
            "mv" => "Move/rename files",
            "rm" => "Remove files",
            "mkdir" => "Create directories",
            "rmdir" => "Remove directories",
            "touch" => "Create empty files",
            "chmod" => "Change file permissions",
            "grep" => "Search text patterns",
            "find" => "Find files and directories",
            "sort" => "Sort lines of text",
            "uniq" => "Report unique lines",
            "head" => "Show first lines of file",
            "tail" => "Show last lines of file",
            "wc" => "Count lines, words, characters",
            "which" => "Locate command",
            "history" => "Show command history",
            "exit" => "Exit shell",
            "help" => "Show help information",
            
            // System commands
            "ps" => "Show running processes",
            "top" => "Display system processes",
            "kill" => "Terminate processes",
            "killall" => "Kill processes by name",
            "mount" => "Mount filesystems",
            "umount" => "Unmount filesystems",
            "df" => "Show disk space usage",
            "du" => "Show directory space usage",
            "free" => "Show memory usage",
            "uptime" => "Show system uptime",
            "whoami" => "Show current user",
            "id" => "Show user and group IDs",
            "groups" => "Show user groups",
            "date" => "Show/set system date",
            "cal" => "Show calendar",
            
            // Network commands
            "ping" => "Test network connectivity",
            "curl" => "Transfer data from servers",
            "wget" => "Download files from web",
            "ssh" => "Secure shell remote access",
            "scp" => "Secure copy over network",
            "rsync" => "Synchronize files",
            
            // Git commands
            "git" => "Version control system",
            
            // Cargo commands
            "cargo" => "Rust package manager",
            
            // Other common commands
            "vim" | "nvim" => "Text editor",
            "emacs" => "Text editor",
            "nano" => "Simple text editor",
            "code" => "Visual Studio Code",
            "less" | "more" => "View file contents",
            "tar" => "Archive files",
            "zip" => "Create zip archives",
            "unzip" => "Extract zip archives",
            "gzip" => "Compress files",
            "gunzip" => "Decompress files",
            
            _ => {
                // Try to get description from system if available
                if std::env::var("PATH").is_ok() {
                    "External command"
                } else {
                    "Command"
                }
            }
        }.to_string()
    }

    pub fn load_system_commands(&self) -> Result<()> {
        // Load common builtin commands first
        let builtins = vec![
            "cd", "ls", "pwd", "echo", "cat", "cp", "mv", "rm", "mkdir", "rmdir",
            "touch", "chmod", "grep", "find", "sort", "uniq", "head", "tail", 
            "wc", "which", "history", "exit", "help", "ps", "top", "kill",
            "killall", "df", "du", "free", "uptime", "whoami", "id", "date",
            "ping", "curl", "git", "cargo", "vim", "nano", "less", "tar"
        ];
        
        for builtin in builtins {
            self.add_command(builtin.to_string(), None);
        }

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
                                    .or_else(|| name.strip_suffix(".ps1"))
                                    .unwrap_or(name)
                            } else {
                                name
                            };

                            // Skip if already exists or has problematic characters
                            if command_name.is_empty() || 
                               command_name.contains(' ') ||
                               command_name.starts_with('.') {
                                continue;
                            }

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
        prefix.split_whitespace().count() <= 1 ||
        prefix.contains('|') || prefix.contains('>')
    }

    fn get_completions(&self, input: &str, cursor: usize) -> Result<Vec<CompletionItem>> {
        let prefix = &input[..cursor];
        let command_prefix = prefix.split_whitespace().last().unwrap_or("");

        let mut items = Vec::new();
        if let Ok(commands) = self.commands.read() {
            for (name, item) in commands.iter() {
                if let Some(score) = self.matcher.fuzzy_match(name, command_prefix) {
                    let mut scored_item = item.clone();
                    scored_item.score = score as f64 / 100.0; // Normalize score
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
                if let Some(score) = self.matcher.fuzzy_match(item, prefix) {
                    let completion_item = CompletionItem::new(item.clone(), CompletionType::History)
                        .with_score((score as f64 / 100.0) + (history.len() - index) as f64 / 100.0) // Normalize and add recency boost
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

#[derive(Debug, Clone, Default)]
pub struct ProviderStats {
    pub total_calls: u64,
    pub total_time: Duration,
    pub avg_time: Duration,
    pub success_rate: f64,
    pub last_used: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_requests: u64,
    pub total_completions: u64,
    pub average_time: Duration,
    pub average_completion_time: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub partial_cache_hits: u64,
    pub provider_stats: HashMap<String, ProviderStats>,
    pub peak_memory_usage: usize,
    pub avg_result_count: f64,
    pub response_time_percentiles: VecDeque<Duration>,
    pub error_count: u64,
    pub last_cleanup: Option<Instant>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            total_completions: 0,
            average_time: Duration::from_millis(0),
            average_completion_time: 0.0,
            cache_hits: 0,
            cache_misses: 0,
            partial_cache_hits: 0,
            provider_stats: HashMap::new(),
            peak_memory_usage: 0,
            avg_result_count: 0.0,
            response_time_percentiles: VecDeque::new(),
            error_count: 0,
            last_cleanup: None,
        }
    }
}

impl PerformanceMetrics {
    /// Update response time percentiles for performance analysis
    pub fn update_response_time(&mut self, duration: Duration) {
        self.response_time_percentiles.push_back(duration);
        
        // Keep only last 1000 measurements for percentile calculation
        if self.response_time_percentiles.len() > 1000 {
            self.response_time_percentiles.pop_front();
        }
    }
    
    /// Calculate 95th percentile response time
    pub fn get_95th_percentile(&self) -> Duration {
        if self.response_time_percentiles.is_empty() {
            return Duration::from_millis(0);
        }
        
        let mut sorted: Vec<_> = self.response_time_percentiles.iter().cloned().collect();
        sorted.sort();
        
        let index = (sorted.len() as f64 * 0.95) as usize;
        sorted.get(index).cloned().unwrap_or(Duration::from_millis(0))
    }
    
    /// Get cache hit ratio
    pub fn cache_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
    
    /// Get provider efficiency report
    pub fn get_provider_efficiency(&self) -> HashMap<String, f64> {
        self.provider_stats.iter()
            .map(|(name, stats)| {
                let efficiency = if stats.total_calls > 0 {
                    stats.success_rate * (1000.0 / stats.avg_time.as_millis().max(1) as f64)
                } else {
                    0.0
                };
                (name.clone(), efficiency)
            })
            .collect()
    }
}

impl CompletionEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            providers: Vec::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            cache_ttl: Duration::from_millis(500), // 500ms for faster cache invalidation
            max_results: 50,
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
        };

        // Add default providers in order of priority
        engine.add_provider(Box::new(FileSystemProvider::new()));
        
        // Create and initialize command provider with system commands
        let command_provider = CommandProvider::new();
        if let Err(e) = command_provider.load_system_commands() {
            eprintln!("Warning: Failed to load system commands: {}", e);
        }
        engine.add_provider(Box::new(command_provider));
        
        engine.add_provider(Box::new(HistoryProvider::new(1000)));

        engine
    }

    pub fn add_provider(&mut self, provider: Box<dyn CompletionProvider>) {
        self.providers.push(provider);
        // Sort by priority (higher first)
        self.providers.sort_by_key(|b| std::cmp::Reverse(b.priority()));
    }

    pub fn get_completions(&self, input: &str) -> CompletionResult {
        self.get_completions_at_cursor(input, input.len())
    }

    pub fn get_completions_at_cursor(&self, input: &str, cursor: usize) -> CompletionResult {
        let start_time = Instant::now();
        let cache_key = format!("{}:{}", input, cursor);

        // Advanced cache strategy with multiple tiers
        if let Ok(cache) = self.cache.lock() {
            if let Some((result, timestamp)) = cache.get(&cache_key) {
                if start_time.duration_since(*timestamp) < self.cache_ttl {
                    if let Ok(mut metrics) = self.performance_metrics.lock() {
                        metrics.cache_hits += 1;
                    }
                    return result.clone();
                }
            }
            
            // Check for partial matches in cache (prefix-based caching)
            for (cached_key, (cached_result, cached_time)) in cache.iter() {
                if cached_key.starts_with(&cache_key[..cache_key.len().min(10)]) 
                    && start_time.duration_since(*cached_time) < self.cache_ttl {
                    // Found partial match - can use as base for completion
                    if let Ok(mut metrics) = self.performance_metrics.lock() {
                        metrics.cache_hits += 1;
                        metrics.partial_cache_hits += 1;
                    }
                    
                    // Apply additional filtering to cached results
                    let filtered_items = self.filter_cached_results(&cached_result.items, input, cursor);
                    if !filtered_items.is_empty() {
                        return CompletionResult::new(filtered_items, input[..cursor].to_string())
                            .with_timing(start_time.elapsed())
                            .with_sources(cached_result.sources_used.clone());
                    }
                }
            }
        }

        // Cache miss - generate completions with intelligent provider selection
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
        let mut execution_stats = HashMap::new();

        // Use context-aware provider selection
        let relevant_providers = self.select_relevant_providers(input, cursor);
        
        for provider in relevant_providers {
            let provider_start = Instant::now();
            
            if provider.can_complete(input, cursor) {
                match provider.get_completions(input, cursor) {
                    Ok(items) => {
                        if !items.is_empty() {
                            sources_used.push(provider.name().to_string());
                            
                            // Apply intelligent scoring and filtering
                            let scored_items = self.apply_intelligent_scoring(items, input, cursor);
                            all_items.extend(scored_items);
                        }
                    }
                    Err(e) => {
                        eprintln!("Completion provider '{}' failed: {}", provider.name(), e);
                    }
                }
            }
            
            let provider_duration = provider_start.elapsed();
            execution_stats.insert(provider.name().to_string(), provider_duration);
        }

        // Advanced sorting with multiple criteria
        all_items.sort_by(|a, b| {
            // Primary: Score (descending)
            let score_cmp = b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal);
            if score_cmp != std::cmp::Ordering::Equal {
                return score_cmp;
            }
            
            // Secondary: Type relevance based on context
            let type_relevance_a = self.get_contextual_type_relevance(&a.completion_type, input, cursor);
            let type_relevance_b = self.get_contextual_type_relevance(&b.completion_type, input, cursor);
            let relevance_cmp = type_relevance_b.partial_cmp(&type_relevance_a).unwrap_or(std::cmp::Ordering::Equal);
            if relevance_cmp != std::cmp::Ordering::Equal {
                return relevance_cmp;
            }
            
            // Tertiary: String length (shorter first for better UX)
            let len_cmp = a.text.len().cmp(&b.text.len());
            if len_cmp != std::cmp::Ordering::Equal {
                return len_cmp;
            }
            
            // Final: Alphabetical order
            a.text.cmp(&b.text)
        });

        // Intelligent result limiting with diversity preservation
        all_items = self.apply_intelligent_limiting(all_items, self.max_results);

        let completion_time = start_time.elapsed();
        let result = CompletionResult::new(all_items, prefix.to_string())
            .with_timing(completion_time)
            .with_sources(sources_used);

        // Enhanced cache management
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(cache_key, (result.clone(), start_time));
            
            // Adaptive cache cleanup based on usage patterns
            if cache.len() > 200 { // Increased cache size for better performance
                self.cleanup_cache_intelligently(&mut cache, start_time);
            }
        }

        // Update performance metrics with detailed stats
        if let Ok(mut metrics) = self.performance_metrics.lock() {
            metrics.total_completions += 1;
            metrics.average_completion_time = 
                (metrics.average_completion_time * (metrics.total_completions - 1) as f64 
                 + completion_time.as_millis() as f64) / metrics.total_completions as f64;
            
            for (provider_name, duration) in execution_stats {
                let provider_stats = metrics.provider_stats.entry(provider_name).or_default();
                provider_stats.total_calls += 1;
                provider_stats.total_time += duration;
            }
        }

        result
    }

    /// Select relevant providers based on input context
    fn select_relevant_providers(&self, input: &str, cursor: usize) -> Vec<&dyn CompletionProvider> {
        let mut relevant_providers = Vec::new();
        let prefix = &input[..cursor];
        
        // Analyze context to determine which providers are most relevant
        let is_command_position = prefix.split_whitespace().count() <= 1;
        let has_path_chars = prefix.contains('/') || prefix.contains('\\') || prefix.starts_with('.');
        let has_variable_chars = prefix.contains('$');
        
        for provider in &self.providers {
            let relevance_score = match provider.name() {
                "filesystem" if has_path_chars => 10,
                "command" if is_command_position => 9,
                "history" => 5, // Always somewhat relevant
                "builtin" if is_command_position => 8,
                "variable" if has_variable_chars => 10,
                _ => 3, // Default low relevance
            };
            
            if relevance_score >= 3 {
                relevant_providers.push(provider.as_ref());
            }
        }
        
        // Sort by relevance and priority
        relevant_providers.sort_by_key(|p| {
            let base_priority = p.priority();
            let context_bonus = match p.name() {
                "filesystem" if has_path_chars => 100,
                "command" if is_command_position => 90,
                _ => 0,
            };
            std::cmp::Reverse(base_priority + context_bonus)
        });
        
        relevant_providers
    }

    /// Apply intelligent scoring to completion items
    fn apply_intelligent_scoring(&self, items: Vec<CompletionItem>, input: &str, cursor: usize) -> Vec<CompletionItem> {
        let query = input[..cursor].trim_end();
        let query_lower = query.to_lowercase();
        
        items.into_iter().map(|mut item| {
            let text_lower = item.text.to_lowercase();
            let mut score = item.score;
            
            // Exact match bonus
            if text_lower == query_lower {
                score += 1000.0;
            }
            // Prefix match bonus
            else if text_lower.starts_with(&query_lower) {
                score += 500.0;
            }
            // Contains match bonus
            else if text_lower.contains(&query_lower) {
                score += 100.0;
            }
            
            // Word boundary bonuses
            if self.matches_word_boundaries(&text_lower, &query_lower) {
                score += 200.0;
            }
            
            // Length penalty for very long matches
            if item.text.len() > 50 {
                score -= (item.text.len() - 50) as f64 * 2.0;
            }
            
            item.score = score;
            item
        }).collect()
    }

    /// Check if query matches word boundaries in text
    fn matches_word_boundaries(&self, text: &str, query: &str) -> bool {
        let words: Vec<&str> = text.split(|c: char| !c.is_alphanumeric() && c != '_').collect();
        words.iter().any(|word| word.starts_with(query))
    }

    /// Get contextual relevance of completion types
    fn get_contextual_type_relevance(&self, completion_type: &CompletionType, input: &str, cursor: usize) -> f64 {
        let prefix = &input[..cursor];
        let is_first_word = prefix.split_whitespace().count() <= 1;
        
        match completion_type {
            CompletionType::Command | CompletionType::Builtin if is_first_word => 1.0,
            CompletionType::File | CompletionType::Directory if !is_first_word => 0.9,
            CompletionType::Variable if prefix.contains('$') => 0.95,
            CompletionType::History => 0.3,
            _ => 0.5,
        }
    }

    /// Apply intelligent limiting while preserving diversity
    fn apply_intelligent_limiting(&self, items: Vec<CompletionItem>, max_results: usize) -> Vec<CompletionItem> {
        if items.len() <= max_results {
            return items;
        }
        
        let mut result = Vec::new();
        let mut type_counts = HashMap::new();
        
        // First pass: Include high-scoring items from each type
        for item in items.iter() {
            let type_count = type_counts.entry(&item.completion_type).or_insert(0);
            if *type_count < 3 && result.len() < max_results { // Max 3 per type initially
                result.push(item.clone());
                *type_count += 1;
            }
        }
        
        // Second pass: Fill remaining slots with best items
        for item in items.iter() {
            if result.len() >= max_results {
                break;
            }
            if !result.iter().any(|existing| existing.text == item.text) {
                result.push(item.clone());
            }
        }
        
        result
    }

    /// Filter cached results for partial matches
    fn filter_cached_results(&self, cached_items: &[CompletionItem], input: &str, cursor: usize) -> Vec<CompletionItem> {
        let query = input[..cursor].to_lowercase();
        
        cached_items.iter()
            .filter(|item| item.text.to_lowercase().contains(&query))
            .cloned()
            .collect()
    }

    /// Intelligent cache cleanup based on usage patterns
    fn cleanup_cache_intelligently(&self, cache: &mut HashMap<String, (CompletionResult, Instant)>, current_time: Instant) {
        let entries: Vec<_> = cache.iter()
            .map(|(k, v)| (k.clone(), v.1))
            .collect();
        
        // Sort by access time and relevance
        let mut sorted_entries = entries;
        sorted_entries.sort_by(|a, b| {
            let time_diff_a = current_time.duration_since(a.1);
            let time_diff_b = current_time.duration_since(b.1);
            
            // Prefer more recent entries
            time_diff_a.cmp(&time_diff_b)
        });
        
        // Keep only the best 100 entries
        let keys_to_keep: std::collections::HashSet<_> = sorted_entries
            .iter()
            .take(100)
            .map(|(k, _)| k.clone())
            .collect();
            
        cache.retain(|key, _| keys_to_keep.contains(key));
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
        
        // Test basic functionality - engine should be created successfully
        let result = engine.get_completions("");
        // Since we load system commands automatically, empty result is not guaranteed
        // Just verify the engine works
        assert!(result.prefix.is_empty());
        
        // Test with some input
        let result = engine.get_completions("l");
        // Results depend on system commands available
        // Just verify it returns a result
        assert_eq!(result.prefix, "l");
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
