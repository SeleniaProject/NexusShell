//! `mkdir` command – create directories.
//! Usage:
//!   mkdir [-p] DIR ...
//! If `-p` is supplied, parent directories are created as needed and existing
//! directories are not treated as error.

use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

pub async fn mkdir_cli(args: &[String]) -> Result<()> {
    if args.is_empty() { return Err(anyhow!("mkdir: missing operand")); }
    let mut create_parents = false;
    let mut dirs: Vec<&str> = Vec::new();
    for arg in args {
        if arg == "-p" { create_parents = true; continue; }
        dirs.push(arg);
    }
    if dirs.is_empty() { return Err(anyhow!("mkdir: missing operand")); }
    for d in dirs {
        let path = Path::new(d);
        if create_parents {
            fs::create_dir_all(path)?;
        } else {
            fs::create_dir(path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn mkdir_basic() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("newdir");
        mkdir_cli(&[target.to_string_lossy().into()]).await.unwrap();
        assert!(target.is_dir());
    }
} 