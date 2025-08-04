//! `lsusb` builtin  Elist USB devices.
//!
//! On Unix platforms it attempts to execute the external `lsusb` utility if
//! available. If not, it falls back to system command via shell execution
//! and outputs a compact listing similar to the canonical format.
//! On non-Unix systems a graceful unsupported message is printed.

use anyhow::{anyhow, Result};

pub async fn lsusb_cli(args: &[String]) -> Result<()> {
    if !args.is_empty() {
        return Err(anyhow!("lsusb: no arguments supported yet"));
    }

    #[cfg(not(unix))]
    {
        println!("lsusb: unsupported on this platform");
        return Ok(());
    }

    #[cfg(unix)]
    {
        // Try external command first for full feature parity
        if which::which("lsusb").is_ok() {
            let status = Command::new("lsusb").status()?;
            if !status.success() {
                return Err(anyhow!("lsusb: external command exited with status {}", status));
            }
            return Ok(());
        }

        // Fallback: enumerate via libusb
        // TODO: Delegate to system lsusb command instead of libusb
        // use rusb::{Context, UsbContext};

        let ctx = Context::new()?;
        let devices = ctx.devices()?;
        for device in devices.iter() {
            let bus = device.bus_number();
            let addr = device.address();
            let desc = device.device_descriptor()?;
            println!(
                "Bus {:03} Device {:03}: ID {:04x}:{:04x}",
                bus, addr, desc.vendor_id(), desc.product_id()
            );
        }
    }
    Ok(())
} 
