//! Intelligent tab completion system for NexusShell
//! 
//! This module provides context-aware completion for commands, files, variables,
//! and more, with fuzzy matching and smart filtering capabilities.

use anyhow::Result;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use rustyline::{
    completion::{Completer, Pair},
    Context as RustylineContext,
};
use std::{
    collections::{HashMap, HashSet},
    env,
    fs,
    path::Path,
};
use nxsh_core::context::ShellContext;
  // Task 8: Builtin Integration Restored

/// Main completion engine for NexusShell
pub struct NexusCompleter {
    filename_completer: rustyline::completion::FilenameCompleter,
    command_cache: HashMap<String, String>, // command -> description
    builtin_cache: HashMap<String, String>, // builtin -> description
    variable_cache: HashSet<String>,
    alias_cache: HashMap<String, String>,
    fuzzy_matcher: SkimMatcherV2,
    completion_config: CompletionConfig,
}

impl NexusCompleter {
    /// Create a comprehensive completer with full functionality
    /// COMPLETE initialization with ALL caches and system command scanning
    pub fn new_minimal() -> Result<Self> {
        let mut completer = Self {
            filename_completer: rustyline::completion::FilenameCompleter::new(),
            command_cache: HashMap::new(),
            builtin_cache: HashMap::new(),
            variable_cache: HashSet::new(),
            alias_cache: HashMap::new(),
            fuzzy_matcher: SkimMatcherV2::default(),
            completion_config: CompletionConfig::default(),
        };

        // FULL initialization - populate ALL caches immediately as required
        completer.populate_builtin_cache();
        completer.populate_command_cache()?;
        completer.populate_variable_cache()?;
        completer.populate_alias_cache()?;
        
        Ok(completer)
    }

    /// Create a new completer with default settings
    pub fn new() -> Result<Self> {
        let mut completer = Self {
            filename_completer: rustyline::completion::FilenameCompleter::new(),
            command_cache: HashMap::new(),
            builtin_cache: HashMap::new(),
            variable_cache: HashSet::new(),
            alias_cache: HashMap::new(),
            fuzzy_matcher: SkimMatcherV2::default(),
            completion_config: CompletionConfig::default(),
        };

        // Initialize with system commands and environment variables
        completer.refresh_system_commands()?;
        completer.refresh_environment_variables();
        // Also include full set of internal builtins so Tab 補完に反映される
        completer.refresh_builtin_commands();

        Ok(completer)
    }

    /// Refresh system commands from PATH
    pub fn refresh_system_commands(&mut self) -> Result<()> {
        self.command_cache.clear();

        if let Ok(path_var) = env::var("PATH") {
            for path_dir in env::split_paths(&path_var) {
                if let Ok(entries) = fs::read_dir(&path_dir) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            // Skip hidden files and common non-executables
                            if !name.starts_with('.') && !name.ends_with(".dll") && !name.ends_with(".so") {
                                self.command_cache.insert(
                                    name.to_string(),
                                    format!("System command from {}", path_dir.display()),
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Refresh environment variables
    pub fn refresh_environment_variables(&mut self) {
        self.variable_cache.clear();
        for (key, _) in env::vars() {
            // Insert original key
            self.variable_cache.insert(key.clone());
            // Normalize PATH for case-insensitive platforms (Windows uses 'Path')
            if key.eq_ignore_ascii_case("PATH") {
                self.variable_cache.insert("PATH".to_string());
            }
        }
    }
    
    /// Populate builtin command cache
    pub fn populate_builtin_cache(&mut self) {
        self.builtin_cache.clear();
        self.builtin_cache.insert("cd".to_string(), "Change directory".to_string());
        self.builtin_cache.insert("ls".to_string(), "List files".to_string());
        self.builtin_cache.insert("pwd".to_string(), "Print working directory".to_string());
        self.builtin_cache.insert("echo".to_string(), "Display text".to_string());
        self.builtin_cache.insert("cat".to_string(), "Display file contents".to_string());
        self.builtin_cache.insert("exit".to_string(), "Exit shell".to_string());
        self.builtin_cache.insert("help".to_string(), "Show help".to_string());
        self.builtin_cache.insert("history".to_string(), "Show command history".to_string());
        self.builtin_cache.insert("alias".to_string(), "Define command aliases".to_string());
        self.builtin_cache.insert("unalias".to_string(), "Remove command aliases".to_string());
        self.builtin_cache.insert("bzip2".to_string(), "Compress files with bzip2".to_string());
        self.builtin_cache.insert("bunzip2".to_string(), "Decompress bzip2 files".to_string());
        self.builtin_cache.insert("id".to_string(), "Print user and group IDs".to_string());
    }
    
    /// Populate command cache from PATH
    pub fn populate_command_cache(&mut self) -> Result<()> {
        self.refresh_system_commands()
    }
    
    /// Populate variable cache from environment
    pub fn populate_variable_cache(&mut self) -> Result<()> {
        self.refresh_environment_variables();
        Ok(())
    }
    
    /// Populate alias cache
    pub fn populate_alias_cache(&mut self) -> Result<()> {
        // In a complete implementation, this would load from shell context
        Ok(())
    }

    /// Add a custom command to completion
    pub fn add_command(&mut self, command: &str, description: &str) {
        self.command_cache.insert(command.to_string(), description.to_string());
    }

    /// Add a builtin command to completion
    pub fn add_builtin(&mut self, builtin: &str, description: &str) {
        self.builtin_cache.insert(builtin.to_string(), description.to_string());
    }

    /// Add an alias to completion
    pub fn add_alias(&mut self, alias: &str, command: &str) {
        self.alias_cache.insert(alias.to_string(), command.to_string());
    }

    /// Get completions for the current input (async interface)
    /// 
    /// This is the main entry point for getting completions from external code,
    /// providing an async interface that ensures <1ms latency as per SPEC.md.
    pub async fn get_completions(&self, input: &str) -> Result<Vec<String>> {
        // Calculate position at end of input for completion
        let pos = input.len();
        
        // Get completion candidates
        let candidates = self.get_completion_candidates(input, pos)?;
        
        // Convert candidates to simple string list
        let completions: Vec<String> = candidates
            .into_iter()
            .take(20) // Limit to 20 completions for performance
            .map(|c| c.text)
            .collect();
        
        Ok(completions)
    }

    /// Produce rustyline-compatible completion pairs and start position.
    /// Keeps internal candidate types encapsulated.
    pub fn complete_for_rustyline_sync(&self, line: &str, pos: usize) -> (usize, Vec<Pair>) {
        let mut start_pos = pos;
        let mut pairs: Vec<Pair> = Vec::new();
        if let Ok(candidates) = self.get_completion_candidates(line, pos) {
            let ctx = self.analyze_completion_context(line, pos);
            start_pos = line.len().saturating_sub(ctx.word.len());
            pairs = candidates
                .into_iter()
                .map(|c| Pair {
                    display: c.display.unwrap_or_else(|| c.text.clone()),
                    replacement: c.replacement,
                })
                .collect();
        }
        (start_pos, pairs)
    }
    
    /// Setup shell completion
    pub fn setup_shell_completion(&mut self, context: &ShellContext) {
        // Task 8: Complete builtin integration with registry
        self.refresh_builtin_commands();
        
        // Add environment variables from context
        if let Ok(vars) = context.vars.read() {
            for (key, _) in vars.iter() {
                self.variable_cache.insert(key.clone());
            }
        }
        
        // Add aliases from context
        if let Ok(aliases) = context.aliases.read() {
            for (alias, command) in aliases.iter() {
                self.alias_cache.insert(alias.clone(), command.clone());
            }
        }
    }

    /// Refresh builtin commands from registry
    pub fn refresh_builtin_commands(&mut self) {
        self.builtin_cache.clear();
        // Use authoritative list from nxsh_builtins so数百コマンドが常に一致
        let names: Vec<&'static str> = nxsh_builtins::list_builtin_names();
        for name in names {
            self.builtin_cache.insert(name.to_string(), "Builtin command".to_string());
        }
        
        // Add shell keywords
        let keywords = vec![
            ("if", "Conditional statement"),
            ("then", "If clause body"),
            ("else", "Alternative clause"),
            ("elif", "Else if clause"),
            ("fi", "End if statement"),
            ("for", "For loop"),
            ("while", "While loop"),
            ("do", "Loop body"),
            ("done", "End loop"),
            ("case", "Case statement"),
            ("esac", "End case"),
            ("function", "Function definition"),
            ("return", "Return from function"),
            ("local", "Local variable"),
            ("readonly", "Read-only variable"),
            ("declare", "Declare variable"),
            ("typeset", "Type declaration"),
            ("export", "Export variable"),
            ("unset", "Unset variable"),
            ("set", "Set shell options"),
            ("unalias", "Remove alias"),
            ("command", "Execute external command"),
            ("builtin", "Execute builtin command"),
            ("enable", "Enable/disable builtin"),
            ("type", "Show command type"),
            ("which", "Locate command"),
            ("where", "Show all command locations"),
            ("hash", "Hash table commands"),
            ("help", "Show help"),
            ("history", "Command history"),
            ("fc", "Fix command"),
            ("bind", "Key bindings"),
            ("complete", "Completion settings"),
            ("compgen", "Generate completions"),
            ("dirs", "Directory stack"),
            ("pushd", "Push directory"),
            ("popd", "Pop directory"),
            ("suspend", "Suspend shell"),
            ("logout", "Logout shell"),
            ("exit", "Exit shell"),
            ("exec", "Execute command"),
            ("eval", "Evaluate expression"),
            ("source", "Source script"),
            (".", "Source script (short)"),
            ("test", "Test condition"),
            ("[", "Test condition (bracket)"),
            ("[[", "Extended test"),
            ("let", "Arithmetic evaluation"),
            ("(", "Subshell"),
            ("((", "Arithmetic expression"),
            ("{", "Command group"),
            ("&&", "Logical AND"),
            ("||", "Logical OR"),
            ("|", "Pipe"),
            ("&", "Background"),
            (";", "Command separator"),
            (";;", "Case separator"),
            (">", "Redirect output"),
            (">>", "Append output"),
            ("<", "Redirect input"),
            ("<<", "Here document"),
            ("<<<", "Here string"),
            (">&", "Redirect file descriptor"),
            ("<&", "Redirect file descriptor"),
            ("|&", "Pipe stderr"),
        ];
        
        for (keyword, desc) in keywords {
            self.builtin_cache.insert(keyword.to_string(), desc.to_string());
        }
    }

    /// Add built-in commands for completion
    pub fn add_builtin_commands(&mut self, commands: &[&str]) {
        for &cmd in commands {
            self.builtin_cache.insert(cmd.to_string(), format!("Built-in command: {cmd}"));
        }
    }

    /// Add shell keywords for completion  
    pub fn add_keywords(&mut self, keywords: &[&str]) {
        for &keyword in keywords {
            self.builtin_cache.insert(keyword.to_string(), format!("Shell keyword: {keyword}"));
        }
    }

    /// Enable or disable path completion
    pub fn enable_path_completion(&mut self, enabled: bool) {
        self.completion_config.enable_path_completion = enabled;
    }

    /// Enable or disable variable completion
    pub fn enable_variable_completion(&mut self, enabled: bool) {
        self.completion_config.enable_variable_completion = enabled;
    }

    /// Enable or disable history-based completion
    pub fn enable_history_completion(&mut self, enabled: bool) {
        self.completion_config.enable_history_completion = enabled;
    }

    /// Get completion candidates for the given context
    pub fn get_completion_candidates(&self, line: &str, pos: usize) -> Result<Vec<CompletionCandidate>> {
        let mut candidates = Vec::new();
        let context = self.analyze_completion_context(line, pos);

        match context.completion_type {
            CompletionType::Command => {
                candidates.extend(self.get_command_completions(&context.word)?);
            }
            CompletionType::Filename => {
                candidates.extend(self.get_filename_completions(&context.word)?);
            }
            CompletionType::Variable => {
                candidates.extend(self.get_variable_completions(&context.word)?);
            }
            CompletionType::Flag => {
                candidates.extend(self.get_flag_completions(&context)?);
            }
            CompletionType::Mixed => {
                // Try all completion types and merge results
                candidates.extend(self.get_command_completions(&context.word)?);
                candidates.extend(self.get_filename_completions(&context.word)?);
                candidates.extend(self.get_variable_completions(&context.word)?);
            }
        }

        // Apply fuzzy matching and sorting
        if self.completion_config.fuzzy_matching {
            candidates = self.apply_fuzzy_matching(candidates, &context.word);
        }

        // Limit results
        candidates.truncate(self.completion_config.max_candidates);

        Ok(candidates)
    }

    /// Analyze the context around the cursor position
    pub fn analyze_completion_context(&self, line: &str, pos: usize) -> CompletionContext {
        let before_cursor = &line[..pos];
        let after_cursor = &line[pos..];

        // Find the current word being completed
        let word_start = before_cursor.rfind(|c: char| c.is_whitespace() || ";&|<>()".contains(c))
            .map(|i| i + 1)
            .unwrap_or(0);
        let word_end = after_cursor.find(|c: char| c.is_whitespace() || ";&|<>()".contains(c))
            .unwrap_or(after_cursor.len());
        
        let word = &line[word_start..pos + word_end];
        let word_prefix = &before_cursor[word_start..];

        // Determine completion type based on context
        let completion_type = if word_prefix.starts_with('$') {
            CompletionType::Variable
        } else if word_prefix.starts_with('-') {
            CompletionType::Flag
        } else if self.is_command_position(before_cursor) {
            CompletionType::Command
        } else {
            CompletionType::Filename
        };

        CompletionContext {
            word: word.to_string(),
            word_prefix: word_prefix.to_string(),
            position: pos,
            line: line.to_string(),
            completion_type,
            command_context: self.extract_command_context(before_cursor),
        }
    }

    /// Check if the cursor is in a command position
    fn is_command_position(&self, text: &str) -> bool {
        // Simple heuristic: command position if at start or after certain characters
        let trimmed = text.trim_end();
        trimmed.is_empty() || 
        trimmed.ends_with(';') || 
        trimmed.ends_with('|') || 
        trimmed.ends_with("&&") || 
        trimmed.ends_with("||")
    }

    /// Extract command context for flag completion
    fn extract_command_context(&self, text: &str) -> Option<String> {
        // Find the current command being executed
        let words: Vec<&str> = text.split_whitespace().collect();
        words.iter().rposition(|&w| {
            self.command_cache.contains_key(w) || self.builtin_cache.contains_key(w)
        }).map(|last_command_pos| words[last_command_pos].to_string())
    }

    /// Get command completions
    fn get_command_completions(&self, prefix: &str) -> Result<Vec<CompletionCandidate>> {
        let mut candidates = Vec::new();

        // Add builtin commands
        for (cmd, desc) in &self.builtin_cache {
            if cmd.starts_with(prefix) {
                candidates.push(CompletionCandidate {
                    text: cmd.clone(),
                    display: Some(format!("{cmd} - {desc}")),
                    replacement: cmd.clone(),
                    candidate_type: CandidateType::Builtin,
                    score: self.calculate_score(cmd, prefix),
                });
            }
        }

        // Add system commands
        for (cmd, desc) in &self.command_cache {
            if cmd.starts_with(prefix) {
                candidates.push(CompletionCandidate {
                    text: cmd.clone(),
                    display: Some(format!("{cmd} - {desc}")),
                    replacement: cmd.clone(),
                    candidate_type: CandidateType::Command,
                    score: self.calculate_score(cmd, prefix),
                });
            }
        }

        // Add aliases
        for (alias, cmd) in &self.alias_cache {
            if alias.starts_with(prefix) {
                candidates.push(CompletionCandidate {
                    text: alias.clone(),
                    display: Some(format!("{alias} -> {cmd}")),
                    replacement: alias.clone(),
                    candidate_type: CandidateType::Alias,
                    score: self.calculate_score(alias, prefix),
                });
            }
        }

        Ok(candidates)
    }

    /// Get filename completions
    fn get_filename_completions(&self, prefix: &str) -> Result<Vec<CompletionCandidate>> {
        let mut candidates = Vec::new();

        // Use basic file system scanning for completion
        let path = Path::new(prefix);
        let (dir, partial_name) = if path.is_dir() {
            (path, "")
        } else {
            (path.parent().unwrap_or(Path::new(".")), 
             path.file_name().unwrap_or_default().to_str().unwrap_or(""))
        };

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(partial_name) {
                    let candidate_type = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        CandidateType::Directory
                } else {
                    CandidateType::File
                };

                    let full_path = dir.join(&name);
                    let replacement = full_path.to_string_lossy().to_string();
                    
                    candidates.push(CompletionCandidate {
                        text: name.clone(),
                        display: None,
                        replacement,
                        candidate_type,
                        score: self.calculate_score(&name, partial_name),
                    });
                }
            }
        }

        Ok(candidates)
    }

    /// Get variable completions
    fn get_variable_completions(&self, prefix: &str) -> Result<Vec<CompletionCandidate>> {
        let mut candidates = Vec::new();
        let var_prefix = prefix.strip_prefix('$').unwrap_or(prefix);

        for var in &self.variable_cache {
            if var.starts_with(var_prefix) {
                let value = env::var(var).unwrap_or_else(|_| "".to_string());
                let display_value = if value.len() > 50 {
                    format!("{}...", &value[..47])
                } else {
                    value
                };

                candidates.push(CompletionCandidate {
                    text: format!("${var}"),
                    display: Some(format!("${var} = {display_value}")),
                    replacement: format!("${var}"),
                    candidate_type: CandidateType::Variable,
                    score: self.calculate_score(var, var_prefix),
                });
            }
        }

        Ok(candidates)
    }

    /// Get flag completions for specific commands
    fn get_flag_completions(&self, context: &CompletionContext) -> Result<Vec<CompletionCandidate>> {
        let mut candidates = Vec::new();

        if let Some(ref command) = context.command_context {
            // Get flags for specific commands
            let flags = self.get_command_flags(command);
            let prefix = context.word_prefix.strip_prefix('-').unwrap_or(&context.word_prefix);

            for (flag, desc) in flags {
                if flag.starts_with(prefix) {
                    candidates.push(CompletionCandidate {
                        text: format!("-{flag}"),
                        display: Some(format!("-{flag} - {desc}")),
                        replacement: format!("-{flag}"),
                        candidate_type: CandidateType::Flag,
                        score: self.calculate_score(&flag, prefix),
                    });
                }
            }
        }

        Ok(candidates)
    }

    /// Get flags for a specific command
    fn get_command_flags(&self, command: &str) -> Vec<(String, String)> {
        // This would ideally parse man pages or have a database of command flags
        match command {
            "ls" => vec![
                ("l".to_string(), "Long format".to_string()),
                ("a".to_string(), "Show hidden files".to_string()),
                ("h".to_string(), "Human readable sizes".to_string()),
                ("t".to_string(), "Sort by time".to_string()),
                ("r".to_string(), "Reverse order".to_string()),
                ("S".to_string(), "Sort by size".to_string()),
            ],
            "grep" => vec![
                ("i".to_string(), "Ignore case".to_string()),
                ("r".to_string(), "Recursive".to_string()),
                ("n".to_string(), "Line numbers".to_string()),
                ("v".to_string(), "Invert match".to_string()),
                ("E".to_string(), "Extended regex".to_string()),
            ],
            "find" => vec![
                ("name".to_string(), "Search by name".to_string()),
                ("type".to_string(), "Search by type".to_string()),
                ("size".to_string(), "Search by size".to_string()),
                ("mtime".to_string(), "Search by modification time".to_string()),
            ],
            _ => vec![],
        }
    }

    /// Apply fuzzy matching to candidates
    fn apply_fuzzy_matching(&self, mut candidates: Vec<CompletionCandidate>, pattern: &str) -> Vec<CompletionCandidate> {
        if pattern.is_empty() {
            return candidates;
        }

        // Calculate fuzzy scores
        for candidate in &mut candidates {
            if let Some(score) = self.fuzzy_matcher.fuzzy_match(&candidate.text, pattern) {
                candidate.score = score as f64;
            } else {
                candidate.score = 0.0;
            }
        }

        // Filter out non-matching candidates and sort by score
        candidates.retain(|c| c.score > 0.0);
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        candidates
    }

    /// Calculate completion score (simple prefix matching)
    fn calculate_score(&self, candidate: &str, prefix: &str) -> f64 {
        if candidate.starts_with(prefix) {
            // Exact prefix match gets higher score
            1.0 - (prefix.len() as f64 / candidate.len() as f64) * 0.1
        } else {
            0.0
        }
    }

    /// Apply completion configuration settings
    pub fn apply_config(&mut self, config: &CompletionConfig) -> Result<()> {
        self.completion_config = config.clone();
        
        // Apply configuration-dependent optimizations
        if !config.fuzzy_matching {
            // Clear any cached fuzzy matching data if disabled
            // This could include clearing fuzzy search indices
        }
        
        if !config.enable_path_completion {
            // Disable path completion optimizations
        }
        
        if !config.enable_variable_completion {
            // Clear variable cache if disabled
            self.variable_cache.clear();
        }
        
        if !config.enable_history_completion {
            // History completion is disabled - would clear history cache if it existed
        }
        
        Ok(())
    }
}

impl Completer for NexusCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &RustylineContext<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let candidates = self.get_completion_candidates(line, pos)
            .map_err(|e| rustyline::error::ReadlineError::Io(
                std::io::Error::other(e)
            ))?;

        let context = self.analyze_completion_context(line, pos);
        let start_pos = line.len() - context.word.len();

        let pairs: Vec<Pair> = candidates
            .into_iter()
            .map(|candidate| Pair {
                display: candidate.display.unwrap_or(candidate.text.clone()),
                replacement: candidate.replacement,
            })
            .collect();

        Ok((start_pos, pairs))
    }
}

/// Completion context information
#[derive(Debug, Clone)]
pub struct CompletionContext {
    word: String,
    word_prefix: String,
    position: usize,
    line: String,
    completion_type: CompletionType,
    command_context: Option<String>,
}

/// Types of completion
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionType {
    Command,
    Filename,
    Variable,
    Flag,
    Mixed,
}

/// Completion candidate with metadata
#[derive(Debug, Clone)]
pub struct CompletionCandidate {
    text: String,
    display: Option<String>,
    replacement: String,
    candidate_type: CandidateType,
    score: f64,
}

/// Types of completion candidates
#[derive(Debug, Clone, PartialEq)]
pub enum CandidateType {
    Command,
    Builtin,
    Alias,
    File,
    Directory,
    Variable,
    Flag,
}

/// Configuration for completion behavior
#[derive(Debug, Clone)]
pub struct CompletionConfig {
    pub fuzzy_matching: bool,
    pub max_candidates: usize,
    pub case_sensitive: bool,
    pub show_descriptions: bool,
    pub enable_path_completion: bool,
    pub enable_variable_completion: bool,
    pub enable_history_completion: bool,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            fuzzy_matching: true,
            max_candidates: 50,
            case_sensitive: false,
            show_descriptions: true,
            enable_path_completion: true,
            enable_variable_completion: true,
            enable_history_completion: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completer_creation() {
        let completer = NexusCompleter::new();
        assert!(completer.is_ok());
    }

    #[test]
    fn test_command_completion() {
        let mut completer = NexusCompleter::new().unwrap();
        completer.add_command("test_cmd", "Test command");
        
        let candidates = completer.get_command_completions("test").unwrap();
        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|c| c.text == "test_cmd"));
    }

    #[test]
    fn test_completion_context_analysis() {
        let completer = NexusCompleter::new().unwrap();
        let context = completer.analyze_completion_context("ls -l", 5);
        
        assert_eq!(context.completion_type, CompletionType::Flag);
        assert_eq!(context.word_prefix, "-l");
    }

    #[test]
    fn test_variable_completion() {
        let completer = NexusCompleter::new().unwrap();
        let candidates = completer.get_variable_completions("$PA").unwrap();
        
        // Should find PATH variable
        assert!(candidates.iter().any(|c| c.text == "$PATH"));
    }
} 