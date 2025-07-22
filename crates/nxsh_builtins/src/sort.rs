//! `sort` command – basic line sorting utility.
//!
//! Supported subset:
//!   sort [-r] [-n] [FILE...]
//!
//! • -r : reverse (descending) order
//! • -n : numeric sort (parse leading number, fallback to lexicographic)
//! • FILE of "-" reads STDIN; with no files STDIN is the default.
//!
//! Locale handling, key fields, and stability flags are out of scope for this minimal implementation.

use anyhow::{anyhow, Result};
use std::cmp::Ordering;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

pub fn sort_cli(args: &[String]) -> Result<()> {
    let mut idx = 0;
    let mut reverse = false;
    let mut numeric = false;

    while idx < args.len() {
        match args[idx].as_str() {
            "-r" => reverse = true,
            "-n" => numeric = true,
            "--" => { idx += 1; break; },
            s if s.starts_with('-') && s.len() > 1 => {
                for ch in s.chars().skip(1) {
                    match ch {
                        'r' => reverse = true,
                        'n' => numeric = true,
                        _ => return Err(anyhow!(format!("sort: invalid option -- '{}"), ch))),
                    }
                }
            }
            _ => break,
        }
        idx += 1;
    }

    // Collect lines from files/stdin
    let mut lines: Vec<String> = Vec::new();
    if idx >= args.len() {
        read_into(&mut lines, "-", numeric)?;
    } else {
        for p in &args[idx..] {
            read_into(&mut lines, p, numeric)?;
        }
    }

    // Sort
    lines.sort_by(|a,b| compare(a,b,numeric));
    if reverse { lines.reverse(); }

    let mut out = io::stdout();
    for l in lines { writeln!(out, "{}", l)?; }
    Ok(())
}

fn read_into(lines: &mut Vec<String>, path: &str, _numeric: bool) -> Result<()> {
    if path == "-" {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        lines.extend(input.split('\n').map(|s| s.to_string()));
    } else {
        let file = File::open(Path::new(path))?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            lines.push(line?);
        }
    }
    Ok(())
}

fn compare(a: &str, b: &str, numeric: bool) -> Ordering {
    if numeric {
        let na = a.trim_start().split_whitespace().next().unwrap_or("");
        let nb = b.trim_start().split_whitespace().next().unwrap_or("");
        let pa = na.parse::<f64>();
        let pb = nb.parse::<f64>();
        match (pa, pb) {
            (Ok(x), Ok(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
            _ => a.cmp(b),
        }
    } else {
        a.cmp(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_cmp() {
        assert_eq!(compare("10", "2", true), Ordering::Greater);
    }
} 