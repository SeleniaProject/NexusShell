//! `paste` command – horizontal file merging.
//!
//! Supported subset:
//!   paste [-d DELIM] FILE...
//!   • FILE of "-" means STDIN
//!   • Lines from each file are joined with DELIM (default TAB) pair-wise.
//!   • If files have different lengths, missing fields are treated as empty.
//!   • Serial (-s) mode is not implemented in this minimal version.
//!
//! This implementation aims for practical daily use without covering every GNU paste option.

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

pub fn paste_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("paste: missing file operands"));
    }
    let mut idx = 0;
    let mut delim = '\t';
    while idx < args.len() {
        match args[idx].as_str() {
            "-d" => {
                idx += 1;
                if idx >= args.len() { return Err(anyhow!("paste: option requires argument -- d")); }
                let d = unescape(&args[idx])?;
                if d.chars().count() != 1 { return Err(anyhow!("paste: delimiter must be single character")); }
                delim = d.chars().next().unwrap();
                idx += 1;
            }
            "--" => { idx += 1; break; }
            _ => break,
        }
    }

    if idx >= args.len() {
        return Err(anyhow!("paste: missing file operands"));
    }

    // Open files/stdin
    let mut readers: Vec<Box<dyn BufRead>> = Vec::new();
    for p in &args[idx..] {
        if p == "-" {
            readers.push(Box::new(BufReader::new(io::stdin())));
        } else {
            let f = File::open(Path::new(p))?;
            readers.push(Box::new(BufReader::new(f)));
        }
    }

    process_paste(&mut readers, delim)?;
    Ok(())
}

fn process_paste(readers: &mut [Box<dyn BufRead>], delim: char) -> Result<()> {
    let mut buffers: Vec<String> = vec![String::new(); readers.len()];
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    loop {
        let mut eof_count = 0;
        for (i, rdr) in readers.iter_mut().enumerate() {
            buffers[i].clear();
            let n = rdr.read_line(&mut buffers[i])?;
            if n == 0 {
                eof_count += 1;
            } else {
                // trim trailing newline
                if buffers[i].ends_with('\n') { buffers[i].pop(); }
                if buffers[i].ends_with('\r') { buffers[i].pop(); }
            }
        }
        if eof_count == readers.len() { break; }
        for (i, buf) in buffers.iter().enumerate() {
            if i != 0 { write!(handle, "{}", delim)?; }
            handle.write_all(buf.as_bytes())?;
        }
        handle.write_all(b"\n")?;
    }
    Ok(())
}

fn unescape(s: &str) -> Result<String> {
    if s.starts_with('\\') {
        Ok(match &s[1..] {
            "t" => "\t".to_string(),
            "n" => "\n".to_string(),
            "r" => "\r".to_string(),
            other => other.to_string(),
        })
    } else {
        Ok(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unesc() {
        assert_eq!(unescape("\\t").unwrap(), "\t");
    }
} 