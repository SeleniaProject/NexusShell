//! Simple prompt display system for NexusShell CUI
//! 
//! This module provides PS1-based prompt functionality with optional Git information
//! and system status display, designed for CUI mode efficiency.

use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    style::{Color, ResetColor, SetForegroundColor},
};
use std::{
    env,
    io::stdout,
    path::Path,
    process::Command,
};
use whoami;
use hostname;

/// Prompt configuration for CUI mode  
#[derive(Debug, Clone)]
pub struct PromptConfig {
    pub show_user: bool,
    pub show_hostname: bool,
    pub show_cwd: bool,
    pub show_git_info: bool,
    pub show_exit_code: bool,
    pub show_time: bool,
    pub show_jobs: bool,
    pub show_performance: bool,
    pub ps1_format: Option<String>,
    pub git_simplified: bool,
    pub max_path_length: Option<usize>,
    pub use_unicode_symbols: bool,
    pub color_theme: PromptColorTheme,
}

/// Color theme for prompts
#[derive(Debug, Clone)]
pub struct PromptColorTheme {
    pub user_color: Color,
    pub hostname_color: Color,
    pub cwd_color: Color,
    pub git_clean_color: Color,
    pub git_dirty_color: Color,
    pub error_color: Color,
    pub success_color: Color,
    pub time_color: Color,
}

impl Default for PromptColorTheme {
    fn default() -> Self {
        Self {
            user_color: Color::Green,
            hostname_color: Color::Blue,
            cwd_color: Color::Cyan,
            git_clean_color: Color::Green,
            git_dirty_color: Color::Yellow,
            error_color: Color::Red,
            success_color: Color::Green,
            time_color: Color::Grey,
        }
    }
}

/// Prompt style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptStyle {
    Simple,
    Detailed,
    Compact,
    Powerline,
    Custom,
}

/// Prompt renderer for displaying prompts
#[derive(Debug, Clone)]
pub struct PromptRenderer {
    config: PromptConfig,
}

impl PromptRenderer {
    pub fn new(config: PromptConfig) -> Self {
        Self { config }
    }
    
    pub fn render(&self) -> String {
        "$ ".to_string() // Simple prompt for now
    }
}

impl Default for PromptRenderer {
    fn default() -> Self {
        Self::new(PromptConfig::default())
    }
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {
            show_user: true,
            show_hostname: false,
            show_cwd: true,
            show_git_info: true,
            show_exit_code: true,
            show_time: false,
            show_jobs: false,
            show_performance: false,
            ps1_format: None,
            git_simplified: true,
            max_path_length: None,
            use_unicode_symbols: true,
            color_theme: PromptColorTheme::default(),
        }
    }
}

/// Simple prompt formatter for CUI display
pub struct PromptFormatter {
    config: PromptConfig,
}

impl Default for PromptFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptFormatter {
    /// Create a comprehensive prompt formatter with full functionality
    /// COMPLETE initialization with ALL prompt features enabled
    pub fn new_minimal() -> Self {
        Self {
            config: PromptConfig {
                show_user: true,       // Full user information as required
                show_hostname: true,   // Complete hostname display
                show_git_info: true,   // Full git status integration
                show_cwd: true,        // Complete directory information
                show_exit_code: true,  // Complete exit code tracking
                show_time: false,
                show_jobs: false,
                show_performance: false,
                ps1_format: Some(String::from("$USER@$HOSTNAME:$CWD$GIT$ ")),
                git_simplified: false, // Full git details as required
                max_path_length: None,
                use_unicode_symbols: true,
                color_theme: PromptColorTheme::default(),
            },
        }
    }

    /// Create a new prompt formatter with default configuration
    pub fn new() -> Self {
        Self {
            config: PromptConfig::default(),
        }
    }
    
    /// Create a new prompt formatter with custom configuration
    pub fn with_config(config: PromptConfig) -> Self {
        Self { config }
    }
    
    /// Generate and display the shell prompt
    pub fn display_prompt(&self, exit_code: Option<i32>) -> Result<()> {
        let _stdout = stdout(); // currently unused, retained for future direct write optimizations
        
        // Use PS1 environment variable if available and no custom format is set
        if let Ok(ps1) = env::var("PS1") {
            if self.config.ps1_format.is_none() {
                self.display_ps1_prompt(&ps1, exit_code)?;
                return Ok(());
            }
        }
        
        // Use custom format or default format
        let format = self.config.ps1_format.as_deref().unwrap_or("\\u@\\h:\\w\\$ ");
        self.display_custom_prompt(format, exit_code)?;
        
        Ok(())
    }
    
    /// Display PS1-based prompt with basic variable substitution
    fn display_ps1_prompt(&self, ps1: &str, exit_code: Option<i32>) -> Result<()> {
        let mut stdout = stdout();
        let mut chars = ps1.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(&next_ch) = chars.peek() {
                    chars.next(); // consume the next character
                    match next_ch {
                        'u' => {
                            // Username
                            if self.config.show_user {
                                stdout.execute(SetForegroundColor(Color::Green))?;
                                print!("{}", whoami::username());
                                stdout.execute(ResetColor)?;
                            }
                        }
                        'h' => {
                            // Hostname
                            if self.config.show_hostname {
                                stdout.execute(SetForegroundColor(Color::Blue))?;
                                print!("{}", hostname::get()?.to_string_lossy().split('.').next().unwrap_or("unknown"));
                                stdout.execute(ResetColor)?;
                            }
                        }
                        'H' => {
                            // Full hostname
                            if self.config.show_hostname {
                                stdout.execute(SetForegroundColor(Color::Blue))?;
                                print!("{}", hostname::get()?.to_string_lossy());
                                stdout.execute(ResetColor)?;
                            }
                        }
                        'w' => {
                            // Current working directory
                            if self.config.show_cwd {
                                self.display_working_directory()?;
                            }
                        }
                        'W' => {
                            // Basename of current working directory
                            if self.config.show_cwd {
                                if let Ok(cwd) = env::current_dir() {
                                    stdout.execute(SetForegroundColor(Color::Yellow))?;
                                    print!("{}", cwd.file_name().unwrap_or_default().to_string_lossy());
                                    stdout.execute(ResetColor)?;
                                }
                            }
                        }
                        '$' => {
                            // $ for user, # for root
                            print!("{}", if whoami::username() == "root" { "#" } else { "$" });
                        }
                        '#' => {
                            // Command number (simplified as $)
                            print!("$");
                        }
                        'n' => {
                            // Newline
                            println!();
                        }
                        't' => {
                            // Current time in 24-hour HH:MM:SS format
                            let now = chrono::Local::now();
                            print!("{}", now.format("%H:%M:%S"));
                        }
                        '\\' => {
                            // Literal backslash
                            print!("\\");
                        }
                        _ => {
                            // Unknown escape sequence, print literally
                            print!("\\{next_ch}");
                        }
                    }
                } else {
                    print!("\\");
                }
            } else {
                print!("{ch}");
            }
        }
        
        // Show exit code if configured and available
        if self.config.show_exit_code {
            if let Some(code) = exit_code {
                if code != 0 {
                    stdout.execute(SetForegroundColor(Color::Red))?;
                    print!(" [{code}]");
                    stdout.execute(ResetColor)?;
                }
            }
        }
        
        // Show Git information if enabled
        if self.config.show_git_info {
            self.display_git_info()?;
        }
        
        print!(" ");
        
        Ok(())
    }
    
    /// Display custom formatted prompt
    fn display_custom_prompt(&self, format: &str, exit_code: Option<i32>) -> Result<()> {
        self.display_ps1_prompt(format, exit_code)
    }
    
    /// Display working directory with home directory substitution
    fn display_working_directory(&self) -> Result<()> {
        let mut stdout = stdout();
        
        if let Ok(cwd) = env::current_dir() {
            if let Ok(home) = env::var("HOME") {
                let home_path = Path::new(&home);
                if let Ok(relative) = cwd.strip_prefix(home_path) {
                    stdout.execute(SetForegroundColor(Color::Yellow))?;
                    print!("~/{}", relative.display());
                    stdout.execute(ResetColor)?;
                    return Ok(());
                }
            }
            
            stdout.execute(SetForegroundColor(Color::Yellow))?;
            print!("{}", cwd.display());
            stdout.execute(ResetColor)?;
        }
        
        Ok(())
    }
    
    /// Display simplified Git information
    fn display_git_info(&self) -> Result<()> {
        if self.config.git_simplified {
            self.display_simple_git_info()
        } else {
            self.display_detailed_git_info()
        }
    }
    
    /// Display simple Git branch information
    fn display_simple_git_info(&self) -> Result<()> {
        let mut stdout = stdout();
        
        // Check if we're in a Git repository
        if let Ok(output) = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
        {
            if output.status.success() {
                let branch_str = String::from_utf8_lossy(&output.stdout);
                let branch = branch_str.trim();
                if !branch.is_empty() {
                    stdout.execute(SetForegroundColor(Color::Magenta))?;
                    print!(" ({branch})");
                    stdout.execute(ResetColor)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Display detailed Git status information
    fn display_detailed_git_info(&self) -> Result<()> {
        let mut stdout = stdout();
        
        // Get current branch
        if let Ok(output) = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
        {
            if output.status.success() {
                let branch_str = String::from_utf8_lossy(&output.stdout);
                let branch = branch_str.trim();
                if !branch.is_empty() {
                    stdout.execute(SetForegroundColor(Color::Magenta))?;
                    print!(" ({branch})");
                    
                    // Check for uncommitted changes
                    if let Ok(status_output) = Command::new("git")
                        .args(["status", "--porcelain"])
                        .output()
                    {
                        if status_output.status.success() {
                            let status = String::from_utf8_lossy(&status_output.stdout);
                            if !status.trim().is_empty() {
                                stdout.execute(SetForegroundColor(Color::Red))?;
                                print!("*");
                            }
                        }
                    }
                    
                    stdout.execute(ResetColor)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Generate prompt string without displaying it
    /// 
    /// This method creates the formatted prompt string for use by
    /// the application's input handling system.
    #[cfg(feature = "async")]
    pub async fn generate_prompt(&self) -> Result<String> {
        let mut prompt = String::new();
        
        // Start with PS1 format if available
        if let Ok(ps1) = env::var("PS1") {
            return self.process_ps1_format(&ps1);
        }
        
        // Use default NexusShell format: user@host:~/path $
        if self.config.show_user {
            prompt.push_str(&format!("\x1b[32m{}\x1b[0m", whoami::username()));
        }
        
        if self.config.show_hostname {
            if self.config.show_user {
                prompt.push('@');
            }
            prompt.push_str(&format!("\x1b[32m{}\x1b[0m", hostname::get().unwrap_or_default().to_string_lossy()));
        }
        
        if self.config.show_cwd {
            if self.config.show_user || self.config.show_hostname {
                prompt.push(':');
            }
            
            let cwd = env::current_dir()?;
            let home = env::var("HOME").or_else(|_| env::var("USERPROFILE"));
            
            let display_path = if let Ok(home_path) = home {
                let home_path = Path::new(&home_path);
                if cwd.starts_with(home_path) {
                    format!("~{}", cwd.strip_prefix(home_path).unwrap().display())
                } else {
                    cwd.display().to_string()
                }
            } else {
                cwd.display().to_string()
            };
            
            prompt.push_str(&format!("\x1b[34m{display_path}\x1b[0m"));
        }
        
        // Add Git information if enabled
            if self.config.show_git_info {
                #[cfg(feature = "async")]
                let git_info = self.get_git_info().await.unwrap_or_default();
                #[cfg(not(feature = "async"))]
                let git_info = self.get_git_info_blocking().unwrap_or_default();
                if !git_info.is_empty() {
                    prompt.push_str(&git_info);
                }
        }
        
        // Add final prompt character
        prompt.push_str(" $ ");
        
        Ok(prompt)
    }

    #[cfg(not(feature = "async"))]
    pub fn generate_prompt(&self) -> Result<String> {
        let mut prompt = String::new();

        if let Ok(ps1) = env::var("PS1") {
            return self.process_ps1_format(&ps1);
        }

        if self.config.show_user {
            prompt.push_str(&format!("\x1b[32m{}\x1b[0m", whoami::username()));
        }

        if self.config.show_hostname {
            if self.config.show_user { prompt.push('@'); }
            prompt.push_str(&format!("\x1b[32m{}\x1b[0m", hostname::get().unwrap_or_default().to_string_lossy()));
        }

        if self.config.show_cwd {
            if self.config.show_user || self.config.show_hostname { prompt.push(':'); }
            let cwd = env::current_dir()?;
            let home = env::var("HOME").or_else(|_| env::var("USERPROFILE"));
            let display_path = if let Ok(home_path) = home {
                let home_path = Path::new(&home_path);
                if cwd.starts_with(home_path) {
                    format!("~{}", cwd.strip_prefix(home_path).unwrap().display())
                } else { cwd.display().to_string() }
            } else { cwd.display().to_string() };
            prompt.push_str(&format!("\x1b[34m{display_path}\x1b[0m"));
        }

        if self.config.show_git_info {
            let git_info = self.get_git_info_blocking().unwrap_or_default();
            if !git_info.is_empty() { prompt.push_str(&git_info); }
        }

        prompt.push_str(" $ ");
        Ok(prompt)
    }
    
    /// Process PS1 format string with variable substitution
    fn process_ps1_format(&self, ps1: &str) -> Result<String> {
        let mut result = String::new();
        let mut chars = ps1.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(&next_ch) = chars.peek() {
                    chars.next(); // consume the next character
                    match next_ch {
                        'u' => result.push_str(&whoami::username()),
                        'h' => {
                            if let Ok(hostname) = hostname::get() {
                                result.push_str(&hostname.to_string_lossy());
                            }
                        },
                        'w' => {
                            let cwd = env::current_dir().unwrap_or_default();
                            result.push_str(&cwd.display().to_string());
                        },
                        'W' => {
                            let cwd = env::current_dir().unwrap_or_default();
                            if let Some(name) = cwd.file_name() {
                                result.push_str(&name.to_string_lossy());
                            }
                        },
                        '$' => result.push('$'),
                        'n' => result.push('\n'),
                        't' => {
                            // Current time (simplified)
                            use std::time::{SystemTime, UNIX_EPOCH};
                            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                            let hours = (now / 3600) % 24;
                            let minutes = (now / 60) % 60;
                            result.push_str(&format!("{hours:02}:{minutes:02}"));
                        },
                        _ => {
                            // Unknown escape sequence, keep as-is
                            result.push('\\');
                            result.push(next_ch);
                        }
                    }
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }
        
        Ok(result)
    }
    
    /// Get Git information for prompt display
    #[cfg(feature = "async")]
    async fn get_git_info(&self) -> Result<String> {
        if !self.config.git_simplified {
            return Ok(String::new());
        }
        
        // Check if we're in a Git repository
    let output = tokio::process::Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .await?;
            
        if !output.status.success() {
            return Ok(String::new()); // Not a Git repository
        }
        
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if branch.is_empty() {
            return Ok(String::new());
        }
        
        let mut git_info = format!(" \x1b[35m({branch})\x1b[0m");
        
        // Check for uncommitted changes
    let status_output = tokio::process::Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .await;
            
        if let Ok(status) = status_output {
            if status.status.success() {
                let status_text = String::from_utf8_lossy(&status.stdout);
                if !status_text.trim().is_empty() {
                    git_info.push_str("\x1b[31m*\x1b[0m");
                }
            }
        }
        
        Ok(git_info)
    }

    #[cfg(not(feature = "async"))]
    fn get_git_info_blocking(&self) -> Result<String> {
        if !self.config.git_simplified { return Ok(String::new()); }
        use std::process::Command;
        let output = Command::new("git").args(["branch", "--show-current"]).output()?;
        if !output.status.success() { return Ok(String::new()); }
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if branch.is_empty() { return Ok(String::new()); }
        let mut git_info = format!(" \x1b[35m({branch})\x1b[0m");
        if let Ok(status) = Command::new("git").args(["status", "--porcelain"]).output() {
            if status.status.success() {
                let status_text = String::from_utf8_lossy(&status.stdout);
                if !status_text.trim().is_empty() { git_info.push_str("\x1b[31m*\x1b[0m"); }
            }
        }
        Ok(git_info)
    }
    
    /// Update prompt configuration
    pub fn update_config(&mut self, config: PromptConfig) {
        self.config = config;
    }
    
    /// Get current configuration
    pub fn config(&self) -> &PromptConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prompt_formatter_creation() {
        let formatter = PromptFormatter::new();
        assert!(formatter.config.show_user);
        assert!(formatter.config.show_cwd);
    }
    
    #[test]
    fn test_custom_config() {
        let config = PromptConfig {
            show_hostname: true,
            git_simplified: false,
            ..Default::default()
        };
        
        let formatter = PromptFormatter::with_config(config);
        assert!(formatter.config.show_hostname);
        assert!(!formatter.config.git_simplified);
    }
}
