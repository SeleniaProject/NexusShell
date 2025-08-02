//! `ping` command  Ebasic ping implementation that delegates to system ping
//!
//! Supports basic ping functionality by calling system ping command

use anyhow::Result;
use std::process::Command;

#[cfg(unix)]
use nxsh_core::Signals;

// Cross-platform signal constants
#[cfg(unix)]
const SIGINT: i32 = nxsh_core::SIGINT;
#[cfg(windows)] 
const SIGINT: i32 = 2; // Windows equivalent

pub fn ping(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow::anyhow!("ping: missing destination"));
    }

    // Use system ping command
    #[cfg(windows)]
    let mut cmd = Command::new("ping");
    
    #[cfg(unix)]
    let mut cmd = Command::new("ping");

    // Add all arguments
    for arg in args {
        cmd.arg(arg);
    }

    let output = cmd.output()?;
    
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("ping command failed"));
    }
    
    Ok(())
}

// Placeholder for compatibility - will be removed when old ping.rs is replaced
pub fn getuid() -> u32 { 0 }
pub fn getpid() -> u32 { std::process::id() }

// Type aliases for compatibility
pub type c_void = std::ffi::c_void;
pub type c_int = std::ffi::c_int;
