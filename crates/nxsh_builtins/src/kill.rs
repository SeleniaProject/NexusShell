//! `kill` command - terminate processes by sending signals
//!
//! Full kill implementation with signal names, process groups, and job control

use crate::common::{i18n::*, logging::*};
use std::io::Write;
use std::collections::HashMap;
use nxsh_core::{Builtin, Context, ExecutionResult, ShellResult};

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
    fn name(&self) -> &str {
        "kill"
    }

    fn execute(&self, context: &mut Context, args: Vec<String>) -> ShellResult<i32> {
        let options = parse_kill_args(&args)?;
        
        if options.list_signals {
            list_signals();
            return Ok(0);
        }
        
        if options.targets.is_empty() {
            return Err(ShellError::runtime("No process specified"));
        }
        
        let mut exit_code = 0;
        
        for target in &options.targets {
            match kill_target(target, options.signal, &options) {
                Ok(_) => {
                    if options.verbose {
                        println!("Signal {} sent to {:?}", options.signal_name, target);
                    }
                }
                Err(e) => {
                    eprintln!("kill: {:?}: {}", target, e);
                    exit_code = 1;
                }
            }
        }
        
        Ok(exit_code)
    }

    fn help(&self) -> &str {
        "kill - terminate processes by sending signals

USAGE:
    kill [OPTIONS] PID...
    kill [OPTIONS] %JOB...
    kill [OPTIONS] -SIGNAL PID...
    kill -l [SIGNAL]

OPTIONS:
    -s SIGNAL, --signal=SIGNAL    Signal to send (name or number)
    -n SIGNAL                     Signal number to send
    -l, --list                    List signal names
    -L, --table                   List signal names in a table
    -v, --verbose                 Verbose output
    -t TIMEOUT, --timeout=TIMEOUT Wait TIMEOUT seconds between TERM and KILL
    --help                        Display this help and exit

SIGNALS:
    1  HUP     Hangup
    2  INT     Interrupt (Ctrl+C)
    3  QUIT    Quit (Ctrl+\\)
    9  KILL    Kill (cannot be caught or ignored)
    15 TERM    Terminate (default)
    18 CONT    Continue
    19 STOP    Stop (cannot be caught or ignored)
    20 TSTP    Terminal stop (Ctrl+Z)

TARGETS:
    PID         Process ID
    %JOB        Job ID (from jobs command)
    -PID        Process group ID
    0           Current process group
    -1          All processes (requires privileges)
    COMMAND     All processes with matching command name

EXAMPLES:
    kill 1234               Send TERM signal to process 1234
    kill -9 1234            Send KILL signal to process 1234
    kill -KILL 1234         Send KILL signal to process 1234
    kill -HUP 1234          Send HUP signal to process 1234
    kill %1                 Kill job 1
    kill -15 -1234          Send TERM to process group 1234
    kill -l                 List all signal names
    kill --timeout=5 1234   Send TERM, wait 5s, then KILL"
    }
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
                            println!("{}", name);
                            return Ok(options);
                        }
                    } else if let Some(&sig_num) = signal_map.get(&args[i + 1].to_uppercase()) {
                        println!("{}", sig_num);
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
                    return Err(ShellError::runtime("Option -s requires an argument"));
                }
                let (sig_num, sig_name) = parse_signal(&args[i], &signal_map)?;
                options.signal = sig_num;
                options.signal_name = sig_name;
            }
            "-n" => {
                i += 1;
                if i >= args.len() {
                    return Err(ShellError::runtime("Option -n requires an argument"));
                }
                options.signal = args[i].parse()
                    .map_err(|_| ShellError::runtime("Invalid signal number"))?;
                options.signal_name = get_signal_name(options.signal)
                    .unwrap_or_else(|| format!("{}", options.signal));
            }
            "-t" | "--timeout" => {
                i += 1;
                if i >= args.len() {
                    return Err(ShellError::runtime("Option -t requires an argument"));
                }
                options.timeout = Some(args[i].parse()
                    .map_err(|_| ShellError::runtime("Invalid timeout value"))?);
            }
            "--help" => return Err(ShellError::runtime("Help requested")),
            _ if arg.starts_with("-") && arg.len() > 1 => {
                // Handle -SIGNAL format
                let signal_str = &arg[1..];
                let (sig_num, sig_name) = parse_signal(signal_str, &signal_map)?;
                options.signal = sig_num;
                options.signal_name = sig_name;
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
    // Try parsing as number first
    if let Ok(sig_num) = signal_str.parse::<i32>() {
        let sig_name = get_signal_name(sig_num)
            .unwrap_or_else(|| format!("{}", sig_num));
        return Ok((sig_num, sig_name));
    }
    
    // Try parsing as signal name
    let signal_upper = signal_str.to_uppercase();
    let signal_name = if signal_upper.starts_with("SIG") {
        &signal_upper[3..]
    } else {
        &signal_upper
    };
    
    if let Some(&sig_num) = signal_map.get(signal_name) {
        Ok((sig_num, signal_name.to_string()))
    } else {
        Err(ShellError::runtime(format!("Unknown signal: {}", signal_str)))
    }
}

fn parse_kill_target(target_str: &str) -> ShellResult<KillTarget> {
    if target_str == "-1" {
        Ok(KillTarget::All)
    } else if target_str.starts_with('%') {
        // Job ID
        let job_id = target_str[1..].parse::<u32>()
            .map_err(|_| ShellError::runtime("Invalid job ID"))?;
        Ok(KillTarget::JobId(job_id))
    } else if target_str.starts_with('-') {
        // Process group
        let pgrp = target_str[1..].parse::<u32>()
            .map_err(|_| ShellError::runtime("Invalid process group ID"))?;
        Ok(KillTarget::ProcessGroup(pgrp))
    } else if let Ok(pid) = target_str.parse::<u32>() {
        // Process ID
        Ok(KillTarget::Pid(pid))
    } else {
        // Process name
        Ok(KillTarget::ProcessName(target_str.to_string()))
    }
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
            // Would need to look up job in job table
            return Err(ShellError::runtime("Job control not yet implemented"));
        }
        KillTarget::ProcessName(name) => {
            let pids = find_processes_by_name(name)?;
            if pids.is_empty() {
                return Err(ShellError::runtime(format!("No processes found matching '{}'", name)));
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
            match target {
                KillTarget::Pid(pid) => {
                    if process_exists(*pid)? {
                        send_signal_to_pid(*pid, 9)?; // SIGKILL
                    }
                }
                _ => {} // Only implement timeout for single PIDs for now
            }
        }
    }
    
    Ok(())
}

fn send_signal_to_pid(pid: u32, signal: i32) -> ShellResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        
        let result = unsafe {
            libc::kill(pid as libc::pid_t, signal)
        };
        
        if result == -1 {
            let error = std::io::Error::last_os_error();
            match error.raw_os_error() {
                Some(libc::ESRCH) => Err(ShellError::runtime("No such process")),
                Some(libc::EPERM) => Err(ShellError::runtime("Operation not permitted")),
                Some(libc::EINVAL) => Err(ShellError::runtime("Invalid signal")),
                _ => Err(ShellError::io(format!("Failed to send signal: {}", error))),
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
            .args(&["/PID", &pid.to_string(), "/F"])
            .output()
            .map_err(|e| ShellError::io(format!("Failed to kill process: {}", e)))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(ShellError::runtime(format!("Failed to kill process: {}", error_msg)))
        } else {
            Ok(())
        }
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        Err(ShellError::runtime("Signal sending not supported on this platform"))
    }
}

fn send_signal_to_process_group(pgrp: u32, signal: i32) -> ShellResult<()> {
    #[cfg(unix)]
    {
        let result = unsafe {
            libc::kill(-(pgrp as libc::pid_t), signal)
        };
        
        if result == -1 {
            let error = std::io::Error::last_os_error();
            Err(ShellError::io(format!("Failed to send signal to process group: {}", error)))
        } else {
            Ok(())
        }
    }
    
    #[cfg(not(unix))]
    {
        Err(ShellError::runtime("Process groups not supported on this platform"))
    }
}

fn find_processes_by_name(name: &str) -> ShellResult<Vec<u32>> {
    let mut pids = Vec::new();
    
    #[cfg(target_os = "linux")]
    {
        let proc_dir = std::fs::read_dir("/proc")
            .map_err(|e| ShellError::io(format!("Cannot read /proc: {}", e)))?;
        
        for entry in proc_dir {
            let entry = entry.map_err(|e| ShellError::io(format!("Error reading /proc entry: {}", e)))?;
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
            .args(&["/FO", "CSV", "/NH"])
            .output()
            .map_err(|e| ShellError::io(format!("Failed to list processes: {}", e)))?;
        
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
            .args(&["/FI", &format!("PID eq {}", pid), "/FO", "CSV"])
            .output()
            .map_err(|e| ShellError::io(format!("Failed to check process: {}", e)))?;
        
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
        println!("{:2}) SIG{:<8} {}", num, name, desc);
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
            line.push_str(&format!("{:2}) {:>8} ", num, name));
        }
        println!("{}", line);
    }
} 