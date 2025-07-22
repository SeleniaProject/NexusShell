//! `free` builtin — display memory usage.
//!
//! Supports `-h` flag for human-readable units (MiB/GiB).
//! Output columns: total, used, free, avail.

use anyhow::Result;
use sysinfo::{System, SystemExt};

pub fn free_cli(args: &[String]) -> Result<()> {
    let human = args.get(0).map_or(false, |s| s == "-h" || s == "--human-readable");

    let mut sys = System::new();
    sys.refresh_memory();

    let total = sys.total_memory();
    let free = sys.free_memory();
    let avail = sys.available_memory();
    let used = total - free;

    println!("              total        used        free      avail");
    if human {
        println!(
            "Mem:  {:>8}  {:>8}  {:>8}  {:>8}",
            human_bytes(total),
            human_bytes(used),
            human_bytes(free),
            human_bytes(avail)
        );
    } else {
        println!(
            "Mem: {:>12} {:>12} {:>12} {:>12}",
            total,
            used,
            free,
            avail
        );
    }

    Ok(())
}

fn human_bytes(kib: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if kib >= GB {
        format!("{:.1}Gi", kib as f64 / GB as f64)
    } else if kib >= MB {
        format!("{:.1}Mi", kib as f64 / MB as f64)
    } else if kib >= KB {
        format!("{:.1}Ki", kib as f64 / KB as f64)
    } else {
        format!("{}B", kib)
    }
} 