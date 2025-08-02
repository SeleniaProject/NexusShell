//! Advanced syntax highlighting system for NexusShell
//! 
//! This module provides real-time syntax highlighting for shell commands,
//! scripts, and various programming languages using pure Rust regex-based highlighting.

use anyhow::{Result, Context};
use regex::Regex;
use ratatui::{
    prelude::*,
    text::{Line, Span},
};
use std::collections::HashMap;

/// Main syntax highlighting engine
pub struct SyntaxHighlighter {
    shell_patterns: Vec<(Regex, Style)>,
    keyword_style: Style,
    string_style: Style,
    comment_style: Style,
    variable_style: Style,
    command_style: Style,
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter with default settings
    pub fn new() -> Result<Self> {
        let shell_patterns = vec![
            // Shell keywords
            (Regex::new(r"\b(if|then|else|elif|fi|for|while|do|done|case|esac|function|return|exit|break|continue|echo|cd|ls|pwd|mkdir|rm|cp|mv|grep|find|cat|head|tail|sort|uniq|cut|awk|sed)\b")?, 
             Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            // Strings
            (Regex::new(r#""[^"]*""#)?, 
             Style::default().fg(Color::Green)),
            (Regex::new(r"'[^']*'")?, 
             Style::default().fg(Color::Green)),
            // Comments
            (Regex::new(r"#.*$")?, 
             Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
            // Variables
            (Regex::new(r"\$\w+")?, 
             Style::default().fg(Color::Cyan)),
            (Regex::new(r"\$\{[^}]+\}")?, 
             Style::default().fg(Color::Cyan)),
        ];

        Ok(Self {
            shell_patterns,
            keyword_style: Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
            string_style: Style::default().fg(Color::Green),
            comment_style: Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
            variable_style: Style::default().fg(Color::Cyan),
            command_style: Style::default().fg(Color::Yellow),
        })
    }

    /// Highlight a line of shell code
    pub fn highlight_line(&self, line: &str) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let mut last_end = 0;

        // Apply patterns in order
        for (pattern, style) in &self.shell_patterns {
            for mat in pattern.find_iter(line) {
                // Add text before match with default style
                if mat.start() > last_end {
                    spans.push(Span::styled(
                        line[last_end..mat.start()].to_string(),
                        Style::default()
                    ));
                }
                
                // Add matched text with highlighting
                spans.push(Span::styled(
                    mat.as_str().to_string(),
                    *style
                ));
                
                last_end = mat.end();
            }
        }

        // Add remaining text
        if last_end < line.len() {
            spans.push(Span::styled(
                line[last_end..].to_string(),
                Style::default()
            ));
        }

        // If no highlighting was applied, return the whole line with default style
        if spans.is_empty() {
            spans.push(Span::styled(line.to_string(), Style::default()));
        }

        spans
    }

    /// Set highlighting theme (placeholder for future theme support)
    pub fn set_theme(&mut self, _theme_name: &str) -> Result<()> {
        // TODO: Implement theme changing
        Ok(())
    }

    /// Get available languages for highlighting
    pub fn get_available_languages(&self) -> Vec<String> {
        vec!["shell".to_string(), "bash".to_string()]
    }
}

/// Real-time syntax highlighter for interactive editing
pub struct RealtimeHighlighter {
    highlighter: SyntaxHighlighter,
    cached_lines: HashMap<String, Vec<Span<'static>>>,
}

impl RealtimeHighlighter {
    /// Create a new realtime highlighter
    pub fn new() -> Result<Self> {
        Ok(Self {
            highlighter: SyntaxHighlighter::new()?,
            cached_lines: HashMap::new(),
        })
    }

    /// Highlight text with caching for performance
    pub fn highlight_cached(&mut self, text: &str) -> Result<Vec<Span<'static>>> {
        if let Some(cached) = self.cached_lines.get(text) {
            return Ok(cached.clone());
        }

        let spans = self.highlighter.highlight_line(text);
        self.cached_lines.insert(text.to_string(), spans.clone());
        
        Ok(spans)
    }

    /// Clear highlighting cache
    pub fn clear_cache(&mut self) {
        self.cached_lines.clear();
    }

    /// Set highlighting theme
    pub fn set_theme(&mut self, theme_name: &str) -> Result<()> {
        self.highlighter.set_theme(theme_name)?;
        self.clear_cache(); // Clear cache when theme changes
        Ok(())
    }
}

impl Default for RealtimeHighlighter {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            highlighter: SyntaxHighlighter {
                shell_patterns: Vec::new(),
                keyword_style: Style::default(),
                string_style: Style::default(),
                comment_style: Style::default(),
                variable_style: Style::default(),
                command_style: Style::default(),
            },
            cached_lines: HashMap::new(),
        })
    }
}
