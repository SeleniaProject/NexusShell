/// Advanced CUI Design System for NexusShell
/// 
/// This module implements a comprehensive CUI (Character User Interface) design system
/// that provides beautiful, consistent, and highly readable command-line interfaces
/// for all NexusShell commands and operations.
/// 
/// Features:
/// - Modern flat design with subtle depth
/// - Consistent color palette and typography
/// - Rich table formatting with smart column sizing
/// - Progress indicators and status displays
/// - Error and success message formatting
/// - Unicode icons with ASCII fallbacks
/// - Responsive layout for different terminal sizes
/// - Dark/light theme support
/// - Accessibility compliance

use anyhow::{Result, Context};
use crossterm::{
    style::{Color, Print, ResetColor, SetForegroundColor, SetBackgroundColor, Attribute, SetAttribute},
    execute, terminal,
};
use std::{
    collections::HashMap,
    fmt::Write as FmtWrite,
    io::{self, Write},
};
use unicode_width::UnicodeWidthStr;

/// Main CUI design system controller
#[derive(Debug, Clone)]
pub struct AdvancedCUI {
    /// Current theme configuration
    theme: Theme,
    
    /// Terminal dimensions
    terminal_size: (u16, u16), // (width, height)
    
    /// Whether colors are supported
    color_support: bool,
    
    /// Whether unicode is supported
    unicode_support: bool,
    
    /// Layout configuration
    layout: LayoutConfig,
}

/// Theme configuration for consistent styling
#[derive(Debug, Clone)]
pub struct Theme {
    /// Primary brand color
    pub primary: Color,
    
    /// Secondary accent color
    pub secondary: Color,
    
    /// Success state color (green variants)
    pub success: Color,
    
    /// Warning state color (yellow/orange variants)
    pub warning: Color,
    
    /// Error state color (red variants)
    pub error: Color,
    
    /// Information state color (blue variants)
    pub info: Color,
    
    /// Muted/secondary text color
    pub muted: Color,
    
    /// Background color for panels
    pub background: Color,
    
    /// Border and separator color
    pub border: Color,
    
    /// Text color on colored backgrounds
    pub on_color: Color,
}

/// Layout configuration for responsive design
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Minimum column width for tables
    pub min_column_width: usize,
    
    /// Maximum column width for tables
    pub max_column_width: usize,
    
    /// Padding around content
    pub content_padding: usize,
    
    /// Margin between sections
    pub section_margin: usize,
    
    /// Whether to show borders
    pub show_borders: bool,
    
    /// Whether to use compact mode
    pub compact_mode: bool,
}

/// Icon set for different UI elements
#[derive(Debug, Clone)]
pub struct IconSet {
    /// Success/checkmark icon
    pub success: &'static str,
    
    /// Error/cross icon
    pub error: &'static str,
    
    /// Warning/exclamation icon
    pub warning: &'static str,
    
    /// Information icon
    pub info: &'static str,
    
    /// File icon
    pub file: &'static str,
    
    /// Directory icon
    pub directory: &'static str,
    
    /// Loading/progress icon
    pub loading: &'static str,
    
    /// Arrow right icon
    pub arrow_right: &'static str,
    
    /// Arrow down icon
    pub arrow_down: &'static str,
    
    /// Bullet point icon
    pub bullet: &'static str,
}

/// Table styling configuration
#[derive(Debug, Clone)]
pub struct TableStyle {
    /// Whether to show header
    pub show_header: bool,
    
    /// Whether to alternate row colors
    pub alternate_rows: bool,
    
    /// Whether to show row numbers
    pub show_row_numbers: bool,
    
    /// Whether to show borders
    pub show_borders: bool,
    
    /// Header style
    pub header_style: TextStyle,
    
    /// Data cell style
    pub cell_style: TextStyle,
    
    /// Border characters
    pub border_chars: BorderChars,
}

/// Text styling options
#[derive(Debug, Clone)]
pub struct TextStyle {
    /// Text color
    pub color: Option<Color>,
    
    /// Background color
    pub background: Option<Color>,
    
    /// Bold text
    pub bold: bool,
    
    /// Italic text
    pub italic: bool,
    
    /// Underlined text
    pub underline: bool,
    
    /// Strikethrough text
    pub strikethrough: bool,
}

/// Border character set for table drawing
#[derive(Debug, Clone)]
pub struct BorderChars {
    pub horizontal: char,
    pub vertical: char,
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub cross: char,
    pub tee_down: char,
    pub tee_up: char,
    pub tee_right: char,
    pub tee_left: char,
}

/// Progress bar configuration
#[derive(Debug, Clone)]
pub struct ProgressConfig {
    /// Progress bar width
    pub width: usize,
    
    /// Show percentage
    pub show_percentage: bool,
    
    /// Show current/total counts
    pub show_counts: bool,
    
    /// Progress bar style
    pub style: ProgressStyle,
}

/// Progress bar visual style
#[derive(Debug, Clone)]
pub enum ProgressStyle {
    /// Block characters: █████░░░░░
    Blocks,
    
    /// Bar characters: ━━━━━┅┅┅┅┅
    Bars,
    
    /// ASCII characters: ####......
    Ascii,
    
    /// Dots: ●●●●●○○○○○
    Dots,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            muted: Color::DarkGrey,
            background: Color::Black,
            border: Color::DarkGrey,
            on_color: Color::White,
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            min_column_width: 8,
            max_column_width: 50,
            content_padding: 1,
            section_margin: 1,
            show_borders: true,
            compact_mode: false,
        }
    }
}

impl IconSet {
    /// Unicode icon set for modern terminals
    pub fn unicode() -> Self {
        Self {
            success: "✅",
            error: "❌",
            warning: "⚠️",
            info: "ℹ️",
            file: "📄",
            directory: "📁",
            loading: "⏳",
            arrow_right: "▶",
            arrow_down: "▼",
            bullet: "•",
        }
    }
    
    /// ASCII fallback icon set
    pub fn ascii() -> Self {
        Self {
            success: "[OK]",
            error: "[ERROR]",
            warning: "[WARN]",
            info: "[INFO]",
            file: "[FILE]",
            directory: "[DIR]",
            loading: "[...]",
            arrow_right: ">",
            arrow_down: "v",
            bullet: "*",
        }
    }
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            show_header: true,
            alternate_rows: true,
            show_row_numbers: false,
            show_borders: true,
            header_style: TextStyle {
                color: Some(Color::White),
                background: Some(Color::Blue),
                bold: true,
                italic: false,
                underline: false,
                strikethrough: false,
            },
            cell_style: TextStyle {
                color: None,
                background: None,
                bold: false,
                italic: false,
                underline: false,
                strikethrough: false,
            },
            border_chars: BorderChars::unicode(),
        }
    }
}

impl BorderChars {
    /// Unicode box drawing characters
    pub fn unicode() -> Self {
        Self {
            horizontal: '─',
            vertical: '│',
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            cross: '┼',
            tee_down: '┬',
            tee_up: '┴',
            tee_right: '├',
            tee_left: '┤',
        }
    }
    
    /// ASCII fallback characters
    pub fn ascii() -> Self {
        Self {
            horizontal: '-',
            vertical: '|',
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            cross: '+',
            tee_down: '+',
            tee_up: '+',
            tee_right: '+',
            tee_left: '+',
        }
    }
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            width: 40,
            show_percentage: true,
            show_counts: false,
            style: ProgressStyle::Blocks,
        }
    }
}

impl AdvancedCUI {
    /// Create new CUI design system instance
    pub fn new() -> Result<Self> {
        let terminal_size = terminal::size().unwrap_or((80, 24));
        let color_support = Self::detect_color_support();
        let unicode_support = Self::detect_unicode_support();
        
        Ok(Self {
            theme: Theme::default(),
            terminal_size,
            color_support,
            unicode_support,
            layout: LayoutConfig::default(),
        })
    }
    
    /// Create with custom theme
    pub fn with_theme(theme: Theme) -> Result<Self> {
        let mut cui = Self::new()?;
        cui.theme = theme;
        Ok(cui)
    }
    
    /// Detect if terminal supports colors
    fn detect_color_support() -> bool {
        if let Ok(term) = std::env::var("TERM") {
            !term.contains("mono") && term != "dumb"
        } else {
            true // Assume color support by default
        }
    }
    
    /// Detect if terminal supports Unicode
    fn detect_unicode_support() -> bool {
        if let Ok(lang) = std::env::var("LANG") {
            lang.contains("UTF-8") || lang.contains("utf8")
        } else {
            true // Assume Unicode support by default
        }
    }
    
    /// Get appropriate icon set based on capabilities
    pub fn icons(&self) -> IconSet {
        if self.unicode_support {
            IconSet::unicode()
        } else {
            IconSet::ascii()
        }
    }
    
    /// Get appropriate border characters
    pub fn border_chars(&self) -> BorderChars {
        if self.unicode_support {
            BorderChars::unicode()
        } else {
            BorderChars::ascii()
        }
    }
    
    /// Format a beautiful table
    pub fn format_table(&self, headers: &[String], rows: &[Vec<String>]) -> Result<String> {
        let mut output = String::new();
        let style = TableStyle::default();
        let chars = self.border_chars();
        
        if rows.is_empty() {
            return Ok(self.format_info_message("No data to display"));
        }
        
        // Calculate column widths
        let mut col_widths = vec![0; headers.len()];
        
        // Check header widths
        for (i, header) in headers.iter().enumerate() {
            col_widths[i] = col_widths[i].max(header.width());
        }
        
        // Check data widths
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.width());
                }
            }
        }
        
        // Apply min/max constraints
        for width in &mut col_widths {
            *width = (*width).clamp(self.layout.min_column_width, self.layout.max_column_width);
        }
        
        // Top border
        if style.show_borders {
            output.push(chars.top_left);
            for (i, &width) in col_widths.iter().enumerate() {
                output.push_str(&chars.horizontal.to_string().repeat(width + 2));
                if i < col_widths.len() - 1 {
                    output.push(chars.tee_down);
                }
            }
            output.push(chars.top_right);
            output.push('\n');
        }
        
        // Header row
        if style.show_header {
            if style.show_borders {
                output.push(chars.vertical);
            }
            
            for (i, header) in headers.iter().enumerate() {
                output.push(' ');
                
                // Apply header styling if colors are supported
                if self.color_support {
                    output.push_str(&format!("\x1b[1;37;44m{:width$}\x1b[0m", 
                        header, width = col_widths[i]));
                } else {
                    output.push_str(&format!("{:width$}", header, width = col_widths[i]));
                }
                
                output.push(' ');
                
                if style.show_borders && i < col_widths.len() - 1 {
                    output.push(chars.vertical);
                }
            }
            
            if style.show_borders {
                output.push(chars.vertical);
            }
            output.push('\n');
            
            // Header separator
            if style.show_borders {
                output.push(chars.tee_right);
                for (i, &width) in col_widths.iter().enumerate() {
                    output.push_str(&chars.horizontal.to_string().repeat(width + 2));
                    if i < col_widths.len() - 1 {
                        output.push(chars.cross);
                    }
                }
                output.push(chars.tee_left);
                output.push('\n');
            }
        }
        
        // Data rows
        for (row_idx, row) in rows.iter().enumerate() {
            if style.show_borders {
                output.push(chars.vertical);
            }
            
            for (i, cell) in row.iter().enumerate() {
                if i >= col_widths.len() { break; }
                
                output.push(' ');
                
                // Apply alternating row colors if supported
                if self.color_support && style.alternate_rows && row_idx % 2 == 1 {
                    output.push_str(&format!("\x1b[47;30m{:width$}\x1b[0m", 
                        cell, width = col_widths[i]));
                } else {
                    output.push_str(&format!("{:width$}", cell, width = col_widths[i]));
                }
                
                output.push(' ');
                
                if style.show_borders && i < col_widths.len() - 1 {
                    output.push(chars.vertical);
                }
            }
            
            if style.show_borders {
                output.push(chars.vertical);
            }
            output.push('\n');
        }
        
        // Bottom border
        if style.show_borders {
            output.push(chars.bottom_left);
            for (i, &width) in col_widths.iter().enumerate() {
                output.push_str(&chars.horizontal.to_string().repeat(width + 2));
                if i < col_widths.len() - 1 {
                    output.push(chars.tee_up);
                }
            }
            output.push(chars.bottom_right);
            output.push('\n');
        }
        
        Ok(output)
    }
    
    /// Format a success message
    pub fn format_success_message(&self, message: &str) -> String {
        let icons = self.icons();
        if self.color_support {
            format!("\x1b[32m{} {}\x1b[0m", icons.success, message)
        } else {
            format!("{} {}", icons.success, message)
        }
    }
    
    /// Format an error message
    pub fn format_error_message(&self, message: &str) -> String {
        let icons = self.icons();
        if self.color_support {
            format!("\x1b[31m{} {}\x1b[0m", icons.error, message)
        } else {
            format!("{} {}", icons.error, message)
        }
    }
    
    /// Format a warning message
    pub fn format_warning_message(&self, message: &str) -> String {
        let icons = self.icons();
        if self.color_support {
            format!("\x1b[33m{} {}\x1b[0m", icons.warning, message)
        } else {
            format!("{} {}", icons.warning, message)
        }
    }
    
    /// Format an info message
    pub fn format_info_message(&self, message: &str) -> String {
        let icons = self.icons();
        if self.color_support {
            format!("\x1b[36m{} {}\x1b[0m", icons.info, message)
        } else {
            format!("{} {}", icons.info, message)
        }
    }
    
    /// Format a progress bar
    pub fn format_progress_bar(&self, current: u64, total: u64, message: Option<&str>) -> String {
        let config = ProgressConfig::default();
        let percentage = if total == 0 { 100.0 } else { (current as f64 / total as f64) * 100.0 };
        let filled_width = ((percentage / 100.0) * config.width as f64) as usize;
        let empty_width = config.width - filled_width;
        
        let (filled_char, empty_char) = match config.style {
            ProgressStyle::Blocks => ('█', '░'),
            ProgressStyle::Bars => ('━', '┅'),
            ProgressStyle::Ascii => ('#', '.'),
            ProgressStyle::Dots => ('●', '○'),
        };
        
        let mut output = String::new();
        
        if let Some(msg) = message {
            output.push_str(msg);
            output.push_str(": ");
        }
        
        output.push('[');
        output.push_str(&filled_char.to_string().repeat(filled_width));
        output.push_str(&empty_char.to_string().repeat(empty_width));
        output.push(']');
        
        if config.show_percentage {
            output.push_str(&format!(" {:.1}%", percentage));
        }
        
        if config.show_counts {
            output.push_str(&format!(" ({}/{})", current, total));
        }
        
        if self.color_support {
            format!("\x1b[36m{}\x1b[0m", output)
        } else {
            output
        }
    }
    
    /// Format a section header
    pub fn format_section_header(&self, title: &str) -> String {
        let chars = self.border_chars();
        let width = self.terminal_size.0 as usize - 4;
        let title_with_spaces = format!(" {} ", title);
        let border_width = (width - title_with_spaces.len()) / 2;
        
        let mut output = String::new();
        output.push_str(&chars.horizontal.to_string().repeat(border_width));
        
        if self.color_support {
            output.push_str(&format!("\x1b[1;37m{}\x1b[0m", title_with_spaces));
        } else {
            output.push_str(&title_with_spaces);
        }
        
        output.push_str(&chars.horizontal.to_string().repeat(border_width));
        output
    }
    
    /// Format a key-value pair list
    pub fn format_key_value_list(&self, items: &[(&str, &str)]) -> String {
        let mut output = String::new();
        let max_key_width = items.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
        
        for (key, value) in items {
            if self.color_support {
                output.push_str(&format!("\x1b[36m{:width$}\x1b[0m: {}\n", 
                    key, value, width = max_key_width));
            } else {
                output.push_str(&format!("{:width$}: {}\n", key, value, width = max_key_width));
            }
        }
        
        output
    }
    
    /// Format a bulleted list
    pub fn format_bullet_list(&self, items: &[&str]) -> String {
        let icons = self.icons();
        let mut output = String::new();
        
        for item in items {
            if self.color_support {
                output.push_str(&format!("\x1b[90m{}\x1b[0m {}\n", icons.bullet, item));
            } else {
                output.push_str(&format!("{} {}\n", icons.bullet, item));
            }
        }
        
        output
    }
    
    /// Create a beautiful panel with border
    pub fn format_panel(&self, title: &str, content: &str) -> String {
        let chars = self.border_chars();
        let lines: Vec<&str> = content.lines().collect();
        let max_content_width = lines.iter().map(|line| line.width()).max().unwrap_or(0);
        let title_width = title.width();
        let panel_width = (max_content_width.max(title_width) + 4).min(self.terminal_size.0 as usize - 2);
        
        let mut output = String::new();
        
        // Top border with title
        output.push(chars.top_left);
        output.push(' ');
        
        if self.color_support {
            output.push_str(&format!("\x1b[1;37m{}\x1b[0m", title));
        } else {
            output.push_str(title);
        }
        
        let remaining_width = panel_width - title.width() - 3;
        output.push(' ');
        output.push_str(&chars.horizontal.to_string().repeat(remaining_width));
        output.push(chars.top_right);
        output.push('\n');
        
        // Content lines
        for line in lines {
            output.push(chars.vertical);
            output.push(' ');
            output.push_str(&format!("{:width$}", line, width = panel_width - 4));
            output.push(' ');
            output.push(chars.vertical);
            output.push('\n');
        }
        
        // Bottom border
        output.push(chars.bottom_left);
        output.push_str(&chars.horizontal.to_string().repeat(panel_width - 2));
        output.push(chars.bottom_right);
        output.push('\n');
        
        output
    }
}

/// Convenience functions for direct formatting
impl AdvancedCUI {
    /// Quick success message
    pub fn success(message: &str) -> String {
        Self::new().unwrap().format_success_message(message)
    }
    
    /// Quick error message
    pub fn error(message: &str) -> String {
        Self::new().unwrap().format_error_message(message)
    }
    
    /// Quick warning message
    pub fn warning(message: &str) -> String {
        Self::new().unwrap().format_warning_message(message)
    }
    
    /// Quick info message
    pub fn info(message: &str) -> String {
        Self::new().unwrap().format_info_message(message)
    }
    
    /// Quick table formatting
    pub fn table(headers: &[String], rows: &[Vec<String>]) -> String {
        Self::new().unwrap().format_table(headers, rows).unwrap_or_else(|_| "Error formatting table".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cui_creation() {
        let cui = AdvancedCUI::new().unwrap();
        // Test that we have a valid theme by checking default color values
        assert!(matches!(cui.theme.primary, crossterm::style::Color::Blue | 
                                           crossterm::style::Color::Rgb { .. } | 
                                           crossterm::style::Color::AnsiValue(_)));
    }

    #[test]
    fn test_message_formatting() {
        let cui = AdvancedCUI::new().unwrap();
        
        let success = cui.format_success_message("Operation completed");
        assert!(success.contains("Operation completed"));
        
        let error = cui.format_error_message("Something went wrong");
        assert!(error.contains("Something went wrong"));
    }

    #[test]
    fn test_table_formatting() {
        let cui = AdvancedCUI::new().unwrap();
        let headers = vec!["Name".to_string(), "Age".to_string()];
        let rows = vec![
            vec!["Alice".to_string(), "25".to_string()],
            vec!["Bob".to_string(), "30".to_string()],
        ];
        
        let table = cui.format_table(&headers, &rows).unwrap();
        assert!(table.contains("Name"));
        assert!(table.contains("Alice"));
    }
}
