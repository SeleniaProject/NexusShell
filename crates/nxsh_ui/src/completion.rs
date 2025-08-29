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
use std::process::Command;
/// Completion types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionType {
    Command,
    File,
    Directory,
    Variable,
    Alias,
    Builtin,
    Flag,
    Subcommand,
    EnvVar,
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
    command_specs: HashMap<String, CommandSpec>,
}

#[derive(Debug, Clone)]
struct CommandSpec {
    name: String,
    subcommands: Vec<(&'static str, &'static str)>,   // (name, desc)
    flags: Vec<(&'static str, &'static str)>,         // (flag, desc), includes short/long
    // Hint for default argument completion type when not a flag
    default_arg: ArgKind,
    // Map flag -> expected value kind (e.g., --file <path>)
    flag_value_kind: HashMap<&'static str, ArgKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArgKind { Any, Path, File, Dir, Env, None }

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
            command_specs: HashMap::new(),
        };
        
        // Initialize with basic builtins
        completer.init_builtins();
        completer.init_command_specs();
        completer.refresh_env_cache();
        
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

    fn init_command_specs(&mut self) {
        use ArgKind::*;
        let mut add = |spec: CommandSpec| {
            self.command_specs.insert(spec.name.clone(), spec);
        };

        add(CommandSpec {
            name: "cd".into(),
            subcommands: vec![],
            flags: vec![],
            default_arg: Dir,
            flag_value_kind: HashMap::new(),
        });

        add(CommandSpec {
            name: "ls".into(),
            subcommands: vec![],
            flags: vec![("-a", "show all"), ("-l", "long format"), ("-h", "human readable"), ("--all", "show all"), ("--long", "long format")],
            default_arg: Path,
            flag_value_kind: HashMap::new(),
        });

        add(CommandSpec {
            name: "cat".into(),
            subcommands: vec![],
            flags: vec![],
            default_arg: File,
            flag_value_kind: HashMap::new(),
        });

        add(CommandSpec {
            name: "echo".into(),
            subcommands: vec![],
            flags: vec![("-n", "no newline")],
            default_arg: Any,
            flag_value_kind: HashMap::new(),
        });

        add(CommandSpec {
            name: "git".into(),
            subcommands: vec![
                ("add", "Add file contents to the index"),
                ("commit", "Record changes to the repository"),
                ("status", "Show the working tree status"),
                ("checkout", "Switch branches or restore files"),
                ("branch", "List, create, or delete branches"),
                ("push", "Update remote refs along with objects"),
                ("pull", "Fetch from and integrate with another repo"),
                ("clone", "Clone a repository into a new directory"),
                ("merge", "Join two or more development histories"),
            ],
            flags: vec![("-h", "help"), ("--help", "help"), ("-v", "verbose"), ("--verbose", "verbose")],
            default_arg: Path,
            flag_value_kind: HashMap::new(),
        });

        add(CommandSpec {
            name: "cargo".into(),
            subcommands: vec![
                ("build", "Compile the current package"),
                ("run", "Run a binary or example"),
                ("test", "Run tests"),
                ("bench", "Run benchmarks"),
                ("doc", "Build documentation"),
                ("clean", "Remove generated artifacts"),
            ],
            flags: vec![("-q", "quiet"), ("--release", "optimized build"), ("--bin", "select binary (value)"), ("--example", "select example (value)")],
            default_arg: Any,
            flag_value_kind: HashMap::from_iter([
                ("--bin", Any),
                ("--example", Any),
            ]),
        });
    }

    fn refresh_env_cache(&mut self) {
        self.variable_cache.clear();
        for (k, _v) in env::vars() { self.variable_cache.insert(k); }
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
        let ends_with_space = text.ends_with(' ');
        let parts: Vec<&str> = text.split_whitespace().collect();
        let current = if ends_with_space { "" } else { parts.last().copied().unwrap_or("") };

        // 0) First token -> command補完
        if parts.is_empty() || (parts.len() == 1 && !ends_with_space) {
            return self.complete_command(current);
        }

        // 1) 以降のトークン: コンテキストを見て決定
        let command = parts.first().copied().unwrap_or("");
        let spec_owned = self.get_or_discover_spec_owned(command);

        // 1-a) 環境変数（$で始まる）
        if let Some(stripped) = current.strip_prefix('$') {
            return self.complete_env(stripped);
        }

        // 1-b) フラグ（-で始まる）
        if current.starts_with('-') {
            // used_flags 抽出のため、command+これまでの引数を連結した部分を渡す
            let before_current = if ends_with_space { text } else { &text[..text.len().saturating_sub(current.len())] };
            return self.complete_flags(before_current, current, spec_owned.as_ref());
        }

        // 1-c) サブコマンド（2番目のトークン）
        if parts.len() == 2 && !ends_with_space {
            if let Some(spec) = spec_owned.as_ref() {
                if !spec.subcommands.is_empty() {
                    return self.complete_subcommand(spec, current);
                }
            }
        }

        // 1-d) 引数の種類から補完
    if let Some(spec) = spec_owned.as_ref() {
            // 直前が値受け取りフラグならその種類
            if let Some(prev) = parts.get(parts.len().saturating_sub(2)).copied() {
                if let Some(kind) = spec.flag_value_kind.get(prev) {
                    return self.complete_by_kind(*kind, current);
                }
            }
            return self.complete_by_kind(spec.default_arg, current);
        }

        // それ以外はファイル/ディレクトリ
        self.complete_file(current)
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
                        let mut completion = full_path.to_string_lossy().to_string();
                        if is_dir && !completion.ends_with('/') && !completion.ends_with('\\') {
                            completion.push(std::path::MAIN_SEPARATOR);
                        }
                        
                        // Create properly formatted display with consistent spacing
                        let display_name = if is_dir { 
                            format!("{}/", name) 
                        } else { 
                            name.to_string() 
                        };
                        
                        let display = if self.completion_config.show_descriptions {
                            let file_type = if is_dir { "dir" } else { "file" };
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

    fn complete_env(&self, prefix: &str) -> Vec<CompletionResult> {
        let mut out = Vec::new();
        for var in &self.variable_cache {
            if var.starts_with(prefix) {
                out.push(CompletionResult {
                    completion: format!("${}", var),
                    display: Some(format!("{:<20} env", var)),
                    completion_type: CompletionType::EnvVar,
                    score: self.calculate_score(prefix, var),
                });
            }
        }
        out.sort_by(|a, b| b.score.cmp(&a.score));
        out.truncate(self.completion_config.max_suggestions);
        out
    }

    fn complete_flags(&self, command: &str, current: &str, spec: Option<&CommandSpec>) -> Vec<CompletionResult> {
        let mut out = Vec::new();

        // 既に入力済みのフラグを除外
        let used_flags: HashSet<&str> = command
            .split_whitespace()
            .skip(1)
            .filter(|t| t.starts_with('-'))
            .collect();

    let push_flag = |flag: &str, desc: &str, list: &mut Vec<CompletionResult>| {
            if used_flags.contains(flag) { return; }
            if flag.starts_with(current) {
                list.push(CompletionResult {
                    completion: flag.to_string(),
                    display: Some(format!("{:<20} flag — {}", flag, desc)),
                    completion_type: CompletionType::Flag,
                    score: self.calculate_score(current, flag),
                });
            }
        };

    if let Some(spec) = spec {
            for (flag, desc) in &spec.flags {
                push_flag(flag, desc, &mut out);
            }
        }
    for &(flag, desc) in [("-h", "help"), ("--help", "help"), ("-v", "verbose"), ("--verbose", "verbose")].iter() {
            push_flag(flag, desc, &mut out);
        }

        out.sort_by(|a, b| b.score.cmp(&a.score));
        out.truncate(self.completion_config.max_suggestions);
        out
    }

    fn get_or_discover_spec_owned(&mut self, command: &str) -> Option<CommandSpec> {
        if command.is_empty() { return None; }
        if let Some(spec) = self.command_specs.get(command) {
            return Some(spec.clone());
        }
        if let Some(spec) = self.discover_from_help(command) {
            self.command_specs.insert(command.to_string(), spec.clone());
            return Some(spec);
        }
        None
    }

    fn discover_from_help(&self, command: &str) -> Option<CommandSpec> {
        // Try common help invocations
        let attempts: &[&[&str]] = &[
            &["--help"],
            &["-h"],
            // Windows style for some tools
            &["/?"],
        ];

        let mut output = String::new();
        for args in attempts {
            match Command::new(command).args(*args).output() {
                Ok(out) => {
                    if out.status.success() || !out.stdout.is_empty() || !out.stderr.is_empty() {
                        let text = if !out.stdout.is_empty() {
                            String::from_utf8_lossy(&out.stdout).to_string()
                        } else {
                            String::from_utf8_lossy(&out.stderr).to_string()
                        };
                        if !text.trim().is_empty() {
                            output = text;
                            break;
                        }
                    }
                }
                Err(_) => continue,
            }
        }
        if output.trim().is_empty() { return None; }

        // Parse sections: OPTIONS/FLAGS and SUBCOMMANDS/COMMANDS
        let mut flags: Vec<(&'static str, &'static str)> = Vec::new();
        let mut subs: Vec<(&'static str, &'static str)> = Vec::new();
        let mut flag_value_kind: HashMap<&'static str, ArgKind> = HashMap::new();
        let mut default_arg = ArgKind::Any;

        let mut section = String::new();
        for line in output.lines() {
            let l = line.trim_end();
            let ltrim = l.trim_start();
            let upper = ltrim.to_ascii_uppercase();
            if upper.contains("SUBCOMMANDS") || upper.contains("COMMANDS") {
                section = "SUBS".into();
                continue;
            }
            if upper.contains("OPTIONS") || upper.contains("FLAGS") || upper.starts_with("- ") {
                section = "FLAGS".into();
                continue;
            }
            if upper.contains("USAGE") || upper.starts_with("USAGE:") {
                section = "USAGE".into();
                // try infer default arg kind from typical placeholders
                if l.contains("FILE") || l.contains("FILES") { default_arg = ArgKind::File; }
                else if l.contains("DIR") || l.contains("DIRECTORY") || l.contains("FOLDER") { default_arg = ArgKind::Dir; }
                else if l.contains("PATH") { default_arg = ArgKind::Path; }
                continue;
            }

            match section.as_str() {
                "FLAGS" => {
                    // expect lines like "  -h, --help   Show help" or "  --bin <NAME>  ..."
                    if ltrim.starts_with('-') {
                        let mut parts = ltrim.split_whitespace();
                        let first = parts.next().unwrap_or("");
                        let mut candidates: Vec<String> = Vec::new();
                        if first.starts_with("-") { candidates.push(first.to_string()); }
                        // maybe "," separated long form next
                        if let Some(rest) = ltrim.split_once(',').map(|(_a, b)| b.trim()) {
                            if rest.starts_with("--") {
                                let tok = rest.split_whitespace().next().unwrap_or("");
                                if !tok.is_empty() { candidates.push(tok.to_string()); }
                            }
                        } else {
                            // take second token if it's a flag
                            if let Some(tok2) = parts.next() { if tok2.starts_with('-') { candidates.push(tok2.to_string()); } }
                        }
                        // description is whatever remains
                        let desc = ltrim.split_once("  ").map(|x| x.1).unwrap_or("").trim();
                        for c in candidates {
                            // detect value kind by placeholder following flag
                            let kind = if ltrim.contains("<FILE>") || ltrim.contains("FILE") { ArgKind::File }
                                else if ltrim.contains("<PATH>") || ltrim.contains("PATH") { ArgKind::Path }
                                else if ltrim.contains("<DIR>") || ltrim.contains("DIRECTORY") { ArgKind::Dir }
                                else if ltrim.contains("<ENV>") || ltrim.contains("ENV") { ArgKind::Env }
                                else { ArgKind::Any };
                            // store as 'static via leak (safe here: small, process-lifetime cache)
                            let f: &'static str = Box::leak(c.into_boxed_str());
                            let d: &'static str = Box::leak(desc.to_string().into_boxed_str());
                            flags.push((f, d));
                            if kind != ArgKind::Any { flag_value_kind.insert(f, kind); }
                        }
                    }
                }
                "SUBS" => {
                    // lines like "  build   Compile..."
                    if !ltrim.is_empty() && !ltrim.starts_with('-') && !ltrim.contains(' ') {
                        let name = ltrim;
                        let s: &'static str = Box::leak(name.to_string().into_boxed_str());
                        subs.push((s, ""));
                    } else {
                        // try split at two-spaces boundary
                        if let Some((name, desc)) = ltrim.split_once("  ") {
                            let nm = name.split_whitespace().next().unwrap_or("");
                            if !nm.is_empty() && !nm.starts_with('-') {
                                let s: &'static str = Box::leak(nm.to_string().into_boxed_str());
                                let d: &'static str = Box::leak(desc.trim().to_string().into_boxed_str());
                                subs.push((s, d));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Some(CommandSpec {
            name: command.to_string(),
            subcommands: subs,
            flags,
            default_arg,
            flag_value_kind,
        })
    }

    fn complete_subcommand(&self, spec: &CommandSpec, current: &str) -> Vec<CompletionResult> {
        let mut out = Vec::new();
        for (name, desc) in &spec.subcommands {
            if name.starts_with(current) {
                out.push(CompletionResult {
                    completion: (*name).to_string(),
                    display: Some(format!("{:<20} subcommand — {}", name, desc)),
                    completion_type: CompletionType::Subcommand,
                    score: self.calculate_score(current, name),
                });
            }
        }
        out.sort_by(|a, b| b.score.cmp(&a.score));
        out.truncate(self.completion_config.max_suggestions);
        out
    }

    fn complete_by_kind(&self, kind: ArgKind, current: &str) -> Vec<CompletionResult> {
        match kind {
            ArgKind::Path | ArgKind::File | ArgKind::Dir | ArgKind::Any => self.complete_file(current),
            ArgKind::Env => self.complete_env(current),
            ArgKind::None => Vec::new(),
        }
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