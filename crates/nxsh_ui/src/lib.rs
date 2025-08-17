//! NexusShell UI Library - CUI (Character User Interface) Implementation
//! 
//! This library provides the user interface components for NexusShell,
//! focusing on CUI (Character User Interface) rather than TUI (Terminal User Interface)
//! for improved performance, reduced complexity, and better POSIX compatibility.

// Primary CUI modules - performance and simplicity focused
pub mod app;
pub mod cui_app;
pub mod startup_profiler; // Startup measurement utilities
pub mod status_line; // Status line metrics (CPU/MEM/Net/Battery)
pub mod enhanced_ui;
pub mod line_editor;
pub mod history_crypto; // History encryption (Argon2id + AES-GCM)
pub mod prompt;
pub mod ui_ux; // Advanced UI/UX system

// Supporting modules for CUI functionality
pub mod themes;
pub mod theme_validator; // Theme validation and schema support
pub mod completion;
pub mod config;
pub mod accessibility; // Accessibility support for TTY blind mode and color vision
pub mod ansi_render; // ANSI-to-PNG rendering helpers

// Additional CUI modules for comprehensive functionality
pub mod simple_cui; // Emergency fallback mode

// Legacy TUI modules (deprecated - being phased out for CUI)
pub mod tui; // Re-enabled for accessibility support (supports_color function)
// pub mod widgets; // Disabled - replaced with enhanced_ui formatters
// pub mod highlighting; // Disabled - simplified in line_editor

// Test modules
#[cfg(test)]
pub mod cui_tests;
#[cfg(test)]
pub mod tty_nocolor_tests;
#[cfg(test)]
mod app_tests;

// Export primary CUI interface
pub use cui_app::CUIApp as App;
pub use ui_ux::{UIUXSystem, Theme as UITheme, PromptContext, InputResult, Completion};
pub use enhanced_ui::{
    CuiFormatter, 
    DisplayTheme, 
    TableRow, 
    ProgressIndicator,
    StatusType
};
pub use line_editor::{NexusLineEditor, LineEditorConfig};
pub use prompt::{PromptFormatter, PromptConfig};

// Re-export essential types for compatibility
pub use themes::{NexusTheme, ThemeFormat, ColorScheme, SerializableStyle};
pub use completion::{CompletionConfig, NexusCompleter};
pub use config::{CUIConfig, UiConfig as UIConfig, NexusConfig, CompletionConfig as GlobalCompletionConfig};

// Emergency fallback only as backup
pub use simple_cui::{SimpleCUI, run_emergency_cui};

/// Ultra-minimal CUI entry point for maximum startup performance
/// Bypasses all unnecessary initialization and overhead while maintaining full functionality
pub async fn run_cui_minimal(start_time: std::time::Instant) -> anyhow::Result<()> {
    // Initialize startup measurement (if enabled)
    if startup_profiler::is_enabled() {
        startup_profiler::init_with_cli_start(start_time);
    }
    // Performance tracking for optimization
    let startup_us = start_time.elapsed().as_micros();
    if startup_us > 5000 {  // > 5ms
        eprintln!("⚠️  NexusShell startup: {startup_us}μs (target: <5000μs)");
    } else {
        eprintln!("✅ NexusShell startup: {startup_us}μs");
    }
    
    // COMPLETE initialization as required - ALL components loaded immediately
    let mut app = App::new_minimal()?;  // Now implements FULL functionality
    if startup_profiler::is_enabled() {
        startup_profiler::mark_cui_init_done(std::time::Instant::now());
    }
    #[cfg(feature = "async")]
    { app.run().await }
    #[cfg(not(feature = "async"))]
    { app.run() }
}

/// Main entry point for CUI mode with comprehensive functionality
/// This replaces the previous TUI-based interface with a streamlined CUI
pub async fn run_cui() -> anyhow::Result<()> {
    let mut app = App::new()?;
    #[cfg(feature = "async")]
    { app.run().await }
    #[cfg(not(feature = "async"))]
    { app.run() }
}

/// Main entry point for CUI mode with startup timing and full features
/// This version tracks startup performance for optimization
pub async fn run_cui_with_timing(start_time: std::time::Instant) -> anyhow::Result<()> {
    let mut app = App::new()?;
    if startup_profiler::is_enabled() {
        startup_profiler::init_with_cli_start(start_time);
        startup_profiler::mark_cui_init_done(std::time::Instant::now());
    }
    
    // Display startup performance
    let startup_ms = start_time.elapsed().as_millis();
    if startup_ms > 10 {
        eprintln!("⚠️  NexusShell startup: {startup_ms:.2}ms (target: <5ms)");
    } else {
        eprintln!("✅ NexusShell startup: {startup_ms:.2}ms");
    }
    
    #[cfg(feature = "async")]
    { app.run().await }
    #[cfg(not(feature = "async"))]
    { app.run() }
}

/// Run CUI with custom configuration and full functionality
pub async fn run_cui_with_config(_config: UIConfig) -> anyhow::Result<()> {
    // Create application and apply provided UI configuration before running.
    let mut app = App::new()?;
    // Bridge UI-only configuration into the running app. This uses a dedicated
    // proxy on App that forwards to the internal configuration manager.
    app.apply_ui_config(_config)?;
    #[cfg(feature = "async")]
    { app.run().await }
    #[cfg(not(feature = "async"))]
    { app.run() }
}

/// Check CUI compatibility with comprehensive terminal feature detection
pub fn check_cui_compatibility() -> CUICompatibility {
    use crossterm::terminal;
    
    // Check comprehensive terminal capabilities required for CUI
    let has_ansi_support = std::env::var("TERM").is_ok();
    let has_color_support = crossterm::style::available_color_count() > 8;
    let _has_unicode_support = std::env::var("LANG").map(|l| l.contains("UTF-8")).unwrap_or(false);
    
    if let Ok((width, height)) = terminal::size() {
        if width >= 80 && height >= 24 {
            if has_ansi_support && has_color_support {
                CUICompatibility::FullySupported
            } else if has_ansi_support {
                CUICompatibility::LimitedSupport("Limited color support".to_string())
            } else {
                CUICompatibility::LimitedSupport("No ANSI support".to_string())
            }
        } else {
            CUICompatibility::LimitedSupport(
                format!("Terminal too small: {width}x{height} (minimum: 80x24)")
            )
        }
    } else {
        CUICompatibility::NotSupported("Cannot detect terminal size".to_string())
    }
}

/// CUI compatibility levels with comprehensive feature detection
#[derive(Debug, Clone, PartialEq)]
pub enum CUICompatibility {
    /// Full CUI features available
    FullySupported,
    /// Limited CUI features (fallback mode)
    LimitedSupport(String),
    /// CUI not supported (text-only mode)
    NotSupported(String),
}

impl CUICompatibility {
    /// Generate a human-readable report of compatibility status
    pub fn report(&self) -> String {
        match self {
            CUICompatibility::FullySupported => {
                "CUI Compatibility: Full support available".to_string()
            },
            CUICompatibility::LimitedSupport(reason) => {
                format!("CUI Compatibility: Limited support - {reason}")
            },
            CUICompatibility::NotSupported(reason) => {
                format!("CUI Compatibility: Not supported - {reason}")
            },
        }
    }
    
    /// Check if any level of CUI is supported
    pub fn is_supported(&self) -> bool {
        !matches!(self, CUICompatibility::NotSupported(_))
    }
    
    /// Check if full CUI features are supported
    pub fn is_fully_supported(&self) -> bool {
        matches!(self, CUICompatibility::FullySupported)
    }
}

// Re-export all essential types and functions
pub use anyhow::{Result, Context};
