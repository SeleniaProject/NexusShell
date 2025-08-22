//! Enhanced history management with smart search and auto-completion

use anyhow::Result;
use crate::ui_design::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Advanced history command with smart features
pub struct HistoryAdvanced {
    history_file: PathBuf,
    max_entries: usize,
    search_cache: HashMap<String, Vec<String>>,
}

impl HistoryAdvanced {
    pub fn new() -> Self {
        Self {
            history_file: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".nxsh_history"),
            max_entries: 10000,
            search_cache: HashMap::new(),
        }
    }
    
    /// Enhanced history search with fuzzy matching
    pub fn search_fuzzy(&self, query: &str) -> Result<Vec<String>> {
        let content = fs::read_to_string(&self.history_file).unwrap_or_default();
        let lines: Vec<String> = content.lines()
            .filter(|line| self.fuzzy_match(line, query))
            .map(|s| s.to_string())
            .collect();
        Ok(lines)
    }
    
    /// Fuzzy matching algorithm
    fn fuzzy_match(&self, text: &str, pattern: &str) -> bool {
        let text = text.to_lowercase();
        let pattern = pattern.to_lowercase();
        
        let mut pattern_chars = pattern.chars().peekable();
        let mut pattern_char = pattern_chars.next();
        
        for text_char in text.chars() {
            if let Some(p) = pattern_char {
                if text_char == p {
                    pattern_char = pattern_chars.next();
                    if pattern_char.is_none() {
                        return true;
                    }
                }
            }
        }
        
        pattern_char.is_none()
    }
    
    /// Get command statistics
    pub fn get_stats(&self) -> Result<HashMap<String, usize>> {
        let content = fs::read_to_string(&self.history_file).unwrap_or_default();
        let mut stats = HashMap::new();
        
        for line in content.lines() {
            let command = line.split_whitespace().next().unwrap_or("").to_string();
            if !command.is_empty() {
                *stats.entry(command).or_insert(0) += 1;
            }
        }
        
        Ok(stats)
    }
    
    /// Show top used commands
    pub fn show_top_commands(&self, limit: usize) -> Result<()> {
        let stats = self.get_stats()?;
        let mut sorted: Vec<_> = stats.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        
        println!("{}", "📊 Top Commands".primary().bold());
        println!("{}", "─".repeat(40).muted());
        
        for (i, (command, count)) in sorted.iter().take(limit).enumerate() {
            let rank = format!("{:2}", i + 1);
            let cmd = format!("{:20}", command);
            let cnt = format!("{:>6}", count);
            
            println!("{} {} {}", 
                rank.secondary(),
                cmd.primary(),
                cnt.success()
            );
        }
        
        Ok(())
    }
}

/// CLI function for enhanced history
pub fn history_advanced_cli(args: &[String]) -> Result<()> {
    let history = HistoryAdvanced::new();
    
    match args.get(0).map(|s| s.as_str()) {
        Some("search") | Some("s") => {
            if let Some(query) = args.get(1) {
                let results = history.search_fuzzy(query)?;
                for result in results.iter().take(20) {
                    println!("{}", result.info());
                }
            } else {
                println!("{}", "Usage: history search <query>".warning());
            }
        },
        Some("stats") => {
            history.show_top_commands(20)?;
        },
        Some("top") => {
            let limit = args.get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(10);
            history.show_top_commands(limit)?;
        },
        _ => {
            println!("{}", "Enhanced History Commands:".primary().bold());
            println!("{}", "  history search <query>  - Fuzzy search history".info());
            println!("{}", "  history stats          - Show command statistics".info());
            println!("{}", "  history top [N]        - Show top N commands".info());
        }
    }
    
    Ok(())
}
