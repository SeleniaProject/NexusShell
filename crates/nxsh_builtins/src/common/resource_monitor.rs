//! Basic cross-platform process resource monitoring helpers.
//!
//! When the `system-info` feature is enabled, these functions collect
//! peak memory and coarse network counters using `sysinfo`.
//! Otherwise, they return zeroed defaults to keep builds minimal.

use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct BasicUsage {
    pub cpu_time: Duration,
    pub memory_peak_bytes: u64,
    pub network_rx: u64,
    pub network_tx: u64,
}

#[cfg(all(feature = "async-runtime", feature = "system-info"))]
pub fn spawn_basic_monitor(pid: u32) -> tokio::task::JoinHandle<BasicUsage> {
    use sysinfo::{NetworkExt, NetworksExt, PidExt, ProcessExt, SystemExt};
    use tokio::time::{sleep, Duration as TokioDuration};

    tokio::spawn(async move {
        let mut sys = sysinfo::System::new();
        let start = std::time::Instant::now();
        let mut peak_mem_kib: u64 = 0;
        let mut rx0: u64 = 0;
        let mut tx0: u64 = 0;
        sys.refresh_networks();
        for (_name, data) in sys.networks() {
            rx0 += data.total_received();
            tx0 += data.total_transmitted();
        }

        loop {
            sys.refresh_processes();
            if let Some(p) = sys.process(sysinfo::Pid::from(pid as usize)) {
                let mem_kib = p.memory();
                if mem_kib > peak_mem_kib {
                    peak_mem_kib = mem_kib;
                }
            } else {
                break;
            }
            sleep(TokioDuration::from_millis(200)).await;
        }

        sys.refresh_networks();
        let mut rx1: u64 = 0;
        let mut tx1: u64 = 0;
        for (_name, data) in sys.networks() {
            rx1 += data.total_received();
            tx1 += data.total_transmitted();
        }

        BasicUsage {
            cpu_time: start.elapsed(),
            memory_peak_bytes: peak_mem_kib.saturating_mul(1024),
            network_rx: rx1.saturating_sub(rx0),
            network_tx: tx1.saturating_sub(tx0),
        }
    })
}

#[cfg(all(feature = "async-runtime", not(feature = "system-info")))]
pub fn spawn_basic_monitor(_pid: u32) -> tokio::task::JoinHandle<BasicUsage> {
    tokio::spawn(async move { BasicUsage::default() })
}

// When async runtime is disabled, provide a synchronous no-op monitor that just returns defaults.
#[cfg(not(feature = "async-runtime"))]
pub fn spawn_basic_monitor(_pid: u32) -> BasicUsage {
    BasicUsage::default()
}
