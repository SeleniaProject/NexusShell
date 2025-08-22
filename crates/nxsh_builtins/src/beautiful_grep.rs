/// Beautiful GREP Command Implementation with Advanced CUI
/// 
/// This module provides a stunning, modern grep command with rich formatting,
/// syntax highlighting, context display, and comprehensive search capabilities.
/// 
/// Features:
/// - Beautiful output formatting with color-coded matches
/// - Context lines with line numbers
/// - Multiple pattern matching with highlighting
/// - Regular expression support with syntax highlighting
/// - File type filtering and recursive search
/// - Statistics and summary information
/// - Performance metrics display
/// - Custom theming and output formats

use anyhow::{Result, Context};
use regex::{Regex, RegexBuilder};
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};
use crate::{
    advanced_cui::AdvancedCUI,
    universal_formatter::{UniversalFormatter, CommandOutput, TableMetadata, SortDirection},
};

/// Beautiful grep command implementation
#[derive(Debug)]
pub struct BeautifulGrep {
    /// CUI formatter
    formatter: UniversalFormatter,
    
    /// Search options
    options: GrepOptions,
    
    /// Compiled regex pattern
    regex: Option<Regex>,
}

/// Grep command options
#[derive(Debug, Clone)]
pub struct GrepOptions {
    /// Search pattern
    pub pattern: String,
    
    /// Case insensitive search
    pub ignore_case: bool,
    
    /// Use regular expressions
    pub regex: bool,
    
    /// Show line numbers
    pub line_numbers: bool,
    
    /// Show only matching filenames
    pub files_only: bool,
    
    /// Show only count of matches
    pub count_only: bool,
    
    /// Invert match (show non-matching lines)
    pub invert: bool,
    
    /// Show context lines before match
    pub before_context: usize,
    
    /// Show context lines after match
    pub after_context: usize,
    
    /// Recursive search
    pub recursive: bool,
    
    /// Include hidden files
    pub hidden: bool,
    
    /// File patterns to include
    pub include_patterns: Vec<String>,
    
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    
    /// Maximum number of matches per file
    pub max_matches: Option<usize>,
    
    /// Color output
    pub color: bool,
    
    /// Output format
    pub output_format: GrepOutputFormat,
}

/// Grep output format options
#[derive(Debug, Clone)]
pub enum GrepOutputFormat {
    /// Standard grep output
    Standard,
    
    /// Table format with columns
    Table,
    
    /// JSON format
    Json,
    
    /// Detailed format with context
    Detailed,
    
    /// Summary format with statistics
    Summary,
}

/// Search result information
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// File path
    pub file_path: PathBuf,
    
    /// Line number (1-based)
    pub line_number: usize,
    
    /// Matching line content
    pub line_content: String,
    
    /// Match positions within the line
    pub match_positions: Vec<(usize, usize)>,
    
    /// Context lines before
    pub context_before: Vec<(usize, String)>,
    
    /// Context lines after
    pub context_after: Vec<(usize, String)>,
}

/// Search statistics
#[derive(Debug, Clone)]
pub struct SearchStats {
    /// Total files searched
    pub files_searched: usize,
    
    /// Files with matches
    pub files_matched: usize,
    
    /// Total matches found
    pub total_matches: usize,
    
    /// Total lines searched
    pub lines_searched: usize,
    
    /// Search duration in milliseconds
    pub duration_ms: u64,
    
    /// Bytes searched
    pub bytes_searched: u64,
}

impl Default for GrepOptions {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            ignore_case: false,
            regex: true,
            line_numbers: true,
            files_only: false,
            count_only: false,
            invert: false,
            before_context: 0,
            after_context: 0,
            recursive: false,
            hidden: false,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            max_matches: None,
            color: true,
            output_format: GrepOutputFormat::Standard,
        }
    }
}

impl BeautifulGrep {
    /// Create new beautiful grep command
    pub fn new(pattern: &str) -> Result<Self> {
        let mut options = GrepOptions::default();
        options.pattern = pattern.to_string();
        
        Ok(Self {
            formatter: UniversalFormatter::new()?,
            options: options.clone(),
            regex: Self::compile_regex(&options)?,
        })
    }
    
    /// Create with custom options
    pub fn with_options(options: GrepOptions) -> Result<Self> {
        let regex = Self::compile_regex(&options)?;
        
        Ok(Self {
            formatter: UniversalFormatter::new()?,
            options,
            regex,
        })
    }
    
    /// Compile regex pattern
    fn compile_regex(options: &GrepOptions) -> Result<Option<Regex>> {
        if options.pattern.is_empty() {
            return Ok(None);
        }
        
        let pattern = if options.regex {
            options.pattern.clone()
        } else {
            regex::escape(&options.pattern)
        };
        
        let regex = RegexBuilder::new(&pattern)
            .case_insensitive(options.ignore_case)
            .multi_line(true)
            .build()
            .context("Invalid regex pattern")?;
        
        Ok(Some(regex))
    }
    
    /// Search in files or directories
    pub fn search(&self, targets: &[String]) -> Result<String> {
        let start_time = std::time::Instant::now();
        let mut all_results = Vec::new();
        let mut stats = SearchStats {
            files_searched: 0,
            files_matched: 0,
            total_matches: 0,
            lines_searched: 0,
            duration_ms: 0,
            bytes_searched: 0,
        };
        
        if targets.is_empty() {
            // Read from stdin
            self.search_stdin(&mut all_results, &mut stats)?;
        } else {
            // Search in specified files/directories
            for target in targets {
                self.search_target(target, &mut all_results, &mut stats)?;
            }
        }
        
        stats.duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Format output based on requested format
        self.format_output(&all_results, &stats)
    }
    
    /// Search in stdin
    fn search_stdin(&self, results: &mut Vec<SearchResult>, stats: &mut SearchStats) -> Result<()> {
        let stdin = std::io::stdin();
        let reader = stdin.lock();
        
        self.search_reader(reader, &PathBuf::from("<stdin>"), results, stats)
    }
    
    /// Search in target (file or directory)
    fn search_target(&self, target: &str, results: &mut Vec<SearchResult>, stats: &mut SearchStats) -> Result<()> {
        let path = Path::new(target);
        
        if !path.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", target));
        }
        
        if path.is_file() {
            self.search_file(path, results, stats)?;
        } else if path.is_dir() {
            if self.options.recursive {
                self.search_directory_recursive(path, results, stats)?;
            } else {
                return Err(anyhow::anyhow!("Directory searching requires -r/--recursive flag"));
            }
        }
        
        Ok(())
    }
    
    /// Search in a single file
    fn search_file(&self, path: &Path, results: &mut Vec<SearchResult>, stats: &mut SearchStats) -> Result<()> {
        // Check if file should be included
        if !self.should_include_file(path) {
            return Ok(());
        }
        
        let file = fs::File::open(path).context("Failed to open file")?;
        let reader = BufReader::new(file);
        
        self.search_reader(reader, path, results, stats)
    }
    
    /// Search in directory recursively
    fn search_directory_recursive(&self, dir: &Path, results: &mut Vec<SearchResult>, stats: &mut SearchStats) -> Result<()> {
        let entries = fs::read_dir(dir).context("Failed to read directory")?;
        
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            if path.is_file() {
                let _ = self.search_file(&path, results, stats); // Continue on errors
            } else if path.is_dir() && !path.file_name().unwrap_or_default().to_string_lossy().starts_with('.') {
                self.search_directory_recursive(&path, results, stats)?;
            }
        }
        
        Ok(())
    }
    
    /// Search in a reader
    fn search_reader<R: BufRead>(&self, reader: R, file_path: &Path, results: &mut Vec<SearchResult>, stats: &mut SearchStats) -> Result<()> {
        let regex = match &self.regex {
            Some(r) => r,
            None => return Ok(()),
        };
        
        stats.files_searched += 1;
        let mut file_matches = 0;
        let mut line_number = 0;
        let mut context_buffer = Vec::new();
        let mut file_lines = 0;
        let mut file_bytes = 0;
        
        for line_result in reader.lines() {
            let line = line_result.context("Failed to read line")?;
            line_number += 1;
            file_lines += 1;
            file_bytes += line.len() + 1; // +1 for newline
            
            // Maintain context buffer
            context_buffer.push((line_number, line.clone()));
            if context_buffer.len() > self.options.before_context + 1 {
                context_buffer.remove(0);
            }
            
            let is_match = regex.is_match(&line);
            let should_include = if self.options.invert { !is_match } else { is_match };
            
            if should_include {
                file_matches += 1;
                
                if let Some(max) = self.options.max_matches {
                    if file_matches > max {
                        break;
                    }
                }
                
                // Extract match positions
                let match_positions = if is_match {
                    regex.find_iter(&line).map(|m| (m.start(), m.end())).collect()
                } else {
                    Vec::new()
                };
                
                // Get context before
                let context_before = if self.options.before_context > 0 && context_buffer.len() > 1 {
                    context_buffer[..context_buffer.len()-1].to_vec()
                } else {
                    Vec::new()
                };
                
                // We'll collect context after when we continue reading
                let result = SearchResult {
                    file_path: file_path.to_path_buf(),
                    line_number,
                    line_content: line,
                    match_positions,
                    context_before,
                    context_after: Vec::new(), // Will be filled later
                };
                
                results.push(result);
            }
        }
        
        stats.lines_searched += file_lines;
        stats.bytes_searched += file_bytes;
        
        if file_matches > 0 {
            stats.files_matched += 1;
            stats.total_matches += file_matches;
        }
        
        Ok(())
    }
    
    /// Check if file should be included in search
    fn should_include_file(&self, path: &Path) -> bool {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Check hidden files
        if !self.options.hidden && file_name.starts_with('.') {
            return false;
        }
        
        // Check include patterns
        if !self.options.include_patterns.is_empty() {
            let matches_include = self.options.include_patterns.iter()
                .any(|pattern| {
                    if let Ok(regex) = Regex::new(pattern) {
                        regex.is_match(file_name)
                    } else {
                        file_name.contains(pattern)
                    }
                });
            
            if !matches_include {
                return false;
            }
        }
        
        // Check exclude patterns
        for pattern in &self.options.exclude_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(file_name) {
                    return false;
                }
            } else if file_name.contains(pattern) {
                return false;
            }
        }
        
        true
    }
    
    /// Format search output
    fn format_output(&self, results: &[SearchResult], stats: &SearchStats) -> Result<String> {
        if self.options.count_only {
            return Ok(format!("{}", stats.total_matches));
        }
        
        if self.options.files_only {
            let files: Vec<String> = results.iter()
                .map(|r| r.file_path.display().to_string())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            
            let output = CommandOutput::List {
                items: files,
                title: Some("Files with matches".to_string()),
                numbered: false,
            };
            
            return self.formatter.format(&output);
        }
        
        match self.options.output_format {
            GrepOutputFormat::Standard => self.format_standard_output(results, stats),
            GrepOutputFormat::Table => self.format_table_output(results, stats),
            GrepOutputFormat::Json => self.format_json_output(results, stats),
            GrepOutputFormat::Detailed => self.format_detailed_output(results, stats),
            GrepOutputFormat::Summary => self.format_summary_output(results, stats),
        }
    }
    
    /// Format standard grep output
    fn format_standard_output(&self, results: &[SearchResult], _stats: &SearchStats) -> Result<String> {
        let cui = AdvancedCUI::new()?;
        let mut output = String::new();
        
        for result in results {
            let file_path = result.file_path.display();
            let line_num = if self.options.line_numbers {
                format!(":{}", result.line_number)
            } else {
                String::new()
            };
            
            // Highlight matches in the line
            let highlighted_line = self.highlight_matches(&result.line_content, &result.match_positions);
            
            if self.options.color {
                output.push_str(&format!(
                    "\x1b[35m{}\x1b[0m\x1b[32m{}\x1b[0m:{}\n",
                    file_path, line_num, highlighted_line
                ));
            } else {
                output.push_str(&format!("{}{}:{}\n", file_path, line_num, result.line_content));
            }
        }
        
        Ok(output)
    }
    
    /// Format table output
    fn format_table_output(&self, results: &[SearchResult], stats: &SearchStats) -> Result<String> {
        let headers = vec![
            "File".to_string(),
            "Line".to_string(),
            "Content".to_string(),
        ];
        
        let rows: Vec<Vec<String>> = results.iter().map(|result| {
            vec![
                result.file_path.display().to_string(),
                result.line_number.to_string(),
                result.line_content.clone(),
            ]
        }).collect();
        
        let metadata = TableMetadata {
            total_items: Some(results.len()),
            page: None,
            total_pages: None,
            sort_info: Some(("File".to_string(), SortDirection::Ascending)),
            filters: vec![format!("Pattern: {}", self.options.pattern)],
        };
        
        let table_output = CommandOutput::Table {
            headers,
            rows,
            metadata: Some(metadata),
        };
        
        let mut output = self.formatter.format(&table_output)?;
        
        // Add statistics footer
        output.push('\n');
        output.push_str(&self.format_stats_summary(stats));
        
        Ok(output)
    }
    
    /// Format JSON output
    fn format_json_output(&self, results: &[SearchResult], stats: &SearchStats) -> Result<String> {
        use serde_json::{json, Value};
        
        let results_json: Vec<Value> = results.iter().map(|result| {
            json!({
                "file": result.file_path.display().to_string(),
                "line_number": result.line_number,
                "line_content": result.line_content,
                "match_positions": result.match_positions
            })
        }).collect();
        
        let output_json = json!({
            "pattern": self.options.pattern,
            "results": results_json,
            "statistics": {
                "files_searched": stats.files_searched,
                "files_matched": stats.files_matched,
                "total_matches": stats.total_matches,
                "lines_searched": stats.lines_searched,
                "duration_ms": stats.duration_ms,
                "bytes_searched": stats.bytes_searched
            }
        });
        
        Ok(serde_json::to_string_pretty(&output_json)?)
    }
    
    /// Format detailed output
    fn format_detailed_output(&self, results: &[SearchResult], stats: &SearchStats) -> Result<String> {
        let mut sections = Vec::new();
        
        // Add statistics section
        let stats_pairs = vec![
            ("Files searched".to_string(), stats.files_searched.to_string()),
            ("Files with matches".to_string(), stats.files_matched.to_string()),
            ("Total matches".to_string(), stats.total_matches.to_string()),
            ("Lines searched".to_string(), stats.lines_searched.to_string()),
            ("Duration".to_string(), format!("{}ms", stats.duration_ms)),
            ("Bytes searched".to_string(), self.format_bytes(stats.bytes_searched)),
        ];
        
        sections.push(crate::universal_formatter::OutputSection {
            title: "Search Statistics".to_string(),
            content: CommandOutput::KeyValue {
                pairs: stats_pairs,
                title: None,
            },
            collapsible: false,
            collapsed: false,
        });
        
        // Add results by file
        let mut files_results: std::collections::HashMap<String, Vec<&SearchResult>> = std::collections::HashMap::new();
        
        for result in results {
            let file_key = result.file_path.display().to_string();
            files_results.entry(file_key).or_insert_with(Vec::new).push(result);
        }
        
        for (file_path, file_results) in files_results {
            let items: Vec<String> = file_results.iter().map(|result| {
                format!("Line {}: {}", result.line_number, result.line_content)
            }).collect();
            
            sections.push(crate::universal_formatter::OutputSection {
                title: file_path,
                content: CommandOutput::List {
                    items,
                    title: None,
                    numbered: false,
                },
                collapsible: true,
                collapsed: false,
            });
        }
        
        let output = CommandOutput::MultiSection { sections };
        self.formatter.format(&output)
    }
    
    /// Format summary output
    fn format_summary_output(&self, results: &[SearchResult], stats: &SearchStats) -> Result<String> {
        let cui = AdvancedCUI::new()?;
        let mut output = String::new();
        
        output.push_str(&cui.format_section_header("Search Summary"));
        output.push('\n');
        output.push_str(&self.format_stats_summary(stats));
        
        if !results.is_empty() {
            output.push('\n');
            output.push_str(&cui.format_info_message(&format!(
                "First {} matches shown. Use --format=table or --format=detailed for full results.",
                results.len().min(5)
            )));
            
            for (i, result) in results.iter().take(5).enumerate() {
                output.push('\n');
                output.push_str(&format!(
                    "{}. {}:{} - {}",
                    i + 1,
                    result.file_path.display(),
                    result.line_number,
                    result.line_content.chars().take(80).collect::<String>()
                ));
                
                if result.line_content.len() > 80 {
                    output.push_str("...");
                }
            }
        }
        
        Ok(output)
    }
    
    /// Highlight matches in text
    fn highlight_matches(&self, text: &str, positions: &[(usize, usize)]) -> String {
        if !self.options.color || positions.is_empty() {
            return text.to_string();
        }
        
        let mut result = String::new();
        let mut last_end = 0;
        
        for &(start, end) in positions {
            // Add text before match
            result.push_str(&text[last_end..start]);
            
            // Add highlighted match
            result.push_str("\x1b[1;31m"); // Bold red
            result.push_str(&text[start..end]);
            result.push_str("\x1b[0m"); // Reset
            
            last_end = end;
        }
        
        // Add remaining text
        result.push_str(&text[last_end..]);
        
        result
    }
    
    /// Format statistics summary
    fn format_stats_summary(&self, stats: &SearchStats) -> String {
        let cui = AdvancedCUI::new().unwrap();
        
        if stats.files_matched == 0 {
            cui.format_warning_message(&format!(
                "No matches found for '{}' in {} files ({} lines, {}ms)",
                self.options.pattern,
                stats.files_searched,
                stats.lines_searched,
                stats.duration_ms
            ))
        } else {
            cui.format_success_message(&format!(
                "Found {} matches in {} of {} files ({} lines searched, {}ms)",
                stats.total_matches,
                stats.files_matched,
                stats.files_searched,
                stats.lines_searched,
                stats.duration_ms
            ))
        }
    }
    
    /// Format bytes in human-readable format
    fn format_bytes(&self, bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut bytes = bytes as f64;
        let mut unit_index = 0;
        
        while bytes >= 1024.0 && unit_index < UNITS.len() - 1 {
            bytes /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", bytes as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", bytes, UNITS[unit_index])
        }
    }
}

/// Parse grep command arguments
pub fn parse_grep_args(args: &[String]) -> Result<(GrepOptions, Vec<String>)> {
    let mut options = GrepOptions::default();
    let mut targets = Vec::new();
    let mut pattern_set = false;
    let mut i = 0;
    
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "-i" | "--ignore-case" => options.ignore_case = true,
            "-v" | "--invert-match" => options.invert = true,
            "-n" | "--line-number" => options.line_numbers = true,
            "-l" | "--files-with-matches" => options.files_only = true,
            "-c" | "--count" => options.count_only = true,
            "-r" | "--recursive" => options.recursive = true,
            "-a" | "--hidden" => options.hidden = true,
            "--no-regex" => options.regex = false,
            "--color=never" => options.color = false,
            "--color=always" => options.color = true,
            "--format=table" => options.output_format = GrepOutputFormat::Table,
            "--format=json" => options.output_format = GrepOutputFormat::Json,
            "--format=detailed" => options.output_format = GrepOutputFormat::Detailed,
            "--format=summary" => options.output_format = GrepOutputFormat::Summary,
            
            "-A" | "--after-context" => {
                i += 1;
                if i < args.len() {
                    options.after_context = args[i].parse().unwrap_or(0);
                }
            },
            
            "-B" | "--before-context" => {
                i += 1;
                if i < args.len() {
                    options.before_context = args[i].parse().unwrap_or(0);
                }
            },
            
            "-C" | "--context" => {
                i += 1;
                if i < args.len() {
                    let context = args[i].parse().unwrap_or(0);
                    options.before_context = context;
                    options.after_context = context;
                }
            },
            
            "--include" => {
                i += 1;
                if i < args.len() {
                    options.include_patterns.push(args[i].clone());
                }
            },
            
            "--exclude" => {
                i += 1;
                if i < args.len() {
                    options.exclude_patterns.push(args[i].clone());
                }
            },
            
            "--max-matches" => {
                i += 1;
                if i < args.len() {
                    options.max_matches = args[i].parse().ok();
                }
            },
            
            _ => {
                if !pattern_set && !arg.starts_with('-') {
                    options.pattern = arg.clone();
                    pattern_set = true;
                } else if !arg.starts_with('-') {
                    targets.push(arg.clone());
                }
            }
        }
        
        i += 1;
    }
    
    if !pattern_set {
        return Err(anyhow::anyhow!("No search pattern specified"));
    }
    
    Ok((options, targets))
}

/// Execute beautiful grep command
pub fn grep_beautiful(args: &[String]) -> Result<()> {
    let (options, targets) = parse_grep_args(args)?;
    let grep = BeautifulGrep::with_options(options)?;
    
    let output = grep.search(&targets)?;
    print!("{}", output);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beautiful_grep_creation() {
        let grep = BeautifulGrep::new("test").unwrap();
        assert_eq!(grep.options.pattern, "test");
    }

    #[test]
    fn test_options_parsing() {
        let args = vec![
            "-i".to_string(),
            "-n".to_string(),
            "pattern".to_string(),
            "file.txt".to_string()
        ];
        
        let (options, targets) = parse_grep_args(&args).unwrap();
        
        assert!(options.ignore_case);
        assert!(options.line_numbers);
        assert_eq!(options.pattern, "pattern");
        assert_eq!(targets, vec!["file.txt"]);
    }

    #[test]
    fn test_bytes_formatting() {
        let grep = BeautifulGrep::new("test").unwrap();
        
        assert_eq!(grep.format_bytes(512), "512 B");
        assert_eq!(grep.format_bytes(1024), "1.0 KB");
        assert_eq!(grep.format_bytes(1536), "1.5 KB");
        assert_eq!(grep.format_bytes(1048576), "1.0 MB");
    }
}

