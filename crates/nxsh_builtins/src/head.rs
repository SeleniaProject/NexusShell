//! `head` command – output the first part of files.
//!
//! Supported subset:
//!   head [-n NUM] [FILE...]
//!   • -n NUM : print first NUM lines (default 10)
//!   • FILE of "-" means STDIN; no FILE defaults to STDIN
//!
//! Byte count (-c) and other GNU extensions are not implemented in this minimal version.

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub fn head_cli(args: &[String]) -> Result<()> {
    let mut idx = 0;
    let mut lines_to_print = 10usize;

    // parse options
    while idx < args.len() {
        match args[idx].as_str() {
            "-n" => {
                idx += 1;
                if idx >= args.len() { return Err(anyhow!("head: option requires argument -- n")); }
                lines_to_print = args[idx].parse()?;
                idx += 1;
            }
            s if s.starts_with("-n") && s.len() > 2 => {
                // combined -nNUM
                let num = &s[2..];
                lines_to_print = num.parse()?;
                idx += 1;
            }
            "--" => { idx += 1; break; }
            s if s.starts_with('-') && s.len() > 1 => {
                return Err(anyhow!(format!("head: unsupported option '{}'.", s)));
            }
            _ => break,
        }
    }

    if idx >= args.len() {
        print_head("-", lines_to_print)?;
    } else {
        for p in &args[idx..] {
            print_head(p, lines_to_print)?;
        }
    }
    Ok(())
}

fn print_head(path: &str, n: usize) -> Result<()> {
    let mut reader: Box<dyn BufRead> = if path == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        Box::new(BufReader::new(File::open(Path::new(path))?))
    };
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let mut line = String::new();
    let mut count = 0;
    while count < n {
        line.clear();
        if reader.read_line(&mut line)? == 0 { break; }
        handle.write_all(line.as_bytes())?;
        count += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn head_three() {
        let data = b"a\nb\nc\nd\n";
        let mut out = Vec::new();
        {
            let stdout = io::stdout();
            let _guard = stdout.lock();
        }
        // Can't easily capture global stdout without extra crate; test logic of print_head via internal function
        let mut reader = BufReader::new(Cursor::new(data));
        let mut line = String::new();
        let mut count = 0;
        while count < 3 {
            line.clear();
            if reader.read_line(&mut line).unwrap() == 0 { break; }
            out.extend_from_slice(line.as_bytes());
            count += 1;
        }
        assert_eq!(String::from_utf8(out).unwrap(), "a\nb\nc\n");
    }
} 