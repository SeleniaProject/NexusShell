use std::time::{Duration, Instant};
use std::io::{self, Write};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::app::{AppState, MAX_FPS};
use nxsh_core::{context::ShellContext, executor::Executor, ShellResult};
use crate::scroll_buffer::ScrollBuffer;

type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<std::io::Stdout>>;

/// Run TUI interactive shell. Blocks until user exits with Ctrl+D or :quit.
pub fn run(context: &mut ShellContext, executor: &mut Executor) -> ShellResult<()> {
    enable_raw_mode().map_err(|e| nxsh_core::ShellError::io(format!("Failed to enable raw mode: {}", e)))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| nxsh_core::ShellError::io(format!("Failed to init terminal: {}", e)))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| nxsh_core::ShellError::io(format!("Failed to create terminal: {}", e)))?;

    let mut app = NexusShellApp::new();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(1000 / MAX_FPS);

    loop {
        terminal.draw(|f| app.render(f)).map_err(|e| nxsh_core::ShellError::io(format!("Draw error: {}", e)))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout).map_err(|e| nxsh_core::ShellError::io(format!("Poll error: {}", e)))? {
            if let Event::Key(key) = event::read().map_err(|e| nxsh_core::ShellError::io(format!("Read error: {}", e)))? {
                if !app.handle_key(key, context, executor)? {
                    break;
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.update();
            last_tick = Instant::now();
        }
    }

    disable_raw_mode().map_err(|e| nxsh_core::ShellError::io(format!("Failed to disable raw mode: {}", e)))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)
        .map_err(|e| nxsh_core::ShellError::io(format!("Failed to restore terminal: {}", e)))?;
    terminal.show_cursor().ok();
    Ok(())
}

/// Main application state wrapper for TUI.
pub struct NexusShellApp {
    state: AppState,
    scroll: ScrollBuffer,
}

impl NexusShellApp {
    pub fn new() -> Self {
        Self { state: AppState::default(), scroll: ScrollBuffer::default() }
    }

    /// Render UI.
    pub fn render(&self, f: &mut Frame) {
        self.state.render(f);
    }

    /// Handle key event. Returns false if should quit.
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent, ctx: &mut ShellContext, exec: &mut Executor) -> ShellResult<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                return Ok(false);
            }
            KeyCode::Char('d') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                return Ok(false);
            }
            KeyCode::Char(ch) => {
                self.state.input.push(ch);
            }
            KeyCode::Backspace => {
                self.state.input.pop();
            }
            KeyCode::Enter => {
                let cmd = self.state.input.trim().to_string();
                if cmd == ":quit" {
                    return Ok(false);
                }
                if !cmd.is_empty() {
                    match exec.run(&cmd) {
                        Ok(output) => {
                            self.scroll.push(format!("$ {}", cmd));
                            self.scroll.push(format!("=> {:?}", output));
                        }
                        Err(e) => {
                            self.scroll.push(format!("$ {}", cmd));
                            self.scroll.push(format!("error: {}", e));
                        }
                    }
                }
                self.state.input.clear();
            }
            KeyCode::Tab => {
                // TODO: completion placeholder
            }
            KeyCode::Esc => {
                self.state.input.clear();
            }
            _ => {}
        }
        Ok(true)
    }

    /// Periodic update.
    pub fn update(&mut self) {
        // Remove expired toasts
        self.state.toasts.retain(|t| !t.expired());
    }
} 