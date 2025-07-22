//! `tail` command – output the last part of files.
//!
//! Supported subset:
//!   tail [-n NUM] [FILE...]
//!   • -n NUM : print last NUM lines (default 10)
//!   • FILE of "-" means STDIN; no FILE defaults to STDIN
//!   • Follow (-f) and byte mode (-c) are not implemented.
//!
//! For STDIN we buffer lines in a circular VecDeque to avoid unbounded memory.

use anyhow::{anyhow, Result};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub fn tail_cli(args: &[String]) -> Result<()> {
    let mut idx = 0;
    let mut n = 10usize;

    while idx < args.len() {
        match args[idx].as_str() {
            "-n" => {
                idx += 1;
                if idx >= args.len() { return Err(anyhow!("tail: option requires argument -- n")); }
                n = args[idx].parse()?;
                idx += 1;
            }
            s if s.starts_with("-n") && s.len() > 2 => {
                n = s[2..].parse()?;
                idx += 1;
            }
            "--" => { idx += 1; break; }
            s if s.starts_with('-') && s.len() > 1 => {
                return Err(anyhow!(format!("tail: unsupported option '{}'.", s)));
            }
            _ => break,
        }
    }

    if idx >= args.len() {
        tail_file("-", n)?;
    } else {
        for p in &args[idx..] {
            tail_file(p, n)?;
        }
    }
    Ok(())
}

fn tail_file(path: &str, n: usize) -> Result<()> {
    if path == "-" {
        tail_reader(Box::new(BufReader::new(io::stdin())), n)?;
    } else {
        let f = File::open(Path::new(path))?;
        tail_reader(Box::new(BufReader::new(f)), n)?;
    }
    Ok(())
}

fn tail_reader<R: BufRead>(mut reader: Box<R>, n: usize) -> Result<()> {
    let mut buf = VecDeque::with_capacity(n);
    let mut line = String::new();
    while reader.read_line(&mut line)? != 0 {
        if buf.len() == n { buf.pop_front(); }
        buf.push_back(line.clone());
        line.clear();
    }
    let mut stdout = io::stdout();
    for l in buf {
        stdout.write_all(l.as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn tail_two() {
        let input = b"1\n2\n3\n4\n";
        let mut reader = Box::new(BufReader::new(Cursor::new(&input[..])));
        let mut out = Vec::new();
        {
            // Call inner logic but capture result via custom writer by temporarily
        }
        tail_reader(reader, 2).unwrap(); // prints to stdout; simple execution test
    }
} 