use crate::common::{BuiltinResult, BuiltinError, BuiltinContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// UI Design command implementation
pub fn execute(args: &[String], context: &BuiltinContext) -> BuiltinResult<i32> {
    if args.is_empty() {
        return show_current_theme();
    }

    match args[0].as_str() {
        "list" | "ls" => list_available_themes(),
        "set" => {
            if args.len() < 2 {
                eprintln!("Usage: ui-design set THEME_NAME");
                return Ok(1);
            }
            set_theme(&args[1])
        }
        "create" => {
            if args.len() < 2 {
                return Err(anyhow::anyhow!("Usage: ui-design create THEME_NAME"));
            }
            create_custom_theme(&args[1]).await
        }
        "export" => {
            if args.len() < 2 {
                return Err(anyhow::anyhow!("Usage: ui-design export THEME_NAME"));
            }
            export_theme(&args[1]).await
        }
        "preview" => {
            let theme_name = args.get(1).map(|s| s.clone()).unwrap_or_else(|| "current".to_string());
            preview_theme(&theme_name).await
        }
        "reset" => reset_to_default().await,
        "customize" => {
            if args.len() < 3 {
                return Err(anyhow::anyhow!("Usage: ui-design customize ELEMENT COLOR"));
            }
            customize_element(&args[1], &args[2]).await
        }
        "help" | "--help" => {
            let help_text = show_help();
            Ok(CommandResult::success_with_output(help_text))
        },
        _ => Err(anyhow::anyhow!("Unknown subcommand: {}", args[0]))
    }
}

/// Colorize trait for adding color to text
pub trait Colorize {
    fn red(&self) -> String;
    fn green(&self) -> String;
    fn blue(&self) -> String;
    fn yellow(&self) -> String;
    fn cyan(&self) -> String;
    fn magenta(&self) -> String;
    fn white(&self) -> String;
    fn black(&self) -> String;
    fn bright_red(&self) -> String;
    fn bright_green(&self) -> String;
    fn bright_blue(&self) -> String;
    fn bright_yellow(&self) -> String;
    fn bright_cyan(&self) -> String;
    fn bright_magenta(&self) -> String;
    fn bright_white(&self) -> String;
    fn bold(&self) -> String;
    fn colorize(&self, color: &str) -> String;
    fn dim(&self) -> String;
    fn italic(&self) -> String;
    fn underline(&self) -> String;
    fn bright(&self) -> String;
    fn muted(&self) -> String;
    fn info(&self) -> String;
    fn primary(&self) -> String;
    fn secondary(&self) -> String;
    fn warning(&self) -> String;
    fn success(&self) -> String;
}

impl Colorize for str {
    fn red(&self) -> String { format!("\x1b[31m{}\x1b[0m", self) }
    fn green(&self) -> String { format!("\x1b[32m{}\x1b[0m", self) }
    fn blue(&self) -> String { format!("\x1b[34m{}\x1b[0m", self) }
    fn yellow(&self) -> String { format!("\x1b[33m{}\x1b[0m", self) }
    fn cyan(&self) -> String { format!("\x1b[36m{}\x1b[0m", self) }
    fn magenta(&self) -> String { format!("\x1b[35m{}\x1b[0m", self) }
    fn white(&self) -> String { format!("\x1b[37m{}\x1b[0m", self) }
    fn black(&self) -> String { format!("\x1b[30m{}\x1b[0m", self) }
    fn bright_red(&self) -> String { format!("\x1b[91m{}\x1b[0m", self) }
    fn bright_green(&self) -> String { format!("\x1b[92m{}\x1b[0m", self) }
    fn bright_blue(&self) -> String { format!("\x1b[94m{}\x1b[0m", self) }
    fn bright_yellow(&self) -> String { format!("\x1b[93m{}\x1b[0m", self) }
    fn bright_cyan(&self) -> String { format!("\x1b[96m{}\x1b[0m", self) }
    fn bright_magenta(&self) -> String { format!("\x1b[95m{}\x1b[0m", self) }
    fn bright_white(&self) -> String { format!("\x1b[97m{}\x1b[0m", self) }
    fn bold(&self) -> String { format!("\x1b[1m{}\x1b[0m", self) }
    fn dim(&self) -> String { format!("\x1b[2m{}\x1b[0m", self) }
    fn italic(&self) -> String { format!("\x1b[3m{}\x1b[0m", self) }
    fn underline(&self) -> String { format!("\x1b[4m{}\x1b[0m", self) }
    fn colorize(&self, color: &str) -> String {
        match color.to_lowercase().as_str() {
            "red" => self.red(),
            "green" => self.green(),
            "blue" => self.blue(),
            "yellow" => self.yellow(),
            "cyan" => self.cyan(),
            "magenta" => self.magenta(),
            "white" => self.white(),
            "black" => self.black(),
            "primary" => self.primary(),
            "secondary" => self.secondary(),
            "success" => self.success(),
            "warning" => self.warning(),
            "info" => self.info(),
            "muted" => self.muted(),
            _ => self.to_string(),
        }
    }
    fn bright(&self) -> String { format!("\x1b[1m{}\x1b[0m", self) }
    fn muted(&self) -> String { format!("\x1b[2m{}\x1b[0m", self) }
    fn info(&self) -> String { self.cyan() }
    fn primary(&self) -> String { self.blue() }
    fn secondary(&self) -> String { self.yellow() }
    fn warning(&self) -> String { self.yellow() }
    fn success(&self) -> String { self.green() }
}

impl Colorize for String {
    fn red(&self) -> String { self.as_str().red() }
    fn green(&self) -> String { self.as_str().green() }
    fn blue(&self) -> String { self.as_str().blue() }
    fn yellow(&self) -> String { self.as_str().yellow() }
    fn cyan(&self) -> String { self.as_str().cyan() }
    fn magenta(&self) -> String { self.as_str().magenta() }
    fn white(&self) -> String { self.as_str().white() }
    fn black(&self) -> String { self.as_str().black() }
    fn bright_red(&self) -> String { self.as_str().bright_red() }
    fn bright_green(&self) -> String { self.as_str().bright_green() }
    fn bright_blue(&self) -> String { self.as_str().bright_blue() }
    fn bright_yellow(&self) -> String { self.as_str().bright_yellow() }
    fn bright_cyan(&self) -> String { self.as_str().bright_cyan() }
    fn bright_magenta(&self) -> String { self.as_str().bright_magenta() }
    fn bright_white(&self) -> String { self.as_str().bright_white() }
    fn bold(&self) -> String { self.as_str().bold() }
    fn dim(&self) -> String { self.as_str().dim() }
    fn italic(&self) -> String { self.as_str().italic() }
    fn underline(&self) -> String { self.as_str().underline() }
    fn colorize(&self, _color: &str) -> String { self.as_str().green() }
    fn bright(&self) -> String { self.as_str().bright() }
    fn muted(&self) -> String { self.as_str().muted() }
    fn info(&self) -> String { self.as_str().info() }
    fn primary(&self) -> String { self.as_str().primary() }
    fn secondary(&self) -> String { self.as_str().secondary() }
    fn warning(&self) -> String { self.as_str().warning() }
    fn success(&self) -> String { self.as_str().success() }
}

/// Table formatter for displaying tabular data
#[derive(Clone)]
pub struct TableFormatter {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub options: TableOptions,
}

impl TableFormatter {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            options: TableOptions::default(),
        }
    }

    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    pub fn render(&self) -> String {
        if self.headers.is_empty() && self.rows.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        let col_count = self.headers.len().max(
            self.rows.iter().map(|row| row.len()).max().unwrap_or(0)
        );

        if col_count == 0 {
            return String::new();
        }

        // Calculate column widths
        let mut widths = vec![0; col_count];
        
        // Check header widths
        for (i, header) in self.headers.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(header.len());
            }
        }
        
        // Check row widths
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Add padding
        for width in &mut widths {
            *width += 2; // Add padding
        }

        // Render header if present
        if !self.headers.is_empty() {
            output.push_str(&self.render_separator(&widths, true));
            output.push_str(&self.render_row(&self.headers, &widths));
            output.push_str(&self.render_separator(&widths, false));
        }

        // Render rows
        for row in &self.rows {
            output.push_str(&self.render_row(row, &widths));
        }

        // Bottom border
        output.push_str(&self.render_separator(&widths, true));

        output
    }

    fn render_row(&self, row: &[String], widths: &[usize]) -> String {
        let mut line = String::new();
        line.push('│');
        
        for (i, width) in widths.iter().enumerate() {
            let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
            line.push(' ');
            match self.options.alignment {
                Alignment::Left => line.push_str(&format!("{:<width$}", cell, width = width - 2)),
                Alignment::Right => line.push_str(&format!("{:>width$}", cell, width = width - 2)),
                Alignment::Center => line.push_str(&format!("{:^width$}", cell, width = width - 2)),
            }
            line.push(' ');
            line.push('│');
        }
        line.push('\n');
        line
    }

    fn render_separator(&self, widths: &[usize], is_border: bool) -> String {
        let mut line = String::new();
        
        if is_border {
            line.push('┌');
            for (i, &width) in widths.iter().enumerate() {
                line.push_str(&"─".repeat(width));
                if i < widths.len() - 1 {
                    line.push('┬');
                }
            }
            line.push('┐');
        } else {
            line.push('├');
            for (i, &width) in widths.iter().enumerate() {
                line.push_str(&"─".repeat(width));
                if i < widths.len() - 1 {
                    line.push('┼');
                }
            }
            line.push('┤');
        }
        line.push('\n');
        line
    }

    /// Format file permissions string
    pub fn format_permissions(&self, permissions: u32) -> String {
        let mut result = String::new();
        result.push(if permissions & 0o400 != 0 { 'r' } else { '-' });
        result.push(if permissions & 0o200 != 0 { 'w' } else { '-' });
        result.push(if permissions & 0o100 != 0 { 'x' } else { '-' });
        result.push(if permissions & 0o040 != 0 { 'r' } else { '-' });
        result.push(if permissions & 0o020 != 0 { 'w' } else { '-' });
        result.push(if permissions & 0o010 != 0 { 'x' } else { '-' });
        result.push(if permissions & 0o004 != 0 { 'r' } else { '-' });
        result.push(if permissions & 0o002 != 0 { 'w' } else { '-' });
        result.push(if permissions & 0o001 != 0 { 'x' } else { '-' });
        result
    }

    /// Get file icon for a given filename
    pub fn get_file_icon(&self, name: &str) -> &'static str {
        let ext = std::path::Path::new(name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        match ext {
            "rs" => "🦀",
            "py" => "🐍", 
            "js" => "📄",
            "md" => "📝",
            "txt" => "📄",
            "zip" | "tar" | "gz" => "🗜️",
            "png" | "jpg" | "jpeg" => "🖼️",
            _ => "📄"
        }
    }

    /// Calculate display width of text
    pub fn display_width(&self, text: &str) -> usize {
        text.chars().count()
    }

    /// Create table with title
    pub fn with_title(&self, title: &str) -> Self {
        let mut new_table = self.clone();
        new_table.headers.insert(0, title.to_string());
        new_table
    }

    /// Create table with border configuration
    pub fn with_borders(&self, enabled: bool) -> Self {
        let mut new_table = self.clone();
        new_table.options.show_borders = enabled;
        new_table
    }

    /// Create advanced table with data
    pub fn create_advanced_table(&self, headers: &[String], data: &[Vec<String>]) -> String {
        let mut table = TableFormatter::new();
        table.headers = headers.to_vec();
        table.rows = data.to_vec();
        table.render()
    }

    /// Format table data (alias for create_advanced_table)
    pub fn format_table(&self, data: &[Vec<String>]) -> String {
        self.create_advanced_table(&self.headers, data)
    }

    /// Format file size in human readable format
    pub fn format_size(&self, size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size_f = size as f64;
        let mut unit_index = 0;
        
        while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
            size_f /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size_f, UNITS[unit_index])
        }
    }
}

/// Table formatting options
#[derive(Debug, Clone)]
pub struct TableOptions {
    pub border_style: BorderStyle,
    pub alignment: Alignment,
    pub padding: usize,
    pub show_header: bool,
    pub alternating_rows: bool,
    pub header_alignment: Alignment,
    pub max_width: Option<usize>,
    pub show_borders: bool,
    pub zebra_striping: bool,
    pub compact_mode: bool,
    pub align_columns: bool,
    pub compact: bool,
    pub truncate: bool,        // Add missing truncate field
    pub sort_by: Option<String>,  // Add missing sort_by field
    pub filter: Option<String>,   // Add missing filter field
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            border_style: BorderStyle::Unicode,
            alignment: Alignment::Left,
            padding: 1,
            show_header: true,
            alternating_rows: false,
            header_alignment: Alignment::Left,
            max_width: None,
            show_borders: true,
            zebra_striping: false,
            compact_mode: false,
            align_columns: true,
            compact: false,
            truncate: false,        // Add default for truncate
            sort_by: None,          // Add default for sort_by  
            filter: None,           // Add default for filter
        }
    }
}

/// Border style for tables
#[derive(Debug, Clone)]
pub enum BorderStyle {
    Simple,     // Simple ASCII borders
    Unicode,    // Unicode box drawing characters
    ASCII,      // ASCII-only characters
    None,       // No borders
}

/// Text alignment options
#[derive(Debug, Clone)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

/// Animation effects
#[derive(Debug, Clone)]
pub struct Animation {
    pub enabled: bool,
    pub duration: u16,
    pub effect: AnimationEffect,
}

impl Animation {
    /// Create a spinner animation for loading states
    pub fn spinner() -> Self {
        Self {
            enabled: true,
            duration: 100,
            effect: AnimationEffect::Pulse,
        }
    }
    
    /// Create a new animation with specified parameters
    pub fn new(enabled: bool, duration: u16, effect: AnimationEffect) -> Self {
        Self {
            enabled,
            duration,
            effect,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AnimationEffect {
    FadeIn,
    FadeOut,
    SlideIn,
    SlideOut,
    Bounce,
    Pulse,
}

/// Progress bar for long operations
#[cfg(feature = "progress-ui")]
pub use indicatif::ProgressBar;

#[cfg(not(feature = "progress-ui"))]
pub use nxsh_ui::ProgressBar;

/// Notification system
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub duration: Option<std::time::Duration>,
}

#[derive(Debug, Clone)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

impl Notification {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notification_type: NotificationType::Info,
            duration: None,
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notification_type: NotificationType::Success,
            duration: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notification_type: NotificationType::Warning,
            duration: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notification_type: NotificationType::Error,
            duration: None,
        }
    }

    pub fn with_duration(mut self, duration: std::time::Duration) -> Self {
        self.duration = Some(duration);
        self
    }
}

/// Theme data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub colors: HashMap<String, String>,
    pub icons: HashMap<String, String>,
    pub fonts: FontConfig,
    pub layout: LayoutConfig,
    pub animations: AnimationConfig,
    pub is_default: bool,
    pub is_active: bool,
}

/// Font configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub primary: String,
    pub monospace: String,
    pub size: u16,
    pub line_height: f32,
    pub weight: FontWeight,
}

/// Font weight options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FontWeight {
    Light,
    Normal,
    Bold,
    ExtraBold,
}

/// Layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub padding: u16,
    pub margin: u16,
    pub border_radius: u16,
    pub border_width: u16,
    pub spacing: u16,
}

/// Animation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    pub duration: u16,
    pub easing: EasingType,
    pub enabled: bool,
}

/// Animation easing types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

/// Color palette for UI elements
#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub primary: String,
    pub secondary: String,
    pub background: String,
    pub foreground: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
    pub muted: String,
    pub border: String,
    pub reset: String,
    pub dim: String,
    pub bright: String,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            primary: "#007acc".to_string(),
            secondary: "#6c757d".to_string(),
            background: "#ffffff".to_string(),
            foreground: "#000000".to_string(),
            success: "#28a745".to_string(),
            warning: "#ffc107".to_string(),
            error: "#dc3545".to_string(),
            info: "#17a2b8".to_string(),
            muted: "#6c757d".to_string(),
            border: "#dee2e6".to_string(),
            reset: "\x1b[0m".to_string(),
            dim: "\x1b[2m".to_string(),
            bright: "\x1b[1m".to_string(),
        }
    }
}

impl ColorPalette {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub const ACCENT: &'static str = "#007acc";
    pub const INFO: &'static str = "#17a2b8";
    pub const SUCCESS: &'static str = "#28a745";
    pub const ERROR: &'static str = "#dc3545";
    pub const WARNING: &'static str = "#ffc107";
}

/// Icon set for UI elements
#[derive(Debug, Clone)]
pub struct Icons {
    pub file: String,
    pub folder: String,
    pub success: String,
    pub error: String,
    pub warning: String,
    pub info: String,
    pub loading: String,
    pub arrow_right: String,
    pub arrow_left: String,
    pub arrow_up: String,
    pub arrow_down: String,
    pub user: String,
    pub symlink: String,
    pub bright: String,
    pub document: String,
    pub bullet: String,
    pub code: String,
}

impl Default for Icons {
    fn default() -> Self {
        Self {
            file: "📄".to_string(),
            folder: "📁".to_string(),
            success: "✓".to_string(),
            error: "✗".to_string(),
            warning: "⚠".to_string(),
            info: "ℹ".to_string(),
            loading: "⏳".to_string(),
            arrow_right: "→".to_string(),
            arrow_left: "←".to_string(),
            arrow_up: "↑".to_string(),
            arrow_down: "↓".to_string(),
            user: "👤".to_string(),
            symlink: "🔗".to_string(),
            bright: "💡".to_string(),
            document: "📄".to_string(),
            bullet: "•".to_string(),
            code: "💻".to_string(),
        }
    }
}

impl Icons {
    pub fn new() -> Self {
        Self::default()
    }
    
    // Legacy constants for backward compatibility
    pub const ALIAS: &'static str = "🔗";
    pub const BUILTIN: &'static str = "🔧";
    pub const EXECUTABLE: &'static str = "⚡";
    pub const ERROR: &'static str = "❌";
    pub const FOLDER: &'static str = "📁";
    pub const FOLDER_PLUS: &'static str = "📁➕";
    pub const FOLDER_MINUS: &'static str = "📁➖";
    pub const TRASH: &'static str = "🗑️";
    pub const WARNING: &'static str = "⚠️";
    
    pub fn new_with_unicode(use_unicode: bool) -> Self {
        if use_unicode {
            Self::default()
        } else {
            Self {
                file: "f".to_string(),
                folder: "d".to_string(),
                success: "+".to_string(),
                error: "x".to_string(),
                warning: "!".to_string(),
                info: "i".to_string(),
                loading: "*".to_string(),
                arrow_right: ">".to_string(),
                arrow_left: "<".to_string(),
                arrow_up: "^".to_string(),
                arrow_down: "v".to_string(),
                user: "u".to_string(),
                symlink: "l".to_string(),
                bright: "*".to_string(),
                document: "f".to_string(),
                bullet: "*".to_string(),
                code: "c".to_string(),
            }
        }
    }
}

// Helper functions for theme management

/// Show current active theme
async fn show_current_theme(_context: &ExecutionContext) -> Result<CommandResult> {
    let theme = get_current_theme().await?;
    let mut output = String::new();
    
    output.push_str(&format!("Current Theme: {}\n", theme.name));
    output.push_str(&format!("Description: {}\n", theme.description));
    output.push_str(&format!("Version: {}\n", theme.version));
    output.push_str(&format!("Author: {}\n", theme.author));
    output.push_str("\nColor Scheme:\n");
    
    for (element, color) in &theme.colors {
        output.push_str(&format!("  {}: {}\n", element, color));
    }
    
    output.push_str("\nIcons:\n");
    for (name, icon) in &theme.icons {
        output.push_str(&format!("  {}: {}\n", name, icon));
    }
    
    Ok(CommandResult::success_with_output(output))
}

/// List all available themes
async fn list_available_themes() -> Result<CommandResult> {
    let themes = get_available_themes().await?;
    let mut output = String::new();
    
    output.push_str("Available Themes:\n\n");
    
    for theme in themes {
        output.push_str(&format!("  {} - {}\n", theme.name, theme.description));
        output.push_str(&format!("    Author: {} | Version: {}\n", theme.author, theme.version));
        
        if theme.is_default {
            output.push_str("    [DEFAULT]\n");
        }
        
        if theme.is_active {
            output.push_str("    [ACTIVE]\n");
        }
        
        output.push_str("\n");
    }
    
    Ok(CommandResult::success_with_output(output))
}

/// Set active theme
async fn set_theme(theme_name: &str) -> Result<CommandResult> {
    let available_themes = get_available_themes().await?;
    
    if !available_themes.iter().any(|t| t.name == theme_name) {
        return Err(anyhow::anyhow!("Theme '{}' not found", theme_name));
    }
    
    // Load and apply the theme
    let theme = load_theme(theme_name).await?;
    apply_theme(theme).await?;
    
    Ok(CommandResult::success_with_output(format!(
        "Theme '{}' applied successfully", theme_name
    )))
}

/// Create a custom theme
async fn create_custom_theme(theme_name: &str) -> Result<CommandResult> {
    let template = ThemeTemplate::new(theme_name);
    let theme = Theme::from_template(template);
    
    save_theme(&theme).await?;
    
    Ok(CommandResult::success_with_output(format!(
        "Custom theme '{}' created. Use 'ui-design customize' to modify elements.", 
        theme_name
    )))
}

/// Export theme to file
async fn export_theme(theme_name: &str) -> Result<CommandResult> {
    let theme = load_theme(theme_name).await?;
    let json = serde_json::to_string_pretty(&theme)?;
    
    let filename = format!("{}.json", theme_name);
    std::fs::write(&filename, json)?;
    
    Ok(CommandResult::success_with_output(format!(
        "Theme '{}' exported to {}", theme_name, filename
    )))
}

/// Preview a theme
async fn preview_theme(theme_name: &str) -> Result<CommandResult> {
    let theme = if theme_name == "current" {
        get_current_theme().await?
    } else {
        load_theme(theme_name).await?
    };
    
    let preview = generate_theme_preview(&theme);
    
    Ok(CommandResult::success_with_output(preview))
}

/// Reset to default theme
async fn reset_to_default() -> Result<CommandResult> {
    let default_theme = get_default_theme().await?;
    apply_theme(default_theme).await?;
    
    Ok(CommandResult::success_with_output("Reset to default theme".to_string()))
}

/// Customize a specific UI element
async fn customize_element(element: &str, color: &str) -> Result<CommandResult> {
    let mut theme = get_current_theme().await?;
    
    if !is_valid_color(color) {
        return Err(anyhow::anyhow!("Invalid color format: {}", color));
    }
    
    theme.colors.insert(element.to_string(), color.to_string());
    apply_theme(theme).await?;
    
    Ok(CommandResult::success_with_output(format!(
        "Updated {} to {}", element, color
    )))
}

/// Theme template for creating new themes
#[derive(Debug, Clone)]
pub struct ThemeTemplate {
    pub name: String,
    pub base_colors: HashMap<String, String>,
    pub base_icons: HashMap<String, String>,
}

impl ThemeTemplate {
    pub fn new(name: &str) -> Self {
        let mut base_colors = HashMap::new();
        base_colors.insert("primary".to_string(), "#007acc".to_string());
        base_colors.insert("secondary".to_string(), "#6c757d".to_string());
        base_colors.insert("background".to_string(), "#ffffff".to_string());
        base_colors.insert("foreground".to_string(), "#000000".to_string());
        base_colors.insert("success".to_string(), "#28a745".to_string());
        base_colors.insert("warning".to_string(), "#ffc107".to_string());
        base_colors.insert("error".to_string(), "#dc3545".to_string());
        base_colors.insert("info".to_string(), "#17a2b8".to_string());
        base_colors.insert("muted".to_string(), "#6c757d".to_string());
        base_colors.insert("border".to_string(), "#dee2e6".to_string());
        
        let mut base_icons = HashMap::new();
        base_icons.insert("file".to_string(), "📄".to_string());
        base_icons.insert("folder".to_string(), "📁".to_string());
        base_icons.insert("success".to_string(), "✓".to_string());
        base_icons.insert("error".to_string(), "✗".to_string());
        base_icons.insert("warning".to_string(), "⚠".to_string());
        base_icons.insert("info".to_string(), "ℹ".to_string());
        base_icons.insert("loading".to_string(), "⏳".to_string());
        base_icons.insert("arrow_right".to_string(), "→".to_string());
        base_icons.insert("arrow_left".to_string(), "←".to_string());
        base_icons.insert("arrow_up".to_string(), "↑".to_string());
        base_icons.insert("arrow_down".to_string(), "↓".to_string());
        
        Self {
            name: name.to_string(),
            base_colors,
            base_icons,
        }
    }
}

impl Theme {
    pub fn from_template(template: ThemeTemplate) -> Self {
        Self {
            name: template.name,
            description: "Custom theme".to_string(),
            version: "1.0.0".to_string(),
            author: "User".to_string(),
            colors: template.base_colors,
            icons: template.base_icons,
            fonts: FontConfig {
                primary: "Sans Serif".to_string(),
                monospace: "Monospace".to_string(),
                size: 14,
                line_height: 1.4,
                weight: FontWeight::Normal,
            },
            layout: LayoutConfig {
                padding: 8,
                margin: 4,
                border_radius: 4,
                border_width: 1,
                spacing: 8,
            },
            animations: AnimationConfig {
                duration: 200,
                easing: EasingType::EaseInOut,
                enabled: true,
            },
            is_default: false,
            is_active: false,
        }
    }
}

/// Theme information for listing
#[derive(Debug, Clone)]
pub struct ThemeInfo {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub is_default: bool,
    pub is_active: bool,
}

/// Get current active theme
async fn get_current_theme() -> Result<Theme> {
    // This would normally load from config/state
    Ok(get_default_theme().await?)
}

/// Get list of available themes
async fn get_available_themes() -> Result<Vec<ThemeInfo>> {
    Ok(vec![
        ThemeInfo {
            name: "default".to_string(),
            description: "Default NexusShell theme".to_string(),
            author: "NexusShell Team".to_string(),
            version: "1.0.0".to_string(),
            is_default: true,
            is_active: true,
        },
        ThemeInfo {
            name: "dark".to_string(),
            description: "Dark theme for low-light environments".to_string(),
            author: "NexusShell Team".to_string(),
            version: "1.0.0".to_string(),
            is_default: false,
            is_active: false,
        },
        ThemeInfo {
            name: "cyberpunk".to_string(),
            description: "Futuristic neon theme".to_string(),
            author: "Community".to_string(),
            version: "1.2.0".to_string(),
            is_default: false,
            is_active: false,
        },
        ThemeInfo {
            name: "minimalist".to_string(),
            description: "Clean and simple design".to_string(),
            author: "Community".to_string(),
            version: "1.1.0".to_string(),
            is_default: false,
            is_active: false,
        },
    ])
}

/// Load a specific theme
async fn load_theme(theme_name: &str) -> Result<Theme> {
    match theme_name {
        "default" => get_default_theme().await,
        "dark" => get_dark_theme().await,
        "cyberpunk" => get_cyberpunk_theme().await,
        "minimalist" => get_minimalist_theme().await,
        _ => Err(anyhow::anyhow!("Theme '{}' not found", theme_name))
    }
}

/// Get default theme
async fn get_default_theme() -> Result<Theme> {
    let mut colors = HashMap::new();
    colors.insert("primary".to_string(), "#007acc".to_string());
    colors.insert("secondary".to_string(), "#6c757d".to_string());
    colors.insert("background".to_string(), "#ffffff".to_string());
    colors.insert("foreground".to_string(), "#000000".to_string());
    colors.insert("success".to_string(), "#28a745".to_string());
    colors.insert("warning".to_string(), "#ffc107".to_string());
    colors.insert("error".to_string(), "#dc3545".to_string());
    colors.insert("info".to_string(), "#17a2b8".to_string());
    colors.insert("muted".to_string(), "#6c757d".to_string());
    colors.insert("border".to_string(), "#dee2e6".to_string());
    
    let mut icons = HashMap::new();
    icons.insert("file".to_string(), "📄".to_string());
    icons.insert("folder".to_string(), "📁".to_string());
    icons.insert("success".to_string(), "✓".to_string());
    icons.insert("error".to_string(), "✗".to_string());
    icons.insert("warning".to_string(), "⚠".to_string());
    icons.insert("info".to_string(), "ℹ".to_string());
    icons.insert("loading".to_string(), "⏳".to_string());
    icons.insert("arrow_right".to_string(), "→".to_string());
    icons.insert("arrow_left".to_string(), "←".to_string());
    icons.insert("arrow_up".to_string(), "↑".to_string());
    icons.insert("arrow_down".to_string(), "↓".to_string());
    
    Ok(Theme {
        name: "default".to_string(),
        description: "Default NexusShell theme".to_string(),
        version: "1.0.0".to_string(),
        author: "NexusShell Team".to_string(),
        colors,
        icons,
        fonts: FontConfig {
            primary: "Sans Serif".to_string(),
            monospace: "Monospace".to_string(),
            size: 14,
            line_height: 1.4,
            weight: FontWeight::Normal,
        },
        layout: LayoutConfig {
            padding: 8,
            margin: 4,
            border_radius: 4,
            border_width: 1,
            spacing: 8,
        },
        animations: AnimationConfig {
            duration: 200,
            easing: EasingType::EaseInOut,
            enabled: true,
        },
        is_default: true,
        is_active: true,
    })
}

/// Get dark theme
async fn get_dark_theme() -> Result<Theme> {
    let mut theme = get_default_theme().await?;
    theme.name = "dark".to_string();
    theme.description = "Dark theme for low-light environments".to_string();
    theme.is_default = false;
    theme.is_active = false;
    
    // Override colors for dark theme
    theme.colors.insert("background".to_string(), "#1a1a1a".to_string());
    theme.colors.insert("foreground".to_string(), "#ffffff".to_string());
    theme.colors.insert("primary".to_string(), "#4dabf7".to_string());
    theme.colors.insert("secondary".to_string(), "#868e96".to_string());
    theme.colors.insert("muted".to_string(), "#868e96".to_string());
    theme.colors.insert("border".to_string(), "#343a40".to_string());
    
    Ok(theme)
}

/// Get cyberpunk theme
async fn get_cyberpunk_theme() -> Result<Theme> {
    let mut theme = get_default_theme().await?;
    theme.name = "cyberpunk".to_string();
    theme.description = "Futuristic neon theme".to_string();
    theme.author = "Community".to_string();
    theme.version = "1.2.0".to_string();
    theme.is_default = false;
    theme.is_active = false;
    
    // Cyberpunk colors
    theme.colors.insert("background".to_string(), "#0a0a0a".to_string());
    theme.colors.insert("foreground".to_string(), "#00ff00".to_string());
    theme.colors.insert("primary".to_string(), "#ff006e".to_string());
    theme.colors.insert("secondary".to_string(), "#00f5ff".to_string());
    theme.colors.insert("success".to_string(), "#00ff00".to_string());
    theme.colors.insert("warning".to_string(), "#ffff00".to_string());
    theme.colors.insert("error".to_string(), "#ff0040".to_string());
    theme.colors.insert("info".to_string(), "#00f5ff".to_string());
    theme.colors.insert("muted".to_string(), "#666666".to_string());
    theme.colors.insert("border".to_string(), "#00ff00".to_string());
    
    Ok(theme)
}

/// Get minimalist theme
async fn get_minimalist_theme() -> Result<Theme> {
    let mut theme = get_default_theme().await?;
    theme.name = "minimalist".to_string();
    theme.description = "Clean and simple design".to_string();
    theme.author = "Community".to_string();
    theme.version = "1.1.0".to_string();
    theme.is_default = false;
    theme.is_active = false;
    
    // Minimalist colors (grayscale)
    theme.colors.insert("primary".to_string(), "#333333".to_string());
    theme.colors.insert("secondary".to_string(), "#666666".to_string());
    theme.colors.insert("background".to_string(), "#fafafa".to_string());
    theme.colors.insert("foreground".to_string(), "#333333".to_string());
    theme.colors.insert("success".to_string(), "#666666".to_string());
    theme.colors.insert("warning".to_string(), "#999999".to_string());
    theme.colors.insert("error".to_string(), "#333333".to_string());
    theme.colors.insert("info".to_string(), "#666666".to_string());
    theme.colors.insert("muted".to_string(), "#cccccc".to_string());
    theme.colors.insert("border".to_string(), "#eeeeee".to_string());
    
    // Simple icons
    theme.icons.insert("file".to_string(), "•".to_string());
    theme.icons.insert("folder".to_string(), "◦".to_string());
    theme.icons.insert("success".to_string(), "✓".to_string());
    theme.icons.insert("error".to_string(), "×".to_string());
    theme.icons.insert("warning".to_string(), "!".to_string());
    theme.icons.insert("info".to_string(), "i".to_string());
    theme.icons.insert("loading".to_string(), "…".to_string());
    theme.icons.insert("arrow_right".to_string(), ">".to_string());
    theme.icons.insert("arrow_left".to_string(), "<".to_string());
    theme.icons.insert("arrow_up".to_string(), "^".to_string());
    theme.icons.insert("arrow_down".to_string(), "v".to_string());
    
    Ok(theme)
}

/// Apply theme to the system
async fn apply_theme(mut theme: Theme) -> Result<()> {
    theme.is_active = true;
    // This would normally update the system configuration
    // For now, just simulate success
    Ok(())
}

/// Save theme to storage
async fn save_theme(theme: &Theme) -> Result<()> {
    let json = serde_json::to_string_pretty(theme)?;
    let filename = format!("themes/{}.json", theme.name);
    
    // Create themes directory if it doesn't exist
    std::fs::create_dir_all("themes")?;
    std::fs::write(filename, json)?;
    
    Ok(())
}

/// Generate a preview of the theme
fn generate_theme_preview(theme: &Theme) -> String {
    let mut preview = String::new();
    
    preview.push_str(&format!("Theme Preview: {}\n", theme.name));
    preview.push_str(&format!("Description: {}\n\n", theme.description));
    
    preview.push_str("Sample Table:\n");
    preview.push_str("┌─────────────────┬──────────────┬────────────┐\n");
    preview.push_str("│ File Name       │ Size         │ Status     │\n");
    preview.push_str("├─────────────────┼──────────────┼────────────┤\n");
    preview.push_str("│ document.txt    │ 1.2 KB       │ ✓ Ready    │\n");
    preview.push_str("│ 📁 project/      │ 45.6 MB      │ ⏳ Syncing  │\n");
    preview.push_str("│ config.json     │ 892 B        │ ⚠ Warning  │\n");
    preview.push_str("│ backup.zip      │ 124.5 MB     │ ✗ Error    │\n");
    preview.push_str("└─────────────────┴──────────────┴────────────┘\n\n");
    
    preview.push_str("Progress Indicators:\n");
    preview.push_str("Upload:   [████████████████████████████████────] 75.0%\n");
    preview.push_str("Download: [████████████░░░░░░░░░░░░░░░░░░░░░░░░░░] 30.0%\n");
    preview.push_str("Complete: [████████████████████████████████████] 100.0%\n\n");
    
    preview.push_str("Status Indicators:\n");
    preview.push_str("✓ Success: Operation completed\n");
    preview.push_str("⚠ Warning: Check configuration\n");
    preview.push_str("✗ Error: Connection failed\n");
    preview.push_str("ℹ Info: Processing...\n");
    
    preview
}

/// Validate color format
fn is_valid_color(color: &str) -> bool {
    // Check hex color format
    if color.starts_with('#') && color.len() == 7 {
        return color.chars().skip(1).all(|c| c.is_ascii_hexdigit());
    }
    
    // Check named colors
    matches!(color.to_lowercase().as_str(), 
        "red" | "green" | "blue" | "yellow" | "orange" | "purple" | 
        "pink" | "brown" | "black" | "white" | "gray" | "grey" |
        "cyan" | "magenta" | "lime" | "navy" | "silver" | "maroon"
    )
}

/// Show help information
fn show_help() -> String {
    r#"ui-design - Manage UI themes and design elements

USAGE:
    ui-design [COMMAND] [OPTIONS]

COMMANDS:
    (no args)                       Show current theme information
    list, ls                        List all available themes
    set THEME_NAME                  Set active theme
    create THEME_NAME               Create a new custom theme
    export THEME_NAME               Export theme to JSON file
    preview [THEME_NAME]            Preview theme (current if not specified)
    reset                           Reset to default theme
    customize ELEMENT COLOR         Customize specific UI element
    help, --help                    Show this help message

EXAMPLES:
    ui-design                       # Show current theme
    ui-design list                  # List available themes
    ui-design set dark              # Switch to dark theme
    ui-design create mytheme        # Create custom theme
    ui-design preview cyberpunk     # Preview cyberpunk theme
    ui-design customize primary #ff0000  # Set primary color to red
    ui-design export mytheme        # Export theme to file

CUSTOMIZABLE ELEMENTS:
    primary, secondary, background, foreground, success, warning,
    error, info, muted, border

SUPPORTED COLOR FORMATS:
    Hex colors: #ff0000, #00ff00, #0000ff
    Named colors: red, green, blue, yellow, etc.
"#.to_string()
}
