//! `ps` command - display running processes with comprehensive information
//!
//! Full ps implementation with process tree, filtering, and detailed process information

use std::collections::HashMap;
use nxsh_core::{Builtin, ExecutionResult, executor::{ExecutionStrategy, ExecutionMetrics}, ShellResult, ShellError, ErrorKind};
use nxsh_core::context::ShellContext;
use nxsh_core::error::{RuntimeErrorKind, IoErrorKind};
use nxsh_hal::{ProcessInfo, ProcessManager};
use crate::ui_design::{TableFormatter, Colorize, TableOptions, BorderStyle, Alignment, Animation, ProgressBar, Notification};
// use std::io::BufRead; // Removed unused BufRead import
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Helper error construction functions (used throughout argument parsing and processing)
fn runtime_error(message: &str) -> ShellError {
    // Construct a runtime ShellError with CommandNotFound kind placeholder.
    // If more specific kinds become available, adjust accordingly.
    ShellError::new(
        ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound),
        message.to_string(),
    )
}

fn io_error(message: &str) -> ShellError {
    // Construct an IO ShellError specialized for file read errors.
    ShellError::new(
        ErrorKind::IoError(IoErrorKind::FileReadError),
        message.to_string(),
    )
}

pub struct PsBuiltin;

#[derive(Debug, Clone, Default)]
pub struct PsOptions {
    pub all_processes: bool,
    pub all_users: bool,
    pub show_threads: bool,
    pub full_format: bool,
    pub long_format: bool,
    pub user_format: bool,
    pub tree_format: bool,
    pub no_headers: bool,
    pub sort_by: Option<String>,
    pub filter_user: Option<String>,
    pub filter_pid: Option<u32>,
    pub filter_ppid: Option<u32>,
    pub filter_command: Option<String>,
    pub output_format: Option<String>,
    pub wide_output: bool,
    pub show_environment: bool,
    pub show_command_line: bool,
}

#[derive(Debug, Clone)]
pub struct ProcessEntry {
    pub pid: u32,
    pub ppid: u32,
    pub uid: u32,
    pub gid: u32,
    pub user: String,
    pub group: String,
    pub command: String,
    pub args: Vec<String>,
    pub state: String,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub memory_rss: u64,
    pub memory_vsz: u64,
    pub start_time: SystemTime,
    pub cpu_time: Duration,
    pub priority: i32,
    pub nice: i32,
    pub threads: u32,
    pub tty: String,
    pub session_id: u32,
    pub process_group: u32,
    pub environment: HashMap<String, String>,
}

impl Builtin for PsBuiltin {
    fn name(&self) -> &'static str {
        "ps"
    }

    fn synopsis(&self) -> &'static str {
        "report a snapshot of current processes"
    }

    fn description(&self) -> &'static str {
        "Display information about running processes"
    }

    fn help(&self) -> &'static str {
        "Process status command. Use 'ps --help' for detailed usage information."
    }

    fn execute(&self, _ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        let options = parse_ps_args(args)?;
        let processes = collect_processes(&options)?;
        display_processes(&processes, &options)?;
        Ok(ExecutionResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            execution_time: 0,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        })
    }

    fn usage(&self) -> &'static str {
        "ps - display running processes

USAGE:
    ps [OPTIONS]

OPTIONS:
    -A, -e, --all             Show all processes
    -a                        Show all processes except session leaders
    -x                        Show processes without controlling terminal
    -u, --user=USER           Show processes for specified user
    -p, --pid=PID             Show process with specified PID
    --ppid=PPID               Show processes with specified parent PID
    -C, --command=CMD         Show processes running specified command
    -f, --full                Full format listing
    -l, --long                Long format listing
    -j                        Jobs format
    -s                        Signal format
    -v                        Virtual memory format
    -m                        Show threads
    -H, --forest              Show process hierarchy (tree)
    --no-headers              Don't print headers
    --sort=SPEC               Sort by specified columns
    -o, --format=FORMAT       User-defined format
    -w, --wide                Wide output (don't truncate)
    -e, --environment         Show environment variables
    --help                    Display this help and exit

FORMAT SPECIFIERS:
    pid, ppid, uid, gid, user, group, comm, args, state, pcpu, pmem
    rss, vsz, stime, time, pri, ni, nlwp, tty, sid, pgrp

EXAMPLES:
    ps                        Show processes for current user
    ps aux                    Show all processes with detailed info
    ps -ef                    Show all processes in full format
    ps -u root                Show processes for root user
    ps --forest               Show process tree
    ps -o pid,comm,pcpu       Custom format output
    ps --sort=-pcpu           Sort by CPU usage (descending)"
    }
}

fn parse_ps_args(args: &[String]) -> ShellResult<PsOptions> {
    let mut options = PsOptions {
        all_processes: false,
        all_users: false,
        show_threads: false,
        full_format: false,
        long_format: false,
        user_format: false,
        tree_format: false,
        no_headers: false,
        sort_by: None,
        filter_user: None,
        filter_pid: None,
        filter_ppid: None,
        filter_command: None,
        output_format: None,
        wide_output: false,
        show_environment: false,
        show_command_line: false,
    };

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "-A" | "-e" | "--all" => options.all_processes = true,
            "-a" => options.all_users = true,
            "-x" => options.all_processes = true,
            "-f" | "--full" => options.full_format = true,
            "-l" | "--long" => options.long_format = true,
            "-m" => options.show_threads = true,
            "-H" | "--forest" => options.tree_format = true,
            "--no-headers" => options.no_headers = true,
            "-w" | "--wide" => options.wide_output = true,
            // '--environment' はロングオプションのみ使用。短縮 '-e' は上で all_processes 扱い。
            "--environment" => options.show_environment = true,
            "-u" | "--user" => {
                i += 1;
                if i >= args.len() {
                    return Err(runtime_error("Option -u requires an argument"));
                }
                options.filter_user = Some(args[i].clone());
            }
            "-p" | "--pid" => {
                i += 1;
                if i >= args.len() {
                    return Err(runtime_error("Option -p requires an argument"));
                }
                options.filter_pid = Some(args[i].parse()
                    .map_err(|_| runtime_error("Invalid PID"))?);
            }
            "--ppid" => {
                i += 1;
                if i >= args.len() {
                    return Err(runtime_error("Option --ppid requires an argument"));
                }
                options.filter_ppid = Some(args[i].parse()
                    .map_err(|_| runtime_error("Invalid PPID"))?);
            }
            "-C" | "--command" => {
                i += 1;
                if i >= args.len() {
                    return Err(runtime_error("Option -C requires an argument"));
                }
                options.filter_command = Some(args[i].clone());
            }
            "--sort" => {
                i += 1;
                if i >= args.len() {
                    return Err(runtime_error("Option --sort requires an argument"));
                }
                options.sort_by = Some(args[i].clone());
            }
            "-o" | "--format" => {
                i += 1;
                if i >= args.len() {
                    return Err(runtime_error("Option -o requires an argument"));
                }
                options.output_format = Some(args[i].clone());
            }
            "--help" => return Err(runtime_error("Help requested")),
            _ if arg.starts_with("-") => {
                // Handle BSD-style combined options
                for ch in arg[1..].chars() {
                    match ch {
                        'A' | 'e' => options.all_processes = true,
                        'a' => options.all_users = true,
                        'x' => options.all_processes = true,
                        'f' => options.full_format = true,
                        'l' => options.long_format = true,
                        'u' => options.user_format = true,
                        'm' => options.show_threads = true,
                        'H' => options.tree_format = true,
                        'w' => options.wide_output = true,
                        _ => return Err(runtime_error(&format!("Unknown option: -{ch}"))),
                    }
                }
            }
            _ => return Err(runtime_error(&format!("Unknown argument: {arg}"))),
        }
        i += 1;
    }

    Ok(options)
}

fn collect_processes(options: &PsOptions) -> ShellResult<Vec<ProcessEntry>> {
    let mut processes = Vec::new();
    
    // Get all processes from /proc on Linux, or use platform-specific methods
    #[cfg(target_os = "linux")]
    {
        let proc_dir = fs::read_dir("/proc")
            .map_err(|e| io_error(&format!("Cannot read /proc: {}", e)))?;
        
        for entry in proc_dir {
            let entry = entry.map_err(|e| io_error(&format!("Error reading /proc entry: {}", e)))?;
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            
            if let Ok(pid) = name_str.parse::<u32>() {
                if let Ok(process) = read_process_info(pid) {
                    if should_include_process(&process, options) {
                        processes.push(process);
                    }
                }
            }
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        // Use HAL for other platforms
        let process_manager = ProcessManager::new()?;
        let system_processes = process_manager.get_system_processes()
            .map_err(|e| runtime_error(&format!("Failed to get processes: {e}")))?;
        
        for proc_info in system_processes {
            let process = convert_hal_process(proc_info);
            if should_include_process(&process, options) {
                processes.push(process);
            }
        }
    }
    
    // Sort processes if requested
    if let Some(ref sort_spec) = options.sort_by {
        sort_processes(&mut processes, sort_spec)?;
    }
    
    Ok(processes)
}

#[cfg(target_os = "linux")]
fn read_process_info(pid: u32) -> Result<ProcessEntry, Box<dyn std::error::Error>> {
    let stat_path = format!("/proc/{}/stat", pid);
    let status_path = format!("/proc/{}/status", pid);
    let cmdline_path = format!("/proc/{}/cmdline", pid);
    let environ_path = format!("/proc/{}/environ", pid);
    
    // Read /proc/pid/stat
    let stat_content = fs::read_to_string(&stat_path)?;
    let stat_fields: Vec<&str> = stat_content.split_whitespace().collect();
    
    if stat_fields.len() < 44 {
        return Err("Invalid stat file format".into());
    }
    
    let ppid = stat_fields[3].parse::<u32>()?;
    let pgrp = stat_fields[4].parse::<u32>()?;
    let session = stat_fields[5].parse::<u32>()?;
    let tty_nr = stat_fields[6].parse::<i32>()?;
    let priority = stat_fields[17].parse::<i32>()?;
    let nice = stat_fields[18].parse::<i32>()?;
    let num_threads = stat_fields[19].parse::<u32>()?;
    let starttime = stat_fields[21].parse::<u64>()?;
    let vsize = stat_fields[22].parse::<u64>()?;
    let rss = stat_fields[23].parse::<u64>()? * 4096; // Convert pages to bytes
    
    // Read /proc/pid/status for additional info
    let mut uid = 0;
    let mut gid = 0;
    let mut state = "?".to_string();
    
    if let Ok(status_content) = fs::read_to_string(&status_path) {
        for line in status_content.lines() {
            if line.starts_with("Uid:") {
                if let Some(uid_str) = line.split_whitespace().nth(1) {
                    uid = uid_str.parse().unwrap_or(0);
                }
            } else if line.starts_with("Gid:") {
                if let Some(gid_str) = line.split_whitespace().nth(1) {
                    gid = gid_str.parse().unwrap_or(0);
                }
            } else if line.starts_with("State:") {
                if let Some(state_str) = line.split_whitespace().nth(1) {
                    state = state_str.to_string();
                }
            }
        }
    }
    
    // Read command line
    let mut command = format!("[{}]", pid);
    let mut args = Vec::new();
    
    if let Ok(cmdline_content) = fs::read(&cmdline_path) {
        let cmdline_str = String::from_utf8_lossy(&cmdline_content);
        let parts: Vec<&str> = cmdline_str.split('\0').filter(|s| !s.is_empty()).collect();
        if !parts.is_empty() {
            command = parts[0].to_string();
            args = parts.iter().map(|s| s.to_string()).collect();
        }
    }
    
    // Read environment
    let mut environment = HashMap::new();
    if let Ok(environ_content) = fs::read(&environ_path) {
        let environ_str = String::from_utf8_lossy(&environ_content);
        for env_var in environ_str.split('\0').filter(|s| !s.is_empty()) {
            if let Some(eq_pos) = env_var.find('=') {
                let key = env_var[..eq_pos].to_string();
                let value = env_var[eq_pos + 1..].to_string();
                environment.insert(key, value);
            }
        }
    }
    
    // Get user and group names
    let user = get_username(uid).unwrap_or_else(|| uid.to_string());
    let group = get_groupname(gid).unwrap_or_else(|| gid.to_string());
    
    // Calculate TTY name
    let tty = if tty_nr == 0 {
        "?".to_string()
    } else {
        format!("pts/{}", tty_nr)
    };
    
    // Calculate start time
    let boot_time = get_boot_time().unwrap_or(UNIX_EPOCH);
    let start_time = boot_time + Duration::from_secs(starttime / 100); // Convert jiffies to seconds
    
    Ok(ProcessEntry {
        pid,
        ppid,
        uid,
        gid,
        user,
        group,
        command,
        args,
        state,
        cpu_percent: 0.0, // Would need multiple samples to calculate
        memory_percent: 0.0, // Would need system memory info
        memory_rss: rss,
        memory_vsz: vsize,
        start_time,
        cpu_time: Duration::from_secs(0), // Would need to parse from stat
        priority,
        nice,
        threads: num_threads,
        tty,
        session_id: session,
        process_group: pgrp,
        environment,
    })
}

#[cfg(target_os = "linux")]
fn get_boot_time() -> Option<SystemTime> {
    if let Ok(content) = fs::read_to_string("/proc/stat") {
        for line in content.lines() {
            if line.starts_with("btime ") {
                if let Some(btime_str) = line.split_whitespace().nth(1) {
                    if let Ok(btime) = btime_str.parse::<u64>() {
                        return Some(UNIX_EPOCH + Duration::from_secs(btime));
                    }
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn get_username(uid: u32) -> Option<String> {
    if let Ok(content) = fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 3 {
                if let Ok(file_uid) = fields[2].parse::<u32>() {
                    if file_uid == uid {
                        return Some(fields[0].to_string());
                    }
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn get_groupname(gid: u32) -> Option<String> {
    if let Ok(content) = fs::read_to_string("/etc/group") {
        for line in content.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 3 {
                if let Ok(file_gid) = fields[2].parse::<u32>() {
                    if file_gid == gid {
                        return Some(fields[0].to_string());
                    }
                }
            }
        }
    }
    None
}

#[cfg(not(target_os = "linux"))]
fn convert_hal_process(proc_info: ProcessInfo) -> ProcessEntry {
    ProcessEntry {
        pid: proc_info.pid,
        ppid: proc_info.parent_pid.unwrap_or(0),
        uid: 0, // Would need to get from HAL
        gid: 0,
        user: "unknown".to_string(),
        group: "unknown".to_string(),
        command: proc_info.name,
        args: if proc_info.command_line.is_empty() { vec![] } else { vec![proc_info.command_line] },
        state: format!("{:?}", proc_info.status),
        cpu_percent: 0.0, // cpu_usage field doesn't exist in ProcessInfo
        memory_percent: 0.0,
        memory_rss: proc_info.memory_usage,
        memory_vsz: 0,
        start_time: proc_info.start_time,
        cpu_time: proc_info.cpu_time,
        priority: 0,
        nice: 0,
        threads: 1,
        tty: "?".to_string(),
        session_id: 0,
        process_group: 0,
        environment: HashMap::new(),
    }
}

fn should_include_process(process: &ProcessEntry, options: &PsOptions) -> bool {
    // Apply filters
    if let Some(ref user) = options.filter_user {
        if process.user != *user {
            return false;
        }
    }
    
    if let Some(pid) = options.filter_pid {
        if process.pid != pid {
            return false;
        }
    }
    
    if let Some(ppid) = options.filter_ppid {
        if process.ppid != ppid {
            return false;
        }
    }
    
    if let Some(ref command) = options.filter_command {
        if !process.command.contains(command) {
            return false;
        }
    }
    
    // Apply general filters
    if !options.all_processes && !options.all_users {
        // Show only processes for current user (simplified)
        return true;
    }
    
    true
}

fn sort_processes(processes: &mut [ProcessEntry], sort_spec: &str) -> ShellResult<()> {
    let reverse = sort_spec.starts_with('-');
    let field = if reverse { &sort_spec[1..] } else { sort_spec };
    
    match field {
        "pid" => processes.sort_by_key(|p| p.pid),
        "ppid" => processes.sort_by_key(|p| p.ppid),
        "pcpu" => processes.sort_by(|a, b| a.cpu_percent.partial_cmp(&b.cpu_percent).unwrap_or(std::cmp::Ordering::Equal)),
        "pmem" => processes.sort_by(|a, b| a.memory_percent.partial_cmp(&b.memory_percent).unwrap_or(std::cmp::Ordering::Equal)),
        "rss" => processes.sort_by_key(|p| p.memory_rss),
        "vsz" => processes.sort_by_key(|p| p.memory_vsz),
        "time" => processes.sort_by_key(|p| p.cpu_time),
        "comm" => processes.sort_by(|a, b| a.command.cmp(&b.command)),
        "user" => processes.sort_by(|a, b| a.user.cmp(&b.user)),
        _ => return Err(runtime_error(&format!("Unknown sort field: {field}"))),
    }
    
    if reverse {
        processes.reverse();
    }
    
    Ok(())
}

fn display_processes(processes: &[ProcessEntry], options: &PsOptions) -> ShellResult<()> {
    if processes.is_empty() {
        println!("{}", "No processes found".muted());
        return Ok(());
    }
    
    let formatter = TableFormatter::new();
    
    // Determine output format
    let format = if let Some(ref custom_format) = options.output_format {
        custom_format.clone()
    } else if options.full_format {
        "uid,pid,ppid,c,stime,tty,time,comm".to_string()
    } else if options.long_format {
        "f,s,uid,pid,ppid,c,pri,ni,addr,sz,wchan,tty,time,comm".to_string()
    } else if options.user_format {
        "user,pid,pcpu,pmem,vsz,rss,tty,stat,start,time,comm".to_string()
    } else {
        "pid,tty,time,comm".to_string()
    };
    
    let fields: Vec<&str> = format.split(',').collect();
    
    // Create headers
    let headers = get_beautiful_headers(&fields);
    
    // Create table rows
    let mut rows = Vec::new();
    
    if options.tree_format {
        // For tree format, we'll create a special display
        print_beautiful_process_tree(processes, &fields, options)?;
        return Ok(());
    } else {
        for process in processes {
            let row = create_process_row(process, &fields, options)?;
            rows.push(row);
        }
    }
    
    // Show loading animation for many processes
    if processes.len() > 200 {
        Animation::spinner();
    }
    
    // Configure beautiful table options for process display
    let table_options = TableOptions {
        show_borders: true,
        zebra_striping: processes.len() > 20,
        compact_mode: false,
        max_width: Some(150),
        show_header: !options.no_headers,
        alternating_rows: processes.len() > 20,
        align_columns: true,
        compact: false,
        border_style: if processes.len() > 50 { BorderStyle::Simple } else { BorderStyle::Rounded },
        header_alignment: Alignment::Left,
    };
    
    // Add header with process count
    if !options.no_headers {
        let header_text = format!("Process List ({} processes)", processes.len());
        println!("{}", formatter.create_header(&[&header_text]));
    }
    
    // Print the beautiful advanced table
    let headers_string: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
    print!("{}", formatter.create_advanced_table(headers_string, rows, table_options));
    
    // Show performance summary for system monitoring
    if processes.len() > 100 {
        Notification::info(&format!("System monitoring: {} active processes detected", processes.len()));
    }
    
    Ok(())
}

fn get_beautiful_headers<'a>(fields: &'a [&'a str]) -> Vec<&'a str> {
    fields.iter().map(|field| {
        match *field {
            "pid" => "PID",
            "ppid" => "Parent",
            "uid" => "UID",
            "gid" => "GID",
            "user" => "User",
            "group" => "Group",
            "comm" => "Command",
            "args" => "Arguments",
            "stat" | "state" => "Status",
            "pcpu" => "CPU%",
            "pmem" => "Memory%",
            "rss" => "RSS",
            "vsz" => "Virtual",
            "stime" | "start" => "Started",
            "time" => "Time",
            "pri" => "Priority",
            "ni" => "Nice",
            "tty" => "Terminal",
            "f" => "Flags",
            "s" => "State",
            "c" => "CPU",
            "addr" => "Address",
            "sz" => "Size",
            "wchan" => "Wait Channel",
            _ => field,
        }
    }).collect()
}

fn create_process_row(process: &ProcessEntry, fields: &[&str], _options: &PsOptions) -> ShellResult<Vec<String>> {
    let mut row = Vec::new();
    
    for field in fields {
        let value = match *field {
            "pid" => process.pid.to_string().primary(),
            "ppid" => process.ppid.to_string().secondary(),
            "uid" => process.uid.to_string().muted(),
            "gid" => process.gid.to_string().muted(),
            "user" => process.user.clone().info(),
            "group" => process.group.clone().dim(),
            "comm" => {
                // Color-code different types of processes
                if process.command.contains("kernel") || process.command.starts_with('[') {
                    process.command.clone().muted()
                } else if process.command.contains("systemd") || process.command.contains("init") {
                    process.command.clone().warning()
                } else if process.cpu_percent > 10.0 {
                    process.command.clone().error()
                } else if process.cpu_percent > 5.0 {
                    process.command.clone().warning()
                } else {
                    process.command.clone().success()
                }
            },
            "args" => process.args.join(" ").dim(),
            "stat" | "state" => {
                match process.state.as_str() {
                    "R" => "Running".success(),
                    "S" => "Sleeping".info(),
                    "D" => "Disk Sleep".warning(),
                    "Z" => "Zombie".error(),
                    "T" => "Stopped".muted(),
                    _ => process.state.clone().dim(),
                }
            },
            "pcpu" => {
                if process.cpu_percent > 50.0 {
                    format!("{:.1}%", process.cpu_percent).error()
                } else if process.cpu_percent > 10.0 {
                    format!("{:.1}%", process.cpu_percent).warning()
                } else {
                    format!("{:.1}%", process.cpu_percent).success()
                }
            },
            "pmem" => {
                if process.memory_percent > 20.0 {
                    format!("{:.1}%", process.memory_percent).error()
                } else if process.memory_percent > 5.0 {
                    format!("{:.1}%", process.memory_percent).warning()
                } else {
                    format!("{:.1}%", process.memory_percent).success()
                }
            },
            "rss" => format_memory_size(process.memory_rss).info(),
            "vsz" => format_memory_size(process.memory_vsz).muted(),
            "stime" | "start" => format_systemtime(process.start_time).dim(),
            "time" => format_cpu_duration(process.cpu_time).dim(),
            "pri" => process.priority.to_string().muted(),
            "ni" => process.nice.to_string().muted(),
            "tty" => process.tty.clone().dim(),
            _ => "-".muted(),
        };
        row.push(value);
    }
    
    Ok(row)
}

fn print_beautiful_process_tree(processes: &[ProcessEntry], fields: &[&str], options: &PsOptions) -> ShellResult<()> {
    let formatter = TableFormatter::new();
    
    println!("{}", formatter.create_header(&["Process Tree"]));
    
    // Build process tree structure
    let mut children: HashMap<u32, Vec<&ProcessEntry>> = HashMap::new();
    let mut roots = Vec::new();
    
    for process in processes {
        if process.ppid == 0 || processes.iter().find(|p| p.pid == process.ppid).is_none() {
            roots.push(process);
        } else {
            children.entry(process.ppid).or_insert_with(Vec::new).push(process);
        }
    }
    
    for root in roots {
        print_tree_node(root, &children, 0, "", true, fields)?;
    }
    
    Ok(())
}

fn print_tree_node(
    process: &ProcessEntry,
    children: &HashMap<u32, Vec<&ProcessEntry>>,
    depth: usize,
    prefix: &str,
    is_last: bool,
    fields: &[&str],
) -> ShellResult<()> {
    let formatter = TableFormatter::new();
    
    // Create tree symbols
    let current_prefix = if depth == 0 {
        "".to_string()
    } else {
        format!("{}{}── ", 
            prefix,
            if is_last { "└" } else { "├" }
        )
    };
    
    // Format process info
    let pid_str = process.pid.to_string().primary();
    let name_str = if process.command.contains("kernel") || process.command.starts_with('[') {
        process.command.clone().muted()
    } else {
        process.command.clone().success()
    };
    let cpu_str = if process.cpu_percent > 10.0 {
        format!("({:.1}%)", process.cpu_percent).error()
    } else {
        format!("({:.1}%)", process.cpu_percent).dim()
    };
    
    println!("{}{} {} {} {}", 
        current_prefix.muted(),
        formatter.icons.bullet,
        pid_str,
        name_str,
        cpu_str
    );
    
    // Print children
    if let Some(child_processes) = children.get(&process.pid) {
        let child_count = child_processes.len();
        for (i, child) in child_processes.iter().enumerate() {
            let new_prefix = if depth == 0 {
                "".to_string()
            } else {
                format!("{}{}   ", 
                    prefix,
                    if is_last { " " } else { "│" }
                )
            };
            print_tree_node(child, children, depth + 1, &new_prefix, i == child_count - 1, fields)?;
        }
    }
    
    Ok(())
}

fn format_memory_size(bytes: u64) -> String {
    let formatter = TableFormatter::new();
    formatter.format_size(bytes)
}

fn format_systemtime(time: SystemTime) -> String {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
            let elapsed = now.saturating_sub(duration);
            let elapsed_secs = elapsed.as_secs();
            
            if elapsed_secs < 60 {
                format!("{}s", elapsed_secs)
            } else if elapsed_secs < 3600 {
                format!("{}m", elapsed_secs / 60)
            } else if elapsed_secs < 86400 {
                format!("{}h", elapsed_secs / 3600)
            } else {
                format!("{}d", elapsed_secs / 86400)
            }
        }
        Err(_) => "-".to_string(),
    }
}

fn format_cpu_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}:{:02}", seconds / 60, seconds % 60)
    } else {
        format!("{}:{:02}:{:02}", seconds / 3600, (seconds % 3600) / 60, seconds % 60)
    }
}

fn format_time(timestamp: u64) -> String {
    // Simple time formatting - in real implementation, use proper time formatting
    if timestamp == 0 {
        "-".to_string()
    } else {
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let now = duration.as_secs();
        if now > timestamp {
            let elapsed = now - timestamp;
            if elapsed < 60 {
                format!("{}s", elapsed)
            } else if elapsed < 3600 {
                format!("{}m", elapsed / 60)
            } else if elapsed < 86400 {
                format!("{}h", elapsed / 3600)
            } else {
                format!("{}d", elapsed / 86400)
            }
        } else {
            "now".to_string()
        }
    }
}

fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}:{:02}", seconds / 60, seconds % 60)
    } else {
        format!("{}:{:02}:{:02}", seconds / 3600, (seconds % 3600) / 60, seconds % 60)
    }
}

fn print_headers(fields: &[&str], _options: &PsOptions) {
    let mut header_parts = Vec::new();
    
    for field in fields {
        let header = match *field {
            "pid" => "PID",
            "ppid" => "PPID",
            "uid" => "UID",
            "gid" => "GID",
            "user" => "USER",
            "group" => "GROUP",
            "comm" => "COMMAND",
            "args" => "COMMAND",
            "stat" | "state" => "STAT",
            "pcpu" => "%CPU",
            "pmem" => "%MEM",
            "rss" => "RSS",
            "vsz" => "VSZ",
            "stime" | "start" => "START",
            "time" => "TIME",
            "pri" => "PRI",
            "ni" => "NI",
            "nlwp" => "NLWP",
            "tty" => "TTY",
            "sid" => "SID",
            "pgrp" => "PGRP",
            _ => {
                field
            },
        };
        header_parts.push(format!("{header:>8}"));
    }
    
    println!("{}", header_parts.join(" "));
}

fn print_process_line(process: &ProcessEntry, fields: &[&str], options: &PsOptions) -> ShellResult<()> {
    let mut parts = Vec::new();
    
    for field in fields {
        let value = match *field {
            "pid" => format!("{:>8}", process.pid),
            "ppid" => format!("{:>8}", process.ppid),
            "uid" => format!("{:>8}", process.uid),
            "gid" => format!("{:>8}", process.gid),
            "user" => format!("{:>8}", truncate_string(&process.user, 8)),
            "group" => format!("{:>8}", truncate_string(&process.group, 8)),
            "comm" => {
                let cmd = if options.show_command_line && !process.args.is_empty() {
                    process.args.join(" ")
                } else {
                    process.command.clone()
                };
                if options.wide_output {
                    cmd
                } else {
                    truncate_string(&cmd, 20)
                }
            },
            "args" => {
                let cmd = process.args.join(" ");
                if options.wide_output {
                    cmd
                } else {
                    truncate_string(&cmd, 30)
                }
            },
            "stat" | "state" => format!("{:>8}", process.state),
            "pcpu" => format!("{:>8.1}", process.cpu_percent),
            "pmem" => format!("{:>8.1}", process.memory_percent),
            "rss" => format!("{:>8}", process.memory_rss / 1024), // KB
            "vsz" => format!("{:>8}", process.memory_vsz / 1024), // KB
            "stime" | "start" => {
                let duration = SystemTime::now().duration_since(process.start_time)
                    .unwrap_or(Duration::from_secs(0));
                if duration.as_secs() < 86400 {
                    format!("{:>8}", format_cpu_duration(duration))
                } else {
                    format!("{:>8}", format_date(process.start_time))
                }
            },
            "time" => format!("{:>8}", format_cpu_duration(process.cpu_time)),
            "pri" => format!("{:>8}", process.priority),
            "ni" => format!("{:>8}", process.nice),
            "nlwp" => format!("{:>8}", process.threads),
            "tty" => format!("{:>8}", truncate_string(&process.tty, 8)),
            "sid" => format!("{:>8}", process.session_id),
            "pgrp" => format!("{:>8}", process.process_group),
            _ => format!("{:>8}", "?"),
        };
        parts.push(value);
    }
    
    println!("{}", parts.join(" "));
    
    // Show environment if requested
    if options.show_environment && !process.environment.is_empty() {
        for (key, value) in &process.environment {
            println!("    {key}={value}");
        }
    }
    
    Ok(())
}

fn print_process_tree(processes: &[ProcessEntry], fields: &[&str], options: &PsOptions) -> ShellResult<()> {
    // Build parent-child relationships
    let mut children: HashMap<u32, Vec<&ProcessEntry>> = HashMap::new();
    let mut roots = Vec::new();
    
    for process in processes {
        if process.ppid == 0 || !processes.iter().any(|p| p.pid == process.ppid) {
            roots.push(process);
        } else {
            children.entry(process.ppid).or_default().push(process);
        }
    }
    
    // Print tree recursively
    for root in roots {
        print_process_tree_recursive(root, &children, fields, options, 0)?;
    }
    
    Ok(())
}

fn print_process_tree_recursive(
    process: &ProcessEntry,
    children: &HashMap<u32, Vec<&ProcessEntry>>,
    fields: &[&str],
    options: &PsOptions,
    depth: usize,
) -> ShellResult<()> {
    // Print current process with indentation
    let indent = "  ".repeat(depth);
    print!("{indent}");
    print_process_line(process, fields, options)?;
    
    // Print children
    if let Some(child_processes) = children.get(&process.pid) {
        for child in child_processes {
            print_process_tree_recursive(child, children, fields, options, depth + 1)?;
        }
    }
    
    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn format_date(time: SystemTime) -> String {
    // Simplified date formatting
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let timestamp = duration.as_secs();
            // This is a simplified implementation
            format!("{}", timestamp % 86400 / 3600) // Just show hour
        }
        Err(_) => "?".to_string(),
    }
}

// CLI entry point function
pub fn ps_cli(args: &[String]) -> anyhow::Result<()> {
    let options = parse_ps_args(args).map_err(|e| anyhow::anyhow!("ps error: {}", e))?;
    
    let processes = collect_processes(&options).map_err(|e| anyhow::anyhow!("ps error: {}", e))?;
    if let Err(e) = display_processes(&processes, &options) {
        return Err(anyhow::anyhow!("ps error: {}", e));
    }
    
    Ok(())
} 
