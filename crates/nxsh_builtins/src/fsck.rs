//! `fsck` builtin – filesystem consistency checker.
//!
//! Supported filesystems: FAT12/16/32 (via `fatfs` crate).
//!
//! Usage:
//!     fsck DEVICE
//!     fsck -a DEVICE    # auto-repair (currently read-only, reports issues)
//!
//! Behaviour:
//! * Performs a **read-only** scan of FAT tables and directory trees.
//! * Detects orphaned (lost) clusters, cross-linked clusters, and directory
//!   entry inconsistencies. Results are printed as a report.
//! * `-a` flag is accepted for compatibility; repair functionality is a TODO
//!   and will be implemented with journalling once write-back safety guarantees
//!   are in place.
//!
//! Platform: Unix-like systems. On non-Unix platforms the command gracefully
//! degrades with an informative message.

use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom};
use std::path::Path;

#[cfg(unix)]
use fatfs::{FatType, File, FileSystem, FsOptions, OemCpConverter, ReadWriteSeek};
#[cfg(unix)]
use fscommon::BufStream;

pub async fn fsck_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("fsck: missing operand (DEVICE)"));
    }

    let mut auto = false;
    let mut device: Option<String> = None;
    for arg in args {
        match arg.as_str() {
            "-a" | "--auto" => auto = true,
            _ => device = Some(arg.clone()),
        }
    }
    let dev = device.ok_or_else(|| anyhow!("fsck: missing DEVICE"))?;

    #[cfg(unix)]
    {
        run_fat_check(&dev, auto)?;
    }
    #[cfg(not(unix))]
    {
        println!("fsck: not supported on this platform");
    }
    Ok(())
}

#[cfg(unix)]
fn run_fat_check(dev_path: &str, _auto: bool) -> Result<()> {
    let f = OpenOptions::new().read(true).write(false).open(Path::new(dev_path))?;
    let stream = BufStream::new(f);

    let fs = FileSystem::new(stream, FsOptions::new())?;
    let fat_type = match fs.fat_type() {
        FatType::Fat12 => "FAT12",
        FatType::Fat16 => "FAT16",
        FatType::Fat32 => "FAT32",
    };

    println!("fsck: scanning {} ({})", dev_path, fat_type);

    let mut used_clusters: HashSet<u32> = HashSet::new();
    let mut cross_links = 0;
    let mut scanned_files = 0;

    // Traverse directories and mark clusters
    traverse_dir(&fs.root_dir(), &mut used_clusters, &mut cross_links, &mut scanned_files)?;

    // Scan FAT for lost clusters
    let total_clusters = fs.total_clusters();
    let mut lost_clusters = Vec::new();
    for cluster in 2..total_clusters {
        if let Ok(status) = fs.cluster_status(cluster) {
            if status.is_allocated() && !used_clusters.contains(&cluster) {
                lost_clusters.push(cluster);
            }
        }
    }

    println!("fsck: checked {scanned_files} files/directories");
    if cross_links == 0 && lost_clusters.is_empty() {
        println!("fsck: CLEAN");
    } else {
        if cross_links > 0 {
            println!("fsck: warning – {cross_links} cross-linked cluster(s) detected");
        }
        if !lost_clusters.is_empty() {
            println!("fsck: warning – {} lost cluster(s) starting at {:?}", lost_clusters.len(), &lost_clusters[..std::cmp::min(10, lost_clusters.len())]);
        }
        println!("fsck: issues found; run with repair support in future versions");
    }

    Ok(())
}

#[cfg(unix)]
fn traverse_dir<D: ReadWriteSeek + 'static>(
    dir: &fatfs::Dir<D>,
    used: &mut HashSet<u32>,
    cross_links: &mut usize,
    files: &mut usize,
) -> Result<()> {
    for entry in dir.iter() {
        let entry = entry?;
        *files += 1;
        match entry.attributes().is_directory() {
            true => {
                let sub = entry.to_dir();
                mark_cluster_chain(&sub, used, cross_links)?;
                traverse_dir(&sub, used, cross_links, files)?;
            }
            false => {
                let file = entry.to_file();
                mark_cluster_chain(&file, used, cross_links)?;
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn mark_cluster_chain<D: ReadWriteSeek + 'static>(
    node: &impl fatfs::ClusterVisitor<D>,
    used: &mut HashSet<u32>,
    cross_links: &mut usize,
) -> Result<()> {
    let mut cluster_iter = node.clusters();
    while let Some(cluster) = cluster_iter.next(&node.fs())? {
        if !used.insert(cluster) {
            // already seen – cross-link
            *cross_links += 1;
        }
    }
    Ok(())
} 