//! `hwclock` builtin  Eread or set the hardware clock (RTC).
//!
//! Supported wrapper options:
//!   hwclock            # read RTC time
//!   hwclock -w         # write system time to RTC (requires root)
//!   hwclock -s         # set system time from RTC (requires root)
//!
//! This builtin forwards to the external `hwclock` utility when present. If
//! missing, it reports unsupported. On non-Unix platforms, an informative
//! message is printed.

use anyhow::Result;

pub async fn hwclock_cli(_args: &[String]) -> Result<()> {
    #[cfg(not(unix))]
    {
        println!("hwclock: unsupported on this platform");
        Ok(())
    }

    #[cfg(unix)]
    {
        if which::which("hwclock").is_err() {
            return Err(anyhow!("hwclock: external 'hwclock' not found"));
        }
    let status = Command::new("hwclock").args(_args).status()?;
        if !status.success() {
            return Err(anyhow!("hwclock: exited with status {}", status));
        }
        Ok(())
    }
} 
