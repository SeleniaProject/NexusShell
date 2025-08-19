//! `uname` builtin  Eprint kernel / OS information.
//!
//! Supported flags:
//!   -s   kernel name (default)
//!   -r   kernel release
//!   -v   kernel version
//!   -m   machine hardware name
//!   -n   nodename (hostname)
//!   -a   all (equivalent to -srmn)
//! Unrecognised or no flags => -s.
//!
//! Uses `sysinfo` crate and std::env::consts for some info.

use anyhow::{anyhow, Result};
#[cfg(feature = "system-info")]
use sysinfo::{System, SystemExt};

// Beautiful CUI design
use crate::ui_design::{TableFormatter, ColorPalette, Icons, Colorize};
use color_eyre::owo_colors::OwoColorize;

pub fn uname_cli(args: &[String]) -> Result<()> {
    let flags = if args.is_empty() {
        vec!["-s".to_string()]
    } else {
        args.to_vec()
    };

    #[cfg(feature = "system-info")]
    let (kernel_name, kernel_release, kernel_version, hostname) = {
        let mut sys = System::new();
        sys.refresh_system();
        (
            sys.name().unwrap_or_else(|| "Unknown".to_string()),
            sys.kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            sys.os_version().unwrap_or_else(|| "Unknown".to_string()),
            sys.host_name().unwrap_or_else(|| "Unknown".to_string()),
        )
    };
    #[cfg(not(feature = "system-info"))]
    let (kernel_name, kernel_release, kernel_version, hostname) = (
        std::env::consts::OS.to_string(),
        "0.0".to_string(),
        "unknown".to_string(),
        hostname::get().map(|h| h.to_string_lossy().to_string()).unwrap_or_else(|_| "unknown".to_string()),
    );
    let machine = std::env::consts::ARCH;

    let mut outputs = Vec::new();
    let mut show_table = false;
    
    for flag in flags {
        match flag.as_str() {
            "-s" => outputs.push(("Kernel Name", kernel_name.clone())),
            "-r" => outputs.push(("Release", kernel_release.clone())),
            "-v" => outputs.push(("Version", kernel_version.clone())),
            "-n" => outputs.push(("Hostname", hostname.clone())),
            "-m" => outputs.push(("Architecture", machine.to_string())),
            "-a" => {
                show_table = true;
                outputs = vec![
                    ("Kernel Name", kernel_name.clone()),
                    ("Hostname", hostname.clone()),
                    ("Release", kernel_release.clone()),
                    ("Version", kernel_version.clone()),
                    ("Architecture", machine.to_string()),
                ];
                break;
            }
            _ => return Err(anyhow!("uname: invalid option '{flag}'")),
        }
    }

    if show_table || outputs.len() > 1 {
        // Beautiful system information table
        let colors = ColorPalette::new();
        let icons = Icons::new();
        
        println!("\n{}{}┌─── {} System Information ───┐{}", 
            colors.primary, "═".repeat(5), icons.system, colors.reset);
        
        let table = TableFormatter::new();
        let mut rows = vec![vec!["Property".to_string(), "Value".to_string()]];
        
        for (key, value) in outputs {
            rows.push(vec![
                key.bright_blue().to_string(),
                value.bright_green().to_string()
            ]);
        }
        
        table.print_table(&rows, &["Property", "Value"]);
    } else {
        // Simple output for single flags
        let values: Vec<String> = outputs.into_iter().map(|(_, v)| v).collect();
        println!("{}", values.join(" "));
    }

    Ok(())
}