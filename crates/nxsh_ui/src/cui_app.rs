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

use crate::config::UiConfig;
use crate::themes::{Theme, get_theme};
use crate::completion::{CompletionEngine, CompletionResult};
use crate::completion_integration::{IntegratedCompletionSystem, ShellStateProvider};
use crate::prompt::{Prompt, PromptStyle, StatusInfo};
use crate::scroll_buffer::ScrollBuffer;
use crate::ansi_render::AnsiRenderer;

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
    prompt: Prompt,
    completion_system: IntegratedCompletionSystem,
    scroll_buffer: ScrollBuffer,
    ansi_renderer: AnsiRenderer,
    
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
        let prompt = Prompt::new(config.prompt.clone());
        let completion_system = IntegratedCompletionSystem::new(config.clone());
        let scroll_buffer = ScrollBuffer::new(config.scroll_buffer_size);
        let ansi_renderer = AnsiRenderer::new();
        let terminal_size = terminal::size().unwrap_or((80, 24));
        
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            config,
            theme,
            state: AppState::Normal,
            prompt,
            completion_system,
            scroll_buffer,
            ansi_renderer,
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
                    self.terminal_size = (cols, rows);
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

    /// Insert character
    fn insert_char(&mut self, c: char) {
        self.input_buffer.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// 後方削除
    fn delete_backward(&mut self) {
        if self.cursor_position > 0 {
            self.input_buffer.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    /// 前方削除
    fn delete_forward(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_position);
        }
    }

    /// 単語後方削除
    fn delete_word_backward(&mut self) {
        let mut pos = self.cursor_position;
        while pos > 0 && self.input_buffer.chars().nth(pos - 1).unwrap_or(' ').is_whitespace() {
            pos -= 1;
        }
        while pos > 0 && !self.input_buffer.chars().nth(pos - 1).unwrap_or(' ').is_whitespace() {
            pos -= 1;
        }
        self.input_buffer.drain(pos..self.cursor_position);
        self.cursor_position = pos;
    }

    /// カーソル移動
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

    fn apply_completion(&mut self) {
        if let Some(completions) = &self.current_completions {
            if let Some(item) = completions.items.get(self.completion_index) {
                // Replace the current word with the completion
                let prefix_len = completions.prefix.len();
                let start_pos = self.cursor_position.saturating_sub(prefix_len);
                
                self.input_buffer.drain(start_pos..self.cursor_position);
                self.input_buffer.insert_str(start_pos, &item.text);
                self.cursor_position = start_pos + item.text.len();
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

    /// レンダリング
    fn render(&mut self) -> Result<()> {
        let start_time = Instant::now();

        execute!(stdout(), cursor::MoveTo(0, 0))?;

        // Render output buffer
        self.render_output_buffer()?;

        // Render prompt and input
        self.render_prompt_and_input()?;

        // Render completion panel if visible
        if self.completion_visible {
            self.render_completion_panel()?;
        }

        // Render status line
        self.render_status_line()?;

        stdout().flush()?;

        // Update performance metrics
        let render_time = start_time.elapsed();
        self.frame_times.push_back(render_time);
        if self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }
        self.last_render_time = start_time;

        Ok(())
    }

    fn render_output_buffer(&mut self) -> Result<()> {
        let available_rows = self.terminal_size.1.saturating_sub(3); // Reserve space for prompt and status
        let lines = self.scroll_buffer.get_visible_lines(available_rows as usize);

        for (i, line) in lines.iter().enumerate() {
            execute!(stdout(), cursor::MoveTo(0, i as u16))?;
            let rendered = self.ansi_renderer.render(line, &self.theme)?;
            print!("{}", rendered);
            execute!(stdout(), terminal::Clear(ClearType::UntilNewLine))?;
        }

        Ok(())
    }

    fn render_prompt_and_input(&mut self) -> Result<()> {
        let prompt_row = self.terminal_size.1.saturating_sub(2);
        execute!(stdout(), cursor::MoveTo(0, prompt_row))?;

        // Render prompt
        let prompt_text = self.prompt.render(&self.theme)?;
        print!("{}", prompt_text);

        // Render input
        print!("{}", self.input_buffer);

        // Position cursor
        let prompt_len = self.prompt.get_display_length();
        execute!(stdout(), cursor::MoveTo((prompt_len + self.cursor_position) as u16, prompt_row))?;

        Ok(())
    }

    fn render_completion_panel(&mut self) -> Result<()> {
        if let Some(completions) = &self.current_completions {
            if !completions.items.is_empty() {
                let panel_row = self.terminal_size.1.saturating_sub(3);
                execute!(stdout(), cursor::MoveTo(0, panel_row))?;

                // Show current completion
                if let Some(item) = completions.items.get(self.completion_index) {
                    let text = format!(
                        "[{}/{}] {} - {}",
                        self.completion_index + 1,
                        completions.items.len(),
                        item.text,
                        item.description.as_deref().unwrap_or("No description")
                    );
                    print!("{}", text.with(self.theme.colors.completion_highlight));
                }
            }
        }
        Ok(())
    }

    fn render_status_line(&mut self) -> Result<()> {
        let status_row = self.terminal_size.1.saturating_sub(1);
        execute!(stdout(), cursor::MoveTo(0, status_row))?;

        let status_text = format!(
            "State: {:?} | Size: {}x{} | FPS: {:.1}",
            self.state,
            self.terminal_size.0,
            self.terminal_size.1,
            self.calculate_fps()
        );

        print!("{}", status_text.with(self.theme.colors.status_bar));
        execute!(stdout(), terminal::Clear(ClearType::UntilNewLine))?;

        Ok(())
    }

    fn calculate_fps(&self) -> f64 {
        if self.frame_times.len() < 2 {
            return 0.0;
        }

        let total_time: Duration = self.frame_times.iter().sum();
        let avg_frame_time = total_time.as_secs_f64() / self.frame_times.len() as f64;
        
        if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        }
    }

    /// Update processing
    async fn update(&mut self) -> Result<()> {
        // Periodic update processing
        Ok(())
    }

    /// Execute command
    async fn execute_command(&mut self, command: &str) -> Result<()> {
        // Command execution implementation
        self.add_output(&format!("> {}", command));
        Ok(())
    }

    /// Load configuration
    fn load_history(&mut self) -> Result<()> {
        // Load from history file
        Ok(())
    }

    fn setup_completion_system(&mut self) -> Result<()> {
        // Completion system configuration
        Ok(())
    }

    /// Shutdown
    async fn shutdown(&mut self) -> Result<()> {
        if self.is_raw_mode {
            execute!(
                stdout(),
                cursor::Show,
                event::DisableMouseCapture,
                terminal::LeaveAlternateScreen
            )?;
            disable_raw_mode()?;
            self.is_raw_mode = false;
        }
        Ok(())
    }
}

impl Drop for CuiApp {
    fn drop(&mut self) {
        if self.is_raw_mode {
            let _ = execute!(
                stdout(),
                cursor::Show,
                event::DisableMouseCapture,
                terminal::LeaveAlternateScreen
            );
            let _ = disable_raw_mode();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cui_app_creation() {
        let config = UiConfig::default();
        let app = CuiApp::new(config);
        assert!(app.is_ok());
    }

    #[test]
    fn test_input_handling() {
        let config = UiConfig::default();
        let mut app = CuiApp::new(config).unwrap();

        app.insert_char('h');
        app.insert_char('e');
        app.insert_char('l');
        app.insert_char('l');
        app.insert_char('o');

        assert_eq!(app.input_buffer, "hello");
        assert_eq!(app.cursor_position, 5);
    }

    #[test]
    fn test_cursor_movement() {
        let config = UiConfig::default();
        let mut app = CuiApp::new(config).unwrap();

        app.input_buffer = "hello world".to_string();
        app.cursor_position = 11;

        app.move_cursor_left();
        assert_eq!(app.cursor_position, 10);

        app.move_cursor_right();
        assert_eq!(app.cursor_position, 11);
    }

    #[test]
    fn test_history_management() {
        let config = UiConfig::default();
        let mut app = CuiApp::new(config).unwrap();

        app.add_to_history("ls -la".to_string());
        app.add_to_history("cd /tmp".to_string());

        assert_eq!(app.command_history.len(), 2);

        app.history_previous();
        assert_eq!(app.input_buffer, "cd /tmp");

        app.history_previous();
        assert_eq!(app.input_buffer, "ls -la");
    }
}
