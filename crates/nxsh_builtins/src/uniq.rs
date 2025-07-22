//! `uniq` command – report or filter adjacent duplicate lines.
//!
//! Supported subset:
//!   uniq [-c] [-d] [INPUT [OUTPUT]]
//!
//! • INPUT : file path or '-' for STDIN (default '-')
//! • OUTPUT: path (default STDOUT)
//! • -c : prefix lines by the number of occurrences (like GNU uniq)
//! • -d : only print duplicate lines (those that appear more than once)
//!
//! Options -u, -i, -s, -w などは未実装。

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn uniq_cli(args: &[String]) -> Result<()> {
    let mut idx = 0;
    let mut count_flag = false;
    let mut dup_only = false;

    while idx < args.len() {
        match args[idx].as_str() {
            "-c" => count_flag = true,
            "-d" => dup_only = true,
            "--" => { idx += 1; break; },
            s if s.starts_with('-') && s.len() > 1 => {
                return Err(anyhow!(format!("uniq: unsupported option '{}'.", s)));
            }
            _ => break,
        }
        idx += 1;
    }

    // INPUT
    let input_reader: Box<dyn BufRead> = if idx < args.len() && args[idx] != "-" {
        let f = File::open(Path::new(&args[idx]))?;
        Box::new(BufReader::new(f))
    } else {
        Box::new(BufReader::new(io::stdin()))
    };
    if idx < args.len() && args[idx] != "-" { idx += 1; }

    // OUTPUT
    let mut output: Box<dyn Write> = if idx < args.len() {
        let f = File::create(Path::new(&args[idx]))?;
        Box::new(BufWriter::new(f))
    } else {
        Box::new(io::stdout())
    };

    process_uniq(input_reader, &mut output, count_flag, dup_only)?;
    Ok(())
}

fn process_uniq<R: BufRead, W: Write>(mut reader: R, out: &mut W, count: bool, dup_only: bool) -> Result<()> {
    let mut prev = String::new();
    let mut curr = String::new();
    let mut n = 0usize;

    // prime first line
    if reader.read_line(&mut prev)? == 0 {
        return Ok(()); // empty input
    }
    trim_newline(&mut prev);
    n = 1;
    loop {
        curr.clear();
        let bytes = reader.read_line(&mut curr)?;
        let eof = bytes == 0;
        if !eof { trim_newline(&mut curr); }

        if eof || curr != prev {
            if !dup_only || n > 1 {
                if count {
                    writeln!(out, "{:>7} {}", n, prev)?;
                } else {
                    writeln!(out, "{}", prev)?;
                }
            }
            if eof { break; }
            prev.clear();
            std::mem::swap(&mut prev, &mut curr);
            n = 1;
        } else {
            n += 1;
        }
    }
    Ok(())
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') { s.pop(); }
    if s.ends_with('\r') { s.pop(); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_uniq() {
        let input = b"a\na\nb\n" as &[u8];
        let mut out = Vec::new();
        process_uniq(BufReader::new(input), &mut out, false, false).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "a\nb\n");
    }
} 