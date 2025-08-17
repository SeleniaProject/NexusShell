//! `hostname` builtin  Eget or set system host name.
//!
//! Usage:
//!   hostname            # print full hostname
//!   hostname -s         # print short hostname (segment before first dot)
//!   hostname NEWNAME    # (Unix only) attempt to set hostname
//! Setting hostname requires CAP_SYS_ADMIN and will fail without privilege.

use anyhow::{anyhow, Result};
#[cfg(feature = "system-info")]
use sysinfo::{System, SystemExt};

#[cfg(unix)]
use nix::libc::{sethostname};

pub fn hostname_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        // default: print hostname
        print_hostname(false)
    } else if args.len() == 1 && args[0] == "-s" {
        print_hostname(true)
    } else if args.len() == 1 {
        // Set hostname (Unix only)
        set_hostname(&args[0])
    } else {
        Err(anyhow!("hostname: invalid arguments"))
    }
}

fn print_hostname(short: bool) -> Result<()> {
    #[cfg(feature = "system-info")]
    let host = {
        let mut sys = System::new();
        sys.refresh_system();
        sys.host_name().unwrap_or_else(|| "Unknown".to_string())
    };
    #[cfg(not(feature = "system-info"))]
    let host = hostname::get().map(|h| h.to_string_lossy().to_string()).unwrap_or_else(|_| "unknown".to_string());
    if short {
        let short_name = host.split('.').next().unwrap_or(&host);
        println!("{short_name}");
    } else {
        println!("{host}");
    }
    Ok(())
}

fn set_hostname(_name: &str) -> Result<()> {
    #[cfg(unix)]
    unsafe {
        if sethostname(name.as_ptr() as *const _, name.len()) == 0 {
            return Ok(());
        } else {
            return Err(anyhow!("hostname: failed to set hostname"));
        }
    }
    #[cfg(windows)]
    {
        Err(anyhow!("hostname: setting hostname not supported on Windows yet"))
    }
} 
