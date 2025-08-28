/// CUI Output Formatter for NexusShell
/// 
/// This module provides structured output formatting for CUI mode,
/// replacing TUI widgets with ANSI-formatted text output that maintains
/// readability and follows the design specifications.
/// 
/// Features:
/// - Table formatting with automatic column width adjustment
/// - Error and success message formatting
/// - Progress bar display using ANSI characters
/// - File listing with icons and colors
/// - System information display

use anyhow::{Result, Context};
use crossterm::{
    style::{Color, Print, ResetColor, SetForegroundColor, SetBackgroundColor, Attribute, SetAttribute},
    execute,
};
use std::{
    collections::HashMap,
    fmt::Write as FmtWrite,
    io::{self, Write},
};
use nxsh_core::executor::ExecutionResult;

/// CUI output formatter with ANSI styling
pub struct CUIOutputFormatter {
    /// Terminal width for formatting
    terminal_width: usize,
    
    /// Whether colors are enabled
    colors_enabled: bool,
    
    /// Whether unicode icons are enabled
    icons_enabled: bool,
    
    /// Column separator string
    column_separator: String,
}

/// Table formatting configuration
#[derive(Debug, Clone)]
pub struct TableConfig {
    /// Show borders around table
    pub show_borders: bool,
    
    /// Show header row
    pub show_header: bool,
    
    /// Alternate row colors
    pub alternate_rows: bool,
    
    /// Maximum column width
    pub max_column_width: usize,
    
    /// Minimum column width
    pub min_column_width: usize,
}

impl Default for TableConfig {
    fn default() -> Self {
        Self {
            show_borders: true,
            show_header: true,
            alternate_rows: true,
            max_column_width: 50,
            min_column_width: 8,
        }
    }
}

/// Progress bar configuration
#[derive(Debug, Clone)]
pub struct ProgressConfig {
    /// Total width of progress bar
    pub width: usize,
    
    /// Characters to use for filled/empty portions
    pub fill_char: char,
    pub empty_char: char,
    
    /// Show percentage
    pub show_percentage: bool,
    
    /// Show ETA
    pub show_eta: bool,
    
    /// Show speed
    pub show_speed: bool,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            width: 40,
            fill_char: '█',
            empty_char: '─',
            show_percentage: true,
            show_eta: true,
            show_speed: true,
        }
    }
}

impl CUIOutputFormatter {
    /// Create new output formatter
    pub fn new() -> Result<Self> {
        let terminal_width = Self::get_terminal_width();
        
        Ok(Self {
            terminal_width,
            colors_enabled: Self::colors_supported(),
            icons_enabled: Self::unicode_supported(),
            column_separator: " │ ".to_string(),
        })
    }
    
    /// Display command execution result
    pub fn display_result(&self, result: &ExecutionResult) -> Result<()> {
        // Display stdout if present
        if !result.stdout.is_empty() {
            print!("{}", result.stdout);
            io::stdout().flush()
                .context("Failed to flush stdout")?;
        }
        
        // Display stderr if present
        if !result.stderr.is_empty() {
            eprintln!("{}", result.stderr);
        }
        
        Ok(())
    }
    
    /// Display error message with formatting
    pub fn display_error(&self, error: &anyhow::Error) -> Result<()> {
        let mut output = String::new();
        
        if self.colors_enabled {
            // Red error icon and text
            output.push_str("\x1b[1;31m");
            if self.icons_enabled {
                output.push_str("✘ Error: ");
            } else {
                output.push_str("ERROR: ");
            }
            output.push_str("\x1b[0m");
            
            // Error message in red
            output.push_str("\x1b[31m");
            output.push_str(&error.to_string());
            output.push_str("\x1b[0m");
        } else {
            output.push_str("ERROR: ");
            output.push_str(&error.to_string());
        }
        
        output.push('\n');
        
        print!("{}", output);
        io::stdout().flush()
            .context("Failed to flush error output")?;
        
        Ok(())
    }
    
    /// Display success message with formatting
    pub fn display_success(&self, message: &str) -> Result<()> {
        let mut output = String::new();
        
        if self.colors_enabled {
            // Green success icon and text
            output.push_str("\x1b[1;32m");
            if self.icons_enabled {
                output.push_str("✓ ");
            }
            output.push_str(message);
            output.push_str("\x1b[0m");
        } else {
            output.push_str(message);
        }
        
        output.push('\n');
        
        print!("{}", output);
        io::stdout().flush()
            .context("Failed to flush success output")?;
        
        Ok(())
    }
    
    /// Display warning message with formatting
    pub fn display_warning(&self, message: &str) -> Result<()> {
        let mut output = String::new();
        
        if self.colors_enabled {
            // Yellow warning icon and text
            output.push_str("\x1b[1;33m");
            if self.icons_enabled {
                output.push_str("⚠ Warning: ");
            } else {
                output.push_str("WARNING: ");
            }
            output.push_str(message);
            output.push_str("\x1b[0m");
        } else {
            output.push_str("WARNING: ");
            output.push_str(message);
        }
        
        output.push('\n');
        
        print!("{}", output);
        io::stdout().flush()
            .context("Failed to flush warning output")?;
        
        Ok(())
    }
    
    /// Display table with automatic column width adjustment
    pub fn display_table(&self, headers: &[String], rows: &[Vec<String>], config: &TableConfig) -> Result<()> {
        if headers.is_empty() || rows.is_empty() {
            return Ok(());
        }
        
        // Calculate column widths
        let column_widths = self.calculate_column_widths(headers, rows, config);
        
        let mut output = String::new();
        
        // Display header if enabled
        if config.show_header {
            self.format_table_row(&mut output, headers, &column_widths, true, config)?;
            
            if config.show_borders {
                self.format_table_separator(&mut output, &column_widths)?;
            }
        }
        
        // Display data rows
        for (i, row) in rows.iter().enumerate() {
            let is_alternate = config.alternate_rows && i % 2 == 1;
            self.format_table_row(&mut output, row, &column_widths, is_alternate, config)?;
        }
        
        print!("{}", output);
        io::stdout().flush()
            .context("Failed to flush table output")?;
        
        Ok(())
    }
    
    /// Display list with optional icons
    pub fn display_list(&self, items: &[String]) -> Result<()> {
        for item in items {
            if self.icons_enabled {
                print!("• {}\n", item);
            } else {
                print!("- {}\n", item);
            }
        }
        
        io::stdout().flush()
            .context("Failed to flush list output")?;
        
        Ok(())
    }
    
    /// Display progress bar
    pub fn display_progress(&self, current: u64, total: u64, config: &ProgressConfig) -> Result<()> {
        let percentage = if total > 0 {
            (current as f64 / total as f64 * 100.0) as u8
        } else {
            0
        };
        
        let filled = if total > 0 {
            (current as f64 / total as f64 * config.width as f64) as usize
        } else {
            0
        };
        let empty = config.width.saturating_sub(filled);
        
        let mut output = String::new();
        
        // Progress bar
        output.push('[');
        for _ in 0..filled {
            output.push(config.fill_char);
        }
        for _ in 0..empty {
            output.push(config.empty_char);
        }
        output.push(']');
        
        // Percentage
        if config.show_percentage {
            write!(output, " {}%", percentage)?;
        }
        
        // ETA and speed would be calculated by caller and passed as parameters
        // This is a simplified implementation
        
        print!("\r{}", output);
        if let Err(e) = io::stdout().flush() {
            // Log the error but don't fail the entire operation
            eprintln!("Warning: Failed to flush progress output: {}", e);
        }
        
        Ok(())
    }
    
    /// Format a single table row
    fn format_table_row(
        &self,
        output: &mut String,
        row: &[String],
        widths: &[usize],
        is_header_or_alternate: bool,
        config: &TableConfig,
    ) -> Result<()> {
        if config.show_borders {
            output.push('│');
        }
        
        for (i, (cell, &width)) in row.iter().zip(widths.iter()).enumerate() {
            if i > 0 {
                output.push_str(&self.column_separator);
            } else {
                output.push(' ');
            }
            
            // Ensure width is at least 1 to prevent panic
            let safe_width = width.max(1);
            
            // Truncate cell if too long, handling Unicode properly
            let cell_content = if cell.chars().count() > safe_width {
                let truncated: String = cell.chars().take(safe_width.saturating_sub(1)).collect();
                format!("{}…", truncated)
            } else {
                format!("{:<width$}", cell, width = safe_width)
            };
            
            if self.colors_enabled && is_header_or_alternate {
                // Apply styling for headers or alternate rows
                output.push_str("\x1b[1m");
                output.push_str(&cell_content);
                output.push_str("\x1b[0m");
            } else {
                output.push_str(&cell_content);
            }
            
            output.push(' ');
        }
        
        if config.show_borders {
            output.push('│');
        }
        
        output.push('\n');
        Ok(())
    }
    
    /// Format table separator line
    fn format_table_separator(&self, output: &mut String, widths: &[usize]) -> Result<()> {
        output.push('├');
        
        for (i, &width) in widths.iter().enumerate() {
            if i > 0 {
                output.push_str("┼");
            }
            
            // Add padding for separator width calculation
            let separator_width = width + self.column_separator.len();
            for _ in 0..separator_width {
                output.push('─');
            }
        }
        
        output.push_str("┤\n");
        Ok(())
    }
    
    /// Calculate optimal column widths
    fn calculate_column_widths(
        &self,
        headers: &[String],
        rows: &[Vec<String>],
        config: &TableConfig,
    ) -> Vec<usize> {
        let num_columns = headers.len();
        let mut widths = vec![config.min_column_width; num_columns];
        
        // Check header widths
        for (i, header) in headers.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(header.len());
            }
        }
        
        // Check row widths
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }
        
        // Apply maximum width constraint
        for width in &mut widths {
            *width = (*width).min(config.max_column_width);
        }
        
        // Adjust to fit terminal width
        let total_width: usize = widths.iter().sum::<usize>() 
            + (num_columns - 1) * self.column_separator.len()
            + if config.show_borders { 4 } else { 0 }; // Border padding
        
        if total_width > self.terminal_width {
            self.adjust_column_widths_to_fit(&mut widths, total_width);
        }
        
        widths
    }
    
    /// Adjust column widths to fit terminal
    fn adjust_column_widths_to_fit(&self, widths: &mut [usize], total_width: usize) {
        let excess = total_width - self.terminal_width;
        let avg_reduction = excess / widths.len();
        
        for width in widths.iter_mut() {
            *width = (*width).saturating_sub(avg_reduction);
            *width = (*width).max(8); // Minimum readable width
        }
    }
    
    /// Get terminal width
    fn get_terminal_width() -> usize {
        if let Ok((width, _)) = crossterm::terminal::size() {
            width as usize
        } else {
            80 // Default fallback
        }
    }
    
    /// Check if colors are supported
    fn colors_supported() -> bool {
        // Check TERM environment variable and NO_COLOR
        if std::env::var("NO_COLOR").is_ok() {
            return false;
        }
        
        if let Ok(term) = std::env::var("TERM") {
            !term.contains("dumb") && term != "unknown"
        } else {
            true // Default to enabled
        }
    }
    
    /// Check if unicode is supported
    fn unicode_supported() -> bool {
        // Check locale and terminal capabilities
        if let Ok(lang) = std::env::var("LANG") {
            lang.to_lowercase().contains("utf")
        } else if let Ok(lc_all) = std::env::var("LC_ALL") {
            lc_all.to_lowercase().contains("utf")
        } else {
            true // Default to enabled
        }
    }
    
    /// Set whether colors are enabled
    pub fn set_colors_enabled(&mut self, enabled: bool) {
        self.colors_enabled = enabled;
    }
    
    /// Set whether icons are enabled
    pub fn set_icons_enabled(&mut self, enabled: bool) {
        self.icons_enabled = enabled;
    }
    
    /// Get current colors enabled state
    pub fn colors_enabled(&self) -> bool {
        self.colors_enabled
    }
    
    /// Get current icons enabled state  
    pub fn icons_enabled(&self) -> bool {
        self.icons_enabled
    }
}
