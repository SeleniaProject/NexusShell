//! Advanced syntax highlighting system for NexusShell
//! 
//! This module provides real-time syntax highlighting for shell commands,
//! scripts, and various programming languages using the syntect library.

use anyhow::{Result, Context};
use syntect::{
    highlighting::{Color, FontStyle, Style, Theme, ThemeSet},
    parsing::{SyntaxSet, SyntaxReference},
    util::LinesWithEndings,
};
use ratatui::{
    prelude::*,
    text::{Line, Span},
};
use std::collections::HashMap;
use crate::themes::NexusTheme;

/// Main syntax highlighting engine
pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_theme: String,
    shell_syntax: Option<SyntaxReference>,
    language_cache: HashMap<String, SyntaxReference>,
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter with default settings
    pub fn new() -> Result<Self> {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        
        // Find shell syntax
        let shell_syntax = syntax_set.find_syntax_by_extension("sh").cloned();
        
        Ok(Self {
            syntax_set,
            theme_set,
            current_theme: "base16-ocean.dark".to_string(),
            shell_syntax,
            language_cache: HashMap::new(),
        })
    }

    /// Set the current theme
    pub fn set_theme(&mut self, theme_name: &str) -> Result<()> {
        if self.theme_set.themes.contains_key(theme_name) {
            self.current_theme = theme_name.to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Theme '{}' not found", theme_name))
        }
    }

    /// Get available themes
    pub fn available_themes(&self) -> Vec<&String> {
        self.theme_set.themes.keys().collect()
    }

    /// Highlight shell command text and return ratatui spans
    pub fn highlight_shell_command(&self, text: &str) -> Result<Vec<Span>> {
        let theme = self.theme_set.themes.get(&self.current_theme)
            .context("Failed to get current theme")?;
        
        let syntax = self.shell_syntax.as_ref()
            .or_else(|| self.syntax_set.find_syntax_plain_text())
            .context("No suitable syntax found")?;

        self.highlight_text(text, syntax, theme)
    }

    /// Highlight text for a specific language
    pub fn highlight_language(&mut self, text: &str, language: &str) -> Result<Vec<Span>> {
        let theme = self.theme_set.themes.get(&self.current_theme)
            .context("Failed to get current theme")?;

        // Try to find syntax by language name or extension
        let syntax = if let Some(cached) = self.language_cache.get(language) {
            cached
        } else {
            let found_syntax = self.syntax_set.find_syntax_by_name(language)
                .or_else(|| self.syntax_set.find_syntax_by_extension(language))
                .or_else(|| self.syntax_set.find_syntax_by_first_line(text))
                .context(format!("Syntax not found for language: {}", language))?;
            
            self.language_cache.insert(language.to_string(), found_syntax.clone());
            found_syntax
        };

        self.highlight_text(text, syntax, theme)
    }

    /// Core highlighting logic
    fn highlight_text(&self, text: &str, syntax: &SyntaxReference, theme: &Theme) -> Result<Vec<Span>> {
        use syntect::easy::HighlightLines;
        
        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut spans = Vec::new();

        for line in LinesWithEndings::from(text) {
            let ranges = highlighter.highlight_line(line, &self.syntax_set)
                .context("Failed to highlight line")?;
            
            for (style, text) in ranges {
                let span = self.style_to_span(style, text);
                spans.push(span);
            }
        }

        Ok(spans)
    }

    /// Convert syntect style to ratatui span
    fn style_to_span(&self, style: Style, text: &str) -> Span {
        let mut ratatui_style = ratatui::prelude::Style::default();

        // Convert color
        if let Some(fg) = self.syntect_color_to_ratatui(style.foreground) {
            ratatui_style = ratatui_style.fg(fg);
        }

        // Convert font style
        if style.font_style.contains(FontStyle::BOLD) {
            ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
        }
        if style.font_style.contains(FontStyle::ITALIC) {
            ratatui_style = ratatui_style.add_modifier(Modifier::ITALIC);
        }
        if style.font_style.contains(FontStyle::UNDERLINE) {
            ratatui_style = ratatui_style.add_modifier(Modifier::UNDERLINED);
        }

        Span::styled(text.to_string(), ratatui_style)
    }

    /// Convert syntect color to ratatui color
    fn syntect_color_to_ratatui(&self, color: Color) -> Option<ratatui::prelude::Color> {
        Some(ratatui::prelude::Color::Rgb(color.r, color.g, color.b))
    }

    /// Highlight command with context-aware parsing
    pub fn highlight_command_with_context(&self, command: &str, cursor_pos: usize) -> Result<HighlightedCommand> {
        let spans = self.highlight_shell_command(command)?;
        
        // Parse command structure for enhanced highlighting
        let parts = self.parse_command_structure(command);
        
        Ok(HighlightedCommand {
            spans,
            parts,
            cursor_pos,
            suggestions: self.generate_suggestions(command, cursor_pos)?,
        })
    }

    /// Parse command structure for better highlighting
    fn parse_command_structure(&self, command: &str) -> CommandParts {
        let mut parts = CommandParts::default();
        let tokens: Vec<&str> = command.split_whitespace().collect();
        
        if let Some(first) = tokens.first() {
            parts.command = first.to_string();
            parts.args = tokens[1..].iter().map(|s| s.to_string()).collect();
        }

        // Detect pipes, redirections, etc.
        if command.contains('|') {
            parts.has_pipe = true;
        }
        if command.contains('>') || command.contains('<') {
            parts.has_redirection = true;
        }
        if command.contains('&') {
            parts.has_background = true;
        }

        parts
    }

    /// Generate context-aware suggestions
    fn generate_suggestions(&self, command: &str, cursor_pos: usize) -> Result<Vec<String>> {
        let mut suggestions = Vec::new();
        
        // Basic command suggestions based on cursor position
        if cursor_pos == 0 || command[..cursor_pos].trim().is_empty() {
            // Suggest commands
            suggestions.extend(vec![
                "ls".to_string(),
                "cd".to_string(),
                "grep".to_string(),
                "find".to_string(),
                "cat".to_string(),
                "echo".to_string(),
                "pwd".to_string(),
            ]);
        } else {
            // Suggest file/directory names, flags, etc.
            // This would integrate with the completion system
        }

        Ok(suggestions)
    }
}

/// Highlighted command with metadata
#[derive(Debug, Clone)]
pub struct HighlightedCommand {
    pub spans: Vec<Span<'static>>,
    pub parts: CommandParts,
    pub cursor_pos: usize,
    pub suggestions: Vec<String>,
}

/// Parsed command structure
#[derive(Debug, Clone, Default)]
pub struct CommandParts {
    pub command: String,
    pub args: Vec<String>,
    pub has_pipe: bool,
    pub has_redirection: bool,
    pub has_background: bool,
}

/// Real-time highlighting manager
pub struct RealtimeHighlighter {
    highlighter: SyntaxHighlighter,
    cache: HashMap<String, Vec<Span<'static>>>,
    max_cache_size: usize,
}

impl RealtimeHighlighter {
    pub fn new() -> Result<Self> {
        Ok(Self {
            highlighter: SyntaxHighlighter::new()?,
            cache: HashMap::new(),
            max_cache_size: 1000,
        })
    }

    /// Highlight with caching for performance
    pub fn highlight_cached(&mut self, text: &str) -> Result<Vec<Span<'static>>> {
        if let Some(cached) = self.cache.get(text) {
            return Ok(cached.clone());
        }

        let spans = self.highlighter.highlight_shell_command(text)?;
        
        // Convert to owned spans for caching
        let owned_spans: Vec<Span<'static>> = spans.into_iter()
            .map(|span| Span::styled(span.content.to_string(), span.style))
            .collect();

        // Manage cache size
        if self.cache.len() >= self.max_cache_size {
            self.cache.clear();
        }

        self.cache.insert(text.to_string(), owned_spans.clone());
        Ok(owned_spans)
    }

    /// Set theme for the highlighter
    pub fn set_theme(&mut self, theme: &str) -> Result<()> {
        self.highlighter.set_theme(theme)?;
        self.cache.clear(); // Clear cache when theme changes
        Ok(())
    }

    /// Get available themes
    pub fn available_themes(&self) -> Vec<&String> {
        self.highlighter.available_themes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_highlighter_creation() {
        let highlighter = SyntaxHighlighter::new();
        assert!(highlighter.is_ok());
    }

    #[test]
    fn test_shell_command_highlighting() {
        let mut highlighter = SyntaxHighlighter::new().unwrap();
        let result = highlighter.highlight_shell_command("ls -la | grep test");
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_theme_switching() {
        let mut highlighter = SyntaxHighlighter::new().unwrap();
        let themes = highlighter.available_themes();
        assert!(!themes.is_empty());
        
        if let Some(theme) = themes.first() {
            assert!(highlighter.set_theme(theme).is_ok());
        }
    }

    #[test]
    fn test_realtime_highlighting_cache() {
        let mut rt_highlighter = RealtimeHighlighter::new().unwrap();
        let text = "echo hello world";
        
        // First call should compute
        let result1 = rt_highlighter.highlight_cached(text).unwrap();
        
        // Second call should use cache
        let result2 = rt_highlighter.highlight_cached(text).unwrap();
        
        assert_eq!(result1.len(), result2.len());
    }
} 