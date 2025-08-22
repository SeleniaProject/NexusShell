//! `nc` (netcat) builtin - Network connection utility.
//!
//! Delegates to the system `nc` or `netcat` binary when available to provide
//! complete networking functionality. When the binary is unavailable, falls
//! back to a basic internal implementation for simple TCP connections.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

/// Entry point for the `nc` builtin.
pub fn nc_cli(args: &[String]) -> Result<()> {
    // Try common netcat binary names
    let nc_commands = vec!["nc", "netcat", "ncat"];
    
    for nc_cmd in nc_commands {
        if let Ok(path) = which(nc_cmd) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("nc: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }
    
    // Basic internal fallback
    if args.len() < 2 {
        return Err(anyhow!("nc: usage: nc host port"));
    }
    
    let host = &args[0];
    let port = args[1].parse::<u16>()
        .map_err(|_| anyhow!("nc: invalid port: {}", args[1]))?;
    
    // Simple TCP connection test
    use std::net::TcpStream;
    use std::time::Duration;
    
    println!("Connecting to {host} port {port}");
    
    let addr = format!("{host}:{port}").parse()
        .map_err(|e| anyhow!("nc: invalid address {}:{}: {}", host, port, e))?;
    
    match TcpStream::connect_timeout(&addr, Duration::from_secs(10)) {
        Ok(_stream) => {
            println!("Connection to {host} {port} port [tcp/*] succeeded!");
            Ok(())
        }
        Err(e) => {
            Err(anyhow!("nc: connect to {} port {}: {}", host, port, e))
        }
    }
}

