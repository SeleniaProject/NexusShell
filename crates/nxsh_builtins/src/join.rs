//! `join` command  Ecombine two text files on a common field (relational join).
//!
//! Minimal subset implemented:
//!   join FILE1 FILE2
//!   • Assumes inputs are sorted on the join field.
//!   • Join field is the first whitespace-separated field in each line.
//!   • Output format: key TAB line1_rest TAB line2_rest
//!   • Lines with unmatched keys are skipped (inner join).
//!   • No options (-1, -2, -o, -a, -e, etc.) are supported yet.
//!   • FILE of "-" refers to STDIN (only for FILE1; FILE2 must be path to avoid
//!     consuming the same STDIN twice).
//!
//! This covers the common case of joining two pre-sorted files by their first
//! column.

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub fn join_cli(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        return Err(anyhow!("join: missing file operands"));
    }
    let file1 = &args[0];
    let file2 = &args[1];

    let reader1: Box<dyn BufRead> = if file1 == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        Box::new(BufReader::new(File::open(Path::new(file1))?))
    };
    let reader2: Box<dyn BufRead> = Box::new(BufReader::new(File::open(Path::new(file2))?));

    join_streams(reader1, reader2)?;
    Ok(())
}

fn split_key(line: &str) -> (&str, &str) {
    if let Some(idx) = line.find(char::is_whitespace) {
        let (k, rest) = line.split_at(idx);
        let rest_trim = rest.trim_start_matches(char::is_whitespace);
        (k, rest_trim)
    } else {
        (line.trim_end_matches('\n'), "")
    }
}

fn join_streams<R1: BufRead, R2: BufRead>(mut r1: R1, mut r2: R2) -> Result<()> {
    let mut l1 = String::new();
    let mut l2 = String::new();

    let mut eof1 = r1.read_line(&mut l1)? == 0;
    let mut eof2 = r2.read_line(&mut l2)? == 0;

    let mut out = io::stdout();

    while !(eof1 || eof2) {
        let (k1, rest1) = split_key(l1.trim_end_matches('\n'));
        let (k2, rest2) = split_key(l2.trim_end_matches('\n'));

        match k1.cmp(&k2) {
            std::cmp::Ordering::Equal => {
                writeln!(out, "{}\t{}\t{}", k1, rest1, rest2)?;
                l2.clear();
                eof2 = r2.read_line(&mut l2)? == 0;
            }
            std::cmp::Ordering::Less => {
                l1.clear();
                eof1 = r1.read_line(&mut l1)? == 0;
            }
            std::cmp::Ordering::Greater => {
                l2.clear();
                eof2 = r2.read_line(&mut l2)? == 0;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn join_basic() {
        let data1 = b"a 1\nb 2\nc 3\n";
        let data2 = b"a X\nc Z\n";
        // Execute join_streams; ensure it returns Ok.
        let _ = join_streams(
            BufReader::new(Cursor::new(&data1[..])),
            BufReader::new(Cursor::new(&data2[..]))
        ).unwrap();
    }
} 
