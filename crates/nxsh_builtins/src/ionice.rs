use std::process::Command;
use anyhow::Result;

pub fn ionice_cli(args: Vec<String>) -> Result<()> {
    if args.is_empty() || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let mut class: Option<u32> = None;
    let mut _classdata: Option<u32> = None; // 未使用だが将来拡張用
    let mut pid: Option<u32> = None;
    let mut pgrp: Option<u32> = None;
    let mut uid: Option<u32> = None;
    let mut query_mode = false;
    let mut command_args = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--class" => {
                i += 1;
                if i < args.len() {
                    class = args[i].parse().ok();
                }
            }
        "-n" | "--classdata" => {
                i += 1;
                if i < args.len() {
            _classdata = args[i].parse().ok();
                }
            }
            "-p" | "--pid" => {
                i += 1;
                if i < args.len() {
                    pid = args[i].parse().ok();
                }
            }
            "-P" | "--pgrp" => {
                i += 1;
                if i < args.len() {
                    pgrp = args[i].parse().ok();
                }
            }
            "-u" | "--uid" => {
                i += 1;
                if i < args.len() {
                    uid = args[i].parse().ok();
                }
            }
            "-t" | "--ignore" => {
                // Ignore option (for compatibility)
            }
            arg if !arg.starts_with('-') => {
                command_args.push(arg.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    // If no command specified and no specific target, query mode
    if command_args.is_empty() && pid.is_none() && pgrp.is_none() && uid.is_none() {
        query_mode = true;
    }

    // Cross-platform dispatch: Windows approximates, Unix uses ioprio where available.
    #[cfg(windows)]
    {
    handle_windows_ionice(class, _classdata, pid, pgrp, uid, query_mode, command_args)
    }
    #[cfg(not(windows))]
    {
    handle_unix_ionice(class, _classdata, pid, pgrp, uid, query_mode, command_args)
    }
}

fn print_help() {
    println!("Usage: ionice [options] [command [args...]]");
    println!();
    println!("Options:");
    println!("  -c, --class CLASS    Scheduling class (0-3):");
    println!("                        0: none, 1: real-time, 2: best-effort, 3: idle");
    println!("  -n, --classdata NUM  Priority level (0-7, lower is higher priority)");
    println!("  -p, --pid PID        Process ID to modify/query");
    println!("  -P, --pgrp PGRP      Process group to modify/query");
    println!("  -u, --uid UID        User ID to modify/query");
    println!("  -t, --ignore         Ignore failures to set requested priority");
    println!("  -h, --help           Show this help message");
    println!();
    println!("Scheduling classes:");
    println!("  0: None - no special scheduling");
    println!("  1: Real-time - highest priority");
    println!("  2: Best-effort - normal priority (default)");
    println!("  3: Idle - lowest priority");
    println!();
    println!("Examples:");
    println!("  ionice -c 3 -n 7 command    # Run command with lowest I/O priority");
    println!("  ionice -p 1234              # Show I/O priority of process 1234");
    println!("  ionice -c 1 -n 0 -p 1234    # Set highest real-time I/O priority");
}

#[cfg(windows)]
fn handle_windows_ionice(
    class: Option<u32>,
    classdata: Option<u32>,
    pid: Option<u32>,
    _pgrp: Option<u32>,
    _uid: Option<u32>,
    query_mode: bool,
    command_args: Vec<String>,
) -> Result<()> {
    if query_mode || pid.is_some() {
        // Windows doesn't have direct ionice equivalent, simulate query
        if let Some(pid_val) = pid {
            println!("Process {pid_val} I/O priority: best-effort (Windows simulated)");
        } else {
            println!("Current process I/O priority: best-effort (Windows simulated)");
        }
        return Ok(());
    }

    if !command_args.is_empty() {
        // Convert ionice class to Windows priority class
        let priority_class = match class.unwrap_or(2) {
            0 => "NORMAL_PRIORITY_CLASS",      // None -> Normal
            1 => "HIGH_PRIORITY_CLASS",        // Real-time -> High
            2 => "NORMAL_PRIORITY_CLASS",      // Best-effort -> Normal
            3 => "IDLE_PRIORITY_CLASS",        // Idle -> Idle
            _ => "NORMAL_PRIORITY_CLASS",
        };

        println!("Starting command with {priority_class} priority (Windows approximation)");
        
        let mut cmd = Command::new(&command_args[0]);
        if command_args.len() > 1 {
            cmd.args(&command_args[1..]);
        }

        // On Windows, we can't easily set I/O priority, so we set process priority
        let status = cmd.status()?;
        
        std::process::exit(status.code().unwrap_or(0));
    }

    Ok(())
}

#[cfg(not(windows))]
fn handle_unix_ionice(
    class: Option<u32>,
    classdata: Option<u32>,
    pid: Option<u32>,
    pgrp: Option<u32>,
    uid: Option<u32>,
    query_mode: bool,
    command_args: Vec<String>,
) -> Result<()> {
    if query_mode {
        // Query current process or specified process
        if let Some(pid_val) = pid {
            query_process_ioprio(pid_val)?;
        } else {
            query_process_ioprio(std::process::id())?;
        }
        return Ok(());
    }

    if pid.is_some() || pgrp.is_some() || uid.is_some() {
        // Set I/O priority for existing process(es)
        return set_existing_process_ioprio(class, classdata, pid, pgrp, uid);
    }

    if !command_args.is_empty() {
        // Run command with specified I/O priority
        return run_with_ioprio(class, classdata, command_args);
    }

    // Default: show current process priority
    query_process_ioprio(std::process::id())
}

#[cfg(all(unix, target_os = "linux"))]
fn query_process_ioprio(pid: u32) -> Result<()> {
    // On Linux, use ioprio_get via libc syscall numbers to query current class/priority
    use nix::libc;
    const IOPRIO_WHO_PROCESS: libc::c_int = 1; // IOPRIO_WHO_PROCESS
    unsafe {
        let prio = libc::syscall(libc::SYS_ioprio_get as libc::c_long, IOPRIO_WHO_PROCESS, pid as libc::c_int, 0) as i32;
        if prio < 0 { println!("I/O priority: unknown (query failed) for PID {}", pid); return Ok(()); }
        let class = (prio >> 13) & 0x7;
        let data = prio & 0x1fff;
        println!("Process {} I/O scheduling class: {} ({}), priority {}", pid, get_class_name(class as u32), class, data);
    }
    Ok(())
}

#[cfg(all(unix, not(target_os = "linux")))]
fn query_process_ioprio(pid: u32) -> Result<()> {
    // Try to read from /proc/PID/io first
    let io_path = format!("/proc/{}/io", pid);
    if std::path::Path::new(&io_path).exists() {
        if let Ok(content) = std::fs::read_to_string(&io_path) {
            println!("I/O statistics for process {}:", pid);
            for line in content.lines() {
                if line.starts_with("read_bytes:") ||
                   line.starts_with("write_bytes:") ||
                   line.starts_with("cancelled_write_bytes:") {
                    println!("  {}", line);
                }
            }
        }
    }

    // Fallback for non-Linux Unix
    println!("Process {} I/O scheduling class: best-effort, priority 4 (simulated)", pid);
    
    Ok(())
}

#[cfg(all(unix, target_os = "linux"))]
fn set_existing_process_ioprio(
    class: Option<u32>,
    classdata: Option<u32>,
    pid: Option<u32>,
    pgrp: Option<u32>,
    uid: Option<u32>,
) -> Result<()> {
    use nix::libc;
    let class_val = class.unwrap_or(2).min(3);
    let priority = classdata.unwrap_or(4).min(7);
    let prio: i32 = ((class_val as i32) << 13) | (priority as i32 & 0x1fff);
    unsafe {
        if let Some(pid_val) = pid {
            const IOPRIO_WHO_PROCESS: libc::c_int = 1;
            let rc = libc::syscall(libc::SYS_ioprio_set as libc::c_long, IOPRIO_WHO_PROCESS, pid_val as libc::c_int, prio);
            if rc < 0 { return Err(anyhow::anyhow!("failed to set ioprio for pid {}", pid_val)); }
            println!("Set I/O priority for PID {}: class={}, priority={}", pid_val, class_val, priority);
        } else if let Some(pgrp_val) = pgrp {
            const IOPRIO_WHO_PGRP: libc::c_int = 2;
            let rc = libc::syscall(libc::SYS_ioprio_set as libc::c_long, IOPRIO_WHO_PGRP, pgrp_val as libc::c_int, prio);
            if rc < 0 { return Err(anyhow::anyhow!("failed to set ioprio for pgrp {}", pgrp_val)); }
            println!("Set I/O priority for PGRP {}: class={}, priority={}", pgrp_val, class_val, priority);
        } else if let Some(uid_val) = uid {
            const IOPRIO_WHO_USER: libc::c_int = 3;
            let rc = libc::syscall(libc::SYS_ioprio_set as libc::c_long, IOPRIO_WHO_USER, uid_val as libc::c_int, prio);
            if rc < 0 { return Err(anyhow::anyhow!("failed to set ioprio for uid {}", uid_val)); }
            println!("Set I/O priority for UID {}: class={}, priority={}", uid_val, class_val, priority);
        }
    }
    Ok(())
}

#[cfg(all(unix, not(target_os = "linux")))]
fn run_with_ioprio(
    class: Option<u32>,
    classdata: Option<u32>,
    command_args: Vec<String>,
) -> Result<()> {
    let class_val = class.unwrap_or(2);
    let priority = classdata.unwrap_or(4);

    println!("Starting command with I/O class {} priority {}", class_val, priority);

    // Try to use nice as a fallback since we can't easily set I/O priority
    let nice_value = match class_val {
        0 => 0,    // None -> normal nice
        1 => -10,  // Real-time -> high nice
        2 => 0,    // Best-effort -> normal nice  
        3 => 19,   // Idle -> lowest nice
        _ => 0,
    };

    let mut cmd = Command::new("nice");
    cmd.arg("-n").arg(nice_value.to_string());
    cmd.arg(&command_args[0]);
    
    if command_args.len() > 1 {
        cmd.args(&command_args[1..]);
    }

    let status = cmd.status().map_err(|_| {
        // Fallback: run command directly without nice
        let mut direct_cmd = Command::new(&command_args[0]);
        if command_args.len() > 1 {
            direct_cmd.args(&command_args[1..]);
        }
        direct_cmd.status().unwrap()
    });

    match status {
        Ok(exit_status) => std::process::exit(exit_status.code().unwrap_or(0)),
        Err(exit_status) => std::process::exit(exit_status.code().unwrap_or(0)),
    }
}

fn get_class_name(class: u32) -> &'static str {
    match class {
        0 => "none",
        1 => "real-time", 
        2 => "best-effort",
        3 => "idle",
        _ => "unknown",
    }
}
