//! `uname` builtin — print kernel / OS information.
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
use sysinfo::{System, SystemExt};

pub fn uname_cli(args: &[String]) -> Result<()> {
    let flags = if args.is_empty() {
        vec!["-s".to_string()]
    } else {
        args.to_vec()
    };

    let mut sys = System::new();
    sys.refresh_system();

    let kernel_name = sys.name().unwrap_or_else(|| "Unknown".to_string());
    let kernel_release = sys.kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let kernel_version = sys.os_version().unwrap_or_else(|| "Unknown".to_string());
    let hostname = sys.host_name().unwrap_or_else(|| "Unknown".to_string());
    let machine = std::env::consts::ARCH;

    let mut outputs = Vec::new();

    for flag in flags {
        match flag.as_str() {
            "-s" => outputs.push(kernel_name.clone()),
            "-r" => outputs.push(kernel_release.clone()),
            "-v" => outputs.push(kernel_version.clone()),
            "-n" => outputs.push(hostname.clone()),
            "-m" => outputs.push(machine.to_string()),
            "-a" => {
                outputs = vec![
                    kernel_name.clone(),
                    hostname.clone(),
                    kernel_release.clone(),
                    kernel_version.clone(),
                    machine.to_string(),
                ];
                break;
            }
            _ => return Err(anyhow!("uname: invalid option '{flag}'")),
        }
    }

    println!("{}", outputs.join(" "));
    Ok(())
} 