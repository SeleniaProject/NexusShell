//! `smart_alias` command - Simple alias management system

use anyhow::Result;
use crate::ui_design::Colorize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SmartAliasManager {
    pub aliases: HashMap<String, AliasInfo>,
}

#[derive(Debug, Clone)]
pub struct AliasInfo {
    pub name: String,
    pub command: String,
    pub description: String,
}

impl SmartAliasManager {
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
        }
    }
    
    pub fn initialize_default_aliases(&mut self) {
        let default_aliases = vec![
            ("l", "ls", "List files"),
            ("ll", "ls -la", "Long list with hidden files"),
            ("la", "ls -la", "List all files"),
            ("c", "clear", "Clear screen"),
            ("x", "exit", "Exit shell"),
            ("g", "grep --color=auto", "Colorized grep"),
            ("..", "cd ..", "Go up one level"),
            ("...", "cd ../..", "Go up two levels"),
        ];
        
        for (name, command, desc) in default_aliases {
            let alias_info = AliasInfo {
                name: name.to_string(),
                command: command.to_string(),
                description: desc.to_string(),
            };
            
            self.aliases.insert(name.to_string(), alias_info);
        }
    }
    
    pub fn create_alias(&mut self, name: &str, command: &str, description: Option<&str>) -> Result<()> {
        let alias = AliasInfo {
            name: name.to_string(),
            command: command.to_string(),
            description: description.unwrap_or("User-defined alias").to_string(),
        };
        
        self.aliases.insert(name.to_string(), alias);
        println!("{}", format!("Created alias: {} -> {}", name, command).success());
        Ok(())
    }
    
    pub fn load_from_file(_path: &str) -> Result<Self> {
        // Simplified implementation
        Ok(Self::new())
    }
}

/// CLI function for smart alias management
pub fn smart_alias_cli(args: &[String]) -> Result<()> {
    let mut manager = SmartAliasManager::new();
    manager.initialize_default_aliases();
    
    match args.get(0).map(|s| s.as_str()) {
        Some("list") | Some("ls") => {
            println!("{}", "Smart Aliases".primary());
            println!("{}", "-".repeat(40).dim());
            
            for alias in manager.aliases.values() {
                println!("{} -> {}", 
                    alias.name.primary(), 
                    alias.command.success()
                );
                println!("  {}", alias.description.dim());
            }
        },
        Some("create") | Some("add") => {
            if args.len() >= 3 {
                let name = &args[1];
                let command = &args[2];
                let description = args.get(3).map(|s| s.as_str());
                manager.create_alias(name, command, description)?;
            } else {
                println!("{}", "Usage: smart_alias create <name> <command> [description]".warning());
            }
        },
        None => {
            println!("{}", "Smart Alias Management".primary());
            println!("{}", "  smart_alias list           - Show all aliases".info());
            println!("{}", "  smart_alias create <n> <c> - Create new alias".info());
        },
        Some(cmd) => {
            println!("{}", format!("Unknown command: {}", cmd).red());
        }
    }
    
    Ok(())
}
