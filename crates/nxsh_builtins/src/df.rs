//! `df` command  Ereport filesystem disk space usage.
//! Usage: df [-h] [PATH]
//!   -h : human readable sizes
//! If PATH omitted, uses current directory.

use anyhow::{anyhow, Result};
use std::path::Path;
use tokio::task;

#[cfg(unix)]
use nix::libc::{statvfs, c_ulong};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

pub async fn df_cli(args: &[String]) -> Result<()> {
    let mut human = false;
    let mut path = ".".to_string();
    for arg in args {
        if arg == "-h" { human = true; continue; }
        path = arg.clone();
    }
    let p = Path::new(&path).to_path_buf();
    let (blocks, bfree, bavail, bsize) = task::spawn_blocking(move || stat_fs(p)).await??;
    let total = blocks * bsize;
    let avail = bavail * bsize;
    let used = total - avail;
    if human {
        println!("Filesystem Size Used Avail");
        println!("/ {} {} {}", 
            bytesize::ByteSize::b(total).to_string_as(true),
            bytesize::ByteSize::b(used).to_string_as(true),
            bytesize::ByteSize::b(avail).to_string_as(true));
    } else {
        println!("Filesystem 1K-blocks Used Available");
        println!("/ {} {} {}", total/1024, used/1024, avail/1024);
    }
    Ok(())
}

#[cfg(unix)]
fn stat_fs(p: std::path::PathBuf) -> Result<(u64,u64,u64,u64)> {
    let mut vfs: statvfs = unsafe { std::mem::zeroed() };
    let ret = unsafe { statvfs(p.as_os_str().as_bytes().as_ptr() as *const i8, &mut vfs) };
    if ret != 0 { return Err(anyhow!("df: statvfs failed")); }
    Ok((vfs.f_blocks as u64, vfs.f_bfree as u64, vfs.f_bavail as u64, vfs.f_bsize as u64))
}

#[cfg(windows)]
#[cfg(windows)]
fn stat_fs(p: std::path::PathBuf) -> Result<(u64,u64,u64,u64)> {
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    use std::os::windows::ffi::OsStrExt;
    
    let mut free_bytes: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut avail_bytes: u64 = 0;
    let wide_path: Vec<u16> = p.as_os_str().encode_wide().chain(Some(0)).collect();
    let ok = unsafe {
        GetDiskFreeSpaceExW(wide_path.as_ptr(), &mut avail_bytes, &mut total_bytes, &mut free_bytes)
    }!=0;
    if !ok { return Err(anyhow!("df: GetDiskFreeSpaceEx failed")); }
    Ok((total_bytes/4096, free_bytes/4096, avail_bytes/4096, 4096))
}

#[cfg(test)]
mod tests { use super::*; #[tokio::test] async fn df_runs(){ df_cli(&[]).await.unwrap();}} 
