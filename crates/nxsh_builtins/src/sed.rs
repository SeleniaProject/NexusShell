//! `sed` command – minimal stream editor implementation.
//!
//! Supported subset:
//!   sed [-n] 's/REGEX/REPLACEMENT/[g]' [FILE...]
//!   • Only the substitution command `s` is recognised.
//!   • Delimiter can be any character after `s` (like traditional sed).
//!   • Global flag `g` supported; otherwise first occurrence per line.
//!   • With `-n`, automatic printing is suppressed (mimicking GNU sed `-n`).
//!
//! The goal is to cover the most common use-case (simple replacements) without
//! pulling in a full-featured parser. For complex scripts users should rely on
//! external sed (`/usr/bin/sed`).

use anyhow::{anyhow, Result};
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub fn sed_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("sed: missing script"));
    }

    let mut idx = 0;
    let mut suppress_auto_print = false;
    if args[idx] == "-n" {
        suppress_auto_print = true;
        idx += 1;
    }

    if idx >= args.len() {
        return Err(anyhow!("sed: missing script"));
    }
    // Script string like "s/old/new/g"
    let script = &args[idx];
    idx += 1;

    let (regex, replacement, global) = parse_substitution(script)?;

    // Remaining args are files; if none, read stdin
    if idx >= args.len() {
        process_reader(&regex, &replacement, global, suppress_auto_print, BufReader::new(io::stdin()))?;
    } else {
        for path in &args[idx..] {
            let f = File::open(Path::new(path))?;
            process_reader(&regex, &replacement, global, suppress_auto_print, BufReader::new(f))?;
        }
    }
    Ok(())
}

fn process_reader<R: BufRead>(re: &Regex, repl: &str, global: bool, no_auto: bool, mut reader: R) -> Result<()> {
    let mut line = String::new();
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    while {
        line.clear();
        reader.read_line(&mut line)? != 0
    } {
        // remove trailing newline to avoid duplications later
        let newline = line.ends_with('\n');
        if newline {
            line.pop();
            if line.ends_with('\r') { line.pop(); }
        }

        let new_line = if global {
            re.replace_all(&line, repl).into_owned()
        } else {
            re.replace(&line, repl).into_owned()
        };

        if !no_auto {
            handle.write_all(new_line.as_bytes())?;
            if newline { handle.write_all(b"\n")?; }
        }
    }
    Ok(())
}

/// Parse a sed substitution command. Returns (regex, replacement, global_flag)
fn parse_substitution(cmd: &str) -> Result<(Regex, String, bool)> {
    if !cmd.starts_with('s') {
        return Err(anyhow!("sed: only substitution command supported"));
    }
    let mut chars = cmd.chars();
    chars.next(); // skip 's'
    let delim = chars.next().ok_or_else(|| anyhow!("sed: invalid substitution syntax"))?;
    let rest: String = chars.collect();
    let parts: Vec<&str> = rest.splitn(3, delim).collect();
    if parts.len() < 3 {
        return Err(anyhow!("sed: invalid substitution syntax"));
    }
    let pattern = parts[0];
    let replacement = parts[1].replace("\\", "\\"); // unescape backslash
    let flags = parts[2];
    let global = flags.contains('g');

    let regex = Regex::new(pattern).map_err(|e| anyhow!("sed: invalid regex: {}", e))?;
    Ok((regex, replacement, global))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic() {
        let (re, rep, g) = parse_substitution("s/abc/xyz/g").unwrap();
        assert!(re.is_match("abc"));
        assert_eq!(rep, "xyz");
        assert!(g);
    }
} 