//! `ping` builtin - ICMP echo request utility with cross-platform support.
//!
//! Delegates to the system `ping` binary when available to provide complete
//! ICMP functionality. When the binary is unavailable, falls back to a simple
//! TCP connectivity test for basic network diagnostics.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

/// Entry point for the `ping` builtin.
pub fn ping_cli(args: &[String]) -> Result<()> {
    // Try platform-specific ping commands
    let ping_commands = if cfg!(windows) {
        vec!["ping"]
    } else {
        vec!["ping", "ping6"]
    };
    
    for ping_cmd in ping_commands {
        if let Ok(path) = which(ping_cmd) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("ping: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }
    
    // Fallback: basic connectivity test
    if args.is_empty() {
        return Err(anyhow!("ping: no host specified"));
    }
    
    let host = &args[0];
    println!("PING {host} (TCP connectivity test)");
    println!("Note: This is a basic connectivity test, not true ICMP ping");
    println!("Install system ping for full ICMP functionality");
    
    // Simple TCP connectivity test to port 80
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::{Duration, Instant};
    
    let address = format!("{host}:80");
    let timeout = Duration::from_secs(1);
    
    for i in 1..=4 {
        let start = Instant::now();
        
        match address.to_socket_addrs() {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    match TcpStream::connect_timeout(&addr, timeout) {
                        Ok(_) => {
                            let elapsed = start.elapsed();
                            println!("64 bytes from {}: icmp_seq={} time={:.1}ms (TCP port 80)", 
                                    host, i, elapsed.as_secs_f64() * 1000.0);
                        }
                        Err(_) => {
                            println!("From {host}: icmp_seq={i} Destination Host Unreachable");
                        }
                    }
                } else {
                    println!("ping: cannot resolve {host}: Unknown host");
                    break;
                }
            }
            Err(_) => {
                println!("ping: cannot resolve {host}: Unknown host");
                break;
            }
        }
        
        if i < 4 {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    
    Ok(())
}
