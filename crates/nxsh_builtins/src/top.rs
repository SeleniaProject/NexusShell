//! `top` command — dynamic system monitor in TUI.
//!
//! Key bindings:
//!   q / Esc : quit
//!
//! This implementation uses `ratatui` for rendering and `sysinfo` for metrics.
//! Columns: PID, CMD, CPU%, MEM(KiB)
//! Update interval: 500 ms.
//!
//! Note: In future, we can extend with per-CPU graph and sorting options.

use anyhow::Result;
use crossterm::{
    event::{poll, read, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Terminal,
};
use std::{
    cmp::Ordering,
    io::{stdout, Stdout},
    thread,
    time::Duration,
};
use sysinfo::{ProcessExt, System, SystemExt};

pub fn top_cli(_args: &[String]) -> Result<()> {
    // Prepare terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_top(&mut terminal);

    // Restore terminal even if run_top errored.
    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    res
}

fn run_top(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let mut sys = System::new_all();

    loop {
        // Handle input (non-blocking)
        if poll(Duration::from_millis(1))? {
            if let Event::Key(ev) = read()? {
                match ev.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }

        // Refresh system metrics
        sys.refresh_processes();

        // Build process list sorted by CPU descending
        let mut procs: Vec<_> = sys
            .processes()
            .iter()
            .map(|(pid, p)| (*pid, *p))
            .collect();
        procs.sort_by(|a, b| {
            b.1
                .cpu_usage()
                .partial_cmp(&a.1.cpu_usage())
                .unwrap_or(Ordering::Equal)
        });

        // Compose table rows (limit first 40 for performance & fit)
        let rows: Vec<Row> = procs
            .iter()
            .take(40)
            .map(|(pid, p)| {
                Row::new(vec![
                    Cell::from(pid.to_string()),
                    Cell::from(p.name().to_string()),
                    Cell::from(format!("{:.1}", p.cpu_usage())),
                    Cell::from(format!("{:.0}", p.memory())),
                ])
            })
            .collect();

        // Render frame
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

            let header = Row::new(vec![
                Cell::from("PID").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("CMD").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("CPU% ").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("MEM KiB").style(Style::default().add_modifier(Modifier::BOLD)),
            ]);

            let table = Table::new(rows)
                .header(header)
                .block(Block::default().borders(Borders::ALL).title("NexusShell Top — q to quit"))
                .widths(&[
                    Constraint::Length(7),
                    Constraint::Percentage(50),
                    Constraint::Length(7),
                    Constraint::Length(10),
                ])
                .column_spacing(1);

            f.render_widget(table, chunks[0]);
        })?;

        thread::sleep(Duration::from_millis(500));
    }

    Ok(())
} 