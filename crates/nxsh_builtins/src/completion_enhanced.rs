//! Enhanced completion engine with intelligent suggestions

use anyhow::Result;
use crate::ui_design::Colorize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Enhanced completion engine
pub struct CompletionEnhanced {
    command_cache: HashMap<String, Vec<String>>,
    path_cache: HashMap<String, Vec<PathBuf>>,
    context_cache: HashMap<String, Vec<String>>,
}

impl CompletionEnhanced {
    pub fn new() -> Self {
        Self {
            command_cache: HashMap::new(),
            path_cache: HashMap::new(),
            context_cache: HashMap::new(),
        }
    }
    
    /// Smart command completion with context awareness
    pub fn complete_command(&mut self, partial: &str, context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // Built-in commands
        let builtins = vec![
            "ls", "cd", "pwd", "cat", "echo", "cp", "mv", "rm", "mkdir", "touch",
            "head", "tail", "grep", "find", "which", "history", "alias", "exit",
            "clear", "env", "export", "source", "help", "man", "ps", "kill",
            "chmod", "chown", "ln", "df", "du", "free", "uptime", "whoami",
            "date", "cal", "wc", "sort", "uniq", "cut", "awk", "sed", "tar",
            "gzip", "gunzip", "zip", "unzip", "curl", "wget", "ssh", "scp",
            "git", "vim", "nano", "code", "python", "node", "cargo", "make"
        ];
        
        for cmd in builtins {
            if cmd.starts_with(partial) {
                suggestions.push(cmd.to_string());
            }
        }
        
        // Smart context-based suggestions
        match context {
            "file_operation" => {
                suggestions.extend(self.get_file_suggestions(partial));
            },
            "directory_navigation" => {
                suggestions.extend(self.get_directory_suggestions(partial));
            },
            "git_command" => {
                suggestions.extend(self.get_git_suggestions(partial));
            },
            _ => {}
        }
        
        suggestions.sort();
        suggestions.dedup();
        suggestions
    }
    
    /// Get file-based suggestions
    fn get_file_suggestions(&self, partial: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(partial) {
                        suggestions.push(name.to_string());
                    }
                }
            }
        }
        
        suggestions
    }
    
    /// Get directory-based suggestions
    fn get_directory_suggestions(&self, partial: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with(partial) {
                            suggestions.push(format!("{}/", name));
                        }
                    }
                }
            }
        }
        
        suggestions
    }
    
    /// Get Git command suggestions
    fn get_git_suggestions(&self, partial: &str) -> Vec<String> {
        let git_commands = vec![
            "add", "commit", "push", "pull", "clone", "status", "log", "diff",
            "branch", "checkout", "merge", "rebase", "reset", "revert", "tag",
            "remote", "fetch", "stash", "cherry-pick", "bisect", "blame", "show"
        ];
        
        git_commands.into_iter()
            .filter(|cmd| cmd.starts_with(partial))
            .map(|s| s.to_string())
            .collect()
    }
    
    /// Display completion suggestions with colors
    pub fn display_suggestions(&self, suggestions: &[String], query: &str) -> Result<()> {
        if suggestions.is_empty() {
            println!("{}", "No suggestions found".muted());
            return Ok(());
        }
        
        println!("{}", format!("Suggestions for '{}':", query).primary());
        println!("{}", "─".repeat(40).muted());
        
        for (i, suggestion) in suggestions.iter().enumerate() {
            let prefix = if i < 9 { format!("{} ", i + 1) } else { "  ".to_string() };
            
            if suggestion.ends_with('/') {
                println!("{}{}", prefix.secondary(), suggestion.cyan().bold());
            } else if suggestion.contains('.') {
                println!("{}{}", prefix.secondary(), suggestion.green());
            } else {
                println!("{}{}", prefix.secondary(), suggestion.primary());
            }
        }
        
        Ok(())
    }
}

/// CLI function for enhanced completion
pub fn completion_enhanced_cli(args: &[String]) -> Result<()> {
    let mut completion = CompletionEnhanced::new();
    
    match args.get(0).map(|s| s.as_str()) {
        Some("complete") => {
            let partial = args.get(1).unwrap_or(&String::new());
            let context = args.get(2).unwrap_or(&"general".to_string());
            
            let suggestions = completion.complete_command(partial, context);
            completion.display_suggestions(&suggestions, partial)?;
        },
        Some("test") => {
            println!("{}", "Testing completion engine...".info());
            let test_cases = vec!["l", "g", "cd", "git"];
            
            for case in test_cases {
                println!("\n{}", format!("Testing '{}':", case).primary());
                let suggestions = completion.complete_command(case, "general");
                completion.display_suggestions(&suggestions, case)?;
            }
        },
        _ => {
            println!("{}", "Enhanced Completion Engine".primary().bold());
            println!("{}", "  completion complete <partial> [context]  - Get completions".info());
            println!("{}", "  completion test                         - Test completion".info());
        }
    }
    
    Ok(())
}
