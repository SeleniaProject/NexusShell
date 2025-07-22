//! `vmstat` builtin — report virtual memory statistics.
//!
//! Simplified output inspired by Linux `vmstat`.
//! Currently prints a single snapshot with columns:
//!   free available swap_free swap_used cpu%.
//! Future work: support periodic output and additional metrics (io, system, procs).

use anyhow::Result;
use sysinfo::{CpuExt, System, SystemExt};

pub fn vmstat_cli(_args: &[String]) -> Result<()> {
    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_cpu();
    sys.refresh_components();

    let total = sys.total_memory();
    let free = sys.free_memory();
    let avail = sys.available_memory();
    let swap_total = sys.total_swap();
    let swap_free = sys.free_swap();
    let swap_used = swap_total - swap_free;
    let cpu = sys.global_cpu_info().cpu_usage();

    println!("    memory (KiB)                swap (KiB)     cpu");
    println!("   free  avail   used      free   used     %util");
    println!("{:>8} {:>8} {:>8} {:>10} {:>8} {:>8.1}",
        free,
        avail,
        total - free,
        swap_free,
        swap_used,
        cpu);
    Ok(())
} 