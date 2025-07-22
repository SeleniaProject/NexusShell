//! `echo` builtin – output arguments with optional escape interpretation.
//! Supported options:
//!   -n  : do not output trailing newline
//!   -e  : enable interpretation of backslash escapes (default off)
//!   -E  : disable interpretation (default)
//!
//! Recognized escapes when -e is active:
//!   \n \t \r \b \a \v \f \\ \xHH (hex) \0NNN (octal up to 3 digits)

use anyhow::Result;
use std::io::{self, Write};

pub fn echo_cli(args: &[String]) -> Result<()> {
    let mut newline = true;
    let mut interpret = false;

    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "-n" => { newline = false; idx += 1; },
            "-e" => { interpret = true; idx += 1; },
            "-E" => { interpret = false; idx += 1; },
            _ => break,
        }
    }

    let output_parts: Vec<String> = args[idx..]
        .iter()
        .map(|s| if interpret { unescape(s) } else { s.clone() })
        .collect();

    let mut stdout = io::stdout();
    stdout.write_all(output_parts.join(" ").as_bytes())?;
    if newline {
        stdout.write_all(b"\n")?;
    }
    stdout.flush()?;
    Ok(())
}

fn unescape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        // Got backslash
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('a') => out.push('\x07'),
            Some('b') => out.push('\x08'),
            Some('v') => out.push('\x0b'),
            Some('f') => out.push('\x0c'),
            Some('\\') => out.push('\\'),
            Some('x') => {
                let h1 = chars.next();
                let h2 = chars.next();
                if let (Some(h1), Some(h2)) = (h1, h2) {
                    if let Ok(byte) = u8::from_str_radix(&format!("{}{}", h1, h2), 16) {
                        out.push(byte as char);
                    }
                }
            }
            Some('0') => {
                // up to 3 octal digits already consumed one 0
                let mut oct = String::from("0");
                for _ in 0..2 {
                    if let Some(c) = chars.peek() {
                        if c.is_digit(8) { oct.push(*c); chars.next(); } else { break; }
                    }
                }
                if let Ok(val) = u8::from_str_radix(&oct, 8) {
                    out.push(val as char);
                }
            }
            Some(other) => out.push(other),
            None => break,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_echo() {
        echo_cli(&["hello".into()]).unwrap();
    }
    #[test]
    fn echo_no_newline() {
        echo_cli(&["-n".into(), "hi".into()]).unwrap();
    }
    #[test]
    fn echo_escape() {
        let res = unescape("hello\\nworld");
        assert_eq!(res, "hello\nworld");
    }
} 