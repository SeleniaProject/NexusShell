//! Enhanced ls command with structured output (NexusShell-inspired)

use std::path::PathBuf;
use anyhow::Result;
use nxsh_core::structured_data::{PipelineData, StructuredValue};
use nxsh_core::structured_commands::paths_to_table;

/// Enhanced ls command with structured table output (NexusShell-inspired)
pub fn ls_table_cli(args: &[String]) -> Result<()> {
    match list_directory_structured(args) {
        Ok(data) => {
            let output = data.format_table();
            println!("{}", output);
            Ok(())
        }
        Err(e) => {
            eprintln!("ls-table: {}", e);
            Err(e)
        }
    }
}

/// List directory contents in structured format
pub fn list_directory_structured(args: &[String]) -> Result<PipelineData> {
    let target_dir = if args.is_empty() {
        std::env::current_dir()?
    } else {
        PathBuf::from(&args[0])
    };

    let mut paths = Vec::new();
    
    if target_dir.is_dir() {
        for entry in std::fs::read_dir(&target_dir)? {
            let entry = entry?;
            paths.push(entry.path());
        }
    } else {
        paths.push(target_dir);
    }

    let table = paths_to_table(&paths)?;
    Ok(PipelineData::new(table))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_structured_ls() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create test files
        fs::write(temp_path.join("test.txt"), "hello").unwrap();
        fs::create_dir(temp_path.join("subdir")).unwrap();
        
        let args = vec![temp_path.to_str().unwrap().to_string()];
        let result = list_directory_structured(&args).unwrap();
        
        if let StructuredValue::Table(rows) = result.value {
            assert!(rows.len() >= 2);
            
            // Check that we have name, type, size columns
            for row in &rows {
                assert!(row.contains_key("name"));
                assert!(row.contains_key("type"));
                assert!(row.contains_key("size"));
            }
        } else {
            panic!("Expected table output");
        }
    }
}
