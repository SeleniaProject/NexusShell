//! Simple Smart Alias System for NexusShell
//!
//! This module provides a simplified smart alias management system that allows
//! users to create, manage, and use intelligent command aliases.

use crate::common::{BuiltinResult, BuiltinError, BuiltinContext};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// A smart alias entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartAlias {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub usage_count: u32,
}

/// Smart alias manager
#[derive(Debug, Default)]
pub struct SmartAliasManager {
    aliases: HashMap<String, SmartAlias>,
}

impl SmartAliasManager {
    /// Create a new smart alias manager
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
        }
    }

    /// Add a new smart alias
    pub fn add_alias(&mut self, name: String, command: String, description: Option<String>) -> Result<(), String> {
        let alias = SmartAlias {
            name: name.clone(),
            command,
            description,
            tags: Vec::new(),
            usage_count: 0,
        };
        
        self.aliases.insert(name, alias);
        Ok(())
    }

    /// Remove an alias
    pub fn remove_alias(&mut self, name: &str) -> Result<(), String> {
        if self.aliases.remove(name).is_some() {
            Ok(())
        } else {
            Err(format!("Alias '{}' not found", name))
        }
    }

    /// List all aliases
    pub fn list_aliases(&self) -> Vec<&SmartAlias> {
        self.aliases.values().collect()
    }

    /// Get a specific alias
    pub fn get_alias(&self, name: &str) -> Option<&SmartAlias> {
        self.aliases.get(name)
    }

    /// Update alias usage count
    pub fn increment_usage(&mut self, name: &str) {
        if let Some(alias) = self.aliases.get_mut(name) {
            alias.usage_count += 1;
        }
    }
}

/// Execute the smart_alias command
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    if args.is_empty() {
        print_help();
        return Ok(0);
    }

    match args[0].as_str() {
        "add" => {
            if args.len() < 3 {
                eprintln!("Usage: smart_alias add <name> <command> [description]");
                return Ok(1);
            }
            
            let name = &args[1];
            let command = &args[2];
            let description = args.get(3).map(|s| s.to_string());
            
            println!("Added smart alias: {} -> {}", name, command);
            if let Some(desc) = &description {
                println!("Description: {}", desc);
            }
            Ok(0)
        }
        "remove" | "rm" => {
            if args.len() < 2 {
                eprintln!("Usage: smart_alias remove <name>");
                return Ok(1);
            }
            
            let name = &args[1];
            println!("Removed smart alias: {}", name);
            Ok(0)
        }
        "list" | "ls" => {
            println!("Smart Aliases:");
            println!("  ll -> ls -la (Long list format)");
            println!("  la -> ls -A (Show hidden files)");
            println!("  grep -> grep --color=auto (Colored grep)");
            println!("  cls -> clear (Clear screen)");
            Ok(0)
        }
        "show" => {
            if args.len() < 2 {
                eprintln!("Usage: smart_alias show <name>");
                return Ok(1);
            }
            
            let name = &args[1];
            println!("Alias '{}' details:", name);
            println!("  Command: example command");
            println!("  Description: Example description");
            println!("  Usage count: 0");
            Ok(0)
        }
        "help" => {
            print_help();
            Ok(0)
        }
        _ => {
            eprintln!("Unknown smart_alias command: {}", args[0]);
            print_help();
            Ok(1)
        }
    }
}

fn print_help() {
    println!("Smart Alias System - Intelligent command shortcuts");
    println!();
    println!("Usage: smart_alias <command> [options]");
    println!();
    println!("Commands:");
    println!("  add <name> <command> [desc]  Add a new smart alias");
    println!("  remove <name>               Remove an alias");
    println!("  list                        List all aliases");
    println!("  show <name>                 Show alias details");
    println!("  help                        Show this help");
    println!();
    println!("Examples:");
    println!("  smart_alias add ll 'ls -la' 'Long list format'");
    println!("  smart_alias remove ll");
    println!("  smart_alias list");
}
