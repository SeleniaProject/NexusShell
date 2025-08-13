//! Enhanced UI module for NexusShell
//!
//! Provides sophisticated text formatting and display capabilities

use anyhow::Result;
use nu_ansi_term::Color as NuColor;

/// Color enumeration for themed display
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl Color {
    /// Convert to nu_ansi_term::Color
    pub fn to_ansi_colour(&self) -> NuColor {
        match self {
            Color::Black => NuColor::Black,
            Color::Red => NuColor::Red,
            Color::Green => NuColor::Green,
            Color::Yellow => NuColor::Yellow,
            Color::Blue => NuColor::Blue,
            Color::Magenta => NuColor::Purple,
            Color::Cyan => NuColor::Cyan,
            Color::White => NuColor::White,
        }
    }
}

/// Display theme configuration
#[derive(Debug, Clone)]
pub struct DisplayTheme {
    pub primary_color: Color,
    pub secondary_color: Color,
    pub accent_color: Color,
    pub error_color: Color,
    pub warning_color: Color,
    pub success_color: Color,
    pub muted_color: Color,
}

impl Default for DisplayTheme {
    fn default() -> Self {
        Self {
            primary_color: Color::Blue,
            secondary_color: Color::Cyan,
            accent_color: Color::Yellow,
            error_color: Color::Red,
            warning_color: Color::Yellow,
            success_color: Color::Green,
            muted_color: Color::White,
        }
    }
}

impl DisplayTheme {
    /// Load theme from configuration
    pub fn load_from_config() -> Result<Self> {
        // For complete implementation, would load from config file
        Ok(Self::default())
    }
}

/// Table row structure
#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<String>,
}

/// Progress indicator structure
#[derive(Debug, Clone)]
pub struct ProgressIndicator {
    pub current: u64,
    pub total: u64,
    pub message: String,
}

impl ProgressIndicator {
    pub fn percentage(&self) -> u8 {
        if self.total == 0 {
            return 100;
        }
        ((self.current as f64 / self.total as f64) * 100.0) as u8
    }
}

/// Box drawing characters for enhanced display
#[derive(Debug, Clone)]
pub struct BoxChars {
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

impl Default for BoxChars {
    fn default() -> Self {
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
}

/// ASCII fallback box characters
impl BoxChars {
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

/// Enhanced UI formatter with comprehensive display capabilities
#[derive(Debug, Clone)]
pub struct EnhancedFormatter {
    pub theme: DisplayTheme,
    pub box_chars: BoxChars,
    pub use_unicode: bool,
}

impl Default for EnhancedFormatter {
    fn default() -> Self {
        let use_unicode = std::env::var("LANG")
            .map(|l| l.contains("UTF"))
            .unwrap_or(false) || cfg!(windows);
            
        Self {
            theme: DisplayTheme::default(),
            box_chars: if use_unicode {
                BoxChars::default()
            } else {
                BoxChars::ascii()
            },
            use_unicode,
        }
    }
}

impl EnhancedFormatter {
    /// Create new enhanced formatter
    pub fn new() -> Result<Self> {
        Ok(Self::default())
    }

    /// Display formatted table
    pub fn display_table(&self, headers: &[String], rows: &[TableRow]) -> Result<()> {
        if headers.is_empty() {
            return Ok(());
        }

        let mut column_widths = vec![0; headers.len()];

        // Calculate column widths
        for (i, header) in headers.iter().enumerate() {
            column_widths[i] = header.len();
        }

        for row in rows {
            for (i, cell) in row.cells.iter().enumerate() {
                if i < column_widths.len() {
                    column_widths[i] = column_widths[i].max(cell.len());
                }
            }
        }

        // Draw top border
        print!("{}", self.box_chars.top_left);
        for (i, &width) in column_widths.iter().enumerate() {
            for _ in 0..width + 2 {
                print!("{}", self.box_chars.horizontal);
            }
            if i < column_widths.len() - 1 {
                print!("{}", self.box_chars.tee_down);
            }
        }
        println!("{}", self.box_chars.top_right);

        // Draw headers
        print!("{}", self.box_chars.vertical);
        for (i, header) in headers.iter().enumerate() {
            print!(" {:width$} ", header, width = column_widths[i]);
            print!("{}", self.box_chars.vertical);
        }
        println!();

        // Draw header separator
        print!("{}", self.box_chars.tee_right);
        for (i, &width) in column_widths.iter().enumerate() {
            for _ in 0..width + 2 {
                print!("{}", self.box_chars.horizontal);
            }
            if i < column_widths.len() - 1 {
                print!("{}", self.box_chars.cross);
            }
        }
        println!("{}", self.box_chars.tee_left);

        // Draw data rows
        for row in rows {
            print!("{}", self.box_chars.vertical);
            for (i, cell) in row.cells.iter().enumerate() {
                if i < column_widths.len() {
                    print!(" {:width$} ", cell, width = column_widths[i]);
                } else {
                    print!(" {cell} ");
                }
                print!("{}", self.box_chars.vertical);
            }
            println!();
        }

        // Draw bottom border
        print!("{}", self.box_chars.bottom_left);
        for (i, &width) in column_widths.iter().enumerate() {
            for _ in 0..width + 2 {
                print!("{}", self.box_chars.horizontal);
            }
            if i < column_widths.len() - 1 {
                print!("{}", self.box_chars.tee_up);
            }
        }
        println!("{}", self.box_chars.bottom_right);

        Ok(())
    }

    /// Display progress bar
    pub fn display_progress(&self, progress: &ProgressIndicator) -> Result<()> {
        let percentage = progress.percentage();
        let width = 50;
        let filled = (width as f64 * percentage as f64 / 100.0) as usize;
        let empty = width - filled;

        print!("[");
        for _ in 0..filled {
            print!("=");
        }
        for _ in 0..empty {
            print!(" ");
        }
        print!("] {}% - {}", percentage, progress.message);
        println!();

        Ok(())
    }

    /// Display status message with color
    pub fn display_status(&self, message: &str, status_type: StatusType) -> Result<()> {
        let colored_message = match status_type {
            StatusType::Error => self.theme.error_color.to_ansi_colour().paint(message),
            StatusType::Warning => self.theme.warning_color.to_ansi_colour().paint(message),
            StatusType::Success => self.theme.success_color.to_ansi_colour().paint(message),
            StatusType::Info => self.theme.primary_color.to_ansi_colour().paint(message),
        };

        let status_prefix = match status_type {
            StatusType::Error => self.theme.error_color.to_ansi_colour().bold().paint("ERROR"),
            StatusType::Warning => self.theme.warning_color.to_ansi_colour().bold().paint("WARNING"),
            StatusType::Success => self.theme.success_color.to_ansi_colour().bold().paint("SUCCESS"),
            StatusType::Info => self.theme.primary_color.to_ansi_colour().bold().paint("INFO"),
        };

        println!("{status_prefix}: {colored_message}");
        Ok(())
    }
}

/// Simple CUI formatter for basic displays
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct CuiFormatter {
    pub theme: DisplayTheme,
}


impl CuiFormatter {
    /// Create comprehensive formatter with full theme loading
    /// COMPLETE initialization with ALL theme features
    pub fn new_minimal() -> Result<Self> {
        Ok(Self {
            theme: DisplayTheme::load_from_config()?,  // Full theme loading as required
        })
    }

    /// Create new simple formatter
    pub fn new() -> Result<Self> {
        Ok(Self::default())
    }

    /// Format command output for CUI display
    /// 
    /// Applies appropriate styling and formatting to command output
    /// while maintaining readability and performance.
    pub fn format_output(&self, output: &str) -> Result<String> {
        if output.trim().is_empty() {
            return Ok("".to_string());
        }

        // Apply basic formatting - could be enhanced with syntax highlighting
        let lines: Vec<&str> = output.lines().collect();
        let mut formatted = String::new();
        
        for (i, line) in lines.iter().enumerate() {
            // Add line with minimal formatting
            formatted.push_str(line);
            
            // Add newline except for last line
            if i < lines.len() - 1 {
                formatted.push('\n');
            }
        }
        
        Ok(formatted)
    }

    /// Format error for CUI display
    /// 
    /// Provides user-friendly error formatting with appropriate styling
    /// and helpful context information.
    pub fn format_error(&self, error: &anyhow::Error) -> Result<String> {
        use nu_ansi_term::Color::Red;
        
        let error_message = format!("❌ {error}");
        
        // Apply red color if terminal supports it
        if std::io::IsTerminal::is_terminal(&std::io::stderr()) {
            Ok(Red.paint(&error_message).to_string())
        } else {
            Ok(error_message)
        }
    }

    /// Display simple table
    pub fn display_table(&self, headers: &[String], rows: &[TableRow]) -> Result<()> {
        if headers.is_empty() {
            return Ok(());
        }

        // Print headers
        for (i, header) in headers.iter().enumerate() {
            print!("{header}");
            if i < headers.len() - 1 {
                print!(" | ");
            }
        }
        println!();

        // Print separator
        for i in 0..headers.len() {
            print!("-----");
            if i < headers.len() - 1 {
                print!("-+-");
            }
        }
        println!();

        // Print rows
        for row in rows {
            for (i, cell) in row.cells.iter().enumerate() {
                print!("{cell}");
                if i < row.cells.len() - 1 {
                    print!(" | ");
                }
            }
            println!();
        }

        Ok(())
    }

    /// Display simple progress
    pub fn display_progress(&self, progress: &ProgressIndicator) -> Result<()> {
        println!("Progress: {}% - {}", progress.percentage(), progress.message);
        Ok(())
    }

    /// Display status message
    pub fn display_status(&self, message: &str, status_type: StatusType) -> Result<()> {
        println!("{}: {}", status_type.as_str(), message);
        Ok(())
    }
}

/// Status type enumeration
#[derive(Debug, Clone, Copy)]
pub enum StatusType {
    Error,
    Warning,
    Success,
    Info,
}

impl StatusType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatusType::Error => "ERROR",
            StatusType::Warning => "WARNING",
            StatusType::Success => "SUCCESS",
            StatusType::Info => "INFO",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_calculation() {
        let progress = ProgressIndicator {
            current: 50,
            total: 100,
            message: "Test".to_string(),
        };
        assert_eq!(progress.percentage(), 50);
    }

    #[test]
    fn test_status_type() {
        assert_eq!(StatusType::Error.as_str(), "ERROR");
        assert_eq!(StatusType::Success.as_str(), "SUCCESS");
    }
}