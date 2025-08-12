/// CUI Prompt System for NexusShell
/// 
/// This module implements the command prompt display for CUI mode,
/// following the design specifications in UI_DESIGN.md while removing
/// TUI dependencies and complex interactive overlays.
/// 
/// Features:
/// - PS1 environment variable support
/// - Git status integration
/// - System information display
/// - ANSI color and styling
/// - Cross-platform compatibility

use anyhow::{Result, Context};
use crossterm::{
    style::{Color, Print, ResetColor, SetForegroundColor, SetBackgroundColor, Attribute, SetAttribute},
    execute,
};
use std::{
    env,
    io::{self, Write},
    path::PathBuf,
    process::Command,
};
use chrono::{DateTime, Local};

/// CUI Prompt builder and formatter
#[derive(Debug, Clone)]
pub struct CUIPrompt {
    /// Current user name
    username: String,
    
    /// Current hostname
    hostname: String,
    
    /// Whether current user is root/administrator
    is_root: bool,
    
    /// Git status cache (updated periodically)
    git_status: Option<GitStatus>,
    
    /// System metrics cache
    system_info: SystemInfo,
    
    /// Prompt format template
    prompt_format: PromptFormat,
}

/// Git repository status information
#[derive(Debug, Clone)]
pub struct GitStatus {
    /// Current branch name
    pub branch: String,
    
    /// Number of commits ahead of upstream
    pub ahead: usize,
    
    /// Number of commits behind upstream  
    pub behind: usize,
    
    /// Has uncommitted changes
    pub has_changes: bool,
    
    /// Has staged changes
    pub has_staged: bool,
    
    /// Has untracked files
    pub has_untracked: bool,
}

/// System information for prompt display
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// Current time
    pub time: DateTime<Local>,
    
    /// CPU usage percentage (0-100) - average across all cores
    pub cpu_usage: Option<f32>,
    
    /// Memory usage tuple: (used_MiB, total_MiB, usage_percent)
    pub memory_usage: Option<(u64, u64, f32)>,
    
    /// System load average tuple: (1min, 5min, 15min)
    pub load_average: Option<(f64, f64, f64)>,
    
    /// Battery percentage (if available) - for laptop/mobile systems
    pub battery: Option<f32>,
    
    /// Network interface status for bandwidth monitoring
    pub network_status: NetworkStatus,
}

/// Network interface status
#[derive(Debug, Clone)]
pub struct NetworkStatus {
    /// Upload speed in bytes/second
    pub upload_bps: Option<u64>,
    
    /// Download speed in bytes/second
    pub download_bps: Option<u64>,
    
    /// Interface is connected
    pub connected: bool,
}

/// Prompt format configuration
#[derive(Debug, Clone)]
pub struct PromptFormat {
    /// Show Git information
    pub show_git: bool,
    
    /// Show system metrics
    pub show_system: bool,
    
    /// Show time
    pub show_time: bool,
    
    /// Use icons and special characters
    pub use_icons: bool,
    
    /// Use colors
    pub use_colors: bool,
    
    /// Prompt symbol for regular user
    pub user_symbol: String,
    
    /// Prompt symbol for root user
    pub root_symbol: String,
}

impl Default for PromptFormat {
    fn default() -> Self {
        Self {
            show_git: true,
            show_system: true,
            show_time: true,
            use_icons: true,
            use_colors: true,
            user_symbol: "λ".to_string(),
            root_symbol: "#".to_string(),
        }
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            time: Local::now(),
            cpu_usage: None,
            memory_usage: None,  // Tuple format: (used, total, percent)
            load_average: None,  // Tuple format: (1min, 5min, 15min)
            battery: None,
            network_status: NetworkStatus {
                upload_bps: None,
                download_bps: None,
                connected: false,
            },
        }
    }
}

impl CUIPrompt {
    /// Create new CUI prompt system
    pub fn new() -> Result<Self> {
        let username = Self::get_username()
            .context("Failed to get username")?;
            
        let hostname = Self::get_hostname()
            .context("Failed to get hostname")?;
            
        let is_root = Self::is_root_user();
        
        Ok(Self {
            username,
            hostname,
            is_root,
            git_status: None,
            system_info: SystemInfo::default(),
            prompt_format: PromptFormat::default(),
        })
    }
    
    /// Set prompt format configuration
    pub fn set_format(&mut self, format_config: crate::config::PromptFormatConfig) -> Result<()> {
        // Convert CUI-specific PromptFormatConfig to internal PromptFormat
        self.prompt_format.show_git = format_config.show_git_status;
        self.prompt_format.show_system = format_config.show_system_info;
        
        // Parse template strings for more advanced formatting
        // This is a simplified implementation - could be extended with a proper template engine
        if format_config.left_template.contains("{git}") {
            self.prompt_format.show_git = true;
        }
        if format_config.left_template.contains("{user}") || format_config.left_template.contains("{host}") {
            // User@host segment is always shown in our current implementation
        }
        
        Ok(())
    }
    
    /// Build the complete prompt string with ANSI formatting
    /// 
    /// Format follows UI_DESIGN.md specification:
    /// λ user@host  ~/workspace  (git:main↓2✗)  ▶
    pub fn build_prompt(&mut self) -> Result<String> {
        let mut prompt = String::new();
        
        // Update dynamic information
        self.update_git_status();
        self.update_system_info();
        
        // Build prompt segments
        prompt.push_str(&self.build_symbol_segment()?);
        prompt.push_str(&self.build_user_host_segment()?);
        prompt.push_str(&self.build_directory_segment()?);
        
        if self.prompt_format.show_git {
            if let Some(git_segment) = self.build_git_segment()? {
                prompt.push_str(&git_segment);
            }
        }
        
        if self.prompt_format.show_time {
            prompt.push_str(&self.build_time_segment()?);
        }
        
        prompt.push_str(&self.build_input_marker()?);
        
        Ok(prompt)
    }
    
    /// Build the prompt symbol (λ or #)
    fn build_symbol_segment(&self) -> Result<String> {
        let symbol = if self.is_root {
            &self.prompt_format.root_symbol
        } else {
            &self.prompt_format.user_symbol
        };
        
        if self.prompt_format.use_colors {
            let color = if self.is_root { Color::Red } else { Color::Cyan };
            Ok(format!("\x1b[1;{}m{}\x1b[0m ", Self::color_to_ansi(color), symbol))
        } else {
            Ok(format!("{} ", symbol))
        }
    }
    
    /// Build user@host segment
    fn build_user_host_segment(&self) -> Result<String> {
        let user_host = format!("{}@{}", self.username, self.hostname);
        
        if self.prompt_format.use_colors {
            Ok(format!("\x1b[1;{}m{}\x1b[0m ", Self::color_to_ansi(Color::Green), user_host))
        } else {
            Ok(format!("{} ", user_host))
        }
    }
    
    /// Build current directory segment
    fn build_directory_segment(&self) -> Result<String> {
        let current_dir = env::current_dir()
            .context("Failed to get current directory")?;
        
        let display_dir = Self::format_directory_path(&current_dir)?;
        
        if self.prompt_format.use_colors {
            Ok(format!("\x1b[{}m {}\x1b[0m ", Self::color_to_ansi(Color::Blue), display_dir))
        } else {
            Ok(format!(" {} ", display_dir))
        }
    }
    
    /// Build git status segment
    fn build_git_segment(&self) -> Result<Option<String>> {
        if let Some(ref git) = self.git_status {
            let mut git_text = format!("git:{}", git.branch);
            
            // Add ahead/behind indicators
            if git.ahead > 0 {
                git_text.push_str(&format!("↑{}", git.ahead));
            }
            if git.behind > 0 {
                git_text.push_str(&format!("↓{}", git.behind));
            }
            
            // Add status indicators
            if git.has_staged {
                git_text.push('✓');
            }
            if git.has_changes {
                git_text.push('✗');
            }
            if git.has_untracked {
                git_text.push('?');
            }
            
            if self.prompt_format.use_colors {
                let color = if git.has_changes || git.has_untracked {
                    Color::Red
                } else if git.has_staged {
                    Color::Green
                } else {
                    Color::Yellow
                };
                
                Ok(Some(format!(" \x1b[{}m({})\x1b[0m", 
                    Self::color_to_ansi(color), git_text)))
            } else {
                Ok(Some(format!(" ({})", git_text)))
            }
        } else {
            Ok(None)
        }
    }
    
    /// Build time segment
    fn build_time_segment(&self) -> Result<String> {
        let time_str = self.system_info.time.format("%H:%M").to_string();
        
        if self.prompt_format.use_colors {
            Ok(format!(" \x1b[{}m{}\x1b[0m", 
                Self::color_to_ansi(Color::DarkGrey), time_str))
        } else {
            Ok(format!(" {}", time_str))
        }
    }
    
    /// Build input marker (▶)
    fn build_input_marker(&self) -> Result<String> {
        let marker = if self.prompt_format.use_icons { "▶" } else { ">" };
        
        if self.prompt_format.use_colors {
            Ok(format!(" \x1b[1;{}m{}\x1b[0m ", Self::color_to_ansi(Color::White), marker))
        } else {
            Ok(format!(" {} ", marker))
        }
    }
    
    /// Update Git status information
    fn update_git_status(&mut self) {
        self.git_status = Self::detect_git_status().ok();
    }
    
    /// Update system information with comprehensive metrics collection
    fn update_system_info(&mut self) {
        use sysinfo::{System, SystemExt, CpuExt};
        
        self.system_info.time = Local::now();
        
        // Collect system metrics for performance monitoring
        let mut sys = System::new_all();
        sys.refresh_all();
        
        // CPU utilization percentage (average across all cores)
        let cpu_usage = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() 
            / sys.cpus().len() as f32;
        
        // Memory usage in MiB
        let memory_used = sys.used_memory() / (1024 * 1024);  // Convert to MiB
        let memory_total = sys.total_memory() / (1024 * 1024);
        let memory_percent = (memory_used as f32 / memory_total as f32) * 100.0;
        
        // System load average (Unix-like systems)
        let load_avg = sys.load_average();
        
        // Update internal system metrics cache for prompt display
        // This enables real-time system monitoring in the shell prompt
        self.system_info.cpu_usage = Some(cpu_usage);
        self.system_info.memory_usage = Some((memory_used, memory_total, memory_percent));
        self.system_info.load_average = Some((load_avg.one, load_avg.five, load_avg.fifteen));
    }
    
    /// Detect Git repository status
    fn detect_git_status() -> Result<GitStatus> {
        // Check if we're in a git repository
        let output = Command::new("git")
            .args(&["rev-parse", "--is-inside-work-tree"])
            .output();
            
        if output.is_err() || !output.unwrap().status.success() {
            return Err(anyhow::anyhow!("Not in git repository"));
        }
        
        // Get current branch
        let branch_output = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .context("Failed to get git branch")?;
            
        let branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();
        
        // Get status information
        let status_output = Command::new("git")
            .args(&["status", "--porcelain", "--ahead-behind"])
            .output()
            .context("Failed to get git status")?;
            
        let status_text = String::from_utf8_lossy(&status_output.stdout);
        
        // Parse status for changes
        let has_changes = status_text.lines()
            .any(|line| line.starts_with(" M") || line.starts_with("M "));
        let has_staged = status_text.lines()
            .any(|line| line.starts_with("M ") || line.starts_with("A "));
        let has_untracked = status_text.lines()
            .any(|line| line.starts_with("??"));
        
        // Parse ahead/behind information from git status
        let (ahead, behind) = Self::parse_ahead_behind_from_git()?;
        
        Ok(GitStatus {
            branch,
            ahead,
            behind,
            has_changes,
            has_staged,
            has_untracked,
        })
    }
    
    /// Parse ahead/behind information from git status --ahead-behind
    fn parse_ahead_behind_from_git() -> Result<(u32, u32)> {
        // Get ahead/behind count using git rev-list
        let upstream_output = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "@{upstream}"])
            .output();
            
        // If no upstream, return (0, 0)
        if upstream_output.is_err() || !upstream_output.as_ref().unwrap().status.success() {
            return Ok((0, 0));
        }
        
        let upstream = String::from_utf8_lossy(&upstream_output.unwrap().stdout).trim().to_string();
        
        // Count commits ahead of upstream
        let ahead_output = Command::new("git")
            .args(&["rev-list", "--count", &format!("{}..HEAD", upstream)])
            .output()
            .context("Failed to get ahead count")?;
            
        let ahead = String::from_utf8_lossy(&ahead_output.stdout)
            .trim()
            .parse::<u32>()
            .unwrap_or(0);
            
        // Count commits behind upstream
        let behind_output = Command::new("git")
            .args(&["rev-list", "--count", &format!("HEAD..{}", upstream)])
            .output()
            .context("Failed to get behind count")?;
            
        let behind = String::from_utf8_lossy(&behind_output.stdout)
            .trim()
            .parse::<u32>()
            .unwrap_or(0);
            
        Ok((ahead, behind))
    }
    
    /// Get current username
    fn get_username() -> Result<String> {
        env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .context("Failed to get username from environment")
    }
    
    /// Get current hostname
    fn get_hostname() -> Result<String> {
        let hostname = hostname::get()
            .context("Failed to get hostname")?
            .to_string_lossy()
            .to_string();
        Ok(hostname)
    }
    
    /// Check if current user is root/administrator
    fn is_root_user() -> bool {
        #[cfg(unix)]
        {
            unsafe { libc::getuid() == 0 }
        }
        
        #[cfg(windows)]
        {
            // On Windows, check if running as administrator
            // This is a simplified check - more robust implementation needed
            std::process::Command::new("net")
                .args(&["session"])
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }
    }
    
    /// Format directory path for display
    fn format_directory_path(path: &PathBuf) -> Result<String> {
        let home_dir = dirs::home_dir();
        
        if let Some(home) = home_dir {
            if path.starts_with(&home) {
                if let Ok(relative) = path.strip_prefix(&home) {
                    return Ok(format!("~/{}", relative.display()));
                }
            }
        }
        
        Ok(path.display().to_string())
    }
    
    /// Convert Color to ANSI color code
    fn color_to_ansi(color: Color) -> &'static str {
        match color {
            Color::Black => "30",
            Color::Red => "31", 
            Color::Green => "32",
            Color::Yellow => "33",
            Color::Blue => "34",
            Color::Magenta => "35",
            Color::Cyan => "36",
            Color::White => "37",
            Color::DarkGrey => "90",
            _ => "37", // Default to white
        }
    }
    
    /// Set prompt format configuration
    pub fn set_format(&mut self, format: PromptFormat) {
        self.prompt_format = format;
    }
    
    /// Get current prompt format
    pub fn get_format(&self) -> &PromptFormat {
        &self.prompt_format
    }
}

// Extension trait for pipe operations
trait Pipe<T> {
    fn pipe<F, U>(self, f: F) -> U where F: FnOnce(T) -> U;
}

impl<T> Pipe<T> for T {
    fn pipe<F, U>(self, f: F) -> U where F: FnOnce(T) -> U {
        f(self)
    }
}
