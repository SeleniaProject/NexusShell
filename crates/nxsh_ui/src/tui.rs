//! TUI module disabled for CUI mode
//! This module is kept for compatibility but all functions are stubs

use nxsh_core::{ShellContext, ShellResult, executor::Executor, ErrorKind, error::RuntimeErrorKind};
use crate::app::App;
use crossterm::execute;

/// Disabled TUI run function - returns error
pub async fn run(context: &mut ShellContext, executor: &mut Executor) -> ShellResult<()> {
    Err(nxsh_core::ShellError::new(
        ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound),
        "TUI mode disabled - use CUI mode instead"
    ))
}

/// Disabled TUI event handler
pub async fn handle_events(app: &mut App, _ctx: &mut ShellContext) -> ShellResult<bool> {
    Ok(false)
}

/// Disabled TUI draw function
pub fn draw_ui(frame: &mut DummyFrame, app: &mut App) -> Result<(), std::io::Error> {
    Ok(())
}

/// Dummy frame type for compatibility
pub struct DummyFrame;

/// Disabled TUI startup function
pub async fn tui_startup() -> ShellResult<()> {
    Err(nxsh_core::ShellError::new(
        ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound),
        "TUI startup disabled - use CUI mode instead"
    ))
}

/// Initialize TUI system (disabled)
pub fn init() -> ShellResult<()> {
    Ok(())
}

/// Cleanup TUI system (disabled)
pub fn cleanup() -> ShellResult<()> {
    Ok(())
}

/// Get terminal size for layout calculations.
pub fn get_terminal_size() -> ShellResult<(u16, u16)> {
    let (width, height) = crossterm::terminal::size()
        .map_err(|e| nxsh_core::ShellError::io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    Ok((width, height))
}

/// Get current terminal dimensions.
pub fn terminal_size() -> Result<(u16, u16), std::io::Error> {
    crossterm::terminal::size()
}

/// Check if terminal supports colors.
pub fn supports_color() -> bool {
    // Check for TTY blind mode environment variable first
    if std::env::var("NXSH_TTY_NOCOLOR").is_ok() {
        return false;
    }
    
    // Check standard NO_COLOR environment variable
    if std::env::var("NO_COLOR").is_ok() && !std::env::var("NO_COLOR").unwrap_or_default().is_empty() {
        return false;
    }
    
    true // Most modern terminals support color
}

/// Clear the terminal screen.
pub fn clear_screen() -> ShellResult<()> {
    use std::io::Write;
    execute!(std::io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))
        .map_err(|e| nxsh_core::ShellError::io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    Ok(())
}

/// Move cursor to position.
pub fn move_cursor(x: u16, y: u16) -> ShellResult<()> {
    use std::io::Write;
    execute!(std::io::stdout(), crossterm::cursor::MoveTo(x, y))
        .map_err(|e| nxsh_core::ShellError::io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    Ok(())
}
