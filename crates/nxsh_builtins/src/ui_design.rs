/// Advanced CUI Design System for NexusShell
/// 
/// This module provides a comprehensive, beautiful UI design system for all shell commands.
/// Features modern terminal UI with colors, icons, tables, sophisticated formatting,
/// animations, progress indicators, and dynamic theming capabilities.

use std::fmt;
use std::time::{Duration, Instant};
use std::thread;
use std::io::{self, Write};

/// Colorize trait for adding colors to strings
pub trait Colorize {
    fn primary(self) -> String;
    fn secondary(self) -> String;
    fn success(self) -> String;
    fn warning(self) -> String;
    fn error(self) -> String;
    fn info(self) -> String;
    fn muted(self) -> String;
    fn bright(self) -> String;
    fn dim(self) -> String;
    fn accent(self) -> String;
    fn highlight(self) -> String;
    fn colorize(self, color: &str) -> String;
}

impl Colorize for &str {
    fn primary(self) -> String { format!("\x1b[38;5;39m{}\x1b[0m", self) }
    fn secondary(self) -> String { format!("\x1b[38;5;141m{}\x1b[0m", self) }
    fn success(self) -> String { format!("\x1b[38;5;46m{}\x1b[0m", self) }
    fn warning(self) -> String { format!("\x1b[38;5;220m{}\x1b[0m", self) }
    fn error(self) -> String { format!("\x1b[38;5;196m{}\x1b[0m", self) }
    fn info(self) -> String { format!("\x1b[38;5;51m{}\x1b[0m", self) }
    fn muted(self) -> String { format!("\x1b[38;5;244m{}\x1b[0m", self) }
    fn bright(self) -> String { format!("\x1b[1m{}\x1b[0m", self) }
    fn dim(self) -> String { format!("\x1b[2m{}\x1b[0m", self) }
    fn accent(self) -> String { format!("\x1b[38;5;208m{}\x1b[0m", self) }
    fn highlight(self) -> String { format!("\x1b[48;5;234m{}\x1b[0m", self) }
    fn colorize(self, color: &str) -> String { format!("{}{}\x1b[0m", color, self) }
}

impl Colorize for String {
    fn primary(self) -> String { format!("\x1b[38;5;39m{}\x1b[0m", self) }
    fn secondary(self) -> String { format!("\x1b[38;5;141m{}\x1b[0m", self) }
    fn success(self) -> String { format!("\x1b[38;5;46m{}\x1b[0m", self) }
    fn warning(self) -> String { format!("\x1b[38;5;220m{}\x1b[0m", self) }
    fn error(self) -> String { format!("\x1b[38;5;196m{}\x1b[0m", self) }
    fn info(self) -> String { format!("\x1b[38;5;51m{}\x1b[0m", self) }
    fn muted(self) -> String { format!("\x1b[38;5;244m{}\x1b[0m", self) }
    fn bright(self) -> String { format!("\x1b[1m{}\x1b[0m", self) }
    fn dim(self) -> String { format!("\x1b[2m{}\x1b[0m", self) }
    fn accent(self) -> String { format!("\x1b[38;5;208m{}\x1b[0m", self) }
    fn highlight(self) -> String { format!("\x1b[48;5;234m{}\x1b[0m", self) }
    fn colorize(self, color: &str) -> String { format!("{}{}\x1b[0m", color, self) }
}

/// Advanced color palette with gradient and theme support
#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub primary: &'static str,
    pub secondary: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    pub error: &'static str,
    pub info: &'static str,
    pub muted: &'static str,
    pub bright: &'static str,
    pub dim: &'static str,
    pub accent: &'static str,
    pub highlight: &'static str,
    pub background: &'static str,
    pub border: &'static str,
}

impl ColorPalette {
    // Constant access methods for backward compatibility
    pub const BORDER: &'static str = "\x1b[38;5;240m";
    pub const ACCENT: &'static str = "\x1b[38;5;208m";
    pub const INFO: &'static str = "\x1b[38;5;51m";
    pub const SUCCESS: &'static str = "\x1b[38;5;46m";
    pub const WARNING: &'static str = "\x1b[38;5;220m";
    pub const ERROR: &'static str = "\x1b[38;5;196m";
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            primary: "\x1b[38;5;39m",     // Bright blue
            secondary: "\x1b[38;5;141m",  // Purple
            success: "\x1b[38;5;46m",     // Green
            warning: "\x1b[38;5;220m",    // Yellow
            error: "\x1b[38;5;196m",      // Red
            info: "\x1b[38;5;51m",        // Cyan
            muted: "\x1b[38;5;244m",      // Gray
            bright: "\x1b[1m",            // Bold
            dim: "\x1b[2m",               // Dim
            accent: "\x1b[38;5;208m",     // Orange
            highlight: "\x1b[48;5;234m",  // Dark gray background
            background: "\x1b[48;5;235m", // Darker background
            border: "\x1b[38;5;240m",     // Border gray
        }
    }
}

/// Reset color sequence
pub const RESET: &str = "\x1b[0m";

/// Table formatting options
#[derive(Debug, Clone)]
pub struct TableOptions {
    pub show_borders: bool,
    pub zebra_striping: bool,
    pub compact_mode: bool,
    pub max_width: Option<usize>,
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            show_borders: true,
            zebra_striping: false,
            compact_mode: false,
            max_width: None,
        }
    }
}

/// Icon set for file types and UI elements
#[derive(Debug, Clone)]
pub struct Icons {
    // File type icons
    pub directory: &'static str,
    pub file: &'static str,
    pub executable: &'static str,
    pub link: &'static str,
    pub archive: &'static str,
    pub image: &'static str,
    pub video: &'static str,
    pub audio: &'static str,
    pub document: &'static str,
    pub code: &'static str,
    
    // UI element icons
    pub arrow_right: &'static str,
    pub arrow_down: &'static str,
    pub bullet: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    pub error: &'static str,
    pub info: &'static str,
    pub spinner: [&'static str; 4],
}

impl Default for Icons {
    fn default() -> Self {
        Self {
            // File type icons (Unicode with ASCII fallback)
            directory: "📁",
            file: "📄",
            executable: "⚡",
            link: "🔗",
            archive: "📦",
            image: "🖼️",
            video: "🎬",
            audio: "🎵",
            document: "📋",
            code: "💻",
            
            // UI element icons
            arrow_right: "▶",
            arrow_down: "▼",
            bullet: "•",
            success: "✓",
            warning: "⚠",
            error: "✗",
            info: "ℹ",
            spinner: ["⠋", "⠙", "⠹", "⠸"],
        }
    }
}

impl Icons {
    pub fn new() -> Self {
        Self::default()
    }
    
    // Additional icon constants used by various commands
    pub const ENVIRONMENT: &'static str = "🌍";
    pub const STOPWATCH: &'static str = "⏱️";
    pub const CLOCK: &'static str = "🕐";
    pub const CPU: &'static str = "⚙️";
    pub const SYSTEM: &'static str = "🖥️";
}

/// ASCII fallback icons for compatibility
impl Icons {
    pub fn ascii() -> Self {
        Self {
            directory: "[DIR]",
            file: "[FILE]",
            executable: "[EXE]",
            link: "[LINK]",
            archive: "[ARC]",
            image: "[IMG]",
            video: "[VID]",
            audio: "[AUD]",
            document: "[DOC]",
            code: "[CODE]",
            
            arrow_right: ">",
            arrow_down: "v",
            bullet: "*",
            success: "+",
            warning: "!",
            error: "x",
            info: "i",
            spinner: ["|", "/", "-", "\\"],
        }
    }
}

/// Advanced table formatter for beautiful command output
#[derive(Debug)]
pub struct TableFormatter {
    colors: ColorPalette,
    pub icons: Icons,
    use_unicode: bool,
}

impl TableFormatter {
    pub fn new() -> Self {
        Self {
            colors: ColorPalette::default(),
            icons: Icons::default(),
            use_unicode: true,
        }
    }
    
    pub fn ascii_mode(mut self) -> Self {
        self.use_unicode = false;
        self.icons = Icons::ascii();
        self
    }
    
    /// Create a beautiful table with headers and rows
    pub fn create_table(&self, headers: &[&str], rows: &[Vec<String>]) -> String {
        if rows.is_empty() {
            return String::new();
        }
        
        // Calculate column widths
        let mut widths = vec![0; headers.len()];
        
        // Header widths
        for (i, header) in headers.iter().enumerate() {
            widths[i] = widths[i].max(self.display_width(header));
        }
        
        // Row widths
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(self.display_width(&self.strip_ansi(cell)));
                }
            }
        }
        
        let mut result = String::new();
        
        // Table top border
        result.push_str(&self.create_border(&widths, true, false, false));
        
        // Headers
        result.push_str(&self.create_row(headers.iter().map(|s| s.to_string()).collect(), &widths, true));
        
        // Header separator
        result.push_str(&self.create_border(&widths, false, true, false));
        
        // Data rows
        for (i, row) in rows.iter().enumerate() {
            result.push_str(&self.create_row(row.clone(), &widths, false));
            
            // Add separator between rows if needed
            if i < rows.len() - 1 && rows.len() > 10 {
                // Only add separators for large tables every 5 rows
                if (i + 1) % 5 == 0 {
                    result.push_str(&self.create_separator(&widths));
                }
            }
        }
        
        // Table bottom border
        result.push_str(&self.create_border(&widths, false, false, true));
        
        result
    }
    
    /// Print a table directly to stdout
    pub fn print_table(&self, rows: &[Vec<String>], headers: &[&str]) {
        let table = self.create_table(headers, rows);
        print!("{}", table);
    }
    
    /// Create an advanced table with multiple formatting options
    pub fn create_advanced_table(&self, headers: Vec<String>, rows: Vec<Vec<String>>, options: TableOptions) -> String {
        if headers.is_empty() || rows.is_empty() {
            return String::new();
        }
        
        // Calculate column widths
        let mut widths = vec![0; headers.len()];
        
        // Header widths
        for (i, header) in headers.iter().enumerate() {
            widths[i] = widths[i].max(self.display_width(header));
        }
        
        // Row widths
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(self.display_width(&self.strip_ansi(cell)));
                }
            }
        }
        
        let mut result = String::new();
        
        // Table top border
        result.push_str(&self.create_border(&widths, true, false, false));
        
        // Headers
        result.push_str(&self.create_row(headers.iter().map(|s| s.to_string()).collect(), &widths, true));
        
        // Header separator
        result.push_str(&self.create_border(&widths, false, true, false));
        
        // Data rows
        for (i, row) in rows.iter().enumerate() {
            result.push_str(&self.create_row(row.clone(), &widths, false));
            
            // Add separator between rows if needed
            if i < rows.len() - 1 && rows.len() > 10 {
                // Only add separators for large tables every 5 rows
                if (i + 1) % 5 == 0 {
                    result.push_str(&self.create_separator(&widths));
                }
            }
        }
        
        // Table bottom border
        result.push_str(&self.create_border(&widths, false, false, true));
        
        result
    }
    
    /// Create a row with proper padding and colors
    fn create_row(&self, cells: Vec<String>, widths: &[usize], is_header: bool) -> String {
        let mut row = String::new();
        
        // Left border
        row.push_str(&format!("{}│{}", self.colors.muted, RESET));
        
        for (i, cell) in cells.iter().enumerate() {
            if i < widths.len() {
                let content = if is_header {
                    format!("{}{}{}{}", self.colors.bright, self.colors.primary, cell, RESET)
                } else {
                    cell.clone()
                };
                
                let padding = widths[i].saturating_sub(self.display_width(&self.strip_ansi(cell)));
                row.push_str(&format!(" {}{} ", content, " ".repeat(padding)));
                
                // Column separator
                if i < widths.len() - 1 {
                    row.push_str(&format!("{}│{}", self.colors.muted, RESET));
                }
            }
        }
        
        // Right border
        row.push_str(&format!(" {}│{}\n", self.colors.muted, RESET));
        
        row
    }
    
    /// Create border lines
    fn create_border(&self, widths: &[usize], is_top: bool, is_middle: bool, is_bottom: bool) -> String {
        let mut border = String::new();
        
        // Corner characters
        let (left, right, junction) = if self.use_unicode {
            if is_top {
                ("┌", "┐", "┬")
            } else if is_middle {
                ("├", "┤", "┼")
            } else if is_bottom {
                ("└", "┘", "┴")
            } else {
                ("├", "┤", "┼")
            }
        } else {
            ("+", "+", "+")
        };
        
        let line_char = if self.use_unicode { "─" } else { "-" };
        
        border.push_str(&format!("{}{}{}", self.colors.muted, left, RESET));
        
        for (i, &width) in widths.iter().enumerate() {
            border.push_str(&line_char.repeat(width + 2));
            
            if i < widths.len() - 1 {
                border.push_str(&format!("{}{}{}", self.colors.muted, junction, RESET));
            }
        }
        
        border.push_str(&format!("{}{}{}\n", self.colors.muted, right, RESET));
        
        border
    }
    
    /// Create a simple separator
    fn create_separator(&self, widths: &[usize]) -> String {
        self.create_border(widths, false, true, false)
    }
    
    /// Create an advanced table with multiple formatting options
    pub fn create_advanced_table(&self, headers: Vec<String>, rows: Vec<Vec<String>>, options: TableOptions) -> String {
        if headers.is_empty() || rows.is_empty() {
            return String::new();
        }

        let num_cols = headers.len();
        let mut col_widths = vec![0; num_cols];
        
        // Calculate column widths
        for (i, header) in headers.iter().enumerate() {
            col_widths[i] = col_widths[i].max(self.display_width(header));
        }
        
        for row in &rows {
            for (i, cell) in row.iter().enumerate().take(num_cols) {
                col_widths[i] = col_widths[i].max(self.display_width(cell));
            }
        }
        
        let mut table = String::new();
        
        // Table header with styling
        if options.show_header {
            table.push_str(&self.create_border(&col_widths, BorderType::Top, &options));
            table.push('\n');
            
            // Header row
            table.push_str(&options.border_style.vertical);
            for (i, header) in headers.iter().enumerate() {
                let padded = self.pad_text(header, col_widths[i], options.alignment);
                table.push_str(&format!(" {} ", padded.bright()));
                table.push_str(&options.border_style.vertical);
            }
            table.push('\n');
            
            table.push_str(&self.create_border(&col_widths, BorderType::Middle, &options));
            table.push('\n');
        }
        
        // Data rows with alternating colors
        for (row_idx, row) in rows.iter().enumerate() {
            table.push_str(&options.border_style.vertical);
            for (i, cell) in row.iter().enumerate().take(num_cols) {
                let padded = self.pad_text(cell, col_widths[i], options.alignment);
                let colored_cell = if options.alternating_rows && row_idx % 2 == 1 {
                    padded.colorize(&current_theme().color_palette().highlight)
                } else {
                    padded
                };
                table.push_str(&format!(" {} ", colored_cell));
                table.push_str(&options.border_style.vertical);
            }
            table.push('\n');
        }
        
        // Bottom border
        table.push_str(&self.create_border(&col_widths, BorderType::Bottom, &options));
        
        table
    }
    
    fn create_border(&self, col_widths: &[usize], border_type: BorderType, options: &TableOptions) -> String {
        let mut border = String::new();
        
        let (start, middle, end, horizontal) = match border_type {
            BorderType::Top => (&options.border_style.top_left, &options.border_style.top_middle, &options.border_style.top_right, &options.border_style.horizontal),
            BorderType::Middle => (&options.border_style.middle_left, &options.border_style.cross, &options.border_style.middle_right, &options.border_style.horizontal),
            BorderType::Bottom => (&options.border_style.bottom_left, &options.border_style.bottom_middle, &options.border_style.bottom_right, &options.border_style.horizontal),
        };
        
        border.push_str(start);
        for (i, &width) in col_widths.iter().enumerate() {
            border.push_str(&horizontal.repeat(width + 2));
            if i < col_widths.len() - 1 {
                border.push_str(middle);
            }
        }
        border.push_str(end);
        
        border.colorize(&current_theme().color_palette().border)
    }
    
    fn pad_text(&self, text: &str, width: usize, alignment: TextAlignment) -> String {
        let text_width = self.display_width(text);
        if text_width >= width {
            return text.to_string();
        }
        
        let padding = width - text_width;
        match alignment {
            TextAlignment::Left => format!("{}{}", text, " ".repeat(padding)),
            TextAlignment::Right => format!("{}{}", " ".repeat(padding), text),
            TextAlignment::Center => {
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
            }
        }
    }
    
    /// Calculate display width ignoring ANSI escape codes
    pub fn display_width(&self, text: &str) -> usize {
        self.strip_ansi(text).chars().count()
    }
    
    /// Strip ANSI escape codes
    fn strip_ansi(&self, text: &str) -> String {
        // Simple ANSI stripping - in real implementation, use a proper library
        let mut result = String::new();
        let mut in_escape = false;
        
        for ch in text.chars() {
            if ch == '\x1b' {
                in_escape = true;
            } else if in_escape && ch == 'm' {
                in_escape = false;
            } else if !in_escape {
                result.push(ch);
            }
        }
        
        result
    }
    
    /// Format file size in human-readable format
    pub fn format_size(&self, size: u64) -> String {
        const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
        
        if size == 0 {
            return format!("{}0{} B", self.colors.muted, RESET);
        }
        
        let mut size = size as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        let color = match unit_index {
            0 => self.colors.muted,    // Bytes
            1 => self.colors.info,     // KB
            2 => self.colors.primary,  // MB
            3 => self.colors.warning,  // GB
            _ => self.colors.error,    // TB+
        };
        
        if unit_index == 0 {
            format!("{}{:.0}{} {}", color, size, RESET, UNITS[unit_index])
        } else {
            format!("{}{:.1}{} {}", color, size, RESET, UNITS[unit_index])
        }
    }
    
    /// Format permissions with colors
    pub fn format_permissions(&self, mode: u32) -> String {
        let mut result = String::new();
        
        // File type
        let file_type = match mode & 0o170000 {
            0o040000 => format!("{}d{}", self.colors.primary, RESET),
            0o120000 => format!("{}l{}", self.colors.info, RESET),
            0o010000 => format!("{}p{}", self.colors.warning, RESET),
            0o020000 => format!("{}c{}", self.colors.warning, RESET),
            0o060000 => format!("{}b{}", self.colors.warning, RESET),
            0o140000 => format!("{}s{}", self.colors.warning, RESET),
            _ => format!("{}-{}", self.colors.muted, RESET),
        };
        result.push_str(&file_type);
        
        // Owner permissions
        let owner = mode & 0o700;
        result.push_str(&self.format_permission_group(owner >> 6, true));
        
        // Group permissions
        let group = mode & 0o070;
        result.push_str(&self.format_permission_group(group >> 3, false));
        
        // Other permissions
        let other = mode & 0o007;
        result.push_str(&self.format_permission_group(other, false));
        
        result
    }
    
    fn format_permission_group(&self, perm: u32, is_owner: bool) -> String {
        let r = if perm & 4 != 0 { "r" } else { "-" };
        let w = if perm & 2 != 0 { "w" } else { "-" };
        let x = if perm & 1 != 0 { "x" } else { "-" };
        
        let color = if is_owner {
            self.colors.success
        } else if perm != 0 {
            self.colors.warning
        } else {
            self.colors.muted
        };
        
        format!("{}{}{}{}{}", color, r, w, x, RESET)
    }
    
    /// Get file type icon based on file extension or type
    pub fn get_file_icon(&self, path: &std::path::Path, is_dir: bool, is_executable: bool) -> &str {
        if is_dir {
            return self.icons.directory;
        }
        
        if is_executable {
            return self.icons.executable;
        }
        
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "svg" | "ico" => self.icons.image,
                "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" => self.icons.video,
                "mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" => self.icons.audio,
                "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => self.icons.archive,
                "txt" | "md" | "doc" | "docx" | "pdf" | "rtf" => self.icons.document,
                "rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "java" | "go" | "rb" => self.icons.code,
                _ => self.icons.file,
            }
        } else {
            self.icons.file
        }
    }
    
    /// Create a status line with icon and color
    pub fn create_status(&self, status: &str, message: &str, is_success: bool) -> String {
        let (icon, color) = if is_success {
            (self.icons.success, self.colors.success)
        } else {
            (self.icons.error, self.colors.error)
        };
        
        format!("{}{} {}{} {}\n", color, icon, status, RESET, message)
    }
    
    /// Create a progress bar
    pub fn create_progress(&self, current: usize, total: usize, width: usize) -> String {
        if total == 0 {
            return String::new();
        }
        
        let percentage = (current * 100) / total;
        let filled = (current * width) / total;
        let empty = width - filled;
        
        let bar = if self.use_unicode {
            format!("{}█{}{}{}", 
                self.colors.success, 
                "█".repeat(filled),
                "░".repeat(empty),
                RESET)
        } else {
            format!("{}#{}{}{}", 
                self.colors.success,
                "#".repeat(filled),
                "-".repeat(empty),
                RESET)
        };
        
        format!("[{}] {}%", bar, percentage)
    }
    
    /// Create a header with decorative border
    pub fn create_header(&self, title: &str) -> String {
        let width = 60;
        let title_len = title.len();
        let padding = (width - title_len - 2) / 2;
        
        let border = if self.use_unicode { "═" } else { "=" };
        let corner = if self.use_unicode { "╔╗╚╝" } else { "++++" };
        
        format!("{}{}{}{}{}\n{}{}{}{}{}{}{}{}\n{}{}{}{}{}\n",
            self.colors.primary,
            corner.chars().nth(0).unwrap(),
            border.repeat(width - 2),
            corner.chars().nth(1).unwrap(),
            RESET,
            self.colors.primary,
            corner.chars().nth(0).unwrap(),
            " ".repeat(padding),
            self.colors.bright,
            title,
            RESET,
            " ".repeat(width - title_len - padding - 2),
            corner.chars().nth(1).unwrap(),
            self.colors.primary,
            corner.chars().nth(2).unwrap(),
            border.repeat(width - 2),
            corner.chars().nth(3).unwrap(),
            RESET)
    }

impl Default for TableFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for colorizing text
pub trait Colorize {
    fn primary(&self) -> String;
    fn secondary(&self) -> String;
    fn success(&self) -> String;
    fn warning(&self) -> String;
    fn error(&self) -> String;
    fn info(&self) -> String;
    fn muted(&self) -> String;
    fn bright(&self) -> String;
    fn dim(&self) -> String;
}

impl Colorize for str {
    fn primary(&self) -> String {
        format!("{}{}{}", ColorPalette::default().primary, self, RESET)
    }
    
    fn secondary(&self) -> String {
        format!("{}{}{}", ColorPalette::default().secondary, self, RESET)
    }
    
    fn success(&self) -> String {
        format!("{}{}{}", ColorPalette::default().success, self, RESET)
    }
    
    fn warning(&self) -> String {
        format!("{}{}{}", ColorPalette::default().warning, self, RESET)
    }
    
    fn error(&self) -> String {
        format!("{}{}{}", ColorPalette::default().error, self, RESET)
    }
    
    fn info(&self) -> String {
        format!("{}{}{}", ColorPalette::default().info, self, RESET)
    }
    
    fn muted(&self) -> String {
        format!("{}{}{}", ColorPalette::default().muted, self, RESET)
    }
    
    fn bright(&self) -> String {
        format!("{}{}{}", ColorPalette::default().bright, self, RESET)
    }
    
    fn dim(&self) -> String {
        format!("{}{}{}", ColorPalette::default().dim, self, RESET)
    }
}

impl Colorize for String {
    fn primary(&self) -> String {
        self.as_str().primary()
    }
    
    fn secondary(&self) -> String {
        self.as_str().secondary()
    }
    
    fn success(&self) -> String {
        self.as_str().success()
    }
    
    fn warning(&self) -> String {
        self.as_str().warning()
    }
    
    fn error(&self) -> String {
        self.as_str().error()
    }
    
    fn info(&self) -> String {
        self.as_str().info()
    }
    
    fn muted(&self) -> String {
        self.as_str().muted()
    }
    
    fn bright(&self) -> String {
        self.as_str().bright()
    }
    
    fn dim(&self) -> String {
        self.as_str().dim()
    }
}

/// Advanced progress bar with customizable styling
#[derive(Debug, Clone)]
pub struct ProgressBar {
    pub total: u64,
    pub current: u64,
    pub width: usize,
    pub style: ProgressStyle,
    pub label: String,
    pub start_time: Instant,
}

#[derive(Debug, Clone)]
pub enum ProgressStyle {
    Classic,
    Modern,
    Minimal,
    Animated,
}

impl ProgressBar {
    pub fn new(total: u64, width: usize) -> Self {
        Self {
            total,
            current: 0,
            width,
            style: ProgressStyle::Modern,
            label: String::new(),
            start_time: Instant::now(),
        }
    }
    
    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }
    
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    
    pub fn update(&mut self, current: u64) {
        self.current = current;
        self.render();
    }
    
    pub fn increment(&mut self) {
        self.current = (self.current + 1).min(self.total);
        self.render();
    }
    
    pub fn finish(&mut self) {
        self.current = self.total;
        self.render();
        println!(); // New line after completion
    }
    
    fn render(&self) {
        let percentage = if self.total > 0 {
            (self.current as f64 / self.total as f64 * 100.0) as u64
        } else {
            0
        };
        
        let filled = (self.current as f64 / self.total as f64 * self.width as f64) as usize;
        let empty = self.width.saturating_sub(filled);
        
        let elapsed = self.start_time.elapsed();
        let eta = if self.current > 0 {
            let rate = self.current as f64 / elapsed.as_secs_f64();
            let remaining = (self.total - self.current) as f64 / rate;
            Duration::from_secs_f64(remaining)
        } else {
            Duration::from_secs(0)
        };
        
        let bar = match self.style {
            ProgressStyle::Classic => {
                format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
            },
            ProgressStyle::Modern => {
                format!("{}{}{}{}",
                    "█".repeat(filled).colorize(&ColorPalette::SUCCESS),
                    "░".repeat(empty).colorize(&ColorPalette::MUTED),
                    "",
                    ""
                )
            },
            ProgressStyle::Minimal => {
                format!("{}{}",
                    "▓".repeat(filled).colorize(&ColorPalette::PRIMARY),
                    "▒".repeat(empty).colorize(&ColorPalette::MUTED)
                )
            },
            ProgressStyle::Animated => {
                let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                let spinner_idx = (elapsed.as_millis() / 100) as usize % spinner_chars.len();
                format!("{} {}{}",
                    spinner_chars[spinner_idx].colorize(&ColorPalette::INFO),
                    "▓".repeat(filled).colorize(&ColorPalette::SUCCESS),
                    "▒".repeat(empty).colorize(&ColorPalette::MUTED)
                )
            }
        };
        
        print!("\r{} {} {}% ETA: {:02}:{:02} ",
            if !self.label.is_empty() { &self.label } else { "Progress" },
            bar,
            percentage,
            eta.as_secs() / 60,
            eta.as_secs() % 60
        );
        io::stdout().flush().ok();
    }
}

/// Animation utilities for enhanced UX
pub struct Animation;

impl Animation {
    /// Typewriter effect for text output
    pub fn typewriter(text: &str, delay_ms: u64) {
        for ch in text.chars() {
            print!("{}", ch);
            io::stdout().flush().ok();
            thread::sleep(Duration::from_millis(delay_ms));
        }
        println!();
    }
    
    /// Loading spinner animation
    pub fn spinner(duration_ms: u64, message: &str) {
        let chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let start = Instant::now();
        let mut i = 0;
        
        while start.elapsed().as_millis() < duration_ms as u128 {
            print!("\r{} {}", chars[i % chars.len()].colorize(&ColorPalette::INFO), message);
            io::stdout().flush().ok();
            thread::sleep(Duration::from_millis(100));
            i += 1;
        }
        print!("\r{} {}\n", Icons::CHECK.colorize(&ColorPalette::SUCCESS), message);
    }
    
    /// Pulsing text effect
    pub fn pulse_text(text: &str, cycles: u32) {
        for _ in 0..cycles {
            print!("\r{}", text.bright());
            io::stdout().flush().ok();
            thread::sleep(Duration::from_millis(300));
            print!("\r{}", text.dim());
            io::stdout().flush().ok();
            thread::sleep(Duration::from_millis(300));
        }
        println!("\r{}", text); // Reset to normal
    }
}

/// Theme management system
#[derive(Debug, Clone)]
pub enum Theme {
    Default,
    Dark,
    Light,
    Ocean,
    Forest,
    Sunset,
}

impl Theme {
    pub fn color_palette(&self) -> ColorPalette {
        match self {
            Theme::Default => ColorPalette::default(),
            Theme::Dark => ColorPalette {
                primary: "\x1b[38;5;75m",
                secondary: "\x1b[38;5;105m",
                success: "\x1b[38;5;76m",
                warning: "\x1b[38;5;178m",
                error: "\x1b[38;5;160m",
                info: "\x1b[38;5;87m",
                muted: "\x1b[38;5;59m",
                bright: "\x1b[1m",
                dim: "\x1b[2m",
                accent: "\x1b[38;5;215m",
                highlight: "\x1b[48;5;236m",
                background: "\x1b[48;5;233m",
                border: "\x1b[38;5;102m",
            },
            Theme::Light => ColorPalette {
                primary: "\x1b[38;5;25m",
                secondary: "\x1b[38;5;55m",
                success: "\x1b[38;5;22m",
                warning: "\x1b[38;5;94m",
                error: "\x1b[38;5;88m",
                info: "\x1b[38;5;23m",
                muted: "\x1b[38;5;102m",
                bright: "\x1b[1m",
                dim: "\x1b[2m",
                accent: "\x1b[38;5;130m",
                highlight: "\x1b[48;5;254m",
                background: "\x1b[48;5;255m",
                border: "\x1b[38;5;145m",
            },
            Theme::Ocean => ColorPalette {
                primary: "\x1b[38;5;33m",
                secondary: "\x1b[38;5;69m",
                success: "\x1b[38;5;36m",
                warning: "\x1b[38;5;172m",
                error: "\x1b[38;5;124m",
                info: "\x1b[38;5;45m",
                muted: "\x1b[38;5;67m",
                bright: "\x1b[1m",
                dim: "\x1b[2m",
                accent: "\x1b[38;5;74m",
                highlight: "\x1b[48;5;17m",
                background: "\x1b[48;5;16m",
                border: "\x1b[38;5;24m",
            },
            Theme::Forest => ColorPalette {
                primary: "\x1b[38;5;28m",
                secondary: "\x1b[38;5;58m",
                success: "\x1b[38;5;34m",
                warning: "\x1b[38;5;136m",
                error: "\x1b[38;5;88m",
                info: "\x1b[38;5;37m",
                muted: "\x1b[38;5;101m",
                bright: "\x1b[1m",
                dim: "\x1b[2m",
                accent: "\x1b[38;5;64m",
                highlight: "\x1b[48;5;22m",
                background: "\x1b[48;5;16m",
                border: "\x1b[38;5;58m",
            },
            Theme::Sunset => ColorPalette {
                primary: "\x1b[38;5;202m",
                secondary: "\x1b[38;5;198m",
                success: "\x1b[38;5;214m",
                warning: "\x1b[38;5;220m",
                error: "\x1b[38;5;196m",
                info: "\x1b[38;5;211m",
                muted: "\x1b[38;5;95m",
                bright: "\x1b[1m",
                dim: "\x1b[2m",
                accent: "\x1b[38;5;208m",
                highlight: "\x1b[48;5;52m",
                background: "\x1b[48;5;16m",
                border: "\x1b[38;5;130m",
            },
        }
    }
}

/// Global theme state
static mut CURRENT_THEME: Theme = Theme::Default;

pub fn set_theme(theme: Theme) {
    unsafe {
        CURRENT_THEME = theme;
    }
}

pub fn current_theme() -> &'static Theme {
    unsafe { &CURRENT_THEME }
}

/// Advanced table formatting options
#[derive(Debug, Clone)]
pub struct TableOptions {
    pub border_style: BorderStyle,
    pub show_header: bool,
    pub alternating_rows: bool,
    pub alignment: TextAlignment,
    pub max_width: Option<usize>,
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            border_style: BorderStyle::rounded(),
            show_header: true,
            alternating_rows: false,
            alignment: TextAlignment::Left,
            max_width: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BorderStyle {
    pub horizontal: String,
    pub vertical: String,
    pub top_left: String,
    pub top_right: String,
    pub bottom_left: String,
    pub bottom_right: String,
    pub top_middle: String,
    pub bottom_middle: String,
    pub middle_left: String,
    pub middle_right: String,
    pub cross: String,
}

impl BorderStyle {
    pub fn rounded() -> Self {
        Self {
            horizontal: "─".to_string(),
            vertical: "│".to_string(),
            top_left: "╭".to_string(),
            top_right: "╮".to_string(),
            bottom_left: "╰".to_string(),
            bottom_right: "╯".to_string(),
            top_middle: "┬".to_string(),
            bottom_middle: "┴".to_string(),
            middle_left: "├".to_string(),
            middle_right: "┤".to_string(),
            cross: "┼".to_string(),
        }
    }
    
    pub fn classic() -> Self {
        Self {
            horizontal: "-".to_string(),
            vertical: "|".to_string(),
            top_left: "+".to_string(),
            top_right: "+".to_string(),
            bottom_left: "+".to_string(),
            bottom_right: "+".to_string(),
            top_middle: "+".to_string(),
            bottom_middle: "+".to_string(),
            middle_left: "+".to_string(),
            middle_right: "+".to_string(),
            cross: "+".to_string(),
        }
    }
    
    pub fn double() -> Self {
        Self {
            horizontal: "═".to_string(),
            vertical: "║".to_string(),
            top_left: "╔".to_string(),
            top_right: "╗".to_string(),
            bottom_left: "╚".to_string(),
            bottom_right: "╝".to_string(),
            top_middle: "╦".to_string(),
            bottom_middle: "╩".to_string(),
            middle_left: "╠".to_string(),
            middle_right: "╣".to_string(),
            cross: "╬".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TextAlignment {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone)]
enum BorderType {
    Top,
    Middle,
    Bottom,
}

/// Interactive elements for enhanced user experience
pub struct Interactive;

impl Interactive {
    /// Create a confirmation prompt with styling
    pub fn confirm(message: &str, default: bool) -> bool {
        let default_text = if default { "Y/n" } else { "y/N" };
        print!("{} {} [{}]: ", 
            Icons::QUESTION.colorize(&ColorPalette::INFO),
            message,
            default_text.colorize(&ColorPalette::MUTED)
        );
        io::stdout().flush().ok();
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let input = input.trim().to_lowercase();
            match input.as_str() {
                "y" | "yes" => true,
                "n" | "no" => false,
                "" => default,
                _ => default,
            }
        } else {
            default
        }
    }
    
    /// Create a selection menu
    pub fn select(prompt: &str, options: &[&str]) -> Option<usize> {
        println!("{} {}", Icons::MENU.colorize(&ColorPalette::INFO), prompt);
        
        for (i, option) in options.iter().enumerate() {
            println!("  {}. {}", 
                (i + 1).to_string().colorize(&ColorPalette::ACCENT),
                option
            );
        }
        
        print!("Enter choice (1-{}): ", options.len());
        io::stdout().flush().ok();
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            if let Ok(choice) = input.trim().parse::<usize>() {
                if choice > 0 && choice <= options.len() {
                    return Some(choice - 1);
                }
            }
        }
        None
    }
}

/// Notification system for user feedback
pub struct Notification;

impl Notification {
    pub fn success(message: &str) {
        println!("{} {}", Icons::CHECK.colorize(&ColorPalette::SUCCESS), message);
    }
    
    pub fn warning(message: &str) {
        println!("{} {}", Icons::WARNING.colorize(&ColorPalette::WARNING), message);
    }
    
    pub fn error(message: &str) {
        println!("{} {}", Icons::ERROR.colorize(&ColorPalette::ERROR), message);
    }
    
    pub fn info(message: &str) {
        println!("{} {}", Icons::INFO.colorize(&ColorPalette::INFO), message);
    }
    
    pub fn banner(title: &str, subtitle: Option<&str>) {
        let width = 60;
        let border = "═".repeat(width);
        
        println!("{}", border.colorize(&ColorPalette::BORDER));
        println!("{}", format!("  {}", title).colorize(&ColorPalette::BRIGHT));
        
        if let Some(sub) = subtitle {
            println!("{}", format!("  {}", sub).colorize(&ColorPalette::MUTED));
        }
        
        println!("{}", border.colorize(&ColorPalette::BORDER));
    }
}

// ============================================================================
// Advanced Interactive Command Wizard System
// ============================================================================

#[derive(Debug, Clone)]
pub struct CommandWizard {
    pub title: String,
    pub steps: Vec<WizardStep>,
    pub current_step: usize,
}

#[derive(Debug, Clone)]
pub struct WizardStep {
    pub title: String,
    pub description: String,
    pub input_type: InputType,
    pub options: Vec<String>,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub enum InputType {
    Text,
    Number,
    Selection,
    MultiSelection,
    Boolean,
    FilePath,
    DirectoryPath,
}

impl CommandWizard {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            steps: Vec::new(),
            current_step: 0,
        }
    }
    
    pub fn add_step(&mut self, step: WizardStep) {
        self.steps.push(step);
    }
    
    pub fn run(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        
        println!("{}", format!("🧙‍♂️ {} Wizard", self.title).primary());
        println!("{}", "═".repeat(50).dim());
        
        for (index, step) in self.steps.iter().enumerate() {
            self.current_step = index;
            
            println!("\n{} {} {}/{}", 
                "Step".info(), 
                (index + 1).to_string().primary(),
                (index + 1).to_string().dim(),
                self.steps.len().to_string().dim()
            );
            
            println!("{}", step.title.primary());
            if !step.description.is_empty() {
                println!("   {}", step.description.dim());
            }
            
            let result = self.handle_input(step)?;
            results.push(result);
            
            // Show progress
            let progress = ((index + 1) as f32 / self.steps.len() as f32 * 100.0) as usize;
            let bar_width = 30;
            let filled = (progress * bar_width / 100).min(bar_width);
            let empty = bar_width - filled;
            println!("   Progress: [{}{}] {}%", 
                "█".repeat(filled).success(),
                "░".repeat(empty).dim(),
                progress.to_string().info()
            );
        }
        
        println!("\n{}", "✅ Wizard completed successfully!".success());
        Ok(results)
    }
    
    fn handle_input(&self, step: &WizardStep) -> Result<String, Box<dyn std::error::Error>> {
        use std::io;
        use std::io::Write;
        
        match step.input_type {
            InputType::Selection => {
                println!("\n{}", "Available options:".info());
                for (i, option) in step.options.iter().enumerate() {
                    println!("   {}. {}", (i + 1).to_string().primary(), option);
                }
                print!("Enter your choice (1-{}): ", step.options.len());
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let choice: usize = input.trim().parse().unwrap_or(0);
                
                if choice > 0 && choice <= step.options.len() {
                    Ok(step.options[choice - 1].clone())
                } else {
                    Err("Invalid selection".into())
                }
            },
            InputType::Boolean => {
                print!("   {} (y/n): ", step.title);
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let response = input.trim().to_lowercase();
                
                Ok(if response == "y" || response == "yes" { "true" } else { "false" }.to_string())
            },
            _ => {
                print!("   Enter value: ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                Ok(input.trim().to_string())
            }
        }
    }
}

// ============================================================================
// Advanced Status Dashboard System  
// ============================================================================

#[derive(Debug, Clone)]
pub struct StatusDashboard {
    pub title: String,
    pub sections: Vec<DashboardSection>,
    pub auto_refresh: bool,
}

#[derive(Debug, Clone)]
pub struct DashboardSection {
    pub title: String,
    pub items: Vec<StatusItem>,
    pub style: SectionStyle,
}

#[derive(Debug, Clone)]
pub struct StatusItem {
    pub label: String,
    pub value: String,
    pub status: ItemStatus,
    pub icon: String,
}

#[derive(Debug, Clone)]
pub enum SectionStyle {
    Simple,
    Boxed,
    Highlighted,
    Compact,
}

#[derive(Debug, Clone)]
pub enum ItemStatus {
    Good,
    Warning,
    Error,
    Info,
    Unknown,
}

impl StatusDashboard {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            sections: Vec::new(),
            auto_refresh: false,
        }
    }
    
    pub fn add_section(&mut self, section: DashboardSection) {
        self.sections.push(section);
    }
    
    pub fn render(&self) -> String {
        let mut output = String::new();
        
        // Header
        output.push_str(&format!("📊 {}\n", self.title.primary()));
        output.push_str(&format!("{}\n", "═".repeat(60).dim()));
        
        // Sections
        for section in &self.sections {
            output.push_str(&self.render_section(section));
            output.push('\n');
        }
        
        output
    }
    
    fn render_section(&self, section: &DashboardSection) -> String {
        let mut output = String::new();
        
        match section.style {
            SectionStyle::Boxed => {
                let title_len = section.title.len();
                let remaining = if title_len < 50 { 50 - title_len } else { 5 };
                output.push_str(&format!("┌─ {} ─{}\n", 
                    section.title.primary(),
                    "─".repeat(remaining).dim()));
                
                for item in &section.items {
                    output.push_str(&format!("│ {} {} {} {}\n",
                        item.icon,
                        item.label.info(),
                        "│".dim(),
                        self.format_value(&item.value, &item.status)));
                }
                
                output.push_str(&format!("└{}\n", "─".repeat(58).dim()));
            },
            SectionStyle::Highlighted => {
                output.push_str(&format!("▶ {}\n", section.title.primary()));
                for item in &section.items {
                    output.push_str(&format!("  {} {} {}\n",
                        item.icon,
                        item.label.info(),
                        self.format_value(&item.value, &item.status)));
                }
            },
            _ => {
                output.push_str(&format!("{}\n", section.title.primary()));
                for item in &section.items {
                    output.push_str(&format!("  {} {} {}\n",
                        item.icon,
                        item.label,
                        self.format_value(&item.value, &item.status)));
                }
            }
        }
        
        output
    }
    
    fn format_value(&self, value: &str, status: &ItemStatus) -> String {
        match status {
            ItemStatus::Good => value.success(),
            ItemStatus::Warning => value.warning(),
            ItemStatus::Error => value.error(),
            ItemStatus::Info => value.info(),
            ItemStatus::Unknown => value.dim(),
        }
    }
}

// ============================================================================
// Enhanced File Preview System
// ============================================================================

#[derive(Debug, Clone)]
pub struct FilePreview {
    pub file_path: String,
    pub preview_type: PreviewType,
    pub max_lines: usize,
    pub show_line_numbers: bool,
    pub syntax_highlighting: bool,
}

#[derive(Debug, Clone)]
pub enum PreviewType {
    Text,
    Binary,
    Image,
    Archive,
    Unknown,
}

impl FilePreview {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            preview_type: PreviewType::Unknown,
            max_lines: 20,
            show_line_numbers: true,
            syntax_highlighting: false,
        }
    }
    
    pub fn render(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut output = String::new();
        
        // Header
        output.push_str(&format!("📄 File Preview: {}\n", self.file_path.info()));
        output.push_str(&format!("{}\n", "─".repeat(60).dim()));
        
        // File info
        if let Ok(metadata) = std::fs::metadata(&self.file_path) {
            output.push_str(&format!("📏 Size: {}\n", 
                bytesize::ByteSize::b(metadata.len()).to_string().info()));
        }
        
        output.push('\n');
        
        // Content preview
        match std::fs::read_to_string(&self.file_path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let preview_lines = std::cmp::min(lines.len(), self.max_lines);
                
                for (i, line) in lines.iter().take(preview_lines).enumerate() {
                    if self.show_line_numbers {
                        output.push_str(&format!("{:3} │ {}\n", 
                            (i + 1).to_string().dim(), 
                            line));
                    } else {
                        output.push_str(&format!("{}\n", line));
                    }
                }
                
                if lines.len() > self.max_lines {
                    output.push_str(&format!("... {} more lines\n", 
                        (lines.len() - self.max_lines).to_string().dim()));
                }
            },
            Err(_) => {
                output.push_str(&"📋 Binary file or access denied\n".warning());
            }
        }
        
        Ok(output)
    }
}
