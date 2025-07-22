//! `mv` command – move or rename files and directories.
//! Minimal support:
//!   mv SRC DST
//!   mv SRC1 SRC2 ... DIR
//! Implements cross-device fallback (copy then remove) when rename fails with EXDEV.

use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::task;

pub async fn mv_cli(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        return Err(anyhow!("mv: missing operands"));
    }
    let dst_arg = args.last().unwrap();
    let dst_path = PathBuf::from(dst_arg);
    let srcs = &args[..args.len()-1];
    if srcs.len() > 1 && !dst_path.is_dir() {
        return Err(anyhow!("mv: destination not directory"));
    }
    for src in srcs {
        let src_path = Path::new(src);
        let target = if dst_path.is_dir() { dst_path.join(src_path.file_name().unwrap()) } else { dst_path.clone() };
        let src_clone = src_path.to_path_buf();
        let tgt_clone = target.clone();
        task::spawn_blocking(move || rename_or_copy(src_clone, tgt_clone)).await??;
    }
    Ok(())
}

fn rename_or_copy(src: PathBuf, dst: PathBuf) -> Result<()> {
    match fs::rename(&src, &dst) {
        Ok(_) => Ok(()),
        Err(e) if e.raw_os_error() == Some(libc::EXDEV) => {
            // Cross-filesystem; perform copy then remove
            if src.is_dir() {
                copy_dir_recursively(&src, &dst)?;
                fs::remove_dir_all(&src)?;
            } else {
                fs::copy(&src, &dst)?;
                fs::remove_file(&src)?;
            }
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
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
    async fn rename_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("a.txt");
        let dst = dir.path().join("b.txt");
        let mut f = fs::File::create(&src).unwrap();
        writeln!(f, "hello").unwrap();
        mv_cli(&[src.to_string_lossy().into(), dst.to_string_lossy().into()]).await.unwrap();
        assert!(dst.exists());
        assert!(!src.exists());
    }
} 