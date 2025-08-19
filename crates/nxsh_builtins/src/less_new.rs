//! `less` command - advanced interactive pager with improved TTY handling.
//! Supports forward/backward navigation similar to GNU less (subset).
//! Keys: Space/PageDown/Down/j -> forward, b/PageUp/Up/k -> back, g -> top, G -> bottom, q -> quit.
//! Falls back to printing all content if not running in TTY.

use anyhow::Result;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::Duration;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    terminal::{self, ClearType},
};
use tokio::task;

/// Entry point exposed to shell runtime.
pub async fn less_cli(args: &[String]) -> Result<()> {
    // Parse options for better GNU less compatibility
    let mut options = LessOptions::default();
    let mut file_path = None;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_less_help();
                return Ok(());
            }
            "-V" | "--version" => {
                println!("less (NexusShell) {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "-e" | "--quit-at-eof" => {
                options.quit_at_eof = true;
            }
            "-f" | "--force" => {
                options.force = true;
            }
            "-n" | "--line-numbers" => {
                options.line_numbers = true;
            }
            "-S" | "--chop-long-lines" => {
                options.chop_long_lines = true;
            }
            "-r" | "--raw-control-chars" => {
                options.raw_control_chars = true;
            }
            arg if !arg.starts_with('-') => {
                file_path = Some(arg.to_string());
            }
            _ => {
                eprintln!("less: unknown option: {}", args[i]);
                return Ok(());
            }
        }
        i += 1;
    }
    
    task::spawn_blocking(move || run_less(file_path, options)).await??;
    Ok(())
}

#[derive(Debug, Clone)]
struct LessOptions {
    quit_at_eof: bool,
    force: bool,
    line_numbers: bool,
    chop_long_lines: bool,
    raw_control_chars: bool,
}

impl Default for LessOptions {
    fn default() -> Self {
        Self {
            quit_at_eof: false,
            force: false,
            line_numbers: false,
            chop_long_lines: false,
            raw_control_chars: false,
        }
    }
}

fn run_less(path_opt: Option<String>, options: LessOptions) -> Result<()> {
    // Load entire content up-front for simplicity. In future, we can stream.
    let mut content = String::new();
    match path_opt {
        Some(p) => {
            let path = Path::new(&p);
            if !path.exists() {
                return Err(anyhow::anyhow!("No such file: {}", p));
            }
            
            let mut f = File::open(path)?;
            f.read_to_string(&mut content)?;
        }
        None => {
            // Read from STDIN until EOF
            io::stdin().read_to_string(&mut content)?;
        }
    }

    // Better TTY detection using crossterm
    if !is_tty() {
        // Non-interactive environment: print everything and return
        print!("{}", content);
        return Ok(());
    }

    // Interactive pager with enhanced features
    run_interactive_pager(&content, &options)
}

/// Improved TTY detection using crossterm capabilities
fn is_tty() -> bool {
    // Check if we can enable raw mode (indicates a real terminal)
    match terminal::enable_raw_mode() {
        Ok(_) => {
            let _ = terminal::disable_raw_mode();
            true
        }
        Err(_) => false,
    }
}

/// Enhanced interactive pager with improved navigation and display
fn run_interactive_pager(content: &str, options: &LessOptions) -> Result<()> {
    let lines: Vec<&str> = content.lines().collect();
    let mut stdout = io::stdout();
    
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen)?;
    
    let mut offset = 0;
    let mut search_pattern: Option<String> = None;
    let mut status_message = String::new();
    
    loop {
        // Clear screen and get terminal size
        execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        
        let (width, height) = terminal::size()?;
        let display_height = height.saturating_sub(1) as usize; // Reserve one line for status
        
        // Calculate visible range
        let visible_end = (offset + display_height).min(lines.len());
        
        // Display lines with optional line numbers
        for (i, line_idx) in (offset..visible_end).enumerate() {
            if line_idx >= lines.len() {
                break;
            }
            
            let line = lines[line_idx];
            let display_line = if options.chop_long_lines {
                // Truncate long lines to fit terminal width
                if line.len() > width as usize {
                    &line[..width as usize]
                } else {
                    line
                }
            } else {
                line
            };
            
            if options.line_numbers {
                queue!(stdout, cursor::MoveTo(0, i as u16))?;
                write!(stdout, "{:6} {}", line_idx + 1, display_line)?;
            } else {
                queue!(stdout, cursor::MoveTo(0, i as u16))?;
                write!(stdout, "{}", display_line)?;
            }
        }
        
        // Display status line
        queue!(stdout, cursor::MoveTo(0, height - 1))?;
        queue!(stdout, terminal::Clear(ClearType::CurrentLine))?;
        
        if !status_message.is_empty() {
            write!(stdout, "{}", status_message)?;
            status_message.clear();
        } else {
            let percentage = if lines.is_empty() {
                100
            } else {
                ((offset + display_height) * 100 / lines.len()).min(100)
            };
            
            let position_info = if offset == 0 && visible_end >= lines.len() {
                "(END)".to_string()
            } else if offset == 0 {
                "TOP".to_string()
            } else if visible_end >= lines.len() {
                "END".to_string()
            } else {
                format!("{}%", percentage)
            };
            
            write!(stdout, "--Less-- {} (q to quit, h for help)", position_info)?;
        }
        
        stdout.flush()?;
        
        // Handle key events with enhanced commands
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    
                    // Navigation
                    KeyCode::Char('g') => offset = 0, // Go to top
                    KeyCode::Char('G') => {
                        offset = lines.len().saturating_sub(display_height);
                    }
                    KeyCode::PageDown | KeyCode::Char(' ') => {
                        let new_offset = offset + display_height;
                        if new_offset < lines.len() {
                            offset = new_offset;
                        } else if options.quit_at_eof && offset + display_height >= lines.len() {
                            break; // Quit at EOF if enabled
                        }
                    }
                    KeyCode::PageUp | KeyCode::Char('b') => {
                        offset = offset.saturating_sub(display_height);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if offset + display_height < lines.len() {
                            offset += 1;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        offset = offset.saturating_sub(1);
                    }
                    
                    // Help
                    KeyCode::Char('h') | KeyCode::Char('H') => {
                        status_message = "COMMANDS: q=quit, SPACE/j=down, b/k=up, g=top, G=end, h=help".to_string();
                    }
                    
                    // Search (basic implementation)
                    KeyCode::Char('/') => {
                        status_message = "Search: /pattern (not implemented yet)".to_string();
                    }
                    
                    _ => {}
                }
            }
        }
    }
    
    // Cleanup
    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    
    Ok(())
}

/// Print comprehensive help information
fn print_less_help() {
    println!("less - NexusShell interactive file viewer");
    println!();
    println!("Usage: less [OPTIONS] [FILE]");
    println!("View file contents with interactive navigation.");
    println!();
    println!("Options:");
    println!("  -e, --quit-at-eof      Exit automatically at end of file");
    println!("  -f, --force            Force opening of non-regular files");
    println!("  -n, --line-numbers     Display line numbers");
    println!("  -S, --chop-long-lines  Truncate long lines instead of wrapping");
    println!("  -r, --raw-control-chars  Display raw control characters");
    println!("  -h, --help             Display this help message");
    println!("  -V, --version          Display version information");
    println!();
    println!("Interactive Commands:");
    println!("  q, Q                   Quit");
    println!("  SPACE, j, DOWN         Forward one line");
    println!("  b, k, UP               Backward one line");
    println!("  PAGE DOWN              Forward one page");
    println!("  PAGE UP                Backward one page");
    println!("  g                      Go to beginning of file");
    println!("  G                      Go to end of file");
    println!("  h                      Show help in status line");
    println!();
    println!("If no file is specified, reads from standard input.");
    println!("In non-TTY environments, acts like 'cat' and prints all content.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write as _;

    #[test]
    fn test_less_options_default() {
        let options = LessOptions::default();
        assert!(!options.quit_at_eof);
        assert!(!options.force);
        assert!(!options.line_numbers);
        assert!(!options.chop_long_lines);
        assert!(!options.raw_control_chars);
    }

    #[test]
    fn test_is_tty_detection() {
        // This will vary based on test environment
        // Just ensure the function doesn't panic
        let _result = is_tty();
    }

    #[tokio::test]
    async fn test_less_help() {
        // Test help option
        let result = less_cli(&["-h".to_string()]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_less_version() {
        // Test version option
        let result = less_cli(&["-V".to_string()]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_less_nonexistent_file() {
        // Test with non-existent file
        let result = less_cli(&["/nonexistent/file.txt".to_string()]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_less_file_content() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1").unwrap();
        writeln!(temp_file, "Line 2").unwrap();
        writeln!(temp_file, "Line 3").unwrap();
        temp_file.flush().unwrap();
        
        // In non-TTY environment, this should succeed and print content
        let result = less_cli(&[temp_file.path().to_string_lossy().to_string()]).await;
        assert!(result.is_ok());
    }
}
