//! Character User Interface Application - NexusShell CUI App
//!
//! This module provides the Character User Interface (CUI) implementation for NexusShell.
//! Delivers a high-performance and responsive command-line experience.

use std::collections::{HashMap, VecDeque};
use std::io::{self, Write, stdout, stderr};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use anyhow::{Result, Error};
use crossterm::{
    cursor, event, execute, queue, style, terminal,
    event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent},
    style::{Color, Stylize, Attribute},
    terminal::{ClearType, disable_raw_mode, enable_raw_mode},
};
use tokio::sync::mpsc;
use unicode_width::UnicodeWidthStr;

use crate::config::UiConfig;
use crate::themes::{Theme, get_theme};
use crate::completion::{CompletionEngine, CompletionResult};
use crate::completion_integration::{IntegratedCompletionSystem, ShellStateProvider};
use crate::prompt::PromptFormatter;
use crate::scroll_buffer::ScrollBuffer;
use crate::ansi_render::AnsiRenderer;
use crate::highlighting::SyntaxHighlighter;

/// CUI application state
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Normal,
    Completing,
    Scrolling,
    Searching,
    CommandMode,
    VisualMode,
    InputMode,
    Exiting,
}

/// Input processing result
#[derive(Debug, Clone, PartialEq)]
pub enum InputResult {
    Continue,
    Execute(String),
    Complete,
    Scroll(ScrollDirection),
    Search(String),
    ChangeMode(AppState),
    Quit,
    Refresh,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
}

/// Main CUI application struct
pub struct CuiApp {
    config: UiConfig,
    theme: Theme,
    state: AppState,
    
    // UI Components
    prompt_formatter: PromptFormatter,
    completion_system: IntegratedCompletionSystem,
    scroll_buffer: ScrollBuffer,
    ansi_renderer: AnsiRenderer,
    highlighter: SyntaxHighlighter,
    
    // Input handling
    input_buffer: String,
    cursor_position: usize,
    selection_start: Option<usize>,
    
    // Completion state
    current_completions: Option<CompletionResult>,
    completion_index: usize,
    completion_visible: bool,
    
    // Terminal state
    terminal_size: (u16, u16),
    is_raw_mode: bool,
    
    // History
    command_history: VecDeque<String>,
    history_index: Option<usize>,
    
    // Performance metrics
    frame_times: VecDeque<Duration>,
    last_render_time: Instant,
    
    // Event channels
    event_sender: Option<mpsc::UnboundedSender<AppEvent>>,
    event_receiver: Option<mpsc::UnboundedReceiver<AppEvent>>,
}

/// Application event
#[derive(Debug, Clone)]
pub enum AppEvent {
    KeyPress(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Refresh,
    Completion(CompletionResult),
    Output(String),
    Error(String),
    StateChange(AppState),
}

impl CuiApp {
    /// Create a new CUI application
    pub fn new(config: UiConfig) -> Result<Self> {
        let theme = get_theme(&config.theme_name)?;
    let prompt_formatter = PromptFormatter::new();
        let completion_system = IntegratedCompletionSystem::new(config.clone());
        let scroll_buffer = ScrollBuffer::new(config.scroll_buffer_size);
    let ansi_renderer = AnsiRenderer::new();
    let highlighter = SyntaxHighlighter::new()?;
        let terminal_size = terminal::size().unwrap_or((80, 24));
        
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            config,
            theme,
            state: AppState::Normal,
            prompt_formatter,
            completion_system,
            scroll_buffer,
            ansi_renderer,
            highlighter,
            input_buffer: String::new(),
            cursor_position: 0,
            selection_start: None,
            current_completions: None,
            completion_index: 0,
            completion_visible: false,
            terminal_size,
            is_raw_mode: false,
            command_history: VecDeque::new(),
            history_index: None,
            frame_times: VecDeque::new(),
            last_render_time: Instant::now(),
            event_sender: Some(event_sender),
            event_receiver: Some(event_receiver),
        })
    }

    /// Initialize the application
    pub async fn initialize(&mut self) -> Result<()> {
        // Enable raw mode
        enable_raw_mode()?;
        self.is_raw_mode = true;

        // Setup terminal
        execute!(
            stdout(),
            terminal::EnterAlternateScreen,
            cursor::EnableBlinking,
            event::EnableMouseCapture
        )?;

        // Load configuration
        self.load_history()?;
        self.setup_completion_system()?;

        // Initial render
        self.render()?;

        Ok(())
    }

    /// Run the application
    pub async fn run(&mut self) -> Result<()> {
        self.initialize().await?;

        let mut event_receiver = self.event_receiver.take().unwrap();

        loop {
            // Handle events
            tokio::select! {
                // Terminal events
                _ = self.handle_terminal_events() => {},
                
                // Application events
                event = event_receiver.recv() => {
                    if let Some(event) = event {
                        match self.handle_app_event(event).await? {
                            InputResult::Quit => break,
                            InputResult::Execute(command) => {
                                self.execute_command(&command).await?;
                            }
                            InputResult::Refresh => {
                                self.render()?;
                            }
                            _ => {}
                        }
                    }
                }
                
                // Periodic updates
                _ = tokio::time::sleep(Duration::from_millis(16)) => {
                    self.update().await?;
                }
            }
        }

        self.shutdown().await?;
        Ok(())
    }

    /// Handle terminal events
    async fn handle_terminal_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key_event) => {
                    if let Some(sender) = &self.event_sender {
                        sender.send(AppEvent::KeyPress(key_event))?;
                    }
                }
                Event::Mouse(mouse_event) => {
                    if let Some(sender) = &self.event_sender {
                        sender.send(AppEvent::Mouse(mouse_event))?;
                    }
                }
                Event::Resize(cols, rows) => {
                    self.handle_resize(cols, rows).await?;
                    if let Some(sender) = &self.event_sender {
                        sender.send(AppEvent::Resize(cols, rows))?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle application events
    async fn handle_app_event(&mut self, event: AppEvent) -> Result<InputResult> {
        match event {
            AppEvent::KeyPress(key_event) => {
                self.handle_key_event(key_event).await
            }
            AppEvent::Mouse(mouse_event) => {
                self.handle_mouse_event(mouse_event).await
            }
            AppEvent::Resize(cols, rows) => {
                self.handle_resize(cols, rows).await
            }
            AppEvent::Refresh => {
                Ok(InputResult::Refresh)
            }
            AppEvent::Completion(result) => {
                self.handle_completion_result(result).await
            }
            AppEvent::Output(text) => {
                self.add_output(&text);
                Ok(InputResult::Refresh)
            }
            AppEvent::Error(text) => {
                self.add_error(&text);
                Ok(InputResult::Refresh)
            }
            AppEvent::StateChange(new_state) => {
                self.state = new_state;
                Ok(InputResult::Refresh)
            }
        }
    }

    /// Handle key events
    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<InputResult> {
        match self.state {
            AppState::Normal => self.handle_normal_mode_key(key_event).await,
            AppState::Completing => self.handle_completion_mode_key(key_event).await,
            AppState::Scrolling => self.handle_scroll_mode_key(key_event).await,
            AppState::Searching => self.handle_search_mode_key(key_event).await,
            AppState::CommandMode => self.handle_command_mode_key(key_event).await,
            AppState::VisualMode => self.handle_visual_mode_key(key_event).await,
            AppState::InputMode => self.handle_input_mode_key(key_event).await,
            AppState::Exiting => Ok(InputResult::Quit),
        }
    }

    /// Handle normal mode key input
    async fn handle_normal_mode_key(&mut self, key_event: KeyEvent) -> Result<InputResult> {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Enter, _) => {
                if !self.input_buffer.is_empty() {
                    let command = self.input_buffer.clone();
                    self.add_to_history(command.clone());
                    self.clear_input();
                    Ok(InputResult::Execute(command))
                } else {
                    Ok(InputResult::Continue)
                }
            }
            
            (KeyCode::Tab, _) => {
                self.trigger_completion().await
            }
            
            (KeyCode::Char(c), KeyModifiers::CONTROL) => {
                match c {
                    'c' => Ok(InputResult::Quit),
                    'd' => {
                        if self.input_buffer.is_empty() {
                            Ok(InputResult::Quit)
                        } else {
                            Ok(InputResult::Continue)
                        }
                    }
                    'l' => {
                        execute!(stdout(), terminal::Clear(ClearType::All))?;
                        Ok(InputResult::Refresh)
                    }
                    'r' => {
                        self.trigger_search().await
                    }
                    'u' => {
                        self.clear_input();
                        Ok(InputResult::Refresh)
                    }
                    'w' => {
                        self.delete_word_backward();
                        Ok(InputResult::Refresh)
                    }
                    'a' => {
                        self.cursor_position = 0;
                        Ok(InputResult::Refresh)
                    }
                    'e' => {
                        self.cursor_position = self.input_buffer.len();
                        Ok(InputResult::Refresh)
                    }
                    _ => Ok(InputResult::Continue),
                }
            }
            
            (KeyCode::Char(c), _) => {
                self.insert_char(c);
                Ok(InputResult::Refresh)
            }
            
            (KeyCode::Backspace, _) => {
                self.delete_backward();
                Ok(InputResult::Refresh)
            }
            
            (KeyCode::Delete, _) => {
                self.delete_forward();
                Ok(InputResult::Refresh)
            }
            
            (KeyCode::Left, _) => {
                self.move_cursor_left();
                Ok(InputResult::Refresh)
            }
            
            (KeyCode::Right, _) => {
                self.move_cursor_right();
                Ok(InputResult::Refresh)
            }
            
            (KeyCode::Up, _) => {
                self.history_previous();
                Ok(InputResult::Refresh)
            }
            
            (KeyCode::Down, _) => {
                self.history_next();
                Ok(InputResult::Refresh)
            }
            
            (KeyCode::Home, _) => {
                self.cursor_position = 0;
                Ok(InputResult::Refresh)
            }
            
            (KeyCode::End, _) => {
                self.cursor_position = self.input_buffer.len();
                Ok(InputResult::Refresh)
            }
            
            (KeyCode::PageUp, _) => {
                Ok(InputResult::Scroll(ScrollDirection::PageUp))
            }
            
            (KeyCode::PageDown, _) => {
                Ok(InputResult::Scroll(ScrollDirection::PageDown))
            }
            
            _ => Ok(InputResult::Continue),
        }
    }

    /// Handle completion mode key input
    async fn handle_completion_mode_key(&mut self, key_event: KeyEvent) -> Result<InputResult> {
        match key_event.code {
            KeyCode::Tab => {
                self.select_next_completion();
                Ok(InputResult::Refresh)
            }
            KeyCode::BackTab => {
                self.select_previous_completion();
                Ok(InputResult::Refresh)
            }
            KeyCode::Enter => {
                self.apply_completion();
                self.state = AppState::Normal;
                Ok(InputResult::Refresh)
            }
            KeyCode::Esc => {
                self.cancel_completion();
                self.state = AppState::Normal;
                Ok(InputResult::Refresh)
            }
            KeyCode::Up => {
                self.select_previous_completion();
                Ok(InputResult::Refresh)
            }
            KeyCode::Down => {
                self.select_next_completion();
                Ok(InputResult::Refresh)
            }
            KeyCode::PageUp => {
                self.select_previous_completion_page();
                Ok(InputResult::Refresh)
            }
            KeyCode::PageDown => {
                self.select_next_completion_page();
                Ok(InputResult::Refresh)
            }
            KeyCode::Home => {
                self.select_first_completion();
                Ok(InputResult::Refresh)
            }
            KeyCode::End => {
                self.select_last_completion();
                Ok(InputResult::Refresh)
            }
            // Allow character input to continue typing and filter completions
            KeyCode::Char(c) => {
                self.insert_char(c);
                // Update completions with new input
                self.trigger_completion().await
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.delete_backward();
                    // Update completions with modified input
                    if self.input_buffer.is_empty() {
                        self.cancel_completion();
                        self.state = AppState::Normal;
                    } else {
                        self.trigger_completion().await?;
                    }
                }
                Ok(InputResult::Refresh)
            }
            _ => {
                // Cancel completion and handle as normal
                self.cancel_completion();
                self.state = AppState::Normal;
                self.handle_normal_mode_key(key_event).await
            }
        }
    }

    /// Handle scroll mode key input
    async fn handle_scroll_mode_key(&mut self, key_event: KeyEvent) -> Result<InputResult> {
        match key_event.code {
            KeyCode::Esc => {
                self.state = AppState::Normal;
                Ok(InputResult::Refresh)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                Ok(InputResult::Scroll(ScrollDirection::Up))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                Ok(InputResult::Scroll(ScrollDirection::Down))
            }
            KeyCode::PageUp => {
                Ok(InputResult::Scroll(ScrollDirection::PageUp))
            }
            KeyCode::PageDown => {
                Ok(InputResult::Scroll(ScrollDirection::PageDown))
            }
            KeyCode::Home => {
                Ok(InputResult::Scroll(ScrollDirection::Home))
            }
            KeyCode::End => {
                Ok(InputResult::Scroll(ScrollDirection::End))
            }
            _ => Ok(InputResult::Continue),
        }
    }

    /// Handle search mode key input
    async fn handle_search_mode_key(&mut self, key_event: KeyEvent) -> Result<InputResult> {
        // Search functionality implementation
        match key_event.code {
            KeyCode::Enter => {
                self.execute_search();
                self.state = AppState::Normal;
                Ok(InputResult::Refresh)
            }
            KeyCode::Esc => {
                self.cancel_search();
                self.state = AppState::Normal;
                Ok(InputResult::Refresh)
            }
            _ => {
                // Handle search input
                Ok(InputResult::Continue)
            }
        }
    }

    /// Handle command mode key input
    async fn handle_command_mode_key(&mut self, key_event: KeyEvent) -> Result<InputResult> {
        // Vi-style command mode
        Ok(InputResult::Continue)
    }

    /// Handle visual mode key input
    async fn handle_visual_mode_key(&mut self, key_event: KeyEvent) -> Result<InputResult> {
        // Text selection mode
        Ok(InputResult::Continue)
    }

    /// Handle input mode key input
    async fn handle_input_mode_key(&mut self, key_event: KeyEvent) -> Result<InputResult> {
        // Special input mode
        Ok(InputResult::Continue)
    }

    /// Handle mouse events
    async fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> Result<InputResult> {
        // Mouse operation implementation
        Ok(InputResult::Continue)
    }

    /// Handle resize events
    async fn handle_resize(&mut self, cols: u16, rows: u16) -> Result<InputResult> {
        self.terminal_size = (cols, rows);
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        Ok(InputResult::Refresh)
    }

    /// Start completion
    async fn trigger_completion(&mut self) -> Result<InputResult> {
        let completions = self.completion_system.get_intelligent_completions(
            &self.input_buffer,
            self.cursor_position,
        )?;
        
        if !completions.items.is_empty() {
            self.current_completions = Some(completions);
            self.completion_index = 0;
            self.completion_visible = true;
            self.state = AppState::Completing;
        }
        
        Ok(InputResult::Refresh)
    }

    /// Start search
    async fn trigger_search(&mut self) -> Result<InputResult> {
        self.state = AppState::Searching;
        Ok(InputResult::Refresh)
    }

    /// Handle completion result
    async fn handle_completion_result(&mut self, result: CompletionResult) -> Result<InputResult> {
        self.current_completions = Some(result);
        self.completion_index = 0;
        self.completion_visible = true;
        Ok(InputResult::Refresh)
    }

    // duplicate block removed (early render_prompt_and_input and helpers)

    // removed duplicate helper block (history, completion, output, search)

    // duplicate block removed (helpers)

    // removed duplicate helper block (history, completion, output, search)

    // removed duplicate helper block (ansi width + editing helpers)

    // removed duplicate helper block (history, completion, output, search)

    // duplicate block removed

    fn estimate_prompt_length(&self, prompt_text: &str) -> usize {
        let cleaned = Self::strip_ansi_sequences(prompt_text);
        // Use unicode width calculation but ensure it's not zero
        UnicodeWidthStr::width(cleaned.as_str()).max(1)
    }

    fn input_display_width_up_to_cursor(&self) -> usize {
        if self.input_buffer.is_empty() || self.cursor_position == 0 {
            return 0;
        }
        
        let up_to = self.cursor_position.min(self.input_buffer.len());
        
        // Ensure we're at a character boundary
        let safe_up_to = if self.input_buffer.is_char_boundary(up_to) {
            up_to
        } else {
            // Find the previous character boundary
            (0..up_to).rev().find(|&i| self.input_buffer.is_char_boundary(i)).unwrap_or(0)
        };
        
        let slice = &self.input_buffer[..safe_up_to];
        UnicodeWidthStr::width(slice)
    }

    fn strip_ansi_sequences(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let bytes = s.as_bytes();
        let mut i = 0;
        
        while i < bytes.len() {
            if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                // Skip ANSI escape sequence
                i += 2;
                // Find the end of the sequence (letter character)
                while i < bytes.len() {
                    let b = bytes[i];
                    i += 1;
                    if b.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                // Regular character
                if bytes[i].is_ascii() || s.is_char_boundary(i) {
                    out.push(bytes[i] as char);
                }
                i += 1;
            }
        }
        out
    }

    fn insert_char(&mut self, c: char) {
        // keep cursor_position as byte index at char boundary
        self.input_buffer.insert(self.cursor_position, c);
        self.cursor_position += c.len_utf8();
    }

    fn delete_backward(&mut self) {
        if self.cursor_position > 0 {
            let prev = self.prev_char_boundary(self.cursor_position);
            self.input_buffer.drain(prev..self.cursor_position);
            self.cursor_position = prev;
        }
    }

    fn delete_forward(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            let next = self.next_char_boundary(self.cursor_position);
            self.input_buffer.drain(self.cursor_position..next);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position = self.prev_char_boundary(self.cursor_position);
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position = self.next_char_boundary(self.cursor_position);
        }
    }

    #[inline]
    fn prev_char_boundary(&self, idx: usize) -> usize {
        let mut i = idx.saturating_sub(1);
        while i > 0 && !self.input_buffer.is_char_boundary(i) { i -= 1; }
        i
    }

    #[inline]
    fn next_char_boundary(&self, idx: usize) -> usize {
        let mut i = (idx + 1).min(self.input_buffer.len());
        while i < self.input_buffer.len() && !self.input_buffer.is_char_boundary(i) { i += 1; }
        i
    }

    /// Clear input
    fn clear_input(&mut self) {
        self.input_buffer.clear();
        self.cursor_position = 0;
    }

    /// 履歴管理
    fn add_to_history(&mut self, command: String) {
        if !command.trim().is_empty() {
            self.command_history.push_back(command);
            if self.command_history.len() > self.config.history_size {
                self.command_history.pop_front();
            }
        }
        self.history_index = None;
    }

    fn history_previous(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        let new_index = match self.history_index {
            None => self.command_history.len() - 1,
            Some(index) => {
                if index > 0 {
                    index - 1
                } else {
                    return;
                }
            }
        };

        if let Some(command) = self.command_history.get(new_index) {
            self.input_buffer = command.clone();
            self.cursor_position = self.input_buffer.len();
            self.history_index = Some(new_index);
        }
    }

    fn history_next(&mut self) {
        match self.history_index {
            None => return,
            Some(index) => {
                if index + 1 < self.command_history.len() {
                    let new_index = index + 1;
                    if let Some(command) = self.command_history.get(new_index) {
                        self.input_buffer = command.clone();
                        self.cursor_position = self.input_buffer.len();
                        self.history_index = Some(new_index);
                    }
                } else {
                    self.clear_input();
                    self.history_index = None;
                }
            }
        }
    }

    /// Handle terminal resize
    async fn handle_resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        let old_size = self.terminal_size;
        self.terminal_size = (cols, rows);
        
        // Log resize for debugging
        if cols != old_size.0 || rows != old_size.1 {
            self.scroll_buffer.add_line(format!(
                "Terminal resized: {}x{} -> {}x{}", 
                old_size.0, old_size.1, cols, rows
            ));
        }
        
        // Clear prompt cache to force regeneration with new dimensions
        self.prompt_formatter.invalidate_cache();
        
        // Adjust scroll buffer to new size if needed
        let new_buffer_size = (rows as usize).saturating_sub(3).max(10) * 100; // Reasonable buffer size
        if new_buffer_size != self.config.scroll_buffer_size {
            self.scroll_buffer = ScrollBuffer::new(new_buffer_size);
        }
        
        // Force full screen refresh
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        self.render()?;
        
        Ok(())
    }

    /// 補完操作
    fn select_next_completion(&mut self) {
        if let Some(completions) = &self.current_completions {
            if !completions.items.is_empty() {
                self.completion_index = (self.completion_index + 1) % completions.items.len();
            }
        }
    }

    fn select_previous_completion(&mut self) {
        if let Some(completions) = &self.current_completions {
            if !completions.items.is_empty() {
                self.completion_index = if self.completion_index > 0 {
                    self.completion_index - 1
                } else {
                    completions.items.len() - 1
                };
            }
        }
    }

    fn select_next_completion_page(&mut self) {
        if let Some(completions) = &self.current_completions {
            if !completions.items.is_empty() {
                let page_size = 8; // Same as max_items in render_completion_panel
                let new_index = (self.completion_index + page_size).min(completions.items.len() - 1);
                self.completion_index = new_index;
            }
        }
    }

    fn select_previous_completion_page(&mut self) {
        if let Some(completions) = &self.current_completions {
            if !completions.items.is_empty() {
                let page_size = 8; // Same as max_items in render_completion_panel
                let new_index = self.completion_index.saturating_sub(page_size);
                self.completion_index = new_index;
            }
        }
    }

    fn select_first_completion(&mut self) {
        if let Some(completions) = &self.current_completions {
            if !completions.items.is_empty() {
                self.completion_index = 0;
            }
        }
    }

    fn select_last_completion(&mut self) {
        if let Some(completions) = &self.current_completions {
            if !completions.items.is_empty() {
                self.completion_index = completions.items.len() - 1;
            }
        }
    }

    fn apply_completion(&mut self) {
        if let Some(completions) = &self.current_completions {
            if let Some(item) = completions.items.get(self.completion_index) {
                // Replace the current word with the completion
                let prefix_len = completions.prefix.len();
                let start_pos = self.cursor_position.saturating_sub(prefix_len);
                
                self.input_buffer.drain(start_pos..self.cursor_position);
                self.input_buffer.insert_str(start_pos, &item.text);
                self.cursor_position = start_pos + item.text.len();
                
                // Add space after command completions for better UX
                if item.completion_type == crate::completion_engine::CompletionType::Command ||
                   item.completion_type == crate::completion_engine::CompletionType::Builtin {
                    self.input_buffer.insert(self.cursor_position, ' ');
                    self.cursor_position += 1;
                }
            }
        }
        self.current_completions = None;
        self.completion_visible = false;
    }

    fn cancel_completion(&mut self) {
        self.current_completions = None;
        self.completion_visible = false;
        self.completion_index = 0;
    }

    /// 出力管理
    fn add_output(&mut self, text: &str) {
        self.scroll_buffer.add_line(text.to_string());
    }

    fn add_error(&mut self, text: &str) {
        let error_text = format!("{}", text.with(self.theme.colors.error));
        self.scroll_buffer.add_line(error_text);
    }

    /// Search functionality
    fn execute_search(&mut self) {
        // Search functionality implementation
    }

    fn cancel_search(&mut self) {
        // Cancel search
    }

    // duplicate block removed (render)

    // duplicate block removed: render_prompt_and_input and editing helpers

    // duplicate block removed (render and helpers)

    /// 🎨 Enhanced rendering with beautiful visual effects
    fn render(&mut self) -> Result<()> {
        let start_time = Instant::now();

        // Smooth screen management with minimal flicker
        let should_clear_screen = self.should_clear_screen();
        
        if should_clear_screen {
            // Use smooth clear with background color
            execute!(stdout(), 
                style::SetBackgroundColor(Color::Black),
                terminal::Clear(ClearType::All),
                cursor::MoveTo(0, 0)
            )?;
        }

        // 🌟 Progressive rendering with visual hierarchy
        self.render_header_section()?;
        self.render_output_buffer_enhanced()?;
        self.render_interactive_section()?;
        
        // 🎯 Render completion panel with advanced animations
        if self.completion_visible {
            self.render_completion_panel()?;
        }

        // 📊 Enhanced status and info display
        self.render_footer_section()?;

        // 💫 Visual effects and animations
        self.apply_visual_effects()?;

        // Performance-optimized flush
        stdout().flush()?;

        // Update performance metrics
        let render_time = start_time.elapsed();
        self.frame_times.push_back(render_time);
        if self.frame_times.len() > 100 {
            self.frame_times.pop_front();
        }
        self.last_render_time = Instant::now();

        Ok(())
    }

    /// 🎨 Render beautiful header section
    fn render_header_section(&mut self) -> Result<()> {
        if self.config.show_header {
            execute!(stdout(), cursor::MoveTo(0, 0))?;
            
            // Gradient header bar
            let header_text = "🚀 NexusShell - Advanced Command Interface";
            let padding = (self.terminal_size.0 as usize).saturating_sub(header_text.len()) / 2;
            
            print!("\x1b[48;5;24m"); // Deep blue background
            print!("\x1b[38;5;231m"); // White text
            print!("{}", " ".repeat(padding));
            print!("{}", header_text);
            print!("{}", " ".repeat(self.terminal_size.0 as usize - padding - header_text.len()));
            print!("\x1b[0m\n"); // Reset
        }
        Ok(())
    }

    /// 🌈 Enhanced output buffer rendering
    fn render_output_buffer_enhanced(&mut self) -> Result<()> {
        let buffer_height = if self.config.show_header { 
            self.terminal_size.1.saturating_sub(4) 
        } else { 
            self.terminal_size.1.saturating_sub(3) 
        };
        
        let start_row = if self.config.show_header { 1 } else { 0 };
        
        // Render with enhanced styling
        for (i, line) in self.scroll_buffer.visible_lines(buffer_height as usize).iter().enumerate() {
            execute!(stdout(), cursor::MoveTo(0, start_row + i as u16))?;
            execute!(stdout(), terminal::Clear(ClearType::CurrentLine))?;
            
            // Apply smart syntax highlighting and formatting
            if line.starts_with("Error:") || line.contains("error") {
                print!("\x1b[91m🚫 {}\x1b[0m", line); // Red with error icon
            } else if line.starts_with("Warning:") || line.contains("warning") {
                print!("\x1b[93m⚠️  {}\x1b[0m", line); // Yellow with warning icon
            } else if line.starts_with("Success:") || line.contains("✓") {
                print!("\x1b[92m✅ {}\x1b[0m", line); // Green with success icon
            } else if line.starts_with("Info:") || line.contains("info") {
                print!("\x1b[96m💡 {}\x1b[0m", line); // Cyan with info icon
            } else {
                // Apply advanced syntax highlighting
                self.render_syntax_highlighted_line(line)?;
            }
        }
        
        Ok(())
    }

    /// 🎯 Interactive section with prompt and input
    fn render_interactive_section(&mut self) -> Result<()> {
        let prompt_row = self.terminal_size.1.saturating_sub(2);
        execute!(stdout(), cursor::MoveTo(0, prompt_row))?;
        execute!(stdout(), terminal::Clear(ClearType::CurrentLine))?;
        
        // Enhanced prompt rendering
        self.render_enhanced_prompt_and_input()?;
        
        Ok(())
    }

    /// 📊 Beautiful footer section with status
    fn render_footer_section(&mut self) -> Result<()> {
        let footer_row = self.terminal_size.1.saturating_sub(1);
        execute!(stdout(), cursor::MoveTo(0, footer_row))?;
        execute!(stdout(), terminal::Clear(ClearType::CurrentLine))?;
        
        // Enhanced status line with rich information
        self.render_enhanced_status_line()?;
        
        Ok(())
    }

    /// ✨ Apply visual effects and animations
    fn apply_visual_effects(&mut self) -> Result<()> {
        // Subtle cursor blinking animation
        if self.state == AppState::Normal {
            let blink_phase = (self.last_render_time.elapsed().as_millis() / 500) % 2;
            if blink_phase == 0 {
                execute!(stdout(), style::SetAttribute(Attribute::SlowBlink))?;
            } else {
                execute!(stdout(), style::SetAttribute(Attribute::NoBlink))?;
            }
        }
        
        Ok(())
    }
            eprintln!("Warning: Failed to flush output: {}", e);
        }

        // Update performance metrics
        let render_time = start_time.elapsed();
        self.frame_times.push_back(render_time);
        if self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }
        self.last_render_time = start_time;

        Ok(())
    }
    
    /// Determine if screen should be cleared based on various conditions
    fn should_clear_screen(&self) -> bool {
        // Clear screen if:
        // 1. It's the first render (last_render_time is very old)
        // 2. Terminal was likely resized (significant time elapsed)
        // 3. We're switching modes that require full refresh
        let time_since_last_render = self.last_render_time.elapsed();
        
        time_since_last_render > Duration::from_millis(1000) || // 1 second instead of 500ms
        matches!(self.state, AppState::CommandMode | AppState::VisualMode)
    }

    fn render_output_buffer(&mut self) -> Result<()> {
        let available_rows = self.terminal_size.1.saturating_sub(3); // Reserve space for prompt and status
        
        // Safety check for terminal size
        if available_rows == 0 {
            return Ok(()); // Terminal too small to render anything
        }
        
        let lines = self.scroll_buffer.get_visible_lines(available_rows as usize);

        for (i, line) in lines.iter().enumerate() {
            let row = i as u16;
            
            // Safety check to prevent going beyond terminal bounds
            if row >= available_rows {
                break;
            }
            
            execute!(stdout(), cursor::MoveTo(0, row))?;
            
            // Clear the line to prevent artifacts
            execute!(stdout(), terminal::Clear(ClearType::CurrentLine))?;
            
            // Render the line with proper ANSI handling and error recovery
            match self.ansi_renderer.render(line, &self.theme) {
                Ok(rendered) => {
                    // Truncate line if it's too long for the terminal
                    let term_width = self.terminal_size.0 as usize;
                    let display_line = if rendered.len() > term_width {
                        format!("{}…", &rendered[..term_width.saturating_sub(1)])
                    } else {
                        rendered
                    };
                    print!("{}", display_line);
                },
                Err(_) => {
                    // Fallback to sanitized plain text if rendering fails
                    let sanitized = line.chars()
                        .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
                        .collect::<String>();
                    let term_width = self.terminal_size.0 as usize;
                    let display_line = if sanitized.len() > term_width {
                        format!("{}…", &sanitized[..term_width.saturating_sub(1)])
                    } else {
                        sanitized
                    };
                    print!("{}", display_line);
                }
            }
        }

        // Clear any remaining lines in the output area to prevent visual artifacts
        for i in lines.len()..available_rows as usize {
            if i as u16 >= available_rows {
                break; // Safety check
            }
            execute!(stdout(), cursor::MoveTo(0, i as u16))?;
            execute!(stdout(), terminal::Clear(ClearType::CurrentLine))?;
        }

        Ok(())
    }

    fn render_prompt_and_input(&mut self) -> Result<()> {
        let prompt_row = self.terminal_size.1.saturating_sub(2);
        execute!(stdout(), cursor::MoveTo(0, prompt_row))?;

    /// 🎨 Enhanced prompt and input rendering with beautiful styling
    fn render_enhanced_prompt_and_input(&mut self) -> Result<()> {
        let prompt_row = self.terminal_size.1.saturating_sub(2);
        execute!(stdout(), cursor::MoveTo(0, prompt_row))?;
        execute!(stdout(), terminal::Clear(ClearType::CurrentLine))?;

        // 🚀 Generate enhanced prompt with context awareness
        let enhanced_prompt = self.generate_enhanced_prompt()?;
        print!("{}", enhanced_prompt);

        // 🌈 Advanced syntax highlighting for input
        let highlighted_input = self.apply_advanced_highlighting(&self.input_buffer)?;
        
        // 🎯 Smart input display with visual indicators
        let display_input = self.format_input_with_indicators(&highlighted_input);
        print!("{}", display_input);

        // ✨ Enhanced cursor positioning with visual effects
        self.position_enhanced_cursor(prompt_row, &enhanced_prompt)?;

        Ok(())
    }

    /// 🚀 Generate enhanced prompt with rich context information
    fn generate_enhanced_prompt(&mut self) -> Result<String> {
        let base_prompt = self.prompt_formatter.generate_prompt()
            .unwrap_or_else(|_| "$ ".to_string());
        
        // Add context-aware decorations
        let mut enhanced = String::new();
        
        // 🎨 Mode indicator with colored background
        let mode_indicator = match self.state {
            AppState::Normal => "\x1b[42;30m READY \x1b[0m",
            AppState::Completing => "\x1b[44;30m COMP \x1b[0m",
            AppState::CommandMode => "\x1b[45;30m CMD \x1b[0m",
            AppState::VisualMode => "\x1b[46;30m VIS \x1b[0m",
            AppState::Searching => "\x1b[43;30m SRCH \x1b[0m",
            _ => "\x1b[47;30m --- \x1b[0m",
        };
        
        enhanced.push_str(mode_indicator);
        enhanced.push(' ');
        
        // 📊 Add performance indicator
        if let Some(avg_time) = self.get_average_frame_time() {
            if avg_time.as_millis() > 50 {
                enhanced.push_str("\x1b[91m🐌\x1b[0m "); // Slow performance warning
            } else if avg_time.as_millis() < 16 {
                enhanced.push_str("\x1b[92m⚡\x1b[0m "); // Fast performance indicator
            }
        }
        
        // 🎯 Enhanced base prompt with styling
        enhanced.push_str(&format!("\x1b[96;1m{}\x1b[0m", base_prompt));
        
        Ok(enhanced)
    }

    /// 🌈 Apply advanced syntax highlighting to input
    fn apply_advanced_highlighting(&mut self, input: &str) -> Result<String> {
        if input.is_empty() {
            return Ok(String::new());
        }
        
        // Try advanced highlighting first
        if let Ok(highlighted) = self.highlighter.highlight_line(input) {
            return Ok(highlighted);
        }
        
        // Fallback to basic highlighting
        let mut result = String::new();
        let words: Vec<&str> = input.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            if i == 0 {
                // First word (command) - bright green
                result.push_str(&format!("\x1b[92;1m{}\x1b[0m", word));
            } else if word.starts_with('-') {
                // Options - bright yellow
                result.push_str(&format!("\x1b[93m{}\x1b[0m", word));
            } else if word.contains('/') || word.contains('\\') {
                // Paths - bright cyan
                result.push_str(&format!("\x1b[96m{}\x1b[0m", word));
            } else {
                // Arguments - default color
                result.push_str(&format!("\x1b[37m{}\x1b[0m", word));
            }
            
            if i < words.len() - 1 {
                result.push(' ');
            }
        }
        
        Ok(result)
    }

    /// 🎯 Format input with visual indicators
    fn format_input_with_indicators(&self, highlighted_input: &str) -> String {
        let mut result = highlighted_input.to_string();
        
        // Add selection indicators if selection is active
        if let Some(start) = self.selection_start {
            let end = self.cursor_position;
            if start != end {
                // Add selection highlighting (this is simplified)
                result = format!("\x1b[7m{}\x1b[0m", result); // Inverse colors for selection
            }
        }
        
        // Add completion hint if available
        if self.completion_visible && !self.input_buffer.is_empty() {
            if let Some(completions) = &self.current_completions {
                if !completions.items.is_empty() {
                    let hint = &completions.items[self.completion_index % completions.items.len()].text;
                    if hint.len() > self.input_buffer.len() {
                        let completion_hint = &hint[self.input_buffer.len()..];
                        result.push_str(&format!("\x1b[90m{}\x1b[0m", completion_hint)); // Gray hint
                    }
                }
            }
        }
        
        result
    }

    /// ✨ Enhanced cursor positioning with visual effects
    fn position_enhanced_cursor(&mut self, prompt_row: u16, enhanced_prompt: &str) -> Result<()> {
        let prompt_width = self.calculate_display_width(enhanced_prompt) as u16;
        let input_width_to_cursor = self.input_display_width_up_to_cursor() as u16;
        let cursor_x = prompt_width.saturating_add(input_width_to_cursor);
        
        // Ensure cursor stays within bounds
        let safe_cursor_x = cursor_x.min(self.terminal_size.0.saturating_sub(1));
        execute!(stdout(), cursor::MoveTo(safe_cursor_x, prompt_row))?;
        
        // Add cursor enhancement based on mode
        match self.state {
            AppState::Normal => {
                execute!(stdout(), style::SetAttribute(Attribute::SlowBlink))?;
            }
            AppState::Completing => {
                execute!(stdout(), style::SetAttribute(Attribute::RapidBlink))?;
            }
            _ => {
                execute!(stdout(), style::SetAttribute(Attribute::NoBlink))?;
            }
        }
        
        Ok(())
    }

    /// 📊 Calculate actual display width accounting for ANSI sequences
    fn calculate_display_width(&self, text: &str) -> usize {
        // Simple ANSI sequence removal for width calculation
        let clean_text = regex::Regex::new(r"\x1b\[[0-9;]*m")
            .unwrap()
            .replace_all(text, "");
        clean_text.width()
    }

    /// ⏱️ Get average frame time for performance indicators
    fn get_average_frame_time(&self) -> Option<Duration> {
        if self.frame_times.is_empty() {
            return None;
        }
        
        let total: Duration = self.frame_times.iter().sum();
        Some(total / self.frame_times.len() as u32)
    }

    /// 📊 Enhanced status line with rich information display
    fn render_enhanced_status_line(&mut self) -> Result<()> {
        let status_row = self.terminal_size.1.saturating_sub(1);
        execute!(stdout(), cursor::MoveTo(0, status_row))?;
        execute!(stdout(), terminal::Clear(ClearType::CurrentLine))?;

        let width = self.terminal_size.0 as usize;
        
        // 🎨 Left section: Mode and status
        let left_section = self.build_status_left_section();
        
        // 📊 Center section: Information
        let center_section = self.build_status_center_section();
        
        // ⚡ Right section: Performance and system info
        let right_section = self.build_status_right_section();
        
        // Calculate sections and spacing
        let left_width = self.calculate_display_width(&left_section);
        let center_width = self.calculate_display_width(&center_section);
        let right_width = self.calculate_display_width(&right_section);
        
        let total_content = left_width + center_width + right_width;
        
        if total_content <= width {
            // Calculate spacing
            let available_space = width - total_content;
            let left_padding = available_space / 2;
            let right_padding = available_space - left_padding;
            
            // Render with beautiful formatting
            print!("\x1b[48;5;236m"); // Dark gray background
            print!("{}", left_section);
            print!("{}", " ".repeat(left_padding));
            print!("{}", center_section);
            print!("{}", " ".repeat(right_padding));
            print!("{}", right_section);
            print!("\x1b[0m"); // Reset
        } else {
            // Fallback for narrow terminals
            let simple_status = format!(" {} ", self.get_current_mode_display());
            print!("\x1b[48;5;236m\x1b[97m{}\x1b[0m", simple_status);
        }
        
        Ok(())
    }

    /// 🎨 Build left section of status line
    fn build_status_left_section(&self) -> String {
        let mode_display = self.get_current_mode_display();
        let mode_color = self.get_mode_color();
        
        format!(" \x1b[{}m● {}\x1b[0m", mode_color, mode_display)
    }

    /// 📊 Build center section of status line  
    fn build_status_center_section(&self) -> String {
        let mut info_parts = Vec::new();
        
        // Buffer position info
        if self.cursor_position > 0 {
            info_parts.push(format!("🔍 {}/{}", 
                self.cursor_position, 
                self.input_buffer.len()
            ));
        }
        
        // History info
        if let Some(hist_idx) = self.history_index {
            info_parts.push(format!("📜 {}/{}", 
                hist_idx + 1, 
                self.command_history.len()
            ));
        }
        
        // Completion info
        if self.completion_visible {
            if let Some(comps) = &self.current_completions {
                info_parts.push(format!("🔧 {}/{}", 
                    self.completion_index + 1, 
                    comps.items.len()
                ));
            }
        }
        
        if info_parts.is_empty() {
            String::from("🚀 Ready")
        } else {
            format!("\x1b[94m{}\x1b[0m", info_parts.join(" • "))
        }
    }

    /// ⚡ Build right section of status line
    fn build_status_right_section(&self) -> String {
        let mut right_parts = Vec::new();
        
        // Performance indicator
        if let Some(avg_time) = self.get_average_frame_time() {
            let perf_icon = if avg_time.as_millis() > 50 {
                "\x1b[91m🐌\x1b[0m" // Slow
            } else if avg_time.as_millis() < 16 {
                "\x1b[92m⚡\x1b[0m" // Fast
            } else {
                "\x1b[93m⚖️\x1b[0m" // Normal
            };
            right_parts.push(format!("{} {}ms", perf_icon, avg_time.as_millis()));
        }
        
        // Terminal size
        right_parts.push(format!("📐 {}×{}", 
            self.terminal_size.0, 
            self.terminal_size.1
        ));
        
        format!(" {} ", right_parts.join(" "))
    }

    /// Get current mode display string
    fn get_current_mode_display(&self) -> &str {
        match self.state {
            AppState::Normal => "NORMAL",
            AppState::Completing => "COMPLETING",
            AppState::Scrolling => "SCROLLING",
            AppState::Searching => "SEARCHING",
            AppState::CommandMode => "COMMAND",
            AppState::VisualMode => "VISUAL",
            AppState::InputMode => "INPUT",
            AppState::Exiting => "EXITING",
        }
    }

    /// Get color code for current mode
    fn get_mode_color(&self) -> &str {
        match self.state {
            AppState::Normal => "92", // Green
            AppState::Completing => "94", // Blue
            AppState::Scrolling => "96", // Cyan
            AppState::Searching => "93", // Yellow
            AppState::CommandMode => "95", // Magenta
            AppState::VisualMode => "91", // Red
            AppState::InputMode => "97", // White
            AppState::Exiting => "90", // Gray
        }
    }

    /// 🎨 Advanced syntax highlighting for output lines
    fn render_syntax_highlighted_line(&mut self, line: &str) -> Result<()> {
        // Apply context-aware highlighting
        if line.starts_with("$") || line.starts_with(">") {
            // Command line - highlight as shell command
            print!("\x1b[90m{}\x1b[0m", line); // Gray for prompts
        } else if line.contains("http://") || line.contains("https://") {
            // URLs - blue and underlined
            let url_regex = regex::Regex::new(r"(https?://[^\s]+)").unwrap();
            let highlighted = url_regex.replace_all(line, "\x1b[94;4m$1\x1b[0m");
            print!("{}", highlighted);
        } else if line.contains("Error") || line.contains("error") || line.contains("ERROR") {
            // Errors - red background
            print!("\x1b[101;97m🚨 {}\x1b[0m", line);
        } else if line.contains("Warning") || line.contains("warning") || line.contains("WARN") {
            // Warnings - yellow background
            print!("\x1b[103;30m⚠️  {}\x1b[0m", line);
        } else if line.contains("Success") || line.contains("success") || line.contains("OK") {
            // Success - green background
            print!("\x1b[102;30m✅ {}\x1b[0m", line);
        } else if line.contains("Debug") || line.contains("debug") || line.contains("DEBUG") {
            // Debug - cyan text
            print!("\x1b[96m🔍 {}\x1b[0m", line);
        } else {
            // Regular text with subtle styling
            print!("\x1b[37m{}\x1b[0m", line);
        }
        
        Ok(())
    }
            AppState::InputMode => "INPUT",
            AppState::Exiting => "EXITING",
        };

        let cols = self.terminal_size.0 as usize;
        if cols == 0 {
            return Ok(()); // Terminal too narrow
        }
        
        // Create status information with more details
        let avg_frame_time = if !self.frame_times.is_empty() {
            let sum: Duration = self.frame_times.iter().sum();
            sum.as_millis() / self.frame_times.len() as u128
        } else {
            0
        };
        
        let info = format!(
            "[{}] len:{} cur:{} hist:{} fps:{:.1}", 
            mode, 
            self.input_buffer.len(), 
            self.cursor_position,
            self.command_history.len(),
            if avg_frame_time > 0 { 1000.0 / avg_frame_time as f64 } else { 0.0 }
        );
        
        // Safely truncate and display
        if info.len() >= cols {
            let truncated = if cols > 3 {
                format!("{}...", &info[..cols.saturating_sub(3)])
            } else {
                String::new()
            };
            print!("{}", truncated);
        } else {
            print!("{}{}", info, " ".repeat(cols - info.len()));
        }
        
        Ok(())
    }

    /// Render an enhanced completion panel with advanced visual design
    fn render_completion_panel(&mut self) -> Result<()> {
        // Safety check for terminal size
        if self.terminal_size.1 < 6 {
            return Ok(()); // Terminal too small for completion panel
        }
        
        if !self.completion_visible {
            return Ok(());
        }

        let Some(comps) = &self.current_completions else {
            return Ok(());
        };

        let cols = self.terminal_size.0 as usize;
        if cols < 25 {
            return Ok(()); // Terminal too narrow for meaningful display
        }

        // Calculate panel dimensions dynamically
        let max_display_items = 8.min((self.terminal_size.1 as usize).saturating_sub(4).max(3));
        let total_items = comps.items.len();
        let panel_height = max_display_items.min(total_items) + 2; // +2 for header and border
        let panel_start_row = self.terminal_size.1.saturating_sub(panel_height as u16 + 2);
        
        // Calculate scroll offset to keep selected item visible
        let scroll_offset = if self.completion_index >= max_display_items {
            self.completion_index.saturating_sub(max_display_items - 1)
        } else {
            0
        };
        
        // Clear the panel area with smooth transition
        for i in 0..panel_height {
            let row = panel_start_row + i as u16;
            if row < self.terminal_size.1 {
                execute!(stdout(), cursor::MoveTo(0, row))?;
                execute!(stdout(), terminal::Clear(ClearType::CurrentLine))?;
            }
        }

        // Enhanced header with progress indication
        execute!(stdout(), cursor::MoveTo(0, panel_start_row))?;
        let progress = if total_items > 0 { 
            format!(" ({}/{}) ", self.completion_index + 1, total_items) 
        } else { 
            " ".to_string() 
        };
        let header = format!("┌─ 📋 Completions{}", progress);
        let header_padding = cols.saturating_sub(header.len() + 1);
        print!("\x1b[96;1m{}{}\x1b[0m", header, "─".repeat(header_padding.min(cols.saturating_sub(header.len()))));
        if cols > header.len() {
            print!("\x1b[96;1m┐\x1b[0m");
        }

        // Render completion items with enhanced styling
        let visible_items = comps.items.iter()
            .skip(scroll_offset)
            .take(max_display_items)
            .enumerate();

        for (display_index, item) in visible_items {
            let row = panel_start_row + 1 + display_index as u16;
            let actual_index = scroll_offset + display_index;
            execute!(stdout(), cursor::MoveTo(0, row))?;
            
            // Enhanced icons with better categorization
            let (icon, color) = match item.completion_type {
                crate::completion_engine::CompletionType::Command => ("⚡", "\x1b[93m"), // Bright yellow
                crate::completion_engine::CompletionType::Builtin => ("🔧", "\x1b[94m"), // Bright blue
                crate::completion_engine::CompletionType::File => ("📄", "\x1b[92m"), // Bright green
                crate::completion_engine::CompletionType::Directory => ("📁", "\x1b[96m"), // Bright cyan
                crate::completion_engine::CompletionType::Variable => ("🔤", "\x1b[95m"), // Bright magenta
                crate::completion_engine::CompletionType::Alias => ("🔗", "\x1b[91m"), // Bright red
                crate::completion_engine::CompletionType::Option => ("⚙️", "\x1b[90m"), // Bright black
                crate::completion_engine::CompletionType::History => ("🕐", "\x1b[37m"), // White
                _ => ("•", "\x1b[97m"), // Bright white
            };

            // Dynamic width calculation for better layout
            let max_text_width = (cols * 40 / 100).max(15).min(35); // 40% of terminal width
            let display_text = if item.text.len() > max_text_width {
                format!("{}…", &item.text[..max_text_width.saturating_sub(1)])
            } else {
                item.text.clone()
            };

            // Enhanced description with smart truncation
            let description = item.description.as_deref().unwrap_or("");
            let max_desc_width = cols.saturating_sub(max_text_width + 12).max(10);
            let display_desc = if description.len() > max_desc_width {
                format!("{}…", &description[..max_desc_width.saturating_sub(1)])
            } else {
                description.to_string()
            };

            // Render with enhanced visual effects
            if actual_index == self.completion_index {
                // Selected item with gradient-like effect
                print!("\x1b[96;1m│\x1b[0m \x1b[7;1m{}{} {:<width$}\x1b[0m \x1b[37;1m{}\x1b[0m", 
                       icon, color, display_text, display_desc, width = max_text_width);
            } else {
                // Normal item with subtle highlighting
                print!("\x1b[96m│\x1b[0m {}{} \x1b[32m{:<width$}\x1b[0m \x1b[90m{}\x1b[0m", 
                       icon, color, display_text, display_desc, width = max_text_width);
            }

            // Fill remaining space with consistent border
            let content_width = max_text_width + display_desc.len() + 8;
            let padding = cols.saturating_sub(content_width);
            print!("{} \x1b[96m│\x1b[0m", " ".repeat(padding.min(cols.saturating_sub(content_width))));
        }

        // Enhanced footer with smart scroll indicators
        let displayed_items = max_display_items.min(total_items);
        let row = panel_start_row + displayed_items as u16 + 1;
        execute!(stdout(), cursor::MoveTo(0, row))?;
        
        if scroll_offset > 0 || total_items > scroll_offset + max_display_items {
            // Animated scroll indicators
            let scroll_info = if scroll_offset > 0 && total_items > scroll_offset + max_display_items {
                format!("└─ ⬆️  {} more above • {} more below ⬇️ ", 
                       scroll_offset, 
                       total_items - scroll_offset - max_display_items)
            } else if scroll_offset > 0 {
                format!("└─ ⬆️  {} more above ", scroll_offset)
            } else {
                format!("└─ {} more below ⬇️ ", total_items - max_display_items)
            };
            let footer_padding = cols.saturating_sub(scroll_info.len() + 1);
            print!("\x1b[96;1m{}{}\x1b[0m", scroll_info, "─".repeat(footer_padding.min(cols.saturating_sub(scroll_info.len()))));
        } else {
            // Simple footer for complete view
            let footer = "└─ Use ↑↓ or Tab to navigate, Enter to select, Esc to cancel ";
            let footer_padding = cols.saturating_sub(footer.len() + 1);
            print!("\x1b[96m{}{}\x1b[0m", footer, "─".repeat(footer_padding.min(cols.saturating_sub(footer.len()))));
        }
        
        if cols > 1 {
            print!("\x1b[96;1m┘\x1b[0m");
        }

        Ok(())
    }