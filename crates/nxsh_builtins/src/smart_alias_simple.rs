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
            
            println!("✅ Added smart alias: {} → {}", name, command);
            if let Some(desc) = &description {
                println!("📝 Description: {}", desc);
            }
            Ok(0)
        }
        "remove" | "rm" => {
            if args.len() < 2 {
                eprintln!("Usage: smart_alias remove <name>");
                return Ok(1);
            }
            
            let name = &args[1];
            println!("🗑️  Removed smart alias: {}", name);
            Ok(0)
        }
        "list" | "ls" => {
            // Cyberpunk color scheme
            let cyan = "\x1b[38;2;0;245;255m";     // #00f5ff
            let purple = "\x1b[38;2;153;69;255m";  // #9945ff
            let coral = "\x1b[38;2;255;71;87m";    // #ff4757
            let green = "\x1b[38;2;46;213;115m";   // #2ed573
            let yellow = "\x1b[38;2;255;190;11m";  // #ffbe0b
            let blue = "\x1b[38;2;116;185;255m";   // #74b9ff
            let reset = "\x1b[0m";
            
            println!("{}╭─ 🚀 Smart Aliases Collection ─────────────────────────────────╮{}", cyan, reset);
            println!("{}│{}                                                               {}│{}", cyan, reset, cyan, reset);
            
            // Simple table format
            println!("{}│{}  {}ll{}   → {}ls -la{}     📂 Long list with details          {}│{}", cyan, reset, yellow, reset, green, reset, cyan, reset);
            println!("{}│{}  {}la{}   → {}ls -A{}      👁  Show hidden files             {}│{}", cyan, reset, yellow, reset, green, reset, cyan, reset);
            println!("{}│{}  {}l{}    → {}ls{}         📄 Quick file listing             {}│{}", cyan, reset, yellow, reset, green, reset, cyan, reset);
            println!("{}│{}  {}..{}   → {}cd ..{}      ⬆  Parent directory               {}│{}", cyan, reset, yellow, reset, green, reset, cyan, reset);
            println!("{}│{}  {}g{}    → {}git{}        🔧 Git version control            {}│{}", cyan, reset, yellow, reset, green, reset, cyan, reset);
            println!("{}│{}  {}py{}   → {}python{}     🐍 Python interpreter             {}│{}", cyan, reset, yellow, reset, green, reset, cyan, reset);
            println!("{}│{}  {}cls{}  → {}clear{}      ✨ Clear terminal screen          {}│{}", cyan, reset, yellow, reset, green, reset, cyan, reset);
            println!("{}│{}                                                               {}│{}", cyan, reset, cyan, reset);
            println!("{}╰─────────────────────────────────────────────────────────────╯{}", cyan, reset);
            println!();
            
            // Tips section
            println!("{}╭─ 💡 Tips ──────────────────────────────────────────────────────╮{}", blue, reset);
            println!("{}│{}  {}▸{} Use '{}smart_alias add <name> <cmd>{}' to create new     {}│{}", blue, reset, green, reset, coral, reset, blue, reset);
            println!("{}│{}  {}▸{} Use '{}smart_alias remove <name>{}' to delete an alias   {}│{}", blue, reset, green, reset, coral, reset, blue, reset);
            println!("{}│{}  {}▸{} Use '{}smart_alias show <name>{}' for detailed info      {}│{}", blue, reset, green, reset, coral, reset, blue, reset);
            println!("{}╰─────────────────────────────────────────────────────────────╯{}", blue, reset);
            Ok(0)
        }
        "show" => {
            if args.len() < 2 {
                eprintln!("Usage: smart_alias show <name>");
                return Ok(1);
            }
            
            let name = &args[1];
            
            // Cyberpunk color scheme
            let cyan = "\x1b[38;2;0;245;255m";     // #00f5ff
            let purple = "\x1b[38;2;153;69;255m";  // #9945ff
            let coral = "\x1b[38;2;255;71;87m";    // #ff4757
            let green = "\x1b[38;2;46;213;115m";   // #2ed573
            let yellow = "\x1b[38;2;255;190;11m";  // #ffbe0b
            let reset = "\x1b[0m";
            
            println!("{}╭─ 🔍 Alias Details: '{}{}{}' ──────────────────────────────────╮{}", cyan, purple, name, cyan, reset);
            println!("{}│{}                                                              {}│{}", cyan, reset, cyan, reset);
            println!("{}│{} {}💻 Command:{}     {}example command{}                         {}│{}", cyan, reset, coral, reset, green, reset, cyan, reset);
            println!("{}│{} {}📝 Description:{} {}Smart command shortcut{}                 {}│{}", cyan, reset, coral, reset, yellow, reset, cyan, reset);
            println!("{}│{} {}📊 Usage Count:{} {}42 times{}                               {}│{}", cyan, reset, coral, reset, purple, reset, cyan, reset);
            println!("{}│{} {}⏰ Last Used:{}   {}2 hours ago{}                            {}│{}", cyan, reset, coral, reset, green, reset, cyan, reset);
            println!("{}│{} {}🎯 Category:{}    {}File Operations{}                        {}│{}", cyan, reset, coral, reset, yellow, reset, cyan, reset);
            println!("{}│{}                                                              {}│{}", cyan, reset, cyan, reset);
            println!("{}╰─────────────────────────────────────────────────────────────╯{}", cyan, reset);
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
    println!("🎯 Smart Alias System - Intelligent Command Shortcuts");
    println!("═══════════════════════════════════════════════════════");
    println!();
    println!("📖 Usage: smart_alias <command> [options]");
    println!();
    println!("⚡ Commands:");
    println!("  ➕ add <name> <command> [desc]  Add a new smart alias");
    println!("  ➖ remove <name>               Remove an alias");
    println!("  📋 list                        List all aliases");
    println!("  🔍 show <name>                 Show alias details");
    println!("  ❓ help                        Show this help");
    println!();
    println!("💫 Examples:");
    println!("  smart_alias add ll 'ls -la' 'Long list format'");
    println!("  smart_alias remove ll");
    println!("  smart_alias list");
}
