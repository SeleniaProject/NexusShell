//! `tzselect` builtin  Einteractive timezone selector.
//!
//! This command delegates to the external `tzselect` utility when available.
//! If not present, it prints a list of common timezone identifiers and
//! instructs the user to set `$TZ` in their shell configuration.
//!
//! On non-Unix platforms an informative message is shown.

use anyhow::{Result};
use std::process::Command;

pub async fn tzselect_cli(_args: &[String]) -> Result<()> {
    #[cfg(not(unix))]
    {
        println!("tzselect: unsupported on this platform");
        return Ok(());
    }

    #[cfg(unix)]
    {
        if which::which("tzselect").is_ok() {
            Command::new("tzselect").status()?;
        } else {
            println!("tzselect: external 'tzselect' not found.\n\
                      Please consult https://www.iana.org/time-zones for full list.\n\
                      Example to set timezone: export TZ=Asia/Tokyo");
        }
        Ok(())
    }
} 
