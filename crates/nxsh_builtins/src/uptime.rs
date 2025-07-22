//! `uptime` builtin — display system uptime and load averages.
//!
//! Output style roughly emulates GNU uptime (without user count):
//!     14:32:15 up 3 days,  4:23,  load average: 0.42, 0.30, 0.25
//!
//! For portability, we rely on `sysinfo` crate to fetch uptime (seconds)
//! and load averages. User count is omitted for now.

use anyhow::Result;
use chrono::Local;
use sysinfo::{System, SystemExt};

pub fn uptime_cli(_args: &[String]) -> Result<()> {
    let mut sys = System::new();
    sys.refresh_system();

    let uptime_secs = sys.uptime();
    let (days, hours, minutes) = seconds_to_dhm(uptime_secs);

    let now = Local::now();
    let time_str = now.format("%H:%M:%S");

    let load = sys.load_average();

    print!("{} up ", time_str);
    if days > 0 {
        print!("{} days,  ", days);
    }
    print!("{:>2}:{:02},  load average: {:.2}, {:.2}, {:.2}\n", hours, minutes, load.one, load.five, load.fifteen);

    Ok(())
}

fn seconds_to_dhm(mut s: u64) -> (u64, u64, u64) {
    let days = s / 86_400;
    s %= 86_400;
    let hours = s / 3600;
    s %= 3600;
    let minutes = s / 60;
    (days, hours, minutes)
} 