//! `diff` command – display line differences between two files.
//!
//! Minimal subset implemented:
//!   diff FILE1 FILE2
//!   • Compares files line-by-line.
//!   • Output style similar to traditional diff default:
//!       < line (only in FILE1)
//!       > line (only in FILE2)
//!   • Unified/side-by-side modes and option flags are not supported.
//!   • FILE of "-" refers to STDIN (only for FILE1; FILE2 must be path).

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub fn diff_cli(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        return Err(anyhow!("diff: missing file operands"));
    }
    let f1 = &args[0];
    let f2 = &args[1];

    let reader1: Box<dyn BufRead> = if f1 == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        Box::new(BufReader::new(File::open(Path::new(f1))?))
    };
    let reader2: Box<dyn BufRead> = Box::new(BufReader::new(File::open(Path::new(f2))?));

    diff_streams(reader1, reader2)?;
    Ok(())
}

fn diff_streams<R1: BufRead, R2: BufRead>(mut r1: R1, mut r2: R2) -> Result<()> {
    let mut l1 = String::new();
    let mut l2 = String::new();

    loop {
        let eof1 = r1.read_line(&mut l1)? == 0;
        let eof2 = r2.read_line(&mut l2)? == 0;
        if eof1 && eof2 { break; }

        if !eof1 { trim_newline(&mut l1); }
        if !eof2 { trim_newline(&mut l2); }

        match (eof1, eof2) {
            (false, false) => {
                if l1 != l2 {
                    println!("< {}", l1);
                    println!("> {}", l2);
                }
                l1.clear();
                l2.clear();
            }
            (false, true) => {
                println!("< {}", l1);
                l1.clear();
            }
            (true, false) => {
                println!("> {}", l2);
                l2.clear();
            }
            _ => {}
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
    use std::io::Cursor;

    #[test]
    fn diff_no_panic() {
        let a = b"foo\nbar\n";
        let b = b"foo\nbaz\n";
        // Ensure diff runs without error.
        diff_streams(BufReader::new(Cursor::new(a)), BufReader::new(Cursor::new(b))).unwrap();
    }
} 