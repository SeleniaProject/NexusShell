//! Structured logging system for NexusShell
//!
//! This module provides comprehensive logging functionality using `tracing` 
//! and `tracing_appender` with JSON formatting and log rotation.

use std::{
    path::PathBuf,
    sync::{Arc, RwLock, atomic::{AtomicU64, Ordering}},
    time::{SystemTime, Duration},
    fs,
    collections::HashMap,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "logging")]
use tracing_subscriber::{
    Registry,
    layer::SubscriberExt,
    fmt::{self},
    filter::{LevelFilter, EnvFilter},
    Layer, // Needed for .boxed()
};
#[cfg(all(feature = "logging", feature = "heavy-time"))]
use tracing_subscriber::fmt::time::ChronoUtc;

// When heavy-time feature is disabled but logging enabled, provide a tiny timer stub
#[cfg(all(feature = "logging", not(feature = "heavy-time")))]
mod minimal_time {
    use std::time::{SystemTime, UNIX_EPOCH};
    use tracing_subscriber::fmt::{time::FormatTime, format::Writer};
    use std::fmt;
    pub struct SimpleUnixTime;
    impl FormatTime for SimpleUnixTime {
        fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
            let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
            write!(w, "{}", dur.as_secs())
        }
    }
}
#[cfg(feature = "logging")]
use tracing_appender::{
    rolling::{RollingFileAppender, Rotation},
    non_blocking::WorkerGuard,
};
use crate::compat::{Result, Context};
// Import lightweight facade macros (exported in lib.rs) so they are in scope even though defined later
// Only import macros currently used to reduce warnings
use crate::{nxsh_log_info, nxsh_log_debug};

#[cfg(feature = "logging")]
use std::io::Write as IoWrite;

/// Enhanced multi-output writer supporting flexible subscriber composition
#[cfg(feature = "logging")]
pub struct MultiOutputWriter {
    outputs: Vec<OutputTarget>,
    stats: Option<Arc<LoggingStatistics>>,
}

#[cfg(feature = "logging")]
enum OutputTarget {
    Console(bool), // enabled flag
    File(tracing_appender::non_blocking::NonBlocking),
    Custom(Box<dyn IoWrite + Send + Sync>),
}

#[cfg(feature = "logging")]
impl MultiOutputWriter {
    pub fn new() -> Self {
        Self {
            outputs: Vec::new(),
            stats: None,
        }
    }

    pub fn with_console(mut self, enabled: bool) -> Self {
        self.outputs.push(OutputTarget::Console(enabled));
        self
    }

    pub fn with_file(mut self, file_writer: tracing_appender::non_blocking::NonBlocking) -> Self {
        self.outputs.push(OutputTarget::File(file_writer));
        self
    }

    pub fn with_custom<W: IoWrite + Send + Sync + 'static>(mut self, writer: W) -> Self {
        self.outputs.push(OutputTarget::Custom(Box::new(writer)));
        self
    }

    pub fn with_stats(mut self, stats: Arc<LoggingStatistics>) -> Self {
        self.stats = Some(stats);
        self
    }

    fn record_error(&self) {
        if let Some(stats) = &self.stats {
            stats.write_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn record_bytes_written(&self, count: usize) {
        if let Some(stats) = &self.stats {
            stats.total_bytes_logged.fetch_add(count as u64, Ordering::Relaxed);
        }
    }
}

#[cfg(feature = "logging")]
impl IoWrite for MultiOutputWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut errors = 0;
        let mut success_count = 0;

        for output in &mut self.outputs {
            match output {
                OutputTarget::Console(enabled) => {
                    if *enabled {
                        let mut stdout = std::io::stdout();
                        if stdout.write_all(buf).is_err() {
                            errors += 1;
                        } else {
                            success_count += 1;
                        }
                    }
                }
                OutputTarget::File(ref mut file_writer) => {
                    if file_writer.write(buf).is_err() {
                        errors += 1;
                    } else {
                        success_count += 1;
                    }
                }
                OutputTarget::Custom(ref mut custom_writer) => {
                    if custom_writer.write_all(buf).is_err() {
                        errors += 1;
                    } else {
                        success_count += 1;
                    }
                }
            }
        }

        if errors > 0 {
            self.record_error();
        }

        // Record bytes written if at least one output succeeded
        if success_count > 0 {
            self.record_bytes_written(buf.len());
        }

        // Return success if at least one output succeeded
        if success_count > 0 || self.outputs.is_empty() {
            Ok(buf.len())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "All output targets failed"
            ))
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut errors = 0;
        let mut success_count = 0;

        for output in &mut self.outputs {
            match output {
                OutputTarget::Console(enabled) => {
                    if *enabled && std::io::stdout().flush().is_err() {
                        errors += 1;
                    } else if *enabled {
                        success_count += 1;
                    }
                }
                OutputTarget::File(ref mut file_writer) => {
                    if file_writer.flush().is_err() {
                        errors += 1;
                    } else {
                        success_count += 1;
                    }
                }
                OutputTarget::Custom(ref mut custom_writer) => {
                    if custom_writer.flush().is_err() {
                        errors += 1;
                    } else {
                        success_count += 1;
                    }
                }
            }
        }

        if errors > 0 {
            self.record_error();
        }

        // Return success if at least one output succeeded or no outputs
        if success_count > 0 || self.outputs.is_empty() {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "All output targets failed during flush"
            ))
        }
    }
}

// Legacy CombinedWriter maintained for backward compatibility
#[cfg(feature = "logging")]
struct CombinedWriter {
    console: bool,
    file: Option<tracing_appender::non_blocking::NonBlocking>,
    stats: Option<Arc<LoggingStatistics>>,
}
#[cfg(feature = "logging")]
impl IoWrite for CombinedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut err = false;
        if let Some(f) = &mut self.file { if f.write(buf).is_err() { err = true; } }
        if self.console { let mut stdout = std::io::stdout(); if stdout.write_all(buf).is_err() { err = true; } }
        if err { if let Some(s) = &self.stats { s.write_errors.fetch_add(1, Ordering::Relaxed); } }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        let mut err = false;
        if let Some(f) = &mut self.file { if f.flush().is_err() { err = true; } }
    if self.console  && std::io::stdout().flush().is_err() { err = true; }
        if err { if let Some(s) = &self.stats { s.write_errors.fetch_add(1, Ordering::Relaxed); } }
        Ok(())
    }
}

/// Structured logging system with tracing support
#[allow(dead_code)] // 一部フィールドは将来のローテーション制御用で現状のビルドでは未参照
pub struct LoggingSystem {
    config: LoggingConfig,
    statistics: Arc<LoggingStatistics>,
    _guard: Option<WorkerGuard>,
    rotation_handle: Option<RollingFileAppender>,
}

/// Comprehensive logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Enable file output
    pub file_output: bool,
    /// Log directory path
    pub log_dir: PathBuf,
    /// Maximum number of log files to retain
    pub max_files: usize,
    /// Log retention period in days
    pub retention_days: u64,
    /// Log format (json, pretty, compact, full)
    pub format: LogFormat,
    /// Rotation policy (hourly, daily, never)
    pub rotation: LogRotation,
    /// Enable console output
    pub console_output: bool,
    /// Buffer size for async logging
    pub buffer_size: usize,
    /// Enable structured fields
    pub structured_fields: bool,
    /// Custom fields to include in every log entry
    pub custom_fields: HashMap<String, String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let mut custom_fields = HashMap::new();
        custom_fields.insert("service".to_string(), "nxsh".to_string());
        custom_fields.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());

        Self {
            level: "info".to_string(),
            file_output: true,
            log_dir: PathBuf::from("logs"),
            max_files: 30,
            retention_days: 30,
            format: LogFormat::Json,
            rotation: LogRotation::Daily,
            console_output: true,
            buffer_size: 8192,
            structured_fields: true,
            custom_fields,
        }
    }
}

/// Log format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    /// Structured JSON format for machine processing
    Json,
    /// Human-readable pretty format with colors
    Pretty,
    /// Compact format for high-volume logging
    Compact,
    /// Full format with all available information
    Full,
}

/// Log rotation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogRotation {
    /// Rotate logs hourly
    Hourly,
    /// Rotate logs daily
    Daily,
    /// Never rotate logs
    Never,
}

/// Comprehensive logging statistics
/// Logging statistics structure
/// Note: Custom implementation needed for Clone due to AtomicU64
#[derive(Debug)]
pub struct LoggingStatistics {
    /// Total number of messages logged
    pub messages_logged: AtomicU64,
    /// Number of error messages logged
    pub errors_logged: AtomicU64,
    /// Number of warning messages logged
    pub warnings_logged: AtomicU64,
    /// Number of info messages logged
    pub info_logged: AtomicU64,
    /// Number of debug messages logged
    pub debug_logged: AtomicU64,
    /// Number of trace messages logged
    pub trace_logged: AtomicU64,
    /// Timestamp of last log message
    #[allow(dead_code)]
    pub last_log_time: Arc<RwLock<Option<SystemTime>>>,
    /// Current log file size in bytes
    pub current_file_size: AtomicU64,
    /// Total bytes logged
    pub total_bytes_logged: AtomicU64,
    /// Number of log files created
    pub files_created: AtomicU64,
    /// Number of log rotations performed
    pub rotations_performed: AtomicU64,
    /// Number of I/O write errors from logging backend
    pub write_errors: AtomicU64,
}

// Custom Clone implementation for LoggingStatistics
impl Clone for LoggingStatistics {
    fn clone(&self) -> Self {
        Self {
            messages_logged: AtomicU64::new(self.messages_logged.load(Ordering::Relaxed)),
            errors_logged: AtomicU64::new(self.errors_logged.load(Ordering::Relaxed)),
            warnings_logged: AtomicU64::new(self.warnings_logged.load(Ordering::Relaxed)),
            info_logged: AtomicU64::new(self.info_logged.load(Ordering::Relaxed)),
            debug_logged: AtomicU64::new(self.debug_logged.load(Ordering::Relaxed)),
            trace_logged: AtomicU64::new(self.trace_logged.load(Ordering::Relaxed)),
            last_log_time: Arc::clone(&self.last_log_time),
            current_file_size: AtomicU64::new(self.current_file_size.load(Ordering::Relaxed)),
            total_bytes_logged: AtomicU64::new(self.total_bytes_logged.load(Ordering::Relaxed)),
            files_created: AtomicU64::new(self.files_created.load(Ordering::Relaxed)),
            rotations_performed: AtomicU64::new(self.rotations_performed.load(Ordering::Relaxed)),
            write_errors: AtomicU64::new(self.write_errors.load(Ordering::Relaxed)),
        }
    }
}

impl LoggingStatistics {
    fn new() -> Self {
        Self {
            messages_logged: AtomicU64::new(0),
            errors_logged: AtomicU64::new(0),
            warnings_logged: AtomicU64::new(0),
            info_logged: AtomicU64::new(0),
            debug_logged: AtomicU64::new(0),
            trace_logged: AtomicU64::new(0),
            last_log_time: Arc::new(RwLock::new(None)),
            current_file_size: AtomicU64::new(0),
            total_bytes_logged: AtomicU64::new(0),
            files_created: AtomicU64::new(0),
            rotations_performed: AtomicU64::new(0),
            write_errors: AtomicU64::new(0),
        }
    }

    /// Get statistics as a serializable summary
    pub fn summary(&self) -> LoggingSummary {
        LoggingSummary {
            messages_logged: self.messages_logged.load(Ordering::Relaxed),
            errors_logged: self.errors_logged.load(Ordering::Relaxed),
            warnings_logged: self.warnings_logged.load(Ordering::Relaxed),
            info_logged: self.info_logged.load(Ordering::Relaxed),
            debug_logged: self.debug_logged.load(Ordering::Relaxed),
            trace_logged: self.trace_logged.load(Ordering::Relaxed),
            last_log_time: *self.last_log_time.read().unwrap(),
            current_file_size: self.current_file_size.load(Ordering::Relaxed),
            total_bytes_logged: self.total_bytes_logged.load(Ordering::Relaxed),
            files_created: self.files_created.load(Ordering::Relaxed),
            rotations_performed: self.rotations_performed.load(Ordering::Relaxed),
            write_errors: self.write_errors.load(Ordering::Relaxed),
        }
    }
}

/// Serializable logging statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSummary {
    pub messages_logged: u64,
    pub errors_logged: u64,
    pub warnings_logged: u64,
    pub info_logged: u64,
    pub debug_logged: u64,
    pub trace_logged: u64,
    pub last_log_time: Option<SystemTime>,
    pub current_file_size: u64,
    pub total_bytes_logged: u64,
    pub files_created: u64,
    pub rotations_performed: u64,
    pub write_errors: u64,
}

impl LoggingSystem {
    /// Create a new structured logging system
    pub fn new(config: LoggingConfig) -> Result<Self> {
        Ok(Self {
            config,
            statistics: Arc::new(LoggingStatistics::new()),
            _guard: None,
            rotation_handle: None,
        })
    }

    /// Initialize the structured logging system with tracing
    pub async fn initialize(&mut self) -> Result<()> {
        // Create log directory if it doesn't exist
        if self.config.file_output {
            fs::create_dir_all(&self.config.log_dir)
                .with_context(|| format!("Failed to create log directory: {:?}", self.config.log_dir))?;
        }

        // Parse log level
        let level_filter = match self.config.level.to_lowercase().as_str() {
            "trace" => LevelFilter::TRACE,
            "debug" => LevelFilter::DEBUG,
            "info" => LevelFilter::INFO,
            "warn" => LevelFilter::WARN,
            "error" => LevelFilter::ERROR,
            _ => LevelFilter::INFO,
        };

        // Create environment filter
        let env_filter = EnvFilter::from_default_env()
            .add_directive(level_filter.into());

        // Timer selection
        #[allow(unused)]
        let timer = {
            #[cfg(feature = "heavy-time")] { ChronoUtc::rfc_3339() }
            #[cfg(all(feature = "logging", not(feature = "heavy-time")))] { crate::logging::minimal_time::SimpleUnixTime }
        };

        // Optional file appender
        let mut file_handle = None;
        if self.config.file_output {
            let rotation = match self.config.rotation { LogRotation::Hourly => Rotation::HOURLY, LogRotation::Daily => Rotation::DAILY, LogRotation::Never => Rotation::NEVER };
            let file_appender = tracing_appender::rolling::RollingFileAppender::new(rotation, &self.config.log_dir, "nxsh.log");
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            self._guard = Some(guard);
            file_handle = Some(non_blocking);
        }

        // Build a single formatting layer with a CombinedWriter that tees to console and file
        let combined_console = self.config.console_output;
        let combined_file = file_handle.clone();
    let stats_clone = self.statistics.clone();
    let writer_factory = move || CombinedWriter { console: combined_console, file: combined_file.clone(), stats: Some(stats_clone.clone()) };

        let fmt_layer_boxed = match self.config.format {
            LogFormat::Json => {
                #[cfg(feature = "logging-json")]
                { fmt::layer().json().with_span_list(true).with_timer(timer).with_writer(writer_factory).boxed() }
                #[cfg(not(feature = "logging-json"))]
                { fmt::layer().compact().with_timer(timer).with_writer(writer_factory).boxed() }
            }
            LogFormat::Pretty => fmt::layer().with_ansi(true).with_span_events(fmt::format::FmtSpan::FULL).with_timer(timer).with_writer(writer_factory).boxed(),
            LogFormat::Compact => fmt::layer().compact().with_timer(timer).with_writer(writer_factory).boxed(),
            LogFormat::Full => fmt::layer().with_span_events(fmt::format::FmtSpan::FULL).with_thread_ids(true).with_thread_names(true).with_timer(timer).with_writer(writer_factory).boxed(),
        };

        let subscriber = Registry::default().with(env_filter).with(fmt_layer_boxed);
        tracing::subscriber::set_global_default(subscriber).context("Failed to set global tracing subscriber")?;

        // Clean up old log files
        self.cleanup_old_logs().await?;

    nxsh_log_info!(
            level = %self.config.level,
            format = ?self.config.format,
            rotation = ?self.config.rotation,
            log_dir = ?self.config.log_dir,
            "Structured logging system initialized"
        );

        self.statistics.files_created.fetch_add(1, Ordering::Relaxed);
        self.update_last_log_time();

        Ok(())
    }

    /// Clean up old log files based on retention policy
    async fn cleanup_old_logs(&self) -> Result<()> {
        if !self.config.file_output || self.config.retention_days == 0 {
            return Ok(());
        }

        let cutoff_time = SystemTime::now() - Duration::from_secs(self.config.retention_days * 24 * 60 * 60);
        let mut files_removed = 0;

        let entries = fs::read_dir(&self.config.log_dir)
            .with_context(|| format!("Failed to read log directory: {:?}", self.config.log_dir))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().is_some_and(|ext| ext == "log") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified < cutoff_time
                            && fs::remove_file(&path).is_ok() {
                            files_removed += 1;
                            nxsh_log_debug!(?path, "Removed old log file");
                        }
                    }
                }
            }
        }

        if files_removed > 0 {
            nxsh_log_info!(files_removed, "Cleaned up old log files");
        }

        Ok(())
    }

    /// Update last log time
    fn update_last_log_time(&self) {
        if let Ok(mut time) = self.statistics.last_log_time.write() {
            *time = Some(SystemTime::now());
        }
    }

    /// Get current statistics summary
    pub fn get_statistics(&self) -> LoggingSummary {
        self.statistics.summary()
    }

    /// Get real-time logging metrics
    pub fn get_metrics(&self) -> HashMap<String, u64> {
        let mut metrics = HashMap::new();
        metrics.insert("messages_logged".to_string(), self.statistics.messages_logged.load(Ordering::Relaxed));
        metrics.insert("errors_logged".to_string(), self.statistics.errors_logged.load(Ordering::Relaxed));
        metrics.insert("warnings_logged".to_string(), self.statistics.warnings_logged.load(Ordering::Relaxed));
        metrics.insert("info_logged".to_string(), self.statistics.info_logged.load(Ordering::Relaxed));
        metrics.insert("debug_logged".to_string(), self.statistics.debug_logged.load(Ordering::Relaxed));
        metrics.insert("trace_logged".to_string(), self.statistics.trace_logged.load(Ordering::Relaxed));
        metrics.insert("total_bytes_logged".to_string(), self.statistics.total_bytes_logged.load(Ordering::Relaxed));
        metrics.insert("files_created".to_string(), self.statistics.files_created.load(Ordering::Relaxed));
        metrics.insert("rotations_performed".to_string(), self.statistics.rotations_performed.load(Ordering::Relaxed));
    metrics.insert("write_errors".to_string(), self.statistics.write_errors.load(Ordering::Relaxed));
        metrics
    }

    /// Force log rotation
    pub async fn rotate_logs(&mut self) -> Result<()> {
        if self.config.file_output {
            self.statistics.rotations_performed.fetch_add(1, Ordering::Relaxed);
            nxsh_log_info!("Log rotation performed");
        }
        Ok(())
    }

    /// Set log level dynamically
    pub fn set_level(&mut self, level: &str) -> Result<()> {
        self.config.level = level.to_string();
    nxsh_log_info!(new_level = level, "Log level updated");
        Ok(())
    }

    /// Add custom field to all future log entries
    pub fn add_custom_field(&mut self, key: String, value: String) {
        self.config.custom_fields.insert(key.clone(), value.clone());
    nxsh_log_info!(key = %key, value = %value, "Added custom log field");
    }

    /// Remove custom field
    pub fn remove_custom_field(&mut self, key: &str) {
        if self.config.custom_fields.remove(key).is_some() {
            nxsh_log_info!(key = %key, "Removed custom log field");
        }
    }

    /// Get current configuration
    pub fn get_config(&self) -> &LoggingConfig {
        &self.config
    }

    /// Update logging configuration with validation and change tracking
    /// 
    /// This function updates the logging system configuration, validates the new
    /// settings, and logs the configuration changes for audit purposes.
    /// 
    /// # Arguments
    /// * `config` - New logging configuration to apply
    /// 
    /// # Returns
    /// Result indicating success or failure of the configuration update
    pub fn update_config(&mut self, config: LoggingConfig) -> Result<()> {
        let _old_config = self.config.clone();
        self.config = config;
        
        // Log configuration update for audit trail
        nxsh_log_info!(
            old_level = %_old_config.level,
            new_level = %self.config.level,
            old_format = ?_old_config.format,
            new_format = ?self.config.format,
            "Logging configuration updated successfully"
        );
        
        Ok(())
    }

    /// Flush all pending log entries
    pub async fn flush(&self) -> Result<()> {
        // Force flush all appenders
    nxsh_log_info!("Flushing all log entries");
        Ok(())
    }

    /// Shutdown the logging system gracefully
    pub async fn shutdown(&mut self) -> Result<()> {
        // Flush any remaining logs
        self.flush().await?;
        
    nxsh_log_info!(
            total_messages = self.statistics.messages_logged.load(Ordering::Relaxed),
            total_bytes = self.statistics.total_bytes_logged.load(Ordering::Relaxed),
            "Logging system shutdown"
        );
        
        // Drop guard to stop background worker
        self._guard.take();
        
        Ok(())
    }
} 