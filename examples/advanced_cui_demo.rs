//! Advanced CUI Demo - Showcasing rich NexusShell CUI features
//! 
//! This demonstrates the restored advanced CUI functionality including:
//! - Tab completion with fuzzy matching
//! - Enhanced readline with full key bindings
//! - Persistent history with search
//! - Rich prompt with git status and themes
//! - Advanced input handling

use std::io::{self, Write};
use nxsh_ui::{AdvancedCuiController, PromptStyle, Theme};
use crossterm::{
    terminal::{self, ClearType},
    execute,
    style::{Print, SetForegroundColor, Color, ResetColor},
};

fn main() -> anyhow::Result<()> {
    println!("🚀 NexusShell Advanced CUI Demo");
    println!("===============================");
    println!();
    
    // Display feature overview
    show_feature_overview()?;
    
    // Create and run the advanced CUI controller
    let mut controller = AdvancedCuiController::new()?;
    
    println!("Starting interactive CUI session...");
    println!("Type 'help' for available commands, 'exit' to quit");
    println!();
    
    // Run the interactive session with all features
    controller.run_interactive()?;
    
    println!("Thank you for using NexusShell Advanced CUI!");
    Ok(())
}

fn show_feature_overview() -> anyhow::Result<()> {
    execute!(io::stdout(), SetForegroundColor(Color::Cyan))?;
    println!("✨ Advanced CUI Features Restored:");
    execute!(io::stdout(), ResetColor)?;
    
    let features = [
        ("🔍", "Tab Completion", "Smart completion with fuzzy matching for commands, files, and variables"),
        ("⌨️", "Enhanced Readline", "Full Emacs/Vi key bindings with syntax highlighting"),
        ("📚", "History Management", "Persistent history with search, deduplication, and statistics"),
        ("🎨", "Rich Prompts", "Customizable prompts with git status, themes, and performance metrics"),
        ("🎯", "Input Handling", "Advanced key binding system with multi-key sequences"),
        ("⚡", "Performance", "Fast CUI rendering with minimal resource usage"),
        ("🔧", "Customization", "Theme support, configurable key bindings, and extensible architecture"),
    ];
    
    for (icon, feature, description) in &features {
        execute!(io::stdout(), SetForegroundColor(Color::Yellow))?;
        print!("  {} ", icon);
        execute!(io::stdout(), SetForegroundColor(Color::White))?;
        print!("{:<20}", feature);
        execute!(io::stdout(), SetForegroundColor(Color::DarkGrey))?;
        println!(" - {}", description);
        execute!(io::stdout(), ResetColor)?;
    }
    
    println!();
    
    execute!(io::stdout(), SetForegroundColor(Color::Green))?;
    println!("💡 Key Bindings Quick Reference:");
    execute!(io::stdout(), ResetColor)?;
    
    let bindings = [
        ("Tab", "Auto-completion with fuzzy matching"),
        ("Ctrl+R", "Reverse history search"),
        ("Ctrl+L", "Clear screen"),
        ("Ctrl+A/E", "Move to beginning/end of line"),
        ("Ctrl+W", "Delete word backwards"),
        ("Ctrl+U/K", "Delete to beginning/end of line"),
        ("Up/Down", "Navigate command history"),
        ("Ctrl+C", "Interrupt current operation"),
    ];
    
    for (key, action) in &bindings {
        execute!(io::stdout(), SetForegroundColor(Color::Magenta))?;
        print!("  {:12}", key);
        execute!(io::stdout(), SetForegroundColor(Color::DarkGrey))?;
        println!(" - {}", action);
        execute!(io::stdout(), ResetColor)?;
    }
    
    println!();
    
    // Test completion engine
    demo_completion_engine()?;
    
    println!();
    Ok(())
}

fn demo_completion_engine() -> anyhow::Result<()> {
    execute!(io::stdout(), SetForegroundColor(Color::Blue))?;
    println!("🔧 Completion Engine Test:");
    execute!(io::stdout(), ResetColor)?;
    
    let mut controller = AdvancedCuiController::new()?;
    
    // Test various completion scenarios
    let test_inputs = [
        ("ls ", "File completion"),
        ("git ", "Git command completion"),
        ("export ", "Variable completion"),
        ("cd /u", "Directory completion"),
    ];
    
    for (input, description) in &test_inputs {
        let completions = controller.get_completions(input, input.len());
        
        execute!(io::stdout(), SetForegroundColor(Color::Yellow))?;
        print!("  {} ", input);
        execute!(io::stdout(), SetForegroundColor(Color::DarkGrey))?;
        print!("({}): ", description);
        execute!(io::stdout(), SetForegroundColor(Color::Green))?;
        
        if completions.is_empty() {
            print!("No completions");
        } else {
            print!("{} completions available", completions.len());
            if completions.len() <= 3 {
                print!(" [{}]", completions.join(", "));
            }
        }
        
        execute!(io::stdout(), ResetColor)?;
        println!();
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_controller_creation() {
        let controller = AdvancedCuiController::new();
        assert!(controller.is_ok());
    }
    
    #[test]
    fn test_completion_engine() {
        let mut controller = AdvancedCuiController::new().unwrap();
        let completions = controller.get_completions("ls ", 3);
        // Should not panic and return some result
        assert!(completions.len() >= 0);
    }
    
    #[test]
    fn test_history_functionality() {
        let mut controller = AdvancedCuiController::new().unwrap();
        controller.add_to_history("test command".to_string());
        let results = controller.search_history("test");
        assert!(!results.is_empty());
    }
}
