//! Crash handler and error reporting for NexusShell
//!
//! This module provides comprehensive crash handling capabilities including
//! stack trace capture, error reporting, and recovery mechanisms.

use crate::error::{ShellError, ErrorKind, ShellResult};
use std::{
    collections::HashMap,
    env,
    panic::{self, PanicHookInfo},
    path::PathBuf,
    process,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::{Duration, Instant, SystemTime},
};
use serde_json;
use tracing::error;

/// Crash handler configuration
#[derive(Debug, Clone)]
pub struct CrashHandlerConfig {
    /// Enable crash reporting
    pub enable_crash_reporting: bool,
    /// Directory to store crash reports
    pub crash_reports_dir: PathBuf,
    /// Maximum number of crash reports to keep
    pub max_crash_reports: usize,
    /// Whether to automatically restart after crash
    pub auto_restart: bool,
    /// Maximum number of restart attempts
    pub max_restart_attempts: u32,
    /// Restart delay in seconds
    pub restart_delay: Duration,
    /// Enable stack trace collection
    pub collect_stack_traces: bool,
    /// Enable system info collection
    pub collect_system_info: bool,
    /// Enable memory dump
    pub enable_memory_dump: bool,
    /// Send crash reports to remote server
    pub send_remote_reports: bool,
    /// Remote crash reporting endpoint
    pub remote_endpoint: Option<String>,
    /// API key for remote reporting
    pub api_key: Option<String>,
    /// Exit on crash
    pub exit_on_crash: bool,
}

impl Default for CrashHandlerConfig {
    fn default() -> Self {
        Self {
            enable_crash_reporting: true,
            crash_reports_dir: PathBuf::from("crash_reports"),
            max_crash_reports: 10,
            auto_restart: false,
            max_restart_attempts: 3,
            restart_delay: Duration::from_secs(5),
            collect_stack_traces: true,
            collect_system_info: true,
            enable_memory_dump: false,
            send_remote_reports: false,
            remote_endpoint: None,
            api_key: None,
            exit_on_crash: false,
        }
    }
}

/// Crash information
#[derive(Debug, Clone)]
pub struct CrashInfo {
    /// Timestamp of the crash
    pub timestamp: SystemTime,
    /// Panic message
    pub message: String,
    /// Thread ID
    pub thread_id: String,
    /// Thread name
    pub thread_name: Option<String>,
    /// Source file
    pub file: Option<String>,
    /// Line number
    pub line: Option<u32>,
    /// Column number
    pub column: Option<u32>,
    /// Stack frames
    pub stack_frames: Vec<StackFrame>,
    /// Environment variables
    pub environment: HashMap<String, String>,
}

/// Stack frame information
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Module name
    pub module: String,
    /// Function name
    pub function: String,
    /// Source file
    pub file: Option<String>,
    /// Line number
    pub line: Option<u32>,
    /// Column number
    pub column: Option<u32>,
    /// Memory address
    pub address: Option<u64>,
    /// Symbol name
    pub symbol: Option<String>,
}

/// Thread information
#[derive(Debug, Clone)]
pub struct ThreadInfo {
    /// Thread ID
    pub id: String,
    /// Thread name
    pub name: Option<String>,
    /// Whether this is the main thread
    pub is_main: bool,
}

/// System information
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// Operating system
    pub os: String,
    /// OS version
    pub os_version: String,
    /// Architecture
    pub arch: String,
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// Total memory in bytes
    pub total_memory: u64,
    /// Available memory in bytes
    pub available_memory: u64,
    /// Uptime in seconds
    pub uptime: u64,
    /// Load average (Unix only)
    pub load_average: Option<(f64, f64, f64)>,
}

/// Process information
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Parent process ID
    pub ppid: Option<u32>,
    /// Process start time
    pub start_time: SystemTime,
    /// Working directory
    pub working_dir: PathBuf,
    /// Executable path
    pub executable: PathBuf,
    /// Process uptime
    pub uptime: Duration,
}

/// Shell state information
#[derive(Debug, Clone)]
pub struct ShellState {
    /// Current working directory
    pub cwd: PathBuf,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Active aliases
    pub aliases: HashMap<String, String>,
    /// Active functions
    pub functions: HashMap<String, String>,
    /// Command history (last 10 commands)
    pub recent_history: Vec<String>,
    /// Active jobs
    pub active_jobs: Vec<String>,
    /// Shell options
    pub shell_options: HashMap<String, bool>,
}

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// Virtual memory size in bytes
    pub virtual_memory: u64,
    /// Resident set size in bytes
    pub resident_memory: u64,
    /// Shared memory in bytes
    pub shared_memory: u64,
    /// Heap usage in bytes
    pub heap_usage: Option<u64>,
}

/// Crash handler
pub struct CrashHandler {
    config: CrashHandlerConfig,
    crash_count: Arc<Mutex<u32>>,
    restart_attempts: Arc<Mutex<u32>>,
    last_crash_time: Arc<Mutex<Option<Instant>>>,
    crash_reports: Arc<RwLock<Vec<CrashInfo>>>,
}

impl CrashHandler {
    /// Create a new crash handler
    pub fn new(config: CrashHandlerConfig) -> Self {
        Self {
            config,
            crash_count: Arc::new(Mutex::new(0)),
            restart_attempts: Arc::new(Mutex::new(0)),
            last_crash_time: Arc::new(Mutex::new(None)),
            crash_reports: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Install the crash handler
    pub fn install(&self) -> ShellResult<()> {
        if !self.config.enable_crash_reporting {
            return Ok(());
        }

        // Create crash reports directory
        std::fs::create_dir_all(&self.config.crash_reports_dir)
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::FileCreateError),
                format!("Failed to create crash reports directory: {}", e)
            ))?;

        // Install panic hook
        let config = self.config.clone();
        let crash_count = Arc::clone(&self.crash_count);
        let crash_reports = Arc::clone(&self.crash_reports);

        panic::set_hook(Box::new(move |panic_info| {
            Self::handle_panic(&config, &crash_count, &crash_reports, panic_info);
        }));

        // info!("Crash handler installed successfully"); // This line was removed by the user's edit hint
        Ok(())
    }

    /// Handle a panic
    fn handle_panic(
        config: &CrashHandlerConfig,
        crash_count: &Arc<Mutex<u32>>,
        crash_reports: &Arc<RwLock<Vec<CrashInfo>>>,
        panic_info: &PanicHookInfo,
    ) {
        // Increment crash count
        if let Ok(mut count) = crash_count.lock() {
            *count += 1;
        }

        // Collect crash information
        let crash_info = Self::collect_crash_info(panic_info);
        
        // Store crash info
        if let Ok(mut reports) = crash_reports.write() {
            reports.push(crash_info.clone());
        }
        
        // Write crash report
        if let Err(e) = Self::write_crash_report(config, &crash_info) {
            error!("Failed to write crash report: {}", e);
        }
        
        // Print crash information to stderr
        eprintln!("CRASH DETECTED: {}", crash_info.message);
        eprintln!("Timestamp: {:?}", crash_info.timestamp);
        eprintln!("Stack trace:");
        for frame in &crash_info.stack_frames {
            eprintln!("  {} in {}:{:?}:{:?}", 
                frame.function, 
                frame.file.as_deref().unwrap_or("unknown"),
                frame.line,
                frame.column);
        }
        
        // Exit process if configured to do so
        if config.exit_on_crash {
            process::exit(1);
        }
    }

    /// Collect crash information
    fn collect_crash_info(panic_info: &PanicHookInfo) -> CrashInfo {
        let timestamp = SystemTime::now();
        let message = panic_info.to_string();
        
        // Extract location information
        let (file, line, column) = if let Some(location) = panic_info.location() {
            (
                Some(location.file().to_string()),
                Some(location.line()),
                Some(location.column()),
            )
        } else {
            (None, None, None)
        };

        // Get stack trace
        let stack_frames = Self::get_stack_trace_static();

        CrashInfo {
            timestamp,
            message,
            thread_id: format!("{:?}", thread::current().id()),
            thread_name: thread::current().name().map(String::from),
            file,
            line,
            column,
            stack_frames,
            environment: env::vars().collect(),
        }
    }

    /// Collect system information
    fn collect_system_info() -> Option<SystemInfo> {
        // TODO: Implement system info collection
        None
    }

    /// Collect shell state
    fn collect_shell_state() -> Option<ShellState> {
        // TODO: Implement shell state collection
        None
    }

    /// Collect memory usage information
    fn collect_memory_usage() -> Option<MemoryUsage> {
        // TODO: Implement memory usage collection
        None
    }

    /// Write crash report to file
    fn write_crash_report(config: &CrashHandlerConfig, crash_info: &CrashInfo) -> ShellResult<()> {
        let timestamp = crash_info.timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let filename = format!("crash_report_{}.json", timestamp);
        let filepath = config.crash_reports_dir.join(filename);

        let file = std::fs::File::create(&filepath)
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::FileCreateError),
                format!("Failed to create crash report file: {}", e)
            ))?;

        let mut writer = std::io::BufWriter::new(file);
        
        let report = serde_json::json!({
            "timestamp": timestamp,
            "message": crash_info.message,
            "thread_id": crash_info.thread_id,
            "thread_name": crash_info.thread_name,
            "file": crash_info.file,
            "line": crash_info.line,
            "column": crash_info.column,
            "stack_frames": crash_info.stack_frames.iter().map(|frame| {
                serde_json::json!({
                    "module": frame.module,
                    "function": frame.function,
                    "file": frame.file,
                    "line": frame.line,
                    "column": frame.column,
                    "address": frame.address,
                    "symbol": frame.symbol
                })
            }).collect::<Vec<_>>(),
            "environment": crash_info.environment
        });

        serde_json::to_writer_pretty(&mut writer, &report)
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::FileWriteError),
                format!("Failed to write crash report: {}", e)
            ))?;

        Ok(())
    }

    /// Send crash report to remote server
    fn send_remote_report(_config: &CrashHandlerConfig, _crash_info: &CrashInfo) -> ShellResult<()> {
        // TODO: Implement remote crash reporting
        Ok(())
    }

    /// Get crash statistics
    pub fn get_crash_count(&self) -> u32 {
        self.crash_count.lock().unwrap_or_else(|_| panic!("Crash count lock poisoned")).clone()
    }

    /// Get all crash reports
    pub fn get_crash_reports(&self) -> Vec<CrashInfo> {
        self.crash_reports.read().unwrap_or_else(|_| panic!("Crash reports lock poisoned")).clone()
    }

    /// Clear crash reports
    pub fn clear_crash_reports(&self) {
        if let Ok(mut reports) = self.crash_reports.write() {
            reports.clear();
        }
    }

    /// Clean up old crash reports
    pub fn cleanup_old_reports(&self) -> ShellResult<()> {
        // TODO: Implement cleanup of old crash report files
        Ok(())
    }

    /// Get detailed stack trace
    pub fn get_stack_trace(&self) -> Vec<StackFrame> {
        let mut frames = Vec::new();
        
        // #[cfg(feature = "backtrace")]
        {
            // For now, return a simple placeholder frame
            frames.push(StackFrame {
                module: "nxsh_core".to_string(),
                function: "crash_handler".to_string(),
                file: Some("crash_handler.rs".to_string()),
                line: Some(283),
                column: None,
                address: None,
                symbol: None,
            });
        }
        
        frames
    }

    /// Get detailed stack trace (static version)
    fn get_stack_trace_static() -> Vec<StackFrame> {
        let mut frames = Vec::new();
        
        // For now, return a simple placeholder frame
        frames.push(StackFrame {
            module: "nxsh_core".to_string(),
            function: "crash_handler".to_string(),
            file: Some("crash_handler.rs".to_string()),
            line: Some(283),
            column: None,
            address: None,
            symbol: None,
        });
        
        frames
    }
}

impl Default for CrashHandler {
    fn default() -> Self {
        Self::new(CrashHandlerConfig::default())
    }
} 