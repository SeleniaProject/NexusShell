//! NexusShell UI Library - Character User Interface Implementation
//!
//! This library provides the user interface components for NexusShell,
//! focusing on CUI (Character User Interface) rather than TUI (Terminal User Interface)
//! for improved performance, reduced complexity, and better POSIX compatibility.

// Allow some dead-code in UI crate as many parts are platform/feature gated or WIP
#![allow(dead_code)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::blocks_in_conditions)]

// Re-export commonly used types and functions
pub use config::UiConfig;
pub use themes::{NexusTheme as Theme, get_theme_by_name as get_theme};
pub use completion::{CompletionType, CompletionResult, NexusCompleter};
pub use prompt::{PromptRenderer, PromptStyle, PromptConfig};
pub use input_handler::{InputHandler, KeyEvent, InputAction, InputMode};

use std::io::{self, Write};
use crossterm::{
    terminal::{self, ClearType},
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    execute,
    event::{self, Event, KeyCode, KeyModifiers},
};

// Core modules for binary dependencies
pub mod config;
pub mod themes;
pub mod theme_validator;
pub mod ansi_render;
pub mod completion;
pub mod readline;
pub mod history;
pub mod prompt;
pub mod input_handler;
pub mod ui_ux;
pub mod enhanced_line_editor;
pub mod tab_completion;
pub mod completion_panel;
pub mod completion_engine;

/// Animation manager for UI elements
pub struct Animation {
    name: String,
    frames: Vec<String>,
    current_frame: usize,
}

impl Animation {
    /// Create a new animation
    pub fn new(name: String, frames: Vec<String>) -> Self {
        Self {
            name,
            frames,
            current_frame: 0,
        }
    }

    /// Create a spinner animation
    pub fn spinner() -> Self {
        Self::new(
            "spinner".to_string(),
            vec!["|".to_string(), "/".to_string(), "-".to_string(), "\\".to_string()],
        )
    }

    /// Get the next frame
    pub fn next_frame(&mut self) -> &str {
        let frame = &self.frames[self.current_frame];
        self.current_frame = (self.current_frame + 1) % self.frames.len();
        frame
    }
}

/// Border styles for table formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    /// Simple ASCII borders
    Simple,
    /// Rounded Unicode borders
    Rounded,
    /// Heavy Unicode borders
    Heavy,
    /// Double-line borders
    Double,
    /// No borders
    None,
}

/// Table formatting options
#[derive(Debug, Clone)]
pub struct TableOptions {
    pub border_style: BorderStyle,
    pub show_header: bool,
    pub alternating_rows: bool,
    pub header_alignment: Alignment,
    pub max_width: Option<usize>,
    pub show_borders: bool,
    pub zebra_striping: bool,
    pub compact_mode: bool,
    pub align_columns: bool,
    pub compact: bool,
}

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            border_style: BorderStyle::Simple,
            show_header: true,
            alternating_rows: false,
            header_alignment: Alignment::Left,
            max_width: None,
            show_borders: true,
            zebra_striping: false,
            compact_mode: false,
            align_columns: true,
            compact: false,
        }
    }
}

/// Progress bar for long-running operations
pub struct ProgressBar {
    current: u64,
    total: u64,
    width: usize,
    message: String,
}

/// Dummy progress bar type alias for compatibility
pub type DummyProgressBar = ProgressBar;

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(total: u64) -> Self {
        Self {
            current: 0,
            total,
            width: 50,
            message: String::new(),
        }
    }

    /// Set the current progress
    pub fn set_position(&mut self, pos: u64) {
        self.current = pos.min(self.total);
    }

    /// Set the message
    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }

    /// Render the progress bar
    pub fn render(&self) -> String {
        let percentage = if self.total > 0 {
            (self.current as f64 / self.total as f64 * 100.0) as u8
        } else {
            100
        };

        let filled = (self.current as f64 / self.total as f64 * self.width as f64) as usize;
        let empty = self.width - filled;

        format!(
            "{} [{}{}] {}%",
            self.message,
            "=".repeat(filled),
            " ".repeat(empty),
            percentage
        )
    }
}

/// Notification types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

/// Notification structure
pub struct Notification {
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
}

impl Notification {
    /// Create a new notification
    pub fn new(notification_type: NotificationType, title: String, message: String) -> Self {
        Self {
            notification_type,
            title,
            message,
        }
    }

    /// Create an info notification
    pub fn info(title: String, message: String) -> Self {
        Self::new(NotificationType::Info, title, message)
    }

    /// Create a success notification
    pub fn success(title: String, message: String) -> Self {
        Self::new(NotificationType::Success, title, message)
    }

    /// Create a warning notification
    pub fn warning(title: String, message: String) -> Self {
        Self::new(NotificationType::Warning, title, message)
    }

    /// Create an error notification
    pub fn error(title: String, message: String) -> Self {
        Self::new(NotificationType::Error, title, message)
    }
}

/// Advanced CUI controller with interactive features
pub struct AdvancedCuiController {
    theme: Theme,
    prompt: PromptRenderer,
    is_running: bool,
    history: Vec<String>,
}

impl AdvancedCuiController {
    /// Create a new advanced CUI controller
    pub fn new() -> anyhow::Result<Self> {
        let theme = get_theme("nxsh-dark-default")?;
        let prompt = PromptRenderer::new(PromptConfig::default());
        
        Ok(Self {
            theme,
            prompt,
            is_running: false,
            history: Vec::new(),
        })
    }
    
    /// Start the interactive CUI session
    pub fn run_interactive(&mut self) -> anyhow::Result<()> {
        self.is_running = true;
        
        // Enable raw mode for advanced input handling
        terminal::enable_raw_mode()?;
        
        while self.is_running {
            // Render prompt
            let prompt_text = self.prompt.render();
            print!("{}", prompt_text);
            io::stdout().flush()?;
            
            // Read events
            if event::poll(std::time::Duration::from_millis(500))? {
                if let Event::Key(key_event) = event::read()? {
                    match key_event.code {
                        KeyCode::Enter => {
                            println!();
                            // Process command here
                        }
                        KeyCode::Char('q') if key_event.modifiers == KeyModifiers::CONTROL => {
                            self.is_running = false;
                        }
                        KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => {
                            self.is_running = false;
                        }
                        KeyCode::Char('l') if key_event.modifiers == KeyModifiers::CONTROL => {
                            execute!(io::stdout(), terminal::Clear(ClearType::All))?;
                            execute!(io::stdout(), cursor::MoveTo(0, 0))?;
                        }
                        KeyCode::Char(c) => {
                            print!("{}", c);
                            io::stdout().flush()?;
                        }
                        KeyCode::Backspace => {
                            execute!(io::stdout(), cursor::MoveLeft(1))?;
                            execute!(io::stdout(), Print(" "))?;
                            execute!(io::stdout(), cursor::MoveLeft(1))?;
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Disable raw mode
        terminal::disable_raw_mode()?;
        Ok(())
    }
    
    /// Execute a command
    fn execute_command(&mut self, command: &str) -> anyhow::Result<()> {
        match command.trim() {
            "exit" | "quit" => {
                self.is_running = false;
            }
            "clear" => {
                execute!(io::stdout(), terminal::Clear(ClearType::All))?;
                execute!(io::stdout(), cursor::MoveTo(0, 0))?;
            }
            "help" => {
                self.show_help()?;
            }
            cmd => {
                println!("Command executed: {}", cmd);
            }
        }
        Ok(())
    }
    
    /// Show help information
    fn show_help(&self) -> anyhow::Result<()> {
        execute!(io::stdout(), SetForegroundColor(Color::Green))?;
        println!("NexusShell Advanced CUI - Available Commands:");
        execute!(io::stdout(), ResetColor)?;
        
        let help_items = [
            ("Tab", "Auto-completion"),
            ("Ctrl+R", "Reverse history search"),
            ("Ctrl+L", "Clear screen"),
            ("Ctrl+C", "Interrupt/Exit"),
            ("Ctrl+A", "Move to beginning of line"),
            ("Ctrl+E", "Move to end of line"),
            ("clear", "Clear screen"),
            ("help", "Show this help"),
            ("exit/quit", "Exit NexusShell"),
        ];
        
        for (key, desc) in &help_items {
            execute!(io::stdout(), SetForegroundColor(Color::Yellow))?;
            print!("  {:12}", key);
            execute!(io::stdout(), SetForegroundColor(Color::DarkGrey))?;
            println!(" - {}", desc);
            execute!(io::stdout(), ResetColor)?;
        }
        
        Ok(())
    }
    
    /// Get the current theme
    pub fn theme(&self) -> &Theme {
        &self.theme
    }
    
    /// Set a new theme
    pub fn set_theme(&mut self, theme_name: &str) -> anyhow::Result<()> {
        self.theme = get_theme(theme_name)?;
        Ok(())
    }
    
    /// Get completion suggestions for input (simplified version)
    pub fn get_completions(&self, input: &str, _pos: usize) -> Vec<String> {
        // Simple completion example
        let commands = vec![
            "ls", "cd", "pwd", "mkdir", "rmdir", "cp", "mv", "rm",
            "cat", "grep", "find", "git", "cargo", "help", "exit", "clear"
        ];
        
        commands.into_iter()
            .filter(|cmd| cmd.starts_with(input))
            .map(|cmd| cmd.to_string())
            .collect()
    }
    
    /// Add command to history with deduplication and size management
    pub fn add_to_history(&mut self, command: String) {
        // Skip empty commands and duplicates of the last command
        if command.trim().is_empty() {
            return;
        }
        
        if let Some(last) = self.history.last() {
            if last == &command {
                return; // Don't add duplicate consecutive commands
            }
        }
        
        // Add the command
        self.history.push(command);
        
        // Maintain history size limit (default 10000)
        const MAX_HISTORY_SIZE: usize = 10000;
        if self.history.len() > MAX_HISTORY_SIZE {
            self.history.drain(0..self.history.len() - MAX_HISTORY_SIZE);
        }
    }
    
    /// Search history with fuzzy matching and relevance scoring
    pub fn search_history(&self, query: &str) -> Vec<&str> {
        if query.is_empty() {
            return self.history.iter().rev().take(20).map(|s| s.as_str()).collect();
        }
        
        let query_lower = query.to_lowercase();
        let mut matches: Vec<(usize, &str)> = Vec::new();
        
        for (index, cmd) in self.history.iter().enumerate().rev() {
            let cmd_lower = cmd.to_lowercase();
            
            // Calculate relevance score
            let score = if cmd_lower.starts_with(&query_lower) {
                1000 + index // Exact prefix match gets highest score
            } else if cmd_lower.contains(&query_lower) {
                500 + index // Substring match gets medium score  
            } else if self.fuzzy_match(&cmd_lower, &query_lower) {
                100 + index // Fuzzy match gets low score
            } else {
                continue;
            };
            
            matches.push((score, cmd.as_str()));
        }
        
        // Sort by score descending and take top 20 results
        matches.sort_by(|a, b| b.0.cmp(&a.0));
        matches.into_iter().take(20).map(|(_, cmd)| cmd).collect()
    }
    
    /// Simple fuzzy matching algorithm
    fn fuzzy_match(&self, text: &str, pattern: &str) -> bool {
        let mut text_chars = text.chars();
        let pattern_chars = pattern.chars();
        
        for pattern_char in pattern_chars {
            let mut found = false;
            for text_char in text_chars.by_ref() {
                if text_char == pattern_char {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        true
    }
}

/// Simple UI controller for basic testing
pub struct SimpleUiController {
    theme: Theme,
}

impl SimpleUiController {
    /// Create a new simple UI controller
    pub fn new() -> anyhow::Result<Self> {
        let theme = get_theme("nxsh-dark-default")?;
        Ok(Self { theme })
    }

    /// Get the current theme
    pub fn theme(&self) -> &Theme {
        &self.theme
    }
}

impl Default for SimpleUiController {
    fn default() -> Self {
        Self::new().expect("Failed to create simple UI controller")
    }
}
