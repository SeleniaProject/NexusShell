//! 補完システム統合モジュール - NexusShell Completion Integration
//!
//! このモジュールは、NexusShellの補完システムと他のコンポーネントとの統合を管理します。
//! シェル状態、パーサー、プラグインシステムとの連携を提供。

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use anyhow::{Result, Error};
use tokio::sync::mpsc;

use crate::completion::{CompletionEngine, CompletionResult, CompletionItem, CompletionType};
use crate::config::UiConfig;

// シェル状態との統合のためのトレイト
pub trait ShellStateProvider: Send + Sync {
    fn get_current_directory(&self) -> Result<std::path::PathBuf>;
    fn get_environment_variables(&self) -> Result<HashMap<String, String>>;
    fn get_aliases(&self) -> Result<HashMap<String, String>>;
    fn get_functions(&self) -> Result<Vec<String>>;
    fn get_history(&self) -> Result<Vec<String>>;
    fn is_command_available(&self, command: &str) -> bool;
}

// パーサー統合のためのトレイト
pub trait ParserIntegration: Send + Sync {
    fn parse_command_line(&self, input: &str, cursor: usize) -> Result<CommandContext>;
    fn get_completion_context(&self, input: &str, cursor: usize) -> Result<CompletionContext>;
    fn suggest_corrections(&self, input: &str) -> Result<Vec<String>>;
}

// コマンドコンテキスト
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub command: String,
    pub arguments: Vec<String>,
    pub current_argument_index: Option<usize>,
    pub is_in_quote: bool,
    pub quote_type: Option<char>,
    pub pipeline_position: usize,
    pub redirection_target: Option<String>,
}

// 補完コンテキスト
#[derive(Debug, Clone)]
pub struct CompletionContext {
    pub completion_type: ContextType,
    pub prefix: String,
    pub command_name: Option<String>,
    pub argument_position: Option<usize>,
    pub in_pipeline: bool,
    pub expected_types: Vec<CompletionType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContextType {
    Command,
    Argument,
    Option,
    OptionValue,
    File,
    Directory,
    Variable,
    Redirection,
    Pipeline,
}

// プラグイン補完プロバイダー
pub trait PluginCompletionProvider: Send + Sync {
    fn plugin_name(&self) -> &str;
    fn supports_command(&self, command: &str) -> bool;
    fn get_command_completions(&self, command: &str, args: &[String], cursor_arg: usize) -> Result<Vec<CompletionItem>>;
    fn get_option_completions(&self, command: &str, option: &str) -> Result<Vec<CompletionItem>>;
}

// 統合補完システム
pub struct IntegratedCompletionSystem {
    engine: CompletionEngine,
    shell_state: Option<Arc<dyn ShellStateProvider>>,
    parser: Option<Arc<dyn ParserIntegration>>,
    plugin_providers: Vec<Arc<dyn PluginCompletionProvider>>,
    config: UiConfig,
    command_cache: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl IntegratedCompletionSystem {
    pub fn new(config: UiConfig) -> Self {
        Self {
            engine: CompletionEngine::new(),
            shell_state: None,
            parser: None,
            plugin_providers: Vec::new(),
            config,
            command_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set_shell_state_provider(&mut self, provider: Arc<dyn ShellStateProvider>) {
        self.shell_state = Some(provider);
    }

    pub fn set_parser(&mut self, parser: Arc<dyn ParserIntegration>) {
        self.parser = Some(parser);
    }

    pub fn add_plugin_provider(&mut self, provider: Arc<dyn PluginCompletionProvider>) {
        self.plugin_providers.push(provider);
    }

    pub fn get_intelligent_completions(&self, input: &str, cursor: usize) -> Result<CompletionResult> {
        // Get basic completions from engine
        let mut result = self.engine.get_completions_at_cursor(input, cursor);

        // Enhance with context-aware completions
        if let Some(parser) = &self.parser {
            match parser.get_completion_context(input, cursor) {
                Ok(context) => {
                    let enhanced_items = self.get_context_aware_completions(&context, input, cursor)?;
                    result.items.extend(enhanced_items);
                }
                Err(e) => {
                    eprintln!("Parser integration failed: {}", e);
                }
            }
        }

        // Add shell state completions
        if let Some(shell_state) = &self.shell_state {
            let shell_items = self.get_shell_state_completions(shell_state, input, cursor)?;
            result.items.extend(shell_items);
        }

        // Add plugin completions
        let plugin_items = self.get_plugin_completions(input, cursor)?;
        result.items.extend(plugin_items);

        // Sort and deduplicate
        result.items.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
        });
        result.items.dedup_by(|a, b| a.text == b.text);

        // Limit results
        if result.items.len() > self.config.completion.max_suggestions {
            result.items.truncate(self.config.completion.max_suggestions);
        }

        Ok(result)
    }

    fn get_context_aware_completions(
        &self,
        context: &CompletionContext,
        input: &str,
        cursor: usize,
    ) -> Result<Vec<CompletionItem>> {
        let mut items = Vec::new();

        match context.completion_type {
            ContextType::Command => {
                items.extend(self.get_command_completions(&context.prefix)?);
            }
            ContextType::Argument => {
                if let Some(command) = &context.command_name {
                    items.extend(self.get_argument_completions(command, &context.prefix, context.argument_position)?);
                }
            }
            ContextType::Option => {
                if let Some(command) = &context.command_name {
                    items.extend(self.get_option_completions(command, &context.prefix)?);
                }
            }
            ContextType::OptionValue => {
                if let Some(command) = &context.command_name {
                    items.extend(self.get_option_value_completions(command, &context.prefix)?);
                }
            }
            ContextType::File => {
                items.extend(self.get_file_completions(&context.prefix, false)?);
            }
            ContextType::Directory => {
                items.extend(self.get_file_completions(&context.prefix, true)?);
            }
            ContextType::Variable => {
                items.extend(self.get_variable_completions(&context.prefix)?);
            }
            ContextType::Redirection => {
                items.extend(self.get_file_completions(&context.prefix, false)?);
            }
            ContextType::Pipeline => {
                items.extend(self.get_command_completions(&context.prefix)?);
            }
        }

        Ok(items)
    }

    fn get_command_completions(&self, prefix: &str) -> Result<Vec<CompletionItem>> {
        let mut items = Vec::new();

        // Add builtin commands
        let builtins = vec![
            "cd", "ls", "pwd", "echo", "cat", "cp", "mv", "rm", "mkdir", "rmdir",
            "grep", "find", "sort", "uniq", "wc", "head", "tail", "less", "more",
            "ps", "kill", "jobs", "bg", "fg", "history", "alias", "unalias",
            "export", "unset", "source", "exit", "help", "which", "type",
        ];

        for builtin in builtins {
            if builtin.starts_with(prefix) {
                let item = CompletionItem::new(builtin.to_string(), CompletionType::Command)
                    .with_description(format!("Builtin command: {}", builtin))
                    .with_source("builtins".to_string());
                items.push(item);
            }
        }

        // Add aliases if available
        if let Some(shell_state) = &self.shell_state {
            if let Ok(aliases) = shell_state.get_aliases() {
                for (alias, command) in aliases {
                    if alias.starts_with(prefix) {
                        let item = CompletionItem::new(alias.clone(), CompletionType::Alias)
                            .with_description(format!("Alias for: {}", command))
                            .with_source("aliases".to_string());
                        items.push(item);
                    }
                }
            }
        }

        // Add functions if available
        if let Some(shell_state) = &self.shell_state {
            if let Ok(functions) = shell_state.get_functions() {
                for function in functions {
                    if function.starts_with(prefix) {
                        let item = CompletionItem::new(function.clone(), CompletionType::Function)
                            .with_description("Shell function".to_string())
                            .with_source("functions".to_string());
                        items.push(item);
                    }
                }
            }
        }

        Ok(items)
    }

    fn get_argument_completions(
        &self,
        command: &str,
        prefix: &str,
        position: Option<usize>,
    ) -> Result<Vec<CompletionItem>> {
        let mut items = Vec::new();

        // Check if any plugin can handle this command
        for provider in &self.plugin_providers {
            if provider.supports_command(command) {
                let args: Vec<String> = vec![prefix.to_string()];
                match provider.get_command_completions(command, &args, position.unwrap_or(0)) {
                    Ok(plugin_items) => items.extend(plugin_items),
                    Err(e) => eprintln!("Plugin completion failed for {}: {}", command, e),
                }
            }
        }

        // Default to file completions for most commands
        if items.is_empty() {
            items.extend(self.get_file_completions(prefix, false)?);
        }

        Ok(items)
    }

    fn get_option_completions(&self, command: &str, prefix: &str) -> Result<Vec<CompletionItem>> {
        let mut items = Vec::new();

        // Common options for many commands
        let common_options = vec![
            ("--help", "Show help information"),
            ("--version", "Show version information"),
            ("--verbose", "Enable verbose output"),
            ("--quiet", "Enable quiet mode"),
            ("--force", "Force operation"),
            ("-h", "Show help (short)"),
            ("-v", "Verbose output (short)"),
            ("-q", "Quiet mode (short)"),
            ("-f", "Force operation (short)"),
        ];

        for (option, description) in common_options {
            if option.starts_with(prefix) {
                let item = CompletionItem::new(option.to_string(), CompletionType::Option)
                    .with_description(description.to_string())
                    .with_source("common_options".to_string());
                items.push(item);
            }
        }

        // Command-specific options
        match command {
            "ls" => {
                let ls_options = vec![
                    ("--all", "Show hidden files"),
                    ("--long", "Use long listing format"),
                    ("--human-readable", "Human readable sizes"),
                    ("--sort", "Sort by criteria"),
                    ("-a", "Show hidden files (short)"),
                    ("-l", "Long format (short)"),
                    ("-h", "Human readable (short)"),
                ];
                
                for (option, description) in ls_options {
                    if option.starts_with(prefix) {
                        let item = CompletionItem::new(option.to_string(), CompletionType::Option)
                            .with_description(description.to_string())
                            .with_source("ls_options".to_string());
                        items.push(item);
                    }
                }
            }
            "cp" | "mv" => {
                let copy_options = vec![
                    ("--recursive", "Copy directories recursively"),
                    ("--interactive", "Prompt before overwrite"),
                    ("--backup", "Create backup of destination"),
                    ("-r", "Recursive (short)"),
                    ("-i", "Interactive (short)"),
                ];
                
                for (option, description) in copy_options {
                    if option.starts_with(prefix) {
                        let item = CompletionItem::new(option.to_string(), CompletionType::Option)
                            .with_description(description.to_string())
                            .with_source("copy_options".to_string());
                        items.push(item);
                    }
                }
            }
            _ => {
                // Check plugins for command-specific options
                for provider in &self.plugin_providers {
                    if provider.supports_command(command) {
                        match provider.get_option_completions(command, prefix) {
                            Ok(plugin_items) => items.extend(plugin_items),
                            Err(e) => eprintln!("Plugin option completion failed for {}: {}", command, e),
                        }
                    }
                }
            }
        }

        Ok(items)
    }

    fn get_option_value_completions(&self, command: &str, prefix: &str) -> Result<Vec<CompletionItem>> {
        let mut items = Vec::new();

        // This would be expanded based on the specific option being completed
        // For now, default to file completions
        items.extend(self.get_file_completions(prefix, false)?);

        Ok(items)
    }

    fn get_file_completions(&self, prefix: &str, directories_only: bool) -> Result<Vec<CompletionItem>> {
        let mut items = Vec::new();

        // Use the current directory from shell state if available
        let current_dir = if let Some(shell_state) = &self.shell_state {
            shell_state.get_current_directory().unwrap_or_else(|_| std::env::current_dir().unwrap_or_default())
        } else {
            std::env::current_dir().unwrap_or_default()
        };

        let search_dir = if prefix.contains('/') || prefix.contains('\\') {
            let path_part = std::path::Path::new(prefix);
            if let Some(parent) = path_part.parent() {
                current_dir.join(parent)
            } else {
                current_dir
            }
        } else {
            current_dir
        };

        if let Ok(entries) = std::fs::read_dir(search_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with(prefix) {
                        if directories_only && !path.is_dir() {
                            continue;
                        }

                        let completion_type = if path.is_dir() {
                            CompletionType::Directory
                        } else {
                            CompletionType::File
                        };

                        let item = CompletionItem::new(name.to_string(), completion_type)
                            .with_source("filesystem".to_string());
                        items.push(item);
                    }
                }
            }
        }

        Ok(items)
    }

    fn get_variable_completions(&self, prefix: &str) -> Result<Vec<CompletionItem>> {
        let mut items = Vec::new();

        if let Some(shell_state) = &self.shell_state {
            if let Ok(vars) = shell_state.get_environment_variables() {
                for (name, value) in vars {
                    if name.starts_with(prefix) {
                        let item = CompletionItem::new(format!("${}", name), CompletionType::Variable)
                            .with_description(format!("Environment variable: {}", value))
                            .with_source("environment".to_string());
                        items.push(item);
                    }
                }
            }
        }

        Ok(items)
    }

    fn get_shell_state_completions(
        &self,
        shell_state: &Arc<dyn ShellStateProvider>,
        input: &str,
        cursor: usize,
    ) -> Result<Vec<CompletionItem>> {
        let mut items = Vec::new();

        // Add history-based completions
        if let Ok(history) = shell_state.get_history() {
            let prefix = &input[..cursor.min(input.len())];
            for entry in history.iter().rev().take(20) {
                if entry.starts_with(prefix) && entry != input {
                    let item = CompletionItem::new(entry.clone(), CompletionType::History)
                        .with_description("From history".to_string())
                        .with_source("history".to_string());
                    items.push(item);
                }
            }
        }

        Ok(items)
    }

    fn get_plugin_completions(&self, input: &str, cursor: usize) -> Result<Vec<CompletionItem>> {
        let mut items = Vec::new();

        if let Some(parser) = &self.parser {
            if let Ok(context) = parser.parse_command_line(input, cursor) {
                for provider in &self.plugin_providers {
                    if provider.supports_command(&context.command) {
                        match provider.get_command_completions(
                            &context.command,
                            &context.arguments,
                            context.current_argument_index.unwrap_or(0),
                        ) {
                            Ok(plugin_items) => items.extend(plugin_items),
                            Err(e) => eprintln!("Plugin completion failed: {}", e),
                        }
                    }
                }
            }
        }

        Ok(items)
    }

    pub fn invalidate_cache(&self) {
        self.engine.clear_cache();
        if let Ok(mut cache) = self.command_cache.write() {
            cache.clear();
        }
    }

    pub fn get_completion_statistics(&self) -> HashMap<String, u64> {
        let metrics = self.engine.get_performance_metrics();
        let mut stats = HashMap::new();
        
        stats.insert("total_requests".to_string(), metrics.total_requests);
        stats.insert("cache_hits".to_string(), metrics.cache_hits);
        stats.insert("cache_misses".to_string(), metrics.cache_misses);
        stats.insert("average_time_ms".to_string(), metrics.average_time.as_millis() as u64);
        
        stats
    }
}

// Mock implementations for testing
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::path::PathBuf;

    pub struct MockShellState {
        current_dir: PathBuf,
        environment: HashMap<String, String>,
        aliases: HashMap<String, String>,
        history: Vec<String>,
    }

    impl MockShellState {
        pub fn new() -> Self {
            let mut env = HashMap::new();
            env.insert("HOME".to_string(), "/home/user".to_string());
            env.insert("PATH".to_string(), "/bin:/usr/bin".to_string());
            
            let mut aliases = HashMap::new();
            aliases.insert("ll".to_string(), "ls -la".to_string());
            aliases.insert("la".to_string(), "ls -la".to_string());

            Self {
                current_dir: PathBuf::from("/home/user"),
                environment: env,
                aliases,
                history: vec![
                    "ls -la".to_string(),
                    "cd /tmp".to_string(),
                    "echo hello".to_string(),
                ],
            }
        }
    }

    impl ShellStateProvider for MockShellState {
        fn get_current_directory(&self) -> Result<PathBuf> {
            Ok(self.current_dir.clone())
        }

        fn get_environment_variables(&self) -> Result<HashMap<String, String>> {
            Ok(self.environment.clone())
        }

        fn get_aliases(&self) -> Result<HashMap<String, String>> {
            Ok(self.aliases.clone())
        }

        fn get_functions(&self) -> Result<Vec<String>> {
            Ok(vec!["my_function".to_string(), "helper_func".to_string()])
        }

        fn get_history(&self) -> Result<Vec<String>> {
            Ok(self.history.clone())
        }

        fn is_command_available(&self, command: &str) -> bool {
            matches!(command, "ls" | "cd" | "echo" | "cat" | "grep")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::mock::MockShellState;

    #[test]
    fn test_integrated_completion_system() {
        let config = UiConfig::default();
        let mut system = IntegratedCompletionSystem::new(config);
        
        let shell_state = Arc::new(MockShellState::new());
        system.set_shell_state_provider(shell_state);

        // Test basic completion
        let result = system.get_intelligent_completions("l", 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_context() {
        let context = CommandContext {
            command: "ls".to_string(),
            arguments: vec!["-la".to_string()],
            current_argument_index: Some(1),
            is_in_quote: false,
            quote_type: None,
            pipeline_position: 0,
            redirection_target: None,
        };

        assert_eq!(context.command, "ls");
        assert_eq!(context.arguments.len(), 1);
    }

    #[test]
    fn test_completion_context() {
        let context = CompletionContext {
            completion_type: ContextType::Command,
            prefix: "l".to_string(),
            command_name: None,
            argument_position: None,
            in_pipeline: false,
            expected_types: vec![CompletionType::Command],
        };

        assert_eq!(context.completion_type, ContextType::Command);
        assert_eq!(context.prefix, "l");
    }
}
