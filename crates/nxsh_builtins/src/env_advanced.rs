//! Advanced environment variable management

use anyhow::Result;
use crate::ui_design::Colorize;
use std::collections::HashMap;
use std::env;

/// Advanced environment management
pub struct EnvAdvanced;

impl EnvAdvanced {
    /// Display environment variables with smart formatting
    pub fn display_formatted() -> Result<()> {
        let vars: HashMap<String, String> = env::vars().collect();
        let mut sorted_vars: Vec<_> = vars.iter().collect();
        sorted_vars.sort_by(|a, b| a.0.cmp(b.0));
        
        println!("{}", "🌍 Environment Variables".primary().bold());
        println!("{}", "═".repeat(60).muted());
        
        for (key, value) in sorted_vars {
            // Color-code based on variable type
            let colored_key = match key.as_str() {
                k if k.starts_with("PATH") => k.success().bold(),
                k if k.starts_with("HOME") => k.info().bold(),
                k if k.starts_with("USER") || k.starts_with("USERNAME") => k.primary().bold(),
                k if k.contains("LANG") || k.contains("LC_") => k.cyan(),
                k if k.contains("SHELL") => k.magenta().bold(),
                _ => k.white(),
            };
            
            // Truncate long values
            let display_value = if value.len() > 80 {
                format!("{}...", &value[..77])
            } else {
                value.clone()
            };
            
            println!("{} = {}", colored_key, display_value.muted());
        }
        
        Ok(())
    }
    
    /// Search environment variables
    pub fn search(pattern: &str) -> Result<()> {
        let vars: HashMap<String, String> = env::vars().collect();
        let pattern_lower = pattern.to_lowercase();
        
        println!("{}", format!("🔍 Searching for '{}'", pattern).primary().bold());
        println!("{}", "─".repeat(40).muted());
        
        let mut found = false;
        for (key, value) in vars.iter() {
            if key.to_lowercase().contains(&pattern_lower) || 
               value.to_lowercase().contains(&pattern_lower) {
                
                let highlighted_key = key.replace(pattern, &pattern.yellow().bold());
                let highlighted_value = if value.len() > 100 {
                    format!("{}...", &value[..97])
                } else {
                    value.clone()
                }.replace(pattern, &pattern.yellow().bold());
                
                println!("{} = {}", highlighted_key.success(), highlighted_value);
                found = true;
            }
        }
        
        if !found {
            println!("{}", "No matches found".warning());
        }
        
        Ok(())
    }
    
    /// Show PATH components nicely
    pub fn show_path() -> Result<()> {
        if let Ok(path) = env::var("PATH") {
            println!("{}", "📂 PATH Components".primary().bold());
            println!("{}", "─".repeat(40).muted());
            
            for (i, component) in path.split(':').enumerate() {
                let exists = std::path::Path::new(component).exists();
                let status = if exists { "✓".success() } else { "✗".error() };
                let colored_path = if exists { component.cyan() } else { component.muted() };
                
                println!("{:2}. {} {}", (i + 1).to_string().secondary(), status, colored_path);
            }
        } else {
            println!("{}", "PATH environment variable not found".error());
        }
        
        Ok(())
    }
    
    /// Export with validation
    pub fn export_validated(key: &str, value: &str) -> Result<()> {
        // Validate key format
        if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(anyhow::anyhow!("Invalid variable name: {}", key));
        }
        
        env::set_var(key, value);
        println!("{} {} = {}", 
            "✓ Exported:".success(), 
            key.primary().bold(), 
            value.info()
        );
        
        Ok(())
    }
}

/// CLI function for advanced environment management
pub fn env_advanced_cli(args: &[String]) -> Result<()> {
    match args.get(0).map(|s| s.as_str()) {
        Some("list") | Some("ls") | None => {
            EnvAdvanced::display_formatted()?;
        },
        Some("search") | Some("grep") => {
            if let Some(pattern) = args.get(1) {
                EnvAdvanced::search(pattern)?;
            } else {
                println!("{}", "Usage: env search <pattern>".warning());
            }
        },
        Some("path") => {
            EnvAdvanced::show_path()?;
        },
        Some("export") | Some("set") => {
            if args.len() >= 3 {
                EnvAdvanced::export_validated(&args[1], &args[2])?;
            } else {
                println!("{}", "Usage: env export <KEY> <VALUE>".warning());
            }
        },
        Some("help") => {
            println!("{}", "Advanced Environment Management".primary().bold());
            println!("{}", "  env [list]           - Show all variables".info());
            println!("{}", "  env search <pattern> - Search variables".info());
            println!("{}", "  env path            - Show PATH components".info());
            println!("{}", "  env export <K> <V>  - Set variable".info());
        },
        Some(cmd) => {
            println!("{}", format!("Unknown command: {}", cmd).error());
        }
    }
    
    Ok(())
}
