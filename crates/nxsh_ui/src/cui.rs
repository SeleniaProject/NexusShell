/// CUI Entry Point for NexusShell
/// 
/// This module provides the main entry point for running NexusShell in CUI mode,
/// replacing the complex TUI system with a high-performance command line interface.

use anyhow::{Result, Context};
use crate::cui_app::CUIApp;
use crate::config::CUIConfig;  // Configuration structure for CUI mode
use serde::{Serialize, Deserialize};

/// Run NexusShell in CUI mode
/// 
/// This is the primary entry point for the CUI interface, providing:
/// - Fast startup (target: ≤5ms)
/// - Low memory usage (target: ≤15MiB)  
/// - High performance command execution
/// - Standard readline-style editing
/// - ANSI color and formatting support
pub async fn run_cui() -> Result<()> {
    // Initialize and run CUI application
    let mut app = CUIApp::new().await
        .context("Failed to initialize CUI application")?;
    
    app.run().await
        .context("CUI application failed during execution")?;
    
    Ok(())
}

/// Run NexusShell in CUI mode with custom configuration
pub async fn run_cui_with_config(config_path: Option<&str>) -> Result<()> {
    use std::fs;
    
    let mut app = CUIApp::new().await
        .context("Failed to initialize CUI application")?;
    
    if let Some(path) = config_path {
        // Load configuration from TOML file
        match fs::read_to_string(path) {
            Ok(config_content) => {
                match toml::from_str::<CUIConfig>(&config_content) {
                    Ok(config) => {
                        eprintln!("✅ Loaded configuration from: {}", path);
                        app.apply_config(config)
                            .context("Failed to apply configuration")?;
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to parse configuration file {}: {}", path, e);
                        eprintln!("    Using default configuration");
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️  Failed to read configuration file {}: {}", path, e);
                eprintln!("    Using default configuration");
            }
        }
    }
    
    app.run().await
        .context("CUI application failed during execution")?;
    
    Ok(())
}

/// Check CUI mode compatibility
/// 
/// Verifies that the terminal supports the features needed for CUI mode:
/// - ANSI escape sequences
/// - Terminal size detection
/// - Basic readline functionality
pub fn check_cui_compatibility() -> Result<CUICompatibility> {
    let mut compatibility = CUICompatibility::default();
    
    // Check terminal size detection
    if let Ok((width, height)) = crossterm::terminal::size() {
        compatibility.terminal_size_detection = true;
        compatibility.terminal_width = width;
        compatibility.terminal_height = height;
    }
    
    // Check ANSI color support
    compatibility.ansi_colors = check_ansi_color_support();
    
    // Check unicode support
    compatibility.unicode_support = check_unicode_support();
    
    // Check terminal type
    compatibility.terminal_type = detect_terminal_type();
    
    Ok(compatibility)
}

/// CUI compatibility information
#[derive(Debug, Default)]
pub struct CUICompatibility {
    /// Terminal supports size detection
    pub terminal_size_detection: bool,
    
    /// Current terminal width
    pub terminal_width: u16,
    
    /// Current terminal height  
    pub terminal_height: u16,
    
    /// ANSI color codes supported
    pub ansi_colors: bool,
    
    /// Unicode characters supported
    pub unicode_support: bool,
    
    /// Detected terminal type
    pub terminal_type: TerminalType,
}

/// Terminal type detection
#[derive(Debug, Default)]
pub enum TerminalType {
    #[default]
    Unknown,
    WindowsConsole,
    WindowsTerminal,
    ConEmu,
    XTerm,
    Gnome,
    KDE,
    ITerm2,
    VSCode,
    Other(String),
}

impl CUICompatibility {
    /// Check if CUI mode is fully supported
    pub fn is_fully_supported(&self) -> bool {
        self.terminal_size_detection && self.ansi_colors
    }
    
    /// Get a compatibility report string
    pub fn report(&self) -> String {
        format!(
            "CUI Compatibility Report:\n\
             • Terminal Size: {}x{} ({})\n\
             • ANSI Colors: {}\n\
             • Unicode Support: {}\n\
             • Terminal Type: {:?}\n\
             • Fully Supported: {}",
            self.terminal_width,
            self.terminal_height,
            if self.terminal_size_detection { "✓" } else { "✗" },
            if self.ansi_colors { "✓" } else { "✗" },
            if self.unicode_support { "✓" } else { "✗" },
            self.terminal_type,
            if self.is_fully_supported() { "✓" } else { "✗" }
        )
    }
}

/// Check if terminal supports ANSI color codes
fn check_ansi_color_support() -> bool {
    // Check NO_COLOR environment variable
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    
    // Check TERM environment variable
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("color") || term == "xterm-256color" || term == "screen-256color" {
            return true;
        }
        if term == "dumb" || term == "unknown" {
            return false;
        }
    }
    
    // Check COLORTERM environment variable
    if std::env::var("COLORTERM").is_ok() {
        return true;
    }
    
    // Default to true for modern terminals
    true
}

/// Check if terminal supports unicode characters
fn check_unicode_support() -> bool {
    // Check locale settings
    for var in &["LC_ALL", "LC_CTYPE", "LANG"] {
        if let Ok(value) = std::env::var(var) {
            if value.to_lowercase().contains("utf") {
                return true;
            }
        }
    }
    
    // Default to true for modern terminals
    true
}

/// Detect terminal type from environment
fn detect_terminal_type() -> TerminalType {
    // Check various terminal-specific environment variables
    if std::env::var("WT_SESSION").is_ok() {
        return TerminalType::WindowsTerminal;
    }
    
    if std::env::var("ConEmuPID").is_ok() {
        return TerminalType::ConEmu;
    }
    
    if std::env::var("TERM_PROGRAM").as_deref() == Ok("vscode") {
        return TerminalType::VSCode;
    }
    
    if std::env::var("TERM_PROGRAM").as_deref() == Ok("iTerm.app") {
        return TerminalType::ITerm2;
    }
    
    if let Ok(term) = std::env::var("TERM") {
        if term.starts_with("xterm") {
            return TerminalType::XTerm;
        }
        if term.contains("gnome") {
            return TerminalType::Gnome;
        }
        if term.contains("konsole") {
            return TerminalType::KDE;
        }
        
        return TerminalType::Other(term);
    }
    
    // Check if running on Windows
    #[cfg(windows)]
    {
        return TerminalType::WindowsConsole;
    }
    
    TerminalType::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cui_compatibility() {
        let compat = check_cui_compatibility().unwrap();
        println!("{}", compat.report());
        
        // Basic compatibility checks should pass
        assert!(compat.terminal_width > 0);
        assert!(compat.terminal_height > 0);
    }
    
    #[test]
    fn test_ansi_color_detection() {
        // This test may vary depending on environment
        let supports_colors = check_ansi_color_support();
        println!("ANSI colors supported: {}", supports_colors);
    }
    
    #[test]
    fn test_unicode_detection() {
        let supports_unicode = check_unicode_support();
        println!("Unicode supported: {}", supports_unicode);
    }
    
    #[test]
    fn test_terminal_type_detection() {
        let terminal_type = detect_terminal_type();
        println!("Terminal type: {:?}", terminal_type);
    }
}
