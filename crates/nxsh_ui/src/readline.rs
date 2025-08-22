//! Enhanced ReadLine implementation for NexusShell CUI
//! Provides rich line editing with tab completion, history, and syntax highlighting

use std::io::{self, Write, stdout};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent as CrosstermKeyEvent, KeyModifiers},
    terminal::{self, enable_raw_mode, disable_raw_mode},
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    ExecutableCommand,
};
use crate::completion::{CompletionResult, NexusCompleter};
use crate::history::History;
use crate::prompt::PromptRenderer;

/// Key event wrapper
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl From<CrosstermKeyEvent> for KeyEvent {
    fn from(event: CrosstermKeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

/// ReadLine configuration
#[derive(Debug, Clone)]
pub struct ReadLineConfig {
    pub enable_history: bool,
    pub enable_completion: bool,
    pub enable_syntax_highlighting: bool,
    pub history_size: usize,
    pub completion_max_items: usize,
    pub auto_completion: bool,
    pub vi_mode: bool,
}

impl Default for ReadLineConfig {
    fn default() -> Self {
        Self {
            enable_history: true,
            enable_completion: true,
            enable_syntax_highlighting: true,
            history_size: 1000,
            completion_max_items: 50,
            auto_completion: false,
            vi_mode: false,
        }
    }
}

/// Enhanced ReadLine implementation
pub struct ReadLine {
    config: ReadLineConfig,
    completion_engine: NexusCompleter,
    history: History,
    prompt_renderer: PromptRenderer,
    
    // Current line state
    line: String,
    cursor_pos: usize,
    
    // Display state
    prompt: String,
    screen_width: u16,
    
    // Completion state
    completions: Vec<CompletionResult>,
    completion_index: Option<usize>,
    completion_prefix: String,
    
    // History navigation
    history_index: Option<usize>,
    history_search: Option<String>,
}

impl ReadLine {
    pub fn new() -> io::Result<Self> {
        Self::with_config(ReadLineConfig::default())
    }
    
    pub fn with_config(config: ReadLineConfig) -> io::Result<Self> {
        let (width, _) = terminal::size()?;
        
        Ok(Self {
            config,
            completion_engine: NexusCompleter::new(),
            history: History::new(),
            prompt_renderer: PromptRenderer::default(),
            line: String::new(),
            cursor_pos: 0,
            prompt: String::new(),
            screen_width: width,
            completions: Vec::new(),
            completion_index: None,
            completion_prefix: String::new(),
            history_index: None,
            history_search: None,
        })
    }
    
    /// Read a line of input with full editing capabilities
    pub fn read_line(&mut self, prompt: &str) -> io::Result<String> {
        self.prompt = prompt.to_string();
        self.line.clear();
        self.cursor_pos = 0;
        self.clear_completion_state();
        self.history_index = None;
        
        enable_raw_mode()?;
        
        // Display initial prompt
        self.display_prompt()?;
        
        loop {
            match event::read()? {
                Event::Key(key) => {
                    let key_event = KeyEvent::from(key);
                    
                    if let Some(result) = self.handle_key(key_event)? {
                        disable_raw_mode()?;
                        stdout().execute(Print("\n"))?;
                        
                        if !result.trim().is_empty() && self.config.enable_history {
                            self.history.add_entry(result.clone());
                        }
                        
                        return Ok(result);
                    }
                    
                    self.refresh_display()?;
                }
                Event::Resize(width, _) => {
                    self.screen_width = width;
                    self.refresh_display()?;
                }
                _ => {}
            }
        }
    }
    
    fn handle_key(&mut self, key: KeyEvent) -> io::Result<Option<String>> {
        match key.code {
            KeyCode::Enter => {
                return Ok(Some(self.line.clone()));
            }
            
            KeyCode::Esc => {
                self.clear_completion_state();
            }
            
            KeyCode::Tab => {
                if self.config.enable_completion {
                    self.handle_tab_completion()?;
                }
            }
            
            KeyCode::BackTab => {
                if self.config.enable_completion && self.completion_index.is_some() {
                    self.previous_completion();
                }
            }
            
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.line.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                    self.clear_completion_state();
                }
            }
            
            KeyCode::Delete => {
                if self.cursor_pos < self.line.len() {
                    self.line.remove(self.cursor_pos);
                    self.clear_completion_state();
                }
            }
            
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.clear_completion_state();
                }
            }
            
            KeyCode::Right => {
                if self.cursor_pos < self.line.len() {
                    self.cursor_pos += 1;
                    self.clear_completion_state();
                }
            }
            
            KeyCode::Up => {
                if self.config.enable_history {
                    self.history_previous();
                }
            }
            
            KeyCode::Down => {
                if self.config.enable_history {
                    self.history_next();
                }
            }
            
            KeyCode::Home => {
                self.cursor_pos = 0;
                self.clear_completion_state();
            }
            
            KeyCode::End => {
                self.cursor_pos = self.line.len();
                self.clear_completion_state();
            }
            
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'c' => {
                            return Ok(Some(String::new()));
                        }
                        'd' => {
                            if self.line.is_empty() {
                                return Ok(Some(String::new()));
                            }
                        }
                        'a' => {
                            self.cursor_pos = 0;
                        }
                        'e' => {
                            self.cursor_pos = self.line.len();
                        }
                        'k' => {
                            self.line.truncate(self.cursor_pos);
                        }
                        'u' => {
                            self.line.drain(0..self.cursor_pos);
                            self.cursor_pos = 0;
                        }
                        'w' => {
                            self.delete_word_backward();
                        }
                        'l' => {
                            stdout().execute(terminal::Clear(terminal::ClearType::All))?;
                            stdout().execute(cursor::MoveTo(0, 0))?;
                        }
                        'r' => {
                            if self.config.enable_history {
                                self.history_search_backward()?;
                            }
                        }
                        _ => {}
                    }
                } else {
                    self.line.insert(self.cursor_pos, c);
                    self.cursor_pos += 1;
                    self.clear_completion_state();
                }
            }
            
            _ => {}
        }
        
        Ok(None)
    }
    
    fn handle_tab_completion(&mut self) -> io::Result<()> {
        if self.completions.is_empty() {
            // Start new completion
            let completions = self.completion_engine.complete(&self.line, self.cursor_pos);
            
            if completions.is_empty() {
                return Ok(());
            }
            
            if completions.len() == 1 {
                // Single completion - insert it
                self.apply_completion(&completions[0])?;
            } else {
                // Multiple completions - start cycling
                self.completions = completions;
                self.completion_index = Some(0);
                self.completion_prefix = self.get_completion_prefix();
                self.display_completions()?;
            }
        } else {
            // Cycle to next completion
            self.next_completion();
        }
        
        Ok(())
    }
    
    fn apply_completion(&mut self, completion: &CompletionResult) -> io::Result<()> {
        let (_, word_start, _word) = self.completion_engine.parse_completion_context(&self.line, self.cursor_pos);
        
        // Replace the current word with the completion
        self.line.replace_range(word_start..self.cursor_pos, &completion.completion);
        self.cursor_pos = word_start + completion.completion.len();
        
        self.clear_completion_state();
        Ok(())
    }
    
    fn next_completion(&mut self) {
        if let Some(index) = self.completion_index {
            self.completion_index = Some((index + 1) % self.completions.len());
        }
    }
    
    fn previous_completion(&mut self) {
        if let Some(index) = self.completion_index {
            self.completion_index = Some(
                if index == 0 {
                    self.completions.len() - 1
                } else {
                    index - 1
                }
            );
        }
    }
    
    fn get_completion_prefix(&self) -> String {
        let (_, word_start, _) = self.completion_engine.parse_completion_context(&self.line, self.cursor_pos);
        self.line[word_start..self.cursor_pos].to_string()
    }
    
    fn clear_completion_state(&mut self) {
        self.completions.clear();
        self.completion_index = None;
        self.completion_prefix.clear();
    }
    
    fn history_previous(&mut self) {
        if let Some(entry) = self.history.previous() {
            self.line = entry;
            self.cursor_pos = self.line.len();
        }
    }
    
    fn history_next(&mut self) {
        if let Some(entry) = self.history.next() {
            self.line = entry;
            self.cursor_pos = self.line.len();
        } else {
            self.line.clear();
            self.cursor_pos = 0;
        }
    }
    
    fn history_search_backward(&mut self) -> io::Result<()> {
        // Implement reverse search through history
        stdout().execute(Print("\n(reverse-i-search): "))?;
        // Implementation would continue here...
        Ok(())
    }
    
    fn delete_word_backward(&mut self) {
        let mut end = self.cursor_pos;
        
        // Skip whitespace
        while end > 0 && self.line.chars().nth(end - 1).unwrap_or(' ').is_whitespace() {
            end -= 1;
        }
        
        // Delete word
        while end > 0 && !self.line.chars().nth(end - 1).unwrap_or(' ').is_whitespace() {
            end -= 1;
        }
        
        self.line.drain(end..self.cursor_pos);
        self.cursor_pos = end;
    }
    
    fn display_prompt(&mut self) -> io::Result<()> {
        stdout().execute(Print(&self.prompt))?;
        stdout().flush()?;
        Ok(())
    }
    
    fn refresh_display(&mut self) -> io::Result<()> {
        // Move to beginning of line
        stdout().execute(cursor::MoveToColumn(0))?;
        
        // Clear line
        stdout().execute(terminal::Clear(terminal::ClearType::CurrentLine))?;
        
        // Render prompt
        stdout().execute(Print(&self.prompt))?;
        
        // Render line with syntax highlighting
        if self.config.enable_syntax_highlighting {
            self.render_syntax_highlighted_line()?;
        } else {
            stdout().execute(Print(&self.line))?;
        }
        
        // Position cursor
        let prompt_len = self.prompt.chars().count();
        stdout().execute(cursor::MoveToColumn((prompt_len + self.cursor_pos) as u16))?;
        
        // Show completions if active
        if !self.completions.is_empty() {
            self.display_completions()?;
        }
        
        stdout().flush()?;
        Ok(())
    }
    
    fn render_syntax_highlighted_line(&mut self) -> io::Result<()> {
        let words: Vec<&str> = self.line.split_whitespace().collect();
        let mut current_pos = 0;
        
        for (i, word) in words.iter().enumerate() {
            // Find the position of this word in the original string
            if let Some(word_start) = self.line[current_pos..].find(word) {
                let abs_start = current_pos + word_start;
                
                // Print any whitespace before the word
                if abs_start > current_pos {
                    stdout().execute(Print(&self.line[current_pos..abs_start]))?;
                }
                
                // Determine color based on word type
                let color = if i == 0 {
                    // First word is command
                    if self.completion_engine.builtin_cache.contains_key(*word) {
                        Color::Green
                    } else {
                        Color::Blue
                    }
                } else if word.starts_with('-') {
                    // Options
                    Color::Yellow
                } else if word.starts_with('$') {
                    // Variables
                    Color::Cyan
                } else if word.contains('/') || word.contains('\\') {
                    // Paths
                    Color::Magenta
                } else {
                    Color::White
                };
                
                stdout().execute(SetForegroundColor(color))?;
                stdout().execute(Print(word))?;
                stdout().execute(ResetColor)?;
                
                current_pos = abs_start + word.len();
            }
        }
        
        // Print any remaining text
        if current_pos < self.line.len() {
            stdout().execute(Print(&self.line[current_pos..]))?;
        }
        
        Ok(())
    }
    
    fn display_completions(&mut self) -> io::Result<()> {
        if let Some(index) = self.completion_index {
            if let Some(completion) = self.completions.get(index) {
                // Show current completion at the bottom
                let current_row = cursor::position()?.1;
                stdout().execute(cursor::MoveTo(0, current_row + 1))?;
                stdout().execute(terminal::Clear(terminal::ClearType::CurrentLine))?;
                
                stdout().execute(SetForegroundColor(Color::Grey))?;
                stdout().execute(Print(&format!(
                    "[{}/{}] {} {}",
                    index + 1,
                    self.completions.len(),
                    completion.completion,
                    completion.display.as_deref().unwrap_or("")
                )))?;
                stdout().execute(ResetColor)?;
                
                // Move back to input line
                stdout().execute(cursor::MoveTo(0, current_row))?;
            }
        }
        
        Ok(())
    }
}

impl Drop for ReadLine {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

// Extensions for completion engine access
impl NexusCompleter {
    pub fn parse_completion_context<'a>(&self, input: &'a str, cursor_pos: usize) -> (&'a str, usize, &'a str) {
        let input_slice = &input[..cursor_pos];
        
        // Find the start of the current word
        let word_start = input_slice
            .rfind(|c: char| c.is_whitespace() || c == '|' || c == ';' || c == '&')
            .map(|i| i + 1)
            .unwrap_or(0);
        
        let word = &input_slice[word_start..];
        let prefix = &input[..word_start];
        
        (prefix, word_start, word)
    }
}
