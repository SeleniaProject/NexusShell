//! `cut` command - Column extraction utility.
//!
//! Supported subset (field mode only):
//!   cut -f LIST [-d DELIM] [--output-delimiter=STR] [-s] [FILE...]
//!
//! • LIST: comma-separated 1-based field numbers or ranges (e.g. 1,3,5-7)
//! • DELIM: single-byte delimiter character (default TAB). Escape sequences \t,\n,\r allowed.
//! • Multibyte UTF-8 input is treated as bytes for delimiter splitting (matches GNU cut behaviour).
//! • Lines with fewer fields than requested are handled appropriately.
//! • -s suppresses lines with no delimiter.
//! • --output-delimiter sets output delimiter (default: input delimiter).
//!
//! Character mode (-c) extracts Unicode characters (UTF-8 aware).
//! • Byte mode (-b) extracts raw bytes.

use anyhow::{anyhow, Context, Result};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use super::ui_design::{Colorize, TableFormatter, ColorPalette, Icons};

#[derive(Debug, Clone, Copy)]
enum CutMode {
    Fields,
    Characters,
    Bytes,
}

#[derive(Debug, Clone)]
pub struct Range {
    start: usize,
    end: Option<usize>,
}

impl Range {
    fn new(start: usize, end: Option<usize>) -> Self {
        Self { start, end }
    }
    
    fn contains(&self, index: usize) -> bool {
        match self.end {
            Some(end) => index >= self.start && index <= end,
            None => index == self.start,
        }
    }
}

#[derive(Debug)]
struct CutOptions {
    mode: CutMode,
    ranges: Vec<Range>,
    delimiter: char,
    output_delimiter: Option<String>,
    suppress_no_delim: bool,
    files: Vec<String>,
}

impl Default for CutOptions {
    fn default() -> Self {
        Self {
            mode: CutMode::Fields,
            ranges: Vec::new(),
            delimiter: '\t',
            output_delimiter: None,
            suppress_no_delim: false,
            files: Vec::new(),
        }
    }
}

pub fn cut(args: &[String]) -> Result<()> {
    let options = parse_args(args)?;
    
    if options.ranges.is_empty() {
        return Err(anyhow!("No fields specified"));
    }
    
    let formatter = TableFormatter::new();
    
    // Process each file or stdin
    if options.files.is_empty() {
        process_reader(io::stdin().lock(), &options)?;
    } else {
        for file_path in &options.files {
            if file_path == "-" {
                process_reader(io::stdin().lock(), &options)?;
            } else {
                let file = File::open(file_path)
                    .with_context(|| format!("Failed to open file: {}", file_path))?;
                let reader = BufReader::new(file);
                process_reader(reader, &options)?;
            }
        }
    }
    
    Ok(())
}

fn parse_args(args: &[String]) -> Result<CutOptions> {
    let mut options = CutOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--fields" => {
                if i + 1 >= args.len() {
                    return Err(anyhow!("Option -f requires an argument"));
                }
                options.mode = CutMode::Fields;
                options.ranges = parse_field_list(&args[i + 1])?;
                i += 2;
            }
            "-c" | "--characters" => {
                if i + 1 >= args.len() {
                    return Err(anyhow!("Option -c requires an argument"));
                }
                options.mode = CutMode::Characters;
                options.ranges = parse_field_list(&args[i + 1])?;
                i += 2;
            }
            "-b" | "--bytes" => {
                if i + 1 >= args.len() {
                    return Err(anyhow!("Option -b requires an argument"));
                }
                options.mode = CutMode::Bytes;
                options.ranges = parse_field_list(&args[i + 1])?;
                i += 2;
            }
            "-d" | "--delimiter" => {
                if i + 1 >= args.len() {
                    return Err(anyhow!("Option -d requires an argument"));
                }
                let delim_str = &args[i + 1];
                options.delimiter = parse_delimiter(delim_str)?;
                i += 2;
            }
            "--output-delimiter" => {
                if i + 1 >= args.len() {
                    return Err(anyhow!("Option --output-delimiter requires an argument"));
                }
                options.output_delimiter = Some(args[i + 1].clone());
                i += 2;
            }
            "-s" | "--only-delimited" => {
                options.suppress_no_delim = true;
                i += 1;
            }
            _ => {
                if args[i].starts_with('-') {
                    return Err(anyhow!("Unknown option: {}", args[i]));
                }
                options.files.push(args[i].clone());
                i += 1;
            }
        }
    }
    
    Ok(options)
}

fn parse_field_list(fields: &str) -> Result<Vec<Range>> {
    let mut ranges = Vec::new();
    
    for part in fields.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        
        if let Some(dash_pos) = part.find('-') {
            if dash_pos == 0 {
                // -N format
                let end: usize = part[1..].parse()
                    .with_context(|| format!("Invalid range: {}", part))?;
                ranges.push(Range::new(1, Some(end)));
            } else if dash_pos == part.len() - 1 {
                // N- format
                let start: usize = part[..dash_pos].parse()
                    .with_context(|| format!("Invalid range: {}", part))?;
                ranges.push(Range::new(start, None));
            } else {
                // N-M format
                let start: usize = part[..dash_pos].parse()
                    .with_context(|| format!("Invalid range start: {}", part))?;
                let end: usize = part[dash_pos + 1..].parse()
                    .with_context(|| format!("Invalid range end: {}", part))?;
                if start > end {
                    return Err(anyhow!("Invalid range: start {} > end {}", start, end));
                }
                ranges.push(Range::new(start, Some(end)));
            }
        } else {
            // Single field
            let field: usize = part.parse()
                .with_context(|| format!("Invalid field number: {}", part))?;
            if field == 0 {
                return Err(anyhow!("Field numbers start from 1"));
            }
            ranges.push(Range::new(field, None));
        }
    }
    
    Ok(ranges)
}

fn parse_delimiter(delim_str: &str) -> Result<char> {
    match delim_str {
        "\\t" => Ok('\t'),
        "\\n" => Ok('\n'),
        "\\r" => Ok('\r'),
        s if s.len() == 1 => Ok(s.chars().next().unwrap()),
        _ => Err(anyhow!("Delimiter must be a single character")),
    }
}

fn process_reader<R: BufRead>(reader: R, options: &CutOptions) -> Result<()> {
    for line in reader.lines() {
        let line = line?;
        process_line(&line, options)?;
    }
    Ok(())
}

fn process_line(line: &str, options: &CutOptions) -> Result<()> {
    match options.mode {
        CutMode::Fields => process_fields(line, options),
        CutMode::Characters => process_characters(line, options),
        CutMode::Bytes => process_bytes(line, options),
    }
}

fn process_fields(line: &str, options: &CutOptions) -> Result<()> {
    let fields: Vec<&str> = line.split(options.delimiter).collect();
    
    // Check if line has delimiter
    if options.suppress_no_delim && !line.contains(options.delimiter) {
        return Ok(());
    }
    
    let mut selected_fields = Vec::new();
    
    for range in &options.ranges {
        match range.end {
            Some(end) => {
                for i in range.start..=end {
                    if i > 0 && i <= fields.len() {
                        selected_fields.push(fields[i - 1]);
                    }
                }
            }
            None => {
                if range.start > 0 && range.start <= fields.len() {
                    selected_fields.push(fields[range.start - 1]);
                }
            }
        }
    }
    
    let output_delim = options.output_delimiter
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or(&options.delimiter.to_string());
    
    println!("{}", selected_fields.join(output_delim));
    Ok(())
}

fn process_characters(line: &str, options: &CutOptions) -> Result<()> {
    let chars: Vec<char> = line.chars().collect();
    let mut selected_chars = Vec::new();
    
    for range in &options.ranges {
        match range.end {
            Some(end) => {
                for i in range.start..=end {
                    if i > 0 && i <= chars.len() {
                        selected_chars.push(chars[i - 1]);
                    }
                }
            }
            None => {
                if range.start > 0 && range.start <= chars.len() {
                    selected_chars.push(chars[range.start - 1]);
                }
            }
        }
    }
    
    println!("{}", selected_chars.iter().collect::<String>());
    Ok(())
}

fn process_bytes(line: &str, options: &CutOptions) -> Result<()> {
    let bytes = line.as_bytes();
    let mut selected_bytes = Vec::new();
    
    for range in &options.ranges {
        match range.end {
            Some(end) => {
                for i in range.start..=end {
                    if i > 0 && i <= bytes.len() {
                        selected_bytes.push(bytes[i - 1]);
                    }
                }
            }
            None => {
                if range.start > 0 && range.start <= bytes.len() {
                    selected_bytes.push(bytes[range.start - 1]);
                }
            }
        }
    }
    
    // Convert bytes back to string (may not be valid UTF-8)
    match String::from_utf8(selected_bytes) {
        Ok(s) => println!("{}", s),
        Err(_) => {
            // Print as lossy UTF-8
            let s = String::from_utf8_lossy(&selected_bytes);
            println!("{}", s);
        }
    }
    
    Ok(())
}
