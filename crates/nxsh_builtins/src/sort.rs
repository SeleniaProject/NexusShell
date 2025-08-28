//! Sort command implementation for NexusShell
//!
//! Provides text line sorting functionality with various options.

use crate::common::{BuiltinContext, BuiltinError, BuiltinResult};
use std::cmp::Ordering;
use std::io::{BufRead, BufReader, Write};

/// Execute the sort command
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    let config = parse_args(args)?;
    
    if config.help {
        print_help();
        return Ok(0);
    }
    
    let lines = if config.files.is_empty() {
        // Read from stdin
        read_stdin_lines()?
    } else {
        // Read from files
        read_file_lines(&config.files)?
    };
    
    let sorted_lines = sort_lines(lines, &config)?;
    
    // Output sorted lines
    for line in sorted_lines {
        println!("{}", line);
    }
    
    Ok(0)
}

#[derive(Debug)]
struct SortConfig {
    help: bool,
    reverse: bool,
    numeric: bool,
    unique: bool,
    ignore_case: bool,
    files: Vec<String>,
}

impl Default for SortConfig {
    fn default() -> Self {
        Self {
            help: false,
            reverse: false,
            numeric: false,
            unique: false,
            ignore_case: false,
            files: Vec::new(),
        }
    }
}

fn parse_args(args: &[String]) -> BuiltinResult<SortConfig> {
    let mut config = SortConfig::default();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => config.help = true,
            "--reverse" | "-r" => config.reverse = true,
            "--numeric-sort" | "-n" => config.numeric = true,
            "--unique" | "-u" => config.unique = true,
            "--ignore-case" | "-f" => config.ignore_case = true,
            arg if arg.starts_with('-') => {
                return Err(BuiltinError::InvalidArgument(format!("Unknown option: {}", arg)));
            }
            file => config.files.push(file.to_string()),
        }
        i += 1;
    }
    
    Ok(config)
}

fn read_stdin_lines() -> BuiltinResult<Vec<String>> {
    let stdin = std::io::stdin();
    let reader = stdin.lock();
    
    reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| BuiltinError::IoError(format!("Failed to read from stdin: {}", e)))
}

fn read_file_lines(files: &[String]) -> BuiltinResult<Vec<String>> {
    let mut all_lines = Vec::new();
    
    for file_path in files {
        let file = std::fs::File::open(file_path)
            .map_err(|e| BuiltinError::IoError(format!("Failed to open file '{}': {}", file_path, e)))?;
        
        let reader = BufReader::new(file);
        let lines: Result<Vec<_>, _> = reader.lines().collect();
        
        match lines {
            Ok(mut file_lines) => all_lines.append(&mut file_lines),
            Err(e) => return Err(BuiltinError::IoError(format!("Failed to read file '{}': {}", file_path, e))),
        }
    }
    
    Ok(all_lines)
}

fn sort_lines(mut lines: Vec<String>, config: &SortConfig) -> BuiltinResult<Vec<String>> {
    lines.sort_by(|a, b| {
        let ordering = if config.numeric {
            // Numeric sort
            let a_num = a.trim().parse::<f64>().unwrap_or(0.0);
            let b_num = b.trim().parse::<f64>().unwrap_or(0.0);
            a_num.partial_cmp(&b_num).unwrap_or(Ordering::Equal)
        } else if config.ignore_case {
            // Case-insensitive sort
            a.to_lowercase().cmp(&b.to_lowercase())
        } else {
            // Regular lexicographic sort
            a.cmp(b)
        };
        
        if config.reverse {
            ordering.reverse()
        } else {
            ordering
        }
    });
    
    if config.unique {
        lines.dedup();
    }
    
    Ok(lines)
}

fn print_help() {
    println!("sort - sort lines of text files");
    println!();
    println!("USAGE:");
    println!("    sort [OPTIONS] [FILE...]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help              Show this help message");
    println!("    -r, --reverse           Reverse the result of comparisons");
    println!("    -n, --numeric-sort      Compare according to string numerical value");
    println!("    -u, --unique            Output only the first of equal lines");
    println!("    -f, --ignore-case       Fold lower case to upper case characters");
    println!();
    println!("EXAMPLES:");
    println!("    sort file.txt           Sort lines in file.txt");
    println!("    sort -r file.txt        Sort in reverse order");
    println!("    sort -n numbers.txt     Sort numerically");
    println!("    cat file.txt | sort     Sort input from pipe");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::BuiltinContext;
    
    #[test]
    fn test_sort_basic() {
        let context = BuiltinContext::new();
        // Basic functionality test - would need mock input
        let result = execute(&[], &context);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_sort_help() {
        let context = BuiltinContext::new();
        let result = execute(&["--help".to_string()], &context);
        assert_eq!(result.unwrap(), 0);
    }
}
