/// Universal Command Output Formatter for NexusShell
/// 
/// This module provides standardized, beautiful output formatting for all NexusShell commands.
/// It ensures consistent visual presentation across all command outputs while maintaining
/// readability and professional appearance.
/// 
/// Features:
/// - Automatic output type detection (table, list, text, json, etc.)
/// - Smart formatting based on content type
/// - Error and status message styling
/// - Progress indication for long operations
/// - Responsive layout adaptation
/// - Theme consistency across all outputs

use anyhow::{Result, Context};
use crate::advanced_cui::AdvancedCUI;

use serde_json::Value;

/// Universal output formatter for all commands
#[derive(Debug)]
pub struct UniversalFormatter {
    /// CUI design system instance
    cui: AdvancedCUI,
    
    /// Current command context
    command_context: CommandContext,
}

/// Context information for command output formatting
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Name of the command being executed
    pub command_name: String,
    
    /// Command arguments
    pub args: Vec<String>,
    
    /// Whether verbose output is requested
    pub verbose: bool,
    
    /// Output format preference (auto, table, json, yaml, etc.)
    pub format: OutputFormat,
    
    /// Whether to show colors
    pub color: bool,
}

/// Output format options
#[derive(Debug, Clone)]
pub enum OutputFormat {
    /// Automatic format detection
    Auto,
    
    /// Tabular format
    Table,
    
    /// JSON format
    Json,
    
    /// YAML format (if available)
    Yaml,
    
    /// Plain text
    Text,
    
    /// Compact format
    Compact,
    
    /// Detailed format
    Detailed,
}

/// Standard output types
#[derive(Debug)]
pub enum CommandOutput {
    /// Tabular data with headers and rows
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        metadata: Option<TableMetadata>,
    },
    
    /// Key-value pairs
    KeyValue {
        pairs: Vec<(String, String)>,
        title: Option<String>,
    },
    
    /// Simple list of items
    List {
        items: Vec<String>,
        title: Option<String>,
        numbered: bool,
    },
    
    /// Plain text output
    Text {
        content: String,
    },
    
    /// JSON structured data
    Json {
        data: Value,
    },
    
    /// Success message
    Success {
        message: String,
        details: Option<String>,
    },
    
    /// Error message
    Error {
        message: String,
        details: Option<String>,
        code: Option<i32>,
    },
    
    /// Warning message
    Warning {
        message: String,
        details: Option<String>,
    },
    
    /// Information message
    Info {
        message: String,
        details: Option<String>,
    },
    
    /// Progress indication
    Progress {
        current: u64,
        total: u64,
        message: String,
    },
    
    /// Multiple sections
    MultiSection {
        sections: Vec<OutputSection>,
    },
}

/// Table metadata for enhanced display
#[derive(Debug, Clone)]
pub struct TableMetadata {
    /// Total number of items (may be more than displayed rows)
    pub total_items: Option<usize>,
    
    /// Current page number (for paginated results)
    pub page: Option<usize>,
    
    /// Total pages
    pub total_pages: Option<usize>,
    
    /// Sort column and direction
    pub sort_info: Option<(String, SortDirection)>,
    
    /// Applied filters
    pub filters: Vec<String>,
}

/// Sort direction for table metadata
#[derive(Debug, Clone)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Output section for multi-section displays
#[derive(Debug)]
pub struct OutputSection {
    /// Section title
    pub title: String,
    
    /// Section content
    pub content: CommandOutput,
    
    /// Whether this section is collapsible
    pub collapsible: bool,
    
    /// Whether this section is initially collapsed
    pub collapsed: bool,
}

/// File type information for file listings
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub file_type: FileType,
    pub size: Option<u64>,
    pub modified: Option<String>,
    pub permissions: Option<String>,
    pub owner: Option<String>,
    pub group: Option<String>,
}

/// File type enumeration
#[derive(Debug, Clone)]
pub enum FileType {
    RegularFile,
    Directory,
    SymbolicLink,
    BlockDevice,
    CharacterDevice,
    Fifo,
    Socket,
    Unknown,
}

impl UniversalFormatter {
    /// Create new universal formatter
    pub fn new() -> Result<Self> {
        Ok(Self {
            cui: AdvancedCUI::new()?,
            command_context: CommandContext::default(),
        })
    }
    
    /// Create formatter with command context
    pub fn with_context(context: CommandContext) -> Result<Self> {
        Ok(Self {
            cui: AdvancedCUI::new()?,
            command_context: context,
        })
    }
    
    /// Format command output based on its type
    pub fn format(&self, output: &CommandOutput) -> Result<String> {
        match output {
            CommandOutput::Table { headers, rows, metadata } => {
                self.format_table(headers, rows, metadata)
            },
            
            CommandOutput::KeyValue { pairs, title } => {
                self.format_key_value(pairs, title.as_deref())
            },
            
            CommandOutput::List { items, title, numbered } => {
                self.format_list(items, title.as_deref(), *numbered)
            },
            
            CommandOutput::Text { content } => {
                Ok(content.clone())
            },
            
            CommandOutput::Json { data } => {
                self.format_json(data)
            },
            
            CommandOutput::Success { message, details } => {
                Ok(self.format_success(message, details.as_deref()))
            },
            
            CommandOutput::Error { message, details, code } => {
                Ok(self.format_error(message, details.as_deref(), *code))
            },
            
            CommandOutput::Warning { message, details } => {
                Ok(self.format_warning(message, details.as_deref()))
            },
            
            CommandOutput::Info { message, details } => {
                Ok(self.format_info(message, details.as_deref()))
            },
            
            CommandOutput::Progress { current, total, message } => {
                Ok(self.cui.format_progress_bar(*current, *total, Some(message)))
            },
            
            CommandOutput::MultiSection { sections } => {
                self.format_multi_section(sections)
            },
        }
    }
    
    /// Format tabular data with metadata
    fn format_table(&self, headers: &[String], rows: &[Vec<String>], metadata: &Option<TableMetadata>) -> Result<String> {
        let mut output = String::new();
        
        // Add metadata information if available
        if let Some(meta) = metadata {
            if let Some(total) = meta.total_items {
                if total > rows.len() {
                    let mut info_msg = String::with_capacity(50);
                    info_msg.push_str("Showing ");
                    info_msg.push_str(&rows.len().to_string());
                    info_msg.push_str(" of ");
                    info_msg.push_str(&total.to_string());
                    info_msg.push_str(" items");
                    output.push_str(&self.cui.format_info_message(&info_msg));
                    output.push('\n');
                }
            }
            
            if !meta.filters.is_empty() {
                let mut filter_msg = String::with_capacity(30 + meta.filters.join(", ").len());
                filter_msg.push_str("Filters applied: ");
                filter_msg.push_str(&meta.filters.join(", "));
                output.push_str(&self.cui.format_info_message(&filter_msg));
                output.push('\n');
            }
            
            if let Some((col, dir)) = &meta.sort_info {
                let direction = match dir {
                    SortDirection::Ascending => "ascending",
                    SortDirection::Descending => "descending",
                };
                let mut sort_msg = String::with_capacity(30 + col.len() + direction.len());
                sort_msg.push_str("Sorted by ");
                sort_msg.push_str(col);
                sort_msg.push_str(" (");
                sort_msg.push_str(direction);
                sort_msg.push(')');
                output.push_str(&self.cui.format_info_message(&sort_msg));
                output.push('\n');
            }
            
            if !output.is_empty() {
                output.push('\n');
            }
        }
        
        // Format the table
        output.push_str(&self.cui.format_table(headers, rows)?);
        
        // Add summary footer for large tables
        if rows.len() > 20 {
            output.push('\n');
            let mut summary = String::with_capacity(30);
            summary.push_str("Total: ");
            summary.push_str(&rows.len().to_string());
            summary.push_str(" rows");
            output.push_str(&self.cui.format_info_message(&summary));
        }
        
        Ok(output)
    }
    
    /// Format key-value pairs
    fn format_key_value(&self, pairs: &[(String, String)], title: Option<&str>) -> Result<String> {
        let mut output = String::new();
        
        if let Some(title) = title {
            output.push_str(&self.cui.format_section_header(title));
            output.push('\n');
        }
        
        let formatted_pairs: Vec<(&str, &str)> = pairs.iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        
        output.push_str(&self.cui.format_key_value_list(&formatted_pairs));
        
        Ok(output)
    }
    
    /// Format list of items
    fn format_list(&self, items: &[String], title: Option<&str>, numbered: bool) -> Result<String> {
        let mut output = String::new();
        
        if let Some(title) = title {
            output.push_str(&self.cui.format_section_header(title));
            output.push('\n');
        }
        
        if numbered {
            for (i, item) in items.iter().enumerate() {
                let num = i + 1;
                let mut line = String::with_capacity(10 + item.len());
                if num < 10 {
                    line.push_str("  ");
                } else if num < 100 {
                    line.push(' ');
                }
                line.push_str(&num.to_string());
                line.push_str(". ");
                line.push_str(item);
                line.push('\n');
                output.push_str(&line);
            }
        } else {
            let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
            output.push_str(&self.cui.format_bullet_list(&item_refs));
        }
        
        if items.len() > 10 {
            output.push('\n');
            let mut total_msg = String::with_capacity(30);
            total_msg.push_str("Total: ");
            total_msg.push_str(&items.len().to_string());
            total_msg.push_str(" items");
            output.push_str(&self.cui.format_info_message(&total_msg));
            ));
        }
        
        Ok(output)
    }
    
    /// Format JSON data
    fn format_json(&self, data: &Value) -> Result<String> {
        match self.command_context.format {
            OutputFormat::Json => {
                // Pretty-print JSON
                Ok(serde_json::to_string_pretty(data)
                    .context("Failed to serialize JSON")?)
            },
            
            OutputFormat::Table | OutputFormat::Auto => {
                // Try to convert JSON to table format
                self.json_to_table(data)
            },
            
            _ => {
                // Fallback to pretty JSON
                Ok(serde_json::to_string_pretty(data)
                    .context("Failed to serialize JSON")?)
            }
        }
    }
    
    /// Convert JSON to table format if possible
    fn json_to_table(&self, data: &Value) -> Result<String> {
        match data {
            Value::Array(items) => {
                if items.is_empty() {
                    return Ok(self.cui.format_info_message("No data found"));
                }
                
                // Extract headers from first object
                if let Some(Value::Object(first_obj)) = items.first() {
                    let headers: Vec<String> = first_obj.keys().cloned().collect();
                    let mut rows = Vec::new();
                    
                    for item in items {
                        if let Value::Object(obj) = item {
                            let row: Vec<String> = headers.iter()
                                .map(|header| {
                                    obj.get(header)
                                        .map(|v| self.json_value_to_string(v))
                                        .unwrap_or_else(|| "".to_string())
                                })
                                .collect();
                            rows.push(row);
                        }
                    }
                    
                    return self.cui.format_table(&headers, &rows);
                }
                
                // Handle array of simple values
                let items: Vec<String> = items.iter()
                    .map(|v| self.json_value_to_string(v))
                    .collect();
                
                Ok(self.format_list(&items, Some("Items"), false)?)
            },
            
            Value::Object(obj) => {
                let pairs: Vec<(String, String)> = obj.iter()
                    .map(|(k, v)| (k.clone(), self.json_value_to_string(v)))
                    .collect();
                
                self.format_key_value(&pairs, Some("Properties"))
            },
            
            _ => {
                Ok(self.json_value_to_string(data))
            }
        }
    }
    
    /// Convert JSON value to display string
    fn json_value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Array(_) | Value::Object(_) => {
                serde_json::to_string(value).unwrap_or_else(|_| "...".to_string())
            },
        }
    }
    
    /// Format success message
    fn format_success(&self, message: &str, details: Option<&str>) -> String {
        let mut output = self.cui.format_success_message(message);
        if let Some(details) = details {
            output.push('\n');
            output.push_str(details);
        }
        output
    }
    
    /// Format error message
    fn format_error(&self, message: &str, details: Option<&str>, code: Option<i32>) -> String {
        let mut msg = String::with_capacity(message.len() + 20); // pre-allocate
        msg.push_str(message);
        if let Some(code) = code {
            msg.push_str(" (exit code: ");
            msg.push_str(&code.to_string());
            msg.push(')');
        }
        
        let mut output = self.cui.format_error_message(&msg);
        if let Some(details) = details {
            output.push('\n');
            output.push_str(details);
        }
        output
    }
    
    /// Format warning message
    fn format_warning(&self, message: &str, details: Option<&str>) -> String {
        let mut output = self.cui.format_warning_message(message);
        if let Some(details) = details {
            output.push('\n');
            output.push_str(details);
        }
        output
    }
    
    /// Format info message
    fn format_info(&self, message: &str, details: Option<&str>) -> String {
        let mut output = self.cui.format_info_message(message);
        if let Some(details) = details {
            output.push('\n');
            output.push_str(details);
        }
        output
    }
    
    /// Format multi-section output
    fn format_multi_section(&self, sections: &[OutputSection]) -> Result<String> {
        let mut output = String::new();
        
        for (i, section) in sections.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            
            // Section header
            output.push_str(&self.cui.format_section_header(&section.title));
            output.push('\n');
            
            // Section content (if not collapsed)
            if !section.collapsed {
                output.push_str(&self.format(&section.content)?);
            } else if section.collapsible {
                output.push_str(&self.cui.format_info_message("(collapsed - use --expand to show)"));
            }
            
            output.push('\n');
        }
        
        Ok(output)
    }
    
    /// Format file listing with icons and metadata
    pub fn format_file_listing(&self, files: &[FileInfo]) -> Result<String> {
        if files.is_empty() {
            return Ok(self.cui.format_info_message("No files found"));
        }
        
        let icons = self.cui.icons();
        let mut headers = vec!["Type".to_string(), "Name".to_string()];
        let mut rows = Vec::new();
        
        // Determine what columns to show based on available data
        let has_size = files.iter().any(|f| f.size.is_some());
        let has_modified = files.iter().any(|f| f.modified.is_some());
        let has_permissions = files.iter().any(|f| f.permissions.is_some());
        
        if has_permissions {
            headers.push("Permissions".to_string());
        }
        if has_size {
            headers.push("Size".to_string());
        }
        if has_modified {
            headers.push("Modified".to_string());
        }
        
        for file in files {
            let mut row = Vec::new();
            
            // File type icon
            let type_icon = match file.file_type {
                FileType::RegularFile => icons.file,
                FileType::Directory => icons.directory,
                FileType::SymbolicLink => "🔗",
                FileType::BlockDevice => "🟦",
                FileType::CharacterDevice => "🟨",
                FileType::Fifo => "🟫",
                FileType::Socket => "🟪",
                FileType::Unknown => "❁E,
            };
            
            row.push(type_icon.to_string());
            row.push(file.name.clone());
            
            if has_permissions {
                row.push(file.permissions.clone().unwrap_or_else(|| "-".to_string()));
            }
            
            if has_size {
                row.push(file.size.map(|s| self.format_size(s)).unwrap_or_else(|| "-".to_string()));
            }
            
            if has_modified {
                row.push(file.modified.clone().unwrap_or_else(|| "-".to_string()));
            }
            
            rows.push(row);
        }
        
        let metadata = TableMetadata {
            total_items: Some(files.len()),
            page: None,
            total_pages: None,
            sort_info: None,
            filters: Vec::new(),
        };
        
        self.format_table(&headers, &rows, &Some(metadata))
    }
    
    /// Format file size in human-readable format
    fn format_size(&self, size: u64) -> String {
        const UNITS: &[&str] = &["B", "K", "M", "G", "T", "P"];
        let mut size = size as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

impl Default for CommandContext {
    fn default() -> Self {
        Self {
            command_name: "unknown".to_string(),
            args: Vec::new(),
            verbose: false,
            format: OutputFormat::Auto,
            color: true,
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Auto
    }
}

/// Convenience functions for quick formatting
impl UniversalFormatter {
    /// Quick table formatting
    pub fn quick_table(headers: &[&str], rows: &[Vec<&str>]) -> String {
        let formatter = Self::new().unwrap();
        let headers: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
        let rows: Vec<Vec<String>> = rows.iter()
            .map(|row| row.iter().map(|s| s.to_string()).collect())
            .collect();
        
        formatter.format_table(&headers, &rows, &None).unwrap_or_else(|_| "Error formatting table".to_string())
    }
    
    /// Quick success message
    pub fn quick_success(message: &str) -> String {
        Self::new().unwrap().format_success(message, None)
    }
    
    /// Quick error message
    pub fn quick_error(message: &str) -> String {
        Self::new().unwrap().format_error(message, None, None)
    }
    
    /// Quick info message
    pub fn quick_info(message: &str) -> String {
        Self::new().unwrap().format_info(message, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatter_creation() {
        let formatter = UniversalFormatter::new().unwrap();
        assert_eq!(formatter.command_context.command_name, "unknown");
    }

    #[test]
    fn test_quick_table() {
        let headers = vec!["Name", "Age"];
        let rows = vec![
            vec!["Alice", "25"],
            vec!["Bob", "30"],
        ];
        
        let output = UniversalFormatter::quick_table(&headers, &rows);
        assert!(output.contains("Name"));
        assert!(output.contains("Alice"));
    }

    #[test]
    fn test_message_formatting() {
        let success = UniversalFormatter::quick_success("Test successful");
        assert!(success.contains("Test successful"));
        
        let error = UniversalFormatter::quick_error("Test failed");
        assert!(error.contains("Test failed"));
    }
}

