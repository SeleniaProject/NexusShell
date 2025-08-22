//! `fmt` command  Esimple text reformatter (paragraph line wrapping).
//!
//! Supported subset:
//!   fmt [-w WIDTH] [FILE...]
//!   • WIDTH : maximum line width (default 75)
//!   • Paragraphs are separated by blank lines; lines within a paragraph are
//!     reflowed so that each is ≤ WIDTH characters, breaking on whitespace.
//!   • Tabs are treated as single spaces; UTF-8 width is approximate (char count).
//!   • No center/left justification flags implemented.
//!   • FILE of "-" or no FILE reads STDIN.

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn fmt_cli(args: &[String]) -> Result<()> {
    let mut idx = 0;
    let mut width: usize = 75;

    while idx < args.len() {
        match args[idx].as_str() {
            "-w" => {
                idx += 1;
                if idx >= args.len() { return Err(anyhow!("fmt: option requires argument -- w")); }
                width = args[idx].parse()?;
                idx += 1;
            }
            s if s.starts_with("-w") && s.len() > 2 => {
                width = s[2..].parse()?;
                idx += 1;
            }
            "--" => { idx += 1; break; }
            s if s.starts_with('-') && s.len() > 1 => {
                return Err(anyhow!(format!("fmt: unsupported option '{}'.", s)));
            }
            _ => break,
        }
    }

    if idx >= args.len() {
        format_stream("-", width)?;
    } else {
        for p in &args[idx..] {
            format_stream(p, width)?;
        }
    }
    Ok(())
}

fn format_stream(path: &str, width: usize) -> Result<()> {
    let mut input = String::new();
    if path == "-" {
        io::stdin().read_to_string(&mut input)?;
    } else {
        File::open(Path::new(path))?.read_to_string(&mut input)?;
    }
    let mut out = io::stdout();
    for paragraph in input.split("\n\n") {
        let mut line_len = 0usize;
        for word in paragraph.split_whitespace() {
            let wlen = word.chars().count();
            if line_len == 0 {
                write!(out, "{word}")?;
                line_len = wlen;
            } else if line_len + 1 + wlen <= width {
                write!(out, " {word}")?;
                line_len += 1 + wlen;
            } else {
                write!(out, "\n{word}")?;
                line_len = wlen;
            }
        }
        writeln!(out)?; // end paragraph
        writeln!(out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn wrap_width() {
        let text = "word1 word2 word3";
        let buf: Vec<u8> = Vec::new();
        {
            let _ = buf; // compile test only
        }
        let _ = text; // ensure compile
    }
} 

