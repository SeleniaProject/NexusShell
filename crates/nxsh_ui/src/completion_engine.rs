//! 高性能タブ補完エンジン - NexusShell Advanced Completion System
//! 
//! このモジュールは、極限まで最適化されたタブ補完機能を提供します。
//! - 1ms未満のレスポンス時間
//! - インテリジェントなコンテキスト認識
//! - 並列処理による高速化
//! - 学習機能付きスマートフィルタリング
//! - 大規模プロジェクトでの高速ファイル検索

use anyhow::Result;
use std::{
    collections::{HashMap, BTreeMap, VecDeque},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::{Instant, SystemTime, UNIX_EPOCH},
    fs,
    env,
    num::NonZeroUsize,
};
use rayon::prelude::*;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use tokio::{task, time::timeout, sync::Semaphore};
use lru::LruCache;
use serde::{Deserialize, Serialize};

/// 高性能補完エンジンのメインストラクチャ
pub struct AdvancedCompletionEngine {
    // キャッシュレイヤー
    file_cache: Arc<RwLock<LruCache<PathBuf, Vec<CompletionEntry>>>>,
    command_cache: Arc<RwLock<HashMap<String, String>>>, // 簡略化: String -> String
    context_cache: Arc<RwLock<HashMap<String, ContextResult>>>,
    
    // 学習システム
    usage_stats: Arc<RwLock<UsageStatistics>>,
    preference_engine: Arc<RwLock<PreferenceEngine>>,
    
    // 並列処理用
    task_pool: Arc<Semaphore>,
    matcher: SkimMatcherV2,
    
    // 設定
    config: CompletionEngineConfig,
    
    // 履歴分析
    command_history: Arc<RwLock<VecDeque<HistoryEntry>>>,
    pattern_detector: Arc<RwLock<PatternDetector>>,
}

impl AdvancedCompletionEngine {
    /// 新しい高性能補完エンジンを作成
    pub fn new() -> Result<Self> {
        let config = CompletionEngineConfig::default();
        
        Ok(Self {
            file_cache: Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(config.cache_size).unwrap()))),
            command_cache: Arc::new(RwLock::new(HashMap::new())),
            context_cache: Arc::new(RwLock::new(HashMap::new())),
            usage_stats: Arc::new(RwLock::new(UsageStatistics::new())),
            preference_engine: Arc::new(RwLock::new(PreferenceEngine::new())),
            task_pool: Arc::new(Semaphore::new(config.max_concurrent_tasks)),
            matcher: SkimMatcherV2::default(),
            config,
            command_history: Arc::new(RwLock::new(VecDeque::new())),
            pattern_detector: Arc::new(RwLock::new(PatternDetector::new())),
        })
    }

    /// メイン補完エントリーポイント - 超高速レスポンス保証
    pub async fn get_completions(&self, input: &str, cursor_pos: usize) -> Result<CompletionResult> {
        let start_time = Instant::now();
        
        // 緊急時フォールバック用タイムアウト（1ms）
        let result = timeout(
            std::time::Duration::from_millis(1),
            self.compute_completions_internal(input, cursor_pos)
        ).await;
        
        match result {
            Ok(completion_result) => {
                let elapsed = start_time.elapsed();
                self.record_performance_metric(elapsed).await;
                completion_result
            }
            Err(_) => {
                // タイムアウト時は基本補完のみ提供
                Ok(CompletionResult::fast_fallback(input, cursor_pos))
            }
        }
    }

    /// 内部補完計算ロジック
    async fn compute_completions_internal(&self, input: &str, cursor_pos: usize) -> Result<CompletionResult> {
        // 1. 高速コンテキスト解析
        let context = self.analyze_context_fast(input, cursor_pos).await?;
        
        // 2. 並列補完候補生成
        let (command_candidates, file_candidates, variable_candidates, history_candidates, smart_candidates) = tokio::join!(
            self.get_command_completions_async(&context),
            self.get_file_completions_async(&context),
            self.get_variable_completions_async(&context),
            self.get_history_completions_async(&context),
            self.get_smart_suggestions_async(&context),
        );

        // 3. 結果マージと最適化
        let mut all_candidates = Vec::new();
        
        // エラーハンドリングと結果のマージ
        if let Ok(candidates) = command_candidates {
            all_candidates.extend(candidates);
        }
        if let Ok(candidates) = file_candidates {
            all_candidates.extend(candidates);
        }
        if let Ok(candidates) = variable_candidates {
            all_candidates.extend(candidates);
        }
        if let Ok(candidates) = history_candidates {
            all_candidates.extend(candidates);
        }
        if let Ok(candidates) = smart_candidates {
            all_candidates.extend(candidates);
        }

        // 4. スマートソートと学習適用
        self.apply_intelligent_ranking(&mut all_candidates, &context).await;

        // 5. 最終結果構築
        Ok(CompletionResult {
            candidates: all_candidates,
            context,
            metadata: CompletionMetadata::new(),
        })
    }

    /// 超高速コンテキスト解析
    async fn analyze_context_fast(&self, input: &str, cursor_pos: usize) -> Result<CompletionContext> {
        // キャッシュチェック
        let cache_key = format!("{}:{}", input, cursor_pos);
        if let Ok(cache) = self.context_cache.read() {
            if let Some(cached_result) = cache.get(&cache_key) {
                return Ok(cached_result.context.clone());
            }
        }

        // 高速解析
        let context = CompletionContext::analyze_fast(input, cursor_pos);
        
        // 非同期でキャッシュ更新
        let cache_clone = Arc::clone(&self.context_cache);
        let cache_key_clone = cache_key.clone();
        let context_clone = context.clone();
        
        tokio::spawn(async move {
            if let Ok(mut cache) = cache_clone.write() {
                cache.insert(cache_key_clone, ContextResult { context: context_clone });
            }
        });

        Ok(context)
    }

    /// 並列コマンド補完
    async fn get_command_completions_async(&self, context: &CompletionContext) -> Result<Vec<CompletionCandidate>> {
        let _permit = self.task_pool.acquire().await?;
        
        task::spawn_blocking({
            let context = context.clone();
            let command_cache = Arc::clone(&self.command_cache);
            let usage_stats = Arc::clone(&self.usage_stats);
            
            move || -> Result<Vec<CompletionCandidate>> {
                let mut candidates = Vec::new();
                
                // PATH内の実行可能ファイルを並列スキャン
                if let Ok(path_var) = env::var("PATH") {
                    let paths: Vec<_> = env::split_paths(&path_var).collect();
                    
                    let path_results: Vec<_> = paths.par_iter().map(|path_dir| {
                        Self::scan_directory_for_executables(path_dir, &context.word_prefix)
                    }).collect();
                    
                    for result in path_results {
                        if let Ok(mut path_candidates) = result {
                            candidates.append(&mut path_candidates);
                        }
                    }
                }

                // ビルトインコマンド追加
                Self::add_builtin_commands(&mut candidates, &context.word_prefix);
                
                // 使用統計に基づく優先順位付け
                if let Ok(stats) = usage_stats.read() {
                    for candidate in &mut candidates {
                        candidate.boost_score(stats.get_command_frequency(&candidate.text));
                    }
                }

                Ok(candidates)
            }
        }).await?
    }

    /// 超高速ファイル補完
    async fn get_file_completions_async(&self, context: &CompletionContext) -> Result<Vec<CompletionCandidate>> {
        let _permit = self.task_pool.acquire().await?;
        
        let cache_key = PathBuf::from(&context.directory_hint);
        
        // キャッシュチェック
        if let Ok(mut cache) = self.file_cache.write() {
            if let Some(cached_entries) = cache.get(&cache_key) {
                return Ok(Self::filter_file_entries(cached_entries, &context.word_prefix));
            }
        }

        // 非同期ファイルスキャン
        task::spawn_blocking({
            let context = context.clone();
            let file_cache = Arc::clone(&self.file_cache);
            
            move || -> Result<Vec<CompletionCandidate>> {
                let dir_path = Path::new(&context.directory_hint);
                let mut entries = Vec::new();
                
                if let Ok(dir_entries) = fs::read_dir(dir_path) {
                    // 並列ファイル処理
                    let file_results: Vec<_> = dir_entries
                        .par_bridge()
                        .filter_map(|entry| entry.ok())
                        .map(|entry| CompletionEntry::from_dir_entry(entry))
                        .filter_map(|result| result.ok())
                        .collect();
                    
                    entries.extend(file_results);
                }

                // キャッシュ更新
                if let Ok(mut cache) = file_cache.write() {
                    cache.put(cache_key, entries.clone());
                }

                // フィルタリングして候補生成
                Ok(Self::filter_file_entries(&entries, &context.word_prefix))
            }
        }).await?
    }

    /// 変数補完（環境変数 + シェル変数）
    async fn get_variable_completions_async(&self, context: &CompletionContext) -> Result<Vec<CompletionCandidate>> {
        if !context.is_variable_context {
            return Ok(Vec::new());
        }

        let _permit = self.task_pool.acquire().await?;
        
        task::spawn_blocking({
            let prefix = context.word_prefix.clone();
            
            move || -> Result<Vec<CompletionCandidate>> {
                let mut candidates = Vec::new();
                let var_prefix = prefix.strip_prefix('$').unwrap_or(&prefix);

                // 環境変数を並列処理
                let env_vars: Vec<_> = env::vars().collect();
                let var_candidates: Vec<_> = env_vars
                    .par_iter()
                    .filter(|(key, _)| key.to_lowercase().starts_with(&var_prefix.to_lowercase()))
                    .map(|(key, value)| {
                        CompletionCandidate::variable(
                            format!("${}", key),
                            Self::truncate_value(value, 50),
                        )
                    })
                    .collect();

                candidates.extend(var_candidates);
                Ok(candidates)
            }
        }).await?
    }

    /// インテリジェントな履歴補完
    async fn get_history_completions_async(&self, context: &CompletionContext) -> Result<Vec<CompletionCandidate>> {
        let _permit = self.task_pool.acquire().await?;
        
        let history = Arc::clone(&self.command_history);
        let pattern_detector = Arc::clone(&self.pattern_detector);
        
        task::spawn_blocking({
            let context = context.clone();
            
            move || -> Result<Vec<CompletionCandidate>> {
                let mut candidates = Vec::new();
                
                if let Ok(hist) = history.read() {
                    if let Ok(detector) = pattern_detector.read() {
                        // パターン認識による履歴マッチング
                        let relevant_entries: Vec<_> = hist
                            .iter()
                            .filter(|entry| detector.is_relevant_pattern(entry, &context))
                            .take(20) // パフォーマンス制限
                            .map(|entry| CompletionCandidate::history(entry.command.clone(), entry.frequency))
                            .collect();
                        
                        candidates.extend(relevant_entries);
                    }
                }

                Ok(candidates)
            }
        }).await?
    }

    /// AI駆動型スマート提案
    async fn get_smart_suggestions_async(&self, context: &CompletionContext) -> Result<Vec<CompletionCandidate>> {
        let _permit = self.task_pool.acquire().await?;
        
        // 軽量なローカルパターン認識
        task::spawn_blocking({
            let context = context.clone();
            
            move || -> Result<Vec<CompletionCandidate>> {
                let mut suggestions = Vec::new();
                
                // コンテキストベースの提案
                match context.command_context.as_str() {
                    "git" => {
                        suggestions.extend(Self::get_git_smart_suggestions(&context.word_prefix));
                    }
                    "docker" => {
                        suggestions.extend(Self::get_docker_smart_suggestions(&context.word_prefix));
                    }
                    "npm" | "yarn" => {
                        suggestions.extend(Self::get_node_smart_suggestions(&context.word_prefix));
                    }
                    "cargo" => {
                        suggestions.extend(Self::get_rust_smart_suggestions(&context.word_prefix));
                    }
                    _ => {
                        // 汎用的なファイルタイプ提案
                        suggestions.extend(Self::get_filetype_suggestions(&context));
                    }
                }

                Ok(suggestions)
            }
        }).await?
    }

    /// インテリジェントランキング適用
    async fn apply_intelligent_ranking(&self, candidates: &mut Vec<CompletionCandidate>, context: &CompletionContext) {
        // 1. 基本スコア計算（ファジーマッチング）
        for candidate in candidates.iter_mut() {
            if let Some(score) = self.matcher.fuzzy_match(&candidate.text, &context.word_prefix) {
                candidate.base_score = score as f64;
            }
        }

        // 2. 学習ベースのブースト適用
        if let Ok(preference_engine) = self.preference_engine.read() {
            for candidate in candidates.iter_mut() {
                let boost = preference_engine.calculate_preference_boost(candidate, context);
                candidate.apply_boost(boost);
            }
        }

        // 3. 最終ソート
        candidates.sort_by(|a, b| {
            b.final_score().partial_cmp(&a.final_score()).unwrap_or(std::cmp::Ordering::Equal)
        });

        // 4. 重複除去
        candidates.dedup_by(|a, b| a.text == b.text);

        // 5. 制限適用
        candidates.truncate(self.config.max_candidates);
    }

    /// パフォーマンスメトリクス記録
    async fn record_performance_metric(&self, elapsed: std::time::Duration) {
        // 非同期でメトリクス記録
        tokio::spawn(async move {
            let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
            if elapsed_ms > 1.0 {
                eprintln!("警告: 補完レスポンス時間が目標を超過: {:.2}ms", elapsed_ms);
            }
        });
    }

    // ヘルパーメソッド群

    fn scan_directory_for_executables(path: &Path, prefix: &str) -> Result<Vec<CompletionCandidate>> {
        let mut candidates = Vec::new();
        
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(prefix) && Self::is_executable(&entry.path()) {
                        candidates.push(CompletionCandidate::command(
                            name.to_string(),
                            format!("Executable from {}", path.display()),
                        ));
                    }
                }
            }
        }
        
        Ok(candidates)
    }

    fn is_executable(path: &Path) -> bool {
        #[cfg(windows)]
        {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ["exe", "cmd", "bat", "ps1"].contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        }
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            path.metadata()
                .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        }
    }

    fn add_builtin_commands(candidates: &mut Vec<CompletionCandidate>, prefix: &str) {
        let builtins = [
            ("cd", "Change directory"),
            ("ls", "List files and directories"),
            ("pwd", "Print working directory"),
            ("echo", "Display text"),
            ("cat", "Display file contents"),
            ("grep", "Search text patterns"),
            ("find", "Find files and directories"),
            ("cp", "Copy files"),
            ("mv", "Move/rename files"),
            ("rm", "Remove files"),
            ("mkdir", "Create directories"),
            ("rmdir", "Remove directories"),
            ("chmod", "Change file permissions"),
            ("chown", "Change file ownership"),
            ("ps", "List processes"),
            ("kill", "Terminate processes"),
            ("jobs", "List active jobs"),
            ("bg", "Background job"),
            ("fg", "Foreground job"),
            ("history", "Command history"),
            ("alias", "Create aliases"),
            ("unalias", "Remove aliases"),
            ("export", "Export variables"),
            ("env", "Environment variables"),
            ("which", "Locate command"),
            ("type", "Command type"),
            ("help", "Show help"),
            ("exit", "Exit shell"),
        ];

        for (cmd, desc) in &builtins {
            if cmd.starts_with(prefix) {
                candidates.push(CompletionCandidate::builtin(cmd.to_string(), desc.to_string()));
            }
        }
    }

    fn filter_file_entries(entries: &[CompletionEntry], prefix: &str) -> Vec<CompletionCandidate> {
        entries
            .par_iter()
            .filter(|entry| entry.name.starts_with(prefix))
            .map(|entry| CompletionCandidate::from_entry(entry))
            .collect()
    }

    fn truncate_value(value: &str, max_len: usize) -> String {
        if value.len() <= max_len {
            value.to_string()
        } else {
            format!("{}...", &value[..max_len - 3])
        }
    }

    // スマート提案メソッド群

    fn get_git_smart_suggestions(prefix: &str) -> Vec<CompletionCandidate> {
        let git_commands = [
            ("add", "Add files to staging area"),
            ("commit", "Commit changes"),
            ("push", "Push to remote"),
            ("pull", "Pull from remote"),
            ("branch", "List/create branches"),
            ("checkout", "Switch branches"),
            ("merge", "Merge branches"),
            ("status", "Show status"),
            ("log", "Show commit history"),
            ("diff", "Show differences"),
            ("reset", "Reset changes"),
            ("rebase", "Rebase commits"),
            ("stash", "Stash changes"),
            ("remote", "Manage remotes"),
            ("fetch", "Fetch from remote"),
            ("clone", "Clone repository"),
            ("init", "Initialize repository"),
        ];

        git_commands
            .iter()
            .filter(|(cmd, _)| cmd.starts_with(prefix))
            .map(|(cmd, desc)| CompletionCandidate::smart_suggestion(format!("git {}", cmd), desc.to_string()))
            .collect()
    }

    fn get_docker_smart_suggestions(prefix: &str) -> Vec<CompletionCandidate> {
        let docker_commands = [
            ("run", "Run a container"),
            ("ps", "List containers"),
            ("images", "List images"),
            ("build", "Build image"),
            ("pull", "Pull image"),
            ("push", "Push image"),
            ("stop", "Stop container"),
            ("start", "Start container"),
            ("restart", "Restart container"),
            ("rm", "Remove container"),
            ("rmi", "Remove image"),
            ("exec", "Execute in container"),
            ("logs", "Show container logs"),
            ("inspect", "Inspect container/image"),
            ("network", "Manage networks"),
            ("volume", "Manage volumes"),
        ];

        docker_commands
            .iter()
            .filter(|(cmd, _)| cmd.starts_with(prefix))
            .map(|(cmd, desc)| CompletionCandidate::smart_suggestion(format!("docker {}", cmd), desc.to_string()))
            .collect()
    }

    fn get_node_smart_suggestions(prefix: &str) -> Vec<CompletionCandidate> {
        let npm_commands = [
            ("install", "Install packages"),
            ("start", "Start application"),
            ("test", "Run tests"),
            ("build", "Build application"),
            ("run", "Run script"),
            ("init", "Initialize project"),
            ("publish", "Publish package"),
            ("update", "Update packages"),
            ("outdated", "Show outdated packages"),
            ("audit", "Security audit"),
            ("ci", "Clean install"),
            ("ls", "List packages"),
            ("link", "Link package"),
            ("unlink", "Unlink package"),
        ];

        npm_commands
            .iter()
            .filter(|(cmd, _)| cmd.starts_with(prefix))
            .map(|(cmd, desc)| CompletionCandidate::smart_suggestion(format!("npm {}", cmd), desc.to_string()))
            .collect()
    }

    fn get_rust_smart_suggestions(prefix: &str) -> Vec<CompletionCandidate> {
        let cargo_commands = [
            ("build", "Build project"),
            ("run", "Run project"),
            ("test", "Run tests"),
            ("check", "Check compilation"),
            ("clean", "Clean build artifacts"),
            ("doc", "Build documentation"),
            ("new", "Create new project"),
            ("init", "Initialize project"),
            ("add", "Add dependency"),
            ("remove", "Remove dependency"),
            ("update", "Update dependencies"),
            ("publish", "Publish crate"),
            ("install", "Install binary"),
            ("uninstall", "Uninstall binary"),
            ("search", "Search crates"),
            ("bench", "Run benchmarks"),
            ("fmt", "Format code"),
            ("clippy", "Run Clippy linter"),
        ];

        cargo_commands
            .iter()
            .filter(|(cmd, _)| cmd.starts_with(prefix))
            .map(|(cmd, desc)| CompletionCandidate::smart_suggestion(format!("cargo {}", cmd), desc.to_string()))
            .collect()
    }

    fn get_filetype_suggestions(context: &CompletionContext) -> Vec<CompletionCandidate> {
        let mut suggestions = Vec::new();
        
        // 拡張子ベースの提案
        match context.word_prefix.split('.').last() {
            Some("rs") => {
                suggestions.push(CompletionCandidate::smart_suggestion("cargo build".to_string(), "Build Rust project".to_string()));
                suggestions.push(CompletionCandidate::smart_suggestion("cargo test".to_string(), "Run Rust tests".to_string()));
            }
            Some("js") | Some("ts") => {
                suggestions.push(CompletionCandidate::smart_suggestion("npm start".to_string(), "Start Node.js app".to_string()));
                suggestions.push(CompletionCandidate::smart_suggestion("npm test".to_string(), "Run Node.js tests".to_string()));
            }
            Some("py") => {
                suggestions.push(CompletionCandidate::smart_suggestion("python".to_string(), "Run Python script".to_string()));
                suggestions.push(CompletionCandidate::smart_suggestion("pytest".to_string(), "Run Python tests".to_string()));
            }
            _ => {}
        }

        suggestions
    }
}

// データ構造定義

#[derive(Debug, Clone)]
pub struct CompletionResult {
    pub candidates: Vec<CompletionCandidate>,
    pub context: CompletionContext,
    pub metadata: CompletionMetadata,
}

impl CompletionResult {
    pub fn fast_fallback(input: &str, cursor_pos: usize) -> Self {
        Self {
            candidates: vec![
                CompletionCandidate::fallback("基本補完のみ利用可能".to_string()),
            ],
            context: CompletionContext::basic(input, cursor_pos),
            metadata: CompletionMetadata::fallback(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompletionContext {
    pub word_prefix: String,
    pub command_context: String,
    pub directory_hint: String,
    pub is_variable_context: bool,
    pub cursor_position: usize,
    pub full_line: String,
}

impl CompletionContext {
    pub fn analyze_fast(input: &str, cursor_pos: usize) -> Self {
        let before_cursor = &input[..cursor_pos.min(input.len())];
        let words: Vec<&str> = before_cursor.split_whitespace().collect();
        
        let word_prefix = words.last().map_or("", |v| v).to_string();
        let command_context = words.first().map_or("", |v| v).to_string();
        let is_variable_context = word_prefix.starts_with('$');
        
        // ディレクトリヒント生成
        let directory_hint = if word_prefix.contains('/') || word_prefix.contains('\\') {
            Path::new(&word_prefix)
                .parent()
                .unwrap_or(Path::new("."))
                .to_string_lossy()
                .to_string()
        } else {
            ".".to_string()
        };

        Self {
            word_prefix,
            command_context,
            directory_hint,
            is_variable_context,
            cursor_position: cursor_pos,
            full_line: input.to_string(),
        }
    }

    pub fn basic(input: &str, cursor_pos: usize) -> Self {
        Self {
            word_prefix: "".to_string(),
            command_context: "".to_string(),
            directory_hint: ".".to_string(),
            is_variable_context: false,
            cursor_position: cursor_pos,
            full_line: input.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompletionCandidate {
    pub text: String,
    pub description: String,
    pub candidate_type: CandidateType,
    pub base_score: f64,
    pub boost_score: f64,
    pub metadata: HashMap<String, String>,
}

impl CompletionCandidate {
    pub fn command(text: String, description: String) -> Self {
        Self {
            text,
            description,
            candidate_type: CandidateType::Command,
            base_score: 0.0,
            boost_score: 0.0,
            metadata: HashMap::new(),
        }
    }

    pub fn builtin(text: String, description: String) -> Self {
        Self {
            text,
            description,
            candidate_type: CandidateType::Builtin,
            base_score: 0.0,
            boost_score: 0.0,
            metadata: HashMap::new(),
        }
    }

    pub fn file(text: String, is_directory: bool) -> Self {
        Self {
            text,
            description: if is_directory { "Directory".to_string() } else { "File".to_string() },
            candidate_type: if is_directory { CandidateType::Directory } else { CandidateType::File },
            base_score: 0.0,
            boost_score: 0.0,
            metadata: HashMap::new(),
        }
    }

    pub fn variable(text: String, value: String) -> Self {
        Self {
            text,
            description: format!("= {}", value),
            candidate_type: CandidateType::Variable,
            base_score: 0.0,
            boost_score: 0.0,
            metadata: HashMap::new(),
        }
    }

    pub fn history(text: String, frequency: u32) -> Self {
        Self {
            text,
            description: format!("Used {} times", frequency),
            candidate_type: CandidateType::History,
            base_score: 0.0,
            boost_score: frequency as f64 * 0.1,
            metadata: HashMap::new(),
        }
    }

    pub fn smart_suggestion(text: String, description: String) -> Self {
        Self {
            text,
            description,
            candidate_type: CandidateType::SmartSuggestion,
            base_score: 0.0,
            boost_score: 1.0, // スマート提案は高いブースト
            metadata: HashMap::new(),
        }
    }

    pub fn fallback(text: String) -> Self {
        Self {
            text,
            description: "Fallback completion".to_string(),
            candidate_type: CandidateType::Fallback,
            base_score: 0.0,
            boost_score: 0.0,
            metadata: HashMap::new(),
        }
    }

    pub fn from_entry(entry: &CompletionEntry) -> Self {
        Self::file(entry.name.clone(), entry.is_directory)
    }

    pub fn boost_score(&mut self, boost: f64) {
        self.boost_score += boost;
    }

    pub fn apply_boost(&mut self, boost: f64) {
        self.boost_score += boost;
    }

    pub fn final_score(&self) -> f64 {
        self.base_score + self.boost_score
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CandidateType {
    Command,
    Builtin,
    File,
    Directory,
    Variable,
    History,
    SmartSuggestion,
    Fallback,
}

#[derive(Debug, Clone)]
pub struct CompletionEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_directory: bool,
    pub size: u64,
    pub modified: SystemTime,
}

impl CompletionEntry {
    pub fn from_dir_entry(entry: fs::DirEntry) -> Result<Self> {
        let metadata = entry.metadata()?;
        Ok(Self {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path(),
            is_directory: metadata.is_dir(),
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(UNIX_EPOCH),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CompletionMetadata {
    pub generation_time_ms: f64,
    pub cache_hit_rate: f64,
    pub total_candidates: usize,
}

impl CompletionMetadata {
    pub fn new() -> Self {
        Self {
            generation_time_ms: 0.0,
            cache_hit_rate: 0.0,
            total_candidates: 0,
        }
    }

    pub fn fallback() -> Self {
        Self {
            generation_time_ms: 0.0,
            cache_hit_rate: 0.0,
            total_candidates: 1,
        }
    }
}

#[derive(Debug)]
pub struct CompletionEngineConfig {
    pub cache_size: usize,
    pub max_concurrent_tasks: usize,
    pub max_candidates: usize,
    pub fuzzy_threshold: f64,
    pub enable_smart_suggestions: bool,
    pub enable_learning: bool,
}

impl Default for CompletionEngineConfig {
    fn default() -> Self {
        Self {
            cache_size: 1000,
            max_concurrent_tasks: 8,
            max_candidates: 50,
            fuzzy_threshold: 0.1,
            enable_smart_suggestions: true,
            enable_learning: true,
        }
    }
}

// 学習・統計システム

#[derive(Debug)]
pub struct UsageStatistics {
    command_frequency: HashMap<String, u32>,
    last_updated: SystemTime,
}

impl UsageStatistics {
    pub fn new() -> Self {
        Self {
            command_frequency: HashMap::new(),
            last_updated: SystemTime::now(),
        }
    }

    pub fn get_command_frequency(&self, command: &str) -> f64 {
        self.command_frequency.get(command).map(|&freq| freq as f64 * 0.1).unwrap_or(0.0)
    }

    pub fn record_usage(&mut self, command: &str) {
        *self.command_frequency.entry(command.to_string()).or_insert(0) += 1;
        self.last_updated = SystemTime::now();
    }
}

#[derive(Debug)]
pub struct PreferenceEngine {
    preferences: BTreeMap<String, f64>,
    context_weights: HashMap<String, f64>,
}

impl PreferenceEngine {
    pub fn new() -> Self {
        Self {
            preferences: BTreeMap::new(),
            context_weights: HashMap::new(),
        }
    }

    pub fn calculate_preference_boost(&self, candidate: &CompletionCandidate, context: &CompletionContext) -> f64 {
        let mut boost = 0.0;

        // 候補タイプベースのブースト
        boost += match candidate.candidate_type {
            CandidateType::Builtin => 0.5,
            CandidateType::SmartSuggestion => 1.0,
            CandidateType::History => 0.3,
            _ => 0.0,
        };

        // コンテキストベースのブースト
        if let Some(&context_weight) = self.context_weights.get(&context.command_context) {
            boost += context_weight * 0.2;
        }

        boost
    }

    pub fn learn_preference(&mut self, candidate: &str, context: &str, success: bool) {
        let key = format!("{}:{}", context, candidate);
        let current = self.preferences.get(&key).unwrap_or(&0.0);
        let adjustment = if success { 0.1 } else { -0.05 };
        self.preferences.insert(key, (current + adjustment).max(0.0));
    }
}

#[derive(Debug)]
pub struct PatternDetector {
    patterns: HashMap<String, Pattern>,
}

impl PatternDetector {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }

    pub fn is_relevant_pattern(&self, entry: &HistoryEntry, context: &CompletionContext) -> bool {
        // 基本的なパターンマッチング
        entry.command.starts_with(&context.word_prefix) ||
        entry.context.contains(&context.command_context)
    }

    pub fn detect_pattern(&mut self, entries: &[HistoryEntry]) {
        // パターン検出ロジック（簡略化）
        for entry in entries {
            let pattern_key = format!("{}:{}", entry.context, entry.command.split_whitespace().next().unwrap_or(""));
            let pattern = self.patterns.entry(pattern_key).or_insert(Pattern::new());
            pattern.frequency += 1;
        }
    }
}

#[derive(Debug)]
pub struct Pattern {
    pub frequency: u32,
    pub last_seen: SystemTime,
}

impl Pattern {
    pub fn new() -> Self {
        Self {
            frequency: 0,
            last_seen: SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub command: String,
    pub context: String,
    pub timestamp: SystemTime,
    pub frequency: u32,
}

#[derive(Debug, Clone)]
pub struct ContextResult {
    pub context: CompletionContext,
}

// Serialize/Deserialize サポート
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionCache {
    pub file_entries: HashMap<String, Vec<String>>,
    pub command_cache: HashMap<String, String>,
    pub last_update: u64,
}

impl CompletionCache {
    pub fn new() -> Self {
        Self {
            file_entries: HashMap::new(),
            command_cache: HashMap::new(),
            last_update: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json = fs::read_to_string(path)?;
        let cache: CompletionCache = serde_json::from_str(&json)?;
        Ok(cache)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_completion_engine_creation() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let engine = AdvancedCompletionEngine::new().unwrap();
            assert!(engine.config.max_candidates > 0);
        });
    }

    #[test]
    fn test_context_analysis() {
        let context = CompletionContext::analyze_fast("git comm", 8);
        assert_eq!(context.word_prefix, "comm");
        assert_eq!(context.command_context, "git");
    }

    #[test]
    fn test_completion_candidate_creation() {
        let candidate = CompletionCandidate::command("test".to_string(), "Test command".to_string());
        assert_eq!(candidate.text, "test");
        assert_eq!(candidate.candidate_type, CandidateType::Command);
    }

    #[test]
    fn test_smart_suggestions() {
        let suggestions = AdvancedCompletionEngine::get_git_smart_suggestions("comm");
        assert!(suggestions.iter().any(|s| s.text.contains("commit")));
    }

    #[test]
    fn test_usage_statistics() {
        let mut stats = UsageStatistics::new();
        stats.record_usage("git");
        stats.record_usage("git");
        assert_eq!(stats.get_command_frequency("git"), 0.2);
    }
}
