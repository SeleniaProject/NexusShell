use anyhow::Result;
use std::io::{BufRead, BufReader, Write};
use std::fs::File;

/// CLI wrapper function for csplit command
pub fn csplit_cli(args: &[String]) -> Result<()> {
    let mut file_arg = None;
    let mut patterns = Vec::new();
    let mut prefix = "xx".to_string();
    let mut suffix_length = 2;
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--prefix" => {
                if i + 1 < args.len() {
                    prefix = args[i + 1].clone();
                    i += 1;
                }
            }
            "-n" | "--digits" => {
                if i + 1 < args.len() {
                    suffix_length = args[i + 1].parse().unwrap_or(2);
                    i += 1;
                }
            }
            "-h" | "--help" => {
                println!("csplit - split file into sections determined by context lines");
                println!("Usage: csplit [OPTION]... FILE PATTERN...");
                println!("  -f, --prefix=PREFIX  use PREFIX instead of 'xx'");
                println!("  -n, --digits=DIGITS  use DIGITS digits for output filenames");
                println!("  -h, --help           display this help and exit");
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                if file_arg.is_none() {
                    file_arg = Some(arg.to_string());
                } else {
                    patterns.push(arg.to_string());
                }
            }
            _ => {
                eprintln!("csplit: unrecognized option '{}'", args[i]);
                return Err(anyhow::anyhow!("Invalid option"));
            }
        }
        i += 1;
    }
    
    let input_file = file_arg.ok_or_else(|| anyhow::anyhow!("No input file specified"))?;
    
    if patterns.is_empty() {
        return Err(anyhow::anyhow!("No patterns specified"));
    }
    
    // Simple implementation - split on line numbers
    let file = File::open(&input_file)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    
    let mut output_count = 0;
    let mut current_line = 0;
    
    for pattern in &patterns {
        if let Ok(line_num) = pattern.parse::<usize>() {
            if line_num > current_line && line_num <= lines.len() {
                // Create output file
                let output_filename = format!("{prefix}{output_count:0suffix_length$}");
                let mut output_file = File::create(&output_filename)?;
                
                // Write lines to output file
                for line in lines.iter().take(line_num.min(lines.len())).skip(current_line) {
                    writeln!(output_file, "{line}")?;
                }
                
                println!("{output_filename}");
                current_line = line_num;
                output_count += 1;
            }
        }
    }
    
    // Write remaining lines to final file
    if current_line < lines.len() {
        let output_filename = format!("{prefix}{output_count:0suffix_length$}");
        let mut output_file = File::create(&output_filename)?;
        
        for line in lines.iter().skip(current_line) {
            writeln!(output_file, "{line}")?;
        }
        
        println!("{output_filename}");
    }
    
    Ok(())
}
