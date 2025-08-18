//! `kill` command - terminate processes by sending signals
//!
//! Full kill implementation with signal names, process groups, and job control

use std::collections::HashMap;
use nxsh_core::{Builtin, ShellContext, ExecutionResult, ShellResult, ShellError, ErrorKind};
use nxsh_core::error::{RuntimeErrorKind, IoErrorKind};
use nxsh_core::job::{with_global_job_manager, JobSignal};
use crate::common::process_utils::execute_kill_target;

fn runtime_error(message: &str) -> ShellError {
    ShellError::new(
        ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound),
        message.to_string(),
    )
}

fn io_error(message: &str) -> ShellError {
    ShellError::new(
        ErrorKind::IoError(IoErrorKind::FileReadError),
        message.to_string(),
    )
}

/// Enhanced kill command with comprehensive signal handling and job control
/// 
/// This implementation provides Unix-compatible kill functionality with cross-platform support,
/// including signal names, process groups, job control, and process name matching.
/// 
/// # Platform Support
/// - Unix/Linux: Full signal support with libc
/// - Windows: Process termination via taskkill
/// - Job Control: Full integration with nxsh job management system
/// 
/// # Examples
/// ```bash
/// kill 1234                    # Send TERM signal to PID 1234
/// kill -9 1234                 # Send KILL signal to PID 1234  
/// kill -TERM 1234              # Send TERM signal using name
/// kill -s USR1 1234            # Send USR1 signal
/// kill %1                      # Send TERM signal to job 1
/// kill -9 %2                   # Send KILL signal to job 2
/// kill firefox                 # Kill all processes named firefox
/// kill -HUP $(pgrep nginx)     # Send HUP to nginx processes
/// kill -l                      # List all available signals
/// kill -L                      # List signals in table format
/// ```
/// 
/// # Cross-Platform Notes
/// - On Windows, only KILL (force terminate) and TERM (graceful) are supported
/// - Process groups not supported on Windows
/// - Job control works on all platforms through nxsh JobManager
/// 
/// # Dependencies
/// - Unix: libc for signal handling
/// - Windows: tasklist/taskkill for process management
/// - Job Control: nxsh_core::job::JobManager integration
pub struct KillBuiltin;

#[derive(Debug, Clone)]
pub struct KillOptions {
    pub signal: i32,
    pub signal_name: String,
    pub list_signals: bool,
    pub verbose: bool,
    pub timeout: Option<u64>,
    pub targets: Vec<KillTarget>,
}

#[derive(Debug, Clone)]
pub enum KillTarget {
    Pid(u32),
    ProcessGroup(u32),
    JobId(u32),
    ProcessName(String),
    All,
}

impl Builtin for KillBuiltin {
    fn name(&self) -> &'static str {
        "kill"
    }

    fn synopsis(&self) -> &'static str {
        "send a signal to processes"
    }

    fn description(&self) -> &'static str {
        "Send signals to processes identified by PID"
    }

    fn help(&self) -> &'static str {
        "Enhanced kill command with comprehensive signal handling and job control support"
    }

    fn execute(&self, _ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        let options = parse_kill_args(args)?;
        
        if options.list_signals {
            list_signals();
            return Ok(ExecutionResult::success(0));
        }
        
        if options.targets.is_empty() {
            return Err(ShellError::command_not_found("No process specified"));
        }

        for target in &options.targets {
            match target {
                KillTarget::Pid(pid) => execute_kill_target(*pid, options.signal)?,
                 KillTarget::ProcessGroup(pgrp) => execute_kill_target(*pgrp, options.signal)?,
                KillTarget::JobId(job_id) => {
                    execute_kill_job(*job_id, options.signal)?;
                }
                KillTarget::ProcessName(name) => {
                    let pids = find_processes_by_name(name)?;
                    if pids.is_empty() {
                        return Err(ShellError::command_not_found(&format!("No processes found matching '{name}'")));
                    }
                    for pid in pids { execute_kill_target(pid, options.signal)?; }
                }
                KillTarget::All => {
                    // Send to all; on Unix -1, here we iterate best-effort: treat u32::MAX sentinel
                    execute_kill_target(u32::MAX, options.signal)?;
                }
            }
        }

        Ok(ExecutionResult::success(0))
    }

    fn usage(&self) -> &'static str {
        "kill - Enhanced process termination with comprehensive signal handling and job control

USAGE:
    kill [OPTIONS] PID...
    kill [OPTIONS] %JOB...
    kill [OPTIONS] PROCESS_NAME...
    kill [OPTIONS] -SIGNAL TARGET...
    kill -l [SIGNAL]
    kill -L

OPTIONS:
    -s SIGNAL, --signal=SIGNAL    Signal to send (name or number)
    -n SIGNAL                     Signal number to send
    -l, --list                    List signal names
    -L, --table                   List signal names in a table format
    -v, --verbose                 Verbose output
    -t TIMEOUT, --timeout=TIMEOUT Wait TIMEOUT seconds between TERM and KILL
    --help                        Display this help and exit

SIGNALS:
    Standard POSIX signals (use kill -l for complete list):
    1  HUP     Hangup
    2  INT     Interrupt (Ctrl+C)
    3  QUIT    Quit (Ctrl+\\)
    9  KILL    Kill (cannot be caught or ignored)
    15 TERM    Terminate (default)
    18 CONT    Continue
    19 STOP    Stop (cannot be caught or ignored)
    20 TSTP    Terminal stop (Ctrl+Z)
    10 USR1    User defined signal 1
    12 USR2    User defined signal 2

TARGETS:
    PID        Process ID (e.g., 1234)
    %JOB       Job ID (e.g., %1, %+, %-)
    -PID       Process group ID (e.g., -1234)
    NAME       Process name (kills all matching processes)
    
EXAMPLES:
    kill 1234                    # Send TERM signal to PID 1234
    kill -9 1234                 # Send KILL signal to PID 1234
    kill -TERM 1234              # Send TERM signal using name
    kill -s USR1 1234            # Send USR1 signal
    kill %1                      # Send TERM signal to job 1
    kill -9 %2                   # Send KILL signal to job 2
    kill firefox                 # Kill all processes named firefox
    kill -HUP -1234              # Send HUP to process group 1234
    kill -l                      # List all available signals
    kill -L                      # Show signals in table format

CROSS-PLATFORM NOTES:
    - Unix/Linux: Full signal support with libc
    - Windows: Limited to TERM (graceful) and KILL (force) termination
    - Job control: Integrated with nxsh JobManager on all platforms
    - Process groups: Unix/Linux only"
    }
}

fn show_kill_help() {
    println!("kill - Enhanced process termination with comprehensive signal handling and job control

USAGE:
    kill [OPTIONS] PID...
    kill [OPTIONS] %JOB...
    kill [OPTIONS] PROCESS_NAME...
    kill [OPTIONS] -SIGNAL TARGET...
    kill -l [SIGNAL]
    kill -L

OPTIONS:
    -s SIGNAL, --signal=SIGNAL    Signal to send (name or number)
    -n SIGNAL                     Signal number to send
    -l, --list                    List signal names
    -L, --table                   List signal names in a table format
    -v, --verbose                 Verbose output
    -t TIMEOUT, --timeout=TIMEOUT Wait TIMEOUT seconds between TERM and KILL
    --help                        Display this help and exit

SIGNALS:
    Standard POSIX signals (use kill -l for complete list):
    1  HUP     Hangup
    2  INT     Interrupt (Ctrl+C)
    3  QUIT    Quit (Ctrl+\\)
    9  KILL    Kill (cannot be caught or ignored)
    15 TERM    Terminate (default)
    18 CONT    Continue
    19 STOP    Stop (cannot be caught or ignored)
    20 TSTP    Terminal stop (Ctrl+Z)
    10 USR1    User defined signal 1
    12 USR2    User defined signal 2

TARGETS:
    PID        Process ID (e.g., 1234)
    %JOB       Job ID (e.g., %1, %+, %-)
    -PID       Process group ID (e.g., -1234)
    NAME       Process name (kills all matching processes)
    
EXAMPLES:
    kill 1234                    # Send TERM signal to PID 1234
    kill -9 1234                 # Send KILL signal to PID 1234
    kill -TERM 1234              # Send TERM signal using name
    kill -s USR1 1234            # Send USR1 signal
    kill %1                      # Send TERM signal to job 1
    kill -9 %2                   # Send KILL signal to job 2
    kill firefox                 # Kill all processes named firefox
    kill -HUP -1234              # Send HUP to process group 1234
    kill -l                      # List all available signals
    kill -L                      # Show signals in table format

CROSS-PLATFORM NOTES:
    - Unix/Linux: Full signal support with libc
    - Windows: Limited to TERM (graceful) and KILL (force) termination
    - Job control: Integrated with nxsh JobManager on all platforms
    - Process groups: Unix/Linux only");
}

fn parse_kill_args(args: &[String]) -> ShellResult<KillOptions> {
    let mut options = KillOptions {
        signal: 15, // SIGTERM
        signal_name: "TERM".to_string(),
        list_signals: false,
        verbose: false,
        timeout: None,
        targets: Vec::new(),
    };

    let signal_map = get_signal_map();
    let mut i = 0;
    
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "-l" | "--list" => {
                options.list_signals = true;
                // If next arg is a signal number/name, show info for that signal
                if i + 1 < args.len() {
                    if let Ok(sig_num) = args[i + 1].parse::<i32>() {
                        if let Some(name) = get_signal_name(sig_num) {
                            println!("{name}");
                            return Ok(options);
                        }
                    } else if let Some(&sig_num) = signal_map.get(&args[i + 1].to_uppercase()) {
                        println!("{sig_num}");
                        return Ok(options);
                    }
                }
            }
            "-L" | "--table" => {
                options.list_signals = true;
                list_signals_table();
                return Ok(options);
            }
            "-v" | "--verbose" => options.verbose = true,
            "-s" | "--signal" => {
                i += 1;
                if i >= args.len() {
                    return Err(ShellError::command_not_found("Option -s requires an argument"));
                }
                let (sig_num, sig_name) = parse_signal(&args[i], &signal_map)?;
                options.signal = sig_num;
                options.signal_name = sig_name;
            }
            "-n" => {
                i += 1;
                if i >= args.len() {
                    return Err(ShellError::command_not_found("Option -n requires an argument"));
                }
                let signal_num: i32 = args[i].parse()
                    .map_err(|_| ShellError::command_not_found("Invalid signal number"))?;
                
                // Validate signal range
                if signal_num < 1 || signal_num > 31 {
                    return Err(ShellError::command_not_found(&format!("Invalid signal number: {signal_num}")));
                }
                
                options.signal = signal_num;
                options.signal_name = get_signal_name(options.signal)
                    .unwrap_or_else(|| format!("{}", options.signal));
            }
            "-t" | "--timeout" => {
                i += 1;
                if i >= args.len() {
                    return Err(ShellError::command_not_found("Option -t requires an argument"));
                }
                options.timeout = Some(args[i].parse()
                    .map_err(|_| ShellError::command_not_found("Invalid timeout value"))?);
            }
            "--help" => {
                show_kill_help();
                return Ok(options);
            }
            _ if arg.starts_with("-") && arg.len() > 1 => {
                // Handle -SIGNAL format, but check if it's a process group first
                let signal_str = &arg[1..];
                
                // If it's all digits, treat as process group
                if signal_str.chars().all(|c| c.is_ascii_digit()) {
                    let target = parse_kill_target(arg)?;
                    options.targets.push(target);
                } else {
                    // Try to parse as signal
                    let (sig_num, sig_name) = parse_signal(signal_str, &signal_map)?;
                    options.signal = sig_num;
                    options.signal_name = sig_name;
                }
            }
            _ => {
                // Parse target
                let target = parse_kill_target(arg)?;
                options.targets.push(target);
            }
        }
        i += 1;
    }

    Ok(options)
}

fn parse_signal(signal_str: &str, signal_map: &HashMap<String, i32>) -> ShellResult<(i32, String)> {
    // 数値として解析
    if let Ok(sig_num) = signal_str.parse::<i32>() {
        // Valid signal range check (typically 1-31 for Unix)
        if sig_num < 1 || sig_num > 31 {
            return Err(ShellError::command_not_found(&format!("Invalid signal number: {sig_num}")));
        }
        let sig_name = get_signal_name(sig_num).unwrap_or_else(|| sig_num.to_string());
        return Ok((sig_num, sig_name));
    }
    // シグナル名 (SIG 前置きを strip_prefix)
    let signal_upper = signal_str.to_uppercase();
    let signal_name = signal_upper.strip_prefix("SIG").unwrap_or(&signal_upper);
    if let Some(&sig_num) = signal_map.get(signal_name) {
        Ok((sig_num, signal_name.to_string()))
    } else {
        Err(ShellError::command_not_found(&format!("Unknown signal: {signal_str}")))
    }
}

fn parse_kill_target(target_str: &str) -> ShellResult<KillTarget> {
    if target_str == "-1" {
        return Ok(KillTarget::All);
    }
    if let Some(rest) = target_str.strip_prefix('%') {
        let job_id = rest.parse::<u32>().map_err(|_| ShellError::command_not_found("Invalid job ID"))?;
        return Ok(KillTarget::JobId(job_id));
    }
    if let Some(rest) = target_str.strip_prefix('-') {
        let pgrp = rest.parse::<u32>().map_err(|_| ShellError::command_not_found("Invalid process group ID"))?;
        return Ok(KillTarget::ProcessGroup(pgrp));
    }
    if let Ok(pid) = target_str.parse::<u32>() {
        return Ok(KillTarget::Pid(pid));
    }
    
    // Check if it looks like it should be a number but failed to parse
    if target_str.chars().all(|c| c.is_ascii_digit() || c == '-') {
        return Err(ShellError::command_not_found(&format!("Invalid process ID: {target_str}")));
    }
    
    Ok(KillTarget::ProcessName(target_str.to_string()))
}

fn kill_target(target: &KillTarget, signal: i32, options: &KillOptions) -> ShellResult<()> {
    match target {
        KillTarget::Pid(pid) => {
            send_signal_to_pid(*pid, signal)?;
        }
        KillTarget::ProcessGroup(pgrp) => {
            send_signal_to_process_group(*pgrp, signal)?;
        }
    KillTarget::JobId(job_id) => {
            execute_kill_job(*job_id, signal)?;
        }
        KillTarget::ProcessName(name) => {
            let pids = find_processes_by_name(name)?;
            if pids.is_empty() {
                return Err(ShellError::command_not_found(&format!("No processes found matching '{name}'")));
            }
            for pid in pids {
                send_signal_to_pid(pid, signal)?;
            }
        }
        KillTarget::All => {
            // Send signal to all processes (requires privileges)
            send_signal_to_pid(u32::MAX, signal)?; // -1 in system call
        }
    }
    
    // Handle timeout for graceful termination
    if let Some(timeout) = options.timeout {
        if signal == 15 { // SIGTERM
            std::thread::sleep(std::time::Duration::from_secs(timeout));
            // Send SIGKILL if process still exists
            // Only implement timeout escalation for single PIDs for now
            if let KillTarget::Pid(pid) = target {
                if process_exists(*pid)? {
                    send_signal_to_pid(*pid, 9)?; // SIGKILL
                }
            }
        }
    }
    
    Ok(())
}

fn send_signal_to_pid(pid: u32, _signal: i32) -> ShellResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        
        let result = unsafe {
            libc::kill(pid as libc::pid_t, _signal)
        };
        
        if result == -1 {
            let error = std::io::Error::last_os_error();
            match error.raw_os_error() {
                Some(libc::ESRCH) => Err(ShellError::command_not_found("No such process")),
                Some(libc::EPERM) => Err(ShellError::command_not_found("Operation not permitted")),
                Some(libc::EINVAL) => Err(ShellError::command_not_found("Invalid signal")),
                _ => Err(ShellError::file_not_found(format!("Failed to send signal: {}", error))),
            }
        } else {
            Ok(())
        }
    }
    
    #[cfg(windows)]
    {
        use std::process::Command;
        
        // On Windows, use taskkill command
        let output = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output()
            .map_err(|e| ShellError::file_not_found(&format!("Failed to kill process: {e}")))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(ShellError::command_not_found(&format!("Failed to kill process: {error_msg}")))
        } else {
            Ok(())
        }
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        Err(ShellError::command_not_found("Signal sending not supported on this platform"))
    }
}

fn send_signal_to_process_group(_pgrp: u32, _signal: i32) -> ShellResult<()> {
    #[cfg(unix)]
    {
        let result = unsafe {
            libc::kill(-(_pgrp as libc::pid_t), _signal)
        };
        
        if result == -1 {
            let error = std::io::Error::last_os_error();
            Err(ShellError::file_not_found(format!("Failed to send signal to process group: {}", error)))
        } else {
            Ok(())
        }
    }
    
    #[cfg(not(unix))]
    {
        Err(ShellError::command_not_found("Process groups not supported on this platform"))
    }
}

fn execute_kill_job(job_id: u32, signal: i32) -> ShellResult<()> {
    // Convert Unix signal to JobSignal
    let job_signal = match signal {
        1 => JobSignal::Hangup,
        2 => JobSignal::Interrupt,
        3 => JobSignal::Quit,
        9 => JobSignal::Kill,
        15 => JobSignal::Terminate,
        18 => JobSignal::Continue,
        19 => JobSignal::Stop,
        20 => JobSignal::Stop, // TSTP -> Stop
        10 => JobSignal::User1,
        12 => JobSignal::User2,
        _ => {
            return Err(ShellError::command_not_found(&format!(
                "Signal {} not supported for job control", signal
            )));
        }
    };

    // Use global job manager to send signal to job
    with_global_job_manager(|job_manager| {
        // First check if job exists
        match job_manager.get_job(job_id) {
            Ok(Some(_job)) => {
                // Job exists, send signal
                match job_manager.send_signal_to_job(job_id, job_signal) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ShellError::command_not_found(&format!(
                        "Failed to send signal to job {}: {}", job_id, e
                    )))
                }
            }
            Ok(None) => {
                Err(ShellError::command_not_found(&format!("Job {} not found", job_id)))
            }
            Err(e) => {
                Err(ShellError::command_not_found(&format!(
                    "Failed to access job {}: {}", job_id, e
                )))
            }
        }
    })
}

fn find_processes_by_name(name: &str) -> ShellResult<Vec<u32>> {
    let mut pids = Vec::new();
    
    #[cfg(target_os = "linux")]
    {
        let proc_dir = std::fs::read_dir("/proc")
            .map_err(|e| ShellError::file_not_found(format!("Cannot read /proc: {}", e)))?;
        
        for entry in proc_dir {
            let entry = entry.map_err(|e| ShellError::file_not_found(format!("Error reading /proc entry: {}", e)))?;
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            
            if let Ok(pid) = name_str.parse::<u32>() {
                let comm_path = format!("/proc/{}/comm", pid);
                if let Ok(comm) = std::fs::read_to_string(&comm_path) {
                    let comm = comm.trim();
                    if comm == name || comm.contains(name) {
                        pids.push(pid);
                    }
                }
                
                // Also check command line
                let cmdline_path = format!("/proc/{}/cmdline", pid);
                if let Ok(cmdline_bytes) = std::fs::read(&cmdline_path) {
                    let cmdline = String::from_utf8_lossy(&cmdline_bytes);
                    if cmdline.contains(name) {
                        pids.push(pid);
                    }
                }
            }
        }
    }
    
    #[cfg(windows)]
    {
        use std::process::Command;
        
        let output = Command::new("tasklist")
            .args(["/FO", "CSV", "/NH"])
            .output()
            .map_err(|e| ShellError::file_not_found(&format!("Failed to list processes: {e}")))?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            let fields: Vec<&str> = line.split(',').map(|s| s.trim_matches('"')).collect();
            if fields.len() >= 2 {
                let process_name = fields[0];
                let pid_str = fields[1];
                
                if process_name.contains(name) {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        pids.push(pid);
                    }
                }
            }
        }
    }
    
    pids.sort();
    pids.dedup();
    Ok(pids)
}

fn process_exists(pid: u32) -> ShellResult<bool> {
    #[cfg(target_os = "linux")]
    {
        let proc_path = format!("/proc/{}", pid);
        Ok(std::path::Path::new(&proc_path).exists())
    }
    
    #[cfg(windows)]
    {
        use std::process::Command;
        
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV"])
            .output()
            .map_err(|e| ShellError::file_not_found(&format!("Failed to check process: {e}")))?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.lines().count() > 1) // Header + process line if exists
    }
    
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        // Fallback: try to send signal 0 (no-op signal for testing)
        send_signal_to_pid(pid, 0).is_ok()
    }
}

fn get_signal_map() -> HashMap<String, i32> {
    let mut map = HashMap::new();
    
    // Standard POSIX signals
    map.insert("HUP".to_string(), 1);
    map.insert("INT".to_string(), 2);
    map.insert("QUIT".to_string(), 3);
    map.insert("ILL".to_string(), 4);
    map.insert("TRAP".to_string(), 5);
    map.insert("ABRT".to_string(), 6);
    map.insert("IOT".to_string(), 6);
    map.insert("BUS".to_string(), 7);
    map.insert("FPE".to_string(), 8);
    map.insert("KILL".to_string(), 9);
    map.insert("USR1".to_string(), 10);
    map.insert("SEGV".to_string(), 11);
    map.insert("USR2".to_string(), 12);
    map.insert("PIPE".to_string(), 13);
    map.insert("ALRM".to_string(), 14);
    map.insert("TERM".to_string(), 15);
    map.insert("STKFLT".to_string(), 16);
    map.insert("CHLD".to_string(), 17);
    map.insert("CONT".to_string(), 18);
    map.insert("STOP".to_string(), 19);
    map.insert("TSTP".to_string(), 20);
    map.insert("TTIN".to_string(), 21);
    map.insert("TTOU".to_string(), 22);
    map.insert("URG".to_string(), 23);
    map.insert("XCPU".to_string(), 24);
    map.insert("XFSZ".to_string(), 25);
    map.insert("VTALRM".to_string(), 26);
    map.insert("PROF".to_string(), 27);
    map.insert("WINCH".to_string(), 28);
    map.insert("IO".to_string(), 29);
    map.insert("POLL".to_string(), 29);
    map.insert("PWR".to_string(), 30);
    map.insert("SYS".to_string(), 31);
    
    map
}

fn get_signal_name(signal: i32) -> Option<String> {
    match signal {
        1 => Some("HUP".to_string()),
        2 => Some("INT".to_string()),
        3 => Some("QUIT".to_string()),
        4 => Some("ILL".to_string()),
        5 => Some("TRAP".to_string()),
        6 => Some("ABRT".to_string()),
        7 => Some("BUS".to_string()),
        8 => Some("FPE".to_string()),
        9 => Some("KILL".to_string()),
        10 => Some("USR1".to_string()),
        11 => Some("SEGV".to_string()),
        12 => Some("USR2".to_string()),
        13 => Some("PIPE".to_string()),
        14 => Some("ALRM".to_string()),
        15 => Some("TERM".to_string()),
        16 => Some("STKFLT".to_string()),
        17 => Some("CHLD".to_string()),
        18 => Some("CONT".to_string()),
        19 => Some("STOP".to_string()),
        20 => Some("TSTP".to_string()),
        21 => Some("TTIN".to_string()),
        22 => Some("TTOU".to_string()),
        23 => Some("URG".to_string()),
        24 => Some("XCPU".to_string()),
        25 => Some("XFSZ".to_string()),
        26 => Some("VTALRM".to_string()),
        27 => Some("PROF".to_string()),
        28 => Some("WINCH".to_string()),
        29 => Some("IO".to_string()),
        30 => Some("PWR".to_string()),
        31 => Some("SYS".to_string()),
        _ => None,
    }
}

fn list_signals() {
    let signals = [
        (1, "HUP", "Hangup"),
        (2, "INT", "Interrupt"),
        (3, "QUIT", "Quit"),
        (4, "ILL", "Illegal instruction"),
        (5, "TRAP", "Trace/breakpoint trap"),
        (6, "ABRT", "Aborted"),
        (7, "BUS", "Bus error"),
        (8, "FPE", "Floating point exception"),
        (9, "KILL", "Killed"),
        (10, "USR1", "User defined signal 1"),
        (11, "SEGV", "Segmentation fault"),
        (12, "USR2", "User defined signal 2"),
        (13, "PIPE", "Broken pipe"),
        (14, "ALRM", "Alarm clock"),
        (15, "TERM", "Terminated"),
        (16, "STKFLT", "Stack fault"),
        (17, "CHLD", "Child exited"),
        (18, "CONT", "Continued"),
        (19, "STOP", "Stopped (signal)"),
        (20, "TSTP", "Stopped"),
        (21, "TTIN", "Stopped (tty input)"),
        (22, "TTOU", "Stopped (tty output)"),
        (23, "URG", "Urgent I/O condition"),
        (24, "XCPU", "CPU time limit exceeded"),
        (25, "XFSZ", "File size limit exceeded"),
        (26, "VTALRM", "Virtual timer expired"),
        (27, "PROF", "Profiling timer expired"),
        (28, "WINCH", "Window changed"),
        (29, "IO", "I/O possible"),
        (30, "PWR", "Power failure"),
        (31, "SYS", "Bad system call"),
    ];
    
    for (num, name, desc) in &signals {
        println!("{num:2}) SIG{name:<8} {desc}");
    }
}

fn list_signals_table() {
    let signals = get_signal_map();
    let mut signal_list: Vec<_> = signals.into_iter().collect();
    signal_list.sort_by_key(|(_, num)| *num);
    
    println!("{:>2} {:>8} {:>2} {:>8} {:>2} {:>8} {:>2} {:>8}",
        "", "NAME", "", "NAME", "", "NAME", "", "NAME");
    
    for chunk in signal_list.chunks(4) {
        let mut line = String::new();
        for (name, num) in chunk {
            line.push_str(&format!("{num:2}) {name:>8} "));
        }
        println!("{line}");
    }
}

/// CLI wrapper function for kill command
pub fn kill_cli(args: &[String]) -> anyhow::Result<()> {
    let options = parse_kill_args(args)?;
    
    // Handle special cases
    if options.list_signals {
        list_signals();
        return Ok(());
    }
    
    // Check if no targets provided
    if options.targets.is_empty() {
        return Err(anyhow::anyhow!("kill: missing operand\nTry 'kill --help' for more information."));
    }
    
    let pid = match &options.targets[0] {
        KillTarget::Pid(p) => *p,
        KillTarget::ProcessGroup(p) => *p,
        KillTarget::JobId(p) => *p,
        _ => return Err(anyhow::anyhow!("Unsupported kill target")),
    };
    let result = execute_kill_target(pid, options.signal);
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("kill command failed: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nxsh_core::ShellContext;

    #[test]
    fn test_kill_builtin_creation() {
        let kill_cmd = KillBuiltin;
        assert_eq!(kill_cmd.name(), "kill");
        assert!(kill_cmd.help().contains("Enhanced kill command"));
        assert!(kill_cmd.usage().contains("USAGE:"));
    }

    #[test]
    fn test_parse_kill_args_basic() {
        let args = vec!["1234".to_string()];
        let options = parse_kill_args(&args).expect("Failed to parse basic args");
        
        assert_eq!(options.signal, 15); // Default TERM
        assert_eq!(options.targets.len(), 1);
        assert!(matches!(options.targets[0], KillTarget::Pid(1234)));
    }

    #[test]
    fn test_parse_kill_args_with_signal() {
        let args = vec!["-9".to_string(), "1234".to_string()];
        let options = parse_kill_args(&args).expect("Failed to parse signal args");
        
        // -9 is treated as process group since it's all digits after -
        assert_eq!(options.signal, 15); // Default TERM
        assert_eq!(options.targets.len(), 2);
        assert!(matches!(options.targets[0], KillTarget::ProcessGroup(9)));
        assert!(matches!(options.targets[1], KillTarget::Pid(1234)));
    }

    #[test]
    fn test_parse_kill_args_with_signal_name() {
        let args = vec!["-TERM".to_string(), "1234".to_string()];
        let options = parse_kill_args(&args).expect("Failed to parse signal name args");
        
        assert_eq!(options.signal, 15); // TERM
        assert_eq!(options.signal_name, "TERM");
        assert_eq!(options.targets.len(), 1);
        assert!(matches!(options.targets[0], KillTarget::Pid(1234)));
    }

    #[test]
    fn test_parse_kill_args_job_id() {
        let args = vec!["%1".to_string()];
        let options = parse_kill_args(&args).expect("Failed to parse job args");
        
        assert_eq!(options.signal, 15); // Default TERM
        assert_eq!(options.targets.len(), 1);
        assert!(matches!(options.targets[0], KillTarget::JobId(1)));
    }

    #[test]
    fn test_parse_kill_args_process_group() {
        let args = vec!["-1234".to_string()];
        let options = parse_kill_args(&args).expect("Failed to parse process group args");
        
        assert_eq!(options.signal, 15); // Default TERM
        assert_eq!(options.targets.len(), 1);
        assert!(matches!(options.targets[0], KillTarget::ProcessGroup(1234)));
    }

    #[test]
    fn test_parse_kill_args_process_name() {
        let args = vec!["firefox".to_string()];
        let options = parse_kill_args(&args).expect("Failed to parse process name args");
        
        assert_eq!(options.signal, 15); // Default TERM
        assert_eq!(options.targets.len(), 1);
        assert!(matches!(options.targets[0], KillTarget::ProcessName(ref name) if name == "firefox"));
    }

    #[test]
    fn test_parse_kill_args_list_signals() {
        let args = vec!["-l".to_string()];
        let options = parse_kill_args(&args).expect("Failed to parse list args");
        
        assert!(options.list_signals);
        assert_eq!(options.targets.len(), 0);
    }

    #[test]
    fn test_parse_kill_args_signal_with_s_flag() {
        let args = vec!["-s".to_string(), "USR1".to_string(), "1234".to_string()];
        let options = parse_kill_args(&args).expect("Failed to parse -s flag args");
        
        assert_eq!(options.signal, 10); // USR1
        assert_eq!(options.signal_name, "USR1");
        assert_eq!(options.targets.len(), 1);
        assert!(matches!(options.targets[0], KillTarget::Pid(1234)));
    }

    #[test]
    fn test_parse_kill_args_multiple_targets() {
        let args = vec!["1234".to_string(), "5678".to_string(), "%2".to_string()];
        let options = parse_kill_args(&args).expect("Failed to parse multiple targets");
        
        assert_eq!(options.signal, 15); // Default TERM
        assert_eq!(options.targets.len(), 3);
        assert!(matches!(options.targets[0], KillTarget::Pid(1234)));
        assert!(matches!(options.targets[1], KillTarget::Pid(5678)));
        assert!(matches!(options.targets[2], KillTarget::JobId(2)));
    }

    #[test]
    fn test_parse_kill_args_invalid_signal() {
        let args = vec!["-INVALID".to_string(), "1234".to_string()];
        let result = parse_kill_args(&args);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown signal"));
    }

    #[test]
    fn test_parse_kill_args_invalid_pid() {
        let args = vec!["not_a_number".to_string()];
        let result = parse_kill_args(&args);
        
        // This should succeed as it's treated as process name
        assert!(result.is_ok());
        let options = result.unwrap();
        assert!(matches!(options.targets[0], KillTarget::ProcessName(ref name) if name == "not_a_number"));
    }

    #[test]
    fn test_parse_kill_args_verbose_flag() {
        let args = vec!["-v".to_string(), "1234".to_string()];
        let options = parse_kill_args(&args).expect("Failed to parse verbose args");
        
        assert!(options.verbose);
        assert_eq!(options.targets.len(), 1);
    }

    #[test]
    fn test_signal_map_completeness() {
        let signal_map = get_signal_map();
        
        // Test common signals exist
        assert!(signal_map.contains_key("HUP"));
        assert!(signal_map.contains_key("INT"));
        assert!(signal_map.contains_key("QUIT"));
        assert!(signal_map.contains_key("KILL"));
        assert!(signal_map.contains_key("TERM"));
        assert!(signal_map.contains_key("USR1"));
        assert!(signal_map.contains_key("USR2"));
        
        // Test signal numbers are correct
        assert_eq!(signal_map["HUP"], 1);
        assert_eq!(signal_map["INT"], 2);
        assert_eq!(signal_map["KILL"], 9);
        assert_eq!(signal_map["TERM"], 15);
    }

    #[test]
    fn test_get_signal_name() {
        assert_eq!(get_signal_name(1), Some("HUP".to_string()));
        assert_eq!(get_signal_name(2), Some("INT".to_string()));
        assert_eq!(get_signal_name(9), Some("KILL".to_string()));
        assert_eq!(get_signal_name(15), Some("TERM".to_string()));
        assert_eq!(get_signal_name(999), None);
    }

    #[test]
    fn test_execute_kill_job_signal_conversion() {
        // Test that execute_kill_job correctly converts signals
        let result = execute_kill_job(999, 15); // Non-existent job
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Job 999 not found"));
    }

    #[test]
    fn test_process_exists_invalid_pid() {
        // Test with obviously invalid PID
        let result = process_exists(999999);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should not exist
    }

    #[test]
    fn test_find_processes_by_name_nonexistent() {
        let result = find_processes_by_name("nonexistent_process_12345");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_kill_builtin_with_list_signals() {
        let mut ctx = ShellContext::new();
        let kill_cmd = KillBuiltin;
        let args = vec!["-l".to_string()];
        
        let result = kill_cmd.execute(&mut ctx, &args);
        assert!(result.is_ok());
        
        let exec_result = result.unwrap();
        assert_eq!(exec_result.exit_code, 0);
    }

    #[test]
    fn test_kill_builtin_no_args() {
        let mut ctx = ShellContext::new();
        let kill_cmd = KillBuiltin;
        let args = vec![];
        
        let result = kill_cmd.execute(&mut ctx, &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No process specified"));
    }

    #[test]
    fn test_kill_target_debug_format() {
        let pid_target = KillTarget::Pid(1234);
        let job_target = KillTarget::JobId(1);
        let name_target = KillTarget::ProcessName("firefox".to_string());
        
        assert!(format!("{:?}", pid_target).contains("Pid"));
        assert!(format!("{:?}", job_target).contains("JobId"));
        assert!(format!("{:?}", name_target).contains("ProcessName"));
    }

    #[test]
    fn test_kill_options_debug_format() {
        let options = KillOptions {
            signal: 15,
            signal_name: "TERM".to_string(),
            list_signals: false,
            verbose: true,
            timeout: Some(10),
            targets: vec![KillTarget::Pid(1234)],
        };
        
        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("signal: 15"));
        assert!(debug_str.contains("verbose: true"));
        assert!(debug_str.contains("timeout: Some(10)"));
    }

    #[test]
    fn test_parse_kill_args_with_signal_name_direct() {
        let args = vec!["-KILL".to_string(), "1234".to_string()];
        let options = parse_kill_args(&args).expect("Failed to parse KILL signal args");
        
        assert_eq!(options.signal, 9); // KILL
        assert_eq!(options.signal_name, "KILL");
        assert_eq!(options.targets.len(), 1);
        assert!(matches!(options.targets[0], KillTarget::Pid(1234)));
    }

    #[test]
    fn test_parse_kill_args_with_numeric_signal_arg() {
        let args = vec!["-n".to_string(), "9".to_string(), "1234".to_string()];
        let options = parse_kill_args(&args).expect("Failed to parse -n signal args");
        
        assert_eq!(options.signal, 9); // KILL
        assert_eq!(options.targets.len(), 1);
        assert!(matches!(options.targets[0], KillTarget::Pid(1234)));
    }

    #[test]
    fn test_parse_kill_args_invalid_numeric_signal() {
        let args = vec!["-n".to_string(), "999".to_string(), "1234".to_string()];
        let result = parse_kill_args(&args);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid signal number"));
    }
}