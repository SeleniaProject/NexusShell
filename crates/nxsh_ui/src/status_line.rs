use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt, CpuExt, NetworkExt};

/// Snapshot of status metrics for the status line.
#[derive(Debug, Clone)]
pub struct StatusSnapshot {
    pub cpu_percent: f32,
    pub mem_used_mib: f64,
    pub mem_total_mib: f64,
    pub net_rx_bps: f64,
    pub net_tx_bps: f64,
    pub battery_percent: Option<f32>,
    pub last_updated: Instant,
}

impl Default for StatusSnapshot {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            mem_used_mib: 0.0,
            mem_total_mib: 0.0,
            net_rx_bps: 0.0,
            net_tx_bps: 0.0,
            battery_percent: None,
            last_updated: Instant::now(),
        }
    }
}

/// Collector that samples system metrics periodically.
pub struct StatusMetricsCollector {
    inner: Arc<Mutex<StatusSnapshot>>,
}

impl StatusMetricsCollector {
    /// Create and start the collector thread. Sampling interval dynamically adapts
    /// based on system load. Base interval is 100ms; under heavy CPU load it backs off.
    pub fn start() -> Arc<Self> {
        let inner = Arc::new(Mutex::new(StatusSnapshot { last_updated: Instant::now(), ..Default::default() }));
        let cloned = inner.clone();
        thread::spawn(move || {
            let mut sys = System::new();
            // Initial refresh to establish baseline for CPU and networks.
            sys.refresh_cpu();
            sys.refresh_memory();
            // Accumulators for network rate calculation (feature-gated implementation below).
            let mut last_rx: u64 = 0;
            let mut last_tx: u64 = 0;
            let mut last_ts = Instant::now();

            let mut interval = Duration::from_millis(100);
            loop {
                // Sleep first to maintain cadence
                thread::sleep(interval);

                // Refresh measurements
                sys.refresh_cpu();
                sys.refresh_memory();
                // Network refresh happens inside feature-gated section if enabled.

                // CPU: use global average
                let cpu_percent = sys.global_cpu_info().cpu_usage();

                // Memory: MiB
                let mem_used_mib = sys.used_memory() as f64 / 1024.0;
                let mem_total_mib = sys.total_memory() as f64 / 1024.0;

                // Network: aggregate across interfaces and compute B/s (feature-gated).
                let (rx_bps, tx_bps) = {
                    #[cfg(feature = "net-metrics")]
                    {
						// Refresh networks for compatible versions
						sys.refresh_networks();
						let mut total_rx: u64 = 0;
						let mut total_tx: u64 = 0;
						for (_name, data) in sys.networks() {
                            // Use method names compatible with sysinfo 0.29
                            total_rx = total_rx.saturating_add(data.total_received());
                            total_tx = total_tx.saturating_add(data.total_transmitted());
                        }
                        let now = Instant::now();
                        let dt = now.saturating_duration_since(last_ts).as_secs_f64();
                        let pair = if dt > 0.0 {
                            (
                                (total_rx.saturating_sub(last_rx)) as f64 / dt,
                                (total_tx.saturating_sub(last_tx)) as f64 / dt,
                            )
                        } else { (0.0, 0.0) };
                        last_rx = total_rx;
                        last_tx = total_tx;
                        last_ts = now;
                        pair
                    }
                    #[cfg(not(feature = "net-metrics"))]
                    { (0.0, 0.0) }
                };

                // Battery: behind feature flag to avoid new deps by default
                let battery_percent: Option<f32> = get_battery_percent();

                // Publish snapshot
                if let Ok(mut snap) = cloned.lock() {
                    snap.cpu_percent = cpu_percent;
                    snap.mem_used_mib = mem_used_mib;
                    snap.mem_total_mib = mem_total_mib;
                    snap.net_rx_bps = rx_bps;
                    snap.net_tx_bps = tx_bps;
                    snap.battery_percent = battery_percent;
                    snap.last_updated = Instant::now();
                }

                // Adaptive interval: if CPU>85% recently, relax to 250ms; if >95%, 500ms; else 100ms.
                interval = if cpu_percent > 95.0 { Duration::from_millis(500) }
                else if cpu_percent > 85.0 { Duration::from_millis(250) }
                else { Duration::from_millis(100) };
            }
        });
        Arc::new(Self { inner })
    }

    /// Get a copy of the latest snapshot.
    pub fn get(&self) -> StatusSnapshot {
        self.inner.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

/// Attempt to get battery percentage if feature enabled.
#[inline]
fn get_battery_percent() -> Option<f32> {
    get_battery_percent_impl()
}

#[cfg(feature = "battery-metrics")]
fn get_battery_percent_impl() -> Option<f32> {
    // Best-effort: query first battery and return state of charge as percent.
    let manager = battery::Manager::new().ok()?;
    let mut iter = manager.batteries().ok()?;
    if let Some(Ok(bat)) = iter.next() {
        let soc = bat.state_of_charge().value * 100.0;
        Some(soc as f32)
    } else { None }
}

#[cfg(not(feature = "battery-metrics"))]
fn get_battery_percent_impl() -> Option<f32> { None }

/// Format helper with minimal bilingual labels.
pub fn format_status_line(s: &StatusSnapshot, colored: bool) -> String {
    let lang = std::env::var("LANG").unwrap_or_default().to_ascii_lowercase();
    let is_ja = lang.starts_with("ja");
    let (cpu_l, mem_l, net_l, bat_l) = if is_ja { ("CPU", "メモリ", "ネット", "電池") } else { ("CPU", "MEM", "NET", "BAT") };

    let mem = format!("{:.1}/{:.1}MiB", s.mem_used_mib, s.mem_total_mib);
    let (rx_s, tx_s) = (human_bps(s.net_rx_bps), human_bps(s.net_tx_bps));
    let bat = s.battery_percent.map(|p| format!("{:.0}%", p)).unwrap_or_else(|| "N/A".to_string());
    let base = format!(
        "{cpu_l} {cpu:.0}% | {mem_l} {mem} | {net_l} ↓{rx}/↑{tx} | {bat_l} {bat}",
        cpu = s.cpu_percent, mem = mem, rx = rx_s, tx = tx_s, bat = bat
    );
    if colored {
        colorize(&base, s.cpu_percent)
    } else { base }
}

fn human_bps(v: f64) -> String {
    const K: f64 = 1024.0;
    if v < K { format!("{:.0}B/s", v) }
    else if v < K*K { format!("{:.1}KiB/s", v/K) }
    else if v < K*K*K { format!("{:.1}MiB/s", v/(K*K)) }
    else { format!("{:.1}GiB/s", v/(K*K*K)) }
}

fn colorize(text: &str, cpu: f32) -> String {
    // Basic color ramp using ANSI: green (<50%), yellow (<85%), red (>=85%)
    let (r, g, y) = ("\x1b[31m", "\x1b[32m", "\x1b[33m");
    let reset = "\x1b[0m";
    let color = if cpu >= 85.0 { r } else if cpu >= 50.0 { y } else { g };
    format!("{}{}{}", color, text, reset)
}


