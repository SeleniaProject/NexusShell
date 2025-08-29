use crate::common::{BuiltinContext, BuiltinResult};

/// Display information about running processes
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    let mut show_all = false;
    let mut show_full = false;
    let mut show_threads = false;
    let mut show_user_format = false;
    let mut pid_filter: Option<u32> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--all" => show_all = true,
            "-f" | "--full" => show_full = true,
            "-T" | "--threads" => show_threads = true,
            "-u" | "--user" => show_user_format = true,
            "-p" | "--pid" => {
                if i + 1 >= args.len() {
                    eprintln!("ps: option '{}' requires an argument", args[i]);
                    return Ok(1);
                }
                i += 1;
                match args[i].parse::<u32>() {
                    Ok(pid) => pid_filter = Some(pid),
                    Err(_) => {
                        eprintln!("ps: invalid PID '{}'", args[i]);
                        return Ok(1);
                    }
                }
            }
            "-h" | "--help" => {
                print_help();
                return Ok(0);
            }
            "aux" => {
                // BSD-style format
                show_all = true;
                show_user_format = true;
            }
            arg if arg.starts_with('-') => {
                eprintln!("ps: invalid option '{arg}'");
                return Ok(1);
            }
            _ => {
                eprintln!("ps: unexpected argument '{}'", args[i]);
                return Ok(1);
            }
        }
        i += 1;
    }

    match get_process_info(
        show_all,
        show_full,
        show_threads,
        show_user_format,
        pid_filter,
    ) {
        Ok(processes) => {
            display_processes(&processes, show_full, show_user_format);
            Ok(0)
        }
        Err(e) => {
            eprintln!("ps: {e}");
            Ok(1)
        }
    }
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    ppid: u32,
    user: String,
    command: String,
    cpu_percent: f32,
    mem_percent: f32,
    virtual_size: u64,
    resident_size: u64,
    state: String,
    start_time: String,
    tty: String,
    priority: i32,
    nice: i32,
}

fn get_process_info(
    show_all: bool,
    _show_full: bool,
    _show_threads: bool,
    _show_user_format: bool,
    pid_filter: Option<u32>,
) -> Result<Vec<ProcessInfo>, Box<dyn std::error::Error>> {
    let processes;

    #[cfg(target_os = "linux")]
    {
        processes = get_linux_processes(show_all, pid_filter)?;
    }

    #[cfg(target_os = "windows")]
    {
        processes = get_windows_processes(show_all, pid_filter)?;
    }

    #[cfg(target_os = "macos")]
    {
        processes = get_macos_processes(show_all, pid_filter)?;
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        // Fallback for other systems
        processes = get_fallback_processes(show_all, pid_filter)?;
    }

    Ok(processes)
}

#[cfg(target_os = "linux")]
fn get_linux_processes(
    show_all: bool,
    pid_filter: Option<u32>,
) -> Result<Vec<ProcessInfo>, Box<dyn std::error::Error>> {
    let mut processes = Vec::new();
    let proc_dir = Path::new("/proc");

    if !proc_dir.exists() {
        return Err("Cannot access /proc filesystem".into());
    }

    for entry in fs::read_dir(proc_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if let Ok(pid) = name_str.parse::<u32>() {
            if let Some(filter_pid) = pid_filter {
                if pid != filter_pid {
                    continue;
                }
            }

            if let Ok(process) = parse_linux_process(pid) {
                if show_all || process.tty != "?" {
                    processes.push(process);
                }
            }
        }
    }

    Ok(processes)
}

#[cfg(target_os = "linux")]
fn parse_linux_process(pid: u32) -> Result<ProcessInfo, Box<dyn std::error::Error>> {
    let stat_path = format!("/proc/{}/stat", pid);
    let cmdline_path = format!("/proc/{}/cmdline", pid);
    let status_path = format!("/proc/{}/status", pid);

    let stat_content = fs::read_to_string(&stat_path)?;
    let stat_fields: Vec<&str> = stat_content.split_whitespace().collect();

    if stat_fields.len() < 24 {
        return Err("Invalid stat file format".into());
    }

    let ppid = stat_fields[3].parse::<u32>().unwrap_or(0);
    let state = stat_fields[2].to_string();
    let priority = stat_fields[17].parse::<i32>().unwrap_or(0);
    let nice = stat_fields[18].parse::<i32>().unwrap_or(0);
    let virtual_size = stat_fields[22].parse::<u64>().unwrap_or(0);
    let resident_size = stat_fields[23].parse::<u64>().unwrap_or(0) * 4096; // Convert pages to bytes

    let cmdline = fs::read_to_string(&cmdline_path)
        .unwrap_or_default()
        .replace('\0', " ")
        .trim()
        .to_string();

    let command = if cmdline.is_empty() {
        format!(
            "[{}]",
            stat_fields
                .get(1)
                .unwrap_or(&"unknown")
                .trim_matches(['(', ')'])
        )
    } else {
        cmdline
    };

    // Try to get user info from status file
    let user = if let Ok(status_content) = fs::read_to_string(&status_path) {
        parse_user_from_status(&status_content).unwrap_or_else(|| "unknown".to_string())
    } else {
        "unknown".to_string()
    };

    Ok(ProcessInfo {
        pid,
        ppid,
        user,
        command,
        cpu_percent: 0.0, // Would need sampling over time
        mem_percent: 0.0, // Would need system memory info
        virtual_size,
        resident_size,
        state,
        start_time: "?".to_string(), // Would need boot time calculation
        tty: "?".to_string(),        // Would need tty parsing
        priority,
        nice,
    })
}

#[cfg(target_os = "linux")]
fn parse_user_from_status(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("Uid:") {
            if let Some(uid_str) = line.split_whitespace().nth(1) {
                if let Ok(uid) = uid_str.parse::<u32>() {
                    // In a real implementation, would look up username from /etc/passwd
                    return Some(format!("uid{}", uid));
                }
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn get_windows_processes(
    _show_all: bool,
    pid_filter: Option<u32>,
) -> Result<Vec<ProcessInfo>, Box<dyn std::error::Error>> {
    let mut processes = Vec::new();

    // Simplified Windows implementation
    // In a real implementation, would use Windows APIs like EnumProcesses
    let current_pid = std::process::id();

    if let Some(filter_pid) = pid_filter {
        if filter_pid == current_pid {
            processes.push(create_current_process_info());
        }
    } else {
        processes.push(create_current_process_info());
    }

    Ok(processes)
}

#[cfg(any(
    target_os = "macos",
    not(any(target_os = "linux", target_os = "windows"))
))]
fn get_macos_processes(
    _show_all: bool,
    pid_filter: Option<u32>,
) -> Result<Vec<ProcessInfo>, Box<dyn std::error::Error>> {
    get_fallback_processes(_show_all, pid_filter)
}

fn get_fallback_processes(
    _show_all: bool,
    pid_filter: Option<u32>,
) -> Result<Vec<ProcessInfo>, Box<dyn std::error::Error>> {
    let mut processes = Vec::new();
    let current_pid = std::process::id();

    if let Some(filter_pid) = pid_filter {
        if filter_pid == current_pid {
            processes.push(create_current_process_info());
        }
    } else {
        processes.push(create_current_process_info());
    }

    Ok(processes)
}

fn create_current_process_info() -> ProcessInfo {
    ProcessInfo {
        pid: std::process::id(),
        ppid: 0, // Parent PID not easily available in cross-platform way
        user: whoami::username(),
        command: std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "nxsh".to_string()),
        cpu_percent: 0.0,
        mem_percent: 0.0,
        virtual_size: 0,
        resident_size: 0,
        state: "R".to_string(),
        start_time: "?".to_string(),
        tty: "?".to_string(),
        priority: 0,
        nice: 0,
    }
}

fn display_processes(processes: &[ProcessInfo], show_full: bool, show_user_format: bool) {
    if show_user_format {
        println!(
            "{:<8} {:>5} {:>4} {:>4} {:>6} {:>6} {:<8} {:<1} {:>8} {:>8} COMMAND",
            "USER", "PID", "%CPU", "%MEM", "VSZ", "RSS", "TTY", "STAT", "START", "TIME"
        );
    } else {
        println!("{:>5} {:<8} {:>8} CMD", "PID", "TTY", "TIME");
    }

    for process in processes {
        if show_user_format {
            let command = if show_full {
                &process.command
            } else {
                // Show just the command name
                process
                    .command
                    .split_whitespace()
                    .next()
                    .unwrap_or(&process.command)
            };

            println!(
                "{:<8} {:>5} {:>4.1} {:>4.1} {:>6} {:>6} {:<8} {:<1} {:>8} {:>8} {}",
                truncate_string(&process.user, 8),
                process.pid,
                process.cpu_percent,
                process.mem_percent,
                format_size(process.virtual_size),
                format_size(process.resident_size),
                truncate_string(&process.tty, 8),
                process.state,
                process.start_time,
                "00:00:00", // Time would need calculation
                command
            );
        } else {
            let command = if show_full {
                &process.command
            } else {
                process
                    .command
                    .split('/')
                    .next_back()
                    .or_else(|| process.command.split('\\').next_back())
                    .unwrap_or(&process.command)
            };

            println!(
                "{:>5} {:<8} {:>8} {}",
                process.pid,
                truncate_string(&process.tty, 8),
                "00:00:00",
                command
            );
        }
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}+", &s[..max_len - 1])
    }
}

fn format_size(size: u64) -> String {
    if size < 1024 {
        size.to_string()
    } else if size < 1024 * 1024 {
        format!("{}K", size / 1024)
    } else if size < 1024 * 1024 * 1024 {
        format!("{}M", size / (1024 * 1024))
    } else {
        format!("{}G", size / (1024 * 1024 * 1024))
    }
}

fn print_help() {
    println!("Usage: ps [OPTIONS]");
    println!("Display information about running processes.");
    println!();
    println!("Options:");
    println!("  -a, --all       show processes for all users");
    println!("  -f, --full      show full command lines");
    println!("  -T, --threads   show threads");
    println!("  -u, --user      show user-oriented format");
    println!("  -p, --pid PID   show only process with specified PID");
    println!("  -h, --help      display this help and exit");
    println!();
    println!("BSD-style options:");
    println!("  aux             show all processes in user format");
    println!();
    println!("Examples:");
    println!("  ps              Show processes for current user");
    println!("  ps -a           Show all processes");
    println!("  ps aux          Show all processes with detailed info");
    println!("  ps -p 1234      Show process with PID 1234");
}
