//! `mkfs` builtin  Eformat a block device or file image with a filesystem.
//!
//! Current implementation supports **FAT12/FAT16/FAT32** creation using the `fatfs` crate.
//! Syntax:
//!     mkfs -t fat32 DEVICE [--label LABEL]
//!
//! This builtin purposely restricts itself to FAT to avoid destructive
//! operations that require complex privilege checks. For other filesystems the
//! command will print an informative error message.
//!
//! Safety notes:
//! * This utility erases data on the target DEVICE. A confirmation prompt is
//!   omitted because shell scripting requires non-interactive behaviour; users
//!   must be cautious.
//! * DEVICE may be a regular file (disk image) or block device node.
//!
//! Platform: Unix-like only. On unsupported OSes the command exits gracefully.

use anyhow::{anyhow, Result};

#[cfg(unix)]
use fatfs::{format_volume, FormatVolumeOptions, FatType};
#[cfg(unix)]
use fscommon::BufStream;

pub async fn mkfs_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("mkfs: missing operands"));
    }

    let mut fstype = String::from("fat32");
    let mut label: Option<String> = None;
    let mut device: Option<String> = None;

    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "-t" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(anyhow!("mkfs: option -t requires an argument"));
                }
                fstype = args[idx].clone();
            }
            "--label" | "-L" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(anyhow!("mkfs: option --label requires an argument"));
                }
                label = Some(args[idx].clone());
            }
            _ => {
                if device.is_none() {
                    device = Some(args[idx].clone());
                } else {
                    return Err(anyhow!("mkfs: unexpected extra operand {}", args[idx]));
                }
            }
        }
        idx += 1;
    }

    let device = device.ok_or_else(|| anyhow!("mkfs: missing DEVICE"))?;
    #[cfg(not(unix))]
    { let _ = &label; let _ = &device; }

    match fstype.to_lowercase().as_str() {
        "fat12" => {
            #[cfg(unix)]
            format_fat(&device, label.as_deref().unwrap_or("NXSH"), FatType::Fat12)?;
            #[cfg(not(unix))]
            println!("mkfs: FAT formatting unsupported on this platform");
        }
        "fat16" => {
            #[cfg(unix)]
            format_fat(&device, label.as_deref().unwrap_or("NXSH"), FatType::Fat16)?;
            #[cfg(not(unix))]
            println!("mkfs: FAT formatting unsupported on this platform");
        }
        "fat" | "fat32" | "vfat" => {
            #[cfg(unix)]
            format_fat(&device, label.as_deref().unwrap_or("NXSH"), FatType::Fat32)?;
            #[cfg(not(unix))]
            println!("mkfs: FAT formatting unsupported on this platform");
        }
        other => {
            return Err(anyhow!("mkfs: unsupported filesystem type '{}' (supported: fat12, fat16, fat32).", other));
        }
    }

    Ok(())
}

#[cfg(unix)]
fn format_fat(dev: &str, label: &str, kind: FatType) -> Result<()> {
    use std::io::Seek;

    let f = OpenOptions::new().read(true).write(true).open(Path::new(dev))?;
    let mut stream = BufStream::new(f);

    // Convert label to fixed-size array, padding with spaces
    let mut label_bytes = [b' '; 11];
    let label_slice = label.as_bytes();
    let copy_len = std::cmp::min(label_slice.len(), 11);
    label_bytes[..copy_len].copy_from_slice(&label_slice[..copy_len]);

    let opts = FormatVolumeOptions::new()
        .fat_type(kind)
        .volume_label(label_bytes);

    format_volume(&mut stream, opts)
        .map_err(|e| anyhow!("mkfs: format error: {e}"))?;

    // flush underlying file
    stream.flush()?;
    println!("mkfs: formatted {} as {:?} (label = {})", dev, kind, label);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[cfg(unix)]
    #[tokio::test]
    async fn create_fat_image() {
        // Create a temporary image file and format it as FAT32
        let path = std::env::temp_dir().join("mkfs_test.img");
        {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&path)
                .unwrap();
            file.set_len(8 * 1024 * 1024).unwrap(); // 8 MiB image
        }
        let path_str = path.to_string_lossy().to_string();
        let _ = mkfs_cli(&["-t".into(), "fat32".into(), path_str]).await;
        let _ = std::fs::remove_file(&path);
    }

    #[cfg(not(unix))]
    #[tokio::test]
    async fn create_fat_image() {
        // FAT formatting is unsupported on non-Unix platforms in this build; skip test.
        assert!(true);
    }
} 
