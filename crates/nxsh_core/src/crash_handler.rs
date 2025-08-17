//! Comprehensive Crash Handler System for NexusShell
//!
//! This module provides enterprise-grade crash handling, reporting, and recovery
//! capabilities with privacy-aware data collection and automated diagnostics.

use crate::compat::{Result, Context};
use crate::nxsh_log_warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Write, BufWriter, BufRead, BufReader};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

use std::panic::{self, PanicHookInfo};
use std::backtrace::Backtrace;

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
            { dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".")) }
            #[cfg(not(feature = "system-info"))]
            { PathBuf::from(".") }
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
        fs::create_dir_all(&config.crash_report_dir)
            .with_context(|| format!("Failed to create crash report directory: {:?}", config.crash_report_dir))?;

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
                *stats_guard.crashes_by_severity.entry(crash_event.severity).or_insert(0) += 1;
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

            eprintln!("\nCrash report saved to: {:?}", config_guard.crash_report_dir);
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
            { whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string()) }
            #[cfg(not(feature = "system-info"))]
            { "unknown".to_string() }
        };

        let username = if config.privacy_mode {
            "user".to_string()
        } else {
            #[cfg(feature = "system-info")]
            { whoami::username() }
            #[cfg(not(feature = "system-info"))]
            { "unknown".to_string() }
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
                { Self::get_total_memory().unwrap_or(0) }
                #[cfg(not(feature = "system-info"))]
                { 0 }
            },
            memory_available: {
                #[cfg(feature = "system-info")]
                { Self::get_available_memory().unwrap_or(0) }
                #[cfg(not(feature = "system-info"))]
                { 0 }
            },
            cpu_count: {
                #[cfg(feature = "system-info")]
                { num_cpus::get() }
                #[cfg(not(feature = "system-info"))]
                { 0 }
            },
            load_average: {
                #[cfg(feature = "system-info")]
                { Self::get_load_average().unwrap_or(0.0) }
                #[cfg(not(feature = "system-info"))]
                { 0.0 }
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
            open_files: if config.privacy_mode { Vec::new() } else { Self::get_open_files().unwrap_or_default() },
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
        let environment_size = ctx.env.read().map(|m| m.len()).unwrap_or_else(|_| std::env::vars().count());
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
        
        if message_lower.contains("out of memory") || 
           message_lower.contains("segmentation fault") ||
           message_lower.contains("stack overflow") {
            CrashSeverity::Fatal
        } else if message_lower.contains("assertion") ||
                  message_lower.contains("index out of bounds") {
            CrashSeverity::Critical
        } else if message_lower.contains("io error") ||
                  message_lower.contains("permission denied") {
            CrashSeverity::Major
        } else if message_lower.contains("parsing") ||
                  message_lower.contains("format") {
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
                    *stats.crashes_by_severity.entry(crash_event.severity).or_insert(0) += 1;
                    
                    if let Some(crash_time) = SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(crash_event.timestamp)) {
                        stats.most_recent_crash = Some(crash_time);
                    }
                }
                Err(e) => {
                    nxsh_log_warn!("Failed to parse crash report line: {}", e);
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

    fn get_total_memory() -> Option<u64> {
        // Simplified implementation - would need platform-specific code
        None
    }

    fn get_available_memory() -> Option<u64> {
        // Simplified implementation - would need platform-specific code
        None
    }

    fn get_load_average() -> Option<f64> {
        // Simplified implementation - would need platform-specific code
        None
    }

    fn get_parent_pid() -> Option<u32> {
        // Simplified implementation - would need platform-specific code
        None
    }

    fn get_memory_usage() -> Option<MemoryUsage> {
        // Simplified implementation - would need platform-specific code
        None
    }

    fn get_cpu_usage() -> Option<f64> {
        // Simplified implementation - would need platform-specific code
        None
    }

    fn get_open_files() -> Option<Vec<String>> {
        // Simplified implementation - would need platform-specific code
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
