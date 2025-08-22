use anyhow::Result;
use std::io::{self, BufRead, BufReader};
use std::fs::File;

/// CLI wrapper function for nl command (number lines)
pub fn nl_cli(args: &[String]) -> Result<()> {
    let mut number_format = "%6d\t".to_string();
    let mut number_width: usize = 6;
    let mut number_separator: String = "\t".to_string();
    let mut body_numbering = "t"; // t=non-empty lines, a=all lines, n=no lines, pREGEX
    let mut body_pattern: Option<String> = None;
    let mut start_number = 1;
    let mut increment = 1;
    let mut files = Vec::new();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-b" | "--body-numbering" => {
                if i + 1 < args.len() {
                    let val = &args[i + 1];
                    if let Some(p) = val.strip_prefix('p') {
                        body_numbering = "p";
                        body_pattern = Some(p.to_string());
                    } else {
                        body_numbering = val;
                    }
                    i += 1;
                }
            }
            "-n" | "--number-format" => {
                if i + 1 < args.len() {
                    match args[i + 1].as_str() {
                        "ln" => number_format = "%-6d\t".to_string(),
                        "rn" => number_format = "%6d\t".to_string(),
                        "rz" => number_format = "%06d\t".to_string(),
                        _ => number_format = "%6d\t".to_string(),
                    }
                    i += 1;
                }
            }
            "-w" | "--number-width" => {
                if i + 1 < args.len() { number_width = args[i + 1].parse().unwrap_or(6); i += 1; }
            }
            "-s" | "--number-separator" => {
                if i + 1 < args.len() { number_separator = args[i + 1].clone(); i += 1; }
            }
            "-v" | "--starting-line-number" => {
                if i + 1 < args.len() {
                    start_number = args[i + 1].parse().unwrap_or(1);
                    i += 1;
                }
            }
            "-i" | "--line-increment" => {
                if i + 1 < args.len() {
                    increment = args[i + 1].parse().unwrap_or(1);
                    i += 1;
                }
            }
            "-h" | "--help" => {
                println!("nl - number lines of files");
                println!("Usage: nl [OPTION]... [FILE]...");
                println!("  -b, --body-numbering=STYLE    use STYLE for numbering body lines");
                println!("  -n, --number-format=FORMAT    use FORMAT for line numbers");
                println!("  -v, --starting-line-number=N  first line number");
                println!("  -i, --line-increment=N        line number increment");
                println!("  -h, --help                    display this help and exit");
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                files.push(arg.to_string());
            }
            _ => {
                eprintln!("nl: unrecognized option '{}'", args[i]);
                return Err(anyhow::anyhow!("Invalid option"));
            }
        }
        i += 1;
    }
    
    if files.is_empty() {
        // Read from stdin
        let stdin = io::stdin();
        let lines: Vec<String> = stdin.lock().lines().collect::<Result<_, _>>()?;
        number_lines(&lines, &number_format, number_width, &number_separator, body_numbering, body_pattern.as_deref(), start_number, increment)?;
    } else {
        // Read from files
        for filename in files {
            let file = File::open(&filename)?;
            let reader = BufReader::new(file);
            let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
            number_lines(&lines, &number_format, number_width, &number_separator, body_numbering, body_pattern.as_deref(), start_number, increment)?;
        }
    }
    
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn number_lines(
    lines: &[String],
    format: &str,
    width: usize,
    sep: &str,
    numbering_style: &str,
    pattern: Option<&str>,
    start: i32,
    increment: i32
) -> Result<()> {
    let mut line_number = start;
    #[cfg(feature = "advanced-regex")]
    let regex = if numbering_style == "p" { pattern.and_then(|p| fancy_regex::Regex::new(p).ok()) } else { None };
    #[cfg(not(feature = "advanced-regex"))]
    let regex: Option<()> = None;
    
    for line in lines {
        let should_number = match numbering_style {
            "a" => true,  // All lines
            "t" => !line.trim().is_empty(),  // Non-empty lines only
            "n" => false, // No lines
            "p" => {
                #[cfg(feature = "advanced-regex")]
                { regex.as_ref().map(|r| r.is_match(line).unwrap_or(false)).unwrap_or(false) }
                #[cfg(not(feature = "advanced-regex"))]
                { false }
            }
            _ => !line.trim().is_empty(),  // Default: non-empty lines
        };
        
        if should_number {
            if format.contains("%-") {
                print!("{line_number:<width$}{sep}{line}");
            } else if format.contains("%0") {
                print!("{line_number:0width$}{sep}{line}");
            } else {
                print!("{line_number:>width$}{sep}{line}");
            }
            line_number += increment;
        } else {
            print!("{blank}{sep}{line}", blank = " ".repeat(width));
        }
        println!();
    }
    
    Ok(())
}

