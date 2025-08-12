//! `ss` builtin  Esocket statistics utility.
//!
//! This wrapper attempts to invoke the modern `ss` command (from `iproute2`)
//! which supersedes `netstat` on most Linux distributions. If `ss` is not
//! available, we gracefully fall back to `netstat -an` to provide similar
//! information.
//!
//! All arguments provided to the builtin are forwarded unchanged to the chosen
//! backend binary.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn ss_cli(args: &[String]) -> Result<()> {
    // Check for help first
    if !args.is_empty() && (args[0] == "-h" || args[0] == "--help") {
        print_ss_help();
        return Ok(());
    }
    
    // Preferred: ss
    if let Ok(path) = which("ss") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("ss: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // Fallback: netstat -an plus user args
    if let Ok(netstat) = which(if cfg!(windows) { "netstat.exe" } else { "netstat" }) {
        // Pre-prepend default flags when no args are specified to mimic `ss` default.
        let forwarded: Vec<String> = if args.is_empty() {
            vec!["-an".to_string()] // show all sockets numerical
        } else {
            convert_ss_to_netstat_args(args)
        };

        let status = Command::new(netstat)
            .args(&forwarded)
            .status()
            .map_err(|e| anyhow!("ss: fallback netstat failed: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    Err(anyhow!("ss: neither 'ss' nor 'netstat' found in PATH"))
}

fn convert_ss_to_netstat_args(ss_args: &[String]) -> Vec<String> {
    let mut netstat_args = Vec::new();
    let mut i = 0;
    
    while i < ss_args.len() {
        match ss_args[i].as_str() {
            "-t" | "--tcp" => netstat_args.push("-t".to_string()),
            "-u" | "--udp" => netstat_args.push("-u".to_string()),
            "-l" | "--listening" => netstat_args.push("-l".to_string()),
            "-a" | "--all" => netstat_args.push("-a".to_string()),
            "-n" | "--numeric" => netstat_args.push("-n".to_string()),
            "-p" | "--processes" => netstat_args.push("-p".to_string()),
            "-s" | "--summary" => netstat_args.push("-s".to_string()),
            arg => {
                // Pass through other arguments
                netstat_args.push(arg.to_string());
            }
        }
        i += 1;
    }
    
    // If no specific options were given, use some sensible defaults
    if netstat_args.is_empty() {
        netstat_args.extend_from_slice(&["-tuln".to_string()]);
    }
    
    netstat_args
}

fn print_ss_help() {
    println!("Usage: ss [options]");
    println!();
    println!("Socket statistics utility (modern netstat replacement)");
    println!();
    println!("Options:");
    println!("  -h, --help        Show this help message");
    println!("  -t, --tcp         Show TCP sockets");
    println!("  -u, --udp         Show UDP sockets");
    println!("  -l, --listening   Show only listening sockets");
    println!("  -a, --all         Show all sockets");
    println!("  -n, --numeric     Show numerical addresses");
    println!("  -p, --processes   Show process using socket");
    println!("  -s, --summary     Show socket statistics summary");
    println!();
    println!("Examples:");
    println!("  ss                # Show all sockets");
    println!("  ss -tuln          # Show TCP/UDP listening sockets with numbers");
    println!("  ss -t -a          # Show all TCP sockets");
    println!("  ss -p             # Show sockets with processes");
    println!();
    println!("Note: This implementation delegates to system 'ss' or 'netstat' when available.");
} 
