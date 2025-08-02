//! `shred` command  Eoverwrite a file to make recovery difficult.
//! Usage: shred FILE
//! Overwrites with random data once and then truncates to zero.

use anyhow::{anyhow, Result};
use rand::{RngCore, rngs::OsRng};
use std::fs::{self, OpenOptions};
use std::io::{Write, Seek, SeekFrom};
use std::path::Path;
use tokio::task;

pub async fn shred_cli(args: &[String]) -> Result<()> {
    if args.is_empty() { return Err(anyhow!("shred: missing file operand")); }
    for f in args {
        let p = Path::new(f).to_path_buf();
        task::spawn_blocking(move || shred_file(p)).await??;
    }
    Ok(())
}

fn shred_file(path: std::path::PathBuf) -> Result<()> {
    if !path.is_file() { return Err(anyhow!("shred: {}: not a file", path.display())); }
    let metadata = fs::metadata(&path)?;
    let size = metadata.len();
    let mut file = OpenOptions::new().write(true).open(&path)?;
    let mut buf = vec![0u8; 8192];
    let mut remaining = size;
    while remaining > 0 {
        let chunk = std::cmp::min(remaining, buf.len() as u64) as usize;
        OsRng.fill_bytes(&mut buf[..chunk]);
        file.write_all(&buf[..chunk])?;
        remaining -= chunk as u64;
    }
    file.flush()?;
    file.seek(SeekFrom::Start(0))?;
    file.set_len(0)?; // truncate
    fs::remove_file(&path)?;
    Ok(())
}

#[cfg(test)]
mod tests { use super::*; use tempfile::NamedTempFile; use std::io::Write;
#[tokio::test]
async fn shred_basic(){ let mut f=NamedTempFile::new().unwrap(); writeln!(f,"hello").unwrap(); let p=f.path().to_path_buf(); shred_cli(&[p.to_string_lossy().into()]).await.unwrap(); assert!(!p.exists()); }} 
