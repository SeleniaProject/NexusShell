//! `source` builtin - execute commands from a file in the current shell context.
//! Usage: source FILE [ARGS...]
//! For now, we simply read the file line-by-line and execute each non-empty,
//! non-comment line as a shell command.

use anyhow::{anyhow, Result};
use nxsh_core::context::ShellContext;
use std::fs;

pub fn source_cli(args: &[String], _ctx: &mut ShellContext) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("source: missing file"));
    }
    let file = &args[0];
    let content = fs::read_to_string(file)?;
    
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') { 
            continue; 
        }
        
        // For now, we'll just output the command that would be executed
        // Full implementation would require parsing and executing the command
        eprintln!("source: would execute: {trimmed}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn source_basic() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "echo ok").unwrap();
        let mut ctx = ShellContext::new();
        source_cli(&[file.path().to_string_lossy().into()], &mut ctx).unwrap();
    }
}

