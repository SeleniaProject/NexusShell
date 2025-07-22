//! `ps` command – process list (equivalent to simplified `ps aux`).
//!
//! Supported flags: none (future: -e, -o). Always displays all processes.
//! Columns: PID USER CPU% MEM% COMMAND
//! CPU% and MEM% are instantaneous values using sysinfo crate.

use anyhow::Result;
use sysinfo::{ProcessExt, System, SystemExt, UserExt};
use humansize::{FileSize, file_size_opts as opts};

pub fn ps_cli(_args: &[String]) -> Result<()> {
    let mut sys = System::new();
    sys.refresh_processes();
    sys.refresh_users_list();
    sys.refresh_memory();

    println!("{:<6} {:<10} {:>6} {:>6} {}", "PID", "USER", "CPU%", "MEM%", "COMMAND");
    let total_mem = sys.total_memory() as f32;

    for (pid, proc_) in sys.processes() {
        let user = proc_.user_id().and_then(|uid| sys.get_user_by_id(uid)).map(|u| u.name()).unwrap_or("?");
        let cpu = format!("{:.1}", proc_.cpu_usage());
        let mem_percent = if total_mem > 0.0 {
            format!("{:.1}", proc_.memory() as f32 * 100.0 / total_mem)
        } else {
            "0.0".to_string()
        };
        println!("{:<6} {:<10} {:>6} {:>6} {}", pid.as_u32(), user, cpu, mem_percent, proc_.name());
    }
    Ok(())
} 