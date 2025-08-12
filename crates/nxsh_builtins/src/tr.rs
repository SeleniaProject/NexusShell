//! `tr` command  Etranslate or delete characters.
//!
//! Supported subset:
//!   tr SET1 SET2       # translate characters in SET1 to SET2 (1-1, padding with last char)
//!   tr -d SET1         # delete characters in SET1
//!
//! Recognised escape sequences: \n, \t, \r, \0octal (up to 3 digits)
//! Character ranges like `a-z` are expanded. Multi-byte UTF-8 chars are treated as individual code points.
//!
//! This covers the most common use-cases while keeping implementation lightweight.

use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use std::io::{self, Read, Write};

pub fn tr_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("tr: missing operands"));
    }
    let mut idx = 0;
    let mut delete_mode = false;
    if args[idx] == "-d" {
        delete_mode = true;
        idx += 1;
    }
    if (delete_mode && idx >= args.len()) || (!delete_mode && idx + 1 >= args.len()) {
        return Err(anyhow!("tr: missing operands"));
    }

    let set1_raw = &args[idx];
    let set1 = expand_set(set1_raw)?;

    if delete_mode {
        let del_set: HashSet<char> = set1.into_iter().collect();
        process_delete(del_set)?;
        return Ok(());
    }

    let set2_raw = &args[idx + 1];
    let mut set2 = expand_set(set2_raw)?;
    if set2.is_empty() {
        return Err(anyhow!("tr: SET2 is empty"));
    }
    // Pad set2 to match set1 length using last char of set2
    if set2.len() < set1.len() {
        let last = *set2.last().unwrap_or(&' '); // Use space as default if set2 is empty
        set2.resize(set1.len(), last);
    }

    let map: HashMap<char, char> = set1.into_iter().zip(set2).collect();
    process_translate(map)?;
    Ok(())
}

fn process_delete(del: HashSet<char>) -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let output: String = input.chars().filter(|c| !del.contains(c)).collect();
    io::stdout().write_all(output.as_bytes())?;
    Ok(())
}

fn process_translate(map: HashMap<char, char>) -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if let Some(rep) = map.get(&ch) {
            out.push(*rep);
        } else {
            out.push(ch);
        }
    }
    io::stdout().write_all(out.as_bytes())?;
    Ok(())
}

/// Expand a SET string into vector of chars, handling ranges and escapes.
fn expand_set(spec: &str) -> Result<Vec<char>> {
    let mut chars = Vec::new();
    let mut iter = spec.chars().peekable();
    while let Some(c) = iter.next() {
        if c == '\\' {
            // escape sequence
            if let Some(e) = iter.next() {
                let translated = match e {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '0'..='7' => {
                        // octal up to 3 digits (incl current)
                        let mut oct_digits = String::from(e);
                        for _ in 0..2 {
                            if let Some(peek) = iter.peek() {
                                if peek.is_digit(8) {
                                    oct_digits.push(*peek);
                                    iter.next();
                                } else { break; }
                            }
                        }
                        let val = u32::from_str_radix(&oct_digits, 8)?;
                        char::from_u32(val).ok_or_else(|| anyhow!("tr: invalid octal escape"))?
                    }
                    other => other,
                };
                chars.push(translated);
            } else {
                return Err(anyhow!("tr: trailing backslash"));
            }
        } else if let Some('-') = iter.peek() {
            // possible range like a-z
            iter.next(); // consume '-'
            if let Some(end) = iter.next() {
                for ch_code in c as u32..=end as u32 {
                    if let Some(ch) = char::from_u32(ch_code) {
                        chars.push(ch);
                    }
                }
            } else {
                chars.push(c);
                chars.push('-');
            }
        } else {
            chars.push(c);
        }
    }
    Ok(chars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_range() {
        let v = expand_set("a-c").expect("Failed to expand character range");
        assert_eq!(v, vec!['a','b','c']);
    }
} 
