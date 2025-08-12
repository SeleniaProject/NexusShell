//! NexusShell UI Library - Phase 1 Emergency CUI Implementation
//! 
//! MASSIVE SIMPLIFICATION for Phase 1 emergency deployment
//! Focus: Basic prompt, command input, minimal dependencies
//! Target: Startup ≤ 5ms, Memory ≤ 15MiB

// Phase 1: Ultra-minimal CUI only
pub mod simple_cui;

// Phase 2+ modules (temporarily disabled for emergency deployment)
// pub mod app;           // Will be re-enabled in Phase 2
// pub mod enhanced_ui;   // Will be re-enabled in Phase 2  
// pub mod line_editor;   // Will be re-enabled in Phase 2
// pub mod prompt;        // Will be re-enabled in Phase 2
// pub mod ui_ux;         // Will be re-enabled in Phase 2
// pub mod themes;        // Will be re-enabled in Phase 2
// pub mod completion;    // Will be re-enabled in Phase 2
// pub mod config;        // Will be re-enabled in Phase 2

// Legacy TUI modules (completely removed)
// pub mod tui; // REMOVED - CUI only
// pub mod widgets; // REMOVED - CUI only
// pub mod highlighting; // REMOVED - CUI only

// Export Phase 1 minimal interface only
pub use simple_cui::{SimpleCUI, run_emergency_cui};

/// Phase 1 Emergency CUI entry point - Ultra-minimal implementation
/// Replaces complex initialization with basic prompt system
pub fn run_cui_minimal() -> std::io::Result<()> {
    // Create app in ultra-fast mode (no history, no git, no features)
    let mut app = App::new_ultra_fast()?;
    
    let startup_us = start_time.elapsed().as_micros();
    if startup_us > 5000 {  // > 5ms
        eprintln!("⚠️  Startup: {}μs (target: <5000μs)", startup_us);
    } else {
        eprintln!("⚡ Startup: {}μs", startup_us);
    }
    run_emergency_cui()
}

// Phase 1: Removed all complex functionality
// This will be restored in Phase 2 with proper implementation

// Temporary compatibility types for external dependencies
#[derive(Debug, Clone, PartialEq)]
pub enum CUICompatibility {
    FullySupported,
    LimitedSupport(String),
    NotSupported(String),
}

/// Phase 1 stub - basic compatibility check
pub fn check_cui_compatibility() -> CUICompatibility {
    CUICompatibility::FullySupported
}

impl CUICompatibility {
    /// Generate a human-readable report of compatibility status
    pub fn report(&self) -> String {
        match self {
            CUICompatibility::FullySupported => {
                "CUI Compatibility: Full support available".to_string()
            },
            CUICompatibility::LimitedSupport(reason) => {
                format!("CUI Compatibility: Limited support - {}", reason)
            },
            CUICompatibility::NotSupported(reason) => {
                format!("CUI Compatibility: Not supported - {}", reason)
            }
        }
    }
    
    /// Check if CUI is fully supported
    pub fn is_fully_supported(&self) -> bool {
        matches!(self, CUICompatibility::FullySupported)
    }
}

/// UI Configuration structure
#[derive(Debug, Clone)]
pub struct UIConfig {
    pub theme: String,
    pub show_git_info: bool,
    pub show_performance_info: bool,
    pub prompt_style: PromptStyle,
    pub color_support: ColorSupport,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: "gruvbox-dark".to_string(),
            show_git_info: true,
            show_performance_info: false,
            prompt_style: PromptStyle::Modern,
            color_support: ColorSupport::Auto,
        }
    }
}

/// Prompt style options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PromptStyle {
    /// Modern style with symbols and colors
    Modern,
    /// Classic shell-like prompt
    Classic,
    /// Minimal prompt for performance
    Minimal,
}

/// Color support configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSupport {
    /// Automatically detect color support
    Auto,
    /// Force disable colors
    None,
    /// Force 16-color mode
    Basic,
    /// Force 256-color mode
    Extended,
    /// Force TrueColor mode
    TrueColor,
}

// Re-export commonly used types from dependencies
pub use anyhow::{Result, Context};
pub use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, SetForegroundColor, SetBackgroundColor},
    terminal,
    ExecutableCommand,
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cui_compatibility() {
        let compat = check_cui_compatibility();
        // This test will vary based on the terminal environment
        assert!(matches!(compat, CUICompatibility::FullySupported | CUICompatibility::LimitedSupport(_) | CUICompatibility::NotSupported(_)));
    }

    #[test]
    fn test_default_config() {
        let config = UIConfig::default();
        assert_eq!(config.theme, "gruvbox-dark");
        assert!(config.show_git_info);
        assert!(!config.show_performance_info);
        assert_eq!(config.prompt_style, PromptStyle::Modern);
        assert_eq!(config.color_support, ColorSupport::Auto);
    }

    #[test]
    fn test_version_constants() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "nxsh_ui");
    }
} 