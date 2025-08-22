use anyhow::Result;
use std::io::{self, BufRead, BufReader};
use std::fs::File;

/// CLI wrapper function for unexpand command (convert spaces to tabs)
pub fn unexpand_cli(args: &[String]) -> Result<()> {
    let mut tab_stops = vec![8]; // Default tab stop every 8 characters
    let mut all_blanks = false;
    let mut files = Vec::new();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--all" => {
                all_blanks = true;
            }
            "-t" | "--tabs" => {
                if i + 1 < args.len() {
                    if let Ok(stop) = args[i + 1].parse::<usize>() {
                        tab_stops = vec![stop];
                    } else {
                        // Parse comma-separated list
                        tab_stops = args[i + 1]
                            .split(',')
                            .filter_map(|s| s.parse().ok())
                            .collect();
                    }
                    i += 1;
                }
            }
            "-h" | "--help" => {
                println!("unexpand - convert spaces to tabs");
                println!("Usage: unexpand [OPTION]... [FILE]...");
                println!("  -a, --all          convert all blanks, not just initial blanks");
                println!("  -t, --tabs=N       have tabs N characters apart, not 8");
                println!("  -h, --help         display this help and exit");
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                files.push(arg.to_string());
            }
            _ => {
                eprintln!("unexpand: unrecognized option '{}'", args[i]);
                return Err(anyhow::anyhow!("Invalid option"));
            }
        }
        i += 1;
    }
    
    if files.is_empty() {
        // Read from stdin
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            let unexpanded = unexpand_spaces(&line, &tab_stops, all_blanks);
            println!("{unexpanded}");
        }
    } else {
        // Read from files
        for filename in files {
            let file = File::open(&filename)?;
            let reader = BufReader::new(file);
            
            for line in reader.lines() {
                let line = line?;
                let unexpanded = unexpand_spaces(&line, &tab_stops, all_blanks);
                println!("{unexpanded}");
            }
        }
    }
    
    Ok(())
}

fn unexpand_spaces(line: &str, tab_stops: &[usize], all_blanks: bool) -> String {
    if !all_blanks {
        // Only convert leading spaces
        let leading_spaces = line.chars().take_while(|&c| c == ' ').count();
        if leading_spaces == 0 {
            return line.to_string();
        }
        
        let tabs = leading_spaces / tab_stops.first().copied().unwrap_or(8);
        let remaining_spaces = leading_spaces % tab_stops.first().copied().unwrap_or(8);
        
        let mut result = String::new();
        for _ in 0..tabs {
            result.push('\t');
        }
        for _ in 0..remaining_spaces {
            result.push(' ');
        }
        result.push_str(&line[leading_spaces..]);
        return result;
    }
    
    // Convert all spaces to tabs (simplified implementation)
    let mut result = String::new();
    let mut space_count = 0;
    let mut column = 0;
    
    for ch in line.chars() {
        if ch == ' ' {
            space_count += 1;
            column += 1;
            
            // Check if we've reached a tab stop
            if is_tab_stop(column, tab_stops) && space_count > 0 {
                result.push('\t');
                space_count = 0;
            }
        } else {
            // Output any remaining spaces
            for _ in 0..space_count {
                result.push(' ');
            }
            space_count = 0;
            
            result.push(ch);
            column += 1;
        }
    }
    
    // Output any remaining spaces at end of line
    for _ in 0..space_count {
        result.push(' ');
    }
    
    result
}

fn is_tab_stop(column: usize, tab_stops: &[usize]) -> bool {
    for &stop in tab_stops {
        if column == stop {
            return true;
        }
    }
    
    let interval = tab_stops.first().copied().unwrap_or(8);
    column % interval == 0
}

