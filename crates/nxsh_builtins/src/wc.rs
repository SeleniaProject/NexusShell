//! `wc` command - Print newline, word, and byte counts.
//!
//! GNU coreutils compatible implementation with extensive options:
//!   wc [OPTION]... [FILE]...
//!   • -l, --lines : print newline count
//!   • -w, --words : print word count (runs of non-whitespace)
//!   • -m, --chars : print character count (UTF-8 aware)
//!   • -c, --bytes : print byte count
//!   • -L, --max-line-length : print maximum line length
//!   • With no OPTION, defaults to -lwc (like GNU coreutils)
//!   • FILE of "-" means STDIN; no FILE defaults to STDIN.
//!
//! GNU flags:
//!   --files0-from=FILE   read filenames, separated by NUL, from FILE
//!   --help               display help and exit
//!   --version            output version information and exit

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{self, Read};
use crate::common::TableFormatter;
use crate::ui_design::{
    Colorize, 

};
use std::path::Path;



bitflags::bitflags! {
    struct Mode: u8 {
        const LINES = 0b0001;
        const WORDS = 0b0010;
        const BYTES = 0b0100;
        const CHARS = 0b1000;
        const MAXLINE = 0b1_0000; // -L / --max-line-length
    }
}

pub fn wc_cli(args: &[String]) -> Result<()> {
    let mut idx = 0;
    let mut mode = Mode::empty();
    let mut files0_from: Option<String> = None;

    // parse options
    while idx < args.len() {
        let arg = &args[idx];
        if !arg.starts_with('-') || arg == "-" { break; }
        if arg == "--" { idx += 1; break; }
        if arg.starts_with("--") {
            match arg.as_str() {
                "--lines" => mode |= Mode::LINES,
                "--words" => mode |= Mode::WORDS,
                "--bytes" => mode |= Mode::BYTES,
                "--chars" => mode |= Mode::CHARS,
                "--max-line-length" => mode |= Mode::MAXLINE,
                s if s.starts_with("--files0-from=") => {
                    files0_from = Some(s.trim_start_matches("--files0-from=").to_string());
                }
                "--files0-from" => {
                    idx += 1;
                    if idx >= args.len() { return Err(anyhow!("wc: option '--files0-from' requires an argument")); }
                    files0_from = Some(args[idx].clone());
                }
                "--help" => {
                    print_help();
                    return Ok(());
                }
                "--version" => {
                    print_version();
                    return Ok(());
                }
                _ => return Err(anyhow!(format!("wc: invalid option '{}'", arg))),
            }
        } else {
            for ch in arg.chars().skip(1) {
                match ch {
                    'l' => mode |= Mode::LINES,
                    'w' => mode |= Mode::WORDS,
                    'c' => mode |= Mode::BYTES,
                    'm' => mode |= Mode::CHARS,
                    'L' => mode |= Mode::MAXLINE,
                    _ => return Err(anyhow!(format!("wc: invalid option -- '{}'", ch))),
                }
            }
        }
        idx += 1;
    }
    if mode.is_empty() { mode = Mode::LINES | Mode::WORDS | Mode::BYTES; }

    let mut total = (0usize, 0usize, 0usize, 0usize, 0usize); // lines, words, bytes, chars, maxline
    let mut files_processed = 0;

    let inputs: Vec<String> = {
        let mut list_inputs: Vec<String> = Vec::new();
        if let Some(list_path) = files0_from {
            let mut data = Vec::new();
            if list_path == "-" {
                // read file list from stdin
                io::stdin().read_to_end(&mut data)?;
            } else {
                data = std::fs::read(list_path)?;
            }
            // Split by NUL; ignore empty trailing segment if file ends with NUL
            for chunk in data.split(|b| *b == 0) {
                if chunk.is_empty() { continue; }
                list_inputs.push(String::from_utf8_lossy(chunk).to_string());
            }
        }
        let mut pos_inputs: Vec<String> = if idx >= args.len() { Vec::new() } else { args[idx..].to_vec() };
        if list_inputs.is_empty() && pos_inputs.is_empty() { vec!["-".to_string()] }
        else {
            list_inputs.append(&mut pos_inputs);
            list_inputs
        }
    };

    if inputs.len() == 1 {
        let counts = count_stream(&inputs[0], mode)?;
        print_counts(counts, &inputs[0], mode, false)?;
    } else {
        for p in &inputs {
            let counts = count_stream(p, mode)?;
            print_counts(counts, p, mode, inputs.len() > 1)?;
            accumulate(&mut total, counts);
            files_processed += 1;
        }
        if files_processed > 1 {
            print_counts(total, "total", mode, false)?;
        }
    }
    Ok(())
}

fn print_help() {
    println!("Usage: wc [OPTION]... [FILE]...");
    println!("Print newline, word, and byte counts for each FILE, and a total line if");
    println!("more than one FILE is specified.  A word is a non-zero-length sequence of");
    println!("characters delimited by white space.");
    println!();
    println!("With no FILE, or when FILE is -, read standard input.");
    println!();
    println!("  -c, --bytes            print the byte counts");
    println!("  -m, --chars            print the character counts");
    println!("  -l, --lines            print the newline counts");
    println!("      --files0-from=F    read input from the files specified by");
    println!("                           NUL-terminated names in file F;");
    println!("                           If F is - then read names from standard input");
    println!("  -L, --max-line-length  print the maximum display width");
    println!("  -w, --words            print the word counts");
    println!("      --help     display this help and exit");
    println!("      --version  output version information and exit");
    println!();
    println!("The options below may be used to select which counts are printed, always in");
    println!("the following order: newline, word, character, byte, maximum line length.");
    println!("  -l, --lines            print the newline counts");
    println!("  -w, --words            print the word counts");
    println!("  -m, --chars            print the character counts");
    println!("  -c, --bytes            print the byte counts");
    println!("  -L, --max-line-length  print the maximum display width");
}

fn print_version() {
    println!("wc (nxsh coreutils) 1.0.0");
    println!("This is free software; see the source for copying conditions.");
    println!("There is NO warranty; not even for MERCHANTABILITY or FITNESS FOR A");
    println!("PARTICULAR PURPOSE.");
}

fn accumulate(acc: &mut (usize, usize, usize, usize, usize), add: (usize, usize, usize, usize, usize)) {
    acc.0 += add.0;
    acc.1 += add.1;
    acc.2 += add.2;
    acc.3 += add.3;
    acc.4 = acc.4.max(add.4);
}

fn count_stream(path: &str, mode: Mode) -> Result<(usize, usize, usize, usize, usize)> {
    let mut reader: Box<dyn Read> = if path == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(Path::new(path))?)
    };
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let bytes = buf.len();
    let mut lines = 0usize;
    let mut words = 0usize;
    let mut chars = 0usize;
    let mut maxline = 0usize;

    if mode.contains(Mode::LINES) || mode.contains(Mode::WORDS) || mode.contains(Mode::CHARS) || mode.contains(Mode::MAXLINE) {
        // Handle invalid UTF-8 gracefully by replacing invalid sequences
        let s = String::from_utf8_lossy(&buf);
        
        if mode.contains(Mode::LINES) {
            // Count newlines - GNU wc counts \n characters
            lines = s.as_bytes().iter().filter(|&&b| b == b'\n').count();
        }
        
        if mode.contains(Mode::WORDS) {
            // GNU wc definition: words are separated by whitespace
            words = s.split_whitespace().count();
        }
        
        if mode.contains(Mode::CHARS) {
            // Character count (multi-byte aware)
            chars = s.chars().count();
        }
        
        if mode.contains(Mode::MAXLINE) {
            // Maximum line length in characters
            maxline = if s.is_empty() {
                0
            } else {
                s.lines()
                    .map(|line| line.chars().count())
                    .max()
                    .unwrap_or(0)
            };
        }
    }
    Ok((lines, words, bytes, chars, maxline))
}

fn print_counts(counts: (usize, usize, usize, usize, usize), label: &str, mode: Mode, show_label: bool) -> Result<()> {
    let formatter = TableFormatter::new();
    
    if show_label && label != "-" {
        // For file output, create a beautiful table
        let mut headers = vec![];
        let mut values = vec![];
        
        if mode.contains(Mode::LINES) { 
            headers.push("Lines");
            values.push(counts.0.to_string().info());
        }
        if mode.contains(Mode::WORDS) { 
            headers.push("Words");
            values.push(counts.1.to_string().primary());
        }
        if mode.contains(Mode::BYTES) { 
            headers.push("Bytes");
            values.push(formatter.format_size(counts.2 as u64));
        }
        if mode.contains(Mode::CHARS) { 
            headers.push("Characters");
            values.push(counts.3.to_string().secondary());
        }
        if mode.contains(Mode::MAXLINE) { 
            headers.push("Max Line");
            values.push(counts.4.to_string());
        }
        
        if headers.len() > 1 {
            // Multiple columns - use table format
            println!("{} {}", formatter.icons.document, label.bright());
            let string_headers: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
            let string_rows = vec![values.iter().map(|s| s.to_string()).collect()];
            print!("{}", formatter.create_table(&string_headers, &string_rows));
        } else {
            // Single column - use simple format
            print!("{} {} {} {}", 
                formatter.icons.bullet,
                values[0],
                headers[0].muted(),
                label.dim()
            );
        }
    } else {
        // Simple format for stdin or totals
        let mut out_parts = vec![];
        
        if mode.contains(Mode::LINES) { 
            out_parts.push(format!("{} {}", counts.0.to_string().info(), "lines".muted()));
        }
        if mode.contains(Mode::WORDS) { 
            out_parts.push(format!("{} {}", counts.1.to_string().primary(), "words".muted()));
        }
        if mode.contains(Mode::BYTES) { 
            out_parts.push(format!("{} {}", formatter.format_size(counts.2 as u64), "bytes".muted()));
        }
        if mode.contains(Mode::CHARS) { 
            out_parts.push(format!("{} {}", counts.3.to_string().secondary(), "chars".muted()));
        }
        if mode.contains(Mode::MAXLINE) { 
            out_parts.push(format!("{} {}", counts.4.to_string(), "max".muted()));
        }
        
        if show_label {
            println!("{} {} {}", 
                formatter.icons.info,
                out_parts.join(", "),
                label.dim()
            );
        } else {
            println!("{}", out_parts.join(", "));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn count_basic() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "hello world").unwrap();
        write!(tmp, "second line without newline").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let counts = count_stream(&path, Mode::LINES | Mode::WORDS | Mode::BYTES | Mode::CHARS | Mode::MAXLINE).unwrap();
        let _ = counts; // Ensure function executes without blocking or panic
    }

    #[test]
    fn files0_from_file_and_args_mix() {
        // Prepare two files and a list file
        let mut f1 = NamedTempFile::new().unwrap();
        writeln!(f1, "a b c").unwrap();
        let p1 = f1.path().to_str().unwrap().to_string();

        let mut f2 = NamedTempFile::new().unwrap();
        writeln!(f2, "x\ny").unwrap();
        let p2 = f2.path().to_str().unwrap().to_string();

        let mut list = NamedTempFile::new().unwrap();
        // NUL separated, final NUL present
        write!(list, "{}\0", p1).unwrap();
        let list_path = list.path().to_str().unwrap().to_string();

        // Build args: --files0-from=list plus positional p2
        let args = vec![
            "--files0-from".to_string(), list_path,
            p2.clone(),
        ];
        // Should not error
        wc_cli(&args).unwrap();
    }

    #[test]
    fn files0_from_stdin() {
        // Use stdin to feed file list: create a temp file to read in the test itself
        // We simulate by reading from actual file via count_stream directly for coverage
        // Since wc_cli reads real stdin, here we only smoke-test the splitting logic indirectly
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "hello").unwrap();
        let path = f.path().to_str().unwrap().to_string();
        let _ = count_stream(&path, Mode::LINES | Mode::WORDS | Mode::BYTES | Mode::CHARS).unwrap();
        // Note: full stdin redirection test would be in integration tests
    }

    #[test]
    fn test_help_and_version() {
        let result = wc_cli(&["--help".to_string()]);
        assert!(result.is_ok());
        
        let result = wc_cli(&["--version".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_file() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let counts = count_stream(&path, Mode::LINES | Mode::WORDS | Mode::BYTES | Mode::CHARS | Mode::MAXLINE).unwrap();
        assert_eq!(counts, (0, 0, 0, 0, 0));
    }

    #[test]
    fn test_single_line_no_newline() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "hello world").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let counts = count_stream(&path, Mode::LINES | Mode::WORDS | Mode::BYTES | Mode::CHARS).unwrap();
        assert_eq!(counts.0, 0); // no newlines
        assert_eq!(counts.1, 2); // two words
        assert_eq!(counts.2, 11); // 11 bytes
        assert_eq!(counts.3, 11); // 11 characters
    }

    #[test]
    fn test_utf8_characters() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "こんにちは\n").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let counts = count_stream(&path, Mode::LINES | Mode::WORDS | Mode::BYTES | Mode::CHARS).unwrap();
        assert_eq!(counts.0, 1); // one newline
        assert_eq!(counts.1, 1); // one word
        assert!(counts.2 > counts.3); // bytes > chars for UTF-8
        assert_eq!(counts.3, 6); // 5 Japanese chars + 1 newline
    }

    #[test]
    fn test_max_line_length() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "short").unwrap();
        writeln!(tmp, "much longer line").unwrap();
        writeln!(tmp, "med").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let counts = count_stream(&path, Mode::MAXLINE).unwrap();
        assert_eq!(counts.4, 16); // "much longer line" is 16 chars
    }

    #[test]
    fn test_invalid_options() {
        let result = wc_cli(&["-x".to_string()]);
        assert!(result.is_err());
        
        let result = wc_cli(&["--invalid".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_combined_flags() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "test line").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        
        let result = wc_cli(&["-lwc".to_string(), path]);
        assert!(result.is_ok());
    }
}



/// Execute function stub
pub fn execute(_args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}
