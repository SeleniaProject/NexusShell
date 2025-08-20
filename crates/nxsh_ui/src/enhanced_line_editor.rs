//! Enhanced Line Editor with Beautiful Tab Completion
//! 
//! This module provides an advanced line editor that integrates:
//! - Beautiful visual completion panel
//! - Enhanced tab navigation
//! - Smart completion engine
//! - Smooth animations and transitions

use anyhow::{Result, Context};
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers, read, poll},
    terminal::{enable_raw_mode, disable_raw_mode, size},
    cursor::{MoveTo, Show, Hide},
    style::{Print, ResetColor},
    execute,
};
use std::{
    io::{self, Write, stdout},
    time::{Duration, Instant},
    sync::{Arc, Mutex},
};

use crate::{
    tab_completion::{TabCompletionHandler, TabCompletionResult},
    completion_panel::CompletionPanel,
};

/// Enhanced line editor with visual completion
pub struct EnhancedLineEditor {
    /// Current input buffer
    input_buffer: String,
    /// Current cursor position in buffer
    cursor_position: usize,
    /// Tab completion handler
    completion_handler: TabCompletionHandler,
    /// Command history
    history: Vec<String>,
    /// Current history index
    history_index: usize,
    /// Editor configuration
    config: EditorConfig,
    /// Input history for smart suggestions
    input_history: Vec<String>,
}

/// Configuration for the enhanced line editor
#[derive(Debug, Clone)]
pub struct EditorConfig {
    /// Enable visual completion panel
    pub enable_visual_completion: bool,
    /// Enable syntax highlighting
    pub enable_syntax_highlighting: bool,
    /// Maximum history size
    pub max_history_size: usize,
    /// Auto-save history
    pub auto_save_history: bool,
    /// History file path
    pub history_file: Option<String>,
    /// Completion delay in milliseconds
    pub completion_delay_ms: u64,
    /// Enable animations
    pub enable_animations: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            enable_visual_completion: true,
            enable_syntax_highlighting: true,
            max_history_size: 1000,
            auto_save_history: true,
            history_file: None,
            completion_delay_ms: 100,
            enable_animations: true,
        }
    }
}

impl EnhancedLineEditor {
    /// Create a new enhanced line editor
    pub fn new() -> Result<Self> {
        let completion_handler = TabCompletionHandler::new()?;
        
        Ok(Self {
            input_buffer: String::new(),
            cursor_position: 0,
            completion_handler,
            history: Vec::new(),
            history_index: 0,
            config: EditorConfig::default(),
            input_history: Vec::new(),
        })
    }

    /// Create a new enhanced line editor with custom configuration
    pub fn with_config(config: EditorConfig) -> Result<Self> {
        let mut editor = Self::new()?;
        editor.config = config;
        Ok(editor)
    }

    /// Read a line with enhanced completion and editing
    pub async fn read_line(&mut self, prompt: &str) -> Result<String> {
        // Initialize display
        self.display_prompt(prompt)?;
        self.display_input()?;

        // Enable raw mode for custom input handling
        enable_raw_mode()?;
        
        let result = self.input_loop().await;
        
        // Disable raw mode
        disable_raw_mode()?;
        
        // Add successful input to history
        if let Ok(ref input) = result {
            if !input.trim().is_empty() {
                self.add_to_history(input.clone());
                self.completion_handler.add_to_history(input.clone());
            }
        }

        result
    }

    /// Main input handling loop
    async fn input_loop(&mut self) -> Result<String> {
        let mut last_key_time = Instant::now();
        
        loop {
            // Update animations if enabled
            if self.config.enable_animations {
                self.completion_handler.update_animation().await?;
            }

            // Check for input with timeout for animation updates
            if poll(Duration::from_millis(16))? {
                match read()? {
                    Event::Key(key_event) => {
                        let result = self.handle_key_event(key_event).await?;
                        
                        match result {
                            InputResult::Continue => {
                                self.refresh_display()?;
                            }
                            InputResult::Submit(line) => {
                                println!(); // Move to next line
                                return Ok(line);
                            }
                            InputResult::Cancel => {
                                println!(); // Move to next line
                                return Ok(String::new());
                            }
                            InputResult::CompletionUpdate => {
                                // Panel was updated, no need to refresh
                            }
                        }
                        
                        last_key_time = Instant::now();
                    }
                    Event::Resize(_, _) => {
                        // Handle terminal resize
                        self.refresh_display()?;
                    }
                    _ => {}
                }
            }

            // Auto-trigger completion after delay
            if self.config.enable_visual_completion {
                let elapsed = last_key_time.elapsed();
                if elapsed.as_millis() >= self.config.completion_delay_ms as u128 {
                    if !self.completion_handler.is_panel_visible() && !self.input_buffer.is_empty() {
                        // Auto-show completion suggestions
                        let _ = self.completion_handler
                            .handle_tab_key(&self.input_buffer, self.cursor_position)
                            .await;
                    }
                }
            }
        }
    }

    /// Handle individual key events
    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<InputResult> {
        // Check if completion panel is handling the key
        if let Some(completion_result) = self.completion_handler
            .handle_key_during_completion(key_event)
            .await? 
        {
            return self.handle_completion_result(completion_result).await;
        }

        // Handle normal editing keys
        match (key_event.code, key_event.modifiers) {
            // Submit input
            (KeyCode::Enter, KeyModifiers::NONE) => {
                Ok(InputResult::Submit(self.input_buffer.clone()))
            }

            // Cancel input
            (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                Ok(InputResult::Cancel)
            }

            // Tab completion
            (KeyCode::Tab, KeyModifiers::NONE) => {
                let result = self.completion_handler
                    .handle_tab_key(&self.input_buffer, self.cursor_position)
                    .await?;
                self.handle_completion_result(result).await
            }

            // Character input
            (KeyCode::Char(ch), KeyModifiers::NONE) | (KeyCode::Char(ch), KeyModifiers::SHIFT) => {
                self.insert_character(ch);
                Ok(InputResult::Continue)
            }

            // Backspace
            (KeyCode::Backspace, _) => {
                self.delete_character();
                Ok(InputResult::Continue)
            }

            // Delete
            (KeyCode::Delete, _) => {
                self.delete_character_forward();
                Ok(InputResult::Continue)
            }

            // Cursor movement
            (KeyCode::Left, _) => {
                self.move_cursor_left();
                Ok(InputResult::Continue)
            }
            (KeyCode::Right, _) => {
                self.move_cursor_right();
                Ok(InputResult::Continue)
            }
            (KeyCode::Home, _) => {
                self.move_cursor_home();
                Ok(InputResult::Continue)
            }
            (KeyCode::End, _) => {
                self.move_cursor_end();
                Ok(InputResult::Continue)
            }

            // History navigation
            (KeyCode::Up, _) => {
                self.history_previous();
                Ok(InputResult::Continue)
            }
            (KeyCode::Down, _) => {
                self.history_next();
                Ok(InputResult::Continue)
            }

            // Word movement
            (KeyCode::Left, KeyModifiers::CONTROL) => {
                self.move_cursor_word_left();
                Ok(InputResult::Continue)
            }
            (KeyCode::Right, KeyModifiers::CONTROL) => {
                self.move_cursor_word_right();
                Ok(InputResult::Continue)
            }

            // Line manipulation
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                self.move_cursor_home();
                Ok(InputResult::Continue)
            }
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                self.move_cursor_end();
                Ok(InputResult::Continue)
            }
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                self.delete_to_end();
                Ok(InputResult::Continue)
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                self.delete_to_beginning();
                Ok(InputResult::Continue)
            }
            (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                self.delete_word_backward();
                Ok(InputResult::Continue)
            }

            // Clear screen
            (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                self.clear_screen()?;
                Ok(InputResult::Continue)
            }

            _ => Ok(InputResult::Continue),
        }
    }

    /// Handle completion results
    async fn handle_completion_result(&mut self, result: TabCompletionResult) -> Result<InputResult> {
        match result {
            TabCompletionResult::SingleCompletion { text, .. } => {
                self.apply_completion(&text);
                Ok(InputResult::Continue)
            }
            TabCompletionResult::PartialCompletion { text, .. } => {
                self.apply_completion(&text);
                Ok(InputResult::Continue)
            }
            TabCompletionResult::CompletionAccepted { text, .. } => {
                self.apply_completion(&text);
                Ok(InputResult::Continue)
            }
            TabCompletionResult::PanelShown { .. } => {
                Ok(InputResult::CompletionUpdate)
            }
            TabCompletionResult::NavigationUpdate => {
                Ok(InputResult::CompletionUpdate)
            }
            TabCompletionResult::Cancelled => {
                Ok(InputResult::Continue)
            }
            _ => Ok(InputResult::Continue),
        }
    }

    /// Apply completion text to input buffer
    fn apply_completion(&mut self, completion_text: &str) {
        // Find the word boundary to replace
        let word_start = self.find_word_start();
        
        // Replace the current word with completion
        self.input_buffer.drain(word_start..self.cursor_position);
        self.input_buffer.insert_str(word_start, completion_text);
        self.cursor_position = word_start + completion_text.len();
    }

    /// Find the start of the current word
    fn find_word_start(&self) -> usize {
        let mut pos = self.cursor_position;
        
        while pos > 0 {
            let ch = self.input_buffer.chars().nth(pos - 1).unwrap_or(' ');
            if ch.is_whitespace() {
                break;
            }
            pos -= 1;
        }
        
        pos
    }

    // Character manipulation methods
    fn insert_character(&mut self, ch: char) {
        self.input_buffer.insert(self.cursor_position, ch);
        self.cursor_position += 1;
    }

    fn delete_character(&mut self) {
        if self.cursor_position > 0 {
            self.input_buffer.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    fn delete_character_forward(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_position);
        }
    }

    // Cursor movement methods
    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
    }

    fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    fn move_cursor_end(&mut self) {
        self.cursor_position = self.input_buffer.len();
    }

    fn move_cursor_word_left(&mut self) {
        while self.cursor_position > 0 {
            self.cursor_position -= 1;
            let ch = self.input_buffer.chars().nth(self.cursor_position).unwrap_or(' ');
            if ch.is_whitespace() {
                break;
            }
        }
    }

    fn move_cursor_word_right(&mut self) {
        while self.cursor_position < self.input_buffer.len() {
            let ch = self.input_buffer.chars().nth(self.cursor_position).unwrap_or(' ');
            self.cursor_position += 1;
            if ch.is_whitespace() {
                break;
            }
        }
    }

    // Line manipulation methods
    fn delete_to_end(&mut self) {
        self.input_buffer.truncate(self.cursor_position);
    }

    fn delete_to_beginning(&mut self) {
        self.input_buffer.drain(0..self.cursor_position);
        self.cursor_position = 0;
    }

    fn delete_word_backward(&mut self) {
        let start_pos = self.cursor_position;
        self.move_cursor_word_left();
        self.input_buffer.drain(self.cursor_position..start_pos);
    }

    // History methods
    fn history_previous(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            if let Some(entry) = self.history.get(self.history_index) {
                self.input_buffer = entry.clone();
                self.cursor_position = self.input_buffer.len();
            }
        }
    }

    fn history_next(&mut self) {
        if self.history_index < self.history.len() {
            self.history_index += 1;
            if self.history_index >= self.history.len() {
                self.input_buffer.clear();
                self.cursor_position = 0;
            } else if let Some(entry) = self.history.get(self.history_index) {
                self.input_buffer = entry.clone();
                self.cursor_position = self.input_buffer.len();
            }
        }
    }

    fn add_to_history(&mut self, input: String) {
        if !input.trim().is_empty() {
            // Remove duplicate if exists
            self.history.retain(|entry| entry != &input);
            
            // Add to end
            self.history.push(input);
            
            // Maintain max size
            if self.history.len() > self.config.max_history_size {
                self.history.remove(0);
            }
            
            // Reset history index
            self.history_index = self.history.len();
        }
    }

    // Display methods
    fn display_prompt(&self, prompt: &str) -> Result<()> {
        print!("{}", prompt);
        io::stdout().flush()?;
        Ok(())
    }

    fn display_input(&self) -> Result<()> {
        // Move to beginning of input line
        let (_, y) = crossterm::cursor::position()?;
        execute!(stdout(), MoveTo(0, y))?;
        
        // Clear line and redraw
        execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine))?;
        
        // Display input with syntax highlighting if enabled
        if self.config.enable_syntax_highlighting {
            self.display_highlighted_input()?;
        } else {
            print!("{}", self.input_buffer);
        }
        
        // Position cursor
        let cursor_x = self.cursor_position as u16;
        execute!(stdout(), MoveTo(cursor_x, y))?;
        
        io::stdout().flush()?;
        Ok(())
    }

    fn display_highlighted_input(&self) -> Result<()> {
        // Basic syntax highlighting implementation
        let mut in_string = false;
        let mut in_command = true;
        
        for (i, ch) in self.input_buffer.chars().enumerate() {
            match ch {
                '"' | '\'' => {
                    in_string = !in_string;
                    if in_string {
                        execute!(stdout(), crossterm::style::SetForegroundColor(crossterm::style::Color::Green))?;
                    } else {
                        execute!(stdout(), ResetColor)?;
                    }
                    print!("{}", ch);
                }
                ' ' => {
                    if !in_string {
                        in_command = false;
                        execute!(stdout(), ResetColor)?;
                    }
                    print!("{}", ch);
                }
                _ => {
                    if in_command && !in_string && i == 0 {
                        execute!(stdout(), crossterm::style::SetForegroundColor(crossterm::style::Color::Blue))?;
                    }
                    print!("{}", ch);
                }
            }
        }
        
        execute!(stdout(), ResetColor)?;
        Ok(())
    }

    fn refresh_display(&self) -> Result<()> {
        // Clear current line and redraw
        let (_, y) = crossterm::cursor::position()?;
        execute!(stdout(), MoveTo(0, y))?;
        execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine))?;
        
        self.display_input()?;
        Ok(())
    }

    fn clear_screen(&self) -> Result<()> {
        execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        execute!(stdout(), MoveTo(0, 0))?;
        Ok(())
    }

    /// Get current input buffer
    pub fn current_buffer(&self) -> &str {
        &self.input_buffer
    }

    /// Get cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Check if completion panel is visible
    pub fn is_completion_visible(&self) -> bool {
        self.completion_handler.is_panel_visible()
    }

    /// Get completion metrics
    pub fn completion_metrics(&self) -> &crate::tab_completion::CompletionMetrics {
        self.completion_handler.get_metrics()
    }
}

/// Result of input processing
#[derive(Debug, Clone)]
enum InputResult {
    /// Continue input processing
    Continue,
    /// Submit the input line
    Submit(String),
    /// Cancel input
    Cancel,
    /// Completion panel was updated
    CompletionUpdate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_editor_creation() {
        let editor = EnhancedLineEditor::new().unwrap();
        assert_eq!(editor.current_buffer(), "");
        assert_eq!(editor.cursor_position(), 0);
    }

    #[test]
    fn test_character_insertion() {
        let mut editor = EnhancedLineEditor::new().unwrap();
        editor.insert_character('h');
        editor.insert_character('i');
        assert_eq!(editor.current_buffer(), "hi");
        assert_eq!(editor.cursor_position(), 2);
    }

    #[test]
    fn test_cursor_movement() {
        let mut editor = EnhancedLineEditor::new().unwrap();
        editor.insert_character('h');
        editor.insert_character('e');
        editor.insert_character('l');
        editor.insert_character('l');
        editor.insert_character('o');
        
        editor.move_cursor_left();
        editor.move_cursor_left();
        assert_eq!(editor.cursor_position(), 3);
        
        editor.move_cursor_home();
        assert_eq!(editor.cursor_position(), 0);
        
        editor.move_cursor_end();
        assert_eq!(editor.cursor_position(), 5);
    }

    #[test]
    fn test_history_management() {
        let mut editor = EnhancedLineEditor::new().unwrap();
        
        editor.add_to_history("command1".to_string());
        editor.add_to_history("command2".to_string());
        
        assert_eq!(editor.history.len(), 2);
        assert_eq!(editor.history_index, 2);
        
        editor.history_previous();
        assert_eq!(editor.history_index, 1);
        
        editor.history_previous();
        assert_eq!(editor.history_index, 0);
    }

    #[test]
    fn test_word_boundary_finding() {
        let mut editor = EnhancedLineEditor::new().unwrap();
        editor.input_buffer = "git commit -m".to_string();
        editor.cursor_position = 13; // End of string
        
        let word_start = editor.find_word_start();
        assert_eq!(word_start, 11); // Start of "-m"
    }
}
