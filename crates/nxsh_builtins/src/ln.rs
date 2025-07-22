//! `ln` command – create hard or symbolic links.
//! Usage:
//!   ln SRC DST              # hard link
//!   ln -s SRC DST           # symbolic link
//!   ln -s SRC1 SRC2 ... DIR # symlinks into DIR
//!   ln -f                   # force overwrite existing

use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::task;

pub async fn ln_cli(args: &[String]) -> Result<()> {
    if args.len() < 2 { return Err(anyhow!("ln: missing operands")); }
    let mut symlink = false;
    let mut force = false;
    let mut paths: Vec<String> = Vec::new();
    for arg in args {
        match arg.as_str() {
            "-s" => symlink = true,
            "-f" => force = true,
            _ => paths.push(arg.clone()),
        }
    }
    if paths.len() < 2 { return Err(anyhow!("ln: missing destination")); }
    let dst_arg = paths.pop().unwrap();
    let dst_path = PathBuf::from(&dst_arg);
    let multiple = paths.len() > 1;
    if multiple && !dst_path.is_dir() {
        return Err(anyhow!("ln: destination '{}' is not a directory", dst_arg)); }

    for src in paths {
        let src_path = Path::new(&src);
        let target = if multiple { dst_path.join(src_path.file_name().unwrap()) } else { dst_path.clone() };
        let sp = src_path.to_path_buf();
        let tp = target.clone();
        task::spawn_blocking(move || create_link(sp, tp, symlink, force)).await??;
    }
    Ok(())
}

fn create_link(src: PathBuf, dst: PathBuf, symlink: bool, force: bool) -> Result<()> {
    if force && dst.exists() { fs::remove_file(&dst)?; }
    if symlink {
        #[cfg(unix)] { std::os::unix::fs::symlink(src, dst)?; }
        #[cfg(windows)] { std::os::windows::fs::symlink_file(src, dst)?; }
    } else {
        fs::hard_link(src, dst)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;
    #[tokio::test]
    async fn hard_link_file() {
        let d = tempdir().unwrap();
        let src = d.path().join("a.txt");
        let dst = d.path().join("b.txt");
        let mut f = fs::File::create(&src).unwrap();
        writeln!(f, "hi").unwrap();
        ln_cli(&[src.to_string_lossy().into(), dst.to_string_lossy().into()]).await.unwrap();
        assert!(dst.exists());
    }
} 