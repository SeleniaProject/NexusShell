//! `du` command - estimate file space usage.
//! Usage: du [-h] [PATH]
//!   -h : human readable units
//! If PATH omitted, uses current directory.

use anyhow::Result;
use walkdir::WalkDir;
use std::path::Path;

// Beautiful CUI design
use crate::ui_design::{ColorPalette, Icons};

#[cfg(not(feature = "async-runtime"))]
pub fn du_cli(args: &[String]) -> Result<()> {
    let mut human = false;
    let mut path = ".".to_string();
    for arg in args {
        if arg == "-h" { human = true; continue; }
        path = arg.clone();
    }
    
    let colors = ColorPalette::new();
    let icons = Icons::new();
    
    // Beautiful header
    println!("\n{}{}┌─── {} Disk Usage Analysis for {} ───┐{}", 
        colors.primary, "═".repeat(5), Icons::FOLDER, path, colors.reset);
    
    let size = calc_size(Path::new(&path).to_path_buf())?;
    let human_size = bytesize::ByteSize::b(size).to_string_as(true);
    
    // Beautiful table output
    let table = TableFormatter::new();
    let rows = vec![
        vec!["Path".to_string(), "Size".to_string(), "Type".to_string()],
        vec![
            path.to_string(),
            if human { human_size.to_string() } else { size.to_string() },
            "Directory".to_string()
        ]
    ];
    
    println!("{}", table.format());
    Ok(())
}

#[cfg(feature = "async-runtime")]
pub async fn du_cli(args: &[String]) -> Result<()> {
    let mut human = false;
    let mut path = ".".to_string();
    for arg in args {
        if arg == "-h" { human = true; continue; }
        path = arg.clone();
    }
    
    let colors = ColorPalette::new();
    let icons = Icons::new();
    
    // Beautiful header
    println!("\n{}{}┌─── {} Disk Usage Analysis for {} ───┐{}", 
        colors.primary, "═".repeat(5), Icons::FOLDER, path, colors.reset);
    
    let size = calc_size(Path::new(&path).to_path_buf())?;
    let human_size = bytesize::ByteSize::b(size).to_string_as(true);
    
    // Beautiful table output
    let mut table = TableFormatter::new();
    table.add_row(vec!["Path".to_string(), "Size".to_string(), "Type".to_string()]);
    table.add_row(vec![
        path.to_string(),
        if human { human_size } else { size.to_string() },
        "Directory".to_string()
    ]);
    
    println!("{}", table.format());
    Ok(())
}

fn calc_size(path: std::path::PathBuf) -> Result<u64> {
    let mut total = 0;
    
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                total += metadata.len();
            }
        }
    }
    
    Ok(total)
}

// Import statements
use crate::common::TableFormatter;



pub fn execute(args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    #[cfg(feature = "async-runtime")]
    {
        // Use blocking runtime for async code
        use tokio::runtime::Runtime;
        let rt = Runtime::new().map_err(|e| crate::common::BuiltinError::Internal(e.to_string()))?;
        rt.block_on(async {
            match du_cli(args).await {
                Ok(_) => Ok(0),
                Err(e) => {
                    eprintln!("du: {e}");
                    Ok(1)
                }
            }
        })
    }
    #[cfg(not(feature = "async-runtime"))]
    {
        match du_cli(args) {
            Ok(_) => Ok(0),
            Err(e) => {
                eprintln!("du: {}", e);
                Ok(1)
            }
        }
    }
}
