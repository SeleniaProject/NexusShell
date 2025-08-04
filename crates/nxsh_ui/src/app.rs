use anyhow::{Result, Context};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

use crate::{
    config::{ConfigManager, NexusConfig},
    themes::ThemeManager,
    line_editor::NexusLineEditor,
    completion::NexusCompleter,
    highlighting::RealtimeHighlighter,
};
use nxsh_core::{context::ShellContext, executor::Executor};
// use nxsh_builtins::BuiltinRegistry;  // Temporarily disabled

/// Maximum frames per second for the TUI
pub const MAX_FPS: u64 = 60;

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
    executor: Arc<Mutex<Executor>>,
    
    /// Builtin command registry
    // builtin_registry: Arc<BuiltinRegistry>,  // Temporarily disabled
    
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
    
    /// Completion suggestions for overlay display
    completion_suggestions: Vec<String>,
    
    /// History entries for overlay display  
    history_entries: Vec<HistoryEntry>,
    
    /// Current selection in history mode
    history_selection: usize,
    
    /// History filter for search
    history_filter: String,
}

/// Application mode
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    InputMode,
    CompletionMode,
    HistoryMode,
    HistorySearchMode,
    ConfigMode,
    ThemeMode,
    ThemeSelection,
    HelpMode,
}

impl Default for AppMode {
    fn default() -> Self {
        AppMode::Normal
    }
}

/// Application state with input and UI state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current application mode
    pub mode: AppMode,
    /// Current input buffer
    pub input: String,
    /// Scroll history for output
    pub toasts: Vec<String>,
    /// Completion system state
    pub completion_selected_index: usize,
    /// History system state
    pub history_selected_index: usize,
    /// Theme selection state
    pub theme_selection_index: usize,
    /// Help overlay scroll position
    pub help_scroll_offset: u16,
}

impl AppState {
    /// Render the current application state
    pub fn render<B: Backend>(&self, f: &mut Frame, area: Rect) {
        // Implement basic rendering logic here
        let paragraph = Paragraph::new(format!("Mode: {:?}\nInput: {}", self.mode, self.input))
            .block(Block::default().borders(Borders::ALL).title("NexusShell"));
        f.render_widget(paragraph, area);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: AppMode::default(),
            input: String::new(),
            toasts: Vec::new(),
            completion_selected_index: 0,
            history_selected_index: 0,
            theme_selection_index: 0,
            help_scroll_offset: 0,
        }
    }
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

/// History entry for command history overlay
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub success: bool,
    pub execution_time: Option<Duration>,
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

/// Alias for MessageLevel
pub type MessageType = MessageLevel;

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
    /// Render the application (compatibility method)
    pub fn render<B: Backend>(&mut self, f: &mut Frame) -> Result<()> {
        self.draw(f)
    }
    
    /// Handle key event (compatibility method)
    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        self.handle_key_event(key).await
    }
    
    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

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
            ShellContext::default()
        ));
        
        // Initialize command executor
        let executor = Arc::new(Mutex::new(
            Executor::default()
        ));
        
        // Initialize builtin registry
        // let builtin_registry = Arc::new(
        //     BuiltinRegistry::default()
        // );  // Temporarily disabled
        
        // TODO: Setup completion system
        // Setup completion system with comprehensive shell features
        {
            let mut completer = completer.lock().await;
            let context = shell_context.lock().await;
            completer.setup_shell_completion(&context);
            
            // Add built-in command completion
            completer.add_builtin_commands(&[
                "cd", "ls", "pwd", "echo", "cat", "grep", "find", "which",
                "export", "unset", "alias", "unalias", "history", "jobs",
                "bg", "fg", "kill", "ps", "top", "df", "du", "free",
                "chmod", "chown", "chgrp", "ln", "cp", "mv", "rm", "mkdir",
                "rmdir", "touch", "head", "tail", "sort", "uniq", "wc",
                "cut", "sed", "awk", "tar", "gzip", "gunzip", "zip", "unzip"
            ]);
            
            // Add shell keywords completion
            completer.add_keywords(&[
                "if", "then", "else", "elif", "fi", "case", "esac",
                "for", "while", "until", "do", "done", "function",
                "select", "time", "coproc", "declare", "local", "readonly",
                "typeset", "export", "unset", "return", "break", "continue"
            ]);
            
            // Setup path completion
            completer.enable_path_completion(true);
            completer.enable_variable_completion(true);
            completer.enable_history_completion(true);
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
            // builtin_registry,  // Temporarily disabled
            state: AppState {
                mode: AppMode::Normal,
                input: String::new(),
                toasts: Vec::new(),
                completion_selected_index: 0,
                history_selected_index: 0,
                theme_selection_index: 0,
                help_scroll_offset: 0,
            },
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
            completion_suggestions: Vec::new(),
            history_entries: Vec::new(),
            history_selection: 0,
            history_filter: String::new(),
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
    fn draw(&mut self, f: &mut Frame) -> Result<()> {
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
        match self.state.mode {
            AppMode::CompletionMode => self.draw_completion_overlay(f, size)?,
            AppMode::HistoryMode => self.draw_history_overlay(f, size)?,
            AppMode::ConfigMode => self.draw_config_overlay(f, size)?,
            AppMode::ThemeMode => self.draw_theme_overlay(f, size)?,
            AppMode::HelpMode => self.draw_help_overlay(f, size)?,
            _ => {}
        }
        
        Ok(())
    }
    
    /// Draw the output area
    fn draw_output_area(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
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
    fn draw_input_area(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
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
        if self.state.mode == AppMode::InputMode {
            f.set_cursor(
                area.x + self.cursor_position as u16 + 1,
                area.y + 1,
            );
        }
        
        Ok(())
    }
    
    /// Draw the status bar
    fn draw_status_bar(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
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
    fn draw_completion_overlay(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        // Only draw if in completion mode and have suggestions
        if !matches!(self.state.mode, AppMode::CompletionMode) || self.completion_suggestions.is_empty() {
            return Ok(());
        }

        // Calculate overlay size - show up to 10 suggestions
        let max_suggestions = 10.min(self.completion_suggestions.len());
        let overlay_height = (max_suggestions + 2) as u16; // +2 for borders
        let overlay_width = area.width.saturating_sub(4).max(40); // Leave some margin
        
        // Position overlay below the input line if possible, otherwise above
        let input_line_y = area.height.saturating_sub(3); // Input is at bottom-2
        let overlay_y = if input_line_y + overlay_height < area.height {
            input_line_y + 1
        } else {
            input_line_y.saturating_sub(overlay_height)
        };
        
        let overlay_area = Rect {
            x: 2,
            y: overlay_y,
            width: overlay_width,
            height: overlay_height,
        };

        // Get current theme for styling
        let theme = self.theme_manager.current_theme();
        
        // Create completion list items
        let completion_items: Vec<ListItem> = self.completion_suggestions
            .iter()
            .take(max_suggestions)
            .enumerate()
            .map(|(index, suggestion)| {
                let style = if index == self.state.completion_selected_index {
                    // Highlighted selection style
                    Style::default()
                        .bg(Color::Rgb(
                            theme.ui_colors.selection.r,
                            theme.ui_colors.selection.g, 
                            theme.ui_colors.selection.b,
                        ))
                        .fg(Color::Rgb(
                            theme.ui_colors.primary.r,
                            theme.ui_colors.primary.g,
                            theme.ui_colors.primary.b,
                        ))
                        .add_modifier(Modifier::BOLD)
                } else {
                    // Normal item style
                    Style::default()
                        .fg(Color::Rgb(
                            theme.ui_colors.foreground.r,
                            theme.ui_colors.foreground.g,
                            theme.ui_colors.foreground.b,
                        ))
                };

                // Format suggestion with description if available
                let text = format!("{}", suggestion);

                ListItem::new(text).style(style)
            })
            .collect();

        // Create completion list widget
        let completion_list = List::new(completion_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Completions ")
                    .title_style(Style::default().fg(Color::Rgb(
                        theme.ui_colors.accent.r,
                        theme.ui_colors.accent.g,
                        theme.ui_colors.accent.b,
                    )))
                    .border_style(Style::default().fg(Color::Rgb(
                        theme.ui_colors.border.r,
                        theme.ui_colors.border.g,
                        theme.ui_colors.border.b,
                    )))
            );

        // Clear area behind overlay for better visibility
        let clear_area = Block::default()
            .style(Style::default().bg(Color::Rgb(
                theme.ui_colors.background.r,
                theme.ui_colors.background.g,
                theme.ui_colors.background.b,
            )));
        f.render_widget(clear_area, overlay_area);
        
        // Render the completion list
        f.render_widget(completion_list, overlay_area);
        
        Ok(())
    }
    
    /// Draw history overlay
    fn draw_history_overlay(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        // Only draw if in history mode and have history entries
        if !matches!(self.state.mode, AppMode::HistoryMode) || self.history_entries.is_empty() {
            return Ok(());
        }

        // Calculate overlay size - show up to 15 history entries
        let max_entries = 15.min(self.history_entries.len());
        let overlay_height = (max_entries + 2) as u16; // +2 for borders
        let overlay_width = area.width.saturating_sub(4).max(60); // Wider for command history
        
        // Position overlay in the center-left area
        let overlay_y = (area.height.saturating_sub(overlay_height)) / 2;
        let overlay_x = 2;
        
        let overlay_area = Rect {
            x: overlay_x,
            y: overlay_y,
            width: overlay_width,
            height: overlay_height,
        };

        // Get current theme for styling
        let theme = self.theme_manager.current_theme();
        
        // Create history list items (reverse order - newest first)
        let history_items: Vec<ListItem> = self.history_entries
            .iter()
            .rev() // Show newest entries first
            .take(max_entries)
            .enumerate()
            .map(|(index, entry)| {
                let style = if index == self.state.history_selected_index {
                    // Highlighted selection style
                    Style::default()
                        .bg(Color::Rgb(
                            theme.ui_colors.selection.r,
                            theme.ui_colors.selection.g,
                            theme.ui_colors.selection.b,
                        ))
                        .fg(Color::Rgb(
                            theme.ui_colors.primary.r,
                            theme.ui_colors.primary.g,
                            theme.ui_colors.primary.b,
                        ))
                        .add_modifier(Modifier::BOLD)
                } else {
                    // Normal item style with alternating subtle background
                    let bg_color = if index % 2 == 0 {
                        Color::Rgb(
                            theme.ui_colors.background.r,
                            theme.ui_colors.background.g,
                            theme.ui_colors.background.b,
                        )
                    } else {
                        Color::Rgb(
                            (theme.ui_colors.background.r as u16 + 10).min(255) as u8,
                            (theme.ui_colors.background.g as u16 + 10).min(255) as u8,
                            (theme.ui_colors.background.b as u16 + 10).min(255) as u8,
                        )
                    };
                    
                    Style::default()
                        .bg(bg_color)
                        .fg(Color::Rgb(
                            theme.ui_colors.foreground.r,
                            theme.ui_colors.foreground.g,
                            theme.ui_colors.foreground.b,
                        ))
                };

                // Format history entry with timestamp
                let time_str = entry.timestamp.format("%H:%M").to_string();
                let text = format!("{:<6} {}", time_str, entry.command);

                // Truncate long commands for display
                let display_text = if text.len() > (overlay_width - 4) as usize {
                    format!("{}...", &text[..(overlay_width - 7) as usize])
                } else {
                    text
                };

                ListItem::new(display_text).style(style)
            })
            .collect();

        // Create history list widget
        let history_list = List::new(history_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Command History ")
                    .title_style(Style::default().fg(Color::Rgb(
                        theme.ui_colors.info.r,
                        theme.ui_colors.info.g,
                        theme.ui_colors.info.b,
                    )))
                    .border_style(Style::default().fg(Color::Rgb(
                        theme.ui_colors.border.r,
                        theme.ui_colors.border.g,
                        theme.ui_colors.border.b,
                    )))
            );

        // Clear area behind overlay
        let clear_area = Block::default()
            .style(Style::default().bg(Color::Rgb(
                theme.ui_colors.background.r,
                theme.ui_colors.background.g,
                theme.ui_colors.background.b,
            )));
        f.render_widget(clear_area, overlay_area);
        
        // Render the history list
        f.render_widget(history_list, overlay_area);
        
        // Add navigation hint at the bottom
        let hint_area = Rect {
            x: overlay_x,
            y: overlay_y + overlay_height,
            width: overlay_width,
            height: 1,
        };
        
        let hint_text = Paragraph::new("↑↓: Navigate, Enter: Select, Esc: Cancel")
            .style(Style::default().fg(Color::Rgb(
                theme.ui_colors.secondary.r,
                theme.ui_colors.secondary.g,
                theme.ui_colors.secondary.b,
            )));
        f.render_widget(hint_text, hint_area);
        
        Ok(())
    }
    
    /// Draw configuration overlay
    fn draw_config_overlay(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        // Only draw if in configuration mode
        if !matches!(self.state.mode, AppMode::ConfigMode) {
            return Ok(());
        }

        // Calculate overlay size for configuration panel
        let overlay_width = area.width.saturating_sub(10).max(50);
        let overlay_height = area.height.saturating_sub(6).max(20);
        
        // Center the overlay
        let overlay_x = (area.width.saturating_sub(overlay_width)) / 2;
        let overlay_y = (area.height.saturating_sub(overlay_height)) / 2;
        
        let overlay_area = Rect {
            x: overlay_x,
            y: overlay_y,
            width: overlay_width,
            height: overlay_height,
        };

        // Get current theme and configuration
        let theme = self.theme_manager.current_theme();
        let config = self.config_manager.config();
        
        // Create configuration sections
        let config_sections = vec![
            ("Editor", vec![
                format!("Vi mode: {}", config.editor.vi_mode),
                format!("Tab width: {}", config.editor.tab_width),
                format!("Auto-indent: {}", config.editor.auto_indent),
                format!("Show line numbers: {}", config.editor.show_line_numbers),
                format!("Max history size: {}", config.editor.max_history_size),
                format!("Auto completion: {}", config.editor.auto_completion),
            ]),
            ("Theme", vec![
                format!("Current theme: {}", config.theme.current_theme),
                format!("Auto-detect dark mode: {}", config.theme.auto_detect_dark_mode),
                format!("Syntax highlighting: {}", config.theme.syntax_highlighting),
                format!("Color output: {}", config.theme.color_output),
                format!("True color: {}", config.theme.true_color),
            ]),
            ("UI", vec![
                format!("Show status bar: {}", config.ui.show_status_bar),
                format!("Show suggestions: {}", config.ui.show_suggestions),
                format!("Max suggestions: {}", config.ui.max_suggestions),
                format!("Mouse support: {}", config.ui.mouse_support),
                format!("Auto scroll output: {}", config.ui.auto_scroll_output),
            ]),
            ("History", vec![
                format!("Max size: {}", config.history.max_size),
                format!("Save to file: {}", config.history.save_to_file),
                format!("Ignore duplicates: {}", config.history.ignore_duplicates),
                format!("Auto save: {}", config.history.auto_save),
            ]),
        ];

        // Create configuration text
        let mut config_text = Vec::new();
        
        for (section_name, items) in &config_sections {
            // Add section header
            config_text.push(Line::from(vec![
                Span::styled(
                    format!("┌─ {} ", section_name),
                    Style::default()
                        .fg(Color::Rgb(
                            theme.ui_colors.accent.r,
                            theme.ui_colors.accent.g,
                            theme.ui_colors.accent.b,
                        ))
                        .add_modifier(Modifier::BOLD)
                )
            ]));
            
            // Add configuration items
            for item in items {
                config_text.push(Line::from(vec![
                    Span::styled(
                        format!("│ {}", item),
                        Style::default().fg(Color::Rgb(
                            theme.ui_colors.secondary.r,
                            theme.ui_colors.secondary.g,
                            theme.ui_colors.secondary.b,
                        ))
                    )
                ]));
            }
            
            // Add section separator
            config_text.push(Line::from(vec![
                Span::styled(
                    "│",
                    Style::default().fg(Color::Rgb(
                        theme.ui_colors.accent.r,
                        theme.ui_colors.accent.g,
                        theme.ui_colors.accent.b,
                    ))
                )
            ]));
        }

        // Create configuration display paragraph
        let config_paragraph = Paragraph::new(config_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" NexusShell Configuration ")
                    .title_style(Style::default()
                        .fg(Color::Rgb(
                            theme.ui_colors.accent.r,
                            theme.ui_colors.accent.g,
                            theme.ui_colors.accent.b,
                        ))
                        .add_modifier(Modifier::BOLD)
                    )
                    .border_style(Style::default().fg(Color::Rgb(
                        theme.ui_colors.border.r,
                        theme.ui_colors.border.g,
                        theme.ui_colors.border.b,
                    )))
            )
            .wrap(Wrap { trim: true });

        // Clear area behind overlay
        let clear_area = Block::default()
            .style(Style::default().bg(Color::Rgb(
                theme.ui_colors.background.r,
                theme.ui_colors.background.g,
                theme.ui_colors.background.b,
            )));
        f.render_widget(clear_area, overlay_area);
        
        // Render the configuration panel
        f.render_widget(config_paragraph, overlay_area);
        
        // Add navigation hint
        let hint_area = Rect {
            x: overlay_x,
            y: overlay_y + overlay_height,
            width: overlay_width,
            height: 1,
        };
        
        let hint_text = Paragraph::new("Esc: Close, F2: Edit config file")
            .style(Style::default().fg(Color::Rgb(
                theme.ui_colors.secondary.r,
                theme.ui_colors.secondary.g,
                theme.ui_colors.secondary.b,
            )));
        f.render_widget(hint_text, hint_area);
        
        Ok(())
    }
    
    /// Draw theme selection overlay
    fn draw_theme_overlay(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        // Only draw if in theme selection mode
        if !matches!(self.state.mode, AppMode::ThemeSelection) {
            return Ok(());
        }

        // Calculate overlay size for theme selector
        let overlay_width = area.width.saturating_sub(20).max(40);
        let overlay_height = area.height.saturating_sub(8).max(15);
        
        // Center the overlay
        let overlay_x = (area.width.saturating_sub(overlay_width)) / 2;
        let overlay_y = (area.height.saturating_sub(overlay_height)) / 2;
        
        let overlay_area = Rect {
            x: overlay_x,
            y: overlay_y,
            width: overlay_width,
            height: overlay_height,
        };

        // Get current theme and available themes
        let current_theme = self.theme_manager.current_theme();
        let available_themes = self.theme_manager.available_themes();
        let current_theme_name = self.theme_manager.current_theme_name();
        
        // Create theme list items
        let mut theme_items = Vec::new();
        
        // Theme selector header
        theme_items.push(Line::from(vec![
            Span::styled(
                "🎨 Theme Selection",
                Style::default()
                    .fg(Color::Rgb(
                        current_theme.ui_colors.accent.r,
                        current_theme.ui_colors.accent.g,
                        current_theme.ui_colors.accent.b,
                    ))
                    .add_modifier(Modifier::BOLD)
            )
        ]));
        
        theme_items.push(Line::from(""));

        // List available themes
        for (i, theme_name) in available_themes.iter().enumerate() {
            let is_current = theme_name == &current_theme_name;
            let is_selected = i == self.state.theme_selection_index;
            
            let mut line_spans = Vec::new();
            
            // Selection indicator
            if is_selected {
                line_spans.push(Span::styled(
                    "► ",
                    Style::default()
                        .fg(Color::Rgb(
                            current_theme.ui_colors.accent.r,
                            current_theme.ui_colors.accent.g,
                            current_theme.ui_colors.accent.b,
                        ))
                        .add_modifier(Modifier::BOLD)
                ));
            } else {
                line_spans.push(Span::raw("  "));
            }
            
            // Current theme indicator
            if is_current {
                line_spans.push(Span::styled(
                    "✓ ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                ));
            } else {
                line_spans.push(Span::raw("  "));
            }
            
            // Theme name
            let theme_style = if is_selected {
                Style::default()
                    .fg(Color::Rgb(
                        current_theme.ui_colors.primary.r,
                        current_theme.ui_colors.primary.g,
                        current_theme.ui_colors.primary.b,
                    ))
                    .bg(Color::Rgb(
                        current_theme.ui_colors.selection.r,
                        current_theme.ui_colors.selection.g,
                        current_theme.ui_colors.selection.b,
                    ))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(
                    current_theme.ui_colors.primary.r,
                    current_theme.ui_colors.primary.g,
                    current_theme.ui_colors.primary.b,
                ))
            };
            
            line_spans.push(Span::styled(
                format!("{:<15}", theme_name),
                theme_style
            ));
            
            // Theme preview
            if let Ok(preview_theme) = self.theme_manager.get_theme(theme_name) {
                line_spans.push(Span::styled(
                    " ● ",
                    Style::default().fg(Color::Rgb(
                        preview_theme.ui_colors.primary.r,
                        preview_theme.ui_colors.primary.g,
                        preview_theme.ui_colors.primary.b,
                    ))
                ));
                line_spans.push(Span::styled(
                    "Syntax ",
                    Style::default().fg(Color::Rgb(
                        preview_theme.ui_colors.accent.r,
                        preview_theme.ui_colors.accent.g,
                        preview_theme.ui_colors.accent.b,
                    ))
                ));
                line_spans.push(Span::styled(
                    "● ",
                    Style::default().fg(Color::Rgb(
                        preview_theme.ui_colors.warning.r,
                        preview_theme.ui_colors.warning.g,
                        preview_theme.ui_colors.warning.b,
                    ))
                ));
                line_spans.push(Span::styled(
                    "Warning ",
                    Style::default().fg(Color::Rgb(
                        preview_theme.ui_colors.warning.r,
                        preview_theme.ui_colors.warning.g,
                        preview_theme.ui_colors.warning.b,
                    ))
                ));
                line_spans.push(Span::styled(
                    "● ",
                    Style::default().fg(Color::Rgb(
                        preview_theme.ui_colors.error.r,
                        preview_theme.ui_colors.error.g,
                        preview_theme.ui_colors.error.b,
                    ))
                ));
                line_spans.push(Span::styled(
                    "Error",
                    Style::default().fg(Color::Rgb(
                        preview_theme.ui_colors.error.r,
                        preview_theme.ui_colors.error.g,
                        preview_theme.ui_colors.error.b,
                    ))
                ));
            }
            
            theme_items.push(Line::from(line_spans));
        }

        // Create theme selector paragraph
        let theme_paragraph = Paragraph::new(theme_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Available Themes ")
                    .title_style(Style::default()
                        .fg(Color::Rgb(
                            current_theme.ui_colors.accent.r,
                            current_theme.ui_colors.accent.g,
                            current_theme.ui_colors.accent.b,
                        ))
                        .add_modifier(Modifier::BOLD)
                    )
                    .border_style(Style::default().fg(Color::Rgb(
                        current_theme.ui_colors.border.r,
                        current_theme.ui_colors.border.g,
                        current_theme.ui_colors.border.b,
                    )))
            )
            .wrap(Wrap { trim: true });

        // Clear area behind overlay
        let clear_area = Block::default()
            .style(Style::default().bg(Color::Rgb(
                current_theme.ui_colors.background.r,
                current_theme.ui_colors.background.g,
                current_theme.ui_colors.background.b,
            )));
        f.render_widget(clear_area, overlay_area);
        
        // Render the theme selector
        f.render_widget(theme_paragraph, overlay_area);
        
        // Add navigation hint
        let hint_area = Rect {
            x: overlay_x,
            y: overlay_y + overlay_height,
            width: overlay_width,
            height: 1,
        };
        
        let hint_text = Paragraph::new("↑↓: Navigate, Enter: Select, Esc: Cancel")
            .style(Style::default().fg(Color::Rgb(
                current_theme.ui_colors.secondary.r,
                current_theme.ui_colors.secondary.g,
                current_theme.ui_colors.secondary.b,
            )));
        f.render_widget(hint_text, hint_area);
        
        Ok(())
    }
    
    /// Draw help overlay
    fn draw_help_overlay(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        // Only draw if in help mode
        if !matches!(self.state.mode, AppMode::HelpMode) {
            return Ok(());
        }

        // Calculate overlay size for help panel
        let overlay_width = area.width.saturating_sub(8).max(60);
        let overlay_height = area.height.saturating_sub(4).max(25);
        
        // Center the overlay
        let overlay_x = (area.width.saturating_sub(overlay_width)) / 2;
        let overlay_y = (area.height.saturating_sub(overlay_height)) / 2;
        
        let overlay_area = Rect {
            x: overlay_x,
            y: overlay_y,
            width: overlay_width,
            height: overlay_height,
        };

        // Get current theme
        let theme = self.theme_manager.current_theme();
        
        // Create help content
        let help_content = vec![
            Line::from(vec![
                Span::styled(
                    "🚀 NexusShell - Interactive Help System",
                    Style::default()
                        .fg(Color::Rgb(
                            theme.ui_colors.accent.r,
                            theme.ui_colors.accent.g,
                            theme.ui_colors.accent.b,
                        ))
                        .add_modifier(Modifier::BOLD)
                )
            ]),
            Line::from(""),
            
            // Navigation section
            Line::from(vec![
                Span::styled(
                    "📍 Navigation & Control",
                    Style::default()
                        .fg(Color::Rgb(
                            theme.ui_colors.primary.r,
                            theme.ui_colors.primary.g,
                            theme.ui_colors.primary.b,
                        ))
                        .add_modifier(Modifier::BOLD)
                )
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("         Auto-completion suggestions"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("         Command history navigation"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Ctrl+C", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("      Interrupt current command"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Ctrl+D", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("      Exit NexusShell"),
            ]),
            Line::from(""),
            
            // Editor shortcuts
            Line::from(vec![
                Span::styled(
                    "✏️  Editor Shortcuts",
                    Style::default()
                        .fg(Color::Rgb(
                            theme.ui_colors.primary.r,
                            theme.ui_colors.primary.g,
                            theme.ui_colors.primary.b,
                        ))
                        .add_modifier(Modifier::BOLD)
                )
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Ctrl+A", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("      Move to beginning of line"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Ctrl+E", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("      Move to end of line"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Ctrl+K", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("      Delete from cursor to end"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Ctrl+U", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("      Delete entire line"),
            ]),
            Line::from(""),
            
            // System features
            Line::from(vec![
                Span::styled(
                    "⚙️  System Features",
                    Style::default()
                        .fg(Color::Rgb(
                            theme.ui_colors.primary.r,
                            theme.ui_colors.primary.g,
                            theme.ui_colors.primary.b,
                        ))
                        .add_modifier(Modifier::BOLD)
                )
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("F1", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("          Show this help panel"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("F2", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("          Open configuration editor"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("F3", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("          Theme selection"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("F12", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("         Toggle developer tools"),
            ]),
            Line::from(""),
            
            // Built-in commands
            Line::from(vec![
                Span::styled(
                    "🔧 Built-in Commands",
                    Style::default()
                        .fg(Color::Rgb(
                            theme.ui_colors.primary.r,
                            theme.ui_colors.primary.g,
                            theme.ui_colors.primary.b,
                        ))
                        .add_modifier(Modifier::BOLD)
                )
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("        Show command help and usage"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("history", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("     View command history"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("jobs", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("        List background jobs"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("alias", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("       Create command aliases"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("cd", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("          Change directory"),
            ]),
            Line::from(""),
            
            // Tips section
            Line::from(vec![
                Span::styled(
                    "💡 Pro Tips",
                    Style::default()
                        .fg(Color::Rgb(
                            theme.ui_colors.primary.r,
                            theme.ui_colors.primary.g,
                            theme.ui_colors.primary.b,
                        ))
                        .add_modifier(Modifier::BOLD)
                )
            ]),
            Line::from(vec![
                Span::raw("  • Use "),
                Span::styled("command &", Style::default().fg(Color::Green)),
                Span::raw(" to run commands in background"),
            ]),
            Line::from(vec![
                Span::raw("  • Type "),
                Span::styled("!!", Style::default().fg(Color::Green)),
                Span::raw(" to repeat the last command"),
            ]),
            Line::from(vec![
                Span::raw("  • Use "),
                Span::styled("Ctrl+R", Style::default().fg(Color::Green)),
                Span::raw(" for reverse history search"),
            ]),
            Line::from(vec![
                Span::raw("  • Double-click to select words"),
            ]),
        ];

        // Create help paragraph
        let help_paragraph = Paragraph::new(help_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" NexusShell Help & Documentation ")
                    .title_style(Style::default()
                        .fg(Color::Rgb(
                            theme.ui_colors.accent.r,
                            theme.ui_colors.accent.g,
                            theme.ui_colors.accent.b,
                        ))
                        .add_modifier(Modifier::BOLD)
                    )
                    .border_style(Style::default().fg(Color::Rgb(
                        theme.ui_colors.border.r,
                        theme.ui_colors.border.g,
                        theme.ui_colors.border.b,
                    )))
            )
            .wrap(Wrap { trim: true })
            .scroll((self.state.help_scroll_offset, 0));

        // Clear area behind overlay
        let clear_area = Block::default()
            .style(Style::default().bg(Color::Rgb(
                theme.ui_colors.background.r,
                theme.ui_colors.background.g,
                theme.ui_colors.background.b,
            )));
        f.render_widget(clear_area, overlay_area);
        
        // Render the help panel
        f.render_widget(help_paragraph, overlay_area);
        
        // Add navigation hint
        let hint_area = Rect {
            x: overlay_x,
            y: overlay_y + overlay_height,
            width: overlay_width,
            height: 1,
        };
        
        let hint_text = Paragraph::new("↑↓: Scroll, Esc: Close, F1: Toggle help")
            .style(Style::default().fg(Color::Rgb(
                theme.ui_colors.secondary.r,
                theme.ui_colors.secondary.g,
                theme.ui_colors.secondary.b,
            )));
        f.render_widget(hint_text, hint_area);
        
        Ok(())
    }
    
    /// Handle key events
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match self.state.mode {
            AppMode::Normal => self.handle_normal_mode_key(key).await?,
            AppMode::InputMode => self.handle_input_mode_key(key).await?,
            AppMode::CompletionMode => self.handle_completion_mode_key(key).await?,
            AppMode::HistoryMode => self.handle_history_mode_key(key).await?,
            AppMode::HistorySearchMode => self.handle_history_search_mode_key(key).await?,
            AppMode::ConfigMode => self.handle_config_mode_key(key).await?,
            AppMode::ThemeMode => self.handle_theme_mode_key(key).await?,
            AppMode::ThemeSelection => self.handle_theme_selection_mode_key(key).await?,
            AppMode::HelpMode => self.handle_help_mode_key(key).await?,
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
                self.state.mode = AppMode::InputMode;
            }
            (KeyCode::Char('h'), KeyModifiers::NONE) => {
                self.state.mode = AppMode::HelpMode;
            }
            (KeyCode::Char('t'), KeyModifiers::NONE) => {
                self.state.mode = AppMode::ThemeMode;
            }
            (KeyCode::Char('c'), KeyModifiers::NONE) => {
                self.state.mode = AppMode::ConfigMode;
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
                self.state.mode = AppMode::Normal;
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.execute_command().await?;
            }
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.state.mode = AppMode::CompletionMode;
                // TODO: Show completions
            }
            (KeyCode::Up, KeyModifiers::NONE) => {
                self.state.mode = AppMode::HistoryMode;
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
                self.state.mode = AppMode::InputMode;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Handle keys in history mode
    async fn handle_history_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.mode = AppMode::InputMode;
            }
            KeyCode::Up => {
                if self.history_selection > 0 {
                    self.history_selection -= 1;
                }
            }
            KeyCode::Down => {
                if self.history_selection < self.history_entries.len().saturating_sub(1) {
                    self.history_selection += 1;
                }
            }
            KeyCode::Enter => {
                // Select the highlighted history entry
                if self.history_selection < self.history_entries.len() {
                    let selected_command = self.history_entries[self.history_selection].command.clone();
                    self.input_buffer = selected_command;
                    self.cursor_position = self.input_buffer.len();
                    self.state.mode = AppMode::InputMode;
                }
            }
            KeyCode::Delete | KeyCode::Char('d') => {
                // Delete selected history entry
                if self.history_selection < self.history_entries.len() {
                    self.history_entries.remove(self.history_selection);
                    if self.history_selection >= self.history_entries.len() && !self.history_entries.is_empty() {
                        self.history_selection = self.history_entries.len() - 1;
                    }
                    // Also remove from shell context history
                    self.update_shell_history().await?;
                }
            }
            KeyCode::Char('/') => {
                // Enter history search mode
                self.state.mode = AppMode::HistorySearchMode;
                self.history_filter.clear();
            }
            KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                // Clear all history
                self.history_entries.clear();
                self.history_selection = 0;
                self.update_shell_history().await?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys in history search mode
    async fn handle_history_search_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.mode = AppMode::HistoryMode;
                self.history_filter.clear();
            }
            KeyCode::Enter => {
                // Apply filter and return to history mode
                self.filter_history_entries();
                self.state.mode = AppMode::HistoryMode;
            }
            KeyCode::Backspace => {
                self.history_filter.pop();
                self.filter_history_entries();
            }
            KeyCode::Char(c) => {
                self.history_filter.push(c);
                self.filter_history_entries();
            }
            _ => {}
        }
        Ok(())
    }

    /// Update shell history from current entries
    async fn update_shell_history(&mut self) -> Result<()> {
        let context = self.shell_context.lock().await;
        let mut shell_history = context.history.lock().unwrap();
        
        // Update shell context history to match our entries
        shell_history.clear();
        for entry in &self.history_entries {
            shell_history.push(entry.command.clone());
        }
        
        Ok(())
    }

    /// Filter history entries based on current filter
    fn filter_history_entries(&mut self) {
        if self.history_filter.is_empty() {
            // Show all entries
            return;
        }

        // Filter entries that contain the filter string
        let filtered: Vec<HistoryEntry> = self.history_entries
            .iter()
            .filter(|entry| entry.command.contains(&self.history_filter))
            .cloned()
            .collect();
        
        self.history_entries = filtered;
        self.history_selection = 0;
    }
    
    /// Handle keys in config mode
    async fn handle_config_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.mode = AppMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Handle keys in theme mode
    async fn handle_theme_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.mode = AppMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Handle keys in help mode
    async fn handle_help_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.mode = AppMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Handle keys in theme selection mode
    async fn handle_theme_selection_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.mode = AppMode::Normal;
            }
            KeyCode::Up => {
                if self.state.theme_selection_index > 0 {
                    self.state.theme_selection_index -= 1;
                }
            }
            KeyCode::Down => {
                let available_themes = self.theme_manager.available_themes();
                if self.state.theme_selection_index < available_themes.len().saturating_sub(1) {
                    self.state.theme_selection_index += 1;
                }
            }
            KeyCode::Enter => {
                let available_themes = self.theme_manager.available_themes();
                if let Some(theme_name) = available_themes.get(self.state.theme_selection_index) {
                    if let Err(e) = self.theme_manager.switch_theme_sync(theme_name) {
                        self.show_status_message(format!("Failed to switch theme: {}", e), MessageType::Error);
                    } else {
                        self.show_status_message(format!("Switched to theme: {}", theme_name), MessageType::Success);
                    }
                }
                self.state.mode = AppMode::Normal;
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
            let ast_command = Box::new(nxsh_parser::ast::AstNode::Command {
                name: Box::new(nxsh_parser::ast::AstNode::Word("test")),
                args: vec![],
                background: false,
                redirections: vec![],
            });
            executor.execute(&ast_command, &mut context)
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
                output: output.stdout.clone(),
                error: if output.stderr.is_empty() { None } else { Some(output.stderr.clone()) },
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
        self.state.mode = AppMode::Normal;
        
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
        self.config_manager.update_config(config)
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
