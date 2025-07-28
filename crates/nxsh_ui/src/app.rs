use anyhow::{Result, Context};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    io,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

use crate::{
    config::{ConfigManager, NexusConfig},
    themes::{NexusTheme, ThemeManager},
    line_editor::NexusLineEditor,
    completion::NexusCompleter,
    highlighting::RealtimeHighlighter,
};
use nxsh_core::{context::ShellContext, executor::CommandExecutor};
use nxsh_builtins::BuiltinRegistry;

/// Main application state
pub struct App {
    /// Application configuration
    config_manager: ConfigManager,
    
    /// Theme manager
    theme_manager: ThemeManager,
    
    /// Line editor with syntax highlighting and completion
    line_editor: Arc<Mutex<NexusLineEditor>>,
    
    /// Syntax highlighter
    highlighter: Arc<Mutex<RealtimeHighlighter>>,
    
    /// Command completer
    completer: Arc<Mutex<NexusCompleter>>,
    
    /// Shell context
    shell_context: Arc<Mutex<ShellContext>>,
    
    /// Command executor
    executor: Arc<Mutex<CommandExecutor>>,
    
    /// Builtin command registry
    builtin_registry: Arc<BuiltinRegistry>,
    
    /// Application state
    state: AppState,
    
    /// Command history
    command_history: Vec<String>,
    
    /// Output history
    output_history: Vec<OutputEntry>,
    
    /// Current input buffer
    input_buffer: String,
    
    /// Cursor position in input
    cursor_position: usize,
    
    /// Scroll position for output
    scroll_position: usize,
    
    /// Whether the app should quit
    should_quit: bool,
    
    /// Status message
    status_message: Option<StatusMessage>,
    
    /// Performance metrics
    metrics: AppMetrics,
}

/// Application state
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Normal,
    InputMode,
    CompletionMode,
    HistoryMode,
    ConfigMode,
    ThemeMode,
    HelpMode,
}

/// Output entry for command results
#[derive(Debug, Clone)]
pub struct OutputEntry {
    pub command: String,
    pub output: String,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub execution_time: Duration,
}

/// Status message
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub message: String,
    pub level: MessageLevel,
    pub timestamp: Instant,
}

/// Message level for status messages
#[derive(Debug, Clone, PartialEq)]
pub enum MessageLevel {
    Info,
    Warning,
    Error,
    Success,
}

/// Application performance metrics
#[derive(Debug, Clone, Default)]
pub struct AppMetrics {
    pub commands_executed: u64,
    pub total_execution_time: Duration,
    pub average_execution_time: Duration,
    pub last_execution_time: Duration,
    pub memory_usage: u64,
    pub startup_time: Duration,
}

impl App {
    /// Create a new application instance
    pub async fn new() -> Result<Self> {
        let startup_start = Instant::now();
        
        // Initialize configuration
        let config_manager = ConfigManager::new()
            .context("Failed to initialize configuration manager")?;
        
        // Initialize theme manager
        let theme_manager = ThemeManager::new()
            .context("Failed to initialize theme manager")?;
        
        // Initialize syntax highlighter
        let highlighter = Arc::new(Mutex::new(
            RealtimeHighlighter::new()
                .context("Failed to initialize syntax highlighter")?
        ));
        
        // Initialize completer
        let completer = Arc::new(Mutex::new(
            NexusCompleter::new()
                .context("Failed to initialize completer")?
        ));
        
        // Initialize line editor
        let line_editor = Arc::new(Mutex::new(
            NexusLineEditor::new()
                .context("Failed to initialize line editor")?
        ));
        
        // Initialize shell context
        let shell_context = Arc::new(Mutex::new(
            ShellContext::new()
                .context("Failed to initialize shell context")?
        ));
        
        // Initialize command executor
        let executor = Arc::new(Mutex::new(
            CommandExecutor::new()
                .context("Failed to initialize command executor")?
        ));
        
        // Initialize builtin registry
        let builtin_registry = Arc::new(
            BuiltinRegistry::new()
                .context("Failed to initialize builtin registry")?
        );
        
        // Setup completion system
        {
            let mut completer = completer.lock().await;
            let context = shell_context.lock().await;
            completer.setup_shell_completion(&context);
        }
        
        let startup_time = startup_start.elapsed();
        
        Ok(Self {
            config_manager,
            theme_manager,
            line_editor,
            highlighter,
            completer,
            shell_context,
            executor,
            builtin_registry,
            state: AppState::Normal,
            command_history: Vec::new(),
            output_history: Vec::new(),
            input_buffer: String::new(),
            cursor_position: 0,
            scroll_position: 0,
            should_quit: false,
            status_message: None,
            metrics: AppMetrics {
                startup_time,
                ..Default::default()
            },
        })
    }
    
    /// Run the application
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        // Show welcome message
        self.show_status_message(
            "Welcome to NexusShell - Advanced Shell with Plugin Support".to_string(),
            MessageLevel::Info,
        );
        
        loop {
            // Draw the UI
            terminal.draw(|f| {
                if let Err(e) = self.draw(f) {
                    eprintln!("Failed to draw UI: {}", e);
                }
            })?;
            
            // Handle events
            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key).await?;
                }
            }
            
            // Update status messages
            self.update_status_messages();
            
            // Check if we should quit
            if self.should_quit {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Draw the application UI
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>) -> Result<()> {
        let size = f.size();
        
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),      // Output area
                Constraint::Length(3),   // Input area
                Constraint::Length(1),   // Status bar
            ])
            .split(size);
        
        // Draw output area
        self.draw_output_area(f, chunks[0])?;
        
        // Draw input area
        self.draw_input_area(f, chunks[1])?;
        
        // Draw status bar
        self.draw_status_bar(f, chunks[2])?;
        
        // Draw overlays based on state
        match self.state {
            AppState::CompletionMode => self.draw_completion_overlay(f, size)?,
            AppState::HistoryMode => self.draw_history_overlay(f, size)?,
            AppState::ConfigMode => self.draw_config_overlay(f, size)?,
            AppState::ThemeMode => self.draw_theme_overlay(f, size)?,
            AppState::HelpMode => self.draw_help_overlay(f, size)?,
            _ => {}
        }
        
        Ok(())
    }
    
    /// Draw the output area
    fn draw_output_area<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) -> Result<()> {
        let theme = self.theme_manager.current_theme();
        
        // Create output items
        let items: Vec<ListItem> = self.output_history
            .iter()
            .skip(self.scroll_position)
            .map(|entry| {
                let mut spans = vec![
                    Span::styled(
                        format!("[{}] ", entry.timestamp.format("%H:%M:%S")),
                        Style::default().fg(Color::Rgb(
                            theme.ui_colors.info.r,
                            theme.ui_colors.info.g,
                            theme.ui_colors.info.b,
                        )),
                    ),
                    Span::styled(
                        format!("$ {}", entry.command),
                        Style::default().fg(Color::Rgb(
                            theme.ui_colors.prompt.r,
                            theme.ui_colors.prompt.g,
                            theme.ui_colors.prompt.b,
                        )).add_modifier(Modifier::BOLD),
                    ),
                ];
                
                if !entry.output.is_empty() {
                    spans.push(Span::styled(
                        format!("\n{}", entry.output),
                        Style::default().fg(Color::Rgb(
                            theme.ui_colors.foreground.r,
                            theme.ui_colors.foreground.g,
                            theme.ui_colors.foreground.b,
                        )),
                    ));
                }
                
                if let Some(error) = &entry.error {
                    spans.push(Span::styled(
                        format!("\nError: {}", error),
                        Style::default().fg(Color::Rgb(
                            theme.ui_colors.error.r,
                            theme.ui_colors.error.g,
                            theme.ui_colors.error.b,
                        )),
                    ));
                }
                
                ListItem::new(Line::from(spans))
            })
            .collect();
        
        let output_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Output")
                    .border_style(Style::default().fg(Color::Rgb(
                        theme.ui_colors.border.r,
                        theme.ui_colors.border.g,
                        theme.ui_colors.border.b,
                    )))
            )
            .style(Style::default().bg(Color::Rgb(
                theme.ui_colors.background.r,
                theme.ui_colors.background.g,
                theme.ui_colors.background.b,
            )));
        
        f.render_widget(output_list, area);
        Ok(())
    }
    
    /// Draw the input area
    fn draw_input_area<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) -> Result<()> {
        let theme = self.theme_manager.current_theme();
        
        // Apply syntax highlighting to input
        let highlighted_text = if let Ok(mut highlighter) = self.highlighter.try_lock() {
            match highlighter.highlight_cached(&self.input_buffer) {
                Ok(spans) => Text::from(Line::from(spans)),
                Err(_) => Text::from(self.input_buffer.as_str()),
            }
        } else {
            Text::from(self.input_buffer.as_str())
        };
        
        let input_paragraph = Paragraph::new(highlighted_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command Input")
                    .border_style(Style::default().fg(Color::Rgb(
                        theme.ui_colors.border.r,
                        theme.ui_colors.border.g,
                        theme.ui_colors.border.b,
                    )))
            )
            .style(Style::default()
                .bg(Color::Rgb(
                    theme.ui_colors.background.r,
                    theme.ui_colors.background.g,
                    theme.ui_colors.background.b,
                ))
                .fg(Color::Rgb(
                    theme.ui_colors.input_text.r,
                    theme.ui_colors.input_text.g,
                    theme.ui_colors.input_text.b,
                ))
            )
            .wrap(Wrap { trim: true });
        
        f.render_widget(input_paragraph, area);
        
        // Set cursor position
        if self.state == AppState::InputMode {
            f.set_cursor(
                area.x + self.cursor_position as u16 + 1,
                area.y + 1,
            );
        }
        
        Ok(())
    }
    
    /// Draw the status bar
    fn draw_status_bar<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) -> Result<()> {
        let theme = self.theme_manager.current_theme();
        
        let status_text = if let Some(status) = &self.status_message {
            status.message.clone()
        } else {
            format!(
                "Commands: {} | Avg Time: {:?} | Memory: {} MB | Theme: {}",
                self.metrics.commands_executed,
                self.metrics.average_execution_time,
                self.metrics.memory_usage / (1024 * 1024),
                theme.name
            )
        };
        
        let status_color = if let Some(status) = &self.status_message {
            match status.level {
                MessageLevel::Info => Color::Rgb(
                    theme.ui_colors.info.r,
                    theme.ui_colors.info.g,
                    theme.ui_colors.info.b,
                ),
                MessageLevel::Warning => Color::Rgb(
                    theme.ui_colors.warning.r,
                    theme.ui_colors.warning.g,
                    theme.ui_colors.warning.b,
                ),
                MessageLevel::Error => Color::Rgb(
                    theme.ui_colors.error.r,
                    theme.ui_colors.error.g,
                    theme.ui_colors.error.b,
                ),
                MessageLevel::Success => Color::Rgb(
                    theme.ui_colors.success.r,
                    theme.ui_colors.success.g,
                    theme.ui_colors.success.b,
                ),
            }
        } else {
            Color::Rgb(
                theme.ui_colors.status_bar_fg.r,
                theme.ui_colors.status_bar_fg.g,
                theme.ui_colors.status_bar_fg.b,
            )
        };
        
        let status_paragraph = Paragraph::new(status_text)
            .style(Style::default()
                .bg(Color::Rgb(
                    theme.ui_colors.status_bar_bg.r,
                    theme.ui_colors.status_bar_bg.g,
                    theme.ui_colors.status_bar_bg.b,
                ))
                .fg(status_color)
            );
        
        f.render_widget(status_paragraph, area);
        Ok(())
    }
    
    /// Draw completion overlay
    fn draw_completion_overlay<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) -> Result<()> {
        // TODO: Implement completion overlay
        Ok(())
    }
    
    /// Draw history overlay
    fn draw_history_overlay<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) -> Result<()> {
        // TODO: Implement history overlay
        Ok(())
    }
    
    /// Draw configuration overlay
    fn draw_config_overlay<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) -> Result<()> {
        // TODO: Implement configuration overlay
        Ok(())
    }
    
    /// Draw theme overlay
    fn draw_theme_overlay<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) -> Result<()> {
        // TODO: Implement theme overlay
        Ok(())
    }
    
    /// Draw help overlay
    fn draw_help_overlay<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) -> Result<()> {
        // TODO: Implement help overlay
        Ok(())
    }
    
    /// Handle key events
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match self.state {
            AppState::Normal => self.handle_normal_mode_key(key).await?,
            AppState::InputMode => self.handle_input_mode_key(key).await?,
            AppState::CompletionMode => self.handle_completion_mode_key(key).await?,
            AppState::HistoryMode => self.handle_history_mode_key(key).await?,
            AppState::ConfigMode => self.handle_config_mode_key(key).await?,
            AppState::ThemeMode => self.handle_theme_mode_key(key).await?,
            AppState::HelpMode => self.handle_help_mode_key(key).await?,
        }
        Ok(())
    }
    
    /// Handle keys in normal mode
    async fn handle_normal_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            (KeyCode::Char('i'), KeyModifiers::NONE) => {
                self.state = AppState::InputMode;
            }
            (KeyCode::Char('h'), KeyModifiers::NONE) => {
                self.state = AppState::HelpMode;
            }
            (KeyCode::Char('t'), KeyModifiers::NONE) => {
                self.state = AppState::ThemeMode;
            }
            (KeyCode::Char('c'), KeyModifiers::NONE) => {
                self.state = AppState::ConfigMode;
            }
            (KeyCode::Up, KeyModifiers::NONE) => {
                if self.scroll_position > 0 {
                    self.scroll_position -= 1;
                }
            }
            (KeyCode::Down, KeyModifiers::NONE) => {
                if self.scroll_position < self.output_history.len().saturating_sub(1) {
                    self.scroll_position += 1;
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Handle keys in input mode
    async fn handle_input_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.state = AppState::Normal;
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.execute_command().await?;
            }
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.state = AppState::CompletionMode;
                // TODO: Show completions
            }
            (KeyCode::Up, KeyModifiers::NONE) => {
                self.state = AppState::HistoryMode;
                // TODO: Show history
            }
            (KeyCode::Char(c), KeyModifiers::NONE) => {
                self.input_buffer.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                if self.cursor_position > 0 {
                    self.input_buffer.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
            }
            (KeyCode::Left, KeyModifiers::NONE) => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            (KeyCode::Right, KeyModifiers::NONE) => {
                if self.cursor_position < self.input_buffer.len() {
                    self.cursor_position += 1;
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Handle keys in completion mode
    async fn handle_completion_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::InputMode;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Handle keys in history mode
    async fn handle_history_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::InputMode;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Handle keys in config mode
    async fn handle_config_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::Normal;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Handle keys in theme mode
    async fn handle_theme_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::Normal;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Handle keys in help mode
    async fn handle_help_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::Normal;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Execute the current command
    async fn execute_command(&mut self) -> Result<()> {
        if self.input_buffer.trim().is_empty() {
            return Ok(());
        }
        
        let command = self.input_buffer.clone();
        let start_time = Instant::now();
        
        // Add to history
        self.command_history.push(command.clone());
        
        // Execute command
        let result = {
            let mut executor = self.executor.lock().await;
            let mut context = self.shell_context.lock().await;
            executor.execute(&command, &mut context).await
        };
        
        let execution_time = start_time.elapsed();
        
        // Update metrics
        self.metrics.commands_executed += 1;
        self.metrics.total_execution_time += execution_time;
        self.metrics.average_execution_time = 
            self.metrics.total_execution_time / self.metrics.commands_executed as u32;
        self.metrics.last_execution_time = execution_time;
        
        // Create output entry
        let output_entry = match result {
            Ok(output) => OutputEntry {
                command: command.clone(),
                output: output.stdout,
                error: if output.stderr.is_empty() { None } else { Some(output.stderr) },
                timestamp: chrono::Utc::now(),
                execution_time,
            },
            Err(error) => OutputEntry {
                command: command.clone(),
                output: String::new(),
                error: Some(error.to_string()),
                timestamp: chrono::Utc::now(),
                execution_time,
            },
        };
        
        self.output_history.push(output_entry);
        
        // Clear input
        self.input_buffer.clear();
        self.cursor_position = 0;
        
        // Return to normal mode
        self.state = AppState::Normal;
        
        // Show success message
        self.show_status_message(
            format!("Command executed in {:?}", execution_time),
            MessageLevel::Success,
        );
        
        Ok(())
    }
    
    /// Show a status message
    fn show_status_message(&mut self, message: String, level: MessageLevel) {
        self.status_message = Some(StatusMessage {
            message,
            level,
            timestamp: Instant::now(),
        });
    }
    
    /// Update status messages (clear old ones)
    fn update_status_messages(&mut self) {
        if let Some(status) = &self.status_message {
            if status.timestamp.elapsed() > Duration::from_secs(3) {
                self.status_message = None;
            }
        }
    }
    
    /// Get application configuration
    pub fn config(&self) -> &NexusConfig {
        self.config_manager.config()
    }
    
    /// Update application configuration
    pub async fn update_config(&mut self, config: NexusConfig) -> Result<()> {
        self.config_manager.update_config(config).await
            .context("Failed to update configuration")?;
        
        self.show_status_message(
            "Configuration updated successfully".to_string(),
            MessageLevel::Success,
        );
        
        Ok(())
    }
    
    /// Switch theme
    pub async fn switch_theme(&mut self, theme_name: &str) -> Result<()> {
        self.theme_manager.switch_theme(theme_name).await
            .context("Failed to switch theme")?;
        
        self.show_status_message(
            format!("Switched to theme: {}", theme_name),
            MessageLevel::Success,
        );
        
        Ok(())
    }
    
    /// Get application metrics
    pub fn metrics(&self) -> &AppMetrics {
        &self.metrics
    }
} 