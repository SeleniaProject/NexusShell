use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

/// High-precision timing and performance measurement system
#[derive(Debug, Clone)]
pub struct TimeManager {
    start_time: Instant,
    stats: Arc<RwLock<TimingStats>>,
}

impl Default for TimeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeManager {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            stats: Arc::new(RwLock::new(TimingStats::default())),
        }
    }

    /// Get current monotonic time since manager creation
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get high-precision timestamp
    pub fn now() -> SystemTime {
        SystemTime::now()
    }

    /// Get UNIX timestamp in milliseconds
    pub fn unix_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Get UNIX timestamp in nanoseconds
    pub fn unix_timestamp_ns() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    /// Sleep for precise duration
    pub fn sleep(&self, duration: Duration) {
        thread::sleep(duration);
        self.record_sleep(duration);
    }

    /// Sleep until specific time
    pub fn sleep_until(&self, deadline: Instant) {
        let now = Instant::now();
        if deadline > now {
            self.sleep(deadline - now);
        }
    }

    /// Measure execution time of a closure
    pub fn measure<F, R>(&self, f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        self.record_measurement(elapsed);
        (result, elapsed)
    }

    /// Create a timer that can be started/stopped
    pub fn create_timer(&self) -> Timer {
        Timer::new()
    }

    /// Get timing statistics
    pub fn stats(&self) -> TimingStats {
        self.stats.read().unwrap().clone()
    }

    fn record_sleep(&self, duration: Duration) {
        if let Ok(mut stats) = self.stats.write() {
            stats.sleep_count += 1;
            stats.total_sleep_time += duration;
        }
    }

    fn record_measurement(&self, duration: Duration) {
        if let Ok(mut stats) = self.stats.write() {
            stats.measurement_count += 1;
            stats.total_measured_time += duration;

            if duration < stats.min_measured_time || stats.min_measured_time == Duration::ZERO {
                stats.min_measured_time = duration;
            }

            if duration > stats.max_measured_time {
                stats.max_measured_time = duration;
            }
        }
    }
}

/// Individual timer that can be started and stopped
#[derive(Debug)]
pub struct Timer {
    start_time: Option<Instant>,
    accumulated: Duration,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start_time: None,
            accumulated: Duration::ZERO,
        }
    }

    /// Start the timer
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Stop the timer and accumulate elapsed time
    pub fn stop(&mut self) -> Duration {
        if let Some(start) = self.start_time.take() {
            let elapsed = start.elapsed();
            self.accumulated += elapsed;
            elapsed
        } else {
            Duration::ZERO
        }
    }

    /// Get total accumulated time
    pub fn elapsed(&self) -> Duration {
        let current_session = if let Some(start) = self.start_time {
            start.elapsed()
        } else {
            Duration::ZERO
        };
        self.accumulated + current_session
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.start_time = None;
        self.accumulated = Duration::ZERO;
    }

    /// Check if timer is currently running
    pub fn is_running(&self) -> bool {
        self.start_time.is_some()
    }
}

/// Statistics about timing operations
#[derive(Debug, Clone, Default)]
pub struct TimingStats {
    pub sleep_count: u64,
    pub total_sleep_time: Duration,
    pub measurement_count: u64,
    pub total_measured_time: Duration,
    pub min_measured_time: Duration,
    pub max_measured_time: Duration,
}

impl TimingStats {
    pub fn avg_measured_time(&self) -> Duration {
        if self.measurement_count > 0 {
            self.total_measured_time / self.measurement_count as u32
        } else {
            Duration::ZERO
        }
    }

    pub fn avg_sleep_time(&self) -> Duration {
        if self.sleep_count > 0 {
            self.total_sleep_time / self.sleep_count as u32
        } else {
            Duration::ZERO
        }
    }
}

/// Performance monitoring system
#[derive(Debug)]
pub struct PerformanceMonitor {
    counters: Arc<RwLock<HashMap<String, AtomicU64>>>,
    timers: Arc<RwLock<HashMap<String, Duration>>>,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(std::collections::HashMap::new())),
            timers: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Increment a named counter
    pub fn increment_counter(&self, name: &str) {
        if let Ok(counters) = self.counters.read() {
            if let Some(counter) = counters.get(name) {
                counter.fetch_add(1, Ordering::Relaxed);
                return;
            }
        }

        // Counter doesn't exist, create it
        if let Ok(mut counters) = self.counters.write() {
            counters.insert(name.to_string(), AtomicU64::new(1));
        }
    }

    /// Add to a named counter
    pub fn add_to_counter(&self, name: &str, value: u64) {
        if let Ok(counters) = self.counters.read() {
            if let Some(counter) = counters.get(name) {
                counter.fetch_add(value, Ordering::Relaxed);
                return;
            }
        }

        // Counter doesn't exist, create it
        if let Ok(mut counters) = self.counters.write() {
            counters.insert(name.to_string(), AtomicU64::new(value));
        }
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> u64 {
        if let Ok(counters) = self.counters.read() {
            if let Some(counter) = counters.get(name) {
                return counter.load(Ordering::Relaxed);
            }
        }
        0
    }

    /// Record timing for a named operation
    pub fn record_timing(&self, name: &str, duration: Duration) {
        if let Ok(mut timers) = self.timers.write() {
            let total = *timers.get(name).unwrap_or(&Duration::ZERO) + duration;
            timers.insert(name.to_string(), total);
        }
    }

    /// Get total time for a named operation
    pub fn get_timing(&self, name: &str) -> Duration {
        if let Ok(timers) = self.timers.read() {
            return timers.get(name).copied().unwrap_or(Duration::ZERO);
        }
        Duration::ZERO
    }

    /// Get all counter values
    pub fn get_all_counters(&self) -> std::collections::HashMap<String, u64> {
        if let Ok(counters) = self.counters.read() {
            counters
                .iter()
                .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
                .collect()
        } else {
            std::collections::HashMap::new()
        }
    }

    /// Get all timing values
    pub fn get_all_timings(&self) -> std::collections::HashMap<String, Duration> {
        if let Ok(timers) = self.timers.read() {
            timers.clone()
        } else {
            std::collections::HashMap::new()
        }
    }

    /// Clear all statistics
    pub fn clear(&self) {
        if let Ok(mut counters) = self.counters.write() {
            counters.clear();
        }
        if let Ok(mut timers) = self.timers.write() {
            timers.clear();
        }
    }
}

use std::collections::HashMap;

/// Time zone information
#[derive(Debug, Clone)]
pub struct TimeZone {
    pub name: String,
    pub offset_seconds: i32,
    pub is_dst: bool,
}

impl TimeZone {
    /// Get current system timezone
    pub fn current() -> Self {
        // Simplified implementation - in real world would use proper timezone library
        Self {
            name: "UTC".to_string(),
            offset_seconds: 0,
            is_dst: false,
        }
    }

    /// Convert timestamp to this timezone
    pub fn convert_timestamp(&self, timestamp: SystemTime) -> SystemTime {
        timestamp + Duration::from_secs(self.offset_seconds as u64)
    }
}

/// Date/time formatting utilities
pub struct DateTimeFormatter;

impl DateTimeFormatter {
    /// Format timestamp as ISO 8601
    pub fn iso8601(timestamp: SystemTime) -> String {
        let duration = timestamp.duration_since(UNIX_EPOCH).unwrap_or_default();
        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos();

        let years = 1970 + secs / (365 * 24 * 3600);
        let remaining = secs % (365 * 24 * 3600);
        let days = remaining / (24 * 3600);
        let remaining = remaining % (24 * 3600);
        let hours = remaining / 3600;
        let remaining = remaining % 3600;
        let minutes = remaining / 60;
        let seconds = remaining % 60;

        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}Z",
            years,
            1 + days / 30,
            1 + days % 30,
            hours,
            minutes,
            seconds,
            nanos / 1000
        )
    }

    /// Format timestamp as RFC 3339
    pub fn rfc3339(timestamp: SystemTime) -> String {
        Self::iso8601(timestamp)
    }

    /// Format duration in human readable form
    pub fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        let millis = duration.subsec_millis();

        if hours > 0 {
            format!("{hours}h {minutes}m {seconds}s")
        } else if minutes > 0 {
            format!("{}m {}.{}s", minutes, seconds, millis / 100)
        } else if seconds > 0 {
            format!("{seconds}.{millis:03}s")
        } else {
            format!("{}.{:03}ms", millis, duration.subsec_micros() % 1000)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_basic_operations() {
        let mut timer = Timer::new();

        assert!(!timer.is_running());
        assert_eq!(timer.elapsed(), Duration::ZERO);

        timer.start();
        assert!(timer.is_running());

        thread::sleep(Duration::from_millis(10));

        let elapsed = timer.stop();
        assert!(!timer.is_running());
        assert!(elapsed >= Duration::from_millis(10));
        assert!(timer.elapsed() >= Duration::from_millis(10));
    }

    #[test]
    fn test_time_manager_measure() {
        let manager = TimeManager::new();

        let (result, duration) = manager.measure(|| {
            thread::sleep(Duration::from_millis(5));
            42
        });

        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(5));
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();

        monitor.increment_counter("test_counter");
        monitor.increment_counter("test_counter");
        monitor.add_to_counter("test_counter", 3);

        assert_eq!(monitor.get_counter("test_counter"), 5);

        monitor.record_timing("test_timer", Duration::from_millis(100));
        monitor.record_timing("test_timer", Duration::from_millis(200));

        assert_eq!(monitor.get_timing("test_timer"), Duration::from_millis(300));
    }

    #[test]
    fn test_datetime_formatter() {
        let now = SystemTime::now();
        let formatted = DateTimeFormatter::iso8601(now);
        assert!(formatted.contains("T"));
        assert!(formatted.contains("Z"));

        let duration = Duration::from_secs(3661) + Duration::from_millis(500);
        let formatted = DateTimeFormatter::format_duration(duration);
        assert!(formatted.contains("1h"));
        assert!(formatted.contains("1m"));
        assert!(formatted.contains("1s"));
    }
}
