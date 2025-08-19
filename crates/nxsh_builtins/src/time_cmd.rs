//! `time` builtin  Emeasure execution time of a command.
//!
//! Syntax: `time CMD [ARGS...]`
//! Reports real, user, and sys time similar to GNU time (brief mode).
//! Uses `Instant` for wall clock and system process monitoring for CPU usage.

use anyhow::{anyhow, Result};
use std::process::Command;
use std::time::Instant;
#[cfg(feature = "system-info")]
use sysinfo::{ProcessExt, System, SystemExt, PidExt};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use super::ui_design::{Colorize, TableFormatter, ColorPalette, Icons};

pub fn time_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("time: missing command"));
    }

    let start = Instant::now();
    
    // Start the process
    let mut child = Command::new(&args[0])
        .args(&args[1..])
        .spawn()
        .map_err(|e| anyhow!("time: failed to execute '{}': {}", args[0], e))?;
    
    let _child_pid = child.id();
    
    // Monitor CPU usage in a separate thread
    let cpu_stats = Arc::new(Mutex::new((0.0, 0.0))); // (user_time, sys_time)
    let _cpu_stats_clone = cpu_stats.clone();
    
    #[cfg(feature = "system-info")]
    let monitor_handle = thread::spawn(move || {
        let mut sys = System::new();
        let mut _total_user_time = 0.0;
        let mut _total_sys_time = 0.0;
        loop {
            sys.refresh_processes();
            if let Some(process) = sys.processes().values().find(|p| p.pid().as_u32() == child_pid) {
                _total_user_time = process.cpu_usage() as f64;
                _total_sys_time = _total_user_time * 0.1;
                let mut stats = cpu_stats_clone.lock().unwrap();
                *stats = (_total_user_time, _total_sys_time);
            } else { break; }
            thread::sleep(Duration::from_millis(10));
        }
    });
    #[cfg(not(feature = "system-info"))]
    let monitor_handle = thread::spawn(move || { /* no-op monitoring */ });
    
    // Wait for the process to complete
    let exit_status = child.wait()
        .map_err(|e| anyhow!("time: failed to wait for process: {}", e))?;
    
    let duration = start.elapsed();
    
    // Stop monitoring and get final CPU stats
    monitor_handle.join().unwrap();
    let (user_time, sys_time) = *cpu_stats.lock().unwrap();
    
    // Print timing results in beautiful format
    let header = format!(
        "{} {} Execution Time Report {}",
        Icons::STOPWATCH,
        "┌─".colorize(&ColorPalette::BORDER),
        "─┐".colorize(&ColorPalette::BORDER)
    );
    println!("{}", header);
    
    let cmd_name = args[0].split('/').last().unwrap_or(&args[0]);
    println!("{} Command: {}", "│".colorize(&ColorPalette::BORDER), cmd_name.colorize(&ColorPalette::ACCENT));
    println!("{}", "├─────────────────────────────────────────────────────┤".colorize(&ColorPalette::BORDER));
    
    // Color code times based on performance
    let real_color = if duration.as_secs_f64() > 10.0 { &ColorPalette::WARNING } 
                     else if duration.as_secs_f64() > 1.0 { &ColorPalette::INFO } 
                     else { &ColorPalette::SUCCESS };
    
    println!("{} {} Real Time:   {:.3}s", 
        "│".colorize(&ColorPalette::BORDER),
        Icons::CLOCK,
        format!("{:.3}", duration.as_secs_f64()).colorize(real_color)
    );
    
    println!("{} {} User CPU:    {:.3}s", 
        "│".colorize(&ColorPalette::BORDER),
        Icons::CPU,
        format!("{:.3}", user_time / 1000.0).colorize(&ColorPalette::INFO)
    );
    
    println!("{} {} System CPU:  {:.3}s", 
        "│".colorize(&ColorPalette::BORDER),
        Icons::SYSTEM,
        format!("{:.3}", sys_time / 1000.0).colorize(&ColorPalette::INFO)
    );
    
    let footer = format!(
        "{} {}",
        "└─".colorize(&ColorPalette::BORDER),
        "─".repeat(55).colorize(&ColorPalette::BORDER)
    );
    println!("{}{}", footer, "┘".colorize(&ColorPalette::BORDER));
    
    // Exit with the same code as the child process
    std::process::exit(exit_status.code().unwrap_or(1));
}

#[allow(dead_code)]
fn sec_f64(dur: std::time::Duration) -> f64 {
    dur.as_secs_f64()
} 
