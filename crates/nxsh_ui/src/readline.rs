//! Enhanced ReadLine implementation for NexusShell CUI
//! Provides rich line editing with tab completion, history, and syntax highlighting

use std::io::{self, Write, stdout, Stdout};
use unicode_width::UnicodeWidthStr;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent as CrosstermKeyEvent, KeyEventKind, KeyModifiers},
    terminal::{self, enable_raw_mode, disable_raw_mode},
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    ExecutableCommand, QueueableCommand,
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
    // Cached prompt display width
    prompt_width: usize,
    // Number of visual lines in the prompt (for multi-line prompts)
    prompt_lines: usize,
    // Last drawn completion panel height (including borders)
    last_panel_height: usize,
    // Row where the prompt starts (to clear/redraw safely)
    input_row: u16,
    
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
            prompt_width: 0,
            prompt_lines: 1,
            last_panel_height: 0,
            input_row: 0,
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
    // Compute prompt visual metrics with wrapping awareness
    let (rows, last_row_col) = self.compute_prompt_metrics();
    self.prompt_lines = rows.max(1);
    self.prompt_width = last_row_col;
        self.line.clear();
        self.cursor_pos = 0;
        self.clear_completion_state();
    // Ensure no stale panel height from previous sessions
    self.last_panel_height = 0;
        self.history_index = None;
        
        enable_raw_mode()?;
        
        // Display initial prompt
        self.display_prompt()?;
        
        loop {
            match event::read()? {
                Event::Key(key) => {
                    // Ignore key releases and auto-repeats; handle only distinct presses
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
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
                // If completion panel is open, Enter accepts the current selection
                if let Some(idx) = self.completion_index {
                    if let Some(comp) = self.completions.get(idx).cloned() {
                        self.apply_completion(&comp)?;
                        if self.should_add_space_after_completion(&comp) {
                            self.line.insert(self.cursor_pos, ' ');
                            self.cursor_pos += 1;
                        }
                        // Keep panel closed after applying
                        self.clear_completion_state();
                        return Ok(None);
                    }
                }
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
                    // UTF-8 safe backspace: remove the previous char boundary
                    let prev = self.line[..self.cursor_pos].char_indices().last().map(|(i, _)| i).unwrap_or(0);
                    self.line.drain(prev..self.cursor_pos);
                    self.cursor_pos = prev;
                    self.clear_completion_state();
                }
            }
            
            KeyCode::Delete => {
                if self.cursor_pos < self.line.len() {
                    // UTF-8 safe delete: remove next char boundary
                    let mut it = self.line[self.cursor_pos..].char_indices();
                    // Skip current (0) and take next boundary
                    let next = it
                        .nth(0)
                        .map(|(_, ch)| self.cursor_pos + ch.len_utf8())
                        .unwrap_or(self.line.len());
                    self.line.drain(self.cursor_pos..next);
                    self.clear_completion_state();
                }
            }
            
            KeyCode::Left => {
                if self.completion_index.is_some() && !self.completions.is_empty() {
                    self.move_completion_left();
                } else if self.cursor_pos > 0 {
                    // Move left by one Unicode scalar
                    let prev = self.line[..self.cursor_pos].char_indices().last().map(|(i, _)| i).unwrap_or(0);
                    self.cursor_pos = prev;
                    self.clear_completion_state();
                }
            }
            
            KeyCode::Right => {
                if self.completion_index.is_some() && !self.completions.is_empty() {
                    self.move_completion_right();
                } else if self.cursor_pos < self.line.len() {
                    // Move right by one Unicode scalar
                    let mut it = self.line[self.cursor_pos..].char_indices();
                    let next = it.nth(0).map(|(i, ch)| self.cursor_pos + i + ch.len_utf8()).unwrap_or(self.line.len());
                    self.cursor_pos = next;
                    self.clear_completion_state();
                }
            }
            
            KeyCode::Up => {
                if self.completion_index.is_some() && !self.completions.is_empty() {
                    self.previous_completion();
                } else if self.config.enable_history {
                    self.history_previous();
                }
            }
            
            KeyCode::Down => {
                if self.completion_index.is_some() && !self.completions.is_empty() {
                    self.next_completion();
                } else if self.config.enable_history {
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
                        _ => {}
                    }
                } else {
                    // Insert character at cursor (UTF-8 safe)
                    self.line.insert(self.cursor_pos, c);
                    self.cursor_pos += c.len_utf8();
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
                // No completions available - insert space if appropriate
                if self.should_insert_space() {
                    self.line.insert(self.cursor_pos, ' ');
                    self.cursor_pos += 1;
                }
                return Ok(());
            }
            
            if completions.len() == 1 {
                // Single completion - insert it with space if it's a command
                self.apply_completion(&completions[0])?;
                // Add space after command completion
                if self.should_add_space_after_completion(&completions[0]) {
                    self.line.insert(self.cursor_pos, ' ');
                    self.cursor_pos += 1;
                }
            } else {
                // Multiple completions - try common prefix first
                if let Some(common_prefix) = self.find_common_prefix(&completions) {
                    if common_prefix.len() > self.get_completion_prefix().len() {
                        // Apply common prefix completion
                        let dummy_completion = CompletionResult {
                            completion: common_prefix,
                            display: None,
                            completion_type: completions[0].completion_type.clone(),
                            score: 0,
                        };
                        self.apply_completion(&dummy_completion)?;
                        return Ok(());
                    }
                }
                
                // Show multiple completions for cycling
                self.completions = completions;
                self.completion_index = Some(0);
                self.completion_prefix = self.get_completion_prefix();
                // Drawing will occur on next refresh_display
            }
            } else {
            // Cycle to next completion
            self.next_completion();
            // Drawing will occur on next refresh_display
        }
        
        Ok(())
    }
    
    fn apply_completion(&mut self, completion: &CompletionResult) -> io::Result<()> {
        let (_prefix, word_start, word) = self.completion_engine.parse_completion_context(&self.line, self.cursor_pos);
        let mut replacement = completion.completion.clone();

        // Quote paths with spaces or special chars unless already quoted
        if matches!(completion.completion_type, crate::completion::CompletionType::File | crate::completion::CompletionType::Directory) {
            let needs_quote = replacement.chars().any(|c| matches!(c,
                ' ' | '\t' | '(' | ')' | '[' | ']' | '{' | '}' | '&' | ';' | '"' | '\''
            ));
            let already_quoted = word.starts_with('"') || word.starts_with('\'');
            if needs_quote && !already_quoted {
                replacement = format!("\"{}\"", replacement);
            }
        }

        // Replace the current word with the (possibly quoted) completion
        self.line.replace_range(word_start..self.cursor_pos, &replacement);
        self.cursor_pos = word_start + replacement.len();
        
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

    fn move_completion_left(&mut self) {
        // Move selection left by one, wrapping
        self.previous_completion();
    }

    fn move_completion_right(&mut self) {
        // Move selection right by one, wrapping
        self.next_completion();
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
    
    fn should_insert_space(&self) -> bool {
        // Insert space if cursor is at end and last char is not already space
        self.cursor_pos == self.line.len() && 
        !self.line.ends_with(' ') && 
        !self.line.is_empty()
    }
    
    fn should_add_space_after_completion(&self, completion: &CompletionResult) -> bool {
        // Add space after command completions, but not after file/directory completions
        match completion.completion_type {
            crate::completion::CompletionType::Command | 
            crate::completion::CompletionType::Builtin |
            crate::completion::CompletionType::Flag |
            crate::completion::CompletionType::Subcommand |
            crate::completion::CompletionType::EnvVar => true,
            crate::completion::CompletionType::Directory => false, // Allow continuing path
            crate::completion::CompletionType::File => self.cursor_pos == self.line.len(), // Only at end
            _ => false,
        }
    }
    
    fn find_common_prefix(&self, completions: &[CompletionResult]) -> Option<String> {
        if completions.is_empty() {
            return None;
        }
        
        let first = &completions[0].completion;
        let mut common = first.clone();
        
        for completion in &completions[1..] {
            let mut new_common = String::new();
            for (a, b) in common.chars().zip(completion.completion.chars()) {
                if a == b {
                    new_common.push(a);
                } else {
                    break;
                }
            }
            common = new_common;
            if common.is_empty() {
                break;
            }
        }
        
        if !common.is_empty() {
            Some(common)
        } else {
            None
        }
    }
    
    fn history_previous(&mut self) {
        if let Some(entry) = self.history.previous() {
            self.line = entry;
            self.cursor_pos = self.line.len();
        }
    }
    
    fn history_next(&mut self) {
        if let Some(entry) = self.history.next_entry() {
            self.line = entry;
            self.cursor_pos = self.line.len();
        } else {
            self.line.clear();
            self.cursor_pos = 0;
        }
    }
    
    fn history_search_backward(&mut self) -> io::Result<()> {
        // Simple implementation - could be enhanced with incremental search
        if let Some(entry) = self.history.previous() {
            self.line = entry;
            self.cursor_pos = self.line.len();
        }
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
        let mut out = stdout();
        // Capture current row as the prompt start
        out.flush()?;
        self.input_row = cursor::position()?.1;
        let (_, term_height) = terminal::size()?;
        let max_row = term_height.saturating_sub(1);
        // Ensure the full prompt (possibly multi-line) fits within the screen
        let prompt_rows = (self.prompt_lines as u16).max(1);
        let needed_last = self.input_row.saturating_add(prompt_rows.saturating_sub(1));
        if needed_last > max_row {
            // Clamp starting row so that the bottom of the prompt aligns to the last screen row
            self.input_row = max_row.saturating_sub(prompt_rows.saturating_sub(1));
        }

        // Proactively clear the prompt area (all lines the prompt will occupy)
        for r in 0..self.prompt_lines as u16 {
            let row = self.input_row.saturating_add(r);
            if row > max_row { break; }
            out.queue(cursor::MoveTo(0, row))?;
            out.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
        }

    // Print prompt per line (avoid implicit newline side-effects)
        for (i, line) in self.prompt.lines().enumerate() {
            let row = self.input_row.saturating_add(i as u16);
            if row > max_row { break; }
            out.queue(cursor::MoveTo(0, row))?;
            out.queue(Print(line))?;
        }
        out.flush()?;
        Ok(())
    }

    // Compute display width ignoring ANSI escape sequences
    fn visible_width(s: &str) -> usize {
        UnicodeWidthStr::width(Self::strip_ansi(s).as_str())
    }

    // Calculate how many terminal rows the prompt occupies (with wrapping),
    // and what the starting column (0-based) of the final prompt row is.
    fn compute_prompt_metrics(&self) -> (usize, usize) {
        let width = (self.screen_width as usize).max(1);
        let mut rows = 0usize;
        let lines = self.prompt.lines();
        let mut last_line = String::new();
        for line in lines {
            let w = Self::visible_width(line);
            // number of rows for this line (at least 1)
            let add_rows = (w / width) + 1;
            rows += add_rows;
            last_line = line.to_string();
        }
        // Compute last row column offset: visible width of last line modulo terminal width
        let last_w = Self::visible_width(&last_line);
    let last_row_col = last_w % width;
    (rows, last_row_col)
    }

    // Minimal ANSI escape stripper (CSI and OSC) preserving UTF-8 bytes
    // NOTE: Build the output as raw bytes to avoid corrupting multi-byte characters
    fn strip_ansi(s: &str) -> String {
        let bytes = s.as_bytes();
        let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
        let mut i = 0usize;
        while i < bytes.len() {
            if bytes[i] == 0x1B {
                // Start of an escape sequence: CSI (ESC '[') or OSC (ESC ']')
                i += 1;
                if i < bytes.len() && (bytes[i] == b'[' || bytes[i] == b']') {
                    let initiator = bytes[i];
                    i += 1;
                    // Consume until final byte of sequence
                    while i < bytes.len() {
                        let b = bytes[i];
                        i += 1;
                        if initiator == b'[' {
                            // CSI ends at bytes in 0x40..=0x7E
                            if (0x40..=0x7E).contains(&b) { break; }
                        } else {
                            // OSC ends with BEL (0x07) or ST (ESC '\\')
                            if b == 0x07 { break; }
                            if b == 0x1B {
                                if i < bytes.len() && bytes[i] == b'\\' { i += 1; }
                                break;
                            }
                        }
                    }
                    continue;
                }
                // Lone ESC — skip it
                continue;
            }
            // Normal byte, keep as-is
            out.push(bytes[i]);
            i += 1;
        }
        String::from_utf8_lossy(&out).into_owned()
    }
    
    fn refresh_display(&mut self) -> io::Result<()> {
        let mut out = stdout();

        // Clear only the region we own: prompt lines + previous panel (no extra blank line)
        let (_, term_height) = terminal::size()?;
        let max_row = term_height.saturating_sub(1);
        // Clamp starting row if terminal shrank, to keep prompt fully visible
        let prompt_rows = (self.prompt_lines as u16).max(1);
        let needed_last = self.input_row.saturating_add(prompt_rows.saturating_sub(1));
        if needed_last > max_row {
            self.input_row = max_row.saturating_sub(prompt_rows.saturating_sub(1));
        }
        let clear_rows = self.prompt_lines as u16 + (self.last_panel_height as u16);
        for r in 0..clear_rows {
            let row = self.input_row.saturating_add(r);
            if row > max_row { break; }
            out.queue(cursor::MoveTo(0, row))?;
            out.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
        }

        // Render prompt per line at fixed rows
        for (i, line) in self.prompt.lines().enumerate() {
            out.queue(cursor::MoveTo(0, self.input_row + i as u16))?;
            out.queue(terminal::Clear(terminal::ClearType::CurrentLine))?; // ensure full line clean
            // For multi-line prompts, indent subsequent lines slightly to avoid
            // visual confusion with wrapped input. This is purely cosmetic.
            if i > 0 {
                out.queue(Print(""))?; // placeholder (keep future tweak simple)
            }
            out.queue(Print(line))?;
        }

        // Compute caret row and ensure within bounds
        let (_, term_height) = terminal::size()?;
        let max_row = term_height.saturating_sub(1);
        let caret_row = (self.input_row + (self.prompt_lines as u16 - 1)).min(max_row);

        // Render line with syntax highlighting starting after prompt
        if self.config.enable_syntax_highlighting {
            out.queue(cursor::MoveTo(self.prompt_width as u16, caret_row))?;
            self.render_syntax_highlighted_line(&mut out)?;
        } else {
            out.queue(cursor::MoveTo(self.prompt_width as u16, caret_row))?;
            out.queue(Print(&self.line))?;
        }

    // Position cursor using display width (Unicode aware)
        let line_left = &self.line[..self.cursor_pos];
        let line_left_width = UnicodeWidthStr::width(line_left);
        let mut desired_col = (self.prompt_width + line_left_width) as u16;
        if self.screen_width > 0 { desired_col = desired_col.min(self.screen_width - 1); }
        out.queue(cursor::MoveTo(desired_col, caret_row))?;

        // Show completions if active; otherwise clear any previously drawn panel
        if !self.completions.is_empty() {
            // Flush so cursor position is accurate before drawing the panel
            out.flush()?;
            let current_row = caret_row;
            self.display_completions(&mut out, current_row)?;
            // Return cursor to input caret position
            out.queue(cursor::MoveTo(desired_col, current_row))?;
        } else if self.last_panel_height > 0 {
            out.flush()?;
            let current_row = caret_row;
            self.clear_panel_area(&mut out, current_row)?;
            self.last_panel_height = 0;
            out.queue(cursor::MoveTo(desired_col, current_row))?;
        }

        out.flush()?;
        Ok(())
    }
    
    fn render_syntax_highlighted_line(&mut self, out: &mut Stdout) -> io::Result<()> {
        let words: Vec<&str> = self.line.split_whitespace().collect();
        let mut current_pos = 0;
        
        for (i, word) in words.iter().enumerate() {
            // Find the position of this word in the original string
            if let Some(word_start) = self.line[current_pos..].find(word) {
                let abs_start = current_pos + word_start;
                
                // Print any whitespace before the word
                if abs_start > current_pos {
                    out.queue(Print(&self.line[current_pos..abs_start]))?;
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
                
                out.queue(SetForegroundColor(color))?;
                out.queue(Print(word))?;
                out.queue(ResetColor)?;
                
                current_pos = abs_start + word.len();
            }
        }
        
        // Print any remaining text
        if current_pos < self.line.len() {
            out.queue(Print(&self.line[current_pos..]))?;
        }
        
        Ok(())
    }
    
    fn display_completions(&mut self, out: &mut Stdout, current_row: u16) -> io::Result<()> {
        if self.completions.is_empty() || self.completion_index.is_none() {
            return Ok(());
        }
        let width = self.screen_width as usize;
    let (_, term_height) = terminal::size()?;
    let max_row = term_height.saturating_sub(1);

        // Compute column width and layout
        let names: Vec<String> = self.completions.iter().map(|c| {
            if let Some(d) = &c.display { format!("{} — {}", c.completion, d) } else { c.completion.clone() }
        }).collect();

        let max_name = names.iter().map(|s| UnicodeWidthStr::width(s.as_str())).max().unwrap_or(1);
        let col_width = (max_name + 2).min(width.saturating_sub(4)); // padding
        let cols = ((width.saturating_sub(4)) / (col_width.max(1))).max(1);
        let rows = names.len().div_ceil(cols);

        // Draw bordered panel below current line
        let panel_top = current_row.saturating_add(1);
        if panel_top > max_row { return Ok(()); }
        out.queue(cursor::MoveTo(0, panel_top))?;
        for r in 0..(rows + 2) { // +2 for border lines
            let row = panel_top.saturating_add(r as u16);
            if row > max_row { break; }
            out.queue(cursor::MoveTo(0, row))?;
            out.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
            if r == 0 {
                // top border
                out.queue(SetForegroundColor(Color::DarkGrey))?;
                out.queue(Print(format!("┌{:─<width$}┐", "", width = width.saturating_sub(2))))?;
                out.queue(ResetColor)?;
            } else if r == rows + 1 {
                // bottom border
                out.queue(SetForegroundColor(Color::DarkGrey))?;
                out.queue(Print(format!("└{:─<width$}┘", "", width = width.saturating_sub(2))))?;
                out.queue(ResetColor)?;
            } else {
                // content row (r-1)
                let row_idx = r - 1;
                out.queue(SetForegroundColor(Color::DarkGrey))?;
                out.queue(Print("│"))?;
                out.queue(ResetColor)?;

                // Print each cell in the row with proper coloring and padding
                for c in 0..cols {
                    let idx = row_idx + c * rows; // column-major packing
                    if idx < names.len() {
                        let selected = Some(idx) == self.completion_index;
                        let label = &names[idx];
                        let padding = col_width.saturating_sub(UnicodeWidthStr::width(label.as_str()));
                        if selected {
                            out.queue(SetForegroundColor(Color::Cyan))?;
                            out.queue(Print(label))?;
                            out.queue(ResetColor)?;
                        } else {
                            out.queue(Print(label))?;
                        }
                        if padding > 0 {
                            out.queue(Print(" ".repeat(padding)))?;
                        }
                    }
                }

                out.queue(SetForegroundColor(Color::DarkGrey))?;
                out.queue(Print("│"))?;
                out.queue(ResetColor)?;
            }
        }

        // If previous panel was taller, clear the remaining lines to avoid artifacts
        let height = rows + 2; // include top and bottom borders
        if self.last_panel_height > height {
            for r in height..self.last_panel_height {
                let row = panel_top.saturating_add(r as u16);
                if row > max_row { break; }
                out.queue(cursor::MoveTo(0, row))?;
                out.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
            }
        }
        self.last_panel_height = height;
        Ok(())
    }

    fn clear_panel_area(&self, out: &mut Stdout, current_row: u16) -> io::Result<()> {
        let (_, term_height) = terminal::size()?;
        let max_row = term_height.saturating_sub(1);
        let panel_top = current_row.saturating_add(1);
        for r in 0..self.last_panel_height {
            let row = panel_top.saturating_add(r as u16);
            if row > max_row { break; }
            out.queue(cursor::MoveTo(0, row))?;
            out.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn mk() -> ReadLine {
        ReadLine::with_config(ReadLineConfig {
            enable_history: false,
            enable_completion: false,
            enable_syntax_highlighting: false,
            history_size: 10,
            completion_max_items: 5,
            auto_completion: false,
            vi_mode: false,
        }).expect("rl")
    }

    #[test]
    fn utf8_left_right_moves_by_char() {
        let mut rl = mk();
        rl.line = "あいu".to_string();
        rl.cursor_pos = rl.line.len();
        // Left should move to the boundary before 'u'
        let _ = rl.handle_key(KeyEvent { code: KeyCode::Left, modifiers: KeyModifiers::empty() });
        assert!(rl.cursor_pos < rl.line.len());
        // Another left should move before the second multibyte char
        let prev = rl.cursor_pos;
        let _ = rl.handle_key(KeyEvent { code: KeyCode::Left, modifiers: KeyModifiers::empty() });
        assert!(rl.cursor_pos < prev);
        // Right should move forward by one char
        let _ = rl.handle_key(KeyEvent { code: KeyCode::Right, modifiers: KeyModifiers::empty() });
        assert!(rl.cursor_pos > 0);
    }

    #[test]
    fn utf8_backspace_and_delete_remove_one_char() {
        let mut rl = mk();
        rl.line = "あb".to_string();
        rl.cursor_pos = rl.line.len();
        let _ = rl.handle_key(KeyEvent { code: KeyCode::Backspace, modifiers: KeyModifiers::empty() });
        assert_eq!(rl.line, "あ");
        // Insert ASCII and test Delete
        rl.line.push('c');
        rl.cursor_pos = 0;
        let _ = rl.handle_key(KeyEvent { code: KeyCode::Delete, modifiers: KeyModifiers::empty() });
        // First char removed (multibyte)
        assert_eq!(rl.line, "c");
    }
}
