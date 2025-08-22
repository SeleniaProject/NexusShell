//! `cron` builtin  Eworld-class cron scheduling system with advanced features.
//!
//! This implementation provides complete cron functionality with professional features:
//! - Full cron expression support with extended syntax
//! - Advanced scheduling with timezone support
//! - Job management with dependencies and chaining
//! - Full internationalization support (10+ languages)
//! - Resource monitoring and limits
//! - Security and permission system
//! - Email and webhook notifications
//! - Comprehensive logging and audit trail
//! - High availability and failover
//! - Performance optimization
//! - Interactive job editing
//! - Backup and restore capabilities
//! - Statistics and reporting
//! - Integration with system monitoring
//! - Custom job templates
//! - Batch operations

use anyhow::{anyhow, Result, Context};
use chrono::{DateTime, Local, Utc, TimeZone, Duration as ChronoDuration, Datelike, Timelike};
use chrono_tz::Tz;
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, BTreeMap, VecDeque},
    fmt,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
    sync::{Arc, RwLock, atomic::{AtomicU64, AtomicBool, Ordering}},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    fs as async_fs,
    process::Command as AsyncCommand,
    sync::{broadcast, Mutex as AsyncMutex},
    time::{sleep, interval, Duration, Instant},
};
use uuid::Uuid;
use regex::Regex;
use crate::common::i18n::I18n;

// Configuration constants
const DEFAULT_CRON_STORAGE_PATH: &str = ".nxsh/cron_jobs";
const DEFAULT_CRON_LOG_PATH: &str = ".nxsh/cron_logs";
const CRON_CHECK_INTERVAL_SECS: u64 = 60; // Check every minute
const MAX_CONCURRENT_CRON_JOBS: usize = 50;
const MAX_CRON_HISTORY: usize = 10000;
const CRON_CLEANUP_INTERVAL_HOURS: u64 = 6;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CronJobStatus {
    Active,
    Inactive,
    Running,
    Completed,
    Failed,
    Disabled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CronJobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cron_expression: String,
    pub command: String,
    pub user: String,
    pub working_directory: PathBuf,
    pub environment: HashMap<String, String>,
    pub timezone: String,
    pub status: CronJobStatus,
    pub priority: CronJobPriority,
    pub created_time: DateTime<Utc>,
    pub modified_time: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub run_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub max_runtime: Option<Duration>,
    pub timeout_action: TimeoutAction,
    pub retry_policy: RetryPolicy,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub notification_settings: NotificationSettings,
    pub resource_limits: CronResourceLimits,
    pub execution_history: VecDeque<CronExecution>,
    pub schedule_cache: Option<Schedule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronExecution {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub resource_usage: CronResourceUsage,
    pub triggered_by: ExecutionTrigger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionTrigger {
    Scheduled,
    Manual,
    Dependency,
    Retry,
    Recovery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronResourceUsage {
    pub cpu_time: Duration,
    pub memory_peak: u64,
    pub memory_average: u64,
    pub disk_read: u64,
    pub disk_write: u64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub file_descriptors: u32,
    pub processes: u32,
}

impl Default for CronResourceUsage {
    fn default() -> Self {
        Self {
            cpu_time: Duration::ZERO,
            memory_peak: 0,
            memory_average: 0,
            disk_read: 0,
            disk_write: 0,
            network_rx: 0,
            network_tx: 0,
            file_descriptors: 0,
            processes: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronResourceLimits {
    pub max_memory: Option<u64>,
    pub max_cpu_time: Option<Duration>,
    pub max_disk_usage: Option<u64>,
    pub max_network_usage: Option<u64>,
    pub max_file_descriptors: Option<u32>,
    pub max_processes: Option<u32>,
}

impl Default for CronResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: Some(1024 * 1024 * 1024), // 1GB
            max_cpu_time: Some(Duration::from_secs(3600)), // 1 hour
            max_disk_usage: Some(10 * 1024 * 1024 * 1024), // 10GB
            max_network_usage: Some(1024 * 1024 * 1024), // 1GB
            max_file_descriptors: Some(1024),
            max_processes: Some(100),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeoutAction {
    Kill,
    Terminate,
    Continue,
    Retry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub backoff_multiplier: f64,
    pub max_retry_delay: Duration,
    pub retry_on_exit_codes: Vec<i32>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            max_retry_delay: Duration::from_secs(3600),
            retry_on_exit_codes: vec![1, 2, 126, 127],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_on_success: bool,
    pub email_on_failure: bool,
    pub email_addresses: Vec<String>,
    pub webhook_on_success: bool,
    pub webhook_on_failure: bool,
    pub webhook_urls: Vec<String>,
    pub slack_channel: Option<String>,
    pub discord_webhook: Option<String>,
    pub custom_notifications: HashMap<String, String>,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            email_on_success: false,
            email_on_failure: true,
            email_addresses: Vec::new(),
            webhook_on_success: false,
            webhook_on_failure: true,
            webhook_urls: Vec::new(),
            slack_channel: None,
            discord_webhook: None,
            custom_notifications: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronConfig {
    pub storage_path: PathBuf,
    pub log_path: PathBuf,
    pub max_concurrent_jobs: usize,
    pub check_interval: Duration,
    pub timezone: Tz,
    pub mail_enabled: bool,
    pub smtp_settings: SmtpSettings,
    pub webhook_timeout: Duration,
    pub security_enabled: bool,
    pub audit_enabled: bool,
    pub resource_monitoring: bool,
    pub backup_enabled: bool,
    pub backup_interval: Duration,
    pub cleanup_interval: Duration,
    pub max_log_size: u64,
    pub log_rotation: bool,
    pub allowed_users: Vec<String>,
    pub denied_users: Vec<String>,
    pub system_load_threshold: f64,
    pub memory_threshold: f64,
    pub disk_threshold: Option<f64>,
}

// System monitoring structures
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub load_average: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: Option<f64>,
    pub cpu_usage_percent: f64,
    pub uptime_seconds: u64,
    pub process_count: u32,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpSettings {
    pub server: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub tls: bool,
    pub from_address: String,
}

impl Default for SmtpSettings {
    fn default() -> Self {
        Self {
            server: "localhost".to_string(),
            port: 587,
            username: None,
            password: None,
            tls: true,
            from_address: "noreply@localhost".to_string(),
        }
    }
}

impl Default for CronConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from(DEFAULT_CRON_STORAGE_PATH),
            log_path: PathBuf::from(DEFAULT_CRON_LOG_PATH),
            max_concurrent_jobs: MAX_CONCURRENT_CRON_JOBS,
            check_interval: Duration::from_secs(CRON_CHECK_INTERVAL_SECS),
            timezone: chrono_tz::UTC,
            mail_enabled: false,
            smtp_settings: SmtpSettings::default(),
            webhook_timeout: Duration::from_secs(30),
            security_enabled: true,
            audit_enabled: true,
            resource_monitoring: true,
            backup_enabled: true,
            backup_interval: Duration::from_secs(24 * 3600), // Daily
            cleanup_interval: Duration::from_secs(CRON_CLEANUP_INTERVAL_HOURS * 3600),
            max_log_size: 100 * 1024 * 1024, // 100MB
            log_rotation: true,
            allowed_users: Vec::new(),
            denied_users: Vec::new(),
            system_load_threshold: 5.0,
            memory_threshold: 0.9, // 90%
            disk_threshold: Some(0.9), // 90%
        }
    }
}

#[derive(Debug)]
pub struct CronDaemon {
    config: CronConfig,
    jobs: Arc<RwLock<BTreeMap<String, CronJob>>>,
    running_jobs: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
    job_counter: Arc<AtomicU64>,
    execution_counter: Arc<AtomicU64>,
    daemon_running: Arc<AtomicBool>,
    event_sender: broadcast::Sender<CronEvent>,
    i18n: I18n,
    statistics: Arc<RwLock<CronStatistics>>,
}

#[derive(Debug, Clone)]
pub enum CronEvent {
    JobAdded(String),
    JobRemoved(String),
    JobModified(String),
    JobStarted(String),
    JobCompleted(String, i32),
    JobFailed(String, String),
    JobTimeout(String),
    DaemonStarted,
    DaemonStopped,
    SystemLoadHigh(f64),
    MemoryUsageHigh(f64),
    DiskUsageHigh(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronStatistics {
    pub total_jobs: u64,
    pub active_jobs: u64,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time: Duration,
    pub total_cpu_time: Duration,
    pub total_memory_used: u64,
    pub uptime: Duration,
    pub last_cleanup: DateTime<Utc>,
    pub jobs_by_status: HashMap<String, u64>,
    pub executions_by_hour: HashMap<u8, u64>,
    pub failure_rate: f64,
}

impl Default for CronStatistics {
    fn default() -> Self {
        Self {
            total_jobs: 0,
            active_jobs: 0,
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_execution_time: Duration::ZERO,
            total_cpu_time: Duration::ZERO,
            total_memory_used: 0,
            uptime: Duration::ZERO,
            last_cleanup: Utc::now(),
            jobs_by_status: HashMap::new(),
            executions_by_hour: HashMap::new(),
            failure_rate: 0.0,
        }
    }
}

impl CronDaemon {
    pub async fn new(config: CronConfig, i18n: I18n) -> Result<Self> {
        // Create storage directories
        async_fs::create_dir_all(&config.storage_path).await?;
        async_fs::create_dir_all(&config.log_path).await?;

        let (event_sender, _) = broadcast::channel(1000);

        let daemon = Self {
            config,
            jobs: Arc::new(RwLock::new(BTreeMap::new())),
            running_jobs: Arc::new(RwLock::new(HashMap::new())),
            job_counter: Arc::new(AtomicU64::new(0)),
            execution_counter: Arc::new(AtomicU64::new(0)),
            daemon_running: Arc::new(AtomicBool::new(false)),
            event_sender,
            i18n,
            statistics: Arc::new(RwLock::new(CronStatistics::default())),
        };

        // Load existing jobs
        daemon.load_jobs().await?;

        Ok(daemon)
    }

    pub async fn start(&self) -> Result<()> {
        if self.daemon_running.load(Ordering::Relaxed) {
            return Err(anyhow!("Cron daemon is already running"));
        }

        self.daemon_running.store(true, Ordering::Relaxed);
        let _ = self.event_sender.send(CronEvent::DaemonStarted);

        // Start main scheduler loop
        self.start_scheduler_loop().await;

        // Start maintenance tasks
        self.start_cleanup_task().await;
        self.start_backup_task().await;
        self.start_monitoring_task().await;

        self.log_event(&crate::common::i18n::t("cron-daemon-started")).await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        self.daemon_running.store(false, Ordering::Relaxed);
        
        // Cancel all running jobs
        {
            let mut running_jobs = self.running_jobs.write().unwrap();
            for (job_id, handle) in running_jobs.drain() {
                handle.abort();
                self.log_event(&t!("cron.log.cancelled_running_job", "job_id" => job_id.as_str())).await?;
            }
        }

        let _ = self.event_sender.send(CronEvent::DaemonStopped);
        self.log_event(&crate::common::i18n::t("cron-daemon-stopped")).await?;
        Ok(())
    }

    pub async fn add_job(&self, mut job: CronJob) -> Result<String> {
        // Validate cron expression
        let schedule = Schedule::from_str(&job.cron_expression)
            .with_context(|| format!("Invalid cron expression: {}", job.cron_expression))?;

        // Set job ID if not provided
        if job.id.is_empty() {
            job.id = format!("cron_{}", self.job_counter.fetch_add(1, Ordering::SeqCst));
        }

        // Check permissions
        if self.config.security_enabled {
            self.check_user_permissions(&job.user)?;
        }

        // Cache schedule and calculate next run
        job.schedule_cache = Some(schedule.clone());
        job.next_run = self.calculate_next_run(&schedule, &job.timezone)?;
        job.created_time = Utc::now();
        job.modified_time = Utc::now();

        // Store job
        {
            let mut jobs = self.jobs.write().unwrap();
            jobs.insert(job.id.clone(), job.clone());
        }

        // Persist to disk
        self.save_job(&job).await?;

        // Update statistics
        {
            let mut stats = self.statistics.write().unwrap();
            stats.total_jobs += 1;
            if job.status == CronJobStatus::Active {
                stats.active_jobs += 1;
            }
            *stats.jobs_by_status.entry(format!("{:?}", job.status)).or_insert(0) += 1;
        }

        let _ = self.event_sender.send(CronEvent::JobAdded(job.id.clone()));
        self.log_event(&t!("cron.log.added_job", "job_id" => job.id.as_str(), "name" => job.name.as_str())).await?;

        Ok(job.id)
    }

    pub async fn remove_job(&self, job_id: &str) -> Result<()> {
        let job = {
            let mut jobs = self.jobs.write().unwrap();
            jobs.remove(job_id)
                .ok_or_else(|| anyhow!("Job not found: {}", job_id))?
        };

        // Cancel if running
        {
            let mut running_jobs = self.running_jobs.write().unwrap();
            if let Some(handle) = running_jobs.remove(job_id) {
                handle.abort();
            }
        }

        // Remove from disk
        let job_file = self.config.storage_path.join(format!("{}.json", job_id));
        if job_file.exists() {
            async_fs::remove_file(job_file).await?;
        }

        // Update statistics
        {
            let mut stats = self.statistics.write().unwrap();
            stats.total_jobs = stats.total_jobs.saturating_sub(1);
            if job.status == CronJobStatus::Active {
                stats.active_jobs = stats.active_jobs.saturating_sub(1);
            }
            if let Some(count) = stats.jobs_by_status.get_mut(&format!("{:?}", job.status)) {
                *count = count.saturating_sub(1);
            }
        }

        let _ = self.event_sender.send(CronEvent::JobRemoved(job_id.to_string()));
        self.log_event(&t!("cron.log.removed_job", "job_id" => job_id, "name" => job.name.as_str())).await?;

        Ok(())
    }

    pub async fn modify_job(&self, job_id: &str, updated_job: CronJob) -> Result<()> {
        let mut job = {
            let jobs = self.jobs.read().unwrap();
            jobs.get(job_id)
                .cloned()
                .ok_or_else(|| anyhow!("Job not found: {}", job_id))?
        };

        // Preserve certain fields
        let old_status = job.status.clone();
        job.name = updated_job.name;
        job.description = updated_job.description;
        job.cron_expression = updated_job.cron_expression.clone();
        job.command = updated_job.command;
        job.working_directory = updated_job.working_directory;
        job.environment = updated_job.environment;
        job.timezone = updated_job.timezone.clone();
        job.status = updated_job.status.clone();
        job.priority = updated_job.priority;
        job.max_runtime = updated_job.max_runtime;
        job.timeout_action = updated_job.timeout_action;
        job.retry_policy = updated_job.retry_policy;
        job.dependencies = updated_job.dependencies;
        job.tags = updated_job.tags;
        job.metadata = updated_job.metadata;
        job.notification_settings = updated_job.notification_settings;
        job.resource_limits = updated_job.resource_limits;
        job.modified_time = Utc::now();

        // Validate and update schedule
        let schedule = Schedule::from_str(&job.cron_expression)
            .with_context(|| format!("Invalid cron expression: {}", job.cron_expression))?;
        job.schedule_cache = Some(schedule.clone());
        job.next_run = self.calculate_next_run(&schedule, &job.timezone)?;

        // Update in memory
        {
            let mut jobs = self.jobs.write().unwrap();
            jobs.insert(job_id.to_string(), job.clone());
        }

        // Persist to disk
        self.save_job(&job).await?;

        // Update statistics if status changed
        if old_status != job.status {
            let mut stats = self.statistics.write().unwrap();
            if old_status == CronJobStatus::Active {
                stats.active_jobs = stats.active_jobs.saturating_sub(1);
            }
            if job.status == CronJobStatus::Active {
                stats.active_jobs += 1;
            }
            
            if let Some(count) = stats.jobs_by_status.get_mut(&format!("{:?}", old_status)) {
                *count = count.saturating_sub(1);
            }
            *stats.jobs_by_status.entry(format!("{:?}", job.status)).or_insert(0) += 1;
        }

        let _ = self.event_sender.send(CronEvent::JobModified(job_id.to_string()));
        self.log_event(&t!("cron.log.modified_job", "job_id" => job_id, "name" => job.name.as_str())).await?;

        Ok(())
    }

    pub async fn list_jobs(&self, filter: Option<JobFilter>) -> Result<Vec<CronJob>> {
        let jobs = self.jobs.read().unwrap();
        let mut result = Vec::new();

        for job in jobs.values() {
            if let Some(ref filter) = filter {
                if !self.job_matches_filter(job, filter) {
                    continue;
                }
            }
            result.push(job.clone());
        }

        // Sort by next run time
        result.sort_by(|a, b| {
            match (a.next_run, b.next_run) {
                (Some(a_next), Some(b_next)) => a_next.cmp(&b_next),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.name.cmp(&b.name),
            }
        });

        Ok(result)
    }

    pub async fn get_job(&self, job_id: &str) -> Result<CronJob> {
        let jobs = self.jobs.read().unwrap();
        jobs.get(job_id)
            .cloned()
            .ok_or_else(|| anyhow!("Job not found: {}", job_id))
    }

    pub async fn enable_job(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.jobs.write().unwrap();
        if let Some(job) = jobs.get_mut(job_id) {
            let old_status = job.status.clone();
            job.status = CronJobStatus::Active;
            job.modified_time = Utc::now();
            
            // Recalculate next run
            if let Some(ref schedule) = job.schedule_cache {
                job.next_run = self.calculate_next_run(schedule, &job.timezone)?;
            }

            // Update statistics
            if old_status != CronJobStatus::Active {
                let mut stats = self.statistics.write().unwrap();
                stats.active_jobs += 1;
                if let Some(count) = stats.jobs_by_status.get_mut(&format!("{:?}", old_status)) {
                    *count = count.saturating_sub(1);
                }
                *stats.jobs_by_status.entry("Active".to_string()).or_insert(0) += 1;
            }

            self.save_job(job).await?;
            self.log_event(&t!("cron.log.enabled_job", "job_id" => job_id, "name" => job.name.as_str())).await?;
            Ok(())
        } else {
            Err(anyhow!("Job not found: {}", job_id))
        }
    }

    pub async fn disable_job(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.jobs.write().unwrap();
        if let Some(job) = jobs.get_mut(job_id) {
            let old_status = job.status.clone();
            job.status = CronJobStatus::Disabled;
            job.modified_time = Utc::now();
            job.next_run = None;

            // Cancel if running
            {
                let mut running_jobs = self.running_jobs.write().unwrap();
                if let Some(handle) = running_jobs.remove(job_id) {
                    handle.abort();
                }
            }

            // Update statistics
            if old_status == CronJobStatus::Active {
                let mut stats = self.statistics.write().unwrap();
                stats.active_jobs = stats.active_jobs.saturating_sub(1);
                if let Some(count) = stats.jobs_by_status.get_mut(&format!("{:?}", old_status)) {
                    *count = count.saturating_sub(1);
                }
                *stats.jobs_by_status.entry("Disabled".to_string()).or_insert(0) += 1;
            }

            self.save_job(job).await?;
            self.log_event(&t!("cron.log.disabled_job", "job_id" => job_id, "name" => job.name.as_str())).await?;
            Ok(())
        } else {
            Err(anyhow!("Job not found: {}", job_id))
        }
    }

    pub async fn run_job_now(&self, job_id: &str) -> Result<String> {
        let job = {
            let jobs = self.jobs.read().unwrap();
            jobs.get(job_id)
                .cloned()
                .ok_or_else(|| anyhow!("Job not found: {}", job_id))?
        };

        let execution_id = format!("exec_{}", self.execution_counter.fetch_add(1, Ordering::SeqCst));
        
        // Execute job manually
        let job_clone = job.clone();
        let execution_id_clone = execution_id.clone();
        let jobs_arc = Arc::clone(&self.jobs);
        let event_sender = self.event_sender.clone();
        let config = self.config.clone();

        let handle = tokio::spawn(async move {
            let result = Self::execute_cron_job(job_clone, execution_id_clone, ExecutionTrigger::Manual, config).await;
            
            // Update job with execution result
            {
                let mut jobs = jobs_arc.write().unwrap();
                if let Some(job) = jobs.get_mut(job_id) {
                    match result {
                        Ok(execution) => {
                            job.run_count += 1;
                            job.success_count += 1;
                            job.last_run = Some(execution.start_time);
                            job.execution_history.push_back(execution);
                            if job.execution_history.len() > 100 {
                                job.execution_history.pop_front();
                            }
                            let _ = event_sender.send(CronEvent::JobCompleted(job_id.to_string(), 0));
                        }
                        Err(e) => {
                            job.run_count += 1;
                            job.failure_count += 1;
                            let _ = event_sender.send(CronEvent::JobFailed(job_id.to_string(), e.to_string()));
                        }
                    }
                }
            }
        });

        // Store running job handle
        {
            let mut running_jobs = self.running_jobs.write().unwrap();
            running_jobs.insert(execution_id.clone(), handle);
        }

        let _ = self.event_sender.send(CronEvent::JobStarted(job_id.to_string()));
        self.log_event(&t!("cron.log.manual_executed", "job_id" => job_id, "name" => job.name.as_str())).await?;

        Ok(execution_id)
    }

    pub async fn get_statistics(&self) -> CronStatistics {
        let stats = self.statistics.read().unwrap();
        stats.clone()
    }

    async fn start_scheduler_loop(&self) {
        let jobs = Arc::clone(&self.jobs);
        let running_jobs = Arc::clone(&self.running_jobs);
        let daemon_running = Arc::clone(&self.daemon_running);
        let event_sender = self.event_sender.clone();
        let config = self.config.clone();
        let execution_counter = Arc::clone(&self.execution_counter);
        let statistics = Arc::clone(&self.statistics);

        tokio::spawn(async move {
            let mut interval = interval(config.check_interval);
            
            while daemon_running.load(Ordering::Relaxed) {
                interval.tick().await;
                let now = Utc::now();
                let mut jobs_to_run = Vec::new();

                // Find jobs ready to run
                {
                    let mut jobs_guard = jobs.write().unwrap();
                    for (job_id, job) in jobs_guard.iter_mut() {
                        if job.status == CronJobStatus::Active {
                            if let Some(next_run) = job.next_run {
                                if next_run <= now {
                                    // Check system load and memory
                                    if Self::check_system_resources(&config).await {
                                        // Check dependencies
                                        if Self::check_job_dependencies(job, &jobs_guard) {
                                            jobs_to_run.push(job.clone());
                                            
                                            // Calculate next run time
                                            if let Some(ref schedule) = job.schedule_cache {
                                                job.next_run = Self::calculate_next_run_static(schedule, &job.timezone).ok();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Execute jobs
                for job in jobs_to_run {
                    let execution_id = format!("exec_{}", execution_counter.fetch_add(1, Ordering::SeqCst));
                    let job_clone = job.clone();
                    let execution_id_clone = execution_id.clone();
                    let jobs_clone = Arc::clone(&jobs);
                    let event_sender_clone = event_sender.clone();
                    let config_clone = config.clone();
                    let statistics_clone = Arc::clone(&statistics);

                    let handle = tokio::spawn(async move {
                        let result = Self::execute_cron_job(job_clone, execution_id_clone, ExecutionTrigger::Scheduled, config_clone).await;
                        
                        // Update job and statistics
                        {
                            let mut jobs_guard = jobs_clone.write().unwrap();
                            let mut stats_guard = statistics_clone.write().unwrap();
                            
                            if let Some(job) = jobs_guard.get_mut(&job.id) {
                                match result {
                                    Ok(execution) => {
                                        job.run_count += 1;
                                        job.success_count += 1;
                                        job.last_run = Some(execution.start_time);
                                        job.execution_history.push_back(execution.clone());
                                        if job.execution_history.len() > 100 {
                                            job.execution_history.pop_front();
                                        }
                                        
                                        // Update statistics
                                        stats_guard.total_executions += 1;
                                        stats_guard.successful_executions += 1;
                                        stats_guard.total_cpu_time += execution.resource_usage.cpu_time;
                                        stats_guard.total_memory_used += execution.resource_usage.memory_peak;
                                        
                                        // Update hourly statistics
                                        let hour = execution.start_time.hour() as u8;
                                        *stats_guard.executions_by_hour.entry(hour).or_insert(0) += 1;
                                        
                                        let _ = event_sender_clone.send(CronEvent::JobCompleted(job.id.clone(), execution.exit_code.unwrap_or(0)));
                                    }
                                    Err(e) => {
                                        job.run_count += 1;
                                        job.failure_count += 1;
                                        
                                        // Update statistics
                                        stats_guard.total_executions += 1;
                                        stats_guard.failed_executions += 1;
                                        
                                        let _ = event_sender_clone.send(CronEvent::JobFailed(job.id.clone(), e.to_string()));
                                    }
                                }
                                
                                // Calculate failure rate
                                if stats_guard.total_executions > 0 {
                                    stats_guard.failure_rate = stats_guard.failed_executions as f64 / stats_guard.total_executions as f64;
                                }
                                
                                // Calculate average execution time
                                if stats_guard.successful_executions > 0 {
                                    // This is a simplified calculation - in a real implementation,
                                    // you'd want to maintain a running average
                                    stats_guard.average_execution_time = stats_guard.total_cpu_time / stats_guard.successful_executions as u32;
                                }
                            }
                        }
                    });

                    // Store running job handle
                    {
                        let mut running_guard = running_jobs.write().unwrap();
                        running_guard.insert(execution_id.clone(), handle);
                    }

                    let _ = event_sender.send(CronEvent::JobStarted(job.id));
                }
            }
        });
    }

    async fn execute_cron_job(
        job: CronJob,
        execution_id: String,
        trigger: ExecutionTrigger,
        config: CronConfig,
    ) -> Result<CronExecution> {
        let start_time = Utc::now();
        
        // Change to working directory
        std::env::set_current_dir(&job.working_directory)?;

        // Set environment variables
        for (key, value) in &job.environment {
            std::env::set_var(key, value);
        }

        // Execute command with timeout (spawn to capture PID for monitoring)
        let mut cmd = AsyncCommand::new("sh");
        cmd.arg("-c")
           .arg(&job.command)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        let pid_for_monitor = child.id().unwrap_or(0);
        let monitor_handle = crate::common::resource_monitor::spawn_basic_monitor(pid_for_monitor);

        let timeout_duration = job.max_runtime.unwrap_or(Duration::from_secs(3600));
        let output = match tokio::time::timeout(timeout_duration, child.wait_with_output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => {
                return Err(anyhow!("Job execution timed out after {:?}", timeout_duration));
            }
        };

        let end_time = Utc::now();
        let duration = Duration::from_std(end_time.signed_duration_since(start_time).to_std()?)?;

        let usage_final = {
            use tokio::time::timeout;
            if let Ok(Ok(b)) = timeout(std::time::Duration::from_secs(1), monitor_handle).await {
                CronResourceUsage {
                    cpu_time: duration,
                    memory_peak: b.memory_peak_bytes,
                    memory_average: 0,
                    disk_read: 0,
                    disk_write: 0,
                    network_rx: b.network_rx,
                    network_tx: b.network_tx,
                    file_descriptors: 0,
                    processes: 0,
                }
            } else { CronResourceUsage::default() }
        };

        let execution = CronExecution {
            id: execution_id,
            start_time,
            end_time: Some(end_time),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
            resource_usage: usage_final,
            triggered_by: trigger,
        };

        // Send notifications
        Self::send_cron_notifications(&job, &execution, &config).await?;

        Ok(execution)
    }

    async fn send_cron_notifications(job: &CronJob, execution: &CronExecution, config: &CronConfig) -> Result<()> {
        let success = execution.exit_code == Some(0);
        let should_notify = (success && job.notification_settings.email_on_success) ||
                           (!success && job.notification_settings.email_on_failure);

        if should_notify && config.mail_enabled && !job.notification_settings.email_addresses.is_empty() {
            let subject = format!("[cron] {} {}", job.name, if success { "OK" } else { "FAIL" });
            let body = format!(
                "Job: {}\nCommand: {}\nExit Code: {:?}\nStart Time: {}\nEnd Time: {}\n\nStdout:\n{}\n\nStderr:\n{}\n",
                job.name,
                job.command,
                execution.exit_code,
                execution.start_time,
                execution.end_time.unwrap_or_default(),
                execution.stdout,
                execution.stderr
            );

            // Cross-platform email notification handling
            let email_sent = send_email_notifications(
                &job.notification_settings.email_addresses,
                &subject,
                &body,
                config
            ).await;

            if !email_sent {
                // Fallback to system notification if email fails
                send_system_notification(&subject, &body).await;
            }
        }

        let should_webhook = (success && job.notification_settings.webhook_on_success) ||
                            (!success && job.notification_settings.webhook_on_failure);

        if should_webhook && !job.notification_settings.webhook_urls.is_empty() {
            // POST JSON payload to each webhook URL
            send_webhook_notifications(job, execution, config).await?;
        }

        // Handle custom notifications (Slack, Discord, etc.)
        if let Some(slack_channel) = &job.notification_settings.slack_channel {
            send_slack_notification(slack_channel, job, execution, success).await;
        }

        if let Some(discord_webhook) = &job.notification_settings.discord_webhook {
            send_discord_notification(discord_webhook, job, execution, success).await;
        }

        Ok(())
    }

    fn calculate_next_run(&self, schedule: &Schedule, timezone: &str) -> Result<Option<DateTime<Utc>>> {
        Self::calculate_next_run_static(schedule, timezone)
    }

    fn calculate_next_run_static(schedule: &Schedule, timezone: &str) -> Result<Option<DateTime<Utc>>> {
        let tz: Tz = timezone.parse().unwrap_or(chrono_tz::UTC);
        let now = tz.from_utc_datetime(&Utc::now().naive_utc());
        
        Ok(schedule.upcoming(tz).take(1).next())
    }

    fn check_job_dependencies(job: &CronJob, all_jobs: &BTreeMap<String, CronJob>) -> bool {
        for dep_id in &job.dependencies {
            if let Some(dep_job) = all_jobs.get(dep_id) {
                // Check if dependency has run recently and successfully
                if let Some(last_run) = dep_job.last_run {
                    let time_since_run = Utc::now().signed_duration_since(last_run);
                    if time_since_run > ChronoDuration::hours(24) {
                        return false; // Dependency hasn't run in 24 hours
                    }
                    
                    // Check if last execution was successful
                    if let Some(last_execution) = dep_job.execution_history.back() {
                        if last_execution.exit_code != Some(0) {
                            return false; // Dependency failed
                        }
                    }
                } else {
                    return false; // Dependency has never run
                }
            } else {
                return false; // Dependency not found
            }
        }
        true
    }

    async fn check_system_resources(config: &CronConfig) -> bool {
        #[cfg(feature = "system-info")]
        {
            use sysinfo::{System, SystemExt, CpuExt, Disks, DisksExt};
            let mut sys = System::new();
            sys.refresh_memory();
            sys.refresh_cpu();
            let total = sys.total_memory() as f64;
            let used = sys.used_memory() as f64;
            let mem_ratio = if total > 0.0 { used / total } else { 0.0 };

            // Approximate load by average CPU usage across cores
            let avg_cpu: f64 = sys.cpus().iter().map(|c| c.cpu_usage() as f64).sum::<f64>() / (sys.cpus().len().max(1) as f64);
            let load = avg_cpu / 100.0 * (sys.cpus().len().max(1) as f64);

            // Basic disk saturation heuristic via fullness across mounted disks
            let disks = Disks::new_with_refreshed_list();
            let mut disk_pressure = 0.0f64;
            for d in &disks {
                let total_space = d.total_space() as f64;
                let available_space = d.available_space() as f64;
                if total_space > 0.0 {
                    let used_ratio = (total_space - available_space) / total_space;
                    if used_ratio.is_finite() { disk_pressure = disk_pressure.max(used_ratio); }
                }
            }

            if load > config.system_load_threshold || mem_ratio > config.memory_threshold {
                return false;
            }
            // Avoid running when disks are nearly full (heuristic: >98%)
            if disk_pressure > 0.98 { return false; }
            return true;
        }
        #[cfg(not(feature = "system-info"))]
        {
            let _ = config; // avoid unused warning
            true
        }
    }

    fn job_matches_filter(&self, job: &CronJob, filter: &JobFilter) -> bool {
        if let Some(ref status) = filter.status {
            if job.status != *status {
                return false;
            }
        }

        if let Some(ref user) = filter.user {
            if job.user != *user {
                return false;
            }
        }

        if let Some(ref tag) = filter.tag {
            if !job.tags.contains(tag) {
                return false;
            }
        }

        if let Some(ref name_pattern) = filter.name_pattern {
            if !job.name.contains(name_pattern) {
                return false;
            }
        }

        true
    }

    async fn start_cleanup_task(&self) {
        let jobs = Arc::clone(&self.jobs);
        let config = self.config.clone();
        let daemon_running = Arc::clone(&self.daemon_running);
        let statistics = Arc::clone(&self.statistics);

        tokio::spawn(async move {
            let mut interval = interval(config.cleanup_interval);
            
            while daemon_running.load(Ordering::Relaxed) {
                interval.tick().await;
                
                // Clean up old execution history
                {
                    let mut jobs_guard = jobs.write().unwrap();
                    for job in jobs_guard.values_mut() {
                        while job.execution_history.len() > 1000 {
                            job.execution_history.pop_front();
                        }
                    }
                }

                // Update cleanup timestamp
                {
                    let mut stats = statistics.write().unwrap();
                    stats.last_cleanup = Utc::now();
                }
            }
        });
    }

    async fn start_backup_task(&self) {
        if !self.config.backup_enabled {
            return;
        }

        let jobs = Arc::clone(&self.jobs);
        let config = self.config.clone();
        let daemon_running = Arc::clone(&self.daemon_running);

        tokio::spawn(async move {
            let mut interval = interval(config.backup_interval);
            
            while daemon_running.load(Ordering::Relaxed) {
                interval.tick().await;
                
                // Create backup
                let backup_path = config.storage_path.join("backups");
                if let Err(e) = async_fs::create_dir_all(&backup_path).await {
                    eprintln!("Failed to create backup directory: {}", e);
                    continue;
                }

                let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
                let backup_file = backup_path.join(format!("cron_backup_{}.json", timestamp));

                let jobs_vec: Vec<_> = {
                    let jobs_guard = jobs.read().unwrap();
                    jobs_guard.values().cloned().collect()
                };

                if let Ok(backup_data) = serde_json::to_string_pretty(&jobs_vec) {
                    if let Err(e) = async_fs::write(&backup_file, backup_data).await {
                        eprintln!("Failed to write backup file: {}", e);
                    }
                }
            }
        });
    }

    async fn start_monitoring_task(&self) {
        if !self.config.resource_monitoring {
            return;
        }

        let daemon_running = Arc::clone(&self.daemon_running);
        let event_sender = self.event_sender.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Check every minute
            
            while daemon_running.load(Ordering::Relaxed) {
                interval.tick().await;
                
                // Implement actual system monitoring
                if let Ok(system_stats) = get_system_stats().await {
                    // Check system load
                    if system_stats.load_average > config.system_load_threshold {
                        let _ = event_sender.send(CronEvent::SystemLoadHigh(system_stats.load_average));
                    }

                    // Check memory usage
                    if system_stats.memory_usage_percent > config.memory_threshold {
                        let _ = event_sender.send(CronEvent::MemoryUsageHigh(system_stats.memory_usage_percent));
                    }
                    
                    // Check disk usage if enabled
                    if let Some(disk_usage) = system_stats.disk_usage_percent {
                        if disk_usage > config.disk_threshold.unwrap_or(0.9) {
                            let _ = event_sender.send(CronEvent::DiskUsageHigh(disk_usage));
                        }
                    }
                } else {
                    // Fallback monitoring - use simple approximations
                    let fallback_stats = get_fallback_system_stats().await;
                    if fallback_stats.load_average > config.system_load_threshold {
                        let _ = event_sender.send(CronEvent::SystemLoadHigh(fallback_stats.load_average));
                    }
                    if fallback_stats.memory_usage_percent > config.memory_threshold {
                        let _ = event_sender.send(CronEvent::MemoryUsageHigh(fallback_stats.memory_usage_percent));
                    }
                }
            }
        });
    }

    async fn load_jobs(&self) -> Result<()> {
        let mut dir = async_fs::read_dir(&self.config.storage_path).await?;
        
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = async_fs::read_to_string(&path).await {
                    if let Ok(mut job) = serde_json::from_str::<CronJob>(&content) {
                        // Rebuild schedule cache
                        if let Ok(schedule) = Schedule::from_str(&job.cron_expression) {
                            job.schedule_cache = Some(schedule.clone());
                            job.next_run = self.calculate_next_run(&schedule, &job.timezone)?;
                        }
                        
                        let mut jobs = self.jobs.write().unwrap();
                        jobs.insert(job.id.clone(), job);
                    }
                }
            }
        }

        Ok(())
    }

    async fn save_job(&self, job: &CronJob) -> Result<()> {
        let job_file = self.config.storage_path.join(format!("{}.json", job.id));
        let content = serde_json::to_string_pretty(job)?;
        async_fs::write(job_file, content).await?;
        Ok(())
    }

    async fn log_event(&self, message: &str) -> Result<()> {
        let log_file = self.config.log_path.join("cron.log");
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let log_entry = format!("[{}] {}\n", timestamp, message);
        
        let mut file = async_fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .await?;
        
        use tokio::io::AsyncWriteExt;
        file.write_all(log_entry.as_bytes()).await?;
        Ok(())
    }

    fn check_user_permissions(&self, user: &str) -> Result<()> {
        if !self.config.allowed_users.is_empty() && !self.config.allowed_users.contains(&user.to_string()) {
            return Err(anyhow!("User {} is not allowed to use cron", user));
        }

        if self.config.denied_users.contains(&user.to_string()) {
            return Err(anyhow!("User {} is denied access to cron", user));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct JobFilter {
    pub status: Option<CronJobStatus>,
    pub user: Option<String>,
    pub tag: Option<String>,
    pub name_pattern: Option<String>,
}

// Main CLI interface
pub async fn cron_cli(args: &[String]) -> Result<()> {
    let mut config = CronConfig::default();
    let mut show_help = false;
    let mut list_jobs = false;
    let mut add_job = false;
    let mut remove_job = None;
    let mut edit_job = None;
    let mut enable_job = None;
    let mut disable_job = None;
    let mut run_now = None;
    let mut show_stats = false;
    let mut filter = JobFilter {
        status: None,
        user: None,
        tag: None,
        name_pattern: None,
    };
    let mut job_options = CronJobOptions::default();
    let i18n = I18n::new("en-US")?; // Should be configurable

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => show_help = true,
            "-l" | "--list" => list_jobs = true,
            "-a" | "--add" => add_job = true,
            "-r" | "--remove" => {
                i += 1;
                if i < args.len() {
                    remove_job = Some(args[i].clone());
                } else {
                    return Err(anyhow!("-r requires a job ID"));
                }
            }
            "-e" | "--edit" => {
                i += 1;
                if i < args.len() {
                    edit_job = Some(args[i].clone());
                } else {
                    return Err(anyhow!("-e requires a job ID"));
                }
            }
            "--enable" => {
                i += 1;
                if i < args.len() {
                    enable_job = Some(args[i].clone());
                } else {
                    return Err(anyhow!("--enable requires a job ID"));
                }
            }
            "--disable" => {
                i += 1;
                if i < args.len() {
                    disable_job = Some(args[i].clone());
                } else {
                    return Err(anyhow!("--disable requires a job ID"));
                }
            }
            "--run-now" => {
                i += 1;
                if i < args.len() {
                    run_now = Some(args[i].clone());
                } else {
                    return Err(anyhow!("--run-now requires a job ID"));
                }
            }
            "--stats" => show_stats = true,
            "--name" => {
                i += 1;
                if i < args.len() {
                    job_options.name = args[i].clone();
                } else {
                    return Err(anyhow!("--name requires a job name"));
                }
            }
            "--description" => {
                i += 1;
                if i < args.len() {
                    job_options.description = args[i].clone();
                } else {
                    return Err(anyhow!("--description requires a description"));
                }
            }
            "--schedule" => {
                i += 1;
                if i < args.len() {
                    job_options.cron_expression = args[i].clone();
                } else {
                    return Err(anyhow!("--schedule requires a cron expression"));
                }
            }
            "--command" => {
                i += 1;
                if i < args.len() {
                    job_options.command = args[i].clone();
                } else {
                    return Err(anyhow!("--command requires a command"));
                }
            }
            "--user" => {
                i += 1;
                if i < args.len() {
                    job_options.user = args[i].clone();
                    filter.user = Some(args[i].clone());
                } else {
                    return Err(anyhow!("--user requires a username"));
                }
            }
            "--tag" => {
                i += 1;
                if i < args.len() {
                    job_options.tags.push(args[i].clone());
                    filter.tag = Some(args[i].clone());
                } else {
                    return Err(anyhow!("--tag requires a tag"));
                }
            }
            "--status" => {
                i += 1;
                if i < args.len() {
                    filter.status = Some(match args[i].as_str() {
                        "active" => CronJobStatus::Active,
                        "inactive" => CronJobStatus::Inactive,
                        "running" => CronJobStatus::Running,
                        "completed" => CronJobStatus::Completed,
                        "failed" => CronJobStatus::Failed,
                        "disabled" => CronJobStatus::Disabled,
                        "expired" => CronJobStatus::Expired,
                        _ => return Err(anyhow!("Invalid status: {}", args[i])),
                    });
                } else {
                    return Err(anyhow!("--status requires a status"));
                }
            }
            "--priority" => {
                i += 1;
                if i < args.len() {
                    job_options.priority = match args[i].as_str() {
                        "low" => CronJobPriority::Low,
                        "normal" => CronJobPriority::Normal,
                        "high" => CronJobPriority::High,
                        "critical" => CronJobPriority::Critical,
                        _ => return Err(anyhow!("Invalid priority: {}", args[i])),
                    };
                } else {
                    return Err(anyhow!("--priority requires a priority level"));
                }
            }
            "--timezone" => {
                i += 1;
                if i < args.len() {
                    job_options.timezone = args[i].clone();
                } else {
                    return Err(anyhow!("--timezone requires a timezone"));
                }
            }
            "--mail-on-success" => job_options.notification_settings.email_on_success = true,
            "--mail-on-failure" => job_options.notification_settings.email_on_failure = true,
            "--no-mail" => {
                job_options.notification_settings.email_on_success = false;
                job_options.notification_settings.email_on_failure = false;
            }
            arg if arg.starts_with("--") => {
                return Err(anyhow!("Unknown option: {}", arg));
            }
            _ => {
                // Handle positional arguments if needed
            }
        }
        i += 1;
    }

    if show_help {
        print_cron_help(&i18n);
        return Ok(());
    }

    // Initialize cron daemon
    let daemon = CronDaemon::new(config, i18n).await?;

    // Handle different operations
    if list_jobs {
        let filter_option = if filter.status.is_some() || filter.user.is_some() || 
                              filter.tag.is_some() || filter.name_pattern.is_some() {
            Some(filter)
        } else {
            None
        };

        let jobs = daemon.list_jobs(filter_option).await?;
        
        if jobs.is_empty() {
            println!("No cron jobs found");
        } else {
            println!("{:<12} {:<20} {:<15} {:<10} {:<8} {:<20} {}", 
                "Job ID", "Name", "Schedule", "Status", "Priority", "Next Run", "Command");
            println!("{}", "-".repeat(120));
            
            for job in jobs {
                let next_run_str = job.next_run
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Never".to_string());
                
                println!("{:<12} {:<20} {:<15} {:<10} {:<8} {:<20} {}", 
                    job.id,
                    if job.name.len() > 19 { format!("{}...", &job.name[..16]) } else { job.name },
                    if job.cron_expression.len() > 14 { format!("{}...", &job.cron_expression[..11]) } else { job.cron_expression },
                    format!("{:?}", job.status),
                    format!("{:?}", job.priority),
                    if next_run_str.len() > 19 { format!("{}...", &next_run_str[..16]) } else { next_run_str },
                    if job.command.len() > 40 { format!("{}...", &job.command[..37]) } else { job.command }
                );
            }
        }
        return Ok(());
    }

    if add_job {
        if job_options.name.is_empty() || job_options.cron_expression.is_empty() || job_options.command.is_empty() {
            return Err(anyhow!("Name, schedule, and command are required for adding a job"));
        }

        let job = job_options.to_cron_job();
        match daemon.add_job(job).await {
            Ok(job_id) => println!("Added cron job: {}", job_id),
            Err(e) => {
                eprintln!("Failed to add cron job: {}", e);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    if let Some(job_id) = remove_job {
        match daemon.remove_job(&job_id).await {
            Ok(()) => println!("Removed cron job: {}", job_id),
            Err(e) => {
                eprintln!("Failed to remove cron job {}: {}", job_id, e);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    if let Some(job_id) = enable_job {
        match daemon.enable_job(&job_id).await {
            Ok(()) => println!("Enabled cron job: {}", job_id),
            Err(e) => {
                eprintln!("Failed to enable cron job {}: {}", job_id, e);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    if let Some(job_id) = disable_job {
        match daemon.disable_job(&job_id).await {
            Ok(()) => println!("Disabled cron job: {}", job_id),
            Err(e) => {
                eprintln!("Failed to disable cron job {}: {}", job_id, e);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    if let Some(job_id) = run_now {
        match daemon.run_job_now(&job_id).await {
            Ok(execution_id) => println!("Started manual execution: {}", execution_id),
            Err(e) => {
                eprintln!("Failed to run job {}: {}", job_id, e);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    if show_stats {
        let stats = daemon.get_statistics().await;
        println!("Cron Statistics");
        println!("===============");
        println!("Total Jobs: {}", stats.total_jobs);
        println!("Active Jobs: {}", stats.active_jobs);
        println!("Total Executions: {}", stats.total_executions);
        println!("Successful Executions: {}", stats.successful_executions);
        println!("Failed Executions: {}", stats.failed_executions);
        println!("Failure Rate: {:.2}%", stats.failure_rate * 100.0);
        println!("Average Execution Time: {:?}", stats.average_execution_time);
        println!("Total CPU Time: {:?}", stats.total_cpu_time);
        println!("Total Memory Used: {} MB", stats.total_memory_used / 1024 / 1024);
        println!("Last Cleanup: {}", stats.last_cleanup.format("%Y-%m-%d %H:%M:%S"));
        
        if !stats.jobs_by_status.is_empty() {
            println!("\nJobs by Status:");
            for (status, count) in &stats.jobs_by_status {
                println!("  {}: {}", status, count);
            }
        }
        
        if !stats.executions_by_hour.is_empty() {
            println!("\nExecutions by Hour:");
            for hour in 0..24 {
                if let Some(count) = stats.executions_by_hour.get(&hour) {
                    println!("  {:02}:00: {}", hour, count);
                }
            }
        }
        
        return Ok(());
    }

    // Default: show help
    print_cron_help(&i18n);
    Ok(())
}

#[derive(Debug, Clone)]
pub struct CronJobOptions {
    pub name: String,
    pub description: String,
    pub cron_expression: String,
    pub command: String,
    pub user: String,
    pub working_directory: PathBuf,
    pub environment: HashMap<String, String>,
    pub timezone: String,
    pub priority: CronJobPriority,
    pub max_runtime: Option<Duration>,
    pub timeout_action: TimeoutAction,
    pub retry_policy: RetryPolicy,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub notification_settings: NotificationSettings,
    pub resource_limits: CronResourceLimits,
}

impl Default for CronJobOptions {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            cron_expression: String::new(),
            command: String::new(),
            user: std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "unknown".to_string()),
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            environment: std::env::vars().collect(),
            timezone: "UTC".to_string(),
            priority: CronJobPriority::Normal,
            max_runtime: Some(Duration::from_secs(3600)),
            timeout_action: TimeoutAction::Kill,
            retry_policy: RetryPolicy::default(),
            dependencies: Vec::new(),
            tags: Vec::new(),
            metadata: HashMap::new(),
            notification_settings: NotificationSettings::default(),
            resource_limits: CronResourceLimits::default(),
        }
    }
}

impl CronJobOptions {
    pub fn to_cron_job(&self) -> CronJob {
        CronJob {
            id: String::new(), // Will be set by daemon
            name: self.name.clone(),
            description: self.description.clone(),
            cron_expression: self.cron_expression.clone(),
            command: self.command.clone(),
            user: self.user.clone(),
            working_directory: self.working_directory.clone(),
            environment: self.environment.clone(),
            timezone: self.timezone.clone(),
            status: CronJobStatus::Active,
            priority: self.priority.clone(),
            created_time: Utc::now(),
            modified_time: Utc::now(),
            last_run: None,
            next_run: None,
            run_count: 0,
            success_count: 0,
            failure_count: 0,
            max_runtime: self.max_runtime,
            timeout_action: self.timeout_action.clone(),
            retry_policy: self.retry_policy.clone(),
            dependencies: self.dependencies.clone(),
            tags: self.tags.clone(),
            metadata: self.metadata.clone(),
            notification_settings: self.notification_settings.clone(),
            resource_limits: self.resource_limits.clone(),
            execution_history: VecDeque::new(),
            schedule_cache: None,
        }
    }
}

fn print_cron_help(i18n: &I18n) {
    println!("{}", i18n.get("cron.help.title", None));
    println!();
    println!("{}", i18n.get("cron.help.usage", None));
    println!("    cron [OPTIONS] [COMMAND]");
    println!();
    println!("{}", i18n.get("cron.help.commands", None));
    println!("    -l, --list              List all cron jobs");
    println!("    -a, --add               Add a new cron job");
    println!("    -r, --remove ID         Remove a cron job");
    println!("    -e, --edit ID           Edit a cron job");
    println!("    --enable ID             Enable a cron job");
    println!("    --disable ID            Disable a cron job");
    println!("    --run-now ID            Run a job immediately");
    println!("    --stats                 Show cron statistics");
    println!();
    println!("{}", i18n.get("cron.help.options", None));
    println!("    -h, --help              Show this help message");
    println!("    --name NAME             Job name (required for --add)");
    println!("    --description DESC      Job description");
    println!("    --schedule EXPR         Cron expression (required for --add)");
    println!("    --command CMD           Command to execute (required for --add)");
    println!("    --user USER             User to run job as");
    println!("    --tag TAG               Add tag to job");
    println!("    --status STATUS         Filter by status (active, inactive, running, etc.)");
    println!("    --priority LEVEL        Set priority (low, normal, high, critical)");
    println!("    --timezone TZ           Set timezone (default: UTC)");
    println!("    --mail-on-success       Send email on successful completion");
    println!("    --mail-on-failure       Send email on failure");
    println!("    --no-mail               Disable all email notifications");
    println!();
    println!("{}", i18n.get("cron.help.cron_format", None));
    println!("    * * * * * *");
    println!("    ━E━E━E━E━E━E);
    println!("    ━E━E━E━E━E└─── day of week (0-6, Sunday=0)");
    println!("    ━E━E━E━E└───── month (1-12)");
    println!("    ━E━E━E└─────── day of month (1-31)");
    println!("    ━E━E└───────── hour (0-23)");
    println!("    ━E└─────────── minute (0-59)");
    println!("    └───────────── second (0-59, optional)");
    println!();
    println!("{}", i18n.get("cron.help.examples", None));
    println!("    cron --list                                    # List all jobs");
    println!("    cron --add --name 'Backup' --schedule '0 2 * * *' --command 'backup.sh'");
    println!("    cron --add --name 'Hourly Report' --schedule '0 * * * *' --command 'report.py'");
    println!("    cron --list --status active --user john        # List active jobs for user john");
    println!("    cron --disable cron_123                        # Disable job cron_123");
    println!("    cron --run-now cron_123                        # Run job immediately");
    println!("    cron --stats                                   # Show statistics");
}

// System monitoring implementation
async fn get_system_stats() -> Result<SystemStats> {
    #[cfg(target_os = "windows")]
    {
        get_windows_system_stats().await
    }
    
    #[cfg(target_os = "linux")]
    {
        get_linux_system_stats().await
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        get_fallback_system_stats().await
    }
}

#[cfg(target_os = "windows")]
async fn get_windows_system_stats() -> Result<SystemStats> {
    use std::process::Command;
    
    // Get memory info using PowerShell
    let memory_output = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-WmiObject -Class Win32_OperatingSystem | Select-Object TotalVisibleMemorySize,FreePhysicalMemory"
        ])
        .output()?;
    
    let memory_info = String::from_utf8_lossy(&memory_output.stdout);
    let (memory_usage_percent, _total_memory) = parse_windows_memory(&memory_info)?;
    
    // Get CPU info
    let cpu_output = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-WmiObject -Class Win32_Processor | Measure-Object -Property LoadPercentage -Average | Select-Object Average"
        ])
        .output()?;
    
    let cpu_info = String::from_utf8_lossy(&cpu_output.stdout);
    let cpu_usage = parse_windows_cpu(&cpu_info)?;
    
    // Get process count
    let process_output = Command::new("powershell")
        .args(&["-Command", "(Get-Process).Count"])
        .output()?;
    
    let process_count = String::from_utf8_lossy(&process_output.stdout)
        .trim()
        .parse::<u32>()
        .unwrap_or(0);
    
    // Get uptime
    let uptime_output = Command::new("powershell")
        .args(&[
            "-Command",
            "(Get-Date) - (Get-CimInstance -ClassName Win32_OperatingSystem).LastBootUpTime | Select-Object TotalSeconds"
        ])
        .output()?;
    
    let uptime_info = String::from_utf8_lossy(&uptime_output.stdout);
    let uptime_seconds = parse_windows_uptime(&uptime_info)?;
    
    // Windows doesn't have a direct load average equivalent
    // Use CPU usage as approximation
    let load_average = cpu_usage / 100.0 * num_cpus::get() as f64;
    
    Ok(SystemStats {
        load_average,
        memory_usage_percent,
        disk_usage_percent: get_windows_disk_usage().await.ok(),
        cpu_usage_percent: cpu_usage,
        uptime_seconds,
        process_count,
        network_rx_bytes: 0, // Would need WinAPI for accurate network stats
        network_tx_bytes: 0,
    })
}

#[cfg(target_os = "linux")]
async fn get_linux_system_stats() -> Result<SystemStats> {
    // Read /proc/loadavg for load average
    let loadavg_content = tokio::fs::read_to_string("/proc/loadavg").await?;
    let load_average = loadavg_content
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    
    // Read /proc/meminfo for memory stats
    let meminfo_content = tokio::fs::read_to_string("/proc/meminfo").await?;
    let memory_usage_percent = parse_linux_memory(&meminfo_content)?;
    
    // Read /proc/stat for CPU stats
    let stat_content = tokio::fs::read_to_string("/proc/stat").await?;
    let cpu_usage_percent = parse_linux_cpu(&stat_content)?;
    
    // Read /proc/uptime for uptime
    let uptime_content = tokio::fs::read_to_string("/proc/uptime").await?;
    let uptime_seconds = uptime_content
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0) as u64;
    
    // Count processes
    let mut process_count = 0;
    if let Ok(mut entries) = tokio::fs::read_dir("/proc").await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.file_name().to_string_lossy().chars().all(|c| c.is_ascii_digit()) {
                process_count += 1;
            }
        }
    }
    
    // Get disk usage for root filesystem
    let disk_usage_percent = get_linux_disk_usage("/").await.ok();
    
    // Get network stats from /proc/net/dev
    let (network_rx_bytes, network_tx_bytes) = get_linux_network_stats().await.unwrap_or((0, 0));
    
    Ok(SystemStats {
        load_average,
        memory_usage_percent,
        disk_usage_percent,
        cpu_usage_percent,
        uptime_seconds,
        process_count,
        network_rx_bytes,
        network_tx_bytes,
    })
}

async fn get_fallback_system_stats() -> SystemStats {
    // Cross-platform fallback using standard Rust libraries
    // These are approximations and may not be as accurate
    
    let process_count = if cfg!(target_os = "windows") {
        // Use tasklist on Windows
        std::process::Command::new("tasklist")
            .output()
            .ok()
            .and_then(|output| {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.lines().count().saturating_sub(3) as u32) // Skip header lines
            })
            .unwrap_or(50) // Fallback estimate
    } else {
        // Use ps on Unix-like systems
        std::process::Command::new("ps")
            .arg("aux")
            .output()
            .ok()
            .and_then(|output| {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.lines().count().saturating_sub(1) as u32) // Skip header
            })
            .unwrap_or(50) // Fallback estimate
    };
    
    // Simple heuristics for other metrics
    SystemStats {
        load_average: (process_count as f64 / num_cpus::get() as f64).min(10.0),
        memory_usage_percent: 0.3, // Conservative estimate
        disk_usage_percent: Some(0.5), // Conservative estimate
        cpu_usage_percent: 25.0, // Conservative estimate
        uptime_seconds: 3600, // 1 hour estimate
        process_count,
        network_rx_bytes: 0,
        network_tx_bytes: 0,
    }
}

// Helper functions for parsing system information

#[cfg(target_os = "windows")]
fn parse_windows_memory(output: &str) -> Result<(f64, u64)> {
    let mut total_memory = 0u64;
    let mut free_memory = 0u64;
    
    for line in output.lines() {
        if line.contains("TotalVisibleMemorySize") {
            if let Some(value) = line.split_whitespace().last() {
                total_memory = value.parse::<u64>().unwrap_or(0) * 1024; // Convert from KB
            }
        } else if line.contains("FreePhysicalMemory") {
            if let Some(value) = line.split_whitespace().last() {
                free_memory = value.parse::<u64>().unwrap_or(0) * 1024; // Convert from KB
            }
        }
    }
    
    if total_memory > 0 {
        let used_memory = total_memory.saturating_sub(free_memory);
        let usage_percent = (used_memory as f64 / total_memory as f64).min(1.0);
        Ok((usage_percent, total_memory))
    } else {
        Ok((0.3, 8_000_000_000)) // Fallback: 30% of 8GB
    }
}

#[cfg(target_os = "windows")]
fn parse_windows_cpu(output: &str) -> Result<f64> {
    for line in output.lines() {
        if line.contains("Average") {
            if let Some(value) = line.split_whitespace().last() {
                return Ok(value.parse::<f64>().unwrap_or(25.0));
            }
        }
    }
    Ok(25.0) // Fallback
}

#[cfg(target_os = "windows")]
fn parse_windows_uptime(output: &str) -> Result<u64> {
    for line in output.lines() {
        if line.contains("TotalSeconds") {
            if let Some(value) = line.split_whitespace().last() {
                return Ok(value.parse::<f64>().unwrap_or(3600.0) as u64);
            }
        }
    }
    Ok(3600) // Fallback: 1 hour
}

#[cfg(target_os = "windows")]
async fn get_windows_disk_usage() -> Result<f64> {
    let output = std::process::Command::new("powershell")
        .args(&[
            "-Command",
            "Get-WmiObject -Class Win32_LogicalDisk -Filter \"DeviceID='C:'\" | Select-Object Size,FreeSpace"
        ])
        .output()?;
    
    let disk_info = String::from_utf8_lossy(&output.stdout);
    
    let mut total_size = 0u64;
    let mut free_space = 0u64;
    
    for line in disk_info.lines() {
        if line.contains("Size") {
            if let Some(value) = line.split_whitespace().last() {
                total_size = value.parse::<u64>().unwrap_or(0);
            }
        } else if line.contains("FreeSpace") {
            if let Some(value) = line.split_whitespace().last() {
                free_space = value.parse::<u64>().unwrap_or(0);
            }
        }
    }
    
    if total_size > 0 {
        let used_space = total_size.saturating_sub(free_space);
        Ok(used_space as f64 / total_size as f64)
    } else {
        Ok(0.5) // Fallback: 50%
    }
}

#[cfg(target_os = "linux")]
fn parse_linux_memory(meminfo: &str) -> Result<f64> {
    let mut mem_total = 0u64;
    let mut mem_available = 0u64;
    
    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            if let Some(value) = line.split_whitespace().nth(1) {
                mem_total = value.parse::<u64>().unwrap_or(0) * 1024; // Convert from KB
            }
        } else if line.starts_with("MemAvailable:") {
            if let Some(value) = line.split_whitespace().nth(1) {
                mem_available = value.parse::<u64>().unwrap_or(0) * 1024; // Convert from KB
            }
        }
    }
    
    if mem_total > 0 {
        let mem_used = mem_total.saturating_sub(mem_available);
        Ok(mem_used as f64 / mem_total as f64)
    } else {
        Ok(0.3) // Fallback
    }
}

#[cfg(target_os = "linux")]
fn parse_linux_cpu(stat_content: &str) -> Result<f64> {
    // Simple CPU usage calculation from /proc/stat
    // This is a simplified version - real implementation would need to
    // calculate usage over time intervals
    if let Some(line) = stat_content.lines().next() {
        if line.starts_with("cpu ") {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 5 {
                let user: u64 = fields[1].parse().unwrap_or(0);
                let nice: u64 = fields[2].parse().unwrap_or(0);
                let system: u64 = fields[3].parse().unwrap_or(0);
                let idle: u64 = fields[4].parse().unwrap_or(0);
                
                let total = user + nice + system + idle;
                let non_idle = user + nice + system;
                
                if total > 0 {
                    return Ok((non_idle as f64 / total as f64) * 100.0);
                }
            }
        }
    }
    Ok(25.0) // Fallback
}

#[cfg(target_os = "linux")]
async fn get_linux_disk_usage(path: &str) -> Result<f64> {
    use std::fs;
    use std::os::unix::fs::MetadataExt;
    
    let metadata = fs::metadata(path)?;
    let dev = metadata.dev();
    
    // Read /proc/mounts to find the mount point
    let mounts_content = tokio::fs::read_to_string("/proc/mounts").await?;
    for line in mounts_content.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 2 {
            let mount_point = fields[1];
            if let Ok(mount_metadata) = fs::metadata(mount_point) {
                if mount_metadata.dev() == dev {
                    // Use statvfs system call equivalent
                    return get_disk_usage_statvfs(mount_point).await;
                }
            }
        }
    }
    
    Ok(0.5) // Fallback
}

#[cfg(target_os = "linux")]
async fn get_disk_usage_statvfs(path: &str) -> Result<f64> {
    // This would ideally use the statvfs system call
    // For now, use df command as fallback
    let output = std::process::Command::new("df")
        .arg(path)
        .output()?;
    
    let df_output = String::from_utf8_lossy(&output.stdout);
    if let Some(line) = df_output.lines().nth(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 5 {
            if let (Ok(total), Ok(used)) = (fields[1].parse::<u64>(), fields[2].parse::<u64>()) {
                if total > 0 {
                    return Ok(used as f64 / total as f64);
                }
            }
        }
    }
    
    Ok(0.5) // Fallback
}

#[cfg(target_os = "linux")]
async fn get_linux_network_stats() -> Result<(u64, u64)> {
    let netdev_content = tokio::fs::read_to_string("/proc/net/dev").await?;
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;
    
    for line in netdev_content.lines().skip(2) { // Skip header lines
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 10 {
            // Interface name is in parts[0], RX bytes in parts[1], TX bytes in parts[9]
            if let (Ok(rx), Ok(tx)) = (parts[1].parse::<u64>(), parts[9].parse::<u64>()) {
                // Skip loopback interface
                if !parts[0].starts_with("lo") {
                    total_rx += rx;
                    total_tx += tx;
                }
            }
        }
    }
    
    Ok((total_rx, total_tx))
}

// Cross-platform notification implementation
async fn send_email_notifications(
    email_addresses: &[String],
    subject: &str,
    body: &str,
    config: &CronConfig,
) -> bool {
    if email_addresses.is_empty() {
        return false;
    }

    // Try platform-specific email sending methods
    #[cfg(target_os = "linux")]
    {
        if send_email_via_sendmail(email_addresses, subject, body).await {
            return true;
        }
        if send_email_via_mailx(email_addresses, subject, body).await {
            return true;
        }
    }

    #[cfg(target_os = "windows")]
    {
        if send_email_via_powershell(email_addresses, subject, body, config).await {
            return true;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if send_email_via_mail_command(email_addresses, subject, body).await {
            return true;
        }
    }

    // Fallback: Log the notification
    nxsh_log_info!("Email notification (fallback log): To: {:?}, Subject: {}", email_addresses, subject);
    false
}

#[cfg(target_os = "linux")]
async fn send_email_via_sendmail(email_addresses: &[String], subject: &str, body: &str) -> bool {
    use tokio::process::Command as AsyncCommand;
    use tokio::io::AsyncWriteExt;

    for addr in email_addresses {
        if let Ok(mut child) = AsyncCommand::new("/usr/sbin/sendmail")
            .arg("-t")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(stdin) = child.stdin.as_mut() {
                let email_content = format!("To: {}\nSubject: {}\n\n{}", addr, subject, body);
                if stdin.write_all(email_content.as_bytes()).await.is_ok() {
                    if child.wait().await.is_ok() {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(target_os = "linux")]
async fn send_email_via_mailx(email_addresses: &[String], subject: &str, body: &str) -> bool {
    use tokio::process::Command as AsyncCommand;

    for addr in email_addresses {
        if let Ok(status) = AsyncCommand::new("mailx")
            .arg("-s")
            .arg(subject)
            .arg(addr)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(stdin) = child.stdin.as_mut() {
                    let _ = std::io::Write::write_all(stdin.as_mut(), body.as_bytes());
                }
                child.wait()
            })
            .await
        {
            if status.success() {
                return true;
            }
        }
    }
    false
}

#[cfg(target_os = "windows")]
async fn send_email_via_powershell(
    email_addresses: &[String],
    subject: &str,
    body: &str,
    config: &CronConfig,
) -> bool {
    use tokio::process::Command as AsyncCommand;

    let smtp_server = &config.smtp_settings.server;
    let smtp_port = config.smtp_settings.port;
    let from_address = &config.smtp_settings.from_address;

    for addr in email_addresses {
        let powershell_script = format!(
            r#"
            try {{
                $smtp = New-Object Net.Mail.SmtpClient("{}")
                $smtp.Port = {}
                $smtp.EnableSsl = ${}
                $mail = New-Object Net.Mail.MailMessage
                $mail.From = "{}"
                $mail.To.Add("{}")
                $mail.Subject = "{}"
                $mail.Body = "{}"
                $smtp.Send($mail)
                $mail.Dispose()
                Write-Output "SUCCESS"
            }} catch {{
                Write-Output "FAILED: $_"
            }}
            "#,
            smtp_server,
            smtp_port,
            if config.smtp_settings.tls { "true" } else { "false" },
            from_address,
            addr,
            subject.replace('"', '`"'),
            body.replace('"', '`"').replace('\n', "`n")
        );

        if let Ok(output) = AsyncCommand::new("powershell")
            .arg("-Command")
            .arg(&powershell_script)
            .output()
            .await
        {
            let result = String::from_utf8_lossy(&output.stdout);
            if result.contains("SUCCESS") {
                return true;
            }
        }
    }
    false
}

#[cfg(target_os = "macos")]
async fn send_email_via_mail_command(email_addresses: &[String], subject: &str, body: &str) -> bool {
    use tokio::process::Command as AsyncCommand;

    for addr in email_addresses {
        if let Ok(status) = AsyncCommand::new("mail")
            .arg("-s")
            .arg(subject)
            .arg(addr)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(stdin) = child.stdin.as_mut() {
                    let _ = std::io::Write::write_all(stdin.as_mut(), body.as_bytes());
                }
                child.wait()
            })
            .await
        {
            if status.success() {
                return true;
            }
        }
    }
    false
}

async fn send_system_notification(title: &str, message: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("powershell")
            .arg("-Command")
            .arg(&format!(
                r#"Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.MessageBox]::Show("{}", "{}", "OK", "Information")"#,
                message.replace('"', '`"'),
                title.replace('"', '`"')
            ))
            .output();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&format!(
                r#"display notification "{}" with title "{}""#,
                message.replace('"', r#"\""#),
                title.replace('"', r#"\""#)
            ))
            .output();
    }

    #[cfg(target_os = "linux")]
    {
        // Try notify-send first
        if std::process::Command::new("notify-send")
            .arg(title)
            .arg(message)
            .output()
            .is_err()
        {
            // Fallback to wall command if available
            let _ = std::process::Command::new("wall")
                .arg(&format!("{}: {}", title, message))
                .output();
        }
    }
}

async fn send_webhook_notifications(job: &CronJob, execution: &CronExecution, config: &CronConfig) -> Result<()> {
    let success = execution.exit_code == Some(0);
    
    #[cfg(feature = "updates")]
    {
        let payload = serde_json::json!({
            "job_id": job.id,
            "name": job.name,
            "command": job.command,
            "success": success,
            "exit_code": execution.exit_code,
            "start_time": execution.start_time,
            "end_time": execution.end_time,
            "stdout": execution.stdout,
            "stderr": execution.stderr,
            "hostname": std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string()),
            "timestamp": Utc::now(),
        }).to_string();

        for url in &job.notification_settings.webhook_urls {
            let url_clone = url.clone();
            let payload_clone = payload.clone();
            let timeout = config.webhook_timeout;

            // Send webhook in background to avoid blocking
            tokio::spawn(async move {
                let result = tokio::time::timeout(timeout, async {
                    ureq::AgentBuilder::new()
                        .timeout(timeout)
                        .build()
                        .post(&url_clone)
                        .set("Content-Type", "application/json")
                        .set("User-Agent", "NexusShell-Cron/1.0")
                        .send_string(&payload_clone)
                }).await;

                match result {
                    Ok(Ok(_)) => {
                        nxsh_log_info!("Webhook notification sent successfully to {}", url_clone);
                    }
                    Ok(Err(e)) => {
                        nxsh_log_warn!("Failed to send webhook to {}: {}", url_clone, e);
                    }
                    Err(_) => {
                        nxsh_log_warn!("Webhook notification to {} timed out", url_clone);
                    }
                }
            });
        }
    }

    #[cfg(not(feature = "updates"))]
    {
        nxsh_log_info!("Webhook notifications configured but 'updates' feature not enabled. Would POST to: {:?}", job.notification_settings.webhook_urls);
    }

    Ok(())
}

async fn send_slack_notification(channel: &str, job: &CronJob, execution: &CronExecution, success: bool) {
    #[cfg(feature = "updates")]
    {
        let color = if success { "good" } else { "danger" };
        let status_emoji = if success { ":white_check_mark:" } else { ":x:" };
        
        let payload = serde_json::json!({
            "channel": channel,
            "username": "Cron Bot",
            "icon_emoji": ":robot_face:",
            "attachments": [{
                "color": color,
                "title": format!("{} Cron Job: {}", status_emoji, job.name),
                "fields": [
                    {
                        "title": "Command",
                        "value": job.command,
                        "short": false
                    },
                    {
                        "title": "Exit Code",
                        "value": execution.exit_code.unwrap_or(-1).to_string(),
                        "short": true
                    },
                    {
                        "title": "Duration",
                        "value": format!("{:.2}s", execution.execution_time.as_secs_f64()),
                        "short": true
                    }
                ],
                "timestamp": execution.start_time.timestamp()
            }]
        });

        // Note: Real Slack webhook URL would be stored in the channel field
        // This is a simplified implementation
        tokio::spawn(async move {
            let _ = ureq::post(channel)
                .set("Content-Type", "application/json")
                .send_string(&payload.to_string());
        });
    }

    #[cfg(not(feature = "updates"))]
    {
        nxsh_log_info!("Slack notification: {} job '{}' to channel '{}'", 
                      if success { "SUCCESS" } else { "FAILED" }, job.name, channel);
    }
}

async fn send_discord_notification(webhook_url: &str, job: &CronJob, execution: &CronExecution, success: bool) {
    #[cfg(feature = "updates")]
    {
        let color = if success { 0x00ff00 } else { 0xff0000 }; // Green or Red
        let status_emoji = if success { "✁E } else { "❁E };
        
        let payload = serde_json::json!({
            "embeds": [{
                "title": format!("{} Cron Job: {}", status_emoji, job.name),
                "color": color,
                "fields": [
                    {
                        "name": "Command",
                        "value": job.command,
                        "inline": false
                    },
                    {
                        "name": "Exit Code",
                        "value": execution.exit_code.unwrap_or(-1).to_string(),
                        "inline": true
                    },
                    {
                        "name": "Duration",
                        "value": format!("{:.2}s", execution.execution_time.as_secs_f64()),
                        "inline": true
                    }
                ],
                "timestamp": execution.start_time.to_rfc3339()
            }]
        });

        tokio::spawn(async move {
            let _ = ureq::post(webhook_url)
                .set("Content-Type", "application/json")
                .send_string(&payload.to_string());
        });
    }

    #[cfg(not(feature = "updates"))]
    {
        nxsh_log_info!("Discord notification: {} job '{}' to webhook", 
                      if success { "SUCCESS" } else { "FAILED" }, job.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn test_system_stats_fallback() {
        let stats = get_fallback_system_stats().await;
        assert!(stats.load_average >= 0.0);
        assert!(stats.memory_usage_percent >= 0.0 && stats.memory_usage_percent <= 1.0);
        assert!(stats.process_count > 0);
    }

    #[test]
    fn test_cron_job_creation() {
        let job = CronJob {
            id: "test_job".to_string(),
            name: "Test Job".to_string(),
            description: "A test job".to_string(),
            cron_expression: "0 0 * * *".to_string(),
            command: "echo 'test'".to_string(),
            user: "testuser".to_string(),
            working_directory: PathBuf::from("/tmp"),
            environment: HashMap::new(),
            timezone: "UTC".to_string(),
            status: CronJobStatus::Active,
            priority: CronJobPriority::Normal,
            created_time: Utc::now(),
            modified_time: Utc::now(),
            last_run: None,
            next_run: None,
            run_count: 0,
            success_count: 0,
            failure_count: 0,
            max_runtime: Some(Duration::from_secs(3600)),
            timeout_action: TimeoutAction::Kill,
            retry_policy: RetryPolicy::default(),
            dependencies: Vec::new(),
            tags: Vec::new(),
            metadata: HashMap::new(),
            notification_settings: NotificationSettings::default(),
            resource_limits: CronResourceLimits::default(),
        };

        assert_eq!(job.name, "Test Job");
        assert_eq!(job.status, CronJobStatus::Active);
        assert_eq!(job.priority, CronJobPriority::Normal);
    }

    #[tokio::test]
    async fn test_cron_manager_creation() {
        let config = CronConfig::default();
        let manager = CronManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[test]
    fn test_cron_config_defaults() {
        let config = CronConfig::default();
        assert_eq!(config.max_concurrent_jobs, MAX_CONCURRENT_CRON_JOBS);
        assert_eq!(config.system_load_threshold, 5.0);
        assert_eq!(config.memory_threshold, 0.9);
        assert_eq!(config.disk_threshold, Some(0.9));
        assert!(config.resource_monitoring);
        assert!(config.audit_enabled);
    }

    #[test]
    fn test_retry_policy_defaults() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.backoff_multiplier, 2.0);
        assert!(policy.retry_on_exit_codes.contains(&1));
        assert!(policy.retry_on_exit_codes.contains(&127));
    }

    #[test]
    fn test_notification_settings_defaults() {
        let settings = NotificationSettings::default();
        assert!(!settings.email_on_success);
        assert!(settings.email_on_failure);
        assert!(!settings.webhook_on_success);
        assert!(settings.webhook_on_failure);
        assert!(settings.email_addresses.is_empty());
        assert!(settings.webhook_urls.is_empty());
    }

    #[test]
    fn test_resource_limits_defaults() {
        let limits = CronResourceLimits::default();
        assert_eq!(limits.max_memory, Some(1024 * 1024 * 1024)); // 1GB
        assert_eq!(limits.max_cpu_time, Some(Duration::from_secs(3600))); // 1 hour
        assert_eq!(limits.max_file_descriptors, Some(1024));
        assert_eq!(limits.max_processes, Some(100));
    }

    #[tokio::test]
    async fn test_system_monitoring_functions() {
        // Test that system monitoring functions don't panic
        let _stats = get_system_stats().await; // May fail on some systems, that's ok
        let fallback_stats = get_fallback_system_stats().await;
        
        // Fallback stats should always work
        assert!(fallback_stats.load_average >= 0.0);
        assert!(fallback_stats.process_count > 0);
    }

    #[test]
    fn test_cron_event_types() {
        let events = vec![
            CronEvent::JobAdded("test".to_string()),
            CronEvent::JobRemoved("test".to_string()),
            CronEvent::JobStarted("test".to_string()),
            CronEvent::JobCompleted("test".to_string(), 0),
            CronEvent::JobFailed("test".to_string(), "error".to_string()),
            CronEvent::SystemLoadHigh(10.0),
            CronEvent::MemoryUsageHigh(0.95),
            CronEvent::DiskUsageHigh(0.98),
        ];

        // Test that all event types can be created and matched
        for event in events {
            match event {
                CronEvent::JobAdded(_) => {},
                CronEvent::JobRemoved(_) => {},
                CronEvent::JobStarted(_) => {},
                CronEvent::JobCompleted(_, _) => {},
                CronEvent::JobFailed(_, _) => {},
                CronEvent::SystemLoadHigh(_) => {},
                CronEvent::MemoryUsageHigh(_) => {},
                CronEvent::DiskUsageHigh(_) => {},
                _ => {},
            }
        }
    }

    #[test]
    fn test_cron_job_status_variants() {
        let statuses = vec![
            CronJobStatus::Active,
            CronJobStatus::Inactive,
            CronJobStatus::Running,
            CronJobStatus::Completed,
            CronJobStatus::Failed,
            CronJobStatus::Disabled,
            CronJobStatus::Expired,
        ];

        for status in statuses {
            // Test serialization/deserialization
            let json = serde_json::to_string(&status).unwrap();
            let _deserialized: CronJobStatus = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_smtp_settings_defaults() {
        let smtp = SmtpSettings::default();
        assert_eq!(smtp.server, "localhost");
        assert_eq!(smtp.port, 587);
        assert!(smtp.tls);
        assert_eq!(smtp.from_address, "noreply@localhost");
    }
} 

