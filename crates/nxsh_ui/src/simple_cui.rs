//! Ultra-Simple CUI Implementation - Phase 1 Emergency Response
//! 
//! This is a MINIMAL Character User Interface implementation that focuses
//! on basic functionality only:
//! - Basic prompt display
//! - Command input
//! - Simple output formatting
//! - Absolute minimum dependencies
//!
//! Target: Startup ≤ 5ms, Memory ≤ 15MiB

use std::io::{self, Write};
use crossterm::{
    style::{Color, SetForegroundColor, ResetColor, Print},
    execute,
    terminal::{Clear, ClearType},
    cursor,
};

/// Ultra-minimal CUI application for emergency Phase 1 deployment
#[derive(Debug)]
pub struct SimpleCUI {
    /// Current working directory (minimal caching)
    pub current_dir: String,
    /// Last command exit status for prompt indication
    pub last_exit_code: i32,
    /// Running state
    pub running: bool,
}

impl Default for SimpleCUI {
    fn default() -> Self {
        Self {
            current_dir: std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "/".to_string()),
            last_exit_code: 0,
            running: true,
        }
    }
}

impl SimpleCUI {
    /// Create new minimal CUI instance with ≤1ms initialization
    pub fn new() -> io::Result<Self> {
        Ok(Self::default())
    }

    /// Display basic prompt with minimal formatting
    /// Format: user@host:path $ (success) or user@host:path ! (error)
    pub fn display_prompt(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        
        // Get basic user/host info (whoami::hostname が deprecated のため環境変数 + gethostname fallback)
        let username = whoami::username();
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .or_else(|_| {
                // POSIX gethostname fallback (best-effort). 失敗したら "unknown"。
                #[cfg(any(unix, target_os = "linux", target_os = "macos"))]
                {
                    use std::ffi::CStr;
                    let mut buf = [0u8; 256];
                    unsafe {
                        if libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) == 0 {
                            if let Ok(cstr) = CStr::from_bytes_until_nul(&buf) {
                                if let Ok(s) = cstr.to_str() { return Ok(s.to_string()); }
                            }
                        }
                    }
                }
                Ok("unknown".to_string())
            })
            .unwrap_or_else(|_: std::env::VarError| "unknown".to_string());
        
        // Prompt color based on last command status
        let prompt_symbol = if self.last_exit_code == 0 {
            "$"  // Success
        } else {
            "!"  // Error
        };
        
        let prompt_color = if self.last_exit_code == 0 {
            Color::Green
        } else {
            Color::Red
        };

        // Simple prompt format: user@host:path $ 
        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print(&username),
            ResetColor,
            Print("@"),
            SetForegroundColor(Color::Cyan),
            Print(&hostname),
            ResetColor,
            Print(":"),
            SetForegroundColor(Color::Blue),
            Print(&self.current_dir),
            ResetColor,
            Print(" "),
            SetForegroundColor(prompt_color),
            Print(prompt_symbol),
            ResetColor,
            Print(" ")
        )?;

        stdout.flush()?;
        Ok(())
    }

    /// Read command input with basic line editing
    pub fn read_command(&self) -> io::Result<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    /// Update current directory cache
    pub fn update_current_dir(&mut self) {
        self.current_dir = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| self.current_dir.clone());
    }

    /// Set last command exit code for prompt indication
    pub fn set_last_exit_code(&mut self, exit_code: i32) {
        self.last_exit_code = exit_code;
    }

    /// Display simple error message
    pub fn display_error(&self, message: &str) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            SetForegroundColor(Color::Red),
            Print("Error: "),
            ResetColor,
            Print(message),
            Print("\n")
        )?;
        stdout.flush()?;
        Ok(())
    }

    /// Display simple info message
    pub fn display_info(&self, message: &str) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("Info: "),
            ResetColor,
            Print(message),
            Print("\n")
        )?;
        stdout.flush()?;
        Ok(())
    }

    /// Display simple warning message
    pub fn display_warning(&self, message: &str) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("Warning: "),
            ResetColor,
            Print(message),
            Print("\n")
        )?;
        stdout.flush()?;
        Ok(())
    }

    /// Clear screen (basic terminal control)
    pub fn clear_screen(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        stdout.flush()?;
        Ok(())
    }

    /// Main CUI loop - simplified for Phase 1
    pub fn run_loop(&mut self) -> io::Result<()> {
        println!("NexusShell v0.1.0 - Phase 1 Emergency CUI");
        println!("Type 'exit' to quit, 'help' for commands");

        while self.running {
            // Update current directory
            self.update_current_dir();
            
            // Display prompt
            self.display_prompt()?;
            
            // Read command
            let command = self.read_command()?;
            
            // Handle basic built-in commands
            match command.as_str().trim() {
                "exit" | "quit" => {
                    self.running = false;
                    println!("Goodbye!");
                }
                "clear" => {
                    self.clear_screen()?;
                }
                "pwd" => {
                    println!("{}", self.current_dir);
                }
                "help" => {
                    self.display_help();
                }
                "" => {
                    // Empty command - just show prompt again
                    continue;
                }
                _ => {
                    // For Phase 1, just echo the command
                    println!("Command received: {command}");
                    println!("Full command execution will be implemented in Phase 2");
                }
            }
        }

        Ok(())
    }

    /// Display basic help information
    fn display_help(&self) {
        println!("NexusShell Phase 1 - Basic Commands:");
        println!("  exit, quit  - Exit the shell");
        println!("  clear       - Clear screen");
        println!("  pwd         - Show current directory");
        println!("  help        - Show this help");
        println!();
        println!("Full command set will be available in Phase 2");
    }
}

/// Ultra-fast CUI entry point for Phase 1 emergency deployment
pub fn run_emergency_cui() -> io::Result<()> {
    let start_time = std::time::Instant::now();
    
    // Initialize minimal CUI
    let mut cui = SimpleCUI::new()?;
    
    let init_time = start_time.elapsed();
    println!("CUI initialized in {init_time:?}");
    
    // Run main loop
    cui.run_loop()
}
