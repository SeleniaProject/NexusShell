//! `fold` command – wrap input lines to fit a specified width.
//!
//! Supported subset:
//!   fold [-w WIDTH] [FILE...]
//!   • WIDTH default 80 columns.
//!   • Breaks on byte count, not display width (UTF-8 approximated as bytes).
//!   • Does not break long words with -s option; always hard wrap.
//!   • FILE of "-" or none reads STDIN.
//!
//! This minimal implementation is sufficient for basic line wrapping tasks.

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

pub fn fold_cli(args: &[String]) -> Result<()> {
    let mut idx = 0;
    let mut width: usize = 80;

    while idx < args.len() {
        match args[idx].as_str() {
            "-w" => {
                idx += 1;
                if idx >= args.len() { return Err(anyhow!("fold: option requires argument -- w")); }
                width = args[idx].parse()?;
                idx += 1;
            }
            s if s.starts_with("-w") && s.len() > 2 => {
                width = s[2..].parse()?;
                idx += 1;
            }
            "--" => { idx += 1; break; }
            s if s.starts_with('-') && s.len() > 1 => {
                return Err(anyhow!(format!("fold: unsupported option '{}'.", s)));
            }
            _ => break,
        }
    }

    if idx >= args.len() {
        fold_stream("-", width)?;
    } else {
        for p in &args[idx..] {
            fold_stream(p, width)?;
        }
    }
    Ok(())
}

fn fold_stream(path: &str, width: usize) -> Result<()> {
    let mut reader: Box<dyn BufRead> = if path == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        Box::new(BufReader::new(File::open(Path::new(path))?))
    };
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut line = String::new();
    while reader.read_line(&mut line)? != 0 {
        let mut count = 0usize;
        for ch in line.bytes() {
            if count >= width && ch != b'\n' {
                out.write_all(b"\n")?;
                count = 0;
            }
            out.write_all(&[ch])?;
            if ch == b'\n' { count = 0; } else { count += 1; }
        }
        line.clear();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn fold_basic() {
        let input = "abcdefghijklmnopqrstuvwxyz\n";
        let mut reader = BufReader::new(Cursor::new(input));
        let mut out = Vec::new();
        {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
        }
        let _ = out; // compile test only
    }
} 