//! `wc` command  Eprint newline, word, and byte counts.
//!
//! Supported subset:
//!   wc [-lwmc] [FILE...]
//!   • -l : print newline count
//!   • -w : print word count (runs of non-whitespace)
//!   • -m : print character count (UTF-8 aware)
//!   • -c : print byte count
//!   • With no OPTION, defaults to -lwc (like GNU coreutils)
//!   • FILE of "-" means STDIN; no FILE defaults to STDIN.
//!
//! Flags other than the above (e.g. --files0-from) are not implemented.

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

bitflags::bitflags! {
    struct Mode: u8 {
        const LINES = 0b0001;
        const WORDS = 0b0010;
        const BYTES = 0b0100;
        const CHARS = 0b1000;
        const MAXLINE = 0b1_0000; // -L / --max-line-length
    }
}

pub fn wc_cli(args: &[String]) -> Result<()> {
    let mut idx = 0;
    let mut mode = Mode::empty();

    // parse options
    while idx < args.len() {
        let arg = &args[idx];
        if !arg.starts_with('-') || arg == "-" { break; }
        if arg == "--" { idx += 1; break; }
        if arg.starts_with("--") {
            match arg.as_str() {
                "--lines" => mode |= Mode::LINES,
                "--words" => mode |= Mode::WORDS,
                "--bytes" => mode |= Mode::BYTES,
                "--chars" => mode |= Mode::CHARS,
                "--max-line-length" => mode |= Mode::MAXLINE,
                _ => return Err(anyhow!(format!("wc: invalid option '{}'", arg))),
            }
        } else {
            for ch in arg.chars().skip(1) {
                match ch {
                    'l' => mode |= Mode::LINES,
                    'w' => mode |= Mode::WORDS,
                    'c' => mode |= Mode::BYTES,
                    'm' => mode |= Mode::CHARS,
                    'L' => mode |= Mode::MAXLINE,
                    _ => return Err(anyhow!(format!("wc: invalid option -- '{}'", ch))),
                }
            }
        }
        idx += 1;
    }
    if mode.is_empty() { mode = Mode::LINES | Mode::WORDS | Mode::BYTES; }

    let mut total = (0usize, 0usize, 0usize, 0usize, 0usize); // lines, words, bytes, chars, maxline
    let mut files_processed = 0;

    if idx >= args.len() {
        let counts = count_stream("-", mode)?;
        print_counts(counts, "-", mode, false)?;
    } else {
        for p in &args[idx..] {
            let counts = count_stream(p, mode)?;
            print_counts(counts, p, mode, args.len() - idx > 1)?;
            accumulate(&mut total, counts);
            files_processed += 1;
        }
        if files_processed > 1 {
            print_counts(total, "total", mode, false)?;
        }
    }
    Ok(())
}

fn accumulate(acc: &mut (usize, usize, usize, usize, usize), add: (usize, usize, usize, usize, usize)) {
    acc.0 += add.0;
    acc.1 += add.1;
    acc.2 += add.2;
    acc.3 += add.3;
    acc.4 = acc.4.max(add.4);
}

fn count_stream(path: &str, mode: Mode) -> Result<(usize, usize, usize, usize, usize)> {
    let mut reader: Box<dyn Read> = if path == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(Path::new(path))?)
    };
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let bytes = buf.len();
    let mut lines = 0usize;
    let mut words = 0usize;
    let mut chars = 0usize;
    let mut maxline = 0usize;

    if mode.contains(Mode::LINES) || mode.contains(Mode::WORDS) || mode.contains(Mode::CHARS) || mode.contains(Mode::MAXLINE) {
        let s = std::str::from_utf8(&buf).unwrap_or(unsafe { std::str::from_utf8_unchecked(&buf) });
        if mode.contains(Mode::LINES) {
            lines = s.as_bytes().iter().filter(|&&b| b == b'\n').count();
            // If file doesn't end with newline, GNU wc doesn't count the last partial line addition; we follow same.
        }
        if mode.contains(Mode::WORDS) {
            words = s.split_whitespace().count();
        }
        if mode.contains(Mode::CHARS) || mode.contains(Mode::MAXLINE) {
            if mode.contains(Mode::CHARS) { chars = s.chars().count(); }
            if mode.contains(Mode::MAXLINE) {
                maxline = s.split_inclusive('\n')
                    .map(|line| line.trim_end_matches('\n').chars().count())
                    .max().unwrap_or_else(|| s.chars().count());
            }
        }
    }
    Ok((lines, words, bytes, chars, maxline))
}

fn print_counts(counts: (usize, usize, usize, usize, usize), label: &str, mode: Mode, show_label: bool) -> Result<()> {
    let mut out = io::stdout();
    if mode.contains(Mode::LINES) { write!(out, "{:>8}", counts.0)?; }
    if mode.contains(Mode::WORDS) { write!(out, "{:>8}", counts.1)?; }
    if mode.contains(Mode::BYTES) { write!(out, "{:>8}", counts.2)?; }
    if mode.contains(Mode::CHARS) { write!(out, "{:>8}", counts.3)?; }
    if mode.contains(Mode::MAXLINE) { write!(out, "{:>8}", counts.4)?; }
    if show_label { writeln!(out, " {label}")?; } else { writeln!(out)?; }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn count_basic() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "hello world").unwrap();
        write!(tmp, "second line without newline").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let counts = count_stream(&path, Mode::LINES | Mode::WORDS | Mode::BYTES | Mode::CHARS | Mode::MAXLINE).unwrap();
        let _ = counts; // Ensure function executes without blocking or panic
    }
}
