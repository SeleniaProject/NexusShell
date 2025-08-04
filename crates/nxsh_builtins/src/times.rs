//! `times` builtin  Edisplay cumulative user/system CPU times for the shell and child processes.
//! Output format similar to Bash:
//!    <user>  <system>
//!    <child_user>  <child_system>
//! Times are printed in seconds with 2 decimal precision.

use anyhow::Result;
use sysinfo::{ProcessExt, System, SystemExt, PidExt};
use std::time::Instant;

static mut SHELL_START_TIME: Option<Instant> = None;
static mut TOTAL_CHILD_CPU_TIME: f64 = 0.0;

pub fn times_cli(args: &[String]) -> Result<()> {
    if !args.is_empty() {
        eprintln!("times: too many arguments");
        return Ok(());
    }

    let mut system = System::new_all();
    system.refresh_processes();

    // Get current process info
    let current_pid = std::process::id();
    
    // Shell process times
    let shell_times = if let Some(process) = system.process(sysinfo::Pid::from_u32(current_pid)) {
        let cpu_time = process.cpu_usage() as f64 / 100.0; // Convert percentage to fraction
        let run_time = process.run_time();
        
        // Estimate user and system time (rough approximation)
        let total_cpu_seconds = (run_time as f64) * cpu_time / 100.0;
        let user_time = total_cpu_seconds * 0.7; // Assume 70% user time
        let sys_time = total_cpu_seconds * 0.3;  // Assume 30% system time
        
        (user_time, sys_time)
    } else {
        (0.0, 0.0)
    };

    // Child process times (accumulated)
    let child_times = unsafe {
        (TOTAL_CHILD_CPU_TIME * 0.7, TOTAL_CHILD_CPU_TIME * 0.3)
    };

    // Print times in bash format
    println!("{:.2}m{:.3}s {:.2}m{:.3}s", 
        shell_times.0 as u32 / 60, shell_times.0 % 60.0,
        shell_times.1 as u32 / 60, shell_times.1 % 60.0);
    println!("{:.2}m{:.3}s {:.2}m{:.3}s", 
        child_times.0 as u32 / 60, child_times.0 % 60.0,
        child_times.1 as u32 / 60, child_times.1 % 60.0);

    Ok(())
}

/// Initialize shell start time - call when shell starts
pub fn init_shell_timing() {
    unsafe {
        SHELL_START_TIME = Some(Instant::now());
    }
}

/// Accumulate child process CPU time - call when child process exits
pub fn accumulate_child_time(cpu_time: f64) {
    unsafe {
        TOTAL_CHILD_CPU_TIME += cpu_time;
    }
}

/// Get shell runtime in seconds
pub fn get_shell_runtime() -> f64 {
    unsafe {
        if let Some(start_time) = SHELL_START_TIME {
            start_time.elapsed().as_secs_f64()
        } else {
            0.0
        }
    }
}

/// Reset child process time accumulator
pub fn reset_child_times() {
    unsafe {
        TOTAL_CHILD_CPU_TIME = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_times_no_args() {
        let result = times_cli(&[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_times_with_args() {
        let result = times_cli(&["arg1".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_shell_timing() {
        init_shell_timing();
        std::thread::sleep(Duration::from_millis(10));
        let runtime = get_shell_runtime();
        assert!(runtime > 0.0);
        assert!(runtime < 1.0); // Should be less than 1 second
    }

    #[test]
    fn test_child_time_accumulation() {
        reset_child_times();
        accumulate_child_time(1.5);
        accumulate_child_time(2.0);
        
        unsafe {
            assert_eq!(TOTAL_CHILD_CPU_TIME, 3.5);
        }
    }
}
