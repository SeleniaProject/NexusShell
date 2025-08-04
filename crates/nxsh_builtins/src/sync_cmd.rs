//! `sync` command  Eflush file system buffers to disk.
//! Usage: sync
//! Unix: calls libc::sync(); Windows: FlushFileBuffers on all volumes.

use anyhow::Result;

pub async fn sync_cli(_args: &[String]) -> Result<()> {
    #[cfg(unix)]
    {
        // Use nix crate for safe system call instead of direct libc
        use nix::unistd::sync;
        sync();
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::Storage::FileSystem::{FindFirstVolumeW, FindNextVolumeW, FindVolumeClose, CreateFileW, FlushFileBuffers, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL};
        
        
        let mut vol_name: [u16; 1024] = [0; 1024];
        let handle = unsafe { FindFirstVolumeW(vol_name.as_mut_ptr(), vol_name.len() as u32) };
        if handle != std::ptr::null_mut() {
            let mut cur_name = vol_name;
            loop {
                let h = unsafe { CreateFileW(cur_name.as_ptr(), 0, FILE_SHARE_READ|FILE_SHARE_WRITE, std::ptr::null_mut(), OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, std::ptr::null_mut()) };
                if h != windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
                    unsafe { FlushFileBuffers(h); windows_sys::Win32::Foundation::CloseHandle(h); }
                }
                let res = unsafe { FindNextVolumeW(handle, cur_name.as_mut_ptr(), cur_name.len() as u32) };
                if res == 0 { break; }
            }
            unsafe { FindVolumeClose(handle); }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests { use super::*; #[tokio::test] async fn sync_runs(){ sync_cli(&[]).await.unwrap(); }} 
