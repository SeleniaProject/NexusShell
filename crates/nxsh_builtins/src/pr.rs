use anyhow::Result;
use std::io::{self, BufRead, BufReader};
use std::fs::File;

/// CLI wrapper function for pr command (paginate or columnate files)
pub fn pr_cli(args: &[String]) -> Result<()> {
    let mut columns = 1;
    let mut page_length = 66;
    let mut page_width = 72;
    let mut header = true;
    let mut files = Vec::new();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--columns" => {
                if i + 1 < args.len() {
                    columns = args[i + 1].parse().unwrap_or(1);
                    i += 1;
                }
            }
            "-l" | "--length" => {
                if i + 1 < args.len() {
                    page_length = args[i + 1].parse().unwrap_or(66);
                    i += 1;
                }
            }
            "-w" | "--width" => {
                if i + 1 < args.len() {
                    page_width = args[i + 1].parse().unwrap_or(72);
                    i += 1;
                }
            }
            "-t" | "--omit-header" => {
                header = false;
            }
            "-h" | "--help" => {
                println!("pr - convert text files for printing");
                println!("Usage: pr [OPTION]... [FILE]...");
                println!("  -c, --columns=NUMBER   produce output in NUMBER columns");
                println!("  -l, --length=NUMBER    set page length to NUMBER lines");
                println!("  -w, --width=NUMBER     set page width to NUMBER characters");
                println!("  -t, --omit-header      omit headers and footers");
                println!("  -h, --help             display this help and exit");
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                files.push(arg.to_string());
            }
            _ => {
                eprintln!("pr: unrecognized option '{}'", args[i]);
                return Err(anyhow::anyhow!("Invalid option"));
            }
        }
        i += 1;
    }
    
    if files.is_empty() {
        // Read from stdin
        let stdin = io::stdin();
        let lines: Vec<String> = stdin.lock().lines().collect::<Result<_, _>>()?;
        print_pages(&lines, columns, page_length, page_width, header, "stdin")?;
    } else {
        // Read from files
        for filename in files {
            let file = File::open(&filename)?;
            let reader = BufReader::new(file);
            let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
            print_pages(&lines, columns, page_length, page_width, header, &filename)?;
        }
    }
    
    Ok(())
}

fn print_pages(
    lines: &[String],
    columns: usize,
    page_length: usize,
    page_width: usize,
    header: bool,
    filename: &str
) -> Result<()> {
    let header_lines = if header { 5 } else { 0 };
    let content_lines = page_length.saturating_sub(header_lines);
    let column_width = page_width / columns;
    
    let mut page_num = 1;
    let mut line_idx = 0;
    
    while line_idx < lines.len() {
        if header {
            // Print header
            println!();
            println!();
            println!("{filename:^page_width$}Page {page_num}");
            println!();
            println!();
        }
        
        // Print content
        for _page_line in 0..content_lines {
            if line_idx >= lines.len() {
                break;
            }
            
            let mut output_line = String::new();
            for col in 0..columns {
                let current_line_idx = line_idx + col * (content_lines / columns);
                if current_line_idx < lines.len() {
                    let line = &lines[current_line_idx];
                    let truncated = if line.len() > column_width {
                        &line[..column_width]
                    } else {
                        line
                    };
                    output_line.push_str(&format!("{truncated:<column_width$}"));
                }
            }
            println!("{output_line}");
            line_idx += 1;
        }
        
        page_num += 1;
    }
    
    Ok(())
}

