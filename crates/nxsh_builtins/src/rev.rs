//! `rev` command  Ereverse characters of each line.
//!
//! Usage: rev [FILE...]
//!   • With no FILE or FILE "-", reads standard input.
//!   • Outputs each input line with characters reversed, preserving newline.

use anyhow::Result;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub fn rev_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        rev_stream("-")?;
    } else {
        for p in args {
            rev_stream(p)?;
        }
    }
    Ok(())
}

fn rev_stream(path: &str) -> Result<()> {
    let mut reader: Box<dyn BufRead> = if path == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        Box::new(BufReader::new(File::open(Path::new(path))?))
    };
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut line = String::new();
    while reader.read_line(&mut line)? != 0 {
        let mut core = line.trim_end_matches(&['\n','\r'][..]).chars().collect::<Vec<_>>();
        core.reverse();
        for ch in core { write!(out, "{}", ch)?; }
        writeln!(out)?;
        line.clear();
    }
    Ok(())
} 
