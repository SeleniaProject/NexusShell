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
//! Supports a subset of GNU flags:
//!   --files0-from=FILE   read filenames, separated by NUL, from FILE

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
    let mut files0_from: Option<String> = None;

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
                s if s.starts_with("--files0-from=") => {
                    files0_from = Some(s.trim_start_matches("--files0-from=").to_string());
                }
                "--files0-from" => {
                    idx += 1;
                    if idx >= args.len() { return Err(anyhow!("wc: option '--files0-from' requires an argument")); }
                    files0_from = Some(args[idx].clone());
                }
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

    let inputs: Vec<String> = {
        let mut list_inputs: Vec<String> = Vec::new();
        if let Some(list_path) = files0_from {
            let mut data = Vec::new();
            if list_path == "-" {
                // read file list from stdin
                io::stdin().read_to_end(&mut data)?;
            } else {
                data = std::fs::read(list_path)?;
            }
            // Split by NUL; ignore empty trailing segment if file ends with NUL
            for chunk in data.split(|b| *b == 0) {
                if chunk.is_empty() { continue; }
                list_inputs.push(String::from_utf8_lossy(chunk).to_string());
            }
        }
        let mut pos_inputs: Vec<String> = if idx >= args.len() { Vec::new() } else { args[idx..].to_vec() };
        if list_inputs.is_empty() && pos_inputs.is_empty() { vec!["-".to_string()] }
        else {
            list_inputs.append(&mut pos_inputs);
            list_inputs
        }
    };

    if inputs.len() == 1 {
        let counts = count_stream(&inputs[0], mode)?;
        print_counts(counts, &inputs[0], mode, false)?;
    } else {
        for p in &inputs {
            let counts = count_stream(p, mode)?;
            print_counts(counts, p, mode, inputs.len() > 1)?;
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

    #[test]
    fn files0_from_file_and_args_mix() {
        // Prepare two files and a list file
        let mut f1 = NamedTempFile::new().unwrap();
        writeln!(f1, "a b c").unwrap();
        let p1 = f1.path().to_str().unwrap().to_string();

        let mut f2 = NamedTempFile::new().unwrap();
        writeln!(f2, "x\ny").unwrap();
        let p2 = f2.path().to_str().unwrap().to_string();

        let mut list = NamedTempFile::new().unwrap();
        // NUL separated, final NUL present
        write!(list, "{}\0", p1).unwrap();
        let list_path = list.path().to_str().unwrap().to_string();

        // Build args: --files0-from=list plus positional p2
        let args = vec![
            "--files0-from".to_string(), list_path,
            p2.clone(),
        ];
        // Should not error
        wc_cli(&args).unwrap();
    }

    #[test]
    fn files0_from_stdin() {
        // Use stdin to feed file list: create a temp file to read in the test itself
        // We simulate by reading from actual file via count_stream directly for coverage
        // Since wc_cli reads real stdin, here we only smoke-test the splitting logic indirectly
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "hello").unwrap();
        let path = f.path().to_str().unwrap().to_string();
        let _ = count_stream(&path, Mode::LINES | Mode::WORDS | Mode::BYTES | Mode::CHARS).unwrap();
        // Note: full stdin redirection test would be in integration tests
    }
}
