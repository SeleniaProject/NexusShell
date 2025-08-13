//! `more` command  Ebasic pager.
//! Usage: more FILE
//! Reads file and outputs page by page. For non-interactive build, it prints all.

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use terminal_size;
use tokio::task;

pub async fn more_cli(args: &[String]) -> Result<()> {
    if args.is_empty() { return Err(anyhow!("more: missing file operand")); }
    let path = Path::new(&args[0]).to_path_buf();
    task::spawn_blocking(move || pager(path)).await??;
    Ok(())
}

fn pager(path: std::path::PathBuf) -> Result<()> {
    let mut file = File::open(&path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let lines: Vec<&str> = content.lines().collect();
    let height = terminal_size::terminal_size().map(|(_, h)| h.0.saturating_sub(1) as usize).unwrap_or(24);
    let mut stdout = io::stdout();
    let mut idx = 0;
    while idx < lines.len() {
        for _ in 0..height {
            if idx >= lines.len() { break; }
            writeln!(stdout, "{}", lines[idx])?;
            idx += 1;
        }
        if idx < lines.len() {
            write!(stdout, "--More--")?;
            stdout.flush()?;
            let _ = io::stdin().read(&mut [0u8]).unwrap();
            // Move cursor to new line after key press
            writeln!(stdout)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests { use super::*; use tempfile::NamedTempFile; use std::io::Write;
#[tokio::test]
async fn more_basic(){ let mut f=NamedTempFile::new().unwrap(); for i in 0..100{ writeln!(f,"line{}",i).unwrap(); } more_cli(&[f.path().to_string_lossy().into()]).await.unwrap(); }} 
