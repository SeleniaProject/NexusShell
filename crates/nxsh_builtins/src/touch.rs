//! `touch` command – update file timestamps or create if not exist.
//! Usage: touch [-a] [-m] FILE...
//!   -a : change atime only
//!   -m : change mtime only
//! If neither -a nor -m specified, both atime and mtime are updated to now.

use anyhow::{anyhow, Result};
use std::fs::{self, OpenOptions};
use std::path::Path;
use tokio::task;

#[cfg(unix)]
use libc::{timespec, utimensat, AT_FDCWD};

pub async fn touch_cli(args: &[String]) -> Result<()> {
    if args.is_empty() { return Err(anyhow!("touch: missing file operand")); }
    let mut change_a = false;
    let mut change_m = false;
    let mut files: Vec<String> = Vec::new();
    for arg in args {
        match arg.as_str() {
            "-a" => change_a = true,
            "-m" => change_m = true,
            _ => files.push(arg.clone()),
        }
    }
    if !change_a && !change_m { change_a = true; change_m = true; }
    for f in files {
        let p = Path::new(&f).to_path_buf();
        task::spawn_blocking(move || update_time(p, change_a, change_m)).await??;
    }
    Ok(())
}

fn update_time(path: std::path::PathBuf, change_a: bool, change_m: bool) -> Result<()> {
    if !path.exists() {
        OpenOptions::new().write(true).create(true).open(&path)?;
    }
    #[cfg(unix)]
    unsafe {
        let now = std::time::SystemTime::now();
        let secs = now.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
        let nsec = now.duration_since(std::time::UNIX_EPOCH)?.subsec_nanos() as i64;
        let ts = timespec { tv_sec: secs, tv_nsec: nsec };
        let mut arr = [ts, ts];
        if !change_a { arr[0].tv_nsec = libc::UTIME_OMIT; }
        if !change_m { arr[1].tv_nsec = libc::UTIME_OMIT; }
        if utimensat(AT_FDCWD, path.as_os_str().as_bytes().as_ptr() as *const i8, arr.as_ptr(), 0) != 0 {
            return Err(anyhow!("touch: failed to update time"));
        }
    }
    #[cfg(windows)]
    {
        // Windows: use file set_times via std::fs::OpenOptions metadata workaround
        use filetime::{FileTime, set_file_times};
        let now = FileTime::now();
        let meta = fs::metadata(&path)?;
        let atime = if change_a { now } else { FileTime::from_last_access_time(&meta) };
        let mtime = if change_m { now } else { FileTime::from_last_modification_time(&meta) };
        set_file_times(&path, atime, mtime)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn touch_create() {
        let f = NamedTempFile::new().unwrap();
        let p = f.path().to_path_buf();
        fs::remove_file(&p).unwrap();
        touch_cli(&[p.to_string_lossy().into()]).await.unwrap();
        assert!(p.exists());
    }
} 