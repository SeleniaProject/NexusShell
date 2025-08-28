pub mod logging;
#[cfg(feature = "i18n")]
pub mod i18n; // full implementation
#[cfg(not(feature = "i18n"))]
pub mod i18n; // stub (same file exports stub when feature off)
#[cfg(feature = "async-runtime")]
pub mod metrics;
#[cfg(not(feature = "async-runtime"))]
pub mod metrics; // stub when async runtime disabled
pub mod crash_diagnosis; 
#[cfg(feature = "async-runtime")]
pub mod update_system; 
#[cfg(not(feature = "async-runtime"))]
pub mod update_system; // stub
pub mod sed_utils;
pub mod process_utils; 
pub mod resource_monitor;
pub mod locale_format;

use std::collections::HashMap;
use std::env;
use std::io;
use std::path::PathBuf;

/// Result type for built-in commands
pub type BuiltinResult<T> = Result<T, BuiltinError>;

/// Error type for built-in command execution
#[derive(Debug, thiserror::Error)]
pub enum BuiltinError {
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Missing required argument: {0}")]
    MissingArgument(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Directory not found: {0}")]
    DirectoryNotFound(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("Environment error: {0}")]
    EnvironmentError(String),
    
    #[error("Command failed with exit code: {0}")]
    CommandFailed(i32),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Other error: {0}")]
    Other(String),
}

impl From<Box<dyn std::error::Error>> for BuiltinError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        BuiltinError::Other(error.to_string())
    }
}

impl From<&str> for BuiltinError {
    fn from(error: &str) -> Self {
        BuiltinError::Other(error.to_string())
    }
}

/// Context for built-in command execution
#[derive(Debug, Clone)]
pub struct BuiltinContext {
    /// Current working directory
    pub current_dir: PathBuf,
    
    /// Environment variables
    pub environment: HashMap<String, String>,
    
    /// Whether to use colored output
    pub use_colors: bool,
    
    /// Whether to show verbose output
    pub verbose: bool,
    
    /// Whether to show debug information
    pub debug: bool,
    
    /// Shell options
    pub shell_options: HashMap<String, bool>,
}

impl Default for BuiltinContext {
    fn default() -> Self {
        Self {
            current_dir: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            environment: env::vars().collect(),
            use_colors: true,
            verbose: false,
            debug: false,
            shell_options: HashMap::new(),
        }
    }
}

impl BuiltinContext {
    /// Create a new context with current environment
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get an environment variable
    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.environment.get(key)
    }
    
    /// Set an environment variable
    pub fn set_env(&mut self, key: String, value: String) {
        self.environment.insert(key, value);
    }
    
    /// Get a shell option
    pub fn get_option(&self, key: &str) -> bool {
        self.shell_options.get(key).copied().unwrap_or(false)
    }
    
    /// Set a shell option
    pub fn set_option(&mut self, key: String, value: bool) {
        self.shell_options.insert(key, value);
    }
}

/// Table formatter for structured output
#[derive(Debug, Clone)]
pub struct TableFormatter {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    max_widths: Vec<usize>,
    pub icons: Icons,
}

#[derive(Debug, Clone)]
pub struct Icons {
    pub error: &'static str,
    pub success: &'static str,
    pub bullet: &'static str,
    pub code: &'static str,
    pub info: &'static str,
    pub document: &'static str,
}

impl Default for Icons {
    fn default() -> Self {
        Self::new()
    }
}

impl Icons {
    pub fn new() -> Self {
        Self {
            error: "❌",
            success: "✅",
            bullet: "•",
            code: "💻",
            info: "ℹ️",
            document: "📄",
        }
    }
}

impl Default for TableFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl TableFormatter {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            max_widths: Vec::new(),
            icons: Icons::new(),
        }
    }

    pub fn with_headers(headers: Vec<String>) -> Self {
        let max_widths = headers.iter().map(|h| h.len()).collect();
        Self {
            headers,
            rows: Vec::new(),
            max_widths,
            icons: Icons::new(),
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        // Update max widths
        for (i, cell) in row.iter().enumerate() {
            if i >= self.max_widths.len() {
                self.max_widths.push(cell.len());
            } else {
                self.max_widths[i] = self.max_widths[i].max(cell.len());
            }
        }
        self.rows.push(row);
    }

    pub fn create_table(&self, headers: &[String], rows: &[Vec<String>]) -> String {
        let mut output = String::new();
        
        // Print headers
        for (i, header) in headers.iter().enumerate() {
            if i > 0 {
                output.push_str("  ");
            }
            output.push_str(&format!("{header:<12}"));
        }
        output.push('\n');
        
        // Print separator
        for (i, _) in headers.iter().enumerate() {
            if i > 0 {
                output.push_str("  ");
            }
            output.push_str(&"-".repeat(12));
        }
        output.push('\n');
        
        // Print rows
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    output.push_str("  ");
                }
                output.push_str(&format!("{cell:<12}"));
            }
            output.push('\n');
        }
        
        output
    }

    /// Format file permissions string
    pub fn format_permissions(&self, permissions: &std::fs::Permissions) -> String {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = permissions.mode();
            let user = if mode & 0o400 != 0 { "r" } else { "-" }.to_string() +
                      if mode & 0o200 != 0 { "w" } else { "-" } +
                      if mode & 0o100 != 0 { "x" } else { "-" };
            let group = if mode & 0o040 != 0 { "r" } else { "-" }.to_string() +
                       if mode & 0o020 != 0 { "w" } else { "-" } +
                       if mode & 0o010 != 0 { "x" } else { "-" };
            let other = if mode & 0o004 != 0 { "r" } else { "-" }.to_string() +
                       if mode & 0o002 != 0 { "w" } else { "-" } +
                       if mode & 0o001 != 0 { "x" } else { "-" };
            format!("{}{}{}", user, group, other)
        }
        #[cfg(windows)]
        {
            if permissions.readonly() {
                "r--r--r--".to_string()
            } else {
                "rw-rw-rw-".to_string()
            }
        }
    }

    /// Get file icon based on file type
    pub fn get_file_icon(&self, path: &std::path::Path) -> String {
        if path.is_dir() {
            "📁".to_string()
        } else if let Some(ext) = path.extension() {
            match ext.to_str().unwrap_or("") {
                "rs" => "🦀".to_string(),
                "py" => "🐍".to_string(),
                "js" | "ts" => "📜".to_string(),
                "md" => "📝".to_string(),
                "txt" => "📄".to_string(),
                "json" => "📋".to_string(),
                "toml" | "yaml" | "yml" => "⚙️".to_string(),
                _ => "📄".to_string(),
            }
        } else {
            "📄".to_string()
        }
    }

    /// Calculate display width of string
    pub fn display_width(&self, text: &str) -> usize {
        // Simple width calculation - can be enhanced with unicode width
        text.chars().count()
    }

    /// Create table with title
    pub fn with_title(&mut self, title: &str) -> &mut Self {
        // Store title in a simple way - can be enhanced
        self.headers.insert(0, title.to_string());
        self
    }

    /// Create advanced table with styling
    pub fn create_advanced_table(&self, data: &[Vec<String>]) -> String {
        if data.is_empty() {
            return String::new();
        }
        
        let headers = &data[0];
        let rows = &data[1..];
        
        self.create_table(headers, rows)
    }

    /// Format file size
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

    pub fn format(&self) -> String {
        let mut output = String::new();

        // Print headers if they exist
        if !self.headers.is_empty() {
            for (i, header) in self.headers.iter().enumerate() {
                let header_len = header.len();
                let width = self.max_widths.get(i).unwrap_or(&header_len);
                if i > 0 {
                    output.push_str("  ");
                }
                output.push_str(&format!("{header:<width$}"));
            }
            output.push('\n');

            // Print separator
            for (i, _) in self.headers.iter().enumerate() {
                let width = self.max_widths.get(i).unwrap_or(&0);
                if i > 0 {
                    output.push_str("  ");
                }
                output.push_str(&"-".repeat(*width));
            }
            output.push('\n');
        }

        // Print rows
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                let cell_len = cell.len();
                let width = self.max_widths.get(i).unwrap_or(&cell_len);
                if i > 0 {
                    output.push_str("  ");
                }
                output.push_str(&format!("{cell:<width$}"));
            }
            output.push('\n');
        }

        output
    }
}
