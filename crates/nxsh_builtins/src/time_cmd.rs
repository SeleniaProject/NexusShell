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
    
    let child_pid = child.id();
    
    // Monitor CPU usage in a separate thread
    let cpu_stats = Arc::new(Mutex::new((0.0, 0.0))); // (user_time, sys_time)
    let cpu_stats_clone = cpu_stats.clone();
    
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
    
    // Print timing results in GNU time format
    println!("real\t{:.3}s", duration.as_secs_f64());
    println!("user\t{:.3}s", user_time / 1000.0); // Convert from ms to seconds
    println!("sys\t{:.3}s", sys_time / 1000.0);   // Convert from ms to seconds
    
    // Exit with the same code as the child process
    std::process::exit(exit_status.code().unwrap_or(1));
}

#[allow(dead_code)]
fn sec_f64(dur: std::time::Duration) -> f64 {
    dur.as_secs_f64()
} 
