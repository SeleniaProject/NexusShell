//! `source` builtin - execute commands from a file in the current shell context.
//! Usage: source FILE [ARGS...]
//! For now, we simply read the file line-by-line and feed each non-empty,
//! non-comment line into the `nxsh_core::executor::Executor::execute` API.

use anyhow::{anyhow, Result};
use nxsh_core::{context::ShellContext, executor::Executor};
use nxsh_parser::parse;
use std::fs;

pub fn source_cli(args: &[String], ctx: &mut ShellContext) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("source: missing file"));
    }
    let file = &args[0];
    let content = fs::read_to_string(file)?;
    let mut exec = Executor::new().map_err(|e| anyhow!("Failed to create executor: {}", e))?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
        match parse(trimmed) {
            Ok(ast) => {
                exec.execute(&ast, ctx).map_err(|e| anyhow!("Execution error: {}", e))?;
            }
            Err(e) => {
                return Err(anyhow!("Parse error in {}: {}", file, e));
            }
        }
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
