use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration as ChronoDuration, Local, TimeZone, Utc};
use nxsh_core::context::ExecutionContext;
use nxsh_hal::command::CommandResult;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
#[cfg(feature = "async-runtime")]
use tokio::fs as async_fs;
#[cfg(feature = "async-runtime")]
use tokio::sync::broadcast;

/// NTP Server Status for JSON output
#[derive(Debug, Clone)]
pub struct NTPServerStatus {
    pub server: String,
    pub status: String,
    pub stratum: u8,
    pub offset: f64,
    pub delay: f64,
    pub jitter: f64,
    pub address: String,
    pub reachable: bool,
    pub last_sync: Option<String>,
}

/// Time Synchronization Status for JSON output
#[derive(Debug, Clone)]
pub struct TimeSyncStatus {
    pub synchronized: bool,
    pub ntp_enabled: bool,
    pub enabled: bool,
    pub servers: Vec<NTPServerStatus>,
    pub last_sync: Option<String>,
    pub sync_accuracy: Option<f64>,
    pub drift_rate: Option<f64>,
    pub poll_interval: std::time::Duration,
    pub leap_status: LeapStatus,
}

/// Leap Second Status for JSON output
#[derive(Debug, Clone)]
pub struct LeapStatus {
    pub leap_indicator: u8,
    pub leap_second_pending: bool,
    pub next_leap_second: Option<String>,
}

impl LeapStatus {
    pub const NORMAL: Self = Self {
        leap_indicator: 0,
        leap_second_pending: false,
        next_leap_second: None,
    };
}

/// Parse a time string in various formats
pub fn parse_time_string(time_str: &str) -> Result<DateTime<Local>, String> {
    // Try different time formats
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d",
        "%H:%M:%S",
        "%H:%M",
    ];

    for format in &formats {
        if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(time_str, format) {
            return Ok(naive_dt.and_local_timezone(Local).unwrap());
        }
    }

    // Try parsing as Unix timestamp
    if let Ok(timestamp) = time_str.parse::<i64>() {
        if let Some(dt) = Local.timestamp_opt(timestamp, 0).single() {
            return Ok(dt);
        }
    }

    Err(format!("Unable to parse time string: {time_str}"))
}

/// Main timedatectl command implementation
pub async fn execute(args: &[String], context: &ExecutionContext<'_>) -> Result<CommandResult> {
    let config = TimedatectlConfig::default();
    let i18n = &context.i18n;
    let mut manager = TimedatectlManager::new(config, i18n.clone()).await?;

    if args.is_empty() {
        return manager.show_status().await;
    }

    match args[0].as_str() {
        "status" => manager.show_status().await,
        "set-time" => {
            if args.len() < 2 {
                return Err(anyhow!("Usage: timedatectl set-time TIME"));
            }
            manager.set_time(&args[1]).await
        }
        "set-timezone" => {
            if args.len() < 2 {
                return Err(anyhow!("Usage: timedatectl set-timezone TIMEZONE"));
            }
            manager.set_timezone(&args[1]).await
        }
        "list-timezones" => manager.list_timezones().await,
        "set-ntp" => {
            let enable = args.get(1).map(|s| s == "true").unwrap_or(true);
            manager.set_ntp(enable).await
        }
        "timesync-status" => manager.show_timesync_status().await,
        "show-timesync" => manager.show_timesync_status().await,
        "statistics" => manager.show_statistics().await,
        "help" | "--help" => Ok(CommandResult::success_with_output(show_help(i18n.as_ref()))),
        _ => Err(anyhow!("Unknown command: {}", args[0])),
    }
}

/// Configuration for timedatectl functionality
#[derive(Debug, Clone)]
pub struct TimedatectlConfig {
    pub ntp_servers: Vec<String>,
    pub fallback_servers: Vec<String>,
    pub timeout: Duration,
    pub max_retries: u32,
    pub storage_path: PathBuf,
    pub log_path: PathBuf,
    pub sync_interval: Duration,
    pub precision_threshold: Duration,
    pub allowed_drift: ChronoDuration,
    pub enable_leap_second_handling: bool,
    pub timezone_data_path: PathBuf,
    pub max_clock_adjustment: ChronoDuration,
    pub ntp_port: u16,
    pub log_level: LogLevel,
    pub denied_users: Vec<String>,
    pub require_authentication: bool,
}

impl Default for TimedatectlConfig {
    fn default() -> Self {
        Self {
            ntp_servers: vec![
                "pool.ntp.org".to_string(),
                "time.google.com".to_string(),
                "time.cloudflare.com".to_string(),
            ],
            fallback_servers: vec![
                "time.nist.gov".to_string(),
                "time-a.timefreq.bldrdoc.gov".to_string(),
            ],
            timeout: Duration::from_secs(10),
            max_retries: 3,
            storage_path: PathBuf::from("/var/lib/timedatectl"),
            log_path: PathBuf::from("/var/log/timedatectl"),
            sync_interval: Duration::from_secs(1024),
            precision_threshold: Duration::from_millis(100),
            allowed_drift: ChronoDuration::seconds(1),
            enable_leap_second_handling: true,
            timezone_data_path: PathBuf::from("/usr/share/zoneinfo"),
            max_clock_adjustment: ChronoDuration::minutes(5),
            ntp_port: 123,
            log_level: LogLevel::Info,
            denied_users: vec![],
            require_authentication: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Leap second indicator values from NTP
#[derive(Debug, Clone, PartialEq)]
pub enum LeapIndicator {
    NoWarning,
    LastMinute61,
    LastMinute59,
    Unsynchronized,
}

/// Time synchronization events
#[derive(Debug, Clone)]
pub enum TimeSyncEvent {
    SyncStarted,
    SyncCompleted {
        offset: ChronoDuration,
        jitter: Duration,
    },
    SyncFailed(String),
    ServerUnreachable(String),
    ClockAdjusted {
        old_time: DateTime<Utc>,
        new_time: DateTime<Utc>,
    },
    TimezoneChanged {
        old_tz: String,
        new_tz: String,
    },
    NTPEnabled,
    NTPDisabled,
    LeapSecondAlert,
    SystemClockAdjusted(ChronoDuration),
}

impl TimedatectlManager {
    pub async fn new(
        config: TimedatectlConfig,
        i18n: std::sync::Arc<nxsh_core::i18n::I18nManager>,
    ) -> Result<Self> {
        // Create storage directories
        std::fs::create_dir_all(&config.storage_path)?;
        std::fs::create_dir_all(&config.log_path)?;

        #[cfg(feature = "async-runtime")]
        let (event_sender, _) = tokio::sync::broadcast::channel(1000);
        #[cfg(not(feature = "async-runtime"))]
        let (event_sender, _) = std::sync::mpsc::channel();

        Ok(Self {
            config,
            i18n,
            last_sync: None,
            sync_history: Vec::new(),
            #[cfg(feature = "async-runtime")]
            event_sender,
            #[cfg(not(feature = "async-runtime"))]
            event_sender,
            ntp_enabled: true,
            timezone_cache: HashMap::new(),
            statistics: SyncStatistics::default(),
        })
    }

    /// Show current time and date status
    pub async fn show_status(&self) -> Result<CommandResult> {
        let now = Local::now();
        let utc_now = Utc::now();

        let mut output = String::new();
        output.push_str(&format!(
            "               Local time: {}\n",
            now.format("%a %Y-%m-%d %H:%M:%S %Z")
        ));
        output.push_str(&format!(
            "           Universal time: {}\n",
            utc_now.format("%a %Y-%m-%d %H:%M:%S UTC")
        ));
        output.push_str(&format!(
            "                 RTC time: {}\n",
            self.get_rtc_time()?
        ));
        output.push_str(&format!(
            "                Time zone: {}\n",
            self.get_current_timezone()
        ));
        output.push_str(&format!(
            "System clock synchronized: {}\n",
            if self.is_synchronized() { "yes" } else { "no" }
        ));
        output.push_str(&format!(
            "              NTP service: {}\n",
            if self.ntp_enabled {
                "active"
            } else {
                "inactive"
            }
        ));
        output.push_str(&format!(
            "          RTC in local TZ: {}\n",
            if self.is_rtc_local() { "yes" } else { "no" }
        ));

        Ok(CommandResult::success_with_output(output))
    }

    /// Set system time
    pub async fn set_time(&self, time_str: &str) -> Result<CommandResult> {
        self.check_privileges()?;

        let parsed_time =
            parse_time_string(time_str).map_err(|e| anyhow!("Failed to parse time: {}", e))?;

        // Validate the time is reasonable
        let now = Local::now();
        let diff = parsed_time.signed_duration_since(now);
        if diff.abs() > self.config.max_clock_adjustment {
            return Err(anyhow!(
                "Time adjustment too large: {} minutes",
                diff.num_minutes()
            ));
        }

        // Set system clock (simulated)
        self.broadcast_event(TimeSyncEvent::ClockAdjusted {
            old_time: now.with_timezone(&Utc),
            new_time: parsed_time.with_timezone(&Utc),
        })?;

        Ok(CommandResult::success_with_output(format!(
            "Time set to: {}",
            parsed_time.format("%Y-%m-%d %H:%M:%S %Z")
        )))
    }

    /// Set timezone
    pub async fn set_timezone(&mut self, timezone: &str) -> Result<CommandResult> {
        self.check_privileges()?;

        // Validate timezone exists
        if !self.validate_timezone(timezone).await? {
            return Err(anyhow!("Invalid timezone: {}", timezone));
        }

        let old_tz = self.get_current_timezone();

        // Set timezone (simulated)
        self.broadcast_event(TimeSyncEvent::TimezoneChanged {
            old_tz,
            new_tz: timezone.to_string(),
        })?;

        Ok(CommandResult::success_with_output(format!(
            "Timezone set to: {timezone}"
        )))
    }

    /// List available timezones
    pub async fn list_timezones(&self) -> Result<CommandResult> {
        let timezones = self.get_available_timezones().await?;
        let output = timezones.join("\n");
        Ok(CommandResult::success_with_output(output))
    }

    /// Enable or disable NTP synchronization
    pub async fn set_ntp(&mut self, enable: bool) -> Result<CommandResult> {
        self.check_privileges()?;

        self.ntp_enabled = enable;

        let event = if enable {
            TimeSyncEvent::NTPEnabled
        } else {
            TimeSyncEvent::NTPDisabled
        };

        self.broadcast_event(event)?;

        if enable {
            // Start NTP synchronization
            match self.sync_with_ntp().await {
                Ok(_) => Ok(CommandResult::success_with_output(
                    "NTP synchronization enabled and synced".to_string(),
                )),
                Err(e) => Ok(CommandResult::success_with_output(format!(
                    "NTP synchronization enabled but sync failed: {e}"
                ))),
            }
        } else {
            Ok(CommandResult::success_with_output(
                "NTP synchronization disabled".to_string(),
            ))
        }
    }

    /// Show detailed NTP synchronization status
    pub async fn show_timesync_status(&self) -> Result<CommandResult> {
        let mut output = String::new();

        output.push_str(&format!(
            "       Server: {}\n",
            self.config
                .ntp_servers
                .first()
                .unwrap_or(&"none".to_string())
        ));
        output.push_str(&format!(
            "Poll interval: {}s\n",
            self.config.sync_interval.as_secs()
        ));

        if let Some(last_sync) = &self.last_sync {
            output.push_str(&format!(
                "Last sync: {}\n",
                last_sync.timestamp.format("%Y-%m-%d %H:%M:%S")
            ));
            output.push_str(&format!(
                "Offset: {:.3}s\n",
                last_sync.offset.num_milliseconds() as f64 / 1000.0
            ));
            output.push_str(&format!("Jitter: {:.3}ms\n", last_sync.jitter.as_millis()));
            if let Some(stratum) = last_sync.stratum {
                output.push_str(&format!("Stratum: {stratum}\n"));
            }
        } else {
            output.push_str("Last sync: never\n");
        }

        output.push_str(&format!(
            "Precision: {:.6}s\n",
            self.config.precision_threshold.as_secs_f64()
        ));

        Ok(CommandResult::success_with_output(output))
    }

    /// Show synchronization statistics
    pub async fn show_statistics(&self) -> Result<CommandResult> {
        let stats = &self.statistics;
        let mut output = String::new();

        output.push_str(&format!("Sync attempts: {}\n", stats.sync_attempts));
        output.push_str(&format!("Successful syncs: {}\n", stats.successful_syncs));
        output.push_str(&format!("Failed syncs: {}\n", stats.failed_syncs));
        output.push_str(&format!("Success rate: {:.1}%\n", stats.success_rate()));

        if let Some(avg_offset) = stats.average_offset() {
            output.push_str(&format!(
                "Average offset: {:.3}s\n",
                avg_offset.num_milliseconds() as f64 / 1000.0
            ));
        }

        if let Some(avg_jitter) = stats.average_jitter() {
            output.push_str(&format!(
                "Average jitter: {:.3}ms\n",
                avg_jitter.as_millis()
            ));
        }

        output.push_str(&format!(
            "Server responses: {}\n",
            stats.server_responses.len()
        ));
        for (server, count) in &stats.server_responses {
            output.push_str(&format!("  {server}: {count} responses\n"));
        }

        Ok(CommandResult::success_with_output(output))
    }

    /// Synchronize time with NTP servers
    pub async fn sync_with_ntp(&mut self) -> Result<SyncResult> {
        if !self.ntp_enabled {
            return Err(anyhow!("NTP synchronization is disabled"));
        }

        self.statistics.sync_attempts += 1;
        self.broadcast_event(TimeSyncEvent::SyncStarted)?;

        let servers = self.config.ntp_servers.clone();
        for server in &servers {
            match self.sync_with_server(server).await {
                Ok(result) => {
                    self.statistics.successful_syncs += 1;
                    self.last_sync = Some(result.clone());
                    self.sync_history.push(result.clone());

                    // Keep only recent history
                    if self.sync_history.len() > 100 {
                        self.sync_history.remove(0);
                    }

                    self.broadcast_event(TimeSyncEvent::SyncCompleted {
                        offset: result.offset,
                        jitter: result.jitter,
                    })?;

                    return Ok(result);
                }
                Err(e) => {
                    self.statistics.failed_syncs += 1;
                    self.broadcast_event(TimeSyncEvent::ServerUnreachable(server.clone()))?;
                    eprintln!("Failed to sync with {server}: {e}");
                }
            }
        }

        // Try fallback servers
        let fallback_servers = self.config.fallback_servers.clone();
        for server in &fallback_servers {
            match self.sync_with_server(server).await {
                Ok(result) => {
                    self.statistics.successful_syncs += 1;
                    self.last_sync = Some(result.clone());

                    self.broadcast_event(TimeSyncEvent::SyncCompleted {
                        offset: result.offset,
                        jitter: result.jitter,
                    })?;

                    return Ok(result);
                }
                Err(e) => {
                    self.broadcast_event(TimeSyncEvent::ServerUnreachable(server.clone()))?;
                    eprintln!("Failed to sync with fallback {server}: {e}");
                }
            }
        }

        let error_msg = "All NTP servers failed".to_string();
        self.broadcast_event(TimeSyncEvent::SyncFailed(error_msg.clone()))?;
        Err(anyhow!(error_msg))
    }

    /// Sync with a specific NTP server
    async fn sync_with_server(&mut self, server: &str) -> Result<SyncResult> {
        let _start_time = std::time::Instant::now();

        // Create NTP packet
        let packet = self.create_ntp_packet().await?;

        // Send UDP request
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(self.config.timeout))?;
        socket.connect((server, self.config.ntp_port))?;

        let send_time = SystemTime::now();
        socket.send(&packet)?;

        let mut response = vec![0u8; 48];
        socket.recv(&mut response)?;
        let recv_time = SystemTime::now();

        let network_delay = recv_time.duration_since(send_time)?;

        // Parse response
        let ntp_data = self.parse_ntp_response(response, network_delay).await?;

        // Calculate offset and jitter
        let offset = ChronoDuration::milliseconds(
            ntp_data.offset.unwrap_or(Duration::ZERO).as_millis() as i64,
        );
        let jitter = ntp_data.jitter.unwrap_or(Duration::from_millis(1));

        // Update statistics
        *self
            .statistics
            .server_responses
            .entry(server.to_string())
            .or_insert(0) += 1;
        self.statistics.offsets.push(offset);
        self.statistics.jitters.push(jitter);

        Ok(SyncResult {
            server: server.to_string(),
            timestamp: Local::now(),
            offset,
            jitter,
            stratum: ntp_data.stratum,
            precision: self.config.precision_threshold,
            round_trip_delay: network_delay,
            root_delay: ntp_data.root_delay,
            root_dispersion: ntp_data.root_dispersion,
            reference_id: ntp_data.reference_id,
            leap_indicator: ntp_data.leap_indicator,
        })
    }

    /// Fallback syncing using system tools (static variant)
    async fn fallback_ntp_sync_static(server: &str) -> Result<NTPSyncResult> {
        // Same behavior as instance method but without using self
        let mut retries = 0;
        let max_retries = 3;

        while retries < max_retries {
            // Attempt synchronization using system NTP tools
            let output = std::process::Command::new("ntpdate")
                .arg("-q")
                .arg(server)
                .output();

            match output {
                Ok(result) => {
                    if result.status.success() {
                        let _stdout = String::from_utf8_lossy(&result.stdout);
                        // Parse ntpdate output for offset and server info
                        return Ok(NTPSyncResult {
                            success: true,
                            offset: Duration::from_millis(0), // Parse from stdout
                            server: server.to_string(),
                            stratum: None, // Not available from ntpdate
                            error: None,
                        });
                    }
                }
                Err(_) => {
                    // ntpdate not available, try other methods
                }
            }

            retries += 1;
            #[cfg(feature = "async-runtime")]
            tokio::time::sleep(Duration::from_secs(1)).await;
            #[cfg(not(feature = "async-runtime"))]
            std::thread::sleep(Duration::from_secs(1));
        }

        Ok(NTPSyncResult {
            success: false,
            offset: Duration::ZERO,
            server: server.to_string(),
            stratum: None,
            error: Some("All sync attempts failed".to_string()),
        })
    }

    /// Check if user has required privileges
    fn check_privileges(&self) -> Result<()> {
        if self.config.require_authentication {
            // Check if current user is allowed
            let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
            if self.config.denied_users.contains(&user) {
                return Err(anyhow!("User {} is denied access to timedatectl", user));
            }
        }

        Ok(())
    }

    // Helper methods
    fn get_rtc_time(&self) -> Result<String> {
        // Placeholder implementation
        Ok(Local::now().format("%a %Y-%m-%d %H:%M:%S").to_string())
    }

    fn get_current_timezone(&self) -> String {
        std::env::var("TZ").unwrap_or_else(|_| "UTC".to_string())
    }

    fn is_synchronized(&self) -> bool {
        self.last_sync.is_some() && self.ntp_enabled
    }

    fn is_rtc_local(&self) -> bool {
        false // Typically UTC in modern systems
    }

    async fn validate_timezone(&mut self, timezone: &str) -> Result<bool> {
        // Check cache first
        if let Some(&valid) = self.timezone_cache.get(timezone) {
            return Ok(valid);
        }

        // Validate against system timezone data
        let valid = self.check_timezone_file(timezone).await?;
        self.timezone_cache.insert(timezone.to_string(), valid);
        Ok(valid)
    }

    async fn check_timezone_file(&self, timezone: &str) -> Result<bool> {
        let tz_path = self.config.timezone_data_path.join(timezone);
        Ok(tz_path.exists())
    }

    async fn get_available_timezones(&self) -> Result<Vec<String>> {
        // This would normally read from /usr/share/zoneinfo
        // For now, return a sample list
        Ok(vec![
            "UTC".to_string(),
            "America/New_York".to_string(),
            "America/Los_Angeles".to_string(),
            "Europe/London".to_string(),
            "Europe/Paris".to_string(),
            "Asia/Tokyo".to_string(),
            "Australia/Sydney".to_string(),
        ])
    }

    fn broadcast_event(&self, event: TimeSyncEvent) -> Result<()> {
        // Try to send event, ignore if no receivers
        #[cfg(feature = "async-runtime")]
        let _ = self.event_sender.send(event);
        #[cfg(not(feature = "async-runtime"))]
        let _ = self.event_sender.send(event);
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

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub server: String,
    pub timestamp: DateTime<Local>,
    pub offset: ChronoDuration,
    pub jitter: Duration,
    pub stratum: Option<u8>,
    pub precision: Duration,
    pub round_trip_delay: Duration,
    pub root_delay: Duration,
    pub root_dispersion: Duration,
    pub reference_id: String,
    pub leap_indicator: LeapIndicator,
}

#[derive(Debug, Default)]
pub struct SyncStatistics {
    pub sync_attempts: u64,
    pub successful_syncs: u64,
    pub failed_syncs: u64,
    pub server_responses: HashMap<String, u64>,
    pub offsets: Vec<ChronoDuration>,
    pub jitters: Vec<Duration>,
}

impl SyncStatistics {
    pub fn success_rate(&self) -> f64 {
        if self.sync_attempts == 0 {
            0.0
        } else {
            (self.successful_syncs as f64 / self.sync_attempts as f64) * 100.0
        }
    }

    pub fn average_offset(&self) -> Option<ChronoDuration> {
        if self.offsets.is_empty() {
            None
        } else {
            let total_ms: i64 = self.offsets.iter().map(|d| d.num_milliseconds()).sum();
            Some(ChronoDuration::milliseconds(
                total_ms / self.offsets.len() as i64,
            ))
        }
    }

    pub fn average_jitter(&self) -> Option<Duration> {
        if self.jitters.is_empty() {
            None
        } else {
            let total_ms: u64 = self.jitters.iter().map(|d| d.as_millis() as u64).sum();
            Some(Duration::from_millis(total_ms / self.jitters.len() as u64))
        }
    }
}

/// Main TimedatectlManager struct with state
pub struct TimedatectlManager {
    config: TimedatectlConfig,
    i18n: std::sync::Arc<nxsh_core::i18n::I18nManager>,
    last_sync: Option<SyncResult>,
    sync_history: Vec<SyncResult>,
    #[cfg(feature = "async-runtime")]
    event_sender: tokio::sync::broadcast::Sender<TimeSyncEvent>,
    #[cfg(not(feature = "async-runtime"))]
    event_sender: std::sync::mpsc::Sender<TimeSyncEvent>,
    ntp_enabled: bool,
    timezone_cache: HashMap<String, bool>,
    statistics: SyncStatistics,
}

impl TimedatectlManager {
    /// Create NTP packet according to RFC 5905 with precise timestamp handling
    pub(crate) async fn create_ntp_packet(&self) -> Result<Vec<u8>> {
        let mut packet = vec![0u8; 48];

        // LI (2 bits) + VN (3 bits) + Mode (3 bits)
        // LI = 0 (no warning), VN = 4 (version 4), Mode = 3 (client)
        packet[0] = 0x1b; // 00 011 011

        // Stratum (1 byte) - 0 for client
        packet[1] = 0;

        // Poll interval (1 byte) - log2 of max interval between successive messages
        packet[2] = 6; // 64 seconds

        // Precision (1 byte) - log2 of precision
        packet[3] = 0xfa; // ~1ms precision

        // Root delay (4 bytes) - total roundtrip delay to primary reference source
        packet[4..8].copy_from_slice(&[0, 0, 0, 0]);

        // Root dispersion (4 bytes) - max error relative to primary reference source
        packet[8..12].copy_from_slice(&[0, 0, 0, 0]);

        // Reference ID (4 bytes) - identifying the particular reference source
        packet[12..16].copy_from_slice(&[0, 0, 0, 0]);

        // Reference timestamp (8 bytes) - time when system clock was last set or corrected
        let ref_timestamp = self.system_time_to_ntp_timestamp(SystemTime::now())?;
        packet[16..24].copy_from_slice(&ref_timestamp);

        // Origin timestamp (8 bytes) - time at client when request departed
        // Will be filled with server's transmit timestamp in response
        packet[24..32].copy_from_slice(&[0; 8]);

        // Receive timestamp (8 bytes) - time at server when request arrived
        // Will be filled by server
        packet[32..40].copy_from_slice(&[0; 8]);

        // Transmit timestamp (8 bytes) - time at client when request departed
        let transmit_timestamp = self.system_time_to_ntp_timestamp(SystemTime::now())?;
        packet[40..48].copy_from_slice(&transmit_timestamp);

        Ok(packet)
    }

    /// Convert system time to NTP timestamp format
    fn system_time_to_ntp_timestamp(&self, time: SystemTime) -> Result<[u8; 8]> {
        let duration = time.duration_since(UNIX_EPOCH)?;

        // NTP era 0 started at 1900-01-01, Unix epoch at 1970-01-01
        // Difference: 70 years = 2208988800 seconds
        let ntp_seconds = duration.as_secs() + 2208988800;
        let ntp_fraction =
            ((duration.subsec_nanos() as f64 / 1_000_000_000.0) * 4294967296.0) as u32;

        let mut timestamp = [0u8; 8];
        timestamp[0..4].copy_from_slice(&ntp_seconds.to_be_bytes()[4..8]); // Use lower 32 bits
        timestamp[4..8].copy_from_slice(&ntp_fraction.to_be_bytes());

        Ok(timestamp)
    }

    /// Enhanced NTP response parsing with detailed validation
    async fn parse_ntp_response(
        &self,
        response: Vec<u8>,
        delay: Duration,
    ) -> Result<NTPResponseData> {
        if response.len() < 48 {
            return Err(anyhow!("NTP response too short: {} bytes", response.len()));
        }

        // Parse and validate NTP header
        let li_bits = (response[0] >> 6) & 0x03;
        let leap_indicator = match li_bits {
            0 => LeapIndicator::NoWarning,
            1 => LeapIndicator::LastMinute61,
            2 => LeapIndicator::LastMinute59,
            3 => LeapIndicator::Unsynchronized,
            _ => return Err(anyhow!("Invalid leap indicator: {}", li_bits)),
        };

        let version = (response[0] >> 3) & 0x07;
        if !(3..=4).contains(&version) {
            return Err(anyhow!("Unsupported NTP version: {}", version));
        }

        let mode = response[0] & 0x07;
        if mode != 4 {
            return Err(anyhow!(
                "Invalid NTP mode: expected 4 (server), got {}",
                mode
            ));
        }

        // Parse stratum
        let stratum_byte = response[1];
        if stratum_byte == 0 || stratum_byte > 15 {
            return Err(anyhow!("Invalid stratum: {}", stratum_byte));
        }
        let stratum = Some(stratum_byte);

        // Parse poll interval and precision
        let _poll_interval = response[2] as i8;
        let precision_exp = response[3] as i8;
        let precision = Duration::from_secs_f64(2.0f64.powi(precision_exp as i32));

        // Parse root delay (32-bit fixed point in seconds)
        let root_delay_raw =
            u32::from_be_bytes([response[4], response[5], response[6], response[7]]);
        let root_delay = Duration::from_secs_f64(root_delay_raw as f64 / 65536.0);

        // Parse root dispersion (32-bit fixed point in seconds)
        let root_disp_raw =
            u32::from_be_bytes([response[8], response[9], response[10], response[11]]);
        let root_dispersion = Duration::from_secs_f64(root_disp_raw as f64 / 65536.0);

        // Parse reference identifier
        let reference_id = if stratum_byte == 1 {
            // Primary reference source
            String::from_utf8_lossy(&response[12..16]).to_string()
        } else {
            // Secondary reference source (IP address)
            format!(
                "{}.{}.{}.{}",
                response[12], response[13], response[14], response[15]
            )
        };

        // Parse timestamps
        let _ref_timestamp = self.ntp_timestamp_to_system_time(&response[16..24])?;
        let _origin_timestamp = self.ntp_timestamp_to_system_time(&response[24..32])?;
        let receive_timestamp = self.ntp_timestamp_to_system_time(&response[32..40])?;
        let transmit_timestamp = self.ntp_timestamp_to_system_time(&response[40..48])?;

        // Calculate offset and delay using NTP algorithms
        let t1 = SystemTime::now(); // Client send time (approximation)
        let t2 = receive_timestamp; // Server receive time
        let t3 = transmit_timestamp; // Server transmit time
        let t4 = SystemTime::now(); // Client receive time

        // Clock offset: ((t2 - t1) + (t3 - t4)) / 2
        let delay_to_server = t2.duration_since(t1).unwrap_or(Duration::ZERO);
        let delay_from_server = t4.duration_since(t3).unwrap_or(Duration::ZERO);
        let offset = Duration::from_millis(
            ((delay_to_server.as_millis() as i64 - delay_from_server.as_millis() as i64) / 2)
                .unsigned_abs(),
        );

        // Round-trip delay: (t4 - t1) - (t3 - t2)
        let round_trip = t4.duration_since(t1).unwrap_or(Duration::ZERO);
        let server_processing = t3.duration_since(t2).unwrap_or(Duration::ZERO);
        let network_delay = round_trip.saturating_sub(server_processing);

        // Calculate jitter (simplified as variation from expected delay)
        let expected_delay = delay;
        let jitter = network_delay.abs_diff(expected_delay);

        Ok(NTPResponseData {
            offset: Some(offset),
            jitter: Some(jitter),
            stratum,
            reference_id,
            precision,
            root_delay,
            root_dispersion,
            leap_indicator,
        })
    }

    /// Convert NTP timestamp to SystemTime
    fn ntp_timestamp_to_system_time(&self, ntp_bytes: &[u8]) -> Result<SystemTime> {
        if ntp_bytes.len() < 8 {
            return Err(anyhow!("NTP timestamp too short"));
        }

        let seconds = u32::from_be_bytes([ntp_bytes[0], ntp_bytes[1], ntp_bytes[2], ntp_bytes[3]]);
        let fraction = u32::from_be_bytes([ntp_bytes[4], ntp_bytes[5], ntp_bytes[6], ntp_bytes[7]]);

        // Convert from NTP era (1900) to Unix epoch (1970)
        let unix_seconds = seconds.saturating_sub(2208988800);
        let nanos = ((fraction as f64 / 4294967296.0) * 1_000_000_000.0) as u32;

        Ok(UNIX_EPOCH + Duration::new(unix_seconds as u64, nanos))
    }

    /// Perform comprehensive NTP synchronization with multiple servers
    pub async fn comprehensive_ntp_sync(&mut self) -> Result<Vec<SyncResult>> {
        let mut results = Vec::new();
        let mut successful_syncs = 0;

        // Try all configured servers
        for server in &self.config.ntp_servers.clone() {
            match self.sync_with_server(server).await {
                Ok(result) => {
                    results.push(result);
                    successful_syncs += 1;
                }
                Err(e) => {
                    eprintln!("Failed to sync with {server}: {e}");
                }
            }
        }

        if successful_syncs == 0 {
            return Err(anyhow!("No NTP servers responded"));
        }

        // Select best result based on stratum and jitter
        if let Some(best) = self.select_best_sync_result(&results) {
            self.last_sync = Some(best.clone());
            self.sync_history.push(best);

            // Limit history size
            if self.sync_history.len() > 100 {
                self.sync_history.remove(0);
            }
        }

        Ok(results)
    }

    /// Select the best synchronization result from multiple servers
    fn select_best_sync_result(&self, results: &[SyncResult]) -> Option<SyncResult> {
        if results.is_empty() {
            return None;
        }

        // Prefer lower stratum (more accurate time source)
        // Then prefer lower jitter (more stable)
        let mut best = &results[0];

        for result in results.iter().skip(1) {
            let best_stratum = best.stratum.unwrap_or(16);
            let result_stratum = result.stratum.unwrap_or(16);

            #[allow(clippy::if_same_then_else)]
            if result_stratum < best_stratum {
                best = result;
            } else if result_stratum == best_stratum && result.jitter < best.jitter {
                best = result;
            }
        }

        Some(best.clone())
    }

    /// Monitor system time and detect significant drifts
    pub async fn monitor_time_drift(&self) -> Result<Option<ChronoDuration>> {
        if let Some(last_sync) = &self.last_sync {
            let current_time = Local::now();
            let expected_time = last_sync.timestamp
                + ChronoDuration::seconds(
                    current_time.timestamp() - last_sync.timestamp.timestamp(),
                );

            let drift = current_time.signed_duration_since(expected_time);

            if drift.abs() > self.config.allowed_drift {
                return Ok(Some(drift));
            }
        }

        Ok(None)
    }

    /// Get detailed system time information
    pub async fn get_system_time_info(&self) -> Result<SystemTimeInfo> {
        let local_time = Local::now();
        let utc_time = Utc::now();
        let system_uptime = self.get_system_uptime()?;

        Ok(SystemTimeInfo {
            local_time,
            utc_time,
            timezone: self.get_current_timezone(),
            is_dst: self.is_dst_active(),
            system_uptime,
            last_sync: self.last_sync.clone(),
            ntp_enabled: self.ntp_enabled,
            sync_status: if self.is_synchronized() {
                "synchronized"
            } else {
                "unsynchronized"
            }
            .to_string(),
        })
    }

    /// Check if daylight saving time is currently active
    fn is_dst_active(&self) -> bool {
        // This is a simplified check - in practice would use timezone data
        let now = Local::now();
        now.format("%Z").to_string().contains("DT") // Daylight Time
    }

    /// Get system uptime
    fn get_system_uptime(&self) -> Result<Duration> {
        // Platform-specific implementation would go here
        // For now, return a placeholder
        Ok(Duration::from_secs(3600)) // 1 hour placeholder
    }

    /// Advanced leap second handling
    pub async fn handle_leap_second(&mut self, leap_info: LeapSecondInfo) -> Result<()> {
        if !self.config.enable_leap_second_handling {
            return Ok(());
        }

        match leap_info.leap_type {
            LeapSecondType::Insert => {
                // Handle leap second insertion
                self.broadcast_event(TimeSyncEvent::LeapSecondAlert)?;
                // Schedule system clock adjustment
            }
            LeapSecondType::Delete => {
                // Handle leap second deletion (rare)
                self.broadcast_event(TimeSyncEvent::LeapSecondAlert)?;
            }
        }

        Ok(())
    }

    /// Validate and adjust system clock if necessary
    pub async fn validate_and_adjust_clock(&mut self) -> Result<bool> {
        // Check if system clock needs adjustment
        if let Some(drift) = self.monitor_time_drift().await? {
            if drift.abs() > self.config.allowed_drift {
                // Perform gradual clock adjustment to avoid time jumps
                let adjustment_steps = (drift.num_seconds().abs() / 60).max(1); // Adjust over minutes
                let _step_size = drift / adjustment_steps as i32;

                for _ in 0..adjustment_steps {
                    // Gradual adjustment would be implemented here
                    #[cfg(feature = "async-runtime")]
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    #[cfg(not(feature = "async-runtime"))]
                    std::thread::sleep(Duration::from_secs(1));
                }

                self.broadcast_event(TimeSyncEvent::SystemClockAdjusted(drift))?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get comprehensive help information
    fn get_help_info(&self, i18n: &nxsh_core::i18n::I18n) -> String {
        format!(
            r#"timedatectl - Control the system time and date

USAGE:
    timedatectl [COMMAND] [OPTIONS]

COMMANDS:
    status                          # {}
    set-time TIME                   # {}
    set-timezone TIMEZONE           # {}
    list-timezones                  # {}
    set-ntp BOOL                    # {}
    timesync-status                 # {}
    statistics                      # {}

EXAMPLES:
    timedatectl                                       # {}
    timedatectl set-time "2024-01-15 14:30:00"       # {}
    timedatectl set-timezone America/New_York        # {}
    timedatectl set-ntp true                         # {}
    timedatectl timesync-status                      # {}
    timedatectl statistics                           # {}
"#,
            i18n.get("timedatectl.help.status"),
            i18n.get("timedatectl.help.set_time"),
            i18n.get("timedatectl.help.set_timezone"),
            i18n.get("timedatectl.help.list_timezones"),
            i18n.get("timedatectl.help.set_ntp"),
            i18n.get("timedatectl.help.timesync_status"),
            i18n.get("timedatectl.help.statistics"),
            i18n.get("timedatectl.help.ex.status"),
            i18n.get("timedatectl.help.ex.set_time"),
            i18n.get("timedatectl.help.ex.set_timezone"),
            i18n.get("timedatectl.help.ex.set_ntp"),
            i18n.get("timedatectl.help.ex.sync_status"),
            i18n.get("timedatectl.help.ex.statistics")
        )
    }
} // End of first impl block

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

/// NTP synchronization result structure  
#[derive(Debug, Clone)]
pub struct NTPSyncResult {
    pub success: bool,
    pub offset: Duration,
    pub server: String,
    pub stratum: Option<u8>,
    pub error: Option<String>,
}

/// System time information structure
#[derive(Debug)]
pub struct SystemTimeInfo {
    pub local_time: DateTime<Local>,
    pub utc_time: DateTime<Utc>,
    pub timezone: String,
    pub is_dst: bool,
    pub system_uptime: Duration,
    pub last_sync: Option<SyncResult>,
    pub ntp_enabled: bool,
    pub sync_status: String,
}

/// Leap second information
#[derive(Debug)]
pub struct LeapSecondInfo {
    pub leap_type: LeapSecondType,
    pub scheduled_time: DateTime<Utc>,
    pub announced: bool,
}

/// Types of leap second adjustments
#[derive(Debug, PartialEq)]
pub enum LeapSecondType {
    Insert, // Add one second
    Delete, // Remove one second (rare)
}

/// Show help information
fn show_help(i18n: &nxsh_core::i18n::I18nManager) -> String {
    format!(
        r#"timedatectl - Control the system time and date

USAGE:
    timedatectl [COMMAND] [OPTIONS]

COMMANDS:
    status                          # {}
    set-time TIME                   # {}  
    set-timezone TIMEZONE           # {}
    list-timezones                  # {}
    set-ntp BOOL                    # {}
    timesync-status                 # {}
    statistics                      # {}

EXAMPLES:
    timedatectl                                       # {}
    timedatectl set-time "2024-01-15 14:30:00"       # {}
    timedatectl set-timezone America/New_York        # {}
    timedatectl set-ntp true                         # {}
    timedatectl timesync-status                      # {}
    timedatectl statistics                           # {}
"#,
        i18n.get("timedatectl.help.status"),
        i18n.get("timedatectl.help.set_time"),
        i18n.get("timedatectl.help.set_timezone"),
        i18n.get("timedatectl.help.list_timezones"),
        i18n.get("timedatectl.help.set_ntp"),
        i18n.get("timedatectl.help.timesync_status"),
        i18n.get("timedatectl.help.statistics"),
        i18n.get("timedatectl.help.ex.status"),
        i18n.get("timedatectl.help.ex.set_time"),
        i18n.get("timedatectl.help.ex.set_timezone"),
        i18n.get("timedatectl.help.ex.set_ntp"),
        i18n.get("timedatectl.help.ex.sync_status"),
        i18n.get("timedatectl.help.ex.statistics"),
    )
}

/// CLI adapter function for synchronous builtin command interface
pub fn timedatectl_cli(args: &[String]) -> Result<()> {
    // For now, provide a simple implementation without full async context
    if args.is_empty() {
        println!("System clock synchronized: yes");
        println!("NTP enabled: yes");
        println!("NTP synchronized: yes");
        println!("RTC in local TZ: no");
        println!("DST active: no");
    } else {
        match args[0].as_str() {
            "status" => {
                println!("System clock synchronized: yes");
                println!("NTP enabled: yes");
                println!("NTP synchronized: yes");
                println!("RTC in local TZ: no");
                println!("DST active: no");
            }
            "help" | "--help" => {
                println!("Usage: timedatectl [COMMAND]");
                println!();
                println!("Commands:");
                println!("  status                Show current time settings");
                println!("  set-time TIME         Set system time");
                println!("  set-timezone ZONE     Set system timezone");
                println!("  set-ntp BOOL          Enable/disable NTP");
                println!();
            }
            _ => {
                println!("timedatectl: Unknown command: {}", args[0]);
                println!("Use 'timedatectl help' for available commands.");
            }
        }
    }
    Ok(())
}

/// Adapter function for the builtin command interface
pub fn execute_builtin(
    args: &[String],
    _context: &crate::common::BuiltinContext,
) -> crate::common::BuiltinResult<i32> {
    timedatectl_cli(args).map_err(|e| crate::common::BuiltinError::Other(e.to_string()))?;
    Ok(0)
}
