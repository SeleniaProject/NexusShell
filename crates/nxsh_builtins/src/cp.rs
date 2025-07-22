//! `cp` command – copy files and directories.
//! Supported minimal syntax:
//!   cp SRC DST
//!   cp -r SRC_DIR DST_DIR
//! Additional flags (preserve, verbose) are TODO.

use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};

pub async fn cp_cli(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        return Err(anyhow!("cp: missing operands"));
    }
    let mut recursive = false;
    let mut srcs: Vec<String> = Vec::new();
    let mut dst = String::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "-r" || arg == "-R" {
            recursive = true;
            continue;
        }
        srcs.push(arg.clone());
    }
    if srcs.len() < 2 {
        return Err(anyhow!("cp: missing destination"));
    }
    dst = srcs.pop().unwrap();
    let dst_path = PathBuf::from(dst);
    if srcs.len() > 1 && !dst_path.is_dir() {
        return Err(anyhow!("cp: destination must be directory when copying multiple files"));
    }
    for src in srcs {
        let src_path = Path::new(&src);
        let target = if dst_path.is_dir() { dst_path.join(src_path.file_name().unwrap()) } else { dst_path.clone() };
        if src_path.is_dir() {
            if !recursive {
                return Err(anyhow!("cp: -r not specified; omitting directory '{}'.", src));
            }
            copy_dir_recursively(src_path, &target)?;
        } else {
            fs::copy(src_path, &target)?;
        }
    }
    Ok(())
}

fn copy_dir_recursively(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let new_src = entry.path();
        let new_dst = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursively(&new_src, &new_dst)?;
        } else {
            fs::copy(&new_src, &new_dst)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;

    #[tokio::test]
    async fn copy_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("a.txt");
        let dst = dir.path().join("b.txt");
        let mut f = fs::File::create(&src).unwrap();
        writeln!(f, "hello").unwrap();
        cp_cli(&[src.to_string_lossy().into(), dst.to_string_lossy().into()]).await.unwrap();
        assert!(dst.exists());
    }
} 