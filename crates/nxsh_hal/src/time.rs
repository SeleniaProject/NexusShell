//! Time management abstraction layer
//!
//! This module provides platform-agnostic time and timing operations
//! with high precision and timezone support.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime as StdSystemTime, UNIX_EPOCH};

use crate::error::{HalError, HalResult};
use crate::platform::{Capabilities, Platform};
use chrono::{DateTime, Utc};

/// Time management and operations
#[derive(Debug)]
pub struct TimeManager {
    #[allow(dead_code)]
    platform: Platform,
    #[allow(dead_code)]
    capabilities: Capabilities,
}

impl TimeManager {
    pub fn new() -> HalResult<Self> {
        Ok(Self {
            platform: Platform::current(),
            capabilities: Capabilities::current(),
        })
    }

    pub fn now(&self) -> HalResult<SystemTime> {
        let std_time = StdSystemTime::now();
        Ok(SystemTime::from_std(std_time))
    }

    pub fn unix_timestamp(&self) -> HalResult<u64> {
        let std_time = StdSystemTime::now();
        let duration = std_time
            .duration_since(UNIX_EPOCH)
            .map_err(|e| HalError::invalid(&format!("System time error: {e}")))?;
        Ok(duration.as_secs())
    }

    pub fn format_time(&self, time: &SystemTime, format: &str) -> HalResult<String> {
        let datetime: DateTime<Utc> = time.to_std().into();
        Ok(datetime.format(format).to_string())
    }

    pub fn parse_time(&self, time_str: &str, format: &str) -> HalResult<SystemTime> {
        let datetime = DateTime::parse_from_str(time_str, format)
            .map_err(|e| HalError::invalid(&format!("Time parse error: {e}")))?;
        Ok(SystemTime::from_std(datetime.into()))
    }

    pub fn sleep(&self, duration: Duration) -> HalResult<()> {
        thread::sleep(duration);
        Ok(())
    }

    pub fn sleep_until(&self, deadline: SystemTime) -> HalResult<()> {
        let now = self.now()?;
        if deadline > now {
            let duration = deadline
                .duration_since(&now)
                .map_err(|e| HalError::invalid(&format!("Invalid duration: {e}")))?;
            thread::sleep(duration);
        }
        Ok(())
    }

    pub fn elapsed_since(&self, start: &SystemTime) -> HalResult<Duration> {
        let now = self.now()?;
        now.duration_since(start)
            .map_err(|e| HalError::invalid(&format!("Time calculation error: {e}")))
    }

    pub fn add_duration(&self, time: &SystemTime, duration: Duration) -> HalResult<SystemTime> {
        Ok(time.add(duration))
    }

    pub fn sub_duration(&self, time: &SystemTime, duration: Duration) -> HalResult<SystemTime> {
        time.sub(duration)
            .ok_or_else(|| HalError::invalid("Duration subtraction would result in negative time"))
    }

    pub fn high_precision_timer(&self) -> HalResult<HighPrecisionTimer> {
        Ok(HighPrecisionTimer::new())
    }

    pub fn system_uptime(&self) -> HalResult<Duration> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let uptime_str = fs::read_to_string("/proc/uptime")
                .map_err(|e| HalError::io_error("read_uptime", Some("/proc/uptime"), e))?;

            let uptime_secs: f64 = uptime_str
                .split_whitespace()
                .next()
                .ok_or_else(|| HalError::invalid("Invalid uptime format"))?
                .parse()
                .map_err(|_| HalError::invalid("Invalid uptime number"))?;

            Ok(Duration::from_secs_f64(uptime_secs))
        }
        #[cfg(target_os = "macos")]
        {
            // Use pure Rust implementation via /usr/bin/uptime parsing as a safe alternative
            use std::process::Command;

            match Command::new("uptime").output() {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    // Parse "up X days, Y:Z," format or similar
                    if let Some(up_pos) = output_str.find("up ") {
                        let after_up = &output_str[up_pos + 3..];

                        // Try to extract time information using regex-like parsing
                        let mut total_seconds = 0u64;

                        // Look for days
                        if let Some(days_end) = after_up.find(" day") {
                            if let Ok(days) = after_up[..days_end].trim().parse::<u64>() {
                                total_seconds += days * 24 * 3600;
                            }
                        }

                        // Look for hours:minutes pattern
                        if let Some(time_match) = after_up.find(char::is_numeric) {
                            let time_part = &after_up[time_match..];
                            if let Some(comma_pos) = time_part.find(',') {
                                let time_str = &time_part[..comma_pos].trim();
                                if let Some(colon_pos) = time_str.find(':') {
                                    if let (Ok(hours), Ok(minutes)) = (
                                        time_str[..colon_pos].parse::<u64>(),
                                        time_str[colon_pos + 1..].parse::<u64>(),
                                    ) {
                                        total_seconds += hours * 3600 + minutes * 60;
                                    }
                                }
                            }
                        }

                        if total_seconds > 0 {
                            return Ok(Duration::from_secs(total_seconds));
                        }
                    }
                    Err(HalError::invalid("Cannot parse uptime output"))
                }
                Err(_) => {
                    // Fallback: Use /proc/uptime if available (unlikely on macOS, but safe)
                    match std::fs::read_to_string("/proc/uptime") {
                        Ok(content) => {
                            let uptime_str = content
                                .split_whitespace()
                                .next()
                                .ok_or_else(|| HalError::invalid("Invalid uptime format"))?;
                            let uptime_secs: f64 = uptime_str
                                .parse()
                                .map_err(|_| HalError::invalid("Invalid uptime number"))?;
                            Ok(Duration::from_secs_f64(uptime_secs))
                        }
                        Err(_) => Err(HalError::unsupported(
                            "Cannot determine system uptime on this macOS version",
                        )),
                    }
                }
            }
        }
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::SystemInformation::GetTickCount64;
            let ticks = unsafe { GetTickCount64() };
            Ok(Duration::from_millis(ticks))
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        {
            Err(HalError::unsupported(
                "Uptime not supported on this platform",
            ))
        }
    }

    pub fn timezone_offset(&self) -> HalResult<i32> {
        #[cfg(unix)]
        {
            use chrono::Local;

            let local_time = Local::now();
            let offset = local_time.offset().local_minus_utc();

            Ok(offset)
        }
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Time::{GetTimeZoneInformation, TIME_ZONE_INFORMATION};

            let mut tzi = TIME_ZONE_INFORMATION {
                Bias: 0,
                StandardName: [0; 32],
                StandardDate: unsafe { std::mem::zeroed() },
                StandardBias: 0,
                DaylightName: [0; 32],
                DaylightDate: unsafe { std::mem::zeroed() },
                DaylightBias: 0,
            };

            let _result = unsafe { GetTimeZoneInformation(&mut tzi) };

            let offset_seconds = -(tzi.Bias * 60) as i32;

            Ok(offset_seconds)
        }
        #[cfg(not(any(unix, windows)))]
        {
            Ok(0) // Default to UTC offset
        }
    }

    pub fn is_leap_year(&self, year: i32) -> HalResult<bool> {
        Ok((year % 4 == 0 && year % 100 != 0) || (year % 400 == 0))
    }

    pub fn days_in_month(&self, year: i32, month: u32) -> HalResult<u32> {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => Ok(31),
            4 | 6 | 9 | 11 => Ok(30),
            2 => {
                if self.is_leap_year(year)? {
                    Ok(29)
                } else {
                    Ok(28)
                }
            }
            _ => Err(HalError::invalid("Invalid month")),
        }
    }

    pub fn format_duration(&self, duration: Duration) -> HalResult<String> {
        let hours = duration.as_secs() / 3600;
        let minutes = (duration.as_secs() % 3600) / 60;
        let seconds = duration.as_secs() % 60;
        let nanoseconds = duration.subsec_nanos();

        let mut parts = Vec::new();
        if hours > 0 {
            parts.push(format!("{hours}h"));
        }
        if minutes > 0 {
            parts.push(format!("{minutes}m"));
        }
        if seconds > 0 {
            parts.push(format!("{seconds}s"));
        }
        if nanoseconds > 0 {
            parts.push(format!("{nanoseconds}ns"));
        }

        Ok(parts.join(" "))
    }

    pub fn benchmark<F, R>(&self, f: F) -> HalResult<(R, Duration)>
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        Ok((result, elapsed))
    }

    pub fn get_monotonic_time(&self) -> HalResult<Duration> {
        // Use Instant for monotonic time
        static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
        let start = START_TIME.get_or_init(Instant::now);
        Ok(start.elapsed())
    }

    pub fn get_process_time(&self) -> HalResult<Duration> {
        #[cfg(unix)]
        {
            // Use pure Rust implementation via /proc/stat parsing as safe alternative to libc::getrusage
            match std::fs::read_to_string("/proc/self/stat") {
                Ok(stat_content) => {
                    let fields: Vec<&str> = stat_content.split_whitespace().collect();
                    if fields.len() >= 15 {
                        // Fields 13 and 14 are utime and stime in clock ticks
                        let utime_ticks: u64 = fields[13].parse().unwrap_or(0);
                        let stime_ticks: u64 = fields[14].parse().unwrap_or(0);

                        // Get clock ticks per second (usually 100)
                        let ticks_per_sec = 100u64; // Standard value, could also read from sysconf

                        let total_ticks = utime_ticks + stime_ticks;
                        let total_seconds = total_ticks / ticks_per_sec;
                        let remaining_ticks = total_ticks % ticks_per_sec;
                        let nanoseconds = (remaining_ticks * 1_000_000_000) / ticks_per_sec;

                        Ok(Duration::new(total_seconds, nanoseconds as u32))
                    } else {
                        Err(HalError::invalid("Invalid /proc/self/stat format"))
                    }
                }
                Err(_) => {
                    // Fallback: use simple process time estimation via current time
                    // This is not as accurate but avoids C dependencies
                    static PROCESS_START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
                    let start = PROCESS_START.get_or_init(|| Instant::now());
                    Ok(start.elapsed())
                }
            }
        }
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::FILETIME;
            use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};

            let mut creation_time = FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            };
            let mut exit_time = FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            };
            let mut kernel_time = FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            };
            let mut user_time = FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            };

            let result = unsafe {
                GetProcessTimes(
                    GetCurrentProcess(),
                    &mut creation_time,
                    &mut exit_time,
                    &mut kernel_time,
                    &mut user_time,
                )
            };

            if result == 0 {
                return Err(HalError::io_error(
                    "GetProcessTimes",
                    None,
                    std::io::Error::last_os_error(),
                ));
            }

            let user_duration = filetime_to_duration(&user_time);
            let kernel_duration = filetime_to_duration(&kernel_time);

            Ok(user_duration + kernel_duration)
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(HalError::unsupported(
                "CPU time not supported on this platform",
            ))
        }
    }

    pub fn set_timezone(&self, _tz: &str) -> HalResult<()> {
        #[cfg(unix)]
        {
            use chrono::Local;

            let _local_time = Local::now();
            let _offset = _local_time.offset().local_minus_utc();

            Err(HalError::unsupported(
                "Timezone setting not supported on this platform",
            ))
        }
        #[cfg(windows)]
        {
            Err(HalError::unsupported(
                "Timezone setting not supported on this platform",
            ))
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(HalError::unsupported(
                "Timezone setting not supported on this platform",
            ))
        }
    }

    pub fn get_timezone(&self) -> HalResult<String> {
        #[cfg(unix)]
        {
            use chrono::Local;

            let local_time = Local::now();
            let _offset = local_time.offset().local_minus_utc();

            Ok(format!("{}", local_time.offset()))
        }
        #[cfg(windows)]
        {
            Ok("UTC".to_string())
        }
        #[cfg(not(any(unix, windows)))]
        {
            Ok("UTC".to_string())
        }
    }

    pub fn schedule_task<F>(&self, delay: Duration, task: F) -> HalResult<TaskHandle>
    where
        F: FnOnce() + Send + 'static,
    {
        let handle = Arc::new(Mutex::new(None));
        let handle_clone = handle.clone();

        thread::spawn(move || {
            thread::sleep(delay);
            task();
            let mut guard = handle_clone.lock().unwrap();
            *guard = Some(());
        });

        Ok(TaskHandle { handle })
    }

    pub fn create_timer(&self, interval: Duration) -> HalResult<Timer> {
        Ok(Timer::new(interval))
    }
}

impl Default for TimeManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Create a minimal working TimeManager if initialization fails
            TimeManager {
                platform: Platform::current(),
                capabilities: Capabilities::current(),
            }
        })
    }
}

/// Our custom SystemTime wrapper
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemTime {
    inner: StdSystemTime,
}

impl SystemTime {
    pub fn now() -> Self {
        Self {
            inner: StdSystemTime::now(),
        }
    }

    pub fn from_std(time: StdSystemTime) -> Self {
        Self { inner: time }
    }

    pub fn to_std(&self) -> StdSystemTime {
        self.inner
    }

    pub fn duration_since(
        &self,
        earlier: &SystemTime,
    ) -> Result<Duration, std::time::SystemTimeError> {
        self.inner.duration_since(earlier.inner)
    }

    pub fn elapsed(&self) -> Result<Duration, std::time::SystemTimeError> {
        self.inner.elapsed()
    }

    pub fn add(&self, duration: Duration) -> SystemTime {
        SystemTime {
            inner: self.inner + duration,
        }
    }

    pub fn sub(&self, duration: Duration) -> Option<SystemTime> {
        self.inner
            .checked_sub(duration)
            .map(|inner| SystemTime { inner })
    }
}

/// High precision timer
#[derive(Debug)]
pub struct HighPrecisionTimer {
    start: Instant,
}

impl Default for HighPrecisionTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl HighPrecisionTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
    }
}

/// Task handle for scheduled tasks
#[derive(Debug)]
pub struct TaskHandle {
    handle: Arc<Mutex<Option<()>>>,
}

impl TaskHandle {
    pub fn is_finished(&self) -> bool {
        self.handle.lock().unwrap().is_some()
    }
}

/// Timer for periodic operations
#[derive(Debug)]
pub struct Timer {
    interval: Duration,
    last_tick: Instant,
}

impl Timer {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_tick: Instant::now(),
        }
    }

    pub fn tick(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_tick) >= self.interval {
            self.last_tick = now;
            true
        } else {
            false
        }
    }

    pub fn time_until_next_tick(&self) -> Duration {
        let elapsed = self.last_tick.elapsed();
        if elapsed >= self.interval {
            Duration::ZERO
        } else {
            self.interval - elapsed
        }
    }
}

#[cfg(windows)]
fn filetime_to_duration(filetime: &windows_sys::Win32::Foundation::FILETIME) -> Duration {
    let total = ((filetime.dwHighDateTime as u64) << 32) | (filetime.dwLowDateTime as u64);
    // FILETIME is in 100-nanosecond intervals
    Duration::from_nanos(total * 100)
}
