//! Advanced line editor for NexusShell
//! 
//! This module provides a sophisticated line editing experience with syntax highlighting,
//! intelligent completion, multi-line editing support, and customizable key bindings.

use anyhow::{Result, Context};
use rustyline::{
    completion::{Completer, FilenameCompleter, Pair},
    config::{Builder, ColorMode, Config, EditMode, HistoryDuplicates, OutputStreamType},
    error::ReadlineError,
    highlight::{Highlighter, MatchingBracketHighlighter},
    hint::{Hinter, HistoryHinter},
    history::{DefaultHistory, History},
    validate::{MatchingBracketValidator, ValidationContext, ValidationResult, Validator},
    Cmd, ConditionalEventHandler, Context as RustylineContext, Editor, Event, EventHandler, KeyCode, KeyEvent, Modifiers, RepeatCount,
};
use std::{
    borrow::Cow,
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};
use crate::{
    completion::NexusCompleter,
    highlighting::{RealtimeHighlighter, SyntaxHighlighter},
    themes::NexusTheme,
    config::EditorConfig,
};
use nxsh_core::context::ShellContext;
use nxsh_builtins::Builtin;

/// Advanced line editor with integrated syntax highlighting and completion
pub struct NexusLineEditor {
    editor: Editor<NexusHelper, DefaultHistory>,
    config: EditorConfig,
    theme: NexusTheme,
    highlighter: Arc<Mutex<RealtimeHighlighter>>,
    history_file: Option<String>,
}

impl NexusLineEditor {
    /// Create a new line editor with default configuration
    pub fn new() -> Result<Self> {
        let config = EditorConfig::default();
        let theme = NexusTheme::default();
        
        let rustyline_config = Builder::new()
            .history_ignore_space(true)
            .history_ignore_dups(HistoryDuplicates::IgnoreConsecutive)
            .completion_type(rustyline::CompletionType::List)
            .edit_mode(if config.vi_mode { EditMode::Vi } else { EditMode::Emacs })
            .output_stream(OutputStreamType::Stdout)
            .color_mode(ColorMode::Enabled)
            .tab_stop(config.tab_width)
            .max_history_size(config.max_history_size)
            .build();

        let helper = NexusHelper::new()?;
        let mut editor = Editor::with_config(rustyline_config)?;
        editor.set_helper(Some(helper));

        // Set up key bindings
        Self::setup_key_bindings(&mut editor, &config)?;

        let highlighter = Arc::new(Mutex::new(RealtimeHighlighter::new()?));

        Ok(Self {
            editor,
            config,
            theme,
            highlighter,
            history_file: None,
        })
    }

    /// Create a line editor with custom configuration
    pub fn with_config(config: EditorConfig) -> Result<Self> {
        let mut editor = Self::new()?;
        editor.config = config;
        Ok(editor)
    }

    /// Set up custom key bindings
    fn setup_key_bindings(editor: &mut Editor<NexusHelper, DefaultHistory>, config: &EditorConfig) -> Result<()> {
        // Custom key bindings for enhanced functionality
        editor.bind_sequence(
            Event::KeySeq(vec![KeyEvent::new(KeyCode::Char('x'), Modifiers::CTRL)]),
            EventHandler::Simple(Cmd::Abort),
        );

        // Multi-line editing support
        editor.bind_sequence(
            Event::KeySeq(vec![KeyEvent::new(KeyCode::Enter, Modifiers::ALT)]),
            EventHandler::Simple(Cmd::Newline),
        );

        // Advanced navigation
        editor.bind_sequence(
            Event::KeySeq(vec![KeyEvent::new(KeyCode::Left, Modifiers::CTRL)]),
            EventHandler::Simple(Cmd::BackwardWord),
        );
        editor.bind_sequence(
            Event::KeySeq(vec![KeyEvent::new(KeyCode::Right, Modifiers::CTRL)]),
            EventHandler::Simple(Cmd::ForwardWord),
        );

        // History search
        editor.bind_sequence(
            Event::KeySeq(vec![KeyEvent::new(KeyCode::Char('r'), Modifiers::CTRL)]),
            EventHandler::Simple(Cmd::ReverseSearchHistory),
        );

        // Custom completion trigger
        editor.bind_sequence(
            Event::KeySeq(vec![KeyEvent::new(KeyCode::Tab, Modifiers::empty())]),
            EventHandler::Simple(Cmd::Complete),
        );

        Ok(())
    }

    /// Read a line with full editing capabilities
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

    /// Set the syntax highlighting theme
    pub fn set_theme(&mut self, theme: NexusTheme) -> Result<()> {
        self.theme = theme;
        if let Ok(mut highlighter) = self.highlighter.lock() {
            highlighter.set_theme(&self.theme.syntax_theme)?;
        }
        Ok(())
    }

    /// Get current configuration
    pub fn config(&self) -> &EditorConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: EditorConfig) -> Result<()> {
        self.config = config;
        // Recreate editor with new config
        *self = Self::with_config(self.config.clone())?;
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
        self.editor.clear_history();
    }

    /// Set up completion for shell context
    pub fn setup_shell_completion(&mut self, context: &ShellContext) {
        if let Some(helper) = self.editor.helper_mut() {
            helper.setup_shell_completion(context);
        }
    }
}

/// Helper struct that combines all editing features
pub struct NexusHelper {
    completer: NexusCompleter,
    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,
    validator: MatchingBracketValidator,
    colored_prompt: String,
    syntax_highlighter: Arc<Mutex<RealtimeHighlighter>>,
}

impl NexusHelper {
    pub fn new() -> Result<Self> {
        Ok(Self {
            completer: NexusCompleter::new()?,
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter::new(),
            validator: MatchingBracketValidator::new(),
            colored_prompt: String::new(),
            syntax_highlighter: Arc::new(Mutex::new(RealtimeHighlighter::new()?)),
        })
    }

    pub fn add_command(&mut self, command: &str, description: &str) {
        self.completer.add_command(command, description);
    }

    pub fn setup_shell_completion(&mut self, context: &ShellContext) {
        self.completer.setup_shell_completion(context);
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

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        // Use syntax highlighting
        if let Ok(highlighter) = self.syntax_highlighter.lock() {
            if let Ok(spans) = highlighter.highlight_cached(line) {
                // Convert spans to ANSI escape sequences
                let mut result = String::new();
                for span in spans {
                    // Convert ratatui style to ANSI codes
                    result.push_str(&span.content);
                }
                return Cow::Owned(result);
            }
        }

        // Fallback to bracket highlighting
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
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
        let mut quote_count = 0;
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        
        for ch in input.chars() {
            match ch {
                '\'' if !in_double_quote => {
                    in_single_quote = !in_single_quote;
                    quote_count += 1;
                }
                '"' if !in_single_quote => {
                    in_double_quote = !in_double_quote;
                    quote_count += 1;
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
#[derive(Debug, Clone)]
pub struct LineEditorConfig {
    pub vi_mode: bool,
    pub tab_width: usize,
    pub max_history_size: usize,
    pub auto_add_history: bool,
    pub completion_type: CompletionType,
    pub edit_mode: EditMode,
    pub bell_style: BellStyle,
}

impl Default for LineEditorConfig {
    fn default() -> Self {
        Self {
            vi_mode: false,
            tab_width: 4,
            max_history_size: 10000,
            auto_add_history: true,
            completion_type: CompletionType::List,
            edit_mode: EditMode::Emacs,
            bell_style: BellStyle::None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CompletionType {
    Circular,
    List,
}

#[derive(Debug, Clone)]
pub enum BellStyle {
    None,
    Audible,
    Visible,
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
} 