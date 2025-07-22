//! `fdisk` builtin – simple partition table viewer.
//!
//! Supported subcommands:
//!     fdisk -l            # list all block devices and their partitions
//!     fdisk DEVICE        # print partition table summary of DEVICE
//!
//! Only read-only functionality is provided for safety. Partition editing is
//! intentionally omitted because destructive operations require dedicated
//! privilege handling that is beyond the scope of a shell builtin.
//!
//! Platform support: Unix-like systems only. On other platforms the command
//! prints an informative message and exits successfully.

use anyhow::{anyhow, Result};
use std::path::PathBuf;

#[cfg(unix)]
use sysinfo::{System, SystemExt};

#[cfg(unix)]
use mbrman::MBR;

#[cfg(unix)]
pub async fn fdisk_cli(args: &[String]) -> Result<()> {
    // "fdisk -l" or no argument ⇒ list block devices
    if args.is_empty() || (args.len() == 1 && args[0] == "-l") {
        list_block_devices()?;
        return Ok(());
    }

    let device = &args[0];
    let path = PathBuf::from(device);
    if !path.exists() {
        return Err(anyhow!("fdisk: device not found: {}", device));
    }
    print_partition_table(&path)?;
    Ok(())
}

#[cfg(unix)]
fn list_block_devices() -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_disks_list();
    println!("{:<12} {:>10} {:>10}", "Device", "Size", "Mount");
    for disk in sys.disks() {
        let name = disk.name().to_string_lossy();
        let size_gb = disk.total_space() as f64 / 1_000_000_000.0;
        let mount = disk.mount_point().display();
        println!("{:<12} {:>7.1}G {:>10}", name, size_gb, mount);
    }
    Ok(())
}

#[cfg(unix)]
fn print_partition_table(dev: &PathBuf) -> Result<()> {
    use std::fs::File;
    use std::io::{Seek, SeekFrom};

    let mut file = File::open(dev)?;
    let size_bytes = file.seek(SeekFrom::End(0))?;
    file.seek(SeekFrom::Start(0))?;

    let sector_count = (size_bytes / 512) as u64;
    let mbr = MBR::read_from(&mut file, sector_count)
        .map_err(|e| anyhow!("fdisk: failed to read partition table: {e}"))?;

    println!("Disk: {}  Size: {:.1} GiB", dev.display(), size_bytes as f64 / 1_073_741_824.0);
    println!("Device       Boot  Start    Sectors   Id  Type");

    for (index, part) in mbr.partition_iter().enumerate() {
        let start = part.starting_lba();
        let sectors = part.sector_count();
        let id = part.partition_type().as_byte();
        let boot = if part.bootable() { "*" } else { " " };
        println!("{}{}  {} {:>10} {:>10}  {:02X}  {}",
                 dev.display(), index + 1, boot, start, sectors, id, part.partition_type());
    }
    Ok(())
}

#[cfg(not(unix))]
pub async fn fdisk_cli(_args: &[String]) -> Result<()> {
    println!("fdisk: unsupported on this platform");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_disks_stub() {
        let _ = fdisk_cli(&["-l".into()]).await;
    }
} 