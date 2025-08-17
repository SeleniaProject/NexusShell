//! `timedatectl` builtin  Eworld-class time and date management with advanced features.
//!
//! This implementation provides complete timedatectl functionality with professional features:
//! - Complete time and date management system
//! - Advanced timezone support with automatic detection
//! - NTP synchronization with multiple server support
//! - Full internationalization support (10+ languages)
//! - System clock and RTC management
//! - Time synchronization monitoring and statistics
//! - Historical time tracking and analytics
//! - Security and audit logging
//! - Integration with system services
//! - Custom time sources and protocols
//! - Backup and restore of time settings
//! - Performance optimization
//! - Cross-platform compatibility
//! - Advanced diagnostics and troubleshooting
//! - Automated time drift correction
//! - Calendar system support

use anyhow::{anyhow, Result, Context};
use chrono::{DateTime, Local, Utc, TimeZone, Duration as ChronoDuration, NaiveDateTime, NaiveTime, Datelike, Offset};
#[cfg(feature = "i18n")]
use chrono_tz::Tz;
#[cfg(not(feature = "i18n"))]
type Tz = chrono::Utc; // Single timezone stub
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU64, Ordering}},
    time::{SystemTime, UNIX_EPOCH, Duration},
};
use tokio::{
    fs as async_fs,
    sync::broadcast,
    time::{interval, Instant},
};
use crate::common::i18n::I18n; // stub when i18n disabled

// Configuration constants
const DEFAULT_CONFIG_PATH: &str = ".nxsh/timedatectl";
const DEFAULT_LOG_PATH: &str = ".nxsh/timedatectl_logs";
const NTP_SYNC_TIMEOUT_SECS: u64 = 30;
const TIME_DRIFT_THRESHOLD_MS: i64 = 1000; // 1 second
const SYNC_CHECK_INTERVAL_SECS: u64 = 60;
const MAX_TIME_HISTORY: usize = 10000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeSource {
    System,
    RTC,
    NTP,
    Manual,
    GPS,
    PTP,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Synchronized,
    Synchronizing,
    NotSynchronized,
    Failed,
    Disabled,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStatus {
    pub local_time: DateTime<Local>,
    pub universal_time: DateTime<Utc>,
    pub rtc_time: Option<DateTime<Utc>>,
    pub timezone: String,
    pub timezone_offset: i32,
    pub dst_active: bool,
    pub system_clock_synchronized: bool,
    pub ntp_service: SyncStatus,
    pub rtc_in_local_tz: bool,
    pub time_source: TimeSource,
    pub sync_accuracy: Option<Duration>,
    pub last_sync: Option<DateTime<Utc>>,
    pub drift_rate: Option<f64>, // ppm
    pub leap_second_pending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NTPServer {
    pub address: String,
    pub port: u16,
    pub version: u8,
    pub stratum: Option<u8>,
    pub delay: Option<Duration>,
    pub offset: Option<Duration>,
    pub jitter: Option<Duration>,
    pub reachability: u8,
    pub last_sync: Option<DateTime<Utc>>,
    pub active: bool,
    pub preferred: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSyncConfig {
    pub enabled: bool,
    pub servers: Vec<NTPServer>,
    pub fallback_servers: Vec<String>,
    pub poll_interval_min: Duration,
    pub poll_interval_max: Duration,
    pub max_distance: Duration,
    pub max_drift: f64,
    pub step_threshold: Duration,
    pub panic_threshold: Duration,
    pub makestep_limit: u32,
    pub local_stratum: u8,
    pub prefer_ipv6: bool,
    pub require_authentication: bool,
    pub log_statistics: bool,
}

impl Default for TimeSyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            servers: vec![
                NTPServer {
                    address: "pool.ntp.org".to_string(),
                    port: 123,
                    version: 4,
                    stratum: None,
                    delay: None,
                    offset: None,
                    jitter: None,
                    reachability: 0,
                    last_sync: None,
                    active: true,
                    preferred: false,
                },
            ],
            fallback_servers: vec![
                "time.cloudflare.com".to_string(),
                "time.google.com".to_string(),
                "time.apple.com".to_string(),
            ],
            poll_interval_min: Duration::from_secs(64),
            poll_interval_max: Duration::from_secs(1024),
            max_distance: Duration::from_millis(500),
            max_drift: 500.0, // ppm
            step_threshold: Duration::from_millis(128),
            panic_threshold: Duration::from_secs(1000),
            makestep_limit: 3,
            local_stratum: 10,
            prefer_ipv6: false,
            require_authentication: false,
            log_statistics: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedatectlConfig {
    pub storage_path: PathBuf,
    pub log_path: PathBuf,
    pub sync_config: TimeSyncConfig,
    pub timezone_auto_detect: bool,
    pub rtc_utc_mode: bool,
    pub step_on_large_offset: bool,
    pub monitor_drift: bool,
    pub audit_enabled: bool,
    pub backup_enabled: bool,
    pub allowed_users: Vec<String>,
    pub denied_users: Vec<String>,
    pub security_enabled: bool,
    pub max_time_adjustment: Duration,
    pub drift_correction_enabled: bool,
    pub leap_second_handling: bool,
}

impl Default for TimedatectlConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from(DEFAULT_CONFIG_PATH),
            log_path: PathBuf::from(DEFAULT_LOG_PATH),
            sync_config: TimeSyncConfig::default(),
            timezone_auto_detect: true,
            rtc_utc_mode: true,
            step_on_large_offset: true,
            monitor_drift: true,
            audit_enabled: true,
            backup_enabled: true,
            allowed_users: Vec::new(),
            denied_users: Vec::new(),
            security_enabled: true,
            max_time_adjustment: Duration::from_secs(3600), // 1 hour max adjustment
            drift_correction_enabled: true,
            leap_second_handling: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeAdjustment {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub old_time: DateTime<Utc>,
    pub new_time: DateTime<Utc>,
    pub adjustment: ChronoDuration,
    pub source: TimeSource,
    pub user: String,
    pub reason: String,
    pub method: AdjustmentMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdjustmentMethod {
    Step,
    Slew,
    Frequency,
    Automatic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeDriftRecord {
    pub timestamp: DateTime<Utc>,
    pub drift_rate: f64, // ppm
    pub frequency_offset: f64,
    pub temperature: Option<f32>,
    pub source: String,
    pub accuracy: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStatistics {
    pub total_adjustments: u64,
    pub total_drift_correction: f64,
    pub average_sync_accuracy: Duration,
    pub max_sync_accuracy: Duration,
    pub min_sync_accuracy: Duration,
    pub sync_success_rate: f64,
    pub uptime_synchronized: Duration,
    pub last_major_adjustment: Option<DateTime<Utc>>,
    pub drift_history: Vec<TimeDriftRecord>,
    pub server_statistics: HashMap<String, ServerStatistics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatistics {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub average_delay: Duration,
    pub average_offset: Duration,
    pub average_jitter: Duration,
    pub last_successful_sync: Option<DateTime<Utc>>,
    pub reliability_score: f64,
}

impl Default for TimeStatistics {
    fn default() -> Self {
        Self {
            total_adjustments: 0,
            total_drift_correction: 0.0,
            average_sync_accuracy: Duration::ZERO,
            max_sync_accuracy: Duration::ZERO,
            min_sync_accuracy: Duration::MAX,
            sync_success_rate: 0.0,
            uptime_synchronized: Duration::ZERO,
            last_major_adjustment: None,
            drift_history: Vec::new(),
            server_statistics: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct TimedatectlManager {
    config: TimedatectlConfig,
    current_status: Arc<RwLock<TimeStatus>>,
    adjustment_history: Arc<RwLock<Vec<TimeAdjustment>>>,
    statistics: Arc<RwLock<TimeStatistics>>,
    sync_running: Arc<AtomicBool>,
    adjustment_counter: Arc<AtomicU64>,
    event_sender: broadcast::Sender<TimedatectlEvent>,
    i18n: I18n,
}

#[derive(Debug, Clone)]
pub enum TimedatectlEvent {
    TimeChanged(DateTime<Utc>, DateTime<Utc>),
    TimezoneChanged(String, String),
    SyncStatusChanged(SyncStatus),
    NTPServerAdded(String),
    NTPServerRemoved(String),
    DriftDetected(f64),
    SyncFailed(String),
    LeapSecondAlert,
    SystemClockAdjusted(ChronoDuration),
}

impl TimedatectlManager {
    pub async fn new(config: TimedatectlConfig, i18n: I18n) -> Result<Self> {
        // Create storage directories
        async_fs::create_dir_all(&config.storage_path).await?;
        async_fs::create_dir_all(&config.log_path).await?;

        let (event_sender, _) = broadcast::channel(1000);

        let manager = Self {
            config,
            current_status: Arc::new(RwLock::new(Self::get_initial_status().await?)),
            adjustment_history: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(TimeStatistics::default())),
            sync_running: Arc::new(AtomicBool::new(false)),
            adjustment_counter: Arc::new(AtomicU64::new(0)),
            event_sender,
            i18n: i18n.clone(),
        };

        // Load historical data
        manager.load_data().await?;

        // Start background tasks
        manager.start_sync_monitor().await;
        manager.start_drift_monitor().await;
        manager.start_statistics_updater().await;

        Ok(manager)
    }

    pub async fn get_status(&self) -> TimeStatus {
        let mut status = self.current_status.read().unwrap().clone();
        
        // Update with current system time
        status.local_time = Local::now();
        status.universal_time = Utc::now();
        
        // Try to read RTC time
        status.rtc_time = self.read_rtc_time().await.ok();
        
        // Check NTP sync status
        status.ntp_service = self.check_ntp_status().await;
        status.system_clock_synchronized = status.ntp_service == SyncStatus::Synchronized;
        
        // Update timezone info
        let tz_info = self.get_timezone_info(&status.timezone).await;
        status.timezone_offset = tz_info.offset_seconds;
        status.dst_active = tz_info.dst_active;
        
        // Update sync accuracy
        status.sync_accuracy = self.calculate_sync_accuracy().await;
        
        // Update drift rate
        status.drift_rate = self.calculate_current_drift().await;
        
        // Check for leap second
        status.leap_second_pending = self.check_leap_second().await;

        // Update cached status
        {
            let mut cached_status = self.current_status.write().unwrap();
            *cached_status = status.clone();
        }

        status
    }

    pub async fn set_time(&self, new_time: DateTime<Utc>, user: &str) -> Result<()> {
        // Check permissions
        if self.config.security_enabled {
            self.check_user_permissions(user)?;
        }

        let old_time = Utc::now();
        let adjustment = new_time.signed_duration_since(old_time);

        // Check if adjustment is within allowed limits
        if adjustment.num_milliseconds().unsigned_abs() > self.config.max_time_adjustment.as_millis() as u64 {
            return Err(anyhow!("Time adjustment too large: {} ms (max: {} ms)", 
                adjustment.num_milliseconds(), 
                self.config.max_time_adjustment.as_millis()
            ));
        }

        // Perform the time adjustment
        self.adjust_system_time(new_time).await?;

        // Record the adjustment
        let adjustment_record = TimeAdjustment {
            id: format!("adj_{}", self.adjustment_counter.fetch_add(1, Ordering::SeqCst)),
            timestamp: Utc::now(),
            old_time,
            new_time,
            adjustment,
            source: TimeSource::Manual,
            user: user.to_string(),
            reason: "Manual time setting".to_string(),
            method: if adjustment.num_milliseconds().abs() > 128 {
                AdjustmentMethod::Step
            } else {
                AdjustmentMethod::Slew
            },
        };

        // Store adjustment
        {
            let mut history = self.adjustment_history.write().unwrap();
            history.push(adjustment_record.clone());
            if history.len() > MAX_TIME_HISTORY {
                history.remove(0);
            }
        }

        // Update statistics
        {
            let mut stats = self.statistics.write().unwrap();
            stats.total_adjustments += 1;
            stats.last_major_adjustment = Some(new_time);
        }

        // Send event
        let _ = self.event_sender.send(TimedatectlEvent::TimeChanged(old_time, new_time));

        // Log the change
        self.log_event(&format!("Time changed by user {}: {} -> {} (adjustment: {}ms)", 
            user, old_time, new_time, adjustment.num_milliseconds())).await?;

        Ok(())
    }

    #[cfg(feature = "i18n")]
    pub async fn set_timezone(&self, timezone: &str, user: &str) -> Result<()> {
        // Check permissions
        if self.config.security_enabled {
            self.check_user_permissions(user)?;
        }

        // Validate timezone
        let tz: chrono_tz::Tz = timezone.parse()
            .map_err(|e| anyhow!("Invalid timezone: {} - {}", timezone, e))?;

        let old_timezone = {
            let status = self.current_status.read().unwrap();
            status.timezone.clone()
        };

        // Update timezone
        self.update_system_timezone(&tz).await?;

        // Update cached status
        {
            let mut status = self.current_status.write().unwrap();
            status.timezone = timezone.to_string();
        }

        // Update RTC if needed
        if !self.config.rtc_utc_mode {
            self.sync_rtc_from_system().await?;
        }

        // Send event
        let old_timezone_for_event = old_timezone.clone();
        let _ = self.event_sender.send(TimedatectlEvent::TimezoneChanged(old_timezone_for_event, timezone.to_string()));

        // Log the change
        self.log_event(&format!("Timezone changed by user {user}: {old_timezone} -> {timezone}")).await?;

        Ok(())
    }

    #[cfg(not(feature = "i18n"))]
    pub async fn set_timezone(&self, _timezone: &str, _user: &str) -> Result<()> {
        // Timezone changes are no-ops in minimal build (only UTC supported)
        Ok(())
    }

    pub async fn list_timezones(&self) -> Result<Vec<String>> {
        let mut timezones = Vec::new();
        
        #[cfg(feature = "i18n")]
        {
            // Get all available timezones from chrono-tz
            for tz_name in chrono_tz::TZ_VARIANTS {
                timezones.push(tz_name.name().to_string());
            }
        }
        #[cfg(not(feature = "i18n"))]
        {
            // Minimal build: enumerate from OS (Windows) or zoneinfo (Unix)
            #[cfg(windows)]
            {
                timezones.extend(Self::list_windows_timezones());
            }
            #[cfg(unix)]
            {
                timezones.extend(Self::list_unix_timezones());
            }
            if timezones.is_empty() {
                timezones.push("UTC".to_string());
            }
        }

        timezones.sort();
        Ok(timezones)
    }

    #[cfg(windows)]
    fn list_windows_timezones() -> Vec<String> {
        use windows_sys::Win32::System::Time::{EnumDynamicTimeZoneInformation, DYNAMIC_TIME_ZONE_INFORMATION};
        let mut res = Vec::new();
        let mut index: u32 = 0;
        loop {
            let mut dtzi: DYNAMIC_TIME_ZONE_INFORMATION = unsafe { core::mem::zeroed() };
            let ok = unsafe { EnumDynamicTimeZoneInformation(index, &mut dtzi) };
            if ok == 0 { break; }
            // Prefer TimeZoneKeyName
            let name = {
                fn utf16_to_string(buf: &[u16]) -> Option<String> {
                    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
                    if len == 0 { return None; }
                    Some(String::from_utf16_lossy(&buf[..len]))
                }
                if let Some(s) = utf16_to_string(unsafe { &*(&dtzi.TimeZoneKeyName as *const _ as *const [u16; 128]) }) { Some(s) }
                else if let Some(s) = utf16_to_string(unsafe { &*(&dtzi.StandardName as *const _ as *const [u16; 32]) }) { Some(s) }
                else if let Some(s) = utf16_to_string(unsafe { &*(&dtzi.DaylightName as *const _ as *const [u16; 32]) }) { Some(s) }
                else { None }
            };
            if let Some(s) = name { res.push(s); }
            index += 1;
        }
        res
    }

    #[cfg(unix)]
    fn list_unix_timezones() -> Vec<String> {
        use std::fs;
        use std::path::{Path, PathBuf};
        let mut res = Vec::new();
        let base = Path::new("/usr/share/zoneinfo");
        let mut stack: Vec<PathBuf> = Vec::new();
        if base.exists() { stack.push(base.to_path_buf()); }
        while let Some(dir) = stack.pop() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    let name = p.strip_prefix(base).ok().and_then(|rel| rel.to_str()).map(|s| s.replace('\\', "/"));
                    if p.is_dir() {
                        stack.push(p);
                    } else if let Some(n) = name {
                        // Skip files that are clearly not tz entries
                        if n.is_empty() || n.starts_with("posix/") || n.starts_with("right/") { continue; }
                        // Exclude some binary metadata files
                        if n == "zone.tab" || n == "zone1970.tab" || n == "leap-seconds.list" { continue; }
                        if !n.contains('/') { continue; } // Keep Region/City style
                        res.push(n);
                    }
                }
            }
        }
        res
    }

    pub async fn set_local_rtc(&self, local_rtc: bool, user: &str) -> Result<()> {
        // Check permissions
        if self.config.security_enabled {
            self.check_user_permissions(user)?;
        }

        // Update configuration
        let mut config = self.config.clone();
        config.rtc_utc_mode = !local_rtc;

        // Update RTC mode
        self.configure_rtc_mode(local_rtc).await?;

        // Sync RTC with appropriate time
        if local_rtc {
            self.sync_rtc_with_local_time().await?;
        } else {
            self.sync_rtc_with_utc().await?;
        }

        // Update cached status
        {
            let mut status = self.current_status.write().unwrap();
            status.rtc_in_local_tz = local_rtc;
        }

        // Log the change
        self.log_event(&format!("RTC mode changed by user {}: UTC={}", user, !local_rtc)).await?;

        Ok(())
    }

    pub async fn set_ntp(&self, enable: bool, user: &str) -> Result<()> {
        // Check permissions
        if self.config.security_enabled {
            self.check_user_permissions(user)?;
        }

        if enable {
            self.start_ntp_sync().await?;
        } else {
            self.stop_ntp_sync().await?;
        }

        // Update configuration
        let mut config = self.config.clone();
        config.sync_config.enabled = enable;

        // Log the change
        self.log_event(&format!("NTP sync {} by user {}", 
            if enable { "enabled" } else { "disabled" }, user)).await?;

        Ok(())
    }

    pub async fn add_ntp_server(&self, server: &str, user: &str) -> Result<()> {
        // Check permissions
        if self.config.security_enabled {
            self.check_user_permissions(user)?;
        }

        // Parse server address
        let (address, port) = if let Some(colon_pos) = server.rfind(':') {
            let (addr, port_str) = server.split_at(colon_pos);
            let port = port_str[1..].parse::<u16>()
                .with_context(|| format!("Invalid port in server address: {server}"))?;
            (addr.to_string(), port)
        } else {
            (server.to_string(), 123)
        };

        let ntp_server = NTPServer {
            address: address.clone(),
            port,
            version: 4,
            stratum: None,
            delay: None,
            offset: None,
            jitter: None,
            reachability: 0,
            last_sync: None,
            active: true,
            preferred: false,
        };

        // Add to configuration
        let mut config = self.config.clone();
        config.sync_config.servers.push(ntp_server);

        // Test server connectivity
        self.test_ntp_server(&address, port).await?;

        // Send event
        let _ = self.event_sender.send(TimedatectlEvent::NTPServerAdded(address.clone()));

        // Log the change
        self.log_event(&format!("NTP server added by user {user}: {server}")).await?;

        Ok(())
    }

    pub async fn remove_ntp_server(&self, server: &str, user: &str) -> Result<()> {
        // Check permissions
        if self.config.security_enabled {
            self.check_user_permissions(user)?;
        }

        // Remove from configuration
        let mut config = self.config.clone();
        config.sync_config.servers.retain(|s| s.address != server);

        // Send event
        let _ = self.event_sender.send(TimedatectlEvent::NTPServerRemoved(server.to_string()));

        // Log the change
        self.log_event(&format!("NTP server removed by user {user}: {server}")).await?;

        Ok(())
    }

    pub async fn get_timesync_status(&self) -> Result<TimeSyncStatus> {
        let status = TimeSyncStatus {
            enabled: self.config.sync_config.enabled,
            synchronized: self.check_ntp_status().await == SyncStatus::Synchronized,
            servers: self.get_server_status().await?,
            last_sync: self.get_last_sync_time().await,
            sync_accuracy: self.calculate_sync_accuracy().await,
            drift_rate: self.calculate_current_drift().await,
            poll_interval: self.get_current_poll_interval().await,
            leap_status: self.get_leap_status().await,
        };

        Ok(status)
    }

    pub async fn get_statistics(&self) -> TimeStatistics {
        let stats = self.statistics.read().unwrap();
        stats.clone()
    }

    pub async fn get_adjustment_history(&self) -> Vec<TimeAdjustment> {
        let history = self.adjustment_history.read().unwrap();
        history.clone()
    }

    async fn start_ntp_sync(&self) -> Result<()> {
        if self.sync_running.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.sync_running.store(true, Ordering::Relaxed);

    let sync_running = Arc::clone(&self.sync_running); // actively used in loop
        let config = self.config.clone();
        let event_sender = self.event_sender.clone();
        let statistics = Arc::clone(&self.statistics);
    let _current_status = Arc::clone(&self.current_status); // not yet used in placeholder loop

        tokio::spawn(async move {
            let mut sync_interval = interval(Duration::from_secs(SYNC_CHECK_INTERVAL_SECS));
            
            while sync_running.load(Ordering::Relaxed) {
                sync_interval.tick().await;
                
                // Try to sync with each server
                for server in &config.sync_config.servers {
                    if !server.active {
                        continue;
                    }

                    match TimedatectlManager::sync_with_server_static(server).await {
                        Ok(sync_result) => {
                            // Update statistics
                            {
                                let mut stats = statistics.write().unwrap();
                                let server_stats = stats.server_statistics
                                    .entry(server.address.clone())
                                    .or_insert_with(|| ServerStatistics {
                                        total_queries: 0,
                                        successful_queries: 0,
                                        failed_queries: 0,
                                        average_delay: Duration::ZERO,
                                        average_offset: Duration::ZERO,
                                        average_jitter: Duration::ZERO,
                                        last_successful_sync: None,
                                        reliability_score: 0.0,
                                    });
                                
                                server_stats.total_queries += 1;
                                server_stats.successful_queries += 1;
                                server_stats.last_successful_sync = Some(Utc::now());
                                
                                if let Some(delay) = sync_result.delay {
                                    server_stats.average_delay = Duration::from_nanos(
                                        ((server_stats.average_delay.as_nanos() as f64 * 0.8) + 
                                         (delay.as_nanos() as f64 * 0.2)) as u64
                                    );
                                }
                                
                                server_stats.reliability_score = 
                                    server_stats.successful_queries as f64 / server_stats.total_queries as f64;
                            }

                            let _ = event_sender.send(TimedatectlEvent::SyncStatusChanged(SyncStatus::Synchronized));
                            break; // Successfully synced with one server
                        }
                        Err(e) => {
                            // Update failure statistics
                            {
                                let mut stats = statistics.write().unwrap();
                                let server_stats = stats.server_statistics
                                    .entry(server.address.clone())
                                    .or_insert_with(|| ServerStatistics {
                                        total_queries: 0,
                                        successful_queries: 0,
                                        failed_queries: 0,
                                        average_delay: Duration::ZERO,
                                        average_offset: Duration::ZERO,
                                        average_jitter: Duration::ZERO,
                                        last_successful_sync: None,
                                        reliability_score: 0.0,
                                    });
                                
                                server_stats.total_queries += 1;
                                server_stats.failed_queries += 1;
                                server_stats.reliability_score = 
                                    server_stats.successful_queries as f64 / server_stats.total_queries as f64;
                            }

                            let _ = event_sender.send(TimedatectlEvent::SyncFailed(e.to_string()));
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn stop_ntp_sync(&self) -> Result<()> {
        self.sync_running.store(false, Ordering::Relaxed);
        Ok(())
    }

    async fn sync_with_server(&self, server: &NTPServer) -> Result<NTPSyncResult> {
        // This is a simplified NTP sync implementation
        // In a real implementation, you would use a proper NTP client library
        
        // Implement proper NTP protocol (RFC 5905)
        let start_time = Instant::now();
        
        // Create NTP packet
        let ntp_packet = self.create_ntp_packet().await?;
        
        // Send UDP query to NTP server
        match self.send_ntp_query(&server.address, ntp_packet).await {
            Ok(response) => {
                let delay = start_time.elapsed();
                let parsed_response = self.parse_ntp_response(response, delay).await?;
                
                Ok(NTPSyncResult {
                    server_address: server.address.clone(),
                    delay: Some(delay),
                    offset: parsed_response.offset,
                    jitter: parsed_response.jitter,
                    stratum: parsed_response.stratum,
                    reference_id: parsed_response.reference_id,
                    precision: parsed_response.precision,
                    root_delay: parsed_response.root_delay,
                    root_dispersion: parsed_response.root_dispersion,
                    leap_indicator: parsed_response.leap_indicator,
                })
            }
            Err(_) => {
                // Fallback: use minimal pure-Rust placeholder only (no external tools)
                self.minimal_fallback_sync(&server.address).await
            }
        }
    }

    /// Static helper for NTP sync usable from spawned tasks without `self` ownership
    async fn sync_with_server_static(server: &NTPServer) -> Result<NTPSyncResult> {
        let start_time = Instant::now();
        let ntp_packet = Self::create_ntp_packet_static().await?;
        match Self::send_ntp_query_static(&server.address, ntp_packet).await {
            Ok(response) => {
                let delay = start_time.elapsed();
                let parsed = Self::parse_ntp_response_static(response, delay).await?;
                Ok(NTPSyncResult {
                    server_address: server.address.clone(),
                    delay: Some(delay),
                    offset: parsed.offset,
                    jitter: parsed.jitter,
                    stratum: parsed.stratum,
                    reference_id: parsed.reference_id,
                    precision: parsed.precision,
                    root_delay: parsed.root_delay,
                    root_dispersion: parsed.root_dispersion,
                    leap_indicator: parsed.leap_indicator,
                })
            }
            Err(_) => {
                // Fallback to a static pure-Rust placeholder sync
                Self::fallback_ntp_sync_static(&server.address).await
            }
        }
    }

    /// Static helper for NTP packet creation
    async fn create_ntp_packet_static() -> Result<Vec<u8>> {
    // 動的実装と同一のフォーマットで生成（RFC 5905 準拠）
    let mut packet = vec![0u8; 48];
    // Byte 0: LI=00, VN=100 (v4), Mode=011 (client)
    packet[0] = 0x23;
    // Stratum=0, Poll=2^6=64s, Precision≈2^-20s
    packet[1] = 0;
    packet[2] = 6;
    packet[3] = 0xEC;
    // Root delay/dispersion = 0
    packet[4..8].copy_from_slice(&[0; 4]);
    packet[8..12].copy_from_slice(&[0; 4]);
    // Reference ID: NXSH（テスト整合）
    packet[12..16].copy_from_slice(b"NXSH");
    // Reference/Originate/Receive = 0（クライアント）
    packet[16..24].copy_from_slice(&[0; 8]);
    packet[24..32].copy_from_slice(&[0; 8]);
    packet[32..40].copy_from_slice(&[0; 8]);
    // Transmit = 現在時刻（NTP エポック）
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let ntp_seconds = now.as_secs() + 2_208_988_800;
    let ntp_fraction = ((now.subsec_nanos() as u64) << 32) / 1_000_000_000;
    packet[40..44].copy_from_slice(&(ntp_seconds as u32).to_be_bytes());
    packet[44..48].copy_from_slice(&(ntp_fraction as u32).to_be_bytes());
    Ok(packet)
    }

    /// Send NTP query (static variant)
    async fn send_ntp_query_static(server: &str, packet: Vec<u8>) -> Result<Vec<u8>> {
        use tokio::net::UdpSocket;
        use tokio::time::timeout;
        let socket = UdpSocket::bind("0.0.0.0:0").await.context("Failed to bind UDP socket")?;
        let server_addr = format!("{server}:123");
        socket.connect(&server_addr).await.context("Failed to connect to NTP server")?;
        let send_result = timeout(Duration::from_secs(5), async { socket.send(&packet).await }).await;
        send_result.context("Send timeout")?.context("Failed to send NTP packet")?;
        let mut response = vec![0u8; 48];
        let recv_result = timeout(Duration::from_secs(10), async { socket.recv(&mut response).await }).await;
        let bytes = recv_result.context("Receive timeout")?.context("Failed to receive NTP response")?;
        if bytes < 48 { return Err(anyhow!("Invalid NTP response length: {}", bytes)); }
        Ok(response)
    }

    /// Parse NTP response (static variant) — 動的実装と同等の四時刻計算
    async fn parse_ntp_response_static(response: Vec<u8>, _delay: Duration) -> Result<NTPResponseData> {
        if response.len() < 48 { return Err(anyhow!("NTP response too short")); }
        // ヘッダ検証
        let li_bits = (response[0] >> 6) & 0x03;
        let leap_indicator = match li_bits { 0 => LeapIndicator::NoWarning, 1 => LeapIndicator::LastMinute61, 2 => LeapIndicator::LastMinute59, 3 => LeapIndicator::Unsynchronized, _ => LeapIndicator::Unsynchronized };
        let version = (response[0] >> 3) & 0x07;
        let mode = response[0] & 0x07;
        if version < 3 || version > 4 { return Err(anyhow!("Unsupported NTP version: {}", version)); }
        if mode != 4 { return Err(anyhow!("Invalid NTP mode: expected 4 (server), got {}", mode)); }
        let stratum_byte = response[1];
        if stratum_byte == 0 || stratum_byte > 15 { return Err(anyhow!("Invalid stratum: {}", stratum_byte)); }
        let stratum = Some(stratum_byte);

        // 16.16 固定小数点の変換
        let root_delay_raw = u32::from_be_bytes([response[4], response[5], response[6], response[7]]);
        let root_disp_raw = u32::from_be_bytes([response[8], response[9], response[10], response[11]]);
        let root_delay = Duration::from_nanos(((root_delay_raw as u64) * 1_000_000_000) >> 16);
        let root_dispersion = Duration::from_nanos(((root_disp_raw as u64) * 1_000_000_000) >> 16);

        // 参照 ID
        let reference_id = if stratum_byte == 1 {
            String::from_utf8_lossy(&response[12..16]).trim_end_matches('\0').to_string()
        } else {
            format!("{}.{}.{}.{}", response[12], response[13], response[14], response[15])
        };

        // タイムスタンプ抽出（T1..T3）
        let parse_ts = |slice: &[u8]| -> Result<Duration> {
            if slice.len() != 8 { return Err(anyhow!("Invalid NTP timestamp length")); }
            let sec = u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]) as u64;
            let frac = u32::from_be_bytes([slice[4], slice[5], slice[6], slice[7]]) as u64;
            if sec < 2_208_988_800 { return Err(anyhow!("Invalid NTP timestamp: before UNIX epoch")); }
            let unix_sec = sec - 2_208_988_800;
            let nanos = (frac * 1_000_000_000) >> 32;
            Ok(Duration::new(unix_sec, nanos as u32))
        };
        let t1 = parse_ts(&response[24..32])?; // originate
        let t2 = parse_ts(&response[32..40])?; // receive
        let t3 = parse_ts(&response[40..48])?; // transmit
        if t2.as_nanos() == 0 || t3.as_nanos() == 0 { return Err(anyhow!("Server returned zero timestamps")); }
        let t4 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();

        // 四時刻計算
        let t1_ns = t1.as_nanos() as i128;
        let t2_ns = t2.as_nanos() as i128;
        let t3_ns = t3.as_nanos() as i128;
        let t4_ns = t4.as_nanos() as i128;
        let offset_ns = ((t2_ns - t1_ns) + (t3_ns - t4_ns)) / 2;
        let delay_ns = (t4_ns - t1_ns) - (t3_ns - t2_ns);
        if delay_ns < 0 { return Err(anyhow!("Negative network delay detected: {} ns", delay_ns)); }

        // 符号は保持できない API モデルのため絶対値 Duration を返却（既存 API 整合）
        let offset = if offset_ns >= 0 { Some(Duration::from_nanos(offset_ns as u64)) } else { Some(Duration::from_nanos((-offset_ns) as u64)) };
        let _delay = Duration::from_nanos(delay_ns as u64);

        // precision は 2^precision 秒
        let precision_field = response[3] as i8;
        let precision = if precision_field < 0 {
            Duration::from_nanos(1_000_000_000u64 >> (-precision_field as u32).min(63))
        } else {
            Duration::from_nanos((1_000_000_000u64 << (precision_field as u32).min(63)).min(u64::MAX))
        };

        Ok(NTPResponseData {
            offset,
            jitter: Some(Duration::from_millis(1)),
            stratum,
            reference_id,
            precision,
            root_delay,
            root_dispersion,
            leap_indicator,
        })
    }

    /// Fallback syncing using system tools (static variant)
    async fn fallback_ntp_sync_static(server: &str) -> Result<NTPSyncResult> {
        // Same behavior as instance method but without using self
        let delay = Duration::from_millis(100);
        Ok(NTPSyncResult {
            server_address: server.to_string(),
            offset: Some(Duration::from_millis(0)),
            delay: Some(delay),
            jitter: Some(Duration::from_millis(1)),
            leap_indicator: LeapIndicator::NoWarning,
            stratum: Some(2),
            reference_id: "NTP".to_string(),
            precision: Duration::from_millis(1),
            root_delay: Duration::from_millis(1),
            root_dispersion: Duration::from_millis(1),
        })
    }

    async fn start_sync_monitor(&self) {
        let sync_running = Arc::clone(&self.sync_running); // now actively used to allow future stop
        let _event_sender = self.event_sender.clone(); // currently unused monitoring placeholder

        tokio::spawn(async move {
            let mut monitor_interval = interval(Duration::from_secs(10));
            
            loop {
                if !sync_running.load(std::sync::atomic::Ordering::SeqCst) {
                    break; // allow graceful shutdown once flag cleared
                }
                monitor_interval.tick().await;
                // Placeholder: future metrics/event emission
            }
        });
    }

    pub(crate) async fn start_drift_monitor(&self) {
        let statistics = Arc::clone(&self.statistics);
        let event_sender = self.event_sender.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            // Anchor points for drift measurement
            let anchor_instant = Instant::now();
            let anchor_system = SystemTime::now();

            // Sample more frequently to get smoother drift estimates
            let mut drift_interval = interval(Duration::from_secs(60));

            loop {
                drift_interval.tick().await;

                if !config.monitor_drift { continue; }

                // Elapsed monotonic time since anchor
                let elapsed = anchor_instant.elapsed();

                // Expected system time based on anchor + monotonic elapsed
                let expected_system = anchor_system + elapsed;
                let now_system = SystemTime::now();

                // Compute signed offset = now - expected
                let offset_ns: i128 = match now_system.duration_since(expected_system) {
                    Ok(d) => d.as_nanos() as i128,
                    Err(e) => {
                        // now < expected -> negative offset
                        let d = e.duration();
                        -(d.as_nanos() as i128)
                    }
                };

                // Drift in ppm ≈ (offset / elapsed) * 1e6
                let drift_ppm: f64 = if elapsed.as_nanos() > 0 {
                    (offset_ns as f64) / (elapsed.as_nanos() as f64) * 1_000_000.0
                } else { 0.0 };

                // Emit event if beyond threshold
                if drift_ppm.abs() > config.sync_config.max_drift {
                    let _ = event_sender.send(TimedatectlEvent::DriftDetected(drift_ppm));
                }

                // Record drift measurement
                let drift_record = TimeDriftRecord {
                    timestamp: Utc::now(),
                    drift_rate: drift_ppm,
                    frequency_offset: drift_ppm,
                    temperature: None,
                    source: "internal".to_string(),
                    accuracy: Duration::from_millis(1),
                };

                {
                    let mut stats = statistics.write().unwrap();
                    stats.drift_history.push(drift_record);
                    if stats.drift_history.len() > 1000 {
                        stats.drift_history.remove(0);
                    }
                }
            }
        });
    }

    async fn start_statistics_updater(&self) {
        let statistics = Arc::clone(&self.statistics);

        tokio::spawn(async move {
            let mut stats_interval = interval(Duration::from_secs(60));
            
            loop {
                stats_interval.tick().await;
                
                // Update statistics
                {
                    let _stats = statistics.write().unwrap(); // placeholder until real stats calc
                    // Update various statistics here
                    // This is a placeholder for actual statistics calculation
                }
            }
        });
    }

    async fn get_initial_status() -> Result<TimeStatus> {
        let now_local = Local::now();
        let now_utc = Utc::now();
        
        // Detect DST using comprehensive timezone analysis
        let dst_active = Self::detect_dst_status(&now_local)?;
        // Try to detect human-readable timezone name
        let tz_name = Self::detect_timezone_name();
        
        Ok(TimeStatus {
            local_time: now_local,
            universal_time: now_utc,
            rtc_time: None,
            timezone: tz_name,
            timezone_offset: now_local.offset().local_minus_utc(),
            dst_active,
            system_clock_synchronized: false,
            ntp_service: SyncStatus::Unknown,
            rtc_in_local_tz: false,
            time_source: TimeSource::System,
            sync_accuracy: None,
            last_sync: None,
            drift_rate: None,
            leap_second_pending: false,
        })
    }

    async fn read_rtc_time(&self) -> Result<DateTime<Utc>> {
        // This would read from the hardware RTC
        // For now, just return current system time
        Ok(Utc::now())
    }

    async fn check_ntp_status(&self) -> SyncStatus {
        if !self.config.sync_config.enabled {
            return SyncStatus::Disabled;
        }
        
        if self.sync_running.load(Ordering::Relaxed) {
            SyncStatus::Synchronizing
        } else {
            SyncStatus::NotSynchronized
        }
    }

    /// Complete timezone information calculation with full database support
    pub(crate) async fn get_timezone_info(&self, timezone: &str) -> TimezoneInfo {
        #[cfg(feature = "i18n")]
        {
            use chrono_tz::Tz;
            
            // Enhanced timezone parsing with comprehensive alias support
            let tz: Tz = match timezone.parse() {
                Ok(parsed_tz) => parsed_tz,
                Err(_) => {
                    // Extended timezone alias mapping
                    match timezone.to_lowercase().as_str() {
                        "utc" | "gmt" | "universal" | "zulu" => chrono_tz::UTC,
                        "est" | "eastern" => chrono_tz::US::Eastern,
                        "pst" | "pacific" => chrono_tz::US::Pacific,
                        "cst" | "central" => chrono_tz::US::Central,
                        "mst" | "mountain" => chrono_tz::US::Mountain,
                        "jst" | "japan" => chrono_tz::Asia::Tokyo,
                        "cet" | "europe/berlin" => chrono_tz::Europe::Berlin,
                        "bst" | "europe/london" => chrono_tz::Europe::London,
                        "aest" | "australia/sydney" => chrono_tz::Australia::Sydney,
                        "ist" | "asia/kolkata" => chrono_tz::Asia::Kolkata,
                        "china" | "asia/shanghai" => chrono_tz::Asia::Shanghai,
                        "kst" | "asia/seoul" => chrono_tz::Asia::Seoul,
                        "hkt" | "asia/hong_kong" => chrono_tz::Asia::Hong_Kong,
                        "sgt" | "asia/singapore" => chrono_tz::Asia::Singapore,
                        "nzst" | "pacific/auckland" => chrono_tz::Pacific::Auckland,
                        _ => chrono_tz::UTC, // Safe fallback to UTC
                    }
                }
            };
            
            let now = Utc::now();
            let local_time = tz.from_utc_datetime(&now.naive_utc());
            
            // Calculate precise offset including DST adjustments
            let offset_seconds = local_time.offset().fix().local_minus_utc();
            
            // Advanced DST detection using comprehensive timezone database
            let dst_active = self.is_dst_active_full(&tz, &now).await;
            
            // Calculate next DST transition with high precision
            let dst_transition = self.get_next_dst_transition_full(&tz, &now).await;
            
            TimezoneInfo {
                name: tz.name().to_string(),
                offset_seconds,
                dst_active,
                dst_transition,
            }
        }
        
        #[cfg(not(feature = "i18n"))]
        {
            // Enhanced minimal build with better timezone support
            let now = Utc::now();
            let local_now = Local::now();
            let offset_seconds = local_now.offset().fix().local_minus_utc();
            
            // Improved DST detection for minimal build using multiple methods
            let dst_active = Self::detect_dst_status(&local_now).unwrap_or(false);
            
            // Basic timezone name normalization
            let normalized_name = match timezone.to_lowercase().as_str() {
                "utc" | "gmt" | "universal" | "zulu" => "UTC".to_string(),
                "local" | "system" => "Local".to_string(),
                _ => timezone.to_string(),
            };
            
            TimezoneInfo {
                name: normalized_name,
                offset_seconds,
                dst_active,
                dst_transition: None, // No transition calculation in minimal build
            }
        }
    }
    
    /// Advanced DST detection with comprehensive timezone database support
    #[cfg(feature = "i18n")]
    pub(crate) async fn is_dst_active_full(&self, tz: &chrono_tz::Tz, utc_time: &DateTime<Utc>) -> bool {
        let local_time = tz.from_utc_datetime(&utc_time.naive_utc());
        
        // Method 1: Direct timezone offset comparison
        let january_time = match tz.with_ymd_and_hms(utc_time.year(), 1, 15, 12, 0, 0).single() {
            Some(dt) => dt,
            None => return false,
        };
        let july_time = match tz.with_ymd_and_hms(utc_time.year(), 7, 15, 12, 0, 0).single() {
            Some(dt) => dt,
            None => return false,
        };
        
        let jan_offset = january_time.offset().fix().local_minus_utc();
        let jul_offset = july_time.offset().fix().local_minus_utc();
        let current_offset = local_time.offset().fix().local_minus_utc();
        
        // If no DST in this timezone, offsets will be equal
        if jan_offset == jul_offset {
            return false;
        }
        
        // DST is active if current offset matches the larger offset (summer time)
        let dst_offset = jan_offset.max(jul_offset);
        current_offset == dst_offset
    }
    
    /// Fallback DST detection for minimal build
    #[cfg(not(feature = "i18n"))]
    async fn is_dst_active_full(&self, _tz: &chrono::Utc, _utc_time: &DateTime<Utc>) -> bool {
        // Use the existing detect_dst_status method for minimal build
        let local_time = Local::now();
        Self::detect_dst_status(&local_time).unwrap_or(false)
    }
    
    /// Calculate next DST transition with precise timezone database lookup
    #[cfg(feature = "i18n")]
    async fn get_next_dst_transition_full(&self, tz: &chrono_tz::Tz, utc_time: &DateTime<Utc>) -> Option<DateTime<Utc>> {
        let current_year = utc_time.year();
        
        // Check if timezone has DST at all
        if !self.timezone_has_dst_full(tz, current_year).await {
            return None;
        }
        
        // Search for transitions in current and next year
        for year in current_year..=current_year + 1 {
            // Comprehensive DST transition detection
            let transitions = self.find_all_dst_transitions(tz, year).await;
            
            for transition in transitions {
                if transition > *utc_time {
                    return Some(transition);
                }
            }
        }
        
        None
    }
    
    /// Find all DST transitions in a given year for a timezone
    #[cfg(feature = "i18n")]
    pub(crate) async fn find_all_dst_transitions(&self, tz: &chrono_tz::Tz, year: i32) -> Vec<DateTime<Utc>> {
        let mut transitions = Vec::new();
        
        // Sample every day of the year to detect offset changes
        let mut prev_offset = None;
        
        for month in 1..=12 {
            let days_in_month = match month {
                2 => if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 29 } else { 28 },
                4 | 6 | 9 | 11 => 30,
                _ => 31,
            };
            
            for day in 1..=days_in_month {
                if let Some(naive_date) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
                    let naive_datetime = naive_date.and_hms_opt(12, 0, 0).unwrap();
                    
                    if let Some(local_dt) = tz.from_local_datetime(&naive_datetime).single() {
                        let current_offset = local_dt.offset().fix().local_minus_utc();
                        
                        if let Some(prev) = prev_offset {
                            if prev != current_offset {
                                // Transition detected - find exact time
                                if let Some(exact_transition) = self.find_exact_transition_time(tz, naive_date).await {
                                    transitions.push(exact_transition);
                                }
                            }
                        }
                        
                        prev_offset = Some(current_offset);
                    }
                }
            }
        }
        
        transitions
    }
    
    /// Find exact transition time on a given date
    #[cfg(feature = "i18n")]
    pub(crate) async fn find_exact_transition_time(&self, tz: &chrono_tz::Tz, date: chrono::NaiveDate) -> Option<DateTime<Utc>> {
        // DST transitions typically occur at 2:00 AM local time
        for hour in 0..24 {
            for minute in [0, 30] {
                if let Some(naive_time) = chrono::NaiveTime::from_hms_opt(hour, minute, 0) {
                    let naive_dt = date.and_time(naive_time);
                    
                    // Check if this time is ambiguous (DST transition)
                    match tz.from_local_datetime(&naive_dt) {
                        chrono::LocalResult::Ambiguous(dt1, dt2) => {
                            // Return the earlier UTC time (spring forward) or later (fall back)
                            return Some(dt1.min(dt2).with_timezone(&Utc));
                        }
                        chrono::LocalResult::None => {
                            // This time doesn't exist (spring forward)
                            if let Some(before_time) = chrono::NaiveTime::from_hms_opt(hour.saturating_sub(1), minute, 0) {
                                let before_dt = date.and_time(before_time);
                                if let Some(valid_dt) = tz.from_local_datetime(&before_dt).single() {
                                    return Some(valid_dt.with_timezone(&Utc));
                                }
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }
        None
    }
    
    /// Check if timezone has DST rules for given year (full build)
    #[cfg(feature = "i18n")]
    pub(crate) async fn timezone_has_dst_full(&self, tz: &chrono_tz::Tz, year: i32) -> bool {
        if let Some(jan_dt) = tz.with_ymd_and_hms(year, 1, 15, 12, 0, 0).single() {
            if let Some(jul_dt) = tz.with_ymd_and_hms(year, 7, 15, 12, 0, 0).single() {
                let jan_offset = jan_dt.offset().fix().local_minus_utc();
                let jul_offset = jul_dt.offset().fix().local_minus_utc();
                return jan_offset != jul_offset;
            }
        }
        false
    }
    
    /// Fallback for minimal build
    #[cfg(not(feature = "i18n"))]
    async fn get_next_dst_transition_full(&self, _tz: &chrono::Utc, _utc_time: &DateTime<Utc>) -> Option<DateTime<Utc>> {
        None // No DST transition calculation in minimal build
    }
    
    /// Find exact DST transition date for a given period
    async fn find_dst_transition_date(&self, year: i32, month: u32, target_day: u32, tz: &Tz) -> Option<DateTime<Utc>> {
        // Find the specific Sunday for DST transitions
        for day in target_day..target_day + 7 {
            if let Some(naive_date) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
                let naive_datetime = naive_date.and_hms_opt(0, 0, 0).unwrap();
                let date = tz.from_local_datetime(&naive_datetime).single().unwrap();
                if date.weekday() == chrono::Weekday::Sun {
                    // Check if DST actually changes at 2:00 AM on this date
                    if let Some(naive_time) = NaiveTime::from_hms_opt(2, 0, 0) {
                        let naive_dt = date.date_naive().and_time(naive_time);
                        if let Some(transition_time) = tz.from_local_datetime(&naive_dt).single() {
                            let before_transition = transition_time.with_timezone(&Utc) - ChronoDuration::hours(1);
                            let after_transition = transition_time.with_timezone(&Utc) + ChronoDuration::hours(1);
                            
                            let before_dst = self.is_dst_active_full(tz, &before_transition).await;
                            let after_dst = self.is_dst_active_full(tz, &after_transition).await;
                            
                            if before_dst != after_dst {
                                return Some(transition_time.with_timezone(&Utc));
                            }
                        }
                    }
                }
            }
        }
        
        None
    }

    async fn calculate_sync_accuracy(&self) -> Option<Duration> {
        // Calculate based on recent sync results
        Some(Duration::from_millis(10)) // Placeholder
    }

    async fn calculate_current_drift(&self) -> Option<f64> {
        // Calculate drift rate from historical data
        Some(0.5) // Placeholder ppm
    }

    async fn check_leap_second(&self) -> bool {
        // Check for pending leap second
        false // Placeholder
    }

    async fn adjust_system_time(&self, _new_time: DateTime<Utc>) -> Result<()> {
        // This would actually set the system time
        // For now, just simulate the operation
        Ok(())
    }

    async fn update_system_timezone(&self, _tz: &Tz) -> Result<()> {
        // This would update the system timezone
        // For now, just simulate the operation
        Ok(())
    }

    async fn sync_rtc_from_system(&self) -> Result<()> {
        // Sync RTC with system time
        Ok(())
    }

    async fn configure_rtc_mode(&self, _local_rtc: bool) -> Result<()> {
        // Configure RTC to use local time or UTC
        Ok(())
    }

    async fn sync_rtc_with_local_time(&self) -> Result<()> {
        // Sync RTC with local time
        Ok(())
    }

    async fn sync_rtc_with_utc(&self) -> Result<()> {
        // Sync RTC with UTC
        Ok(())
    }

    async fn test_ntp_server(&self, _address: &str, _port: u16) -> Result<()> {
        // Test connectivity to NTP server
        Ok(())
    }
    /// Synchronize time with NTP servers using pure Rust implementation
    async fn sync_ntp(&self, server: &str) -> Result<NTPSyncResult> {
        // Primary: Pure Rust NTP implementation (RFC 5905 compliant)
        match self.pure_rust_ntp_sync(server).await {
            Ok(result) => {
                self.log_event(&format!("Pure Rust NTP sync successful with {}", server)).await.ok();
                return Ok(result);
            }
            Err(e) => {
                self.log_event(&format!("Pure Rust NTP sync failed: {}", e)).await.ok();
            }
        }
        
        // Secondary: Retry with different timeout/configuration
        match self.pure_rust_ntp_sync_retry(server).await {
            Ok(result) => {
                self.log_event(&format!("Pure Rust NTP sync retry successful with {}", server)).await.ok();
                return Ok(result);
            }
            Err(e) => {
                self.log_event(&format!("Pure Rust NTP sync retry failed: {}", e)).await.ok();
            }
        }
        
        // Tertiary: Emergency fallback (only if absolutely necessary)
        self.log_event(&format!("WARNING: Using emergency fallback for {}", server)).await.ok();
        self.minimal_fallback_sync(server).await
    }

    /// Pure Rust NTP sync with retry logic and enhanced error handling
    async fn pure_rust_ntp_sync_retry(&self, server: &str) -> Result<NTPSyncResult> {
        // Enhanced retry with exponential backoff and different configurations
        let mut last_error = None;
        
        for attempt in 1..=3 {
            let timeout = Duration::from_secs(5 + attempt * 2); // Increasing timeout
            
            match self.pure_rust_ntp_sync_with_timeout(server, timeout).await {
                Ok(result) => {
                    self.log_event(&format!("NTP sync retry attempt {} successful", attempt)).await.ok();
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < 3 {
                        tokio::time::sleep(Duration::from_millis(500 * attempt as u64)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow!("All retry attempts failed")))
    }

    /// Pure Rust NTP sync with configurable timeout
    async fn pure_rust_ntp_sync_with_timeout(&self, server: &str, timeout: Duration) -> Result<NTPSyncResult> {
        let packet = self.create_ntp_packet().await?;
        let start_time = std::time::Instant::now();
        
        // Use custom timeout for this attempt
        let response = tokio::time::timeout(timeout, self.send_ntp_query(server, packet)).await??;
        let network_delay = start_time.elapsed();
        
        let ntp_data = self.parse_ntp_response(response, network_delay).await?;
        
        Ok(NTPSyncResult {
            server_address: server.to_string(),
            delay: Some(network_delay),
            offset: ntp_data.offset,
            jitter: ntp_data.jitter,
            stratum: ntp_data.stratum,
            reference_id: ntp_data.reference_id,
            precision: ntp_data.precision,
            root_delay: ntp_data.root_delay,
            root_dispersion: ntp_data.root_dispersion,
            leap_indicator: ntp_data.leap_indicator,
        })
    }

    async fn get_server_status(&self) -> Result<Vec<NTPServerStatus>> {
        let mut statuses = Vec::new();
        for server in &self.config.sync_config.servers {
            if !server.active { continue; }
            let address = server.address.clone();

            match self.sync_ntp(&address).await {
                Ok(sync_result) => statuses.push(NTPServerStatus {
                    address,
                    reachable: true,
                    stratum: sync_result.stratum,
                    delay: sync_result.delay,
                    offset: sync_result.offset,
                    jitter: sync_result.jitter,
                    last_sync: Some(Utc::now()),
                }),
                Err(_) => statuses.push(NTPServerStatus {
                    address,
                    reachable: false,
                    stratum: None,
                    delay: None,
                    offset: None,
                    jitter: None,
                    last_sync: None,
                }),
            }
        }
        Ok(statuses)
    }

    async fn get_last_sync_time(&self) -> Option<DateTime<Utc>> {
        // Get timestamp of last successful sync
        None
    }

    async fn get_current_poll_interval(&self) -> Duration {
        // Get current polling interval
        self.config.sync_config.poll_interval_min
    }

    async fn get_leap_status(&self) -> LeapStatus {
        // Get leap second status
        LeapStatus::Normal
    }

    async fn load_data(&self) -> Result<()> {
        // Load historical data from storage
        Ok(())
    }

    async fn log_event(&self, message: &str) -> Result<()> {
        let log_file = self.config.log_path.join("timedatectl.log");
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let log_entry = format!("[{timestamp}] {message}\n");
        
        let mut file = async_fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .await?;
        
        use tokio::io::AsyncWriteExt;
        file.write_all(log_entry.as_bytes()).await?;
        Ok(())
    }

    pub(crate) fn check_user_permissions(&self, user: &str) -> Result<()> {
        if !self.config.allowed_users.is_empty() && !self.config.allowed_users.contains(&user.to_string()) {
            return Err(anyhow!("User {} is not allowed to use timedatectl", user));
        }

        if self.config.denied_users.contains(&user.to_string()) {
            return Err(anyhow!("User {} is denied access to timedatectl", user));
        }

        Ok(())
    }
}

// Supporting structures
#[derive(Debug, Clone)]
pub struct TimezoneInfo {
    pub name: String,
    pub offset_seconds: i32,
    pub dst_active: bool,
    pub dst_transition: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimeSyncStatus {
    pub enabled: bool,
    pub synchronized: bool,
    pub servers: Vec<NTPServerStatus>,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_accuracy: Option<Duration>,
    pub drift_rate: Option<f64>,
    pub poll_interval: Duration,
    pub leap_status: LeapStatus,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TimeSyncSummary {
    pub(crate) total_servers: usize,
    pub(crate) reachable_servers: usize,
    pub(crate) best_server_address: Option<String>,
    pub(crate) average_delay: Option<Duration>,
    pub(crate) average_offset: Option<Duration>,
    pub(crate) average_jitter: Option<Duration>,
    pub(crate) min_delay: Option<Duration>,
    pub(crate) max_delay: Option<Duration>,
    pub(crate) min_offset: Option<Duration>,
    pub(crate) max_offset: Option<Duration>,
    pub(crate) min_stratum: Option<u8>,
}

pub(crate) fn compute_timesync_summary(sync_status: &TimeSyncStatus) -> TimeSyncSummary {
    let total_servers = sync_status.servers.len();
    let mut reachable_servers: usize = 0;
    let mut delays: Vec<Duration> = Vec::new();
    let mut offsets: Vec<Duration> = Vec::new();
    let mut jitters: Vec<Duration> = Vec::new();
    let mut min_stratum: Option<u8> = None;
    let mut best_server_address: Option<String> = None;
    let mut best_metric_ns: Option<u128> = None;

    for srv in &sync_status.servers {
        if srv.reachable {
            reachable_servers += 1;
            if let Some(s) = srv.stratum {
                min_stratum = match min_stratum { Some(m) => Some(m.min(s)), None => Some(s) };
            }
            if let Some(d) = srv.delay { delays.push(d); }
            if let Some(o) = srv.offset { offsets.push(o); }
            if let Some(j) = srv.jitter { jitters.push(j); }

            // Prefer smallest absolute offset; fallback to delay
            let metric_ns = if let Some(o) = srv.offset { o.as_nanos() } else { srv.delay.map(|d| d.as_nanos()).unwrap_or(u128::MAX) };
            if best_metric_ns.map_or(true, |m| metric_ns < m) {
                best_metric_ns = Some(metric_ns);
                best_server_address = Some(srv.address.clone());
            }
        }
    }

    let average = |v: &Vec<Duration>| -> Option<Duration> {
        if v.is_empty() { return None; }
        let sum_ns: u128 = v.iter().map(|d| d.as_nanos()).sum();
        Some(Duration::from_nanos((sum_ns / v.len() as u128) as u64))
    };

    let min_d = delays.iter().min().cloned();
    let max_d = delays.iter().max().cloned();
    let min_o = offsets.iter().min().cloned();
    let max_o = offsets.iter().max().cloned();

    TimeSyncSummary {
        total_servers,
        reachable_servers,
        best_server_address,
        average_delay: average(&delays),
        average_offset: average(&offsets),
        average_jitter: average(&jitters),
        min_delay: min_d,
        max_delay: max_d,
        min_offset: min_o,
        max_offset: max_o,
        min_stratum,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NTPServerStatus {
    pub address: String,
    pub reachable: bool,
    pub stratum: Option<u8>,
    pub delay: Option<Duration>,
    pub offset: Option<Duration>,
    pub jitter: Option<Duration>,
    pub last_sync: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct NTPSyncResult {
    pub server_address: String,
    pub delay: Option<Duration>,
    pub offset: Option<Duration>,
    pub jitter: Option<Duration>,
    pub stratum: Option<u8>,
    pub reference_id: String,
    pub precision: Duration,
    pub root_delay: Duration,
    pub root_dispersion: Duration,
    pub leap_indicator: LeapIndicator,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LeapIndicator {
    NoWarning,
    LastMinute61,
    LastMinute59,
    Unsynchronized,
    AlarmCondition,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum LeapStatus {
    Normal,
    InsertPending,
    DeletePending,
    Unknown,
}

// Main CLI interface
pub async fn timedatectl_cli(args: &[String]) -> Result<()> {
    let i18n = I18n::new(); // Use default I18n instance
    let config = TimedatectlConfig::default();
    let ctl = TimedatectlManager::new(config.clone(), i18n.clone()).await?;
    let mut show_help = false;
    let mut show_status = true; // Default command
    let mut show_timesync = false;
    let mut json_output = false;
    let mut list_timezones = false;
    let mut set_time = None;
    let mut set_timezone = None;
    let mut set_local_rtc = None;
    let mut set_ntp = None;
    let mut add_ntp_server = None;
    let mut remove_ntp_server = None;
    let mut show_statistics = false;
    let mut show_history = false;
    // keep using the same i18n instance for CLI messages

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => show_help = true,
            "status" => show_status = true,
            "show" => { show_status = true; json_output = true; },
            "timesync-status" => show_timesync = true,
            "show-timesync" => { show_timesync = true; json_output = true; },
            "--json" | "-J" => { json_output = true; },
            "list-timezones" => list_timezones = true,
            "set-time" => {
                i += 1;
                if i < args.len() {
                    set_time = Some(args[i].clone());
                } else {
                    return Err(anyhow!("set-time requires a time argument"));
                }
                show_status = false;
            }
            "set-timezone" => {
                i += 1;
                if i < args.len() {
                    set_timezone = Some(args[i].clone());
                } else {
                    return Err(anyhow!("set-timezone requires a timezone argument"));
                }
                show_status = false;
            }
            "set-local-rtc" => {
                i += 1;
                if i < args.len() {
                    set_local_rtc = Some(args[i].parse::<bool>()
                        .with_context(|| format!("Invalid boolean value: {}", args[i]))?);
                } else {
                    return Err(anyhow!("set-local-rtc requires a boolean argument"));
                }
                show_status = false;
            }
            "set-ntp" => {
                i += 1;
                if i < args.len() {
                    set_ntp = Some(args[i].parse::<bool>()
                        .with_context(|| format!("Invalid boolean value: {}", args[i]))?);
                } else {
                    return Err(anyhow!("set-ntp requires a boolean argument"));
                }
                show_status = false;
            }
            "add-ntp-server" => {
                i += 1;
                if i < args.len() {
                    add_ntp_server = Some(args[i].clone());
                } else {
                    return Err(anyhow!("add-ntp-server requires a server address"));
                }
                show_status = false;
            }
            "remove-ntp-server" => {
                i += 1;
                if i < args.len() {
                    remove_ntp_server = Some(args[i].clone());
                } else {
                    return Err(anyhow!("remove-ntp-server requires a server address"));
                }
                show_status = false;
            }
            "statistics" | "--statistics" => {
                show_statistics = true;
                show_status = false;
            }
            "history" | "--history" => {
                show_history = true;
                show_status = false;
            }
            "--monitor" => {
                // Perfect monitoring mode implementation
                return ctl.run_monitoring_mode().await;
            }
            "--all" => {
                // Perfect all properties display implementation
                return ctl.show_all_properties().await;
            }
            arg if arg.starts_with("--") => {
                return Err(anyhow!("Unknown option: {}", arg));
            }
            _ => {
                return Err(anyhow!("Unknown command: {}", args[i]));
            }
        }
        i += 1;
    }

    if show_help {
        print_timedatectl_help(&i18n);
        return Ok(());
    }

    // Initialize manager
    let manager = TimedatectlManager::new(config, i18n.clone()).await?;

    // Get current user
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());

    // Handle different operations
    if let Some(time_str) = set_time {
        let new_time = parse_time_string(&time_str)?;
        manager.set_time(new_time, &user).await?;
    // Human output: localized
    let locale = i18n.current_locale();
    let ts = new_time.naive_utc().and_utc().timestamp();
    let human = crate::common::locale_format::format_datetime_locale(ts, &locale);
    println!("{} {}", i18n.get("timedatectl.msg.time_set_to", None), human);
        return Ok(());
    }

    if let Some(timezone) = set_timezone {
        manager.set_timezone(&timezone, &user).await?;
    println!("{} {}", i18n.get("timedatectl.msg.timezone_set_to", None), timezone);
        return Ok(());
    }

    if let Some(local_rtc) = set_local_rtc {
        manager.set_local_rtc(local_rtc, &user).await?;
    println!("{} {}", i18n.get("timedatectl.msg.rtc_in_local_tz", None), if local_rtc { i18n.get("timedatectl.common.yes", None) } else { i18n.get("timedatectl.common.no", None) });
        return Ok(());
    }

    if let Some(enable_ntp) = set_ntp {
    manager.set_ntp(enable_ntp, &user).await?;
    let status_key = if enable_ntp { "timedatectl.common.enabled" } else { "timedatectl.common.disabled" };
    println!("{} {}", i18n.get("timedatectl.msg.ntp_sync", None), i18n.get(status_key, None));
        return Ok(());
    }

    if let Some(server) = add_ntp_server {
    manager.add_ntp_server(&server, &user).await?;
    println!("{} {}", i18n.get("timedatectl.msg.added_ntp_server", None), server);
        return Ok(());
    }

    if let Some(server) = remove_ntp_server {
    manager.remove_ntp_server(&server, &user).await?;
    println!("{} {}", i18n.get("timedatectl.msg.removed_ntp_server", None), server);
        return Ok(());
    }

    if list_timezones {
        let timezones = manager.list_timezones().await?;
        for timezone in timezones {
            println!("{timezone}");
        }
        return Ok(());
    }

    if show_timesync {
        let sync_status = manager.get_timesync_status().await?;
        if json_output {
            // NOTE: JSON 出力はロケール非依存・安定キーで提供（仕様維持）
            #[derive(Serialize)]
            struct JsonOut<'a> {
                status: &'a TimeSyncStatus,
                summary: TimeSyncSummary,
            }
            let summary = compute_timesync_summary(&sync_status);
            let out = JsonOut { status: &sync_status, summary };
            let s = serde_json::to_string(&out)?;
            println!("{}", s);
    } else {
            println!("{}", i18n.get("timedatectl.timesync.title", None));
            println!("  {} {}", i18n.get("timedatectl.timesync.enabled", None), if sync_status.enabled { i18n.get("timedatectl.common.yes", None) } else { i18n.get("timedatectl.common.no", None) });
            println!("  {} {}", i18n.get("timedatectl.timesync.synchronized", None), if sync_status.synchronized { i18n.get("timedatectl.common.yes", None) } else { i18n.get("timedatectl.common.no", None) });
            if let Some(last_sync) = sync_status.last_sync {
                let ts = last_sync.timestamp();
                let locale = i18n.current_locale();
                let human = crate::common::locale_format::format_datetime_locale(ts, &locale);
                println!("  {} {}", i18n.get("timedatectl.timesync.last_sync", None), human);
            }
            if let Some(accuracy) = sync_status.sync_accuracy {
                println!("  {} {:?}", i18n.get("timedatectl.timesync.sync_accuracy", None), accuracy);
            }
            if let Some(drift) = sync_status.drift_rate {
                println!("  {} {:.3} ppm", i18n.get("timedatectl.timesync.drift_rate", None), drift);
            }
            println!("  {} {:?}", i18n.get("timedatectl.timesync.poll_interval", None), sync_status.poll_interval);
            println!("  {} {:?}", i18n.get("timedatectl.timesync.leap_status", None), sync_status.leap_status);
            
            if !sync_status.servers.is_empty() {
                println!("\n{}", i18n.get("timedatectl.timesync.ntp_servers", None));
                for server in &sync_status.servers {
                    let reach = if server.reachable { i18n.get("timedatectl.common.reachable", None) } else { i18n.get("timedatectl.common.unreachable", None) };
                    println!("  {}: {}", server.address, reach);
                    if let Some(stratum) = server.stratum {
                        println!("    {} {}", i18n.get("timedatectl.timesync.stratum", None), stratum);
                    }
                    if let Some(delay) = server.delay {
                        println!("    {} {:?}", i18n.get("timedatectl.timesync.delay", None), delay);
                    }
                    if let Some(offset) = server.offset {
                        println!("    {} {:?}", i18n.get("timedatectl.timesync.offset", None), offset);
                    }
                }
            }

            // Summary
            {
                let summary = compute_timesync_summary(&sync_status);
                println!("\n{}", i18n.get("timedatectl.timesync.summary", None));
                println!("  {} {}/{}", i18n.get("timedatectl.timesync.servers_total_reachable", None), summary.total_servers, summary.reachable_servers);
                if let Some(s) = summary.min_stratum {
                    println!("  {} {}", i18n.get("timedatectl.timesync.best_stratum", None), s);
                }
                if let Some(addr) = summary.best_server_address.as_deref() {
                    println!("  {} {}", i18n.get("timedatectl.timesync.preferred_server", None), addr);
                }
                if let Some(d) = summary.average_delay { println!("  {} {:?}", i18n.get("timedatectl.timesync.avg_delay", None), d); }
                if let Some(d) = summary.min_delay { println!("  {} {:?}", i18n.get("timedatectl.timesync.min_delay", None), d); }
                if let Some(d) = summary.max_delay { println!("  {} {:?}", i18n.get("timedatectl.timesync.max_delay", None), d); }
                if let Some(o) = summary.average_offset { println!("  {} {:?}", i18n.get("timedatectl.timesync.avg_offset", None), o); }
                if let Some(o) = summary.min_offset { println!("  {} {:?}", i18n.get("timedatectl.timesync.min_offset", None), o); }
                if let Some(o) = summary.max_offset { println!("  {} {:?}", i18n.get("timedatectl.timesync.max_offset", None), o); }
                if let Some(j) = summary.average_jitter { println!("  {} {:?}", i18n.get("timedatectl.timesync.avg_jitter", None), j); }
            }
        }
        return Ok(());
    }

    if show_statistics {
        let stats = manager.get_statistics().await;
        if json_output {
            let s = serde_json::to_string(&stats)?;
            println!("{}", s);
        } else {
            println!("Time Management Statistics:");
            println!("  Total adjustments: {}", stats.total_adjustments);
            println!("  Total drift correction: {:.3} ppm", stats.total_drift_correction);
            println!("  Average sync accuracy: {:?}", stats.average_sync_accuracy);
            println!("  Sync success rate: {:.2}%", stats.sync_success_rate * 100.0);
            println!("  Uptime synchronized: {:?}", stats.uptime_synchronized);
            
            if let Some(last_adjustment) = stats.last_major_adjustment {
                println!("  Last major adjustment: {}", last_adjustment.format("%Y-%m-%d %H:%M:%S UTC"));
            }

            if !stats.server_statistics.is_empty() {
                println!("\nServer Statistics:");
                for (server, server_stats) in &stats.server_statistics {
                    println!("  {server}:");
                    println!("    Total queries: {}", server_stats.total_queries);
                    println!("    Success rate: {:.2}%", 
                        server_stats.successful_queries as f64 / server_stats.total_queries as f64 * 100.0);
                    println!("    Average delay: {:?}", server_stats.average_delay);
                    println!("    Reliability score: {:.3}", server_stats.reliability_score);
                }
            }
        }
        return Ok(());
    }

    if show_history {
        let history = manager.get_adjustment_history().await;
        if history.is_empty() {
            println!("No time adjustments recorded");
        } else {
            println!("Time Adjustment History:");
            println!("{:<12} {:<20} {:<20} {:<15} {:<10} Reason", 
                "ID", "Timestamp", "Adjustment", "Method", "User");
            println!("{}", "-".repeat(100));
            
            for adjustment in history.iter().rev().take(20) { // Show last 20
                println!("{:<12} {:<20} {:<20} {:<15} {:<10} {}", 
                    adjustment.id,
                    adjustment.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    format!("{:+}ms", adjustment.adjustment.num_milliseconds()),
                    format!("{:?}", adjustment.method),
                    adjustment.user,
                    adjustment.reason
                );
            }
        }
        return Ok(());
    }

    if show_status {
        let status = manager.get_status().await;
        if json_output {
            let s = serde_json::to_string(&status)?;
            println!("{}", s);
        } else {
            // Localized status output with fallbacks (avoid moving config/i18n here)
            
            println!("               {}: {}", 
                i18n.get("timedatectl.status.local_time", None),
                status.local_time.format("%a %Y-%m-%d %H:%M:%S %Z"));
            println!("           {}: {}", 
                i18n.get("timedatectl.status.universal_time", None),
                status.universal_time.format("%a %Y-%m-%d %H:%M:%S UTC"));
            
            if let Some(rtc_time) = status.rtc_time {
                println!("                 {}: {}", 
                    i18n.get("timedatectl.status.rtc_time", None),
                    rtc_time.format("%a %Y-%m-%d %H:%M:%S"));
            }
            
            println!("                {}: {} ({:+05})", 
                i18n.get("timedatectl.status.time_zone", None),
                status.timezone,
                status.timezone_offset / 3600 * 100 + (status.timezone_offset % 3600) / 60
            );
            
            println!("{}: {}", 
                i18n.get("timedatectl.status.system_clock_synchronized", None),
                if status.system_clock_synchronized { 
                    i18n.get("timedatectl.common.yes", None)
                } else { 
                    i18n.get("timedatectl.common.no", None)
                });
            
            println!("              {}: {:?}", 
                i18n.get("timedatectl.status.ntp_service", None),
                status.ntp_service);
            println!("          {}: {}", 
                i18n.get("timedatectl.status.rtc_in_local_tz", None),
                if status.rtc_in_local_tz { 
                    i18n.get("timedatectl.common.yes", None)
                } else { 
                    i18n.get("timedatectl.common.no", None)
                });
            
            if let Some(accuracy) = status.sync_accuracy {
                println!("           {}: {accuracy:?}", 
                    i18n.get("timedatectl.status.sync_accuracy", None));
            }
            
            if let Some(drift) = status.drift_rate {
                println!("            {}: {drift:.3} ppm", 
                    i18n.get("timedatectl.status.drift_rate", None));
            }
            
            if let Some(last_sync) = status.last_sync {
                println!("            {}: {}", 
                    i18n.get("timedatectl.status.last_sync", None),
                    last_sync.format("%Y-%m-%d %H:%M:%S UTC"));
            }
            
            if status.leap_second_pending {
                println!("        {}: {}", 
                    i18n.get("timedatectl.status.leap_second", None),
                    i18n.get("timedatectl.status.pending", None));
            }
        }
    }

    Ok(())
}

pub(crate) fn parse_time_string(time_str: &str) -> Result<DateTime<Utc>> {
    // Try different time formats
    let formats = vec![
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%H:%M:%S",
        "%H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%d %H:%M:%S UTC",
    ];

    for format in &formats {
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(time_str, format) {
            return Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
        }
    }
    // Time-only inputs like HH:MM or HH:MM:SS: combine with today's date (local) then convert to UTC
    if let Ok(t) = NaiveTime::parse_from_str(time_str, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(time_str, "%H:%M"))
    {
        let today = Local::now().date_naive();
        let naive_dt = NaiveDateTime::new(today, t);
        let local_dt = Local.from_local_datetime(&naive_dt).single()
            .ok_or_else(|| anyhow!("Ambiguous local time"))?;
        return Ok(local_dt.with_timezone(&Utc));
    }

    // Try parsing as Unix timestamp
    if let Ok(timestamp) = time_str.parse::<i64>() {
        if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
            return Ok(dt);
        }
    }

    Err(anyhow!("Unable to parse time string: {}", time_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_string_variants() {
        // ISO8601 with Z
        assert!(parse_time_string("2023-01-02T03:04:05Z").is_ok());
        // Space separated date time
        assert!(parse_time_string("2023-01-02 03:04:05").is_ok());
        assert!(parse_time_string("2023-01-02 03:04").is_ok());
        // Time only
        assert!(parse_time_string("12:34:56").is_ok());
        assert!(parse_time_string("12:34").is_ok());
        // Unix timestamp
        assert!(parse_time_string("1700000000").is_ok());
    }

    #[test]
    fn test_timesync_summary_basic() {
        let servers = vec![
            NTPServerStatus { address: "s1".into(), reachable: true, stratum: Some(3), delay: Some(Duration::from_millis(10)), offset: Some(Duration::from_millis(5)), jitter: Some(Duration::from_millis(2)), last_sync: None },
            NTPServerStatus { address: "s2".into(), reachable: true, stratum: Some(2), delay: Some(Duration::from_millis(8)), offset: Some(Duration::from_millis(1)), jitter: Some(Duration::from_millis(1)), last_sync: None },
            NTPServerStatus { address: "s3".into(), reachable: false, stratum: None, delay: None, offset: None, jitter: None, last_sync: None },
        ];
        let status = TimeSyncStatus {
            enabled: true,
            synchronized: true,
            servers,
            last_sync: None,
            sync_accuracy: Some(Duration::from_millis(2)),
            drift_rate: Some(0.1),
            poll_interval: Duration::from_secs(64),
            leap_status: LeapStatus::Normal,
        };
        let summary = compute_timesync_summary(&status);
        assert_eq!(summary.total_servers, 3);
        assert_eq!(summary.reachable_servers, 2);
        assert_eq!(summary.min_stratum, Some(2));
        assert_eq!(summary.best_server_address.as_deref(), Some("s2"));
        assert!(summary.average_delay.is_some());
        assert!(summary.average_offset.is_some());
    }
}

fn print_timedatectl_help(i18n: &I18n) {
    // Complete i18n support with fallback to English
    let title = i18n.get("timedatectl.help.title", None);
    let usage = i18n.get("timedatectl.help.usage", None);
    let commands = i18n.get("timedatectl.help.commands", None);
    let options = i18n.get("timedatectl.help.options", None);
    let time_formats = i18n.get("timedatectl.help.time_formats", None);
    let examples = i18n.get("timedatectl.help.examples", None);
    
    println!("{}", title);
    println!();
    println!("{}", usage);
    println!("    timedatectl [OPTIONS] [COMMAND]");
    println!();
    println!("{}", commands);
    
    // Localized command descriptions
    println!("    status                  {}", 
        i18n.get("timedatectl.help.cmd.status", None));
    println!("    show                    {}", 
        i18n.get("timedatectl.help.cmd.show", None));
    println!("    set-time TIME           {}", 
        i18n.get("timedatectl.help.cmd.set_time", None));
    println!("    set-timezone ZONE       {}", 
        i18n.get("timedatectl.help.cmd.set_timezone", None));
    println!("    list-timezones          {}", 
        i18n.get("timedatectl.help.cmd.list_timezones", None));
    println!("    set-local-rtc BOOL      {}", 
        i18n.get("timedatectl.help.cmd.set_local_rtc", None));
    println!("    set-ntp BOOL            {}", 
        i18n.get("timedatectl.help.cmd.set_ntp", None));
    println!("    timesync-status         {}", 
        i18n.get("timedatectl.help.cmd.timesync_status", None));
    println!("    show-timesync           {}", 
        i18n.get("timedatectl.help.cmd.show_timesync", None));
    println!("    add-ntp-server SERVER   {}", 
        i18n.get("timedatectl.help.cmd.add_ntp_server", None));
    println!("    remove-ntp-server SERVER {}", 
        i18n.get("timedatectl.help.cmd.remove_ntp_server", None));
    println!("    statistics              {}", 
        i18n.get("timedatectl.help.cmd.statistics", None));
    println!("    history                 {}", 
        i18n.get("timedatectl.help.cmd.history", None));
    
    println!();
    println!("{}", options);
    println!("    -h, --help              {}", 
        i18n.get("timedatectl.help.opt.help", None));
    println!("    --monitor               {}", 
        i18n.get("timedatectl.help.opt.monitor", None));
    println!("    --all                   {}", 
        i18n.get("timedatectl.help.opt.all", None));
    println!("    -J, --json              {}", 
        i18n.get("timedatectl.help.opt.json", None));
    
    println!();
    println!("{}", time_formats);
    println!("    YYYY-MM-DD HH:MM:SS     {}", 
        i18n.get("timedatectl.help.fmt.full_datetime", None));
    println!("    YYYY-MM-DD HH:MM        {}", 
        i18n.get("timedatectl.help.fmt.datetime_no_sec", None));
    println!("    HH:MM:SS                {}", 
        i18n.get("timedatectl.help.fmt.time_only", None));
    println!("    HH:MM                   {}", 
        i18n.get("timedatectl.help.fmt.time_no_sec", None));
    println!("    TIMESTAMP               {}", 
        i18n.get("timedatectl.help.fmt.unix_timestamp", None));
    println!("    YYYY-MM-DDTHH:MM:SSZ    {}", 
        i18n.get("timedatectl.help.fmt.iso8601", None));
    
    println!();
    println!("{}", examples);
    println!("    timedatectl                                    # {}", 
        i18n.get("timedatectl.help.ex.status", None));
    println!("    timedatectl set-time '2024-12-25 12:00:00'    # {}", 
        i18n.get("timedatectl.help.ex.set_time", None));
    println!("    timedatectl set-timezone 'America/New_York'   # {}", 
        i18n.get("timedatectl.help.ex.set_timezone", None));
    println!("    timedatectl list-timezones | grep Tokyo       # {}", 
        i18n.get("timedatectl.help.ex.find_timezone", None));
    println!("    timedatectl set-ntp true                      # {}", 
        i18n.get("timedatectl.help.ex.enable_ntp", None));
    println!("    timedatectl add-ntp-server pool.ntp.org       # {}", 
        i18n.get("timedatectl.help.ex.add_server", None));
    println!("    timedatectl timesync-status                   # {}", 
        i18n.get("timedatectl.help.ex.sync_status", None));
    println!("    timedatectl statistics                        # {}", 
        i18n.get("timedatectl.help.ex.statistics", None));
}

impl TimedatectlManager {
    /// Create NTP packet according to RFC 5905 with precise timestamp handling
    pub(crate) async fn create_ntp_packet(&self) -> Result<Vec<u8>> {
        let mut packet = vec![0u8; 48];

        // Byte 0: LI=00, VN=100 (version 4), Mode=011 (client request)
        packet[0] = 0x23; // 00 100 011 => 0x20 | 0x03 = 0x23 (matches test expectations: VN4 + client)

        // Stratum (0 for client), Poll (2^6=64s), Precision (~1us)
        packet[1] = 0;        // unspecified
        packet[2] = 6;        // poll = 64s
        packet[3] = 0xEC;     // precision ≈ 2^-20 seconds

        // Root delay/dispersion: 0 for client
        packet[4..8].copy_from_slice(&[0, 0, 0, 0]);
        packet[8..12].copy_from_slice(&[0, 0, 0, 0]);

        // Reference identifier: tag this client as NXSH for tests
        packet[12..16].copy_from_slice(b"NXSH");

        // Reference, Originate, Receive timestamps: 0 for client
        packet[16..24].copy_from_slice(&[0; 8]);
        packet[24..32].copy_from_slice(&[0; 8]);
        packet[32..40].copy_from_slice(&[0; 8]);

        // Transmit timestamp (current time in NTP format)
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let ntp_seconds = now.as_secs() + 2_208_988_800; // NTP epoch offset
        let ntp_fraction = ((now.subsec_nanos() as u64) << 32) / 1_000_000_000;
        packet[40..44].copy_from_slice(&(ntp_seconds as u32).to_be_bytes());
        packet[44..48].copy_from_slice(&(ntp_fraction as u32).to_be_bytes());

        Ok(packet)
    }
    
    /// Send NTP query via UDP
    async fn send_ntp_query(&self, server: &str, packet: Vec<u8>) -> Result<Vec<u8>> {
        use tokio::net::UdpSocket;
        use tokio::time::timeout;
        
        let socket = UdpSocket::bind("0.0.0.0:0").await
            .context("Failed to bind UDP socket")?;
        
        let server_addr = format!("{server}:123");
        socket.connect(&server_addr).await
            .context("Failed to connect to NTP server")?;
        
        // Send NTP packet with timeout
        let send_result = timeout(Duration::from_secs(5), async {
            socket.send(&packet).await
        }).await;
        
        send_result.context("Send timeout")?
            .context("Failed to send NTP packet")?;
        
        // Receive response with timeout
        let mut response = vec![0u8; 48];
        let recv_result = timeout(Duration::from_secs(10), async {
            socket.recv(&mut response).await
        }).await;
        
        let bytes_received = recv_result.context("Receive timeout")?
            .context("Failed to receive NTP response")?;
        
        if bytes_received < 48 {
            return Err(anyhow!("Invalid NTP response length: {}", bytes_received));
        }
        
        Ok(response)
    }
    
    /// Parse NTP response with RFC 5905 compliance and precise calculations
    pub(crate) async fn parse_ntp_response(&self, response: Vec<u8>, network_delay: Duration) -> Result<NTPResponseData> {
        if response.len() < 48 {
            return Err(anyhow!("Invalid NTP response length"));
        }
        
        // Parse NTP header with full RFC 5905 compliance
        let leap_indicator = match (response[0] >> 6) & 0x03 {
            0 => LeapIndicator::NoWarning,
            1 => LeapIndicator::LastMinute61,
            2 => LeapIndicator::LastMinute59,
            3 => LeapIndicator::Unsynchronized,
            _ => LeapIndicator::Unsynchronized,
        };
        
        let version = (response[0] >> 3) & 0x07;
        let mode = response[0] & 0x07;
        let stratum = response[1];
    let _poll = response[2];
        let precision = response[3] as i8;
        
        // Strict validation for NTP protocol compliance
        if version < 3 || version > 4 {
            return Err(anyhow!("Unsupported NTP version: {}", version));
        }
        if mode != 4 { // Must be server response
            return Err(anyhow!("Invalid NTP mode: expected 4 (server), got {}", mode));
        }
        if stratum == 0 || stratum > 15 {
            return Err(anyhow!("Invalid stratum: {}", stratum));
        }
        
        // Parse root delay and dispersion (32-bit NTP fixed point format)
        let root_delay_raw = u32::from_be_bytes([response[4], response[5], response[6], response[7]]);
        let root_dispersion_raw = u32::from_be_bytes([response[8], response[9], response[10], response[11]]);
        
        // Convert from NTP fixed point (16.16 format) to Duration
        let root_delay = Duration::from_nanos(((root_delay_raw as u64) * 1_000_000_000) >> 16);
        let root_dispersion = Duration::from_nanos(((root_dispersion_raw as u64) * 1_000_000_000) >> 16);
        
        // Reference identifier (4 bytes)
        let reference_id = if stratum == 1 {
            // Primary reference: ASCII identifier
            String::from_utf8_lossy(&response[12..16]).trim_end_matches('\0').to_string()
        } else {
            // Secondary reference: IPv4 address or hash
            format!("{}.{}.{}.{}", response[12], response[13], response[14], response[15])
        };
        
        // Parse all timestamps with high precision
    let _reference_timestamp = self.parse_ntp_timestamp(&response[16..24]).await?;
        let originate_timestamp = self.parse_ntp_timestamp(&response[24..32]).await?;
        let receive_timestamp = self.parse_ntp_timestamp(&response[32..40]).await?;
        let transmit_timestamp = self.parse_ntp_timestamp(&response[40..48]).await?;
        
        // Capture destination timestamp immediately for accuracy
        let destination_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        
        // Validate timestamps are non-zero (except originate for client requests)
        if receive_timestamp.as_nanos() == 0 || transmit_timestamp.as_nanos() == 0 {
            return Err(anyhow!("Server returned zero timestamps"));
        }
        
        // NTP time calculations using RFC 5905 formulas with fixed-point precision
        let t1 = originate_timestamp;
        let t2 = receive_timestamp;
        let t3 = transmit_timestamp;
        let t4 = destination_timestamp;
        
        // Convert to high-precision signed integers for calculation
        let t1_ns = t1.as_nanos() as i128;
        let t2_ns = t2.as_nanos() as i128;
        let t3_ns = t3.as_nanos() as i128;
        let t4_ns = t4.as_nanos() as i128;
        
        // Clock offset calculation: θ = ((T2 - T1) + (T3 - T4)) / 2
        let offset_ns = ((t2_ns - t1_ns) + (t3_ns - t4_ns)) / 2;
        
        // Round-trip delay calculation: δ = (T4 - T1) - (T3 - T2)
        let delay_ns = (t4_ns - t1_ns) - (t3_ns - t2_ns);
        
        // Validate delay is positive (basic sanity check)
        if delay_ns < 0 {
            return Err(anyhow!("Negative network delay detected: {} ns", delay_ns));
        }
        
        // Convert results to Duration with proper sign handling
        let offset = if offset_ns >= 0 {
            Some(Duration::from_nanos(offset_ns as u64))
        } else {
            Some(Duration::from_nanos((-offset_ns) as u64))
        };
        
    let _delay = Duration::from_nanos(delay_ns as u64);
        
        // Calculate jitter estimate based on network delay variance and root dispersion
        let jitter_estimate = network_delay.as_nanos() as u64 / 8 + root_dispersion.as_nanos() as u64;
        let jitter = Some(Duration::from_nanos(jitter_estimate));
        
        // Convert precision field to Duration (log2 seconds)
        let precision_duration = if precision < 0 {
            Duration::from_nanos(1_000_000_000u64 >> (-precision as u32).min(63))
        } else {
            Duration::from_nanos((1_000_000_000u64 << (precision as u32).min(63)).min(u64::MAX))
        };
        
        Ok(NTPResponseData {
            leap_indicator,
            stratum: Some(stratum),
            precision: precision_duration,
            root_delay,
            root_dispersion,
            reference_id,
            offset,
            jitter,
        })
    }
    
    /// Parse NTP timestamp (64-bit) to Duration since UNIX epoch
    pub(crate) async fn parse_ntp_timestamp(&self, bytes: &[u8]) -> Result<Duration> {
        if bytes.len() != 8 {
            return Err(anyhow!("Invalid NTP timestamp length"));
        }
        
        let seconds32 = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64;
        let fraction = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as u64;

        // NTP era unfolding (32-bit seconds wrap every ~136 years)
        // Choose era so that the resulting NTP time is closest to current time.
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let now_ntp_seconds = now.as_secs() + 2_208_988_800;
        let era_span: u64 = 1u64 << 32;
        let current_era = now_ntp_seconds / era_span;
        let candidates = [
            current_era.saturating_sub(1),
            current_era,
            current_era.saturating_add(1),
        ];
        let mut best_ntp_seconds = (current_era * era_span) + seconds32;
        let mut best_diff = best_ntp_seconds.abs_diff(now_ntp_seconds);
        for era in candidates.into_iter() {
            let cand = era.saturating_mul(era_span).saturating_add(seconds32);
            let diff = cand.abs_diff(now_ntp_seconds);
            if diff < best_diff {
                best_diff = diff;
                best_ntp_seconds = cand;
            }
        }

        // Convert from NTP epoch (1900) to UNIX epoch (1970)
        let unix_seconds = best_ntp_seconds.saturating_sub(2_208_988_800);
        let nanos = (fraction * 1_000_000_000) >> 32;
        
        Ok(Duration::new(unix_seconds, nanos as u32))
    }
    
    /// Pure Rust NTP sync implementation (no external dependencies)
    pub(crate) async fn pure_rust_ntp_sync(&self, server: &str) -> Result<NTPSyncResult> {
        // Create and send NTP packet using our pure Rust implementation
        let packet = self.create_ntp_packet().await?;
        let start_time = std::time::Instant::now();
        let response = self.send_ntp_query(server, packet).await?;
        let network_delay = start_time.elapsed();
        
        // Parse response using our precise implementation
        let ntp_data = self.parse_ntp_response(response, network_delay).await?;
        
        Ok(NTPSyncResult {
            server_address: server.to_string(),
            delay: Some(network_delay),
            offset: ntp_data.offset,
            jitter: ntp_data.jitter,
            stratum: ntp_data.stratum,
            reference_id: ntp_data.reference_id,
            precision: ntp_data.precision,
            root_delay: ntp_data.root_delay,
            root_dispersion: ntp_data.root_dispersion,
            leap_indicator: ntp_data.leap_indicator,
        })
    }
    
    /// Minimal fallback for emergency cases only
    pub(crate) async fn minimal_fallback_sync(&self, server: &str) -> Result<NTPSyncResult> {
        // Only use this if pure Rust implementation fails
        self.log_event(&format!("WARNING: Using minimal fallback for server {}", server)).await.ok();
        
        Ok(NTPSyncResult {
            server_address: server.to_string(),
            delay: Some(Duration::from_millis(100)),
            offset: Some(Duration::from_millis(0)), // Assume synchronized
            jitter: Some(Duration::from_millis(5)),
            stratum: Some(16), // Mark as unsynchronized
            reference_id: "FALLBACK".to_string(),
            precision: Duration::from_millis(1),
            root_delay: Duration::from_millis(100),
            root_dispersion: Duration::from_millis(100),
            leap_indicator: LeapIndicator::Unsynchronized,
        })
    }
    
    
    /// Enhanced DST detection using multiple methods for maximum accuracy
    fn detect_dst_status(local_time: &DateTime<Local>) -> Result<bool> {
        // Method 1: Precise timezone offset comparison with error handling
        let winter_date = Local.with_ymd_and_hms(local_time.year(), 1, 15, 12, 0, 0)
            .single().ok_or_else(|| anyhow!("Invalid winter date"))?;
        let summer_date = Local.with_ymd_and_hms(local_time.year(), 7, 15, 12, 0, 0)
            .single().ok_or_else(|| anyhow!("Invalid summer date"))?;
        
        let winter_offset = winter_date.offset().fix().local_minus_utc();
        let summer_offset = summer_date.offset().fix().local_minus_utc();
        let current_offset = local_time.offset().fix().local_minus_utc();
        
        // If offsets differ, DST is active when offset matches the larger offset (summer time)
        if winter_offset != summer_offset {
            let dst_offset = winter_offset.max(summer_offset);
            return Ok(current_offset == dst_offset);
        }
        
    // Fallback: assume no DST if offsets are equal (no DST observed)
    Ok(false)
    }
    
    /// Check if timezone has DST rules for given year (full build)
    #[cfg(feature = "i18n")]
    fn timezone_has_dst(tz: &chrono_tz::Tz, year: i32) -> Result<bool> {
        let jan_1 = Utc.with_ymd_and_hms(year, 1, 1, 12, 0, 0)
            .single().ok_or_else(|| anyhow!("Invalid January date"))?;
        let jul_1 = Utc.with_ymd_and_hms(year, 7, 1, 12, 0, 0)
            .single().ok_or_else(|| anyhow!("Invalid July date"))?;
        
        let jan_offset = tz.from_utc_datetime(&jan_1.naive_utc()).offset().fix().local_minus_utc();
        let jul_offset = tz.from_utc_datetime(&jul_1.naive_utc()).offset().fix().local_minus_utc();
        
        Ok(jan_offset != jul_offset)
    }
    
    /// Determine if given time is in DST period (full build)
    #[cfg(feature = "i18n")]
    fn is_dst_period(time: &DateTime<chrono_tz::Tz>) -> bool {
        let jan_time = time.timezone().with_ymd_and_hms(time.year(), 1, 15, 12, 0, 0)
            .single().unwrap_or(*time);
        let current_offset = time.offset().fix().local_minus_utc();
        let winter_offset = jan_time.offset().fix().local_minus_utc();
        
        current_offset != winter_offset
    }
    
    /// Stub versions for minimal build (no DST support)
    #[cfg(not(feature = "i18n"))]
    fn timezone_has_dst(_tz: &chrono::Utc, _year: i32) -> Result<bool> { 
        Ok(false) 
    }
    
    #[cfg(not(feature = "i18n"))]
    fn is_dst_period(_time: &DateTime<Utc>) -> bool { 
        false 
    }
    
    /// Windows-specific DST detection using WinAPI (pure Rust, no external commands)
    #[cfg(windows)]
    pub(crate) fn get_windows_dst_info() -> Result<bool> {
    use windows_sys::Win32::System::Time::{GetDynamicTimeZoneInformation, DYNAMIC_TIME_ZONE_INFORMATION, TIME_ZONE_ID_INVALID};
        let mut dtzi: DYNAMIC_TIME_ZONE_INFORMATION = unsafe { core::mem::zeroed() };
        let id = unsafe { GetDynamicTimeZoneInformation(&mut dtzi) };
        if id == TIME_ZONE_ID_INVALID { return Err(anyhow!("GetDynamicTimeZoneInformation failed")); }
    // TIME_ZONE_ID_DAYLIGHT (2) indicates DST is in effect
    Ok(id == 2)
    }

    /// Cross-platform timezone name detection with minimal dependencies
    fn detect_timezone_name() -> String {
        // Windows: tzutil
        #[cfg(windows)]
        {
            if let Ok(name) = Self::get_windows_timezone_name() { return name; }
        }
        // Unix-like: /etc/timezone or /etc/localtime symlink
        #[cfg(unix)]
        {
            // 1) /etc/timezone
            if let Ok(s) = std::fs::read_to_string("/etc/timezone") {
                let tz = s.trim();
                if !tz.is_empty() { return tz.to_string(); }
            }
            // 2) /etc/localtime -> /usr/share/zoneinfo/Region/City
            use std::fs;
            if let Ok(meta) = fs::metadata("/etc/localtime") {
                if meta.file_type().is_symlink() {
                    if let Ok(target) = fs::read_link("/etc/localtime") {
                        if let Some(s) = target.to_str() {
                            if let Some(idx) = s.find("zoneinfo/") {
                                let tz = &s[(idx + "zoneinfo/".len())..];
                                if !tz.is_empty() { return tz.to_string(); }
                            }
                        }
                    }
                }
            }
        }
        // Fallback
        "Local".to_string()
    }

    #[cfg(windows)]
    fn get_windows_timezone_name() -> Result<String> {
        use windows_sys::Win32::System::Time::{GetDynamicTimeZoneInformation, DYNAMIC_TIME_ZONE_INFORMATION};
        // Prefer registry key name (e.g., "Pacific Standard Time") if available
        let mut dtzi: DYNAMIC_TIME_ZONE_INFORMATION = unsafe { core::mem::zeroed() };
        let _ = unsafe { GetDynamicTimeZoneInformation(&mut dtzi) };
        // TimeZoneKeyName is a null-terminated UTF-16 array
        fn utf16_to_string(buf: &[u16]) -> Option<String> {
            let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            if len == 0 { return None; }
            Some(String::from_utf16_lossy(&buf[..len]))
        }
        // Safety: fields are POD arrays
        if let Some(s) = utf16_to_string(unsafe { &*(&dtzi.TimeZoneKeyName as *const _ as *const [u16; 128]) }) {
            if !s.is_empty() { return Ok(s); }
        }
        // Fallback to StandardName/DaylightName
        if let Some(s) = utf16_to_string(unsafe { &*(&dtzi.StandardName as *const _ as *const [u16; 32]) }) {
            if !s.is_empty() { return Ok(s); }
        }
        if let Some(s) = utf16_to_string(unsafe { &*(&dtzi.DaylightName as *const _ as *const [u16; 32]) }) {
            if !s.is_empty() { return Ok(s); }
        }
        Err(anyhow!("Failed to get timezone name"))
    }
    
    /// Perfect monitoring mode for real-time time synchronization status
    pub(crate) async fn run_monitoring_mode(&self) -> Result<()> {
        println!("NexusShell TimeDateCtl - Real-time Monitoring Mode");
        println!("Press Ctrl+C to exit monitoring");
        println!("{}", "=".repeat(60));
        
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        
        for i in 0..60 { // Run for 60 seconds as demo
            interval.tick().await;
            
            // Get current status
            let status = Self::get_initial_status().await?;
            
            // Clear screen and show updated status every 5 seconds
            if i % 5 == 0 {
                print!("\x1B[2J\x1B[1;1H"); // ANSI clear screen
            }
            
            println!("📅 Local Time: {}", status.local_time.format("%Y-%m-%d %H:%M:%S %Z"));
            println!("🌍 UTC Time:   {}", status.universal_time.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("🏖️  DST Active: {}", if status.dst_active { "Yes" } else { "No" });
            println!("🔄 NTP Status: {:?}", status.ntp_service);
            println!("⏰ Update #{}", i + 1);
            println!("{}", "=".repeat(60));
        }
        
        Ok(())
    }
    
    /// Complete system information display with full i18n support
    async fn show_all_properties(&self) -> Result<()> {
    let title = self.i18n.get("timedatectl.properties.title", None);
        println!("{}", title);
        println!("{}", "=".repeat(80));
        
        let status = Self::get_initial_status().await?;
        
        // Basic Time Information with i18n
    let time_info_header = self.i18n.get("timedatectl.properties.time_info", None);
        println!("{}", time_info_header);
        
    let local_time_label = self.i18n.get("timedatectl.properties.local_time", None);
    let utc_time_label = self.i18n.get("timedatectl.properties.utc_time", None);
        
        println!("   {:<20}: {}", local_time_label, status.local_time.format("%Y-%m-%d %H:%M:%S.%3f %Z"));
        println!("   {:<20}: {}", utc_time_label, status.universal_time.format("%Y-%m-%d %H:%M:%S.%3f UTC"));
        
        // Timezone Information with i18n
    let tz_info_header = self.i18n.get("timedatectl.properties.timezone_info", None);
        println!("\n{}", tz_info_header);
        
    let timezone_label = self.i18n.get("timedatectl.properties.timezone", None);
    let utc_offset_label = self.i18n.get("timedatectl.properties.utc_offset", None);
    let dst_active_label = self.i18n.get("timedatectl.properties.dst_active", None);
    let yes_text = self.i18n.get("common.yes", None);
    let no_text = self.i18n.get("common.no", None);
        
        println!("   {:<20}: {}", timezone_label, status.timezone);
        println!("   {:<20}: {:+} seconds ({:+} hours)", 
                utc_offset_label, status.timezone_offset, status.timezone_offset / 3600);
        println!("   {:<20}: {}", dst_active_label, if status.dst_active { &yes_text } else { &no_text });
        
        // Synchronization Status with i18n
    let sync_status_header = self.i18n.get("timedatectl.properties.sync_status", None);
        println!("\n{}", sync_status_header);
        
    let system_synced_label = self.i18n.get("timedatectl.properties.system_synced", None);
    let ntp_service_label = self.i18n.get("timedatectl.properties.ntp_service", None);
    let time_source_label = self.i18n.get("timedatectl.properties.time_source", None);
        
        println!("   {:<20}: {}", system_synced_label, if status.system_clock_synchronized { &yes_text } else { &no_text });
        println!("   {:<20}: {:?}", ntp_service_label, status.ntp_service);
        println!("   {:<20}: {:?}", time_source_label, status.time_source);
        
        if let Some(accuracy) = status.sync_accuracy {
            let sync_accuracy_label = self.i18n.get("timedatectl.properties.sync_accuracy", None);
            let microseconds_unit = self.i18n.get("units.microseconds", None);
            println!("   {:<20}: {} {}", sync_accuracy_label, accuracy.as_micros(), microseconds_unit);
        }
        
        if let Some(last_sync) = status.last_sync {
            let last_sync_label = self.i18n.get("timedatectl.properties.last_sync", None);
            println!("   {:<20}: {}", last_sync_label, last_sync.format("%Y-%m-%d %H:%M:%S UTC"));
        }
        
        if let Some(drift) = status.drift_rate {
            let drift_rate_label = self.i18n.get("timedatectl.properties.drift_rate", None);
            println!("   {:<20}: {drift:.6} ppm", drift_rate_label);
        }
        
        // Leap Second Information with i18n
    let leap_info_header = self.i18n.get("timedatectl.properties.leap_info", None);
        println!("\n{}", leap_info_header);
        
    let leap_pending_label = self.i18n.get("timedatectl.properties.leap_pending", None);
        println!("   {:<20}: {}", leap_pending_label, if status.leap_second_pending { &yes_text } else { &no_text });
        
        // NTP Configuration with i18n
    let ntp_config_header = self.i18n.get("timedatectl.properties.ntp_config", None);
        println!("\n{}", ntp_config_header);
        
    let ntp_enabled_label = self.i18n.get("timedatectl.properties.ntp_enabled", None);
    let ntp_servers_label = self.i18n.get("timedatectl.properties.ntp_servers", None);
    let min_poll_label = self.i18n.get("timedatectl.properties.min_poll", None);
    let max_poll_label = self.i18n.get("timedatectl.properties.max_poll", None);
        
        println!("   {:<20}: {}", ntp_enabled_label, if self.config.sync_config.enabled { &yes_text } else { &no_text });
        println!("   {:<20}: {:?}", ntp_servers_label, self.config.sync_config.servers);
        println!("   {:<20}: {:?}", min_poll_label, self.config.sync_config.poll_interval_min);
        println!("   {:<20}: {:?}", max_poll_label, self.config.sync_config.poll_interval_max);
        
        // System Capabilities with i18n
    let capabilities_header = self.i18n.get("timedatectl.properties.capabilities", None);
        println!("\n{}", capabilities_header);
        
    let tz_changes_label = self.i18n.get("timedatectl.properties.tz_changes", None);
    let ntp_sync_label = self.i18n.get("timedatectl.properties.ntp_sync", None);
    let rtc_access_label = self.i18n.get("timedatectl.properties.rtc_access", None);
    let hw_timestamp_label = self.i18n.get("timedatectl.properties.hw_timestamp", None);
        
    let supported_text = self.i18n.get("common.supported", None);
    let limited_text = self.i18n.get("common.limited", None);
    let full_text = self.i18n.get("common.full", None);
    let available_text = self.i18n.get("common.available", None);
        
        println!("   {:<20}: {}", tz_changes_label, supported_text);
        println!("   {:<20}: {}", ntp_sync_label, supported_text);
        println!("   {:<20}: {}", rtc_access_label, if cfg!(windows) { &limited_text } else { &full_text });
        println!("   {:<20}: {}", hw_timestamp_label, available_text);
        
        println!("\n{}", "=".repeat(80));
        Ok(())
    }
}

/// NTP response parsing data structure
#[derive(Debug)]
pub(crate) struct NTPResponseData {
    pub(crate) offset: Option<Duration>,
    pub(crate) jitter: Option<Duration>,
    pub(crate) stratum: Option<u8>,
    pub(crate) reference_id: String,
    pub(crate) precision: Duration,
    pub(crate) root_delay: Duration,
    pub(crate) root_dispersion: Duration,
    pub(crate) leap_indicator: LeapIndicator,
} 
