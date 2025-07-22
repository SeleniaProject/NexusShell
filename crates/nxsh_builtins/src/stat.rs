//! `stat` command – display detailed file status.
//! Minimal output similar to GNU coreutils default format.
//! Usage: stat FILE...
//! Fields shown: Size, Blocks, Mode (octal), Modified (RFC3339).

use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use tokio::task;

pub async fn stat_cli(args: &[String]) -> Result<()> {
    if args.is_empty() { return Err(anyhow!("stat: missing operand")); }
    for p in args {
        let path = Path::new(p).to_path_buf();
        task::spawn_blocking(move || show_stat(path)).await??;
    }
    Ok(())
}

fn show_stat(path: std::path::PathBuf) -> Result<()> {
    let meta = fs::metadata(&path)?;
    #[cfg(unix)]
    {
        let size = meta.size();
        let blocks = meta.blocks();
        let mode = meta.mode() & 0o7777;
        let mtime: DateTime<Local> = DateTime::from(meta.modified()?);
        println!("  File: {}", path.display());
        println!("  Size: {:<10} Blocks: {:<10} Mode: {:04o}", size, blocks, mode);
        println!("Modify: {}", mtime.to_rfc3339());
    }
    #[cfg(windows)]
    {
        let size = meta.len();
        let mtime: DateTime<Local> = DateTime::from(meta.modified()?);
        println!("  File: {}", path.display());
        println!("  Size: {:<10} Mode: N/A", size);
        println!("Modify: {}", mtime.to_rfc3339());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    #[tokio::test]
    async fn stat_file() {
        let f = NamedTempFile::new().unwrap();
        stat_cli(&[f.path().to_string_lossy().into()]).await.unwrap();
    }
} 