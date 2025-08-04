//! `less` command  Eadvanced interactive pager.
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
    // Accept zero or one file argument; if none, read from stdin.
    let path_opt = args.get(0).cloned();
    task::spawn_blocking(move || run_less(path_opt)).await??;
    Ok(())
}

fn run_less(path_opt: Option<String>) -> Result<()> {
    // Load entire content up-front for simplicity. In future, we can stream.
    let mut content = String::new();
    match path_opt {
        Some(p) => {
            let mut f = File::open(Path::new(&p))?;
            f.read_to_string(&mut content)?;
        }
        None => {
            // Read from STDIN until EOF
            io::stdin().read_to_string(&mut content)?;
        }
    }

    let lines: Vec<&str> = content.lines().collect();

    // If not a TTY, print everything and return (non-interactive environment).
    if !is_terminal::IsTerminal::is_terminal(&std::io::stdout()) {
        println!("{}", content);
        return Ok(());
    }

    // Interactive pager.
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    terminal::enable_raw_mode()?;

    let res = pager_loop(&mut stdout, &lines);

    // Restore terminal state regardless of errors.
    terminal::disable_raw_mode()?;
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    res
}

fn pager_loop<W: Write>(mut out: W, lines: &[&str]) -> Result<()> {
    let mut offset: usize = 0;
    loop {
        let (w, h) = terminal::size()?;
        let height = h.saturating_sub(1) as usize; // Leave last line for status

        // Clamp offset to valid range
        if offset > lines.len().saturating_sub(1) {
            offset = lines.len().saturating_sub(1);
        }

        // Clear & redraw screen
        execute!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;
        for (i, line) in lines.iter().skip(offset).take(height).enumerate() {
            queue!(out, cursor::MoveTo(0, i as u16))?;
            // Truncate long lines to width
            let slice = if line.len() > w as usize {
                &line[..w as usize]
            } else {
                line
            };
            write!(out, "{}", slice)?;
        }
        // Status line
        let perc = if lines.is_empty() {
            100
        } else {
            ((offset + height).min(lines.len()) * 100 / lines.len()) as usize
        };
        queue!(out, cursor::MoveTo(0, h - 1))?;
        write!(out, "--Less-- {}% (q to quit)", perc)?;
        out.flush()?;

        // Handle key events
        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break, // quit
                    KeyCode::Char('g') => offset = 0,                // top
                    KeyCode::Char('G') => {
                        offset = lines.len().saturating_sub(height.max(1));
                    }
                    KeyCode::PageDown | KeyCode::Char(' ') => {
                        offset = offset.saturating_add(height);
                    }
                    KeyCode::PageUp | KeyCode::Char('b') => {
                        offset = offset.saturating_sub(height);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        offset = offset.saturating_add(1);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        offset = offset.saturating_sub(1);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write as _;

    #[tokio::test]
    async fn less_basic() {
        let mut f = NamedTempFile::new().unwrap();
        for i in 0..200 {
            writeln!(f, "line{}", i).unwrap();
        }
        // Run pager non-interactive (stdout not a tty) should just print all.
        less_cli(&[f.path().to_string_lossy().into()]).await.unwrap();
    }
} 
