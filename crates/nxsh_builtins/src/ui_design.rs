/// Advanced CUI Design System for NexusShell
/// 
/// This module provides a comprehensive, beautiful UI design system for all shell commands.
/// Features modern terminal UI with colors, icons, tables, and sophisticated formatting.

use std::fmt;

/// Color palette for consistent theming
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
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            primary: "\x1b[38;5;39m",    // Bright blue
            secondary: "\x1b[38;5;141m", // Purple
            success: "\x1b[38;5;46m",    // Green
            warning: "\x1b[38;5;220m",   // Yellow
            error: "\x1b[38;5;196m",     // Red
            info: "\x1b[38;5;51m",       // Cyan
            muted: "\x1b[38;5;244m",     // Gray
            bright: "\x1b[1m",           // Bold
            dim: "\x1b[2m",              // Dim
        }
    }
}

/// Reset color sequence
pub const RESET: &str = "\x1b[0m";

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
