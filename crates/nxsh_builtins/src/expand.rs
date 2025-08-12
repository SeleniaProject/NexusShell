use anyhow::Result;
use std::io::{self, BufRead, BufReader};
use std::fs::File;

/// CLI wrapper function for expand command (convert tabs to spaces)
pub fn expand_cli(args: &[String]) -> Result<()> {
    let mut tab_stops = vec![8]; // Default tab stop every 8 characters
    let mut files = Vec::new();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
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
                println!("expand - convert tabs to spaces");
                println!("Usage: expand [OPTION]... [FILE]...");
                println!("  -t, --tabs=N       have tabs N characters apart, not 8");
                println!("  -h, --help         display this help and exit");
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                files.push(arg.to_string());
            }
            _ => {
                eprintln!("expand: unrecognized option '{}'", args[i]);
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
            let expanded = expand_tabs(&line, &tab_stops);
            println!("{expanded}");
        }
    } else {
        // Read from files
        for filename in files {
            let file = File::open(&filename)?;
            let reader = BufReader::new(file);
            
            for line in reader.lines() {
                let line = line?;
                let expanded = expand_tabs(&line, &tab_stops);
                println!("{expanded}");
            }
        }
    }
    
    Ok(())
}

fn expand_tabs(line: &str, tab_stops: &[usize]) -> String {
    let mut result = String::new();
    let mut column = 0;
    
    for ch in line.chars() {
        if ch == '\t' {
            // Find next tab stop
            let next_stop = find_next_tab_stop(column, tab_stops);
            let spaces_needed = next_stop - column;
            for _ in 0..spaces_needed {
                result.push(' ');
            }
            column = next_stop;
        } else {
            result.push(ch);
            column += 1;
        }
    }
    
    result
}

fn find_next_tab_stop(column: usize, tab_stops: &[usize]) -> usize {
    for &stop in tab_stops {
        if column < stop {
            return stop;
        }
    }
    
    // If past all explicit stops, use the last interval
    let interval = tab_stops.last().copied().unwrap_or(8);
    ((column / interval) + 1) * interval
}
