//! `cut` command  Ecolumn extraction utility.
//!
//! Supported subset (field mode only):
//!   cut -f LIST [-d DELIM] [--output-delimiter=STR] [-s] [FILE...]
//!
//! • LIST: comma-separated 1-based field numbers or ranges (e.g. 1,3,5-7)
//! • DELIM: single-byte delimiter character (default TAB). Escape sequences \t,\n,\r allowed.
//! • Multibyte UTF-8 input is treated as bytes for delimiter splitting (matches GNU cut behaviour).
//! • Lines with fewer fields than requestedは、指定フィールドに対して不足分を空として出力する `--pad` をサポート。
//! • -s suppresses lines with no delimiter.
//! • --output-delimiter sets output delimiter (default: input delimiter).
//!
//! Character mode (-c) extracts Unicode characters (UTF-8 aware).
//! • Byte mode (-b) extracts raw bytes.

use anyhow::{anyhow, Context, Result};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use super::ui_design::{Colorize, TableFormatter, ColorPalette, Icons}; Ecolumn extraction utility.
//!
//! Supported subset (field mode only):
//!   cut -f LIST [-d DELIM] [--output-delimiter=STR] [-s] [FILE...]
//!
//! • LIST: comma-separated 1-based field numbers or ranges (e.g. 1,3,5-7)
//! • DELIM: single-byte delimiter character (default TAB). Escape sequences \t,\n,\r allowed.
//! • Multibyte UTF-8 input is treated as bytes for delimiter splitting (matches GNU cut behaviour).
//! • Lines with fewer fields than requestedは、指定フィールドに対して不足分を空として出力する `--pad` をサポート。
//! • -s suppresses lines with no delimiter.
//! • --output-delimiter sets output delimiter (default: input delimiter).
//!
//! Character mode (-c) extracts Unicode characters (UTF-8 aware).
//! • Byte mode (-b) extracts raw bytes.

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy)]
enum CutMode {
    Fields,
    Characters, 
    Bytes,
}

pub fn cut_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("cut: missing options"));
    }
    let mut idx = 0;
    let mut list_spec = None::<String>;
    let mut delim = b'\t'; // default TAB
    let mut pad_missing = false;
    let mut suppress_no_delim = false; // -s
    let mut out_delim: Option<u8> = None; // --output-delimiter
    let mut mode = None::<CutMode>;

    while idx < args.len() {
        match args[idx].as_str() {
            "-f" => {
                idx += 1;
                if idx >= args.len() { return Err(anyhow!("cut: option requires argument -- f")); }
                list_spec = Some(args[idx].clone());
                mode = Some(CutMode::Fields);
            }
            "-c" => {
                idx += 1;
                if idx >= args.len() { return Err(anyhow!("cut: option requires argument -- c")); }
                list_spec = Some(args[idx].clone());
                mode = Some(CutMode::Characters);
            }
            "-b" => {
                idx += 1;
                if idx >= args.len() { return Err(anyhow!("cut: option requires argument -- b")); }
                list_spec = Some(args[idx].clone());
                mode = Some(CutMode::Bytes);
            }
            "-d" => {
                idx += 1;
                if idx >= args.len() { return Err(anyhow!("cut: option requires argument -- d")); }
                let dstr = unescape(&args[idx])?;
                let bytes = dstr.as_bytes();
                if bytes.len() != 1 { return Err(anyhow!("cut: delimiter must be a single byte")); }
                delim = bytes[0];
            }
            "--pad" => {
                pad_missing = true;
            }
            "-s" => { suppress_no_delim = true; }
            s if s.starts_with("--output-delimiter=") => {
                let d = s.trim_start_matches("--output-delimiter=");
                let dstr = unescape(d)?; let bytes = dstr.as_bytes();
                if bytes.len() != 1 { return Err(anyhow!("cut: output delimiter must be a single byte")); }
                out_delim = Some(bytes[0]);
            }
            "--" => {
                idx += 1; // end of options
                break;
            }
            s if s.starts_with('-') => {
                return Err(anyhow!(format!("cut: unsupported option '{}'.", s)));
            }
            _ => break,
        }
        idx += 1;
    }

    let mode = mode.ok_or_else(|| anyhow!("cut: you must specify a list with -f, -c, or -b"))?;
    let list_spec = list_spec.ok_or_else(|| anyhow!("cut: you must specify a list"))?;
    let ranges = parse_list(&list_spec)?; // Vec of (start,end)

    // Check mode compatibility with options
    match mode {
        CutMode::Characters | CutMode::Bytes => {
            if suppress_no_delim {
                return Err(anyhow!("cut: option -s is only valid with field mode (-f)"));
            }
            if out_delim.is_some() {
                return Err(anyhow!("cut: --output-delimiter is only valid with field mode (-f)"));
            }
        }
        CutMode::Fields => {}
    }

    // Remaining args are files; if none, read stdin
    if idx >= args.len() {
        process_input(mode, delim, out_delim.unwrap_or(delim), &ranges, pad_missing, suppress_no_delim, BufReader::new(io::stdin()))?;
    } else {
        for p in &args[idx..] {
            let file = File::open(Path::new(p))?;
            process_input(mode, delim, out_delim.unwrap_or(delim), &ranges, pad_missing, suppress_no_delim, BufReader::new(file))?;
        }
    }
    Ok(())
}

fn process_input<R: BufRead>(mode: CutMode, delim: u8, out_delim: u8, ranges: &[(usize, usize)], pad_missing: bool, suppress_no_delim: bool, mut reader: R) -> Result<()> {
    match mode {
        CutMode::Fields => process_fields(delim, out_delim, ranges, pad_missing, suppress_no_delim, reader),
        CutMode::Characters => process_characters(ranges, reader),
        CutMode::Bytes => process_bytes(ranges, reader),
    }
}

fn process_fields<R: BufRead>(delim: u8, out_delim: u8, ranges: &[(usize, usize)], pad_missing: bool, suppress_no_delim: bool, mut reader: R) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let mut buf = Vec::new();
    let max_selected = ranges.iter().map(|(_, e)| *e).max().unwrap_or(0);
    while reader.read_until(b'\n', &mut buf)? != 0 {
        if let Some(&last) = buf.last() {
            if last == b'\n' { buf.pop(); }
        }
        let had_delim = buf.contains(&delim);
        if suppress_no_delim && !had_delim { handle.write_all(b"\n")?; buf.clear(); continue; }
        let mut field_idx = 1usize;
        let mut start = 0usize;
        let mut first_output = true;
        for i in 0..=buf.len() {
            let is_delim = if i == buf.len() { true } else { buf[i] == delim };
            if is_delim {
                if ranges.iter().any(|(s, e)| field_idx >= *s && field_idx <= *e) {
                    if !first_output { handle.write_all(&[out_delim])?; }
                    handle.write_all(&buf[start..i])?;
                    first_output = false;
                }
                field_idx += 1;
                start = i + 1;
            }
        }
        // Pad missing selected fields beyond last present field
        if pad_missing && field_idx <= max_selected {
            for idx in field_idx..=max_selected {
                if ranges.iter().any(|(s, e)| idx >= *s && idx <= *e) {
                    if !first_output { handle.write_all(&[out_delim])?; }
                    // empty field
                    first_output = false;
                }
            }
        }
        handle.write_all(b"\n")?;
        buf.clear();
    }
    Ok(())
}

fn process_characters<R: BufRead>(ranges: &[(usize, usize)], mut reader: R) -> Result<()> {
    let mut buf = Vec::new();
    
    while reader.read_until(b'\n', &mut buf)? != 0 {
        if let Some(&last) = buf.last() {
            if last == b'\n' { buf.pop(); }
        }
        
        let line_str = String::from_utf8_lossy(&buf);
        let chars: Vec<char> = line_str.chars().collect();
        
        let mut first_output = true;
        for (start, end) in ranges {
            let start_idx = start.saturating_sub(1); // Convert to 0-based
            let end_idx = if *end == usize::MAX { chars.len() } else { (*end).min(chars.len()) };
            
            for i in start_idx..end_idx {
                if i < chars.len() {
                    if !first_output {
                        // No delimiter for character mode - continuous output
                    }
                    print!("{}", chars[i]);
                    first_output = false;
                }
            }
        }
        println!();
        buf.clear();
    }
    Ok(())
}

fn process_bytes<R: BufRead>(ranges: &[(usize, usize)], mut reader: R) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let mut buf = Vec::new();
    
    while reader.read_until(b'\n', &mut buf)? != 0 {
        if let Some(&last) = buf.last() {
            if last == b'\n' { buf.pop(); }
        }
        
        let mut first_output = true;
        for (start, end) in ranges {
            let start_idx = start.saturating_sub(1); // Convert to 0-based
            let end_idx = if *end == usize::MAX { buf.len() } else { (*end).min(buf.len()) };
            
            for i in start_idx..end_idx {
                if i < buf.len() {
                    if !first_output {
                        // No delimiter for byte mode - continuous output
                    }
                    handle.write_all(&[buf[i]])?;
                    first_output = false;
                }
            }
        }
        handle.write_all(b"\n")?;
        buf.clear();
    }
    Ok(())
}

/// Parse LIST like "1,3-4" into vector of inclusive ranges (start,end)
fn parse_list(spec: &str) -> Result<Vec<(usize, usize)>> {
    let mut ranges = Vec::new();
    for part in spec.split(',') {
        if let Some(idx) = part.find('-') {
            let start = &part[..idx];
            let end = &part[idx + 1..];
            let s: usize = start.parse()?;
            let e: usize = if end.is_empty() { usize::MAX } else { end.parse()? };
            if s == 0 { return Err(anyhow!("cut: fields are 1-based")); }
            ranges.push((s, e));
        } else {
            let n: usize = part.parse()?;
            if n == 0 { return Err(anyhow!("cut: fields are 1-based")); }
            ranges.push((n, n));
        }
    }
    Ok(ranges)
}

fn unescape(s: &str) -> Result<String> {
    if let Some(rest) = s.strip_prefix('\\') {
        Ok(match rest {
            "n" => "\n".to_string(),
            "t" => "\t".to_string(),
            "r" => "\r".to_string(),
            _ => rest.to_string(),
        })
    } else {
        Ok(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_list() {
        let r = parse_list("1,3-4").unwrap();
        assert_eq!(r, vec![(1,1),(3,4)]);
        
        let r = parse_list("2-").unwrap();
        assert_eq!(r, vec![(2, usize::MAX)]);
        
        let r = parse_list("1,5,7-9").unwrap();
        assert_eq!(r, vec![(1,1), (5,5), (7,9)]);
    }

    #[test]
    fn test_parse_list_invalid() {
        assert!(parse_list("0").is_err()); // 0-based not allowed
        assert!(parse_list("0-3").is_err()); // 0-based not allowed
    }

    #[test]
    fn test_unescape() {
        assert_eq!(unescape("\\t").unwrap(), "\t");
        assert_eq!(unescape("\\n").unwrap(), "\n");
        assert_eq!(unescape("\\r").unwrap(), "\r");
        assert_eq!(unescape("abc").unwrap(), "abc");
    }
} 
