/// Advanced UI Design System for NexusShell
/// 
/// This module provides a comprehensive, beautiful UI design system for all shell commands.
/// Features modern terminal UI with colors, icons, tables, sophisticated formatting,
/// animations, progress indicators, and dynamic theming capabilities.

use std::fmt;
use std::time::{Duration, Instant};
use std::thread;
use std::io::{self, Write};

/// Advanced string colorization trait  
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
    fn bright_cyan(self) -> String;
    fn bright_yellow(self) -> String;
    fn bright_green(self) -> String;
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
    fn bright_cyan(self) -> String { format!("\x1b[96m{}\x1b[0m", self) }
    fn bright_yellow(self) -> String { format!("\x1b[93m{}\x1b[0m", self) }
    fn bright_green(self) -> String { format!("\x1b[92m{}\x1b[0m", self) }
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
    fn bright_cyan(self) -> String { format!("\x1b[96m{}\x1b[0m", self) }
    fn bright_yellow(self) -> String { format!("\x1b[93m{}\x1b[0m", self) }
    fn bright_green(self) -> String { format!("\x1b[92m{}\x1b[0m", self) }
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
    pub reset: &'static str,
}

impl ColorPalette {
    pub fn new() -> Self {
        Self::default()
    }
    
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
            reset: "\x1b[0m",             // Reset
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
    pub show_header: bool,
    pub alternating_rows: bool,
    pub align_columns: bool,
    pub compact: bool,
    pub border_style: BorderStyle,
    pub header_alignment: Alignment,
}

#[derive(Debug, Clone)]
pub enum BorderStyle {
    None,
    Simple,
    Heavy,
    Double,
    Rounded,
}

#[derive(Debug, Clone)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

/// Basic types for status management
#[derive(Debug, Clone)]
pub enum ItemStatus {
    Good,
    Warning,
    Critical,
    Unknown,
    Info,
    Error,
}

#[derive(Debug, Clone)]
pub enum SectionStyle {
    Default,
    Compact,
    Detailed,
    Minimal,
    Boxed,
    Highlighted,
    Simple,
}

#[derive(Debug, Clone)]
pub enum InputType {
    Text,
    Number,
    Boolean,
    Select,
    MultiSelect,
}

/// Basic command wizard types  
#[derive(Debug, Clone)]
pub struct CommandWizard {
    pub title: String,
    pub steps: Vec<WizardStep>,
}

#[derive(Debug, Clone)]
pub struct WizardStep {
    pub name: String,
    pub description: String,
    pub input_type: InputType,
    pub title: String,
    pub options: Vec<String>,
    pub required: bool,
}

/// Status dashboard types
#[derive(Debug, Clone)]
pub struct StatusDashboard {
    pub title: String,
    pub sections: Vec<DashboardSection>,
}

#[derive(Debug, Clone)]
pub struct DashboardSection {
    pub title: String,
    pub items: Vec<StatusItem>,
    pub style: SectionStyle,
}

#[derive(Debug, Clone)]
pub struct StatusItem {
    pub name: String,
    pub value: String,
    pub status: ItemStatus,
    pub label: String,
    pub icon: String,
}

/// File preview functionality
#[derive(Debug, Clone)]
pub struct FilePreview {
    pub path: String,
    pub content: String,
    pub line_count: usize,
}

// Placeholder implementations
impl CommandWizard {
    pub fn new(title: String) -> Self {
        Self { title, steps: Vec::new() }
    }
    
    pub fn add_step(&mut self, step: WizardStep) {
        self.steps.push(step);
    }
    
    pub fn run(&self) -> Result<Vec<String>, String> {
        // Simple implementation for now
        Ok(vec!["sample".to_string()])
    }
}

impl StatusDashboard {
    pub fn new(title: String) -> Self {
        Self { title, sections: Vec::new() }
    }
    
    pub fn add_section(&mut self, section: DashboardSection) {
        self.sections.push(section);
    }
    
    pub fn render(&self) -> String {
        let mut output = format!("=== {} ===\n", self.title);
        for section in &self.sections {
            output.push_str(&format!("\n[{}]\n", section.title));
            for item in &section.items {
                output.push_str(&format!("  {}: {}\n", item.name, item.value));
            }
        }
        output
    }
}

impl FilePreview {
    pub fn new(path: String) -> Self {
        Self { path, content: String::new(), line_count: 0 }
    }
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            show_borders: true,
            zebra_striping: false,
            compact_mode: false,
            max_width: None,
            show_header: true,
            alternating_rows: false,
            align_columns: true,
            compact: false,
            border_style: BorderStyle::Simple,
            header_alignment: Alignment::Left,
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
    pub folder: &'static str,
    pub symlink: &'static str,
    pub terminal: &'static str,
    pub log_file: &'static str,
    pub text_file: &'static str,
    pub loading: &'static str,
    
    // UI element icons
    pub arrow_right: &'static str,
    pub arrow_down: &'static str,
    pub bullet: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    pub error: &'static str,
    pub info: &'static str,
    pub spinner: [&'static str; 4],
    
    // Additional icons used by commands
    pub user: &'static str,
    pub system: &'static str,
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
    pub const GLOBE: &'static str = "🌐";
    pub const ERROR: &'static str = "❌";
    pub const FOLDER: &'static str = "📁";
    pub const FILE_ICON: &'static str = "📄";
    pub const NETWORK: &'static str = "🌐";
    pub const TREE: &'static str = "🌳";
    pub const CHECKMARK: &'static str = "✅";
    pub const WARNING_ICON: &'static str = "⚠️";
    pub const MOVE: &'static str = "🔄";
    pub const TRASH: &'static str = "🗑️";
    pub const FOLDER_PLUS: &'static str = "📁➕";
    pub const FOLDER_MINUS: &'static str = "📁➖";
    pub const LINK: &'static str = "🔗";
    pub const HARD_LINK: &'static str = "🔗";
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
            folder: "📁",
            symlink: "🔗",
            terminal: "💻",
            log_file: "📄",
            text_file: "📋",
            loading: "⏳",
            
            // UI element icons
            arrow_right: "▶",
            arrow_down: "▼",
            bullet: "•",
            success: "✓",
            warning: "⚠",
            error: "✗",
            info: "ℹ",
            spinner: ["⠋", "⠙", "⠹", "⠸"],
            
            // Additional icons
            user: "👤",
            system: "🖥️",
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
            success: "[OK]",
            warning: "[!]",
            error: "[X]",
            info: "[i]",
            spinner: ["|", "/", "-", "\\"],
            user: "[USER]",
            system: "[SYS]",
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
        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(self.display_width(&self.strip_ansi(cell)));
                }
            }
        }
        
        let mut result = String::new();
        
        if options.show_borders {
            // Table top border
            result.push_str(&self.create_border(&widths, true, false, false));
        }
        
        // Headers
        result.push_str(&self.create_row(headers, &widths, true));
        
        if options.show_borders {
            // Header separator
            result.push_str(&self.create_border(&widths, false, true, false));
        }
        
        // Data rows
        for (i, row) in rows.iter().enumerate() {
            let is_zebra = options.zebra_striping && i % 2 == 1;
            result.push_str(&self.create_row_with_style(row.clone(), &widths, false, is_zebra));
        }
        
        if options.show_borders {
            // Table bottom border
            result.push_str(&self.create_border(&widths, false, false, true));
        }
        
        result
    }
    
    /// Create a row with proper padding and colors
    fn create_row(&self, cells: Vec<String>, widths: &[usize], is_header: bool) -> String {
        self.create_row_with_style(cells, widths, is_header, false)
    }
    
    /// Create a row with additional styling options
    fn create_row_with_style(&self, cells: Vec<String>, widths: &[usize], is_header: bool, zebra: bool) -> String {
        let mut row = String::new();
        
        // Left border
        row.push_str(&format!("{}│{}", self.colors.muted, RESET));
        
        for (i, cell) in cells.iter().enumerate() {
            if i < widths.len() {
                let content = if is_header {
                    format!("{}{}{}{}", self.colors.bright, self.colors.primary, cell, RESET)
                } else if zebra {
                    format!("{}{}{}", self.colors.highlight, cell, RESET)
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
        
        // Left corner
        border.push_str(&format!("{}{}{}", self.colors.muted, left, RESET));
        
        for (i, &width) in widths.iter().enumerate() {
            // Horizontal line for this column
            border.push_str(&format!("{}{}{}", 
                self.colors.muted, 
                line_char.repeat(width + 2), 
                RESET));
            
            // Junction (if not last column)
            if i < widths.len() - 1 {
                border.push_str(&format!("{}{}{}", self.colors.muted, junction, RESET));
            }
        }
        
        // Right corner
        border.push_str(&format!("{}{}{}\n", self.colors.muted, right, RESET));
        
        border
    }
    
    /// Create a simple separator
    fn create_separator(&self, widths: &[usize]) -> String {
        self.create_border(widths, false, true, false)
    }
    
    /// Format Unix permissions into a readable string
    pub fn format_permissions(&self, mode: u32) -> String {
        let mut perms = String::new();
        
        // File type
        if mode & 0o040000 != 0 { perms.push('d'); }
        else if mode & 0o120000 != 0 { perms.push('l'); }
        else { perms.push('-'); }
        
        // Owner permissions
        perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o100 != 0 { 'x' } else { '-' });
        
        // Group permissions  
        perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o010 != 0 { 'x' } else { '-' });
        
        // Other permissions
        perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o001 != 0 { 'x' } else { '-' });
        
        perms.dim()
    }
    
    /// Calculate display width (excluding ANSI escape sequences)
    pub fn display_width(&self, text: &str) -> usize {
        self.strip_ansi(text).chars().count()
    }
    
    /// Strip ANSI escape sequences from text
    fn strip_ansi(&self, text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars();
        
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Skip escape sequence
                if chars.next() == Some('[') {
                    while let Some(c) = chars.next() {
                        if c.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }
    
    /// Get file icon based on file type
    pub fn get_file_icon(&self, path: &std::path::Path, is_dir: bool, is_executable: bool) -> &str {
        if is_dir {
            return self.icons.directory;
        }
        
        if is_executable {
            return self.icons.executable;
        }
        
        // Determine by extension
        if let Some(ext) = path.extension() {
            match ext.to_string_lossy().to_lowercase().as_str() {
                "zip" | "tar" | "gz" | "7z" | "rar" => self.icons.archive,
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" => self.icons.image,
                "mp4" | "avi" | "mkv" | "mov" | "wmv" => self.icons.video,
                "mp3" | "wav" | "flac" | "ogg" | "m4a" => self.icons.audio,
                "pdf" | "doc" | "docx" | "txt" | "md" => self.icons.document,
                "rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "java" => self.icons.code,
                _ => self.icons.file,
            }
        } else {
            self.icons.file
        }
    }
    
    /// Create header row with proper formatting
    pub fn create_header(&self, headers: &[&str]) -> String {
        let header_strings: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
        let widths: Vec<_> = header_strings.iter()
            .map(|h| self.display_width(h))
            .collect();
        self.create_row(header_strings, &widths, true)
    }
    
    /// Format file size with human-readable units
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

/// Create advanced table with multiple formatting options
pub fn create_advanced_table(headers: &[&str], rows: &[Vec<String>], options: TableOptions) -> String {
    let formatter = TableFormatter::new();
    let header_strings: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
    formatter.create_advanced_table(header_strings, rows.to_vec(), options)
}

impl Default for TableFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress bar with beautiful animations
#[derive(Debug)]
pub struct ProgressBar {
    width: usize,
    current: usize,
    total: usize,
    colors: ColorPalette,
    start_time: Instant,
}

impl ProgressBar {
    pub fn new(total: usize) -> Self {
        Self {
            width: 50,
            current: 0,
            total,
            colors: ColorPalette::default(),
            start_time: Instant::now(),
        }
    }
    
    pub fn update(&mut self, current: usize) {
        self.current = current;
        self.draw();
    }
    
    pub fn finish(&self) {
        println!();
    }
    
    fn draw(&self) {
        let percentage = if self.total > 0 {
            (self.current as f64 / self.total as f64 * 100.0) as usize
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
        
        print!("\r{} {}{}{}{}{}% ({}/{}) ETA: {:02}:{:02}",
            self.colors.info,
            "█".repeat(filled),
            self.colors.muted,
            "░".repeat(empty),
            self.colors.success,
            percentage,
            self.current,
            self.total,
            eta.as_secs() / 60,
            eta.as_secs() % 60);
        
        io::stdout().flush().unwrap();
    }
}

/// Animation helper for loading indicators
#[derive(Debug)]
pub struct Animation {
    frames: Vec<&'static str>,
    current_frame: usize,
    colors: ColorPalette,
}

impl Animation {
    pub fn spinner() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current_frame: 0,
            colors: ColorPalette::default(),
        }
    }
    
    pub fn dots() -> Self {
        Self {
            frames: vec![".", "..", "...", ""],
            current_frame: 0,
            colors: ColorPalette::default(),
        }
    }
    
    pub fn next_frame(&mut self) -> String {
        let frame = self.frames[self.current_frame];
        self.current_frame = (self.current_frame + 1) % self.frames.len();
        format!("{}{}{}", self.colors.primary, frame, self.colors.reset)
    }
}

/// Notification system for beautiful messages
#[derive(Debug, Clone)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug)]
pub struct Notification;

impl Notification {
    pub fn info(message: &str) {
        let colors = ColorPalette::default();
        let icons = Icons::default();
        println!("{} {} {}", 
            format!("{}{}{}", colors.info, icons.info, colors.reset),
            message,
            colors.reset);
    }
    
    pub fn success(message: &str) {
        let colors = ColorPalette::default();
        let icons = Icons::default();
        println!("{} {} {}", 
            format!("{}{}{}", colors.success, icons.success, colors.reset),
            message,
            colors.reset);
    }
    
    pub fn warning(message: &str) {
        let colors = ColorPalette::default();
        let icons = Icons::default();
        println!("{} {} {}", 
            format!("{}{}{}", colors.warning, icons.warning, colors.reset),
            message,
            colors.reset);
    }
    
    pub fn error(message: &str) {
        let colors = ColorPalette::default();
        let icons = Icons::default();
        eprintln!("{} {} {}", 
            format!("{}{}{}", colors.error, icons.error, colors.reset),
            message,
            colors.reset);
    }
}
