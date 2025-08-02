//! `comm` command  Ecompare two sorted files line by line.
//!
//! Minimal subset:
//!   comm FILE1 FILE2
//!   • Assumes both files are sorted lexicographically.
//!   • Output has three TAB-separated columns:
//!       col1: lines only in FILE1
//!       col2: lines only in FILE2
//!       col3: lines common to both
//!   • No column suppression options (-1/-2/-3) implemented yet.
//!
//! FILE of "-" refers to STDIN (only for FILE1 because STDIN can be read once).

use anyhow::{anyhow, Result};
use std::cmp::Ordering;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub fn comm_cli(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        return Err(anyhow!("comm: missing file operands"));
    }
    let f1 = &args[0];
    let f2 = &args[1];

    let reader1: Box<dyn BufRead> = if f1 == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        Box::new(BufReader::new(File::open(Path::new(f1))?))
    };
    let reader2: Box<dyn BufRead> = Box::new(BufReader::new(File::open(Path::new(f2))?));

    comm_streams(reader1, reader2)?;
    Ok(())
}

fn comm_streams<R1: BufRead, R2: BufRead>(mut r1: R1, mut r2: R2) -> Result<()> {
    let mut l1 = String::new();
    let mut l2 = String::new();
    let mut eof1 = r1.read_line(&mut l1)? == 0;
    let mut eof2 = r2.read_line(&mut l2)? == 0;
    let mut out = io::stdout();

    while !(eof1 && eof2) {
        if eof2 {
            write!(out, "{}\n", l1.trim_end())?;
            l1.clear();
            eof1 = r1.read_line(&mut l1)? == 0;
            continue;
        }
        if eof1 {
            write!(out, "\t{}\n", l2.trim_end())?;
            l2.clear();
            eof2 = r2.read_line(&mut l2)? == 0;
            continue;
        }
        match l1.trim_end().cmp(l2.trim_end()) {
            Ordering::Equal => {
                writeln!(out, "\t\t{}", l1.trim_end())?;
                l1.clear();
                l2.clear();
                eof1 = r1.read_line(&mut l1)? == 0;
                eof2 = r2.read_line(&mut l2)? == 0;
            }
            Ordering::Less => {
                write!(out, "{}\n", l1.trim_end())?;
                l1.clear();
                eof1 = r1.read_line(&mut l1)? == 0;
            }
            Ordering::Greater => {
                write!(out, "\t{}\n", l2.trim_end())?;
                l2.clear();
                eof2 = r2.read_line(&mut l2)? == 0;
            }
        }
    }
    Ok(())
} 
