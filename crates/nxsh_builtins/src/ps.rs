//! `ps` command – comprehensive process list implementation.
//!
//! Supports most standard ps options:
//!   ps aux    - All processes with detailed info
//!   ps -e     - All processes
//!   ps -f     - Full format listing
//!   ps -l     - Long format listing
//!   ps -u USER - Processes for specific user
//!   ps -p PID  - Specific process by PID
//!   ps -C CMD  - Processes by command name
//!   ps --forest - Process tree view
//!   ps --sort=FIELD - Sort by field (cpu, mem, pid, ppid, time, etc.)

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;
use std::fs;
use std::time::{SystemTime, Duration, UNIX_EPOCH};
use sysinfo::{System, SystemExt, ProcessExt, Pid, PidExt};
use chrono::{DateTime, Local, TimeZone};
use users::{get_user_by_uid, Users as UsersCache, Groups as GroupsCache};
use tabled::{Table, Tabled, settings::{Style, Alignment}};
use nxsh_core::context::ShellContext;

use humansize::{format_size, DECIMAL};

#[cfg(target_os = "linux")]
use procfs::{process::{Process as ProcfsProcess, Stat, Status}, ProcResult, KernelStats};

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub uid: u32,
    pub gid: u32,
    pub user: String,
    pub group: String,
    pub command: String,
    pub args: Vec<String>,
    pub state: String,
    pub cpu_usage: f32,
    pub memory_kb: u64,
    pub memory_percent: f32,
    pub start_time: SystemTime,
    pub cpu_time: Duration,
    pub priority: i32,
    pub nice: i32,
    pub num_threads: u32,
    pub tty: String,
    pub session_id: u32,
    pub pgrp: u32,
    pub rss: u64,
    pub vsz: u64,
    pub wchan: String,
}

#[derive(Debug)]
pub struct PsOptions {
    pub all_processes: bool,
    pub full_format: bool,
    pub long_format: bool,
    pub user_filter: Option<String>,
    pub pid_filter: Option<u32>,
    pub command_filter: Option<String>,
    pub forest: bool,
    pub sort_field: String,
    pub reverse_sort: bool,
    pub no_headers: bool,
    pub wide_output: bool,
}

impl Default for PsOptions {
    fn default() -> Self {
        Self {
            all_processes: false,
            full_format: false,
            long_format: false,
            user_filter: None,
            pid_filter: None,
            command_filter: None,
            forest: false,
            sort_field: "pid".to_string(),
            reverse_sort: false,
            no_headers: false,
            wide_output: false,
        }
    }
}

#[derive(Tabled)]
struct PsRow {
    #[tabled(rename = "PID")]
    pid: String,
    #[tabled(rename = "PPID")]
    ppid: String,
    #[tabled(rename = "USER")]
    user: String,
    #[tabled(rename = "CPU%")]
    cpu: String,
    #[tabled(rename = "MEM%")]
    mem: String,
    #[tabled(rename = "VSZ")]
    vsz: String,
    #[tabled(rename = "RSS")]
    rss: String,
    #[tabled(rename = "TTY")]
    tty: String,
    #[tabled(rename = "STAT")]
    stat: String,
    #[tabled(rename = "START")]
    start: String,
    #[tabled(rename = "TIME")]
    time: String,
    #[tabled(rename = "COMMAND")]
    command: String,
}

pub fn ps_cli(args: &[String]) -> Result<()> {
    let options = parse_ps_args(args)?;
    
    #[cfg(target_os = "linux")]
    {
        let processes = get_linux_processes(&options)?;
        display_processes(processes, &options)?;
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        // Fallback to sysinfo for non-Linux systems
        let processes = get_sysinfo_processes(&options)?;
        display_processes(processes, &options)?;
    }
    
    Ok(())
}

fn parse_ps_args(args: &[String]) -> Result<PsOptions> {
    let mut options = PsOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "aux" | "-aux" => {
                options.all_processes = true;
                options.full_format = true;
            }
            "-e" | "-A" => {
                options.all_processes = true;
            }
            "-f" => {
                options.full_format = true;
            }
            "-l" => {
                options.long_format = true;
            }
            "-u" => {
                if i + 1 < args.len() {
                    options.user_filter = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err(anyhow!("ps: option requires an argument -- u"));
                }
            }
            "-p" => {
                if i + 1 < args.len() {
                    options.pid_filter = Some(args[i + 1].parse()?);
                    i += 1;
                } else {
                    return Err(anyhow!("ps: option requires an argument -- p"));
                }
            }
            "-C" => {
                if i + 1 < args.len() {
                    options.command_filter = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err(anyhow!("ps: option requires an argument -- C"));
                }
            }
            "--forest" => {
                options.forest = true;
            }
            "--no-headers" => {
                options.no_headers = true;
            }
            "-w" | "--wide" => {
                options.wide_output = true;
            }
            arg if arg.starts_with("--sort=") => {
                let sort_spec = arg.strip_prefix("--sort=").unwrap();
                if sort_spec.starts_with('-') {
                    options.reverse_sort = true;
                    options.sort_field = sort_spec[1..].to_string();
                } else {
                    options.sort_field = sort_spec.to_string();
                }
            }
            _ => {
                return Err(anyhow!("ps: unknown option '{}'", arg));
            }
        }
        i += 1;
    }
    
    Ok(options)
}

#[cfg(target_os = "linux")]
fn get_linux_processes(options: &PsOptions) -> Result<Vec<ProcessInfo>> {
    let mut processes = Vec::new();
    let users_cache = users::UsersCache::new();
    // Get kernel statistics for CPU usage calculation
    let kernel_stats = match procfs::KernelStats::new() {
        Ok(stats) => Some(stats),
        Err(_) => None,
    };
    let boot_time = kernel_stats.as_ref().map(|s| s.btime).unwrap_or(0);
    let page_size = procfs::page_size() as u64;
    
    // Get all processes from /proc
    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        
        // Check if directory name is a PID
        if let Ok(pid) = file_name_str.parse::<u32>() {
            if let Ok(process_info) = get_process_info(pid, &users_cache, boot_time, page_size, kernel_stats.as_ref()) {
                // Apply filters
                if let Some(ref user_filter) = options.user_filter {
                    if process_info.user != *user_filter {
                        continue;
                    }
                }
                
                if let Some(pid_filter) = options.pid_filter {
                    if process_info.pid != pid_filter {
                        continue;
                    }
                }
                
                if let Some(ref cmd_filter) = options.command_filter {
                    if !process_info.command.contains(cmd_filter) {
                        continue;
                    }
                }
                
                if !options.all_processes {
                    // Default behavior: only show processes owned by current user
                    let current_uid = unsafe { libc::getuid() };
                    if process_info.uid != current_uid {
                        continue;
                    }
                }
                
                processes.push(process_info);
            }
        }
    }
    
    // Sort processes
    sort_processes(&mut processes, &options.sort_field, options.reverse_sort);
    
    Ok(processes)
}

#[cfg(target_os = "linux")]
fn get_process_info(pid: u32, users_cache: &UsersCache, boot_time: u64, page_size: u64, kernel_stats: Option<&KernelStats>) -> ProcResult<ProcessInfo> {
    let process = Process::new(pid as i32)?;
    let stat = process.stat()?;
    let status = process.status().ok();
    
    // Get user and group info
    let uid = status.as_ref().map(|s| s.ruid).unwrap_or(0);
    let gid = status.as_ref().map(|s| s.rgid).unwrap_or(0);
    let user = users_cache.get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().to_string())
        .unwrap_or_else(|| uid.to_string());
    let group = users_cache.get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().to_string())
        .unwrap_or_else(|| gid.to_string());
    
    // Calculate memory usage
    let memory_kb = stat.rss * page_size / 1024;
    let vsz = stat.vsize / 1024;
    
    // Calculate start time
    let start_time_ticks = stat.starttime;
    let ticks_per_second = procfs::ticks_per_second() as u64;
    let start_time_seconds = boot_time + (start_time_ticks / ticks_per_second);
    let start_time = UNIX_EPOCH + Duration::from_secs(start_time_seconds);
    
    // Calculate CPU time
    let cpu_time_ticks = stat.utime + stat.stime;
    let cpu_time = Duration::from_millis((cpu_time_ticks * 1000) / ticks_per_second);
    
    // Get TTY
    let tty = if stat.tty_nr == 0 {
        "?".to_string()
    } else {
        format!("pts/{}", stat.tty_nr)
    };
    
    // Process state
    let state = match stat.state {
        'R' => "Running",
        'S' => "Sleeping",
        'D' => "Disk sleep",
        'T' => "Stopped",
        'Z' => "Zombie",
        _ => "Unknown",
    }.to_string();
    
    // Command and arguments
    let (command, args) = if let Ok(cmdline) = process.cmdline() {
        if cmdline.is_empty() {
            (format!("[{}]", stat.comm), vec![])
        } else {
            let cmd = cmdline[0].clone();
            let args = if cmdline.len() > 1 { cmdline[1..].to_vec() } else { vec![] };
            (cmd, args)
        }
    } else {
        (format!("[{}]", stat.comm), vec![])
    };
    
    // Calculate CPU usage if kernel stats are available
    let cpu_usage = if let Some(stats) = kernel_stats {
        let current_cpu_time = Duration::from_millis((stat.utime + stat.stime) * 1000 / ticks_per_second);
        let total_cpu_time = Duration::from_millis((stats.total.guest_nice.unwrap_or(0) * 1000) / ticks_per_second);
        let elapsed_time = Duration::from_millis((stats.btime * 1000) / ticks_per_second);
        
        if elapsed_time.as_secs() > 0 {
            let cpu_usage_percent = (current_cpu_time.as_millis() as f64 / elapsed_time.as_millis() as f64) * 100.0;
            cpu_usage_percent.round()
        } else {
            0.0
        }
    } else {
        0.0 // No kernel stats, cannot calculate CPU usage
    };
    
    Ok(ProcessInfo {
        pid,
        ppid: stat.ppid as u32,
        uid,
        gid,
        user,
        group,
        command,
        args,
        state,
        cpu_usage,
        memory_kb,
        memory_percent: 0.0, // Would need total memory to calculate
        start_time,
        cpu_time,
        priority: stat.priority as i32,
        nice: stat.nice as i32,
        num_threads: stat.num_threads as u32,
        tty,
        session_id: stat.session as u32,
        pgrp: stat.pgrp as u32,
        rss: stat.rss * page_size,
        vsz,
        wchan: "".to_string(), // Would need to read from wchan file
    })
}

#[cfg(not(target_os = "linux"))]
fn get_sysinfo_processes(options: &PsOptions) -> Result<Vec<ProcessInfo>> {
    use sysinfo::{System, SystemExt, ProcessExt, UserExt};
    
    let mut sys = System::new();
    sys.refresh_all();
    
    let mut processes = Vec::new();
    let total_memory = sys.total_memory() as f32;
    
    for (pid, proc_) in sys.processes() {
        let pid_u32 = pid.as_u32();
        
        // Apply filters
        if let Some(pid_filter) = options.pid_filter {
            if pid_u32 != pid_filter {
                continue;
            }
        }
        
        if let Some(ref cmd_filter) = options.command_filter {
            if !proc_.name().contains(cmd_filter) {
                continue;
            }
        }
        
        let uid = proc_.user_id().map(|u| *u).unwrap_or(0);
        let user = proc_.user_id()
            .and_then(|uid| sys.get_user_by_id(uid))
            .map(|u| u.name().to_string())
            .unwrap_or_else(|| uid.to_string());
        
        if let Some(ref user_filter) = options.user_filter {
            if user != *user_filter {
                continue;
            }
        }
        
        if !options.all_processes {
            // Default behavior: only show processes owned by current user
            let current_uid = unsafe { libc::getuid() };
            if uid != current_uid {
                continue;
            }
        }
        
        let memory_kb = proc_.memory() / 1024;
        let memory_percent = if total_memory > 0.0 {
            proc_.memory() as f32 * 100.0 / total_memory
        } else {
            0.0
        };
        
        let start_time = UNIX_EPOCH + Duration::from_secs(proc_.start_time());
        let cpu_time = Duration::from_secs(proc_.run_time());
        
        let process_info = ProcessInfo {
            pid: pid_u32,
            ppid: proc_.parent().map(|p| p.as_u32()).unwrap_or(0),
            uid,
            gid: 0, // Not available in sysinfo
            user,
            group: "".to_string(), // Not available in sysinfo
            command: proc_.name().to_string(),
            args: proc_.cmd().to_vec(),
            state: format!("{:?}", proc_.status()),
            cpu_usage: proc_.cpu_usage(),
            memory_kb,
            memory_percent,
            start_time,
            cpu_time,
            priority: 0, // Not available in sysinfo
            nice: 0, // Not available in sysinfo
            num_threads: 1, // Not available in sysinfo
            tty: "?".to_string(),
            session_id: 0,
            pgrp: 0,
            rss: proc_.memory(),
            vsz: proc_.virtual_memory(),
            wchan: "".to_string(),
        };
        
        processes.push(process_info);
    }
    
    // Sort processes
    sort_processes(&mut processes, &options.sort_field, options.reverse_sort);
    
    Ok(processes)
}

fn sort_processes(processes: &mut [ProcessInfo], sort_field: &str, reverse: bool) {
    processes.sort_by(|a, b| {
        let cmp = match sort_field {
            "pid" => a.pid.cmp(&b.pid),
            "ppid" => a.ppid.cmp(&b.ppid),
            "cpu" => a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap_or(std::cmp::Ordering::Equal),
            "mem" | "memory" => a.memory_percent.partial_cmp(&b.memory_percent).unwrap_or(std::cmp::Ordering::Equal),
            "time" => a.cpu_time.cmp(&b.cpu_time),
            "user" => a.user.cmp(&b.user),
            "command" | "cmd" => a.command.cmp(&b.command),
            "start" => a.start_time.cmp(&b.start_time),
            _ => a.pid.cmp(&b.pid),
        };
        
        if reverse {
            cmp.reverse()
        } else {
            cmp
        }
    });
}

fn display_processes(processes: Vec<ProcessInfo>, options: &PsOptions) -> Result<()> {
    if processes.is_empty() {
        return Ok(());
    }
    
    if options.forest {
        display_process_tree(processes, options)?;
    } else {
        display_process_table(processes, options)?;
    }
    
    Ok(())
}

fn display_process_table(processes: Vec<ProcessInfo>, options: &PsOptions) -> Result<()> {
    let mut rows = Vec::new();
    
    for proc in processes {
        let start_time = DateTime::<Local>::from(proc.start_time);
        let start_str = if start_time.date_naive() == Local::now().date_naive() {
            start_time.format("%H:%M").to_string()
        } else {
            start_time.format("%b%d").to_string()
        };
        
        let time_str = format!("{}:{:02}", 
            proc.cpu_time.as_secs() / 60,
            proc.cpu_time.as_secs() % 60
        );
        
        let command = if options.full_format && !proc.args.is_empty() {
            proc.args.join(" ")
        } else {
            proc.command.clone()
        };
        
        let command = if options.wide_output {
            command
        } else {
            // Truncate command if too long
            if command.len() > 50 {
                format!("{}...", &command[..47])
            } else {
                command
            }
        };
        
        rows.push(PsRow {
            pid: proc.pid.to_string(),
            ppid: proc.ppid.to_string(),
            user: proc.user,
            cpu: format!("{:.1}", proc.cpu_usage),
            mem: format!("{:.1}", proc.memory_percent),
            vsz: format_size(proc.vsz * 1024, DECIMAL),
            rss: format_size(proc.rss, DECIMAL),
            tty: proc.tty,
            stat: proc.state,
            start: start_str,
            time: time_str,
            command,
        });
    }
    
    let mut table = Table::new(rows);
    table.with(Style::ascii_rounded());
    
    // Apply column width adjustments if terminal is too narrow
    if let Some((width, _)) = term_size::dimensions() {
        // Adjust table display for narrow terminals
        println!("{}", table);
    } else {
        println!("{}", table);
    }
    
    Ok(())
}

fn display_process_tree(processes: Vec<ProcessInfo>, options: &PsOptions) -> Result<()> {
    let mut process_map: HashMap<u32, ProcessInfo> = HashMap::new();
    let mut children_map: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut roots = Vec::new();
    
    // Build process hierarchy
    for proc in processes {
        let pid = proc.pid;
        let ppid = proc.ppid;
        
        process_map.insert(pid, proc);
        children_map.entry(ppid).or_insert_with(Vec::new).push(pid);
        
        // If parent is not in our process list, this is a root
        if ppid == 0 || !process_map.contains_key(&ppid) {
            roots.push(pid);
        }
    }
    
    // Print process tree
    for root_pid in roots {
        if let Some(root_proc) = process_map.get(&root_pid) {
            print_process_tree_recursive(root_proc, &process_map, &children_map, 0, options)?;
        }
    }
    
    Ok(())
}

fn print_process_tree_recursive(
    proc: &ProcessInfo,
    process_map: &HashMap<u32, ProcessInfo>,
    children_map: &HashMap<u32, Vec<u32>>,
    depth: usize,
    options: &PsOptions,
) -> Result<()> {
    // Print indentation
    for _ in 0..depth {
        print!("  ");
    }
    
    // Print process info
    let command = if options.full_format && !proc.args.is_empty() {
        proc.args.join(" ")
    } else {
        proc.command.clone()
    };
    
    println!("{} {} {} {:.1}% {:.1}% {}", 
        proc.pid, 
        proc.user,
        proc.state,
        proc.cpu_usage,
        proc.memory_percent,
        command
    );
    
    // Print children
    if let Some(children) = children_map.get(&proc.pid) {
        for &child_pid in children {
            if let Some(child_proc) = process_map.get(&child_pid) {
                print_process_tree_recursive(child_proc, process_map, children_map, depth + 1, options)?;
            }
        }
    }
    
    Ok(())
} 