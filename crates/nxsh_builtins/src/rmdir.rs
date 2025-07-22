//! `rmdir` command – remove empty directories.
//! Usage: rmdir DIR ...
//! If directory is not empty, prints error.

use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use tokio::task;

pub async fn rmdir_cli(args: &[String]) -> Result<()> {
    if args.is_empty() { return Err(anyhow!("rmdir: missing operand")); }
    for d in args {
        let path = Path::new(d).to_path_buf();
        task::spawn_blocking(move || remove_empty_dir(path)).await??;
    }
    Ok(())
}

fn remove_empty_dir(p: std::path::PathBuf) -> Result<()> {
    if !p.exists() { return Err(anyhow!("rmdir: {}: No such directory", p.display())); }
    match fs::remove_dir(&p) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::DirectoryNotEmpty => {
            Err(anyhow!("rmdir: {}: Directory not empty", p.display()))
        }
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn remove_empty_dir() {
        let t = tempdir().unwrap();
        let d = t.path().join("empty");
        fs::create_dir(&d).unwrap();
        rmdir_cli(&[d.to_string_lossy().into()]).await.unwrap();
        assert!(!d.exists());
    }
} 