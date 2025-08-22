//! `netstat` builtin - Network socket status utility with cross-platform support.
//!
//! Delegates to the platform's `netstat` (or equivalent) command when available.
//! On modern Linux distributions where `ss` replaces `netstat`, attempts to call
//! `ss` with compatible arguments. When no external tools are available, provides
//! a basic internal implementation for common socket listing functionality.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct NetstatOptions {
    all: bool,
    listening: bool,
    numeric: bool,
    tcp: bool,
    udp: bool,
    process: bool,
    verbose: bool,
    continuous: bool,
    use_internal: bool,
}


/// Entry point for the `netstat` builtin.
pub fn netstat_cli(args: &[String]) -> Result<()> {
    let options = parse_netstat_args(args)?;
    
    // Prefer the full-featured system implementation when present (unless forced internal).
    if !options.use_internal {
        if let Ok(result) = try_external_netstat(args) {
            return result;
        }
        
        if options.verbose {
            println!("netstat: external binary not found, using internal implementation");
        }
    }
    
    // Use internal implementation
    run_internal_netstat(&options)
}

fn try_external_netstat(args: &[String]) -> Result<Result<()>> {
    // Preferred backends in order.
    let backends = if cfg!(windows) {
        vec!["netstat.exe", "netstat"]
    } else {
        vec!["netstat", "ss"]
    };

    for bin in backends {
        if let Ok(path) = which(bin) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("netstat: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    Err(anyhow!("netstat: no suitable backend found"))
}

fn parse_netstat_args(args: &[String]) -> Result<NetstatOptions> {
    let mut options = NetstatOptions::default();
    
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => {
                print_netstat_help();
                std::process::exit(0);
            }
            "-a" | "--all" => {
                options.all = true;
            }
            "-l" | "--listening" => {
                options.listening = true;
            }
            "-n" | "--numeric" => {
                options.numeric = true;
            }
            "-t" | "--tcp" => {
                options.tcp = true;
            }
            "-u" | "--udp" => {
                options.udp = true;
            }
            "-p" | "--programs" => {
                options.process = true;
            }
            "-v" | "--verbose" => {
                options.verbose = true;
            }
            "-c" | "--continuous" => {
                options.continuous = true;
            }
            "--internal" => {
                options.use_internal = true;
            }
            arg if arg.starts_with('-') => {
                // Handle combined flags like -an, -tulpn
                for ch in arg.chars().skip(1) {
                    match ch {
                        'a' => options.all = true,
                        'l' => options.listening = true,
                        'n' => options.numeric = true,
                        't' => options.tcp = true,
                        'u' => options.udp = true,
                        'p' => options.process = true,
                        'v' => options.verbose = true,
                        'c' => options.continuous = true,
                        _ => return Err(anyhow!("netstat: unknown option: -{}", ch)),
                    }
                }
            }
            _ => {
                return Err(anyhow!("netstat: unknown argument: {}", arg));
            }
        }
    }
    
    Ok(options)
}

fn print_netstat_help() {
    println!("Usage: netstat [options]");
    println!();
    println!("Options:");
    println!("  -h, --help           Show this help message");
    println!("  -a, --all            Show all sockets (default: connected)");
    println!("  -l, --listening      Show only listening ports");
    println!("  -n, --numeric        Show numerical addresses instead of resolving hosts");
    println!("  -t, --tcp            Show TCP sockets");
    println!("  -u, --udp            Show UDP sockets");
    println!("  -p, --programs       Show PID and process name");
    println!("  -v, --verbose        Enable verbose output");
    println!("  -c, --continuous     Continuous listing");
    println!("  --internal           Force use of internal implementation");
    println!();
    println!("Examples:");
    println!("  netstat -a           Show all connections");
    println!("  netstat -an          Show all connections with numeric addresses");
    println!("  netstat -tulpn       Show TCP/UDP listening ports with processes");
    println!("  netstat -l           Show only listening ports");
    println!();
    println!("Note: Internal implementation provides basic socket information");
    println!("      Install system netstat/ss for complete functionality");
}

fn run_internal_netstat(options: &NetstatOptions) -> Result<()> {
    if !options.tcp && !options.udp {
        // Default to both if neither specified
        return run_both_protocols(options);
    }
    
    println!("Active Internet connections ({})", 
             if options.listening { "only servers" } else { "w/o servers" });
    println!("{:<5} {:<6} {:<6} {:<23} {:<23} {:<10}", 
             "Proto", "Recv-Q", "Send-Q", "Local Address", "Foreign Address", "State");
    
    if options.tcp {
        show_tcp_connections(options)?;
    }
    
    if options.udp {
        show_udp_connections(options)?;
    }
    
    Ok(())
}

fn run_both_protocols(options: &NetstatOptions) -> Result<()> {
    let mut tcp_opts = options.clone();
    tcp_opts.tcp = true;
    tcp_opts.udp = false;
    
    let mut udp_opts = options.clone();
    udp_opts.tcp = false;
    udp_opts.udp = true;
    
    run_internal_netstat(&tcp_opts)?;
    println!();
    run_internal_netstat(&udp_opts)?;
    
    Ok(())
}

fn show_tcp_connections(options: &NetstatOptions) -> Result<()> {
    // This is a simplified implementation
    // In a real implementation, you would read from /proc/net/tcp on Linux
    // or use system APIs on Windows
    
    if cfg!(windows) {
        show_windows_connections("tcp", options)
    } else {
        show_unix_connections("tcp", options)
    }
}

fn show_udp_connections(options: &NetstatOptions) -> Result<()> {
    if cfg!(windows) {
        show_windows_connections("udp", options)
    } else {
        show_unix_connections("udp", options)
    }
}

#[cfg(windows)]
fn show_windows_connections(protocol: &str, _options: &NetstatOptions) -> Result<()> {
    // Use PowerShell to get network connections on Windows
    let mut cmd = Command::new("powershell");
    cmd.arg("-Command");
    
    let ps_command = if protocol == "tcp" {
        "Get-NetTCPConnection | Select-Object LocalAddress,LocalPort,RemoteAddress,RemotePort,State | Format-Table -AutoSize"
    } else {
        "Get-NetUDPEndpoint | Select-Object LocalAddress,LocalPort | Format-Table -AutoSize"
    };
    
    cmd.arg(ps_command);
    
    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(3) { // Skip headers
                if !line.trim().is_empty() {
                    println!("{line}");
                }
            }
        }
        Err(_) => {
            // Fallback to basic message
            println!("{:<5} {:<6} {:<6} {:<23} {:<23} {:<10}", 
                     protocol.to_uppercase(), "0", "0", "0.0.0.0:*", "*:*", "UNKNOWN");
            println!("(Use system netstat for detailed information)");
        }
    }
    
    Ok(())
}

#[cfg(not(windows))]
fn show_windows_connections(_protocol: &str, _options: &NetstatOptions) -> Result<()> {
    // This should never be called on non-Windows
    Ok(())
}

#[cfg(not(windows))]
fn show_unix_connections(protocol: &str, options: &NetstatOptions) -> Result<()> {
    // Try to read from /proc/net/tcp or /proc/net/udp
    let proc_file = format!("/proc/net/{}", protocol);
    
    match std::fs::read_to_string(&proc_file) {
        Ok(content) => {
            for (i, line) in content.lines().enumerate() {
                if i == 0 { continue; } // Skip header
                
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let local = parse_socket_addr(parts[1]);
                    let remote = parse_socket_addr(parts[2]);
                    let state = if protocol == "tcp" {
                        parse_tcp_state(parts[3])
                    } else {
                        ""
                    };
                    
                    if options.listening && state != "LISTEN" && protocol == "tcp" {
                        continue;
                    }
                    
                    println!("{:<5} {:<6} {:<6} {:<23} {:<23} {:<10}", 
                             protocol, "0", "0", local, remote, state);
                }
            }
        }
        Err(_) => {
            // Fallback message
            println!("{:<5} {:<6} {:<6} {:<23} {:<23} {:<10}", 
                     protocol.to_uppercase(), "0", "0", "0.0.0.0:*", "*:*", "UNKNOWN");
            println!("(Unable to read {}, use system netstat for detailed information)", proc_file);
        }
    }
    
    Ok(())
}

#[cfg(windows)]
fn show_unix_connections(_protocol: &str, _options: &NetstatOptions) -> Result<()> {
    // This should never be called on Windows
    Ok(())
}

fn parse_socket_addr(hex_addr: &str) -> String {
    if hex_addr.len() < 9 { return hex_addr.to_string(); }
    
    let ip_part = &hex_addr[0..8];
    let port_part = &hex_addr[9..13];
    
    // Parse IP (little-endian)
    if let (Ok(ip_num), Ok(port_num)) = (u32::from_str_radix(ip_part, 16), u16::from_str_radix(port_part, 16)) {
        let ip = std::net::Ipv4Addr::from(ip_num.to_be());
        format!("{ip}:{port_num}")
    } else {
        hex_addr.to_string()
    }
}

fn parse_tcp_state(hex_state: &str) -> &'static str {
    match hex_state {
        "01" => "ESTABLISHED",
        "02" => "SYN_SENT",
        "03" => "SYN_RECV",
        "04" => "FIN_WAIT1",
        "05" => "FIN_WAIT2",
        "06" => "TIME_WAIT",
        "07" => "CLOSE",
        "08" => "CLOSE_WAIT",
        "09" => "LAST_ACK",
        "0A" => "LISTEN",
        "0B" => "CLOSING",
        _ => "UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_netstat_args() {
        let args = vec!["-an".to_string()];
        let options = parse_netstat_args(&args).expect("Failed to parse valid netstat args");
        assert!(options.all);
        assert!(options.numeric);
        
        let args = vec!["-tulpn".to_string()];
        let options = parse_netstat_args(&args).expect("Failed to parse netstat args with multiple flags");
        assert!(options.tcp);
        assert!(options.udp);
        assert!(options.listening);
        assert!(options.process);
        assert!(options.numeric);
    }
}

