use std::time::{Duration, Instant};
use std::io::{self, Write};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, Terminal};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};

use nxsh_core::{ShellContext, ShellResult, executor::Executor};
use crate::app::App;

/// Run TUI interactive shell. Blocks until user exits with Ctrl+D or :quit.
pub async fn run(context: &mut ShellContext, executor: &mut Executor) -> ShellResult<()> {
    enable_raw_mode().map_err(|e| nxsh_core::ShellError::io(e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| nxsh_core::ShellError::io(e))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| nxsh_core::ShellError::io(e))?;

    let mut app = App::new().await.map_err(|e| nxsh_core::ShellError::io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    let tick_rate: Duration = Duration::from_millis(250);
    let mut last_tick: Instant = Instant::now();

    loop {
        terminal.draw(|f| {
            if let Err(e) = app.render::<ratatui::backend::CrosstermBackend<std::io::Stdout>>(f) {
                eprintln!("Failed to render: {}", e);
            }
        }).map_err(|e| nxsh_core::ShellError::io(e))?;

        let timeout: Duration = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout).map_err(|e| nxsh_core::ShellError::io(e))? {
            if let Event::Key(key) = event::read().map_err(|e| nxsh_core::ShellError::io(e))? {
                app.handle_key(key).await.map_err(|e| nxsh_core::ShellError::io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                if app.should_quit() {
                    break;
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    disable_raw_mode().map_err(|e| nxsh_core::ShellError::io(e))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)
        .map_err(|e| nxsh_core::ShellError::io(e))?;
    terminal.show_cursor().ok();
    Ok(())
}

/// Setup terminal for TUI rendering. Returns a Terminal instance ready for rendering.
pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, std::io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore terminal state after TUI rendering.
pub fn restore_terminal() -> Result<(), std::io::Error> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

/// Get terminal size for layout calculations.
pub fn get_terminal_size() -> ShellResult<(u16, u16)> {
    let (width, height) = crossterm::terminal::size()
        .map_err(|e| nxsh_core::ShellError::io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    Ok((width, height))
}

/// Handle terminal events and return true if execution should continue.
pub async fn handle_events(app: &mut App, _ctx: &mut ShellContext) -> ShellResult<bool> {
    if event::poll(Duration::from_millis(100))
        .map_err(|e| nxsh_core::ShellError::io(std::io::Error::new(std::io::ErrorKind::Other, e)))? {
        if let Event::Key(key) = event::read()
            .map_err(|e| nxsh_core::ShellError::io(std::io::Error::new(std::io::ErrorKind::Other, e)))? {
            // Add your key handling logic here
            match key.code {
                crossterm::event::KeyCode::Esc => return Ok(false),
                crossterm::event::KeyCode::Char('q') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => return Ok(false),
                _ => {}
            }
        }
    }
    Ok(true)
}

/// Draw the TUI interface.
pub fn draw_ui(frame: &mut Frame, app: &mut App) -> Result<(), std::io::Error> {
    app.render::<ratatui::backend::CrosstermBackend<std::io::Stdout>>(frame).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

/// Initialize TUI system.
pub fn init() -> ShellResult<()> {
    Ok(())
}

/// Cleanup TUI system.
pub fn cleanup() -> ShellResult<()> {
    let _ = restore_terminal();
    Ok(())
}

/// Run TUI in full-screen mode with proper error handling.
pub async fn run_fullscreen(context: &mut ShellContext, executor: &mut Executor) -> ShellResult<()> {
    let _terminal = setup_terminal()
        .map_err(|e| nxsh_core::ShellError::io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    let result = run(context, executor).await;
    
    let _ = restore_terminal();
    
    result
}

/// Get current terminal dimensions.
pub fn terminal_size() -> Result<(u16, u16), std::io::Error> {
    crossterm::terminal::size()
}

/// Check if terminal supports colors.
pub fn supports_color() -> bool {
    true // Most modern terminals support color
}

/// Clear the terminal screen.
pub fn clear_screen() -> ShellResult<()> {
    execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))
        .map_err(|e| nxsh_core::ShellError::io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    Ok(())
}

/// Move cursor to position.
pub fn move_cursor(x: u16, y: u16) -> ShellResult<()> {
    execute!(io::stdout(), crossterm::cursor::MoveTo(x, y))
        .map_err(|e| nxsh_core::ShellError::io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    Ok(())
}
