//! `df` command - report filesystem disk space usage.
//! Usage: df [-h] [PATH]
//!   -h : human readable sizes
//! If PATH omitted, uses current directory.

use anyhow::{anyhow, Result};
use std::path::Path;
use crate::ui_design::{
    TableFormatter, Colorize, ProgressBar, Animation, TableOptions, BorderStyle, 
    TextAlignment, Notification, NotificationType, create_advanced_table
};
#[cfg(feature = "async-runtime")] use tokio::task;
use std::thread;
use std::time::{Duration, Instant};command  Ereport filesystem disk space usage.
//! Usage: df [-h] [PATH]
//!   -h : human readable sizes
//! If PATH omitted, uses current directory.

use anyhow::{anyhow, Result};
use std::path::Path;
use crate::ui_design::{TableFormatter, Colorize};
#[cfg(feature = "async-runtime")] use tokio::task;

#[cfg(unix)]
use nix::libc::{statvfs, c_ulong};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

#[cfg(not(feature = "async-runtime"))]
pub fn df_cli(args: &[String]) -> Result<()> {
    let mut human = false;
    let mut path = ".".to_string();
    for arg in args { 
        if arg == "-h" { 
            human = true; 
            continue; 
        } 
        path = arg.clone(); 
    }
    
    let formatter = TableFormatter::new();
    
    // Show analysis progress
    println!("{}", formatter.create_header("Disk Space Analysis"));
    println!("{}", Animation::typewriter("Analyzing filesystem...", 20));
    
    let start = Instant::now();
    let mut progress = ProgressBar::new(4, "Scanning disk usage");
    
    progress.update(1, "Reading filesystem info");
    thread::sleep(Duration::from_millis(200));
    
    let (blocks, _bfree, bavail, bsize) = stat_fs(Path::new(&path).to_path_buf())?;
    progress.update(2, "Calculating space usage");
    thread::sleep(Duration::from_millis(150));
    
    let total = blocks * bsize; 
    let avail = bavail * bsize; 
    let used = total - avail;
    let usage_percent = if total > 0 { (used * 100) / total } else { 0 };
    
    progress.update(3, "Formatting results");
    thread::sleep(Duration::from_millis(100));
    
    // Create beautiful table with advanced options
    let headers = vec!["Filesystem", "Total Size", "Used Space", "Available", "Usage %", "Mount Point"];
    let mut rows = vec![];
    
    let size_str = if human {
        bytesize::ByteSize::b(total).to_string_as(true)
    } else {
        format!("{}K", total/1024)
    };
    
    let used_str = if human {
        bytesize::ByteSize::b(used).to_string_as(true)
    } else {
        format!("{}K", used/1024)
    };
    
    let avail_str = if human {
        bytesize::ByteSize::b(avail).to_string_as(true)
    } else {
        format!("{}K", avail/1024)
    };
    
    // Enhanced usage percentage with visual indicators
    let usage_str = match usage_percent {
        90..=100 => format!("{}% ⚠️ CRITICAL", usage_percent).error(),
        80..=89 => format!("{}% ⚠️ HIGH", usage_percent).warning(),
        70..=79 => format!("{}% ⚡ MODERATE", usage_percent).warning(),
        50..=69 => format!("{}% ✓ NORMAL", usage_percent).info(),
        _ => format!("{}% ✓ LOW", usage_percent).success(),
    };
    
    // Create usage bar visualization
    let bar_width = 20;
    let filled = (usage_percent * bar_width / 100) as usize;
    let empty = bar_width - filled;
    let usage_bar = if usage_percent > 90 {
        format!("[{}{}]", "█".repeat(filled).error(), "░".repeat(empty).dim())
    } else if usage_percent > 80 {
        format!("[{}{}]", "█".repeat(filled).warning(), "░".repeat(empty).dim())
    } else {
        format!("[{}{}]", "█".repeat(filled).success(), "░".repeat(empty).dim())
    };
    
    let row = vec![
        format!("{} {}", formatter.icons.archive, path.clone()).primary(),
        size_str.info(),
        used_str.secondary(),
        avail_str.success(),
        format!("{} {}", usage_str, usage_bar),
        path.dim(),
    ];
    rows.push(row);
    
    progress.update(4, "Analysis complete");
    progress.complete("Disk analysis finished");
    
    let analysis_time = start.elapsed();
    
    // Create table with enhanced styling
    let options = TableOptions {
        border_style: BorderStyle::Rounded,
        alternating_rows: false,
        header_alignment: TextAlignment::Center,
        cell_alignment: TextAlignment::Left,
    };
    
    println!("{}", create_advanced_table(&headers, &rows, options));
    
    // Performance and status notifications
    if analysis_time.as_millis() > 500 {
        println!("{}", Notification::new(
            NotificationType::Info,
            "Performance",
            &format!("Analysis completed in {:.2}s", analysis_time.as_secs_f32())
        ).format());
    }
    
    // Storage health warnings
    if usage_percent > 90 {
        println!("{}", Notification::new(
            NotificationType::Warning,
            "Storage Alert",
            "Disk usage is critically high! Consider cleaning up files."
        ).format());
    } else if usage_percent > 80 {
        println!("{}", Notification::new(
            NotificationType::Warning,
            "Storage Notice",
            "Disk usage is getting high. Monitor space regularly."
        ).format());
    }
    
    // Additional disk info
    if human {
        println!("\n{}", "📊 Storage Summary:".primary());
        println!("   • Total Capacity: {}", bytesize::ByteSize::b(total).to_string_as(true).info());
        println!("   • Space Used: {}", bytesize::ByteSize::b(used).to_string_as(true).warning());
        println!("   • Free Space: {}", bytesize::ByteSize::b(avail).to_string_as(true).success());
        println!("   • Utilization: {}%", usage_percent.to_string().primary());
    }
    
    Ok(())
}

#[cfg(feature = "async-runtime")]
pub async fn df_cli(args: &[String]) -> Result<()> {
    let mut human = false;
    let mut path = ".".to_string();
    for arg in args {
        if arg == "-h" { 
            human = true; 
            continue; 
        }
        path = arg.clone();
    }
    
    let formatter = TableFormatter::new();
    
    // Show analysis progress
    println!("{}", formatter.create_header("Disk Space Analysis"));
    println!("{}", Animation::typewriter("Analyzing filesystem...", 20));
    
    let start = Instant::now();
    let mut progress = ProgressBar::new(4, "Scanning disk usage");
    
    progress.update(1, "Reading filesystem info");
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    let p = Path::new(&path).to_path_buf();
    let (blocks, _bfree, bavail, bsize) = task::spawn_blocking(move || stat_fs(p)).await??;
    
    progress.update(2, "Calculating space usage");
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    let total = blocks * bsize;
    let avail = bavail * bsize;
    let used = total - avail;
    let usage_percent = if total > 0 { (used * 100) / total } else { 0 };
    
    progress.update(3, "Formatting results");
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Create beautiful table with advanced options
    let headers = vec!["Filesystem", "Total Size", "Used Space", "Available", "Usage %", "Mount Point"];
    let mut rows = vec![];
    
    let size_str = if human {
        bytesize::ByteSize::b(total).to_string_as(true)
    } else {
        format!("{}K", total/1024)
    };
    
    let used_str = if human {
        bytesize::ByteSize::b(used).to_string_as(true)
    } else {
        format!("{}K", used/1024)
    };
    
    let avail_str = if human {
        bytesize::ByteSize::b(avail).to_string_as(true)
    } else {
        format!("{}K", avail/1024)
    };
    
    // Enhanced usage percentage with visual indicators
    let usage_str = match usage_percent {
        90..=100 => format!("{}% ⚠️ CRITICAL", usage_percent).error(),
        80..=89 => format!("{}% ⚠️ HIGH", usage_percent).warning(),
        70..=79 => format!("{}% ⚡ MODERATE", usage_percent).warning(),
        50..=69 => format!("{}% ✓ NORMAL", usage_percent).info(),
        _ => format!("{}% ✓ LOW", usage_percent).success(),
    };
    
    // Create usage bar visualization
    let bar_width = 20;
    let filled = (usage_percent * bar_width / 100) as usize;
    let empty = bar_width - filled;
    let usage_bar = if usage_percent > 90 {
        format!("[{}{}]", "█".repeat(filled).error(), "░".repeat(empty).dim())
    } else if usage_percent > 80 {
        format!("[{}{}]", "█".repeat(filled).warning(), "░".repeat(empty).dim())
    } else {
        format!("[{}{}]", "█".repeat(filled).success(), "░".repeat(empty).dim())
    };
    
    let row = vec![
        format!("{} {}", formatter.icons.archive, path.clone()).primary(),
        size_str.info(),
        used_str.secondary(),
        avail_str.success(),
        format!("{} {}", usage_str, usage_bar),
        path.dim(),
    ];
    rows.push(row);
    
    progress.update(4, "Analysis complete");
    progress.complete("Disk analysis finished");
    
    let analysis_time = start.elapsed();
    
    // Create table with enhanced styling
    let options = TableOptions {
        border_style: BorderStyle::Rounded,
        alternating_rows: false,
        header_alignment: TextAlignment::Center,
        cell_alignment: TextAlignment::Left,
    };
    
    println!("{}", create_advanced_table(&headers, &rows, options));
    
    // Performance and status notifications
    if analysis_time.as_millis() > 500 {
        println!("{}", Notification::new(
            NotificationType::Info,
            "Performance",
            &format!("Analysis completed in {:.2}s", analysis_time.as_secs_f32())
        ).format());
    }
    
    // Storage health warnings
    if usage_percent > 90 {
        println!("{}", Notification::new(
            NotificationType::Warning,
            "Storage Alert",
            "Disk usage is critically high! Consider cleaning up files."
        ).format());
    } else if usage_percent > 80 {
        println!("{}", Notification::new(
            NotificationType::Warning,
            "Storage Notice",
            "Disk usage is getting high. Monitor space regularly."
        ).format());
    }
    
    // Additional disk info
    if human {
        println!("\n{}", "📊 Storage Summary:".primary());
        println!("   • Total Capacity: {}", bytesize::ByteSize::b(total).to_string_as(true).info());
        println!("   • Space Used: {}", bytesize::ByteSize::b(used).to_string_as(true).warning());
        println!("   • Free Space: {}", bytesize::ByteSize::b(avail).to_string_as(true).success());
        println!("   • Utilization: {}%", usage_percent.to_string().primary());
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
mod tests { use super::*; #[cfg(feature = "async-runtime")] #[tokio::test] async fn df_runs(){ df_cli(&[]).await.unwrap(); } #[cfg(not(feature = "async-runtime"))] #[test] fn df_runs_sync(){ df_cli(&[]).unwrap(); } } 
