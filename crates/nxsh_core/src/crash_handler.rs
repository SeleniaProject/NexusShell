//! Comprehensive Crash Handler System for NexusShell
//!
//! This module provides enterprise-grade crash handling, reporting, and recovery
//! capabilities with privacy-aware data collection and automated diagnostics.

use crate::compat::{Context, Result};
use crate::nxsh_log_warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use std::backtrace::Backtrace;
use std::panic::{self, PanicHookInfo};

/// Crash severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CrashSeverity {
    /// Minor issues that don't affect core functionality
    Minor,
    /// Moderate issues that may affect some features
    Moderate,
    /// Major crashes that affect core functionality
    Major,
    /// Critical system failures
    Critical,
    /// Fatal errors that require immediate shutdown
    Fatal,
}

/// System information collected during crash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub kernel_version: String,
    pub hostname: String,
    pub username: String,
    pub shell_version: String,
    pub uptime_seconds: u64,
    pub memory_total: u64,
    pub memory_available: u64,
    pub cpu_count: usize,
    pub load_average: f64,
}

/// Process information at crash time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: u32,
    pub memory_usage: MemoryUsage,
    pub cpu_usage: f64,
    pub open_files: Vec<String>,
    pub environment_vars: HashMap<String, String>,
    pub command_line: Vec<String>,
    pub working_directory: String,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryUsage {
    pub resident: u64,
    pub virt_mem: u64,
    pub shared: u64,
    pub heap: u64,
    pub stack: u64,
}

/// Shell state at crash time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellState {
    pub current_directory: String,
    pub history_entries: usize,
    pub active_jobs: usize,
    pub loaded_aliases: usize,
    pub environment_size: usize,
    pub last_command: Option<String>,
    pub exit_code: Option<i32>,
}

/// Crash event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashEvent {
    pub id: String,
    pub timestamp: u64,
    pub severity: CrashSeverity,
    pub message: String,
    pub backtrace: Option<String>,
    pub system_info: SystemInfo,
    pub process_info: ProcessInfo,
    pub shell_state: ShellState,
    pub additional_data: HashMap<String, String>,
    pub recovery_attempted: bool,
    pub recovery_successful: bool,
}

/// Crash handler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashHandlerConfig {
    pub enabled: bool,
    pub collect_system_info: bool,
    pub collect_backtrace: bool,
    pub collect_memory_dump: bool,
    pub collect_environment: bool,
    pub max_crash_reports: usize,
    pub crash_report_dir: PathBuf,
    pub auto_restart: bool,
    pub restart_delay: Duration,
    pub send_reports: bool,
    pub report_endpoint: Option<String>,
    pub privacy_mode: bool,
}

/// Statistics about crashes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashStats {
    pub total_crashes: u64,
    pub crashes_by_severity: HashMap<CrashSeverity, u64>,
    pub crashes_last_24h: u64,
    pub most_recent_crash: Option<SystemTime>,
    pub recovery_success_rate: f64,
    pub average_time_to_recovery: Duration,
}

/// Main crash handler
pub struct CrashHandler {
    config: RwLock<CrashHandlerConfig>,
    crash_reports: Arc<Mutex<Vec<CrashEvent>>>,
    stats: Arc<Mutex<CrashStats>>,
    report_file: Arc<Mutex<Option<BufWriter<File>>>>,
}

impl Default for CrashHandlerConfig {
    fn default() -> Self {
        let crash_dir = {
            #[cfg(feature = "system-info")]
            {
                dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."))
            }
            #[cfg(not(feature = "system-info"))]
            {
                PathBuf::from(".")
            }
        }
        .join("nxsh")
        .join("crashes");

        Self {
            enabled: true,
            collect_system_info: true,
            collect_backtrace: true,
            collect_memory_dump: false, // Privacy-conscious default
            collect_environment: false, // Privacy-conscious default
            max_crash_reports: 100,
            crash_report_dir: crash_dir,
            auto_restart: true,
            restart_delay: Duration::from_secs(1),
            send_reports: false, // Privacy-conscious default
            report_endpoint: None,
            privacy_mode: true,
        }
    }
}

impl CrashHandler {
    /// Create a new crash handler
    pub fn new(config: CrashHandlerConfig) -> Result<Self> {
        // Create crash report directory
        fs::create_dir_all(&config.crash_report_dir).with_context(|| {
            format!(
                "Failed to create crash report directory: {:?}",
                config.crash_report_dir
            )
        })?;

        let crash_handler = Self {
            config: RwLock::new(config),
            crash_reports: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(CrashStats::default())),
            report_file: Arc::new(Mutex::new(None)),
        };

        // Initialize crash report file
        crash_handler.init_report_file()?;

        Ok(crash_handler)
    }

    /// Initialize the crash report file
    fn init_report_file(&self) -> Result<()> {
        let config = self.config.read().unwrap();
        let report_path = config.crash_report_dir.join("crashes.jsonl");

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&report_path)
            .with_context(|| format!("Failed to open crash report file: {report_path:?}"))?;

        let mut report_file = self.report_file.lock().unwrap();
        *report_file = Some(BufWriter::new(file));

        Ok(())
    }

    /// Install panic handler
    pub fn install_panic_handler(&self) {
        let crash_reports = Arc::clone(&self.crash_reports);
        let stats = Arc::clone(&self.stats);
        let config = Arc::new(RwLock::new(self.config.read().unwrap().clone()));
        let report_file = Arc::clone(&self.report_file);

        panic::set_hook(Box::new(move |panic_info| {
            let config_guard = config.read().unwrap();
            if !config_guard.enabled {
                return;
            }

            let crash_event = match Self::create_crash_event_from_panic(panic_info, &config_guard) {
                Ok(event) => event,
                Err(e) => {
                    eprintln!("Failed to create crash event: {e}");
                    return;
                }
            };

            // Update statistics
            {
                let mut stats_guard = stats.lock().unwrap();
                stats_guard.total_crashes += 1;
                *stats_guard
                    .crashes_by_severity
                    .entry(crash_event.severity)
                    .or_insert(0) += 1;
                stats_guard.most_recent_crash = Some(SystemTime::now());
            }

            // Store crash report
            {
                let mut reports = crash_reports.lock().unwrap();
                reports.push(crash_event.clone());

                // Limit report history
                if reports.len() > config_guard.max_crash_reports {
                    let excess = reports.len() - config_guard.max_crash_reports;
                    reports.drain(0..excess);
                }
            }

            // Write to file
            if let Ok(mut file) = report_file.lock() {
                if let Some(ref mut writer) = *file {
                    if let Ok(json) = serde_json::to_string(&crash_event) {
                        let _ = writeln!(writer, "{json}");
                        let _ = writer.flush();
                    }
                }
            }

            // Print crash information
            eprintln!("\n🚨 NexusShell Crash Detected!");
            eprintln!("Crash ID: {}", crash_event.id);
            eprintln!("Severity: {:?}", crash_event.severity);
            eprintln!("Message: {}", crash_event.message);

            if config_guard.collect_backtrace {
                if let Some(ref backtrace) = crash_event.backtrace {
                    eprintln!("Backtrace:\n{backtrace}");
                }
            }

            eprintln!(
                "\nCrash report saved to: {:?}",
                config_guard.crash_report_dir
            );
        }));
    }

    /// Create crash event from panic info
    fn create_crash_event_from_panic(
        panic_info: &PanicHookInfo,
        config: &CrashHandlerConfig,
    ) -> Result<CrashEvent> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = format!("crash-{}-{}", timestamp, rand::random::<u16>());

        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let backtrace = if config.collect_backtrace {
            Some(format!("{:?}", Backtrace::force_capture()))
        } else {
            None
        };

        let system_info = Self::collect_system_info(config)?;
        let process_info = Self::collect_process_info(config)?;
        let shell_state = Self::collect_shell_state(config)?;

        let severity = Self::classify_crash_severity(&message);

        Ok(CrashEvent {
            id,
            timestamp,
            severity,
            message,
            backtrace,
            system_info,
            process_info,
            shell_state,
            additional_data: HashMap::new(),
            recovery_attempted: false,
            recovery_successful: false,
        })
    }

    /// Collect system information
    fn collect_system_info(config: &CrashHandlerConfig) -> Result<SystemInfo> {
        if !config.collect_system_info {
            return Ok(SystemInfo {
                os: "redacted".to_string(),
                arch: "redacted".to_string(),
                kernel_version: "redacted".to_string(),
                hostname: "redacted".to_string(),
                username: "redacted".to_string(),
                shell_version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_seconds: 0,
                memory_total: 0,
                memory_available: 0,
                cpu_count: 0,
                load_average: 0.0,
            });
        }

        let hostname = if config.privacy_mode {
            "localhost".to_string()
        } else {
            #[cfg(feature = "system-info")]
            {
                whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string())
            }
            #[cfg(not(feature = "system-info"))]
            {
                "unknown".to_string()
            }
        };

        let username = if config.privacy_mode {
            "user".to_string()
        } else {
            #[cfg(feature = "system-info")]
            {
                whoami::username()
            }
            #[cfg(not(feature = "system-info"))]
            {
                "unknown".to_string()
            }
        };

        Ok(SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            kernel_version: Self::get_kernel_version().unwrap_or_else(|| "unknown".to_string()),
            hostname,
            username,
            shell_version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: Self::get_uptime().unwrap_or(0),
            memory_total: {
                #[cfg(feature = "system-info")]
                {
                    Self::get_total_memory().unwrap_or(0)
                }
                #[cfg(not(feature = "system-info"))]
                {
                    0
                }
            },
            memory_available: {
                #[cfg(feature = "system-info")]
                {
                    Self::get_available_memory().unwrap_or(0)
                }
                #[cfg(not(feature = "system-info"))]
                {
                    0
                }
            },
            cpu_count: {
                #[cfg(feature = "system-info")]
                {
                    num_cpus::get()
                }
                #[cfg(not(feature = "system-info"))]
                {
                    0
                }
            },
            load_average: {
                #[cfg(feature = "system-info")]
                {
                    Self::get_load_average().unwrap_or(0.0)
                }
                #[cfg(not(feature = "system-info"))]
                {
                    0.0
                }
            },
        })
    }

    /// Collect process information
    fn collect_process_info(config: &CrashHandlerConfig) -> Result<ProcessInfo> {
        let pid = std::process::id();

        Ok(ProcessInfo {
            pid,
            parent_pid: Self::get_parent_pid().unwrap_or(0),
            memory_usage: Self::get_memory_usage().unwrap_or_default(),
            cpu_usage: Self::get_cpu_usage().unwrap_or(0.0),
            open_files: if config.privacy_mode {
                Vec::new()
            } else {
                Self::get_open_files().unwrap_or_default()
            },
            environment_vars: if config.collect_environment && !config.privacy_mode {
                std::env::vars().collect()
            } else {
                HashMap::new()
            },
            command_line: std::env::args().collect(),
            working_directory: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "/".to_string()),
        })
    }

    /// Collect shell state
    fn collect_shell_state(_config: &CrashHandlerConfig) -> Result<ShellState> {
        // Best-effort extraction from a fresh ShellContext; in production this would
        // accept a reference to the live context. This implementation aggregates
        // environment size, history length, active jobs, aliases, and last command.
        let ctx = crate::context::ShellContext::new();
        let current_directory = ctx.cwd.to_string_lossy().to_string();
        let history_entries = ctx.get_history().len();
        let active_jobs = ctx.jobs.read().map(|m| m.len()).unwrap_or(0);
        let loaded_aliases = ctx.aliases.read().map(|m| m.len()).unwrap_or(0);
        let environment_size = ctx
            .env
            .read()
            .map(|m| m.len())
            .unwrap_or_else(|_| std::env::vars().count());
        let last_command = ctx.get_history().last().cloned();

        Ok(ShellState {
            current_directory,
            history_entries,
            active_jobs,
            loaded_aliases,
            environment_size,
            last_command,
            exit_code: None,
        })
    }

    /// Classify crash severity based on error message
    fn classify_crash_severity(message: &str) -> CrashSeverity {
        let message_lower = message.to_lowercase();

        if message_lower.contains("out of memory")
            || message_lower.contains("segmentation fault")
            || message_lower.contains("stack overflow")
        {
            CrashSeverity::Fatal
        } else if message_lower.contains("assertion")
            || message_lower.contains("index out of bounds")
        {
            CrashSeverity::Critical
        } else if message_lower.contains("io error") || message_lower.contains("permission denied")
        {
            CrashSeverity::Major
        } else if message_lower.contains("parsing") || message_lower.contains("format") {
            CrashSeverity::Moderate
        } else {
            CrashSeverity::Minor
        }
    }

    /// Get recent crash reports
    pub fn get_recent_crashes(&self, limit: usize) -> Vec<CrashEvent> {
        let reports = self.crash_reports.lock().unwrap();
        let start_idx = reports.len().saturating_sub(limit);
        reports[start_idx..].to_vec()
    }

    /// Get crash statistics
    pub fn get_stats(&self) -> CrashStats {
        self.stats.lock().unwrap().clone()
    }

    /// Load crash reports from file
    pub fn load_crash_reports(&self) -> Result<()> {
        let config = self.config.read().unwrap();
        let report_path = config.crash_report_dir.join("crashes.jsonl");

        if !report_path.exists() {
            return Ok(());
        }

        let file = File::open(&report_path)
            .with_context(|| format!("Failed to open crash report file: {report_path:?}"))?;

        let reader = BufReader::new(file);
        let mut reports = self.crash_reports.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        for line in reader.lines() {
            let line = line.with_context(|| "Failed to read line from crash report file")?;

            match serde_json::from_str::<CrashEvent>(&line) {
                Ok(crash_event) => {
                    reports.push(crash_event.clone());
                    stats.total_crashes += 1;
                    *stats
                        .crashes_by_severity
                        .entry(crash_event.severity)
                        .or_insert(0) += 1;

                    if let Some(crash_time) = SystemTime::UNIX_EPOCH
                        .checked_add(Duration::from_secs(crash_event.timestamp))
                    {
                        stats.most_recent_crash = Some(crash_time);
                    }
                }
                Err(_e) => {
                    nxsh_log_warn!("Failed to parse crash report line: {}", _e);
                }
            }
        }

        // Limit report history
        if reports.len() > config.max_crash_reports {
            let excess = reports.len() - config.max_crash_reports;
            reports.drain(0..excess);
        }

        Ok(())
    }

    // Platform-specific system information helpers
    #[cfg(target_os = "linux")]
    fn get_kernel_version() -> Option<String> {
        std::fs::read_to_string("/proc/version")
            .ok()
            .map(|s| s.trim().to_string())
    }

    #[cfg(not(target_os = "linux"))]
    fn get_kernel_version() -> Option<String> {
        None
    }

    #[cfg(target_os = "linux")]
    fn get_uptime() -> Option<u64> {
        std::fs::read_to_string("/proc/uptime")
            .ok()
            .and_then(|s| s.split_whitespace().next().map(|s| s.to_string()))
            .and_then(|s| s.parse::<f64>().ok())
            .map(|f| f as u64)
    }

    #[cfg(not(target_os = "linux"))]
    fn get_uptime() -> Option<u64> {
        None
    }

    #[allow(dead_code)]
    fn get_total_memory() -> Option<u64> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                for line in meminfo.lines() {
                    if let Some(value) = line.strip_prefix("MemTotal:") {
                        if let Ok(kb) = value
                            .trim()
                            .split_whitespace()
                            .next()
                            .unwrap_or("0")
                            .parse::<u64>()
                        {
                            return Some(kb * 1024); // Convert KB to bytes
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            use std::mem;
            use winapi::um::sysinfoapi::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

            unsafe {
                let mut mem_status: MEMORYSTATUSEX = mem::zeroed();
                mem_status.dwLength = mem::size_of::<MEMORYSTATUSEX>() as u32;

                if GlobalMemoryStatusEx(&mut mem_status) != 0 {
                    return Some(mem_status.ullTotalPhys);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("sysctl").args(["-n", "hw.memsize"]).output() {
                if let Ok(mem_str) = String::from_utf8(output.stdout) {
                    if let Ok(mem_bytes) = mem_str.trim().parse::<u64>() {
                        return Some(mem_bytes);
                    }
                }
            }
        }

        None
    }

    #[allow(dead_code)]
    fn get_available_memory() -> Option<u64> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                for line in meminfo.lines() {
                    if let Some(value) = line.strip_prefix("MemAvailable:") {
                        if let Ok(kb) = value
                            .trim()
                            .split_whitespace()
                            .next()
                            .unwrap_or("0")
                            .parse::<u64>()
                        {
                            return Some(kb * 1024); // Convert KB to bytes
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            use std::mem;
            use winapi::um::sysinfoapi::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

            unsafe {
                let mut mem_status: MEMORYSTATUSEX = mem::zeroed();
                mem_status.dwLength = mem::size_of::<MEMORYSTATUSEX>() as u32;

                if GlobalMemoryStatusEx(&mut mem_status) != 0 {
                    return Some(mem_status.ullAvailPhys);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("vm_stat").output() {
                if let Ok(vm_stat_str) = String::from_utf8(output.stdout) {
                    let mut free_pages = 0u64;
                    for line in vm_stat_str.lines() {
                        if line.starts_with("Pages free:") {
                            if let Some(pages_str) = line.split(':').nth(1) {
                                if let Ok(pages) = pages_str.trim().replace('.', "").parse::<u64>()
                                {
                                    free_pages = pages;
                                    break;
                                }
                            }
                        }
                    }
                    // Assume 4KB page size
                    return Some(free_pages * 4096);
                }
            }
        }

        None
    }

    #[allow(dead_code)]
    fn get_load_average() -> Option<f64> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {
                if let Some(first_load) = loadavg.split_whitespace().next() {
                    if let Ok(load) = first_load.parse::<f64>() {
                        return Some(load);
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("uptime").output() {
                if let Ok(uptime_str) = String::from_utf8(output.stdout) {
                    // Parse load average from uptime output
                    if let Some(load_part) = uptime_str.split("load averages:").nth(1) {
                        if let Some(first_load) = load_part.trim().split_whitespace().next() {
                            if let Ok(load) = first_load.parse::<f64>() {
                                return Some(load);
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows doesn't have direct load average, use CPU usage as approximation
            use std::process::Command;
            if let Ok(output) = Command::new("wmic")
                .args(["cpu", "get", "loadpercentage", "/value"])
                .output()
            {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.starts_with("LoadPercentage=") {
                            if let Some(value_str) = line.split('=').nth(1) {
                                if let Ok(load_percent) = value_str.trim().parse::<f64>() {
                                    return Some(load_percent / 100.0);
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn get_parent_pid() -> Option<u32> {
        #[cfg(unix)]
        {
            use std::os::unix::process::parent_id;
            use std::process;
            Some(parent_id())
        }

        #[cfg(target_os = "windows")]
        {
            use std::mem;
            use std::process;
            use winapi::um::handleapi::CloseHandle;
            use winapi::um::tlhelp32::{
                CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32,
                TH32CS_SNAPPROCESS,
            };

            unsafe {
                let current_pid = process::id();
                let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
                if snapshot == winapi::um::handleapi::INVALID_HANDLE_VALUE {
                    return None;
                }

                let mut entry: PROCESSENTRY32 = mem::zeroed();
                entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

                if Process32First(snapshot, &mut entry) != 0 {
                    loop {
                        if entry.th32ProcessID == current_pid {
                            CloseHandle(snapshot);
                            return Some(entry.th32ParentProcessID);
                        }

                        if Process32Next(snapshot, &mut entry) == 0 {
                            break;
                        }
                    }
                }

                CloseHandle(snapshot);
                None
            }
        }

        #[cfg(not(any(unix, target_os = "windows")))]
        None
    }

    fn get_memory_usage() -> Option<MemoryUsage> {
        #[cfg(target_os = "linux")]
        {
            use std::process;
            let pid = process::id();
            if let Ok(status) = std::fs::read_to_string(format!("/proc/{}/status", pid)) {
                let mut usage = MemoryUsage::default();

                for line in status.lines() {
                    if let Some(value) = line.strip_prefix("VmRSS:") {
                        if let Ok(kb) = value
                            .trim()
                            .split_whitespace()
                            .next()
                            .unwrap_or("0")
                            .parse::<u64>()
                        {
                            usage.resident = kb * 1024;
                        }
                    } else if let Some(value) = line.strip_prefix("VmSize:") {
                        if let Ok(kb) = value
                            .trim()
                            .split_whitespace()
                            .next()
                            .unwrap_or("0")
                            .parse::<u64>()
                        {
                            usage.virt_mem = kb * 1024;
                        }
                    }
                }

                return Some(usage);
            }
        }

        #[cfg(target_os = "windows")]
        {
            use std::mem;
            use winapi::um::processthreadsapi::GetCurrentProcess;
            use winapi::um::psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};

            unsafe {
                let mut pmc: PROCESS_MEMORY_COUNTERS = mem::zeroed();
                pmc.cb = mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

                if GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, pmc.cb) != 0 {
                    return Some(MemoryUsage {
                        resident: pmc.WorkingSetSize as u64,
                        virt_mem: pmc.PagefileUsage as u64,
                        shared: 0,
                        heap: 0,
                        stack: 0,
                    });
                }
            }
        }

        None
    }

    fn get_cpu_usage() -> Option<f64> {
        #[cfg(target_os = "linux")]
        {
            use std::process;
            let pid = process::id();
            if let Ok(stat) = std::fs::read_to_string(format!("/proc/{}/stat", pid)) {
                let fields: Vec<&str> = stat.split_whitespace().collect();
                if fields.len() >= 15 {
                    // utime (14th field) + stime (15th field)
                    if let (Ok(utime), Ok(stime)) =
                        (fields[13].parse::<u64>(), fields[14].parse::<u64>())
                    {
                        let total_time = utime + stime;
                        // Convert from clock ticks to seconds (assume 100 Hz)
                        return Some(total_time as f64 / 100.0);
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            use std::mem;
            use winapi::shared::minwindef::FILETIME;
            use winapi::um::processthreadsapi::{GetCurrentProcess, GetProcessTimes};

            unsafe {
                let mut creation_time: FILETIME = mem::zeroed();
                let mut exit_time: FILETIME = mem::zeroed();
                let mut kernel_time: FILETIME = mem::zeroed();
                let mut user_time: FILETIME = mem::zeroed();

                if GetProcessTimes(
                    GetCurrentProcess(),
                    &mut creation_time,
                    &mut exit_time,
                    &mut kernel_time,
                    &mut user_time,
                ) != 0
                {
                    // Convert FILETIME to u64 (100-nanosecond intervals)
                    let kernel_ticks = ((kernel_time.dwHighDateTime as u64) << 32)
                        | kernel_time.dwLowDateTime as u64;
                    let user_ticks =
                        ((user_time.dwHighDateTime as u64) << 32) | user_time.dwLowDateTime as u64;
                    let total_ticks = kernel_ticks + user_ticks;

                    // Convert to seconds
                    return Some(total_ticks as f64 / 10_000_000.0);
                }
            }
        }

        None
    }

    fn get_open_files() -> Option<Vec<String>> {
        #[cfg(target_os = "linux")]
        {
            use std::process;
            let pid = process::id();
            let fd_dir = format!("/proc/{}/fd", pid);

            if let Ok(entries) = std::fs::read_dir(fd_dir) {
                let mut files = Vec::new();
                for entry in entries.flatten() {
                    if let Ok(target) = std::fs::read_link(entry.path()) {
                        files.push(target.to_string_lossy().to_string());
                    }
                }
                return Some(files);
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows implementation would require more complex API calls
            // For now, return empty list as a reasonable fallback
            Some(Vec::new())
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        None
    }
}

impl Default for CrashStats {
    fn default() -> Self {
        Self {
            total_crashes: 0,
            crashes_by_severity: HashMap::new(),
            crashes_last_24h: 0,
            most_recent_crash: None,
            recovery_success_rate: 0.0,
            average_time_to_recovery: Duration::from_secs(0),
        }
    }
}

// Default for MemoryUsage is derived above

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_crash_handler_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = CrashHandlerConfig {
            crash_report_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let handler = CrashHandler::new(config).unwrap();
        let stats = handler.get_stats();
        assert_eq!(stats.total_crashes, 0);
    }

    #[test]
    fn test_crash_severity_classification() {
        assert_eq!(
            CrashHandler::classify_crash_severity("out of memory"),
            CrashSeverity::Fatal
        );
        assert_eq!(
            CrashHandler::classify_crash_severity("assertion failed"),
            CrashSeverity::Critical
        );
        assert_eq!(
            CrashHandler::classify_crash_severity("io error"),
            CrashSeverity::Major
        );
        assert_eq!(
            CrashHandler::classify_crash_severity("parsing error"),
            CrashSeverity::Moderate
        );
        assert_eq!(
            CrashHandler::classify_crash_severity("unknown error"),
            CrashSeverity::Minor
        );
    }

    #[test]
    fn test_config_privacy_mode() {
        let config = CrashHandlerConfig {
            privacy_mode: true,
            collect_environment: false,
            ..Default::default()
        };

        let system_info = CrashHandler::collect_system_info(&config).unwrap();
        assert_eq!(system_info.hostname, "localhost");
        assert_eq!(system_info.username, "user");

        let process_info = CrashHandler::collect_process_info(&config).unwrap();
        assert!(process_info.open_files.is_empty());
        assert!(process_info.environment_vars.is_empty());
    }
}
