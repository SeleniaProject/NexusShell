//! `rm` command – remove files or directories.
//! Supported options:
//!   -f : ignore nonexistent and never prompt
//!   -r : recursive for directories
//! Usage: rm [-f] [-r] TARGET...

use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use tokio::task;

pub async fn rm_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("rm: missing operands"));
    }
    let mut force = false;
    let mut recursive = false;
    let mut targets: Vec<String> = Vec::new();
    for arg in args {
        match arg.as_str() {
            "-f" => force = true,
            "-r" | "-R" => recursive = true,
            _ => targets.push(arg.clone()),
        }
    }
    if targets.is_empty() {
        return Err(anyhow!("rm: missing operands"));
    }

    for t in targets {
        let p = Path::new(&t).to_path_buf();
        let f = force;
        let rec = recursive;
        task::spawn_blocking(move || remove_path(p, f, rec)).await??;
    }
    Ok(())
}

fn remove_path(p: std::path::PathBuf, force: bool, rec: bool) -> Result<()> {
    if !p.exists() {
        if force { return Ok(()); } else { return Err(anyhow!("rm: {}: No such file or directory", p.display())); }
    }
    let meta = p.metadata()?;
    if meta.is_dir() {
        if !rec {
            return Err(anyhow!("rm: {}: is a directory", p.display()));
        }
        fs::remove_dir_all(&p)?;
    } else {
        // try unlink
        fs::remove_file(&p)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;

    #[tokio::test]
    async fn remove_file() {
        let dir = tempdir().unwrap();
        let fpath = dir.path().join("tmp.txt");
        let mut f = fs::File::create(&fpath).unwrap();
        writeln!(f, "hi").unwrap();
        rm_cli(&[fpath.to_string_lossy().into()]).await.unwrap();
        assert!(!fpath.exists());
    }
} 