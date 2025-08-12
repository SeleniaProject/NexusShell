//! CUI Line Editor for NexusShell
//! 
//! This module provides a standard readline-style line editing experience,
//! with basic history management and simple Tab completion.
//! Complex TUI-specific completion interfaces have been removed in favor of standard functionality.

use anyhow::{Result, Context};
use rustyline::{
    config::{Config},
    error::ReadlineError,
    Editor,
};
use std::{
    path::Path,
    fs,
};
use serde::{Deserialize, Serialize};

/// Standard CUI line editor with basic readline functionality
pub struct NexusLineEditor {
    editor: Editor<(), rustyline::history::DefaultHistory>,
    history_file: Option<String>,
    config: LineEditorConfig,
}

impl NexusLineEditor {
    /// Create a new line editor with standard readline configuration
    pub fn new() -> Result<Self> {
        let rustyline_config = Config::builder()
            .history_ignore_space(true)
            .history_ignore_dups(true)?
            .completion_type(rustyline::CompletionType::List)
            .build();

        let mut editor = Editor::with_config(rustyline_config)?;
        let mut editor = Editor::with_config(rustyline_config)?;
        editor.set_helper(Some(helper));

        // Set up essential key bindings for CUI mode
        Self::setup_cui_key_bindings(&mut editor)?;

        Ok(Self {
            editor,
            history_file: None,
            config: LineEditorConfig::default(),
            config_file: None,
        })
    }
    
    /// Create a line editor specifically configured for CUI mode
    /// This removes TUI-specific features and optimizes for performance
    pub fn new_cui_mode() -> Result<Self> {
        Self::new()
    }

    /// Set up essential key bindings optimized for CUI
    fn setup_cui_key_bindings(editor: &mut Editor<NexusHelper, DefaultHistory>) -> Result<()> {
        use rustyline::config::Configurer;
        use rustyline::{KeyEvent, Cmd};
        
        // Use simple emacs mode for CUI (more predictable)
        editor.set_edit_mode(rustyline::EditMode::Emacs);
        
        // Configure essential behaviors only
        editor.set_completion_type(rustyline::CompletionType::List);
        editor.set_history_ignore_space(true);
        editor.set_tab_stop(4); // Fixed tab width for CUI
        
        // Add explicit completion key binding (Ctrl+Space)
        editor.bind_sequence(
            KeyEvent::ctrl(' '),
            Cmd::Complete
        );
        
        // Also bind Ctrl+I for explicit completion (alternative)
        editor.bind_sequence(
            KeyEvent::ctrl('\t'), 
            Cmd::Complete
        );
        
        Ok(())
    }

    /// Read a line with CUI-optimized editing capabilities
    pub fn readline(&mut self, prompt: &str) -> Result<String> {
        match self.editor.readline(prompt) {
            Ok(line) => {
                // Add to history if not empty and not duplicate
                if !line.trim().is_empty() {
                    self.editor.add_history_entry(line.as_str())?;
                }
                Ok(line)
            }
            Err(ReadlineError::Interrupted) => {
                Err(anyhow::anyhow!("Interrupted"))
            }
            Err(ReadlineError::Eof) => {
                Err(anyhow::anyhow!("EOF"))
            }
            Err(err) => {
                Err(anyhow::anyhow!("Readline error: {}", err))
            }
        }
    }
    
    /// Read a line optimized for CUI mode (async-compatible)
    /// This method provides the same functionality as readline but is designed
    /// to work well with the CUI application's async event loop
    pub async fn readline_cui(&mut self) -> Result<String> {
        // For CUI mode, we use the standard readline but make it async-compatible
        // by running it in a blocking task
        let prompt = ""; // Prompt is handled separately in CUI mode
        
        // Use the editor directly since we need mutable access
        match self.editor.readline(prompt) {
            Ok(line) => {
                // Add to history if not empty and not duplicate
                if !line.trim().is_empty() {
                    self.editor.add_history_entry(line.as_str())?;
                }
                Ok(line)
            }
            Err(ReadlineError::Interrupted) => {
                Err(anyhow::anyhow!("Interrupted"))
            }
            Err(ReadlineError::Eof) => {
                Err(anyhow::anyhow!("EOF"))
            }
            Err(err) => {
                Err(anyhow::anyhow!("Readline error: {}", err))
            }
        }
    }

    /// Read multiple lines (for script editing)
    pub fn read_multiline(&mut self, prompt: &str, continuation_prompt: &str) -> Result<String> {
        let mut lines = Vec::new();
        let mut line_prompt = prompt.to_string();

        loop {
            match self.editor.readline(&line_prompt) {
                Ok(line) => {
                    // Check if line ends with continuation indicator
                    if line.trim_end().ends_with('\\') {
                        lines.push(line.trim_end().trim_end_matches('\\').to_string());
                        line_prompt = continuation_prompt.to_string();
                        continue;
                    } else {
                        lines.push(line);
                        break;
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    return Err(anyhow::anyhow!("Interrupted"));
                }
                Err(ReadlineError::Eof) => {
                    if lines.is_empty() {
                        return Err(anyhow::anyhow!("EOF"));
                    } else {
                        break;
                    }
                }
                Err(err) => {
                    return Err(anyhow::anyhow!("Readline error: {}", err));
                }
            }
        }

        let result = lines.join("\n");
        if !result.trim().is_empty() {
            self.editor.add_history_entry(result.as_str())?;
        }
        Ok(result)
    }

    /// Load history from file
    pub fn load_history<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        if path.exists() {
            self.editor.load_history(path)
                .context("Failed to load history")?;
        }
        self.history_file = Some(path.to_string_lossy().to_string());
        Ok(())
    }

    /// Save history to file
    pub fn save_history(&mut self) -> Result<()> {
        if let Some(ref path) = self.history_file {
            self.editor.save_history(path)
                .context("Failed to save history")?;
        }
        Ok(())
    }

    /// Add a custom command to completion
    pub fn add_command(&mut self, command: &str, description: &str) {
        if let Some(helper) = self.editor.helper_mut() {
            helper.add_command(command, description);
        }
    }

    /// Get command history
    pub fn history(&self) -> Vec<String> {
        self.editor.history()
            .iter()
            .map(|entry| entry.to_string())
            .collect()
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        let _ = self.editor.clear_history();
    }
    
    /// Force completion at current cursor position
    /// This provides explicit completion functionality beyond automatic Tab completion
    pub fn complete_at_cursor(&mut self, input: &str, pos: usize) -> Result<Vec<String>> {
        if let Some(helper) = self.editor.helper() {
            let history = self.editor.history();
            let ctx = RustylineContext::new(history);
            match helper.complete(input, pos, &ctx) {
                Ok((_start, candidates)) => {
                    Ok(candidates.into_iter()
                        .map(|pair| pair.replacement().to_string())
                        .collect())
                }
                Err(e) => Err(anyhow::anyhow!("Completion failed: {}", e))
            }
        } else {
            Ok(vec![])
        }
    }
    
    /// Get completion suggestions for current input
    /// Returns formatted completion list with descriptions
    pub fn get_completions(&mut self, input: &str, pos: usize) -> Result<Vec<(String, String)>> {
        if let Some(helper) = self.editor.helper() {
            let history = self.editor.history();
            let ctx = RustylineContext::new(history);
            match helper.complete(input, pos, &ctx) {
                Ok((_start, candidates)) => {
                    Ok(candidates.into_iter()
                        .map(|pair| {
                            let display = pair.display().to_string();
                            let replacement = pair.replacement().to_string();
                            if display.contains(" - ") {
                                let parts: Vec<&str> = display.splitn(2, " - ").collect();
                                (replacement, parts.get(1).unwrap_or(&"").to_string())
                            } else {
                                (replacement, "".to_string())
                            }
                        })
                        .collect())
                }
                Err(e) => Err(anyhow::anyhow!("Completion failed: {}", e))
            }
        } else {
            Ok(vec![])
        }
    }
    
    /// Load configuration from file
    pub fn load_config<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        self.config = LineEditorConfig::load_from_file(path)?;
        self.config_file = Some(path.to_string_lossy().to_string());
        
        // Apply configuration to editor
        self.apply_config()?;
        
        Ok(())
    }
    
    /// Save current configuration to file
    pub fn save_config(&self) -> Result<()> {
        if let Some(ref path) = self.config_file {
            self.config.save_to_file(path)?;
        } else {
            return Err(anyhow::anyhow!("No config file path set"));
        }
        Ok(())
    }
    
    /// Update configuration value
    pub fn set_config(&mut self, key: &str, value: &str) -> Result<()> {
        self.config.update_value(key, value)?;
        self.apply_config()?;
        Ok(())
    }
    
    /// Get configuration value
    pub fn get_config(&self, key: &str) -> Option<String> {
        let configs = self.config.list_all();
        configs.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }
    
    /// List all configuration options
    pub fn list_config(&self) -> Vec<(String, String)> {
        self.config.list_all()
    }
    
    /// Apply current configuration to the editor
    fn apply_config(&mut self) -> Result<()> {
        use rustyline::config::Configurer;
        
        // Apply edit mode
        if self.config.vi_mode {
            self.editor.set_edit_mode(rustyline::EditMode::Vi);
        } else {
            self.editor.set_edit_mode(rustyline::EditMode::Emacs);
        }
        
        // Apply completion type
        let completion_type = match self.config.completion_type.as_str() {
            "Circular" => rustyline::CompletionType::Circular,
            _ => rustyline::CompletionType::List,
        };
        self.editor.set_completion_type(completion_type);
        
        // Apply other settings
        self.editor.set_history_ignore_space(self.config.history_ignore_space);
        self.editor.set_tab_stop(self.config.tab_width.try_into().unwrap_or(4));
        self.editor.set_completion_prompt_limit(self.config.completion_prompt_limit);
        
        Ok(())
    }
}

/// Simplified helper struct for CUI mode
pub struct NexusHelper {
    completer: SimpleCompleter,
    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,
    validator: MatchingBracketValidator,
    colored_prompt: String,
}

impl NexusHelper {
    pub fn new() -> Result<Self> {
        Ok(Self {
            completer: SimpleCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter::new(),
            validator: MatchingBracketValidator::new(),
            colored_prompt: String::new(),
        })
    }

    pub fn add_command(&mut self, command: &str, description: &str) {
        self.completer.add_command(command, description);
    }
}

/// Simple completer for CUI mode - avoids complex dependencies
pub struct SimpleCompleter {
    commands: Vec<(String, String)>, // (command, description)
}

impl SimpleCompleter {
    pub fn new() -> Self {
        let mut commands = Vec::new();
        
        // Add basic shell commands
        commands.extend([
            ("ls".to_string(), "List directory contents".to_string()),
            ("cd".to_string(), "Change directory".to_string()),
            ("pwd".to_string(), "Print working directory".to_string()),
            ("cat".to_string(), "Display file contents".to_string()),
            ("echo".to_string(), "Display text".to_string()),
            ("grep".to_string(), "Search text patterns".to_string()),
            ("find".to_string(), "Find files and directories".to_string()),
            ("cp".to_string(), "Copy files".to_string()),
            ("mv".to_string(), "Move files".to_string()),
            ("rm".to_string(), "Remove files".to_string()),
            ("mkdir".to_string(), "Create directory".to_string()),
            ("rmdir".to_string(), "Remove directory".to_string()),
            ("exit".to_string(), "Exit shell".to_string()),
            ("quit".to_string(), "Quit shell".to_string()),
            ("clear".to_string(), "Clear screen".to_string()),
            ("history".to_string(), "Show command history".to_string()),
            // Add NexusShell builtin commands
            ("help".to_string(), "Show help information".to_string()),
            ("jobs".to_string(), "List active jobs".to_string()),
            ("fg".to_string(), "Bring job to foreground".to_string()),
            ("bg".to_string(), "Put job in background".to_string()),
            ("kill".to_string(), "Terminate processes".to_string()),
            ("which".to_string(), "Locate command".to_string()),
            ("type".to_string(), "Display command type".to_string()),
            ("alias".to_string(), "Create command aliases".to_string()),
            ("unalias".to_string(), "Remove aliases".to_string()),
            ("export".to_string(), "Set environment variables".to_string()),
            ("unset".to_string(), "Remove environment variables".to_string()),
            ("source".to_string(), "Execute script in current context".to_string()),
            ("exec".to_string(), "Replace shell with command".to_string()),
        ]);
        
        Self { commands }
    }
    
    pub fn add_command(&mut self, command: &str, description: &str) {
        self.commands.push((command.to_string(), description.to_string()));
    }
}

impl Completer for NexusHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &RustylineContext<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Completer for SimpleCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &RustylineContext<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (start, word) = extract_word(line, pos);
        let mut matches = Vec::new();
        
        for (cmd, desc) in &self.commands {
            if cmd.starts_with(word) {
                matches.push(Pair {
                    display: format!("{} - {}", cmd, desc),
                    replacement: cmd.clone(),
                });
            }
        }
        
        Ok((start, matches))
    }
}

/// Extract the word being completed from the line
fn extract_word(line: &str, pos: usize) -> (usize, &str) {
    let line_bytes = line.as_bytes();
    let mut start = pos;
    
    // Find start of word
    while start > 0 && !line_bytes[start - 1].is_ascii_whitespace() {
        start -= 1;
    }
    
    (start, &line[start..pos])
}

impl Hinter for NexusHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &RustylineContext<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for NexusHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Cow::Borrowed(prompt)
        } else {
            Cow::Borrowed(&self.colored_prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[90m{}\x1b[0m", hint)) // Gray hint
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // CUI mode: syntax highlighting disabled
        Cow::Borrowed(line)
    }
}

impl Validator for NexusHelper {
    fn validate(
        &self,
        ctx: &mut ValidationContext,
    ) -> rustyline::Result<ValidationResult> {
        // Custom validation logic
        let input = ctx.input();
        
        // Check for unclosed quotes
        let mut _quote_count = 0;
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        
        for ch in input.chars() {
            match ch {
                '\'' if !in_double_quote => {
                    in_single_quote = !in_single_quote;
                    _quote_count += 1;
                }
                '"' if !in_single_quote => {
                    in_double_quote = !in_double_quote;
                    _quote_count += 1;
                }
                _ => {}
            }
        }
        
        if in_single_quote || in_double_quote {
            return Ok(ValidationResult::Incomplete);
        }

        // Check for unclosed brackets
        self.validator.validate(ctx)
    }
}

impl rustyline::Helper for NexusHelper {}

/// Configuration for the line editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineEditorConfig {
    pub vi_mode: bool,
    pub tab_width: usize,
    pub max_history_size: usize,
    pub auto_add_history: bool,
    pub completion_type: String, // "List" | "Circular"
    pub history_ignore_space: bool,
    pub bracket_highlighting: bool,
    pub completion_prompt_limit: usize,
}

impl Default for LineEditorConfig {
    fn default() -> Self {
        Self {
            vi_mode: false,
            tab_width: 4,
            max_history_size: 10000,
            auto_add_history: true,
            completion_type: "List".to_string(),
            history_ignore_space: true,
            bracket_highlighting: true,
            completion_prompt_limit: 100,
        }
    }
}

impl LineEditorConfig {
    /// Load configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            // Create default config file if it doesn't exist
            let default_config = Self::default();
            default_config.save_to_file(path)?;
            return Ok(default_config);
        }
        
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        
        Ok(config)
    }
    
    /// Save configuration to TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;
        
        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        
        Ok(())
    }
    
    /// Update a configuration value by key
    pub fn update_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "vi_mode" => {
                self.vi_mode = value.parse()
                    .with_context(|| format!("Invalid boolean value for vi_mode: {}", value))?;
            }
            "tab_width" => {
                self.tab_width = value.parse()
                    .with_context(|| format!("Invalid number value for tab_width: {}", value))?;
            }
            "max_history_size" => {
                self.max_history_size = value.parse()
                    .with_context(|| format!("Invalid number value for max_history_size: {}", value))?;
            }
            "auto_add_history" => {
                self.auto_add_history = value.parse()
                    .with_context(|| format!("Invalid boolean value for auto_add_history: {}", value))?;
            }
            "completion_type" => {
                if !matches!(value, "List" | "Circular") {
                    return Err(anyhow::anyhow!("Invalid completion_type: {}. Must be 'List' or 'Circular'", value));
                }
                self.completion_type = value.to_string();
            }
            "history_ignore_space" => {
                self.history_ignore_space = value.parse()
                    .with_context(|| format!("Invalid boolean value for history_ignore_space: {}", value))?;
            }
            "bracket_highlighting" => {
                self.bracket_highlighting = value.parse()
                    .with_context(|| format!("Invalid boolean value for bracket_highlighting: {}", value))?;
            }
            "completion_prompt_limit" => {
                self.completion_prompt_limit = value.parse()
                    .with_context(|| format!("Invalid number value for completion_prompt_limit: {}", value))?;
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown configuration key: {}", key));
            }
        }
        Ok(())
    }
    
    /// Get all configuration keys and current values
    pub fn list_all(&self) -> Vec<(String, String)> {
        vec![
            ("vi_mode".to_string(), self.vi_mode.to_string()),
            ("tab_width".to_string(), self.tab_width.to_string()),
            ("max_history_size".to_string(), self.max_history_size.to_string()),
            ("auto_add_history".to_string(), self.auto_add_history.to_string()),
            ("completion_type".to_string(), self.completion_type.clone()),
            ("history_ignore_space".to_string(), self.history_ignore_space.to_string()),
            ("bracket_highlighting".to_string(), self.bracket_highlighting.to_string()),
            ("completion_prompt_limit".to_string(), self.completion_prompt_limit.to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_editor_creation() {
        let editor = NexusLineEditor::new();
        assert!(editor.is_ok());
    }

    #[test]
    fn test_helper_creation() {
        let helper = NexusHelper::new();
        assert!(helper.is_ok());
    }

    #[test]
    fn test_config_default() {
        let config = LineEditorConfig::default();
        assert!(!config.vi_mode);
        assert_eq!(config.tab_width, 4);
        assert_eq!(config.max_history_size, 10000);
    }

    #[test]
    fn test_extract_word() {
        let (start, word) = extract_word("hello world", 5);
        assert_eq!(word, "hello");
        assert_eq!(start, 0);
        
        let (start, word) = extract_word("hello world", 11);
        assert_eq!(word, "world");
        assert_eq!(start, 6);
    }
} 