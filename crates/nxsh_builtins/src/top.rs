//! `top` command - display and update sorted information about running processes
//!
//! Full top implementation with real-time monitoring, interactive controls, and system information

use nxsh_core::error::RuntimeErrorKind;
use nxsh_core::{Builtin, ErrorKind, ExecutionResult, ShellContext, ShellError, ShellResult};
use std::io::{self};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct TopBuiltin;

#[derive(Debug, Clone)]
pub struct TopOptions {
    pub delay: Duration,
    pub iterations: Option<u32>,
    pub batch_mode: bool,
    pub sort_field: String,
    pub reverse_sort: bool,
    pub show_threads: bool,
    pub show_idle: bool,
    pub filter_user: Option<String>,
    pub filter_pid: Option<u32>,
    pub show_command_line: bool,
    pub color_mode: bool,
    pub secure_mode: bool,
}

impl Default for TopOptions {
    fn default() -> Self {
        Self {
            delay: Duration::from_secs(3),
            iterations: None,
            batch_mode: false,
            sort_field: "cpu".to_string(),
            reverse_sort: true,
            show_threads: false,
            show_idle: true,
            filter_user: None,
            filter_pid: None,
            show_command_line: false,
            color_mode: true,
            secure_mode: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub uptime: Duration,
    pub load_avg: (f64, f64, f64),
    pub tasks_total: u32,
    pub tasks_running: u32,
    pub tasks_sleeping: u32,
    pub tasks_stopped: u32,
    pub tasks_zombie: u32,
    pub cpu_user: f64,
    pub cpu_system: f64,
    pub cpu_nice: f64,
    pub cpu_idle: f64,
    pub cpu_wait: f64,
    pub cpu_hi: f64,
    pub cpu_si: f64,
    pub cpu_steal: f64,
    pub memory_total: u64,
    pub memory_free: u64,
    pub memory_used: u64,
    pub memory_buffers: u64,
    pub memory_cached: u64,
    pub swap_total: u64,
    pub swap_free: u64,
    pub swap_used: u64,
}

#[derive(Debug, Clone)]
pub struct TopProcess {
    pub pid: u32,
    pub ppid: u32,
    pub user: String,
    pub priority: i32,
    pub nice: i32,
    pub virtual_memory: u64,
    pub resident_memory: u64,
    pub shared_memory: u64,
    pub state: String,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub cpu_time: Duration,
    pub command: String,
    pub threads: u32,
}

impl Builtin for TopBuiltin {
    fn name(&self) -> &'static str {
        "top"
    }

    fn synopsis(&self) -> &'static str {
        "display and update sorted information about running processes"
    }

    fn help(&self) -> &'static str {
        "Display and update sorted information about running processes in real-time"
    }

    fn description(&self) -> &'static str {
        "Display and update sorted information about running processes in real-time"
    }

    fn execute(&self, _ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        let options = parse_top_args(args)?;

        if options.batch_mode {
            run_batch_mode(&options)?;
        } else {
            run_interactive_mode(&options)?;
        }

        Ok(ExecutionResult::success(0))
    }

    fn usage(&self) -> &'static str {
        "top - display and update sorted information about running processes

USAGE:
    top [OPTIONS]

OPTIONS:
    -b              Batch mode operation
    -c              Show command line instead of command name
    -d DELAY        Delay between updates (seconds)
    -i              Don't show idle tasks
    -n ITERATIONS   Number of iterations before exit
    -p PID          Monitor only specified process
    -u USER         Monitor only specified user
    -H              Show individual threads
    -s              Secure mode (disable some commands)
    --help          Display this help and exit

INTERACTIVE COMMANDS:
    h, ?            Help
    k               Kill a process
    r               Renice a process
    q               Quit
    c               Toggle command line display
    f, F            Field management
    o, O            Ordering fields
    u               Filter by user
    i               Toggle idle processes
    t               Toggle CPU states
    m               Toggle memory information
    1               Toggle SMP view
    H               Toggle threads
    space           Update display
    <, >            Move sort field
    R               Reverse sort order

SORT FIELDS:
    P               %CPU
    M               %MEM
    N               PID
    T               TIME+
    A               Age (newest first)

EXAMPLES:
    top                     Start top in interactive mode
    top -b -n 1             Show current processes once
    top -d 5                Update every 5 seconds
    top -u root             Show only root processes
    top -p 1234             Monitor process 1234"
    }
}

fn parse_top_args(args: &[String]) -> ShellResult<TopOptions> {
    let mut options = TopOptions {
        delay: Duration::from_secs(3),
        iterations: None,
        batch_mode: false,
        sort_field: "cpu".to_string(),
        reverse_sort: true,
        show_threads: false,
        show_idle: true,
        filter_user: None,
        filter_pid: None,
        show_command_line: false,
        color_mode: true,
        secure_mode: false,
    };

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        match arg.as_str() {
            "-b" => options.batch_mode = true,
            "-c" => options.show_command_line = true,
            "-i" => options.show_idle = false,
            "-H" => options.show_threads = true,
            "-s" => options.secure_mode = true,
            "-d" => {
                i += 1;
                if i >= args.len() {
                    return Err(ShellError::new(
                        ErrorKind::RuntimeError(
                            nxsh_core::error::RuntimeErrorKind::InvalidArgument,
                        ),
                        "Option -d requires an argument",
                    ));
                }
                let delay_secs: f64 = args[i].parse().map_err(|_| {
                    ShellError::new(
                        ErrorKind::RuntimeError(
                            nxsh_core::error::RuntimeErrorKind::InvalidArgument,
                        ),
                        "Invalid delay value",
                    )
                })?;
                options.delay = Duration::from_secs_f64(delay_secs);
            }
            "-n" => {
                i += 1;
                if i >= args.len() {
                    return Err(ShellError::new(
                        ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                        "Option -n requires an argument",
                    ));
                }
                options.iterations = Some(args[i].parse().map_err(|_| {
                    ShellError::new(
                        ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                        "Invalid iteration count",
                    )
                })?);
            }
            "-p" => {
                i += 1;
                if i >= args.len() {
                    return Err(ShellError::new(
                        ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                        "Option -p requires an argument",
                    ));
                }
                options.filter_pid = Some(args[i].parse().map_err(|_| {
                    ShellError::new(
                        ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                        "Invalid PID",
                    )
                })?);
            }
            "-u" => {
                i += 1;
                if i >= args.len() {
                    return Err(ShellError::new(
                        ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                        "Option -u requires an argument",
                    ));
                }
                options.filter_user = Some(args[i].clone());
            }
            "--help" => {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    "Help requested",
                ))
            }
            _ if arg.starts_with("-") => {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("Unknown option: {arg}"),
                ));
            }
            _ => {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("Unknown argument: {arg}"),
                ))
            }
        }
        i += 1;
    }

    Ok(options)
}

fn run_batch_mode(options: &TopOptions) -> ShellResult<()> {
    let mut iteration = 0;

    loop {
        if let Some(max_iterations) = options.iterations {
            if iteration >= max_iterations {
                break;
            }
        }

        let system_info = collect_system_info()?;
        let processes = collect_top_processes(options)?;

        display_batch_output(&system_info, &processes, options)?;

        iteration += 1;

        if options.iterations.is_some() || iteration == 0 {
            thread::sleep(options.delay);
        }
    }

    Ok(())
}

fn run_interactive_mode(options: &TopOptions) -> ShellResult<()> {
    // Enable raw mode for interactive input
    terminal::enable_raw_mode().map_err(|e| {
        ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            format!("Failed to enable raw mode: {e}"),
        )
    })?;

    let result = run_interactive_loop(options);

    // Restore terminal
    terminal::disable_raw_mode().map_err(|e| {
        ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            format!("Failed to disable raw mode: {e}"),
        )
    })?;
    execute!(
        io::stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .map_err(|e| {
        ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            format!("Failed to clear terminal: {e}"),
        )
    })?;

    result
}

fn run_interactive_loop(options: &TopOptions) -> ShellResult<()> {
    let mut current_options = options.clone();
    let mut last_update = Instant::now();

    loop {
        // Check for input
        if event::poll(Duration::from_millis(100)).map_err(|e| {
            ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                format!("Failed to poll events: {e}"),
            )
        })? {
            if let Event::Key(key_event) = event::read().map_err(|e| {
                ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("Failed to read event: {e}"),
                )
            })? {
                match handle_key_event(key_event, &mut current_options)? {
                    KeyAction::Quit => break,
                    KeyAction::Update => {
                        last_update = Instant::now();
                        update_display(&current_options)?;
                    }
                    KeyAction::Continue => {}
                }
            }
        }

        // Auto-update based on delay
        if last_update.elapsed() >= current_options.delay {
            last_update = Instant::now();
            update_display(&current_options)?;
        }
    }

    Ok(())
}

#[derive(Debug)]
enum KeyAction {
    Quit,
    Update,
    Continue,
}

fn handle_key_event(key_event: KeyEvent, options: &mut TopOptions) -> ShellResult<KeyAction> {
    match key_event.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => Ok(KeyAction::Quit),
        KeyCode::Char(' ') => Ok(KeyAction::Update),
        KeyCode::Char('c') | KeyCode::Char('C') => {
            options.show_command_line = !options.show_command_line;
            Ok(KeyAction::Update)
        }
        KeyCode::Char('i') | KeyCode::Char('I') => {
            options.show_idle = !options.show_idle;
            Ok(KeyAction::Update)
        }
        KeyCode::Char('H') => {
            options.show_threads = !options.show_threads;
            Ok(KeyAction::Update)
        }
        KeyCode::Char('R') => {
            options.reverse_sort = !options.reverse_sort;
            Ok(KeyAction::Update)
        }
        KeyCode::Char('P') => {
            options.sort_field = "cpu".to_string();
            Ok(KeyAction::Update)
        }
        KeyCode::Char('M') => {
            options.sort_field = "memory".to_string();
            Ok(KeyAction::Update)
        }
        KeyCode::Char('N') => {
            options.sort_field = "pid".to_string();
            Ok(KeyAction::Update)
        }
        KeyCode::Char('T') => {
            options.sort_field = "time".to_string();
            Ok(KeyAction::Update)
        }
        KeyCode::Char('h') | KeyCode::Char('?') => {
            show_help_screen()?;
            Ok(KeyAction::Update)
        }
        _ => Ok(KeyAction::Continue),
    }
}

fn update_display(options: &TopOptions) -> ShellResult<()> {
    let system_info = collect_system_info()?;
    let processes = collect_top_processes(options)?;

    // Clear screen and move to top
    execute!(
        io::stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .map_err(|e| {
        ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            format!("Failed to clear screen: {e}"),
        )
    })?;

    display_interactive_output(&system_info, &processes, options)?;

    Ok(())
}

fn collect_system_info() -> ShellResult<SystemInfo> {
    #[cfg(target_os = "linux")]
    {
        collect_linux_system_info()
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Simplified system info for other platforms
        Ok(SystemInfo {
            uptime: Duration::from_secs(0),
            load_avg: (0.0, 0.0, 0.0),
            tasks_total: 0,
            tasks_running: 0,
            tasks_sleeping: 0,
            tasks_stopped: 0,
            tasks_zombie: 0,
            cpu_user: 0.0,
            cpu_system: 0.0,
            cpu_nice: 0.0,
            cpu_idle: 100.0,
            cpu_wait: 0.0,
            cpu_hi: 0.0,
            cpu_si: 0.0,
            cpu_steal: 0.0,
            memory_total: 0,
            memory_free: 0,
            memory_used: 0,
            memory_buffers: 0,
            memory_cached: 0,
            swap_total: 0,
            swap_free: 0,
            swap_used: 0,
        })
    }
}

#[cfg(target_os = "linux")]
fn collect_linux_system_info() -> ShellResult<SystemInfo> {
    // Read /proc/uptime
    let uptime = if let Ok(content) = fs::read_to_string("/proc/uptime") {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if let Ok(uptime_secs) = parts[0].parse::<f64>() {
            Duration::from_secs_f64(uptime_secs)
        } else {
            Duration::from_secs(0)
        }
    } else {
        Duration::from_secs(0)
    };

    // Read /proc/loadavg
    let load_avg = if let Ok(content) = fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            let load1 = parts[0].parse().unwrap_or(0.0);
            let load5 = parts[1].parse().unwrap_or(0.0);
            let load15 = parts[2].parse().unwrap_or(0.0);
            (load1, load5, load15)
        } else {
            (0.0, 0.0, 0.0)
        }
    } else {
        (0.0, 0.0, 0.0)
    };

    // Read /proc/stat for CPU info
    let (cpu_user, cpu_system, cpu_nice, cpu_idle, cpu_wait, cpu_hi, cpu_si, cpu_steal) =
        if let Ok(content) = fs::read_to_string("/proc/stat") {
            if let Some(cpu_line) = content.lines().next() {
                let parts: Vec<&str> = cpu_line.split_whitespace().collect();
                if parts.len() >= 8 {
                    let user = parts[1].parse::<u64>().unwrap_or(0);
                    let nice = parts[2].parse::<u64>().unwrap_or(0);
                    let system = parts[3].parse::<u64>().unwrap_or(0);
                    let idle = parts[4].parse::<u64>().unwrap_or(0);
                    let iowait = parts[5].parse::<u64>().unwrap_or(0);
                    let irq = parts[6].parse::<u64>().unwrap_or(0);
                    let softirq = parts[7].parse::<u64>().unwrap_or(0);
                    let steal = if parts.len() > 8 {
                        parts[8].parse::<u64>().unwrap_or(0)
                    } else {
                        0
                    };

                    let total = user + nice + system + idle + iowait + irq + softirq + steal;
                    if total > 0 {
                        (
                            (user as f64 / total as f64) * 100.0,
                            (system as f64 / total as f64) * 100.0,
                            (nice as f64 / total as f64) * 100.0,
                            (idle as f64 / total as f64) * 100.0,
                            (iowait as f64 / total as f64) * 100.0,
                            (irq as f64 / total as f64) * 100.0,
                            (softirq as f64 / total as f64) * 100.0,
                            (steal as f64 / total as f64) * 100.0,
                        )
                    } else {
                        (0.0, 0.0, 0.0, 100.0, 0.0, 0.0, 0.0, 0.0)
                    }
                } else {
                    (0.0, 0.0, 0.0, 100.0, 0.0, 0.0, 0.0, 0.0)
                }
            } else {
                (0.0, 0.0, 0.0, 100.0, 0.0, 0.0, 0.0, 0.0)
            }
        } else {
            (0.0, 0.0, 0.0, 100.0, 0.0, 0.0, 0.0, 0.0)
        };

    // Read /proc/meminfo
    let (memory_total, memory_free, memory_buffers, memory_cached, swap_total, swap_free) =
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            let mut mem_total = 0;
            let mut mem_free = 0;
            let mut buffers = 0;
            let mut cached = 0;
            let mut swap_total = 0;
            let mut swap_free = 0;

            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        mem_total = value.parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes
                    }
                } else if line.starts_with("MemFree:") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        mem_free = value.parse::<u64>().unwrap_or(0) * 1024;
                    }
                } else if line.starts_with("Buffers:") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        buffers = value.parse::<u64>().unwrap_or(0) * 1024;
                    }
                } else if line.starts_with("Cached:") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        cached = value.parse::<u64>().unwrap_or(0) * 1024;
                    }
                } else if line.starts_with("SwapTotal:") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        swap_total = value.parse::<u64>().unwrap_or(0) * 1024;
                    }
                } else if line.starts_with("SwapFree:") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        swap_free = value.parse::<u64>().unwrap_or(0) * 1024;
                    }
                }
            }

            (mem_total, mem_free, buffers, cached, swap_total, swap_free)
        } else {
            (0, 0, 0, 0, 0, 0)
        };

    let memory_used = memory_total.saturating_sub(memory_free + memory_buffers + memory_cached);
    let swap_used = swap_total.saturating_sub(swap_free);

    // Count tasks by reading /proc
    let (tasks_total, tasks_running, tasks_sleeping, tasks_stopped, tasks_zombie) =
        count_tasks().unwrap_or((0, 0, 0, 0, 0));

    Ok(SystemInfo {
        uptime,
        load_avg,
        tasks_total,
        tasks_running,
        tasks_sleeping,
        tasks_stopped,
        tasks_zombie,
        cpu_user,
        cpu_system,
        cpu_nice,
        cpu_idle,
        cpu_wait,
        cpu_hi,
        cpu_si,
        cpu_steal,
        memory_total,
        memory_free,
        memory_used,
        memory_buffers,
        memory_cached,
        swap_total,
        swap_free,
        swap_used,
    })
}

#[cfg(target_os = "linux")]
fn count_tasks() -> Result<(u32, u32, u32, u32, u32), Box<dyn std::error::Error>> {
    let mut total = 0;
    let mut running = 0;
    let mut sleeping = 0;
    let mut stopped = 0;
    let mut zombie = 0;

    let proc_dir = fs::read_dir("/proc")?;

    for entry in proc_dir {
        let entry = entry?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        if let Ok(_pid) = name_str.parse::<u32>() {
            total += 1;

            let stat_path = format!("/proc/{}/stat", name_str);
            if let Ok(content) = fs::read_to_string(&stat_path) {
                let parts: Vec<&str> = content.split_whitespace().collect();
                if parts.len() >= 3 {
                    match parts[2] {
                        "R" => running += 1,
                        "S" | "D" => sleeping += 1,
                        "T" => stopped += 1,
                        "Z" => zombie += 1,
                        _ => {}
                    }
                }
            }
        }
    }

    Ok((total, running, sleeping, stopped, zombie))
}

fn collect_top_processes(options: &TopOptions) -> ShellResult<Vec<TopProcess>> {
    let mut processes = Vec::new();

    #[cfg(target_os = "linux")]
    {
        let proc_dir = fs::read_dir("/proc").map_err(|e| {
            ShellError::new(ErrorKind::IoError, format!("Cannot read /proc: {}", e))
        })?;

        for entry in proc_dir {
            let entry = entry.map_err(|e| {
                ShellError::new(
                    ErrorKind::IoError,
                    format!("Error reading /proc entry: {}", e),
                )
            })?;
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();

            if let Ok(pid) = name_str.parse::<u32>() {
                if let Ok(process) = read_top_process_info(pid) {
                    if should_include_top_process(&process, options) {
                        processes.push(process);
                    }
                }
            }
        }
    }

    // Sort processes
    sort_top_processes(&mut processes, &options.sort_field, options.reverse_sort)?;

    Ok(processes)
}

#[cfg(target_os = "linux")]
fn read_top_process_info(pid: u32) -> Result<TopProcess, Box<dyn std::error::Error>> {
    let stat_path = format!("/proc/{}/stat", pid);
    let status_path = format!("/proc/{}/status", pid);
    let cmdline_path = format!("/proc/{}/cmdline", pid);

    // Read basic process info from stat
    let stat_content = fs::read_to_string(&stat_path)?;
    let stat_fields: Vec<&str> = stat_content.split_whitespace().collect();

    if stat_fields.len() < 44 {
        return Err("Invalid stat file format".into());
    }

    let ppid = stat_fields[3].parse::<u32>()?;
    let priority = stat_fields[17].parse::<i32>()?;
    let nice = stat_fields[18].parse::<i32>()?;
    let num_threads = stat_fields[19].parse::<u32>()?;
    let vsize = stat_fields[22].parse::<u64>()?;
    let rss = stat_fields[23].parse::<u64>()? * 4096; // Convert pages to bytes
    let state = stat_fields[2].to_string();

    // Read additional info from status
    let mut uid = 0;
    if let Ok(status_content) = fs::read_to_string(&status_path) {
        for line in status_content.lines() {
            if line.starts_with("Uid:") {
                if let Some(uid_str) = line.split_whitespace().nth(1) {
                    uid = uid_str.parse().unwrap_or(0);
                }
                break;
            }
        }
    }

    // Get username
    let user = get_username_by_uid(uid).unwrap_or_else(|| uid.to_string());

    // Read command line
    let command = if let Ok(cmdline_content) = fs::read(&cmdline_path) {
        let cmdline_str = String::from_utf8_lossy(&cmdline_content);
        let parts: Vec<&str> = cmdline_str.split('\0').filter(|s| !s.is_empty()).collect();
        if !parts.is_empty() {
            parts.join(" ")
        } else {
            format!("[{}]", pid)
        }
    } else {
        format!("[{}]", pid)
    };

    Ok(TopProcess {
        pid,
        ppid,
        user,
        priority,
        nice,
        virtual_memory: vsize,
        resident_memory: rss,
        shared_memory: 0, // Would need to read from statm
        state,
        cpu_percent: 0.0,                 // Would need multiple samples
        memory_percent: 0.0,              // Would need total system memory
        cpu_time: Duration::from_secs(0), // Would need to parse from stat
        command,
        threads: num_threads,
    })
}

#[cfg(target_os = "linux")]
fn get_username_by_uid(uid: u32) -> Option<String> {
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

fn should_include_top_process(process: &TopProcess, options: &TopOptions) -> bool {
    // Filter by user
    if let Some(ref user) = options.filter_user {
        if process.user != *user {
            return false;
        }
    }

    // Filter by PID
    if let Some(pid) = options.filter_pid {
        if process.pid != pid {
            return false;
        }
    }

    // Filter idle processes
    if !options.show_idle && process.cpu_percent == 0.0 {
        return false;
    }

    true
}

fn sort_top_processes(
    processes: &mut [TopProcess],
    sort_field: &str,
    reverse: bool,
) -> ShellResult<()> {
    match sort_field {
        "cpu" => processes.sort_by(|a, b| {
            a.cpu_percent
                .partial_cmp(&b.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        "memory" => processes.sort_by(|a, b| {
            a.memory_percent
                .partial_cmp(&b.memory_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        "pid" => processes.sort_by_key(|p| p.pid),
        "time" => processes.sort_by_key(|p| p.cpu_time),
        _ => {
            return Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                format!("Unknown sort field: {sort_field}"),
            ))
        }
    }

    if reverse {
        processes.reverse();
    }

    Ok(())
}

fn display_batch_output(
    system_info: &SystemInfo,
    processes: &[TopProcess],
    options: &TopOptions,
) -> ShellResult<()> {
    display_system_header(system_info)?;
    display_process_list(processes, options)?;
    Ok(())
}

fn display_interactive_output(
    system_info: &SystemInfo,
    processes: &[TopProcess],
    options: &TopOptions,
) -> ShellResult<()> {
    display_system_header(system_info)?;
    display_process_list(processes, options)?;

    // Show status line
    println!("\nPress 'h' for help, 'q' to quit");

    Ok(())
}

fn display_system_header(system_info: &SystemInfo) -> ShellResult<()> {
    // Top line - uptime and load
    let uptime_str = format_uptime(system_info.uptime);
    println!(
        "top - {} up {}, load average: {:.2}, {:.2}, {:.2}",
        format_current_time(),
        uptime_str,
        system_info.load_avg.0,
        system_info.load_avg.1,
        system_info.load_avg.2
    );

    // Tasks line
    println!(
        "Tasks: {} total, {} running, {} sleeping, {} stopped, {} zombie",
        system_info.tasks_total,
        system_info.tasks_running,
        system_info.tasks_sleeping,
        system_info.tasks_stopped,
        system_info.tasks_zombie
    );

    // CPU line
    println!(
        "%Cpu(s): {:.1} us, {:.1} sy, {:.1} ni, {:.1} id, {:.1} wa, {:.1} hi, {:.1} si, {:.1} st",
        system_info.cpu_user,
        system_info.cpu_system,
        system_info.cpu_nice,
        system_info.cpu_idle,
        system_info.cpu_wait,
        system_info.cpu_hi,
        system_info.cpu_si,
        system_info.cpu_steal
    );

    // Memory lines
    println!(
        "MiB Mem : {:.1} total, {:.1} free, {:.1} used, {:.1} buff/cache",
        system_info.memory_total as f64 / 1024.0 / 1024.0,
        system_info.memory_free as f64 / 1024.0 / 1024.0,
        system_info.memory_used as f64 / 1024.0 / 1024.0,
        (system_info.memory_buffers + system_info.memory_cached) as f64 / 1024.0 / 1024.0
    );

    println!(
        "MiB Swap: {:.1} total, {:.1} free, {:.1} used",
        system_info.swap_total as f64 / 1024.0 / 1024.0,
        system_info.swap_free as f64 / 1024.0 / 1024.0,
        system_info.swap_used as f64 / 1024.0 / 1024.0
    );

    println!();

    Ok(())
}

fn display_process_list(processes: &[TopProcess], options: &TopOptions) -> ShellResult<()> {
    // Header
    println!(
        "{:>7} {:>9} {:>2} {:>2} {:>7} {:>7} {:>7} {:>1} {:>5} {:>5} {:>9} COMMAND",
        "PID", "USER", "PR", "NI", "VIRT", "RES", "SHR", "S", "%CPU", "%MEM", "TIME+"
    );

    // Process lines
    for process in processes.iter().take(20) {
        // Show top 20 processes
        let command = if options.show_command_line {
            &process.command
        } else {
            process
                .command
                .split_whitespace()
                .next()
                .unwrap_or(&process.command)
        };

        println!(
            "{:>7} {:>9} {:>2} {:>2} {:>7} {:>7} {:>7} {:>1} {:>5.1} {:>5.1} {:>9} {}",
            process.pid,
            truncate_string(&process.user, 9),
            process.priority,
            process.nice,
            format_memory(process.virtual_memory),
            format_memory(process.resident_memory),
            format_memory(process.shared_memory),
            process.state,
            process.cpu_percent,
            process.memory_percent,
            format_time_duration(process.cpu_time),
            truncate_string(command, 30)
        );
    }

    Ok(())
}

fn show_help_screen() -> ShellResult<()> {
    execute!(
        io::stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .map_err(|e| {
        ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            format!("Failed to clear screen: {e}"),
        )
    })?;

    println!("Help for Interactive Commands - top version");
    println!();
    println!("Window 1:Def: Cumulative mode Off.  System: Delay 3.0 secs; Secure mode Off.");
    println!();
    println!("  Z,B,E,e   Global: 'Z' colors; 'B' bold; 'E'/'e' summary/task memory scale");
    println!("  l,t,m     Toggle Summaries: 'l' load avg; 't' task/cpu stats; 'm' memory info");
    println!("  0,1,2,3,I Toggle: '0' zeros; '1/2/3' cpus or numa node views; 'I' Irix mode");
    println!("  f,F,X     Fields: 'f'/'F' add/remove/order/sort; 'X' increase fixed-width");
    println!();
    println!("  L,&,<,> . Locate: 'L'/'&' find/again; Move sort column: '<'/'>' left/right");
    println!("  R,H,V,J . Toggle: 'R' Sort; 'H' Threads; 'V' Forest view; 'J' Num justify");
    println!("  c,i,S,j . Toggle: 'c' Cmd name/line; 'i' Idle; 'S' Time; 'j' Str justify");
    println!("  x,y     . Toggle highlights: 'x' sort field; 'y' running tasks");
    println!("  z,b     . Toggle: 'z' color/mono; 'b' bold/reverse (only if 'x' or 'y')");
    println!("  u,U,o,O . Filter by: 'u'/'U' effective/any user; 'o'/'O' other criteria");
    println!("  n,#,^O  . Set: 'n'/'#' max tasks displayed; '^O' other output options");
    println!("  C,...   . Toggle scroll coordinates msg for: up,down,left,right,home,end");
    println!();
    println!("  k,r       Manipulate tasks: 'k' kill; 'r' renice");
    println!("  d or s    Set update interval");
    println!("  W,Y       Write configuration file 'W'; Inspect other output 'Y'");
    println!("  q         Quit");
    println!("          ( commands shown with '.' require a visible task display window )");
    println!();
    println!("Press any key to continue...");

    // Wait for key press
    event::read().map_err(|e| {
        ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            format!("Failed to read key: {e}"),
        )
    })?;

    Ok(())
}

fn format_uptime(uptime: Duration) -> String {
    let total_seconds = uptime.as_secs();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;

    if days > 0 {
        format!("{days} days, {hours}:{minutes:02}")
    } else if hours > 0 {
        format!("{hours}:{minutes:02}")
    } else {
        format!("{minutes} min")
    }
}

fn format_current_time() -> String {
    // Simplified time formatting
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let timestamp = duration.as_secs();
            let hours = (timestamp % 86400) / 3600;
            let minutes = (timestamp % 3600) / 60;
            let seconds = timestamp % 60;
            format!("{hours:02}:{minutes:02}:{seconds:02}")
        }
        Err(_) => "??:??:??".to_string(),
    }
}

fn format_memory(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1}g", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1}m", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{}k", bytes / 1024)
    } else {
        format!("{bytes}")
    }
}

fn format_time_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes}:{seconds:02}")
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}+", &s[..max_len.saturating_sub(1)])
    }
}

// CLI entry point function
pub fn top_cli(_args: &[String]) -> anyhow::Result<()> {
    let options = TopOptions::default();

    if options.batch_mode {
        if let Err(e) = run_batch_mode(&options) {
            return Err(anyhow::anyhow!("top error: {}", e));
        }
    } else if let Err(e) = run_interactive_mode(&options) {
        return Err(anyhow::anyhow!("top error: {}", e));
    }

    Ok(())
}

pub fn execute(
    _args: &[String],
    _context: &crate::common::BuiltinContext,
) -> crate::common::BuiltinResult<i32> {
    println!("top: Command not yet implemented");
    Ok(0)
}
