//! `pgrep` builtin — search processes by name (regex).
//!
//! Usage: `pgrep PATTERN` (POSIX ERE pattern). Prints matching PIDs, one per line.
//! Options not yet supported (future: -l, -f, -x, etc.).

use anyhow::{anyhow, Result};
use regex::Regex;
use sysinfo::{ProcessExt, System, SystemExt};

pub fn pgrep_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("pgrep: missing PATTERN"));
    }
    let pattern = &args[0];
    let re = Regex::new(pattern).map_err(|e| anyhow!("pgrep: invalid regex: {e}"))?;

    let mut sys = System::new_all();
    sys.refresh_processes();

    for (pid, proc_) in sys.processes() {
        if re.is_match(proc_.name()) {
            println!("{}", pid);
        }
    }
    Ok(())
} 