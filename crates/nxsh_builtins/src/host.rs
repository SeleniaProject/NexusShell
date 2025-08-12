//! `host` builtin - DNS lookup utility for simple domain resolution.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;
use hickory_resolver::config::*;
use hickory_resolver::{Resolver, Name};
use hickory_resolver::proto::rr::{RecordType, RData};
use std::str::FromStr;

/// Entry point for the `host` builtin
pub fn host_cli(args: &[String]) -> Result<()> {
    // Try external binary first
    if let Ok(result) = try_external_host(args) {
        return result;
    }
    
    // Fall back to internal implementation
    run_internal_host(args)
}

fn try_external_host(args: &[String]) -> Result<Result<()>> {
    if let Ok(path) = which("host") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("host: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    Err(anyhow!("host: backend not found in PATH"))
}

fn run_internal_host(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("host: missing hostname argument"));
    }
    
    if args[0] == "-h" || args[0] == "--help" {
        print_host_help();
        return Ok(());
    }
    
    let hostname = &args[0];
    
    let resolver = Resolver::from_system_conf()
        .map_err(|e| anyhow!("host: failed to create resolver: {}", e))?;
    
    // Try A record lookup
    let name = Name::from_str(hostname)
        .map_err(|e| anyhow!("host: invalid hostname '{}': {}", hostname, e))?;
    
    match resolver.lookup(name.clone(), RecordType::A) {
        Ok(response) => {
            for record in response.iter() {
                if let RData::A(addr) = record {
                    println!("{} has address {}", hostname, addr);
                }
            }
        }
        Err(_) => {
            // Try AAAA record if A record failed
            match resolver.lookup(name.clone(), RecordType::AAAA) {
                Ok(response) => {
                    for record in response.iter() {
                        if let RData::AAAA(addr) = record {
                            println!("{} has IPv6 address {}", hostname, addr);
                        }
                    }
                }
                Err(e) => {
                    return Err(anyhow!("host: lookup failed for {}: {}", hostname, e));
                }
            }
        }
    }
    
    // Try MX record lookup
    match resolver.lookup(name, RecordType::MX) {
        Ok(response) => {
            for record in response.iter() {
                if let RData::MX(mx) = record {
                    println!("{} mail is handled by {} {}", hostname, mx.preference(), mx.exchange());
                }
            }
        }
        Err(_) => {
            // MX lookup failed, which is normal for many domains
        }
    }
    
    Ok(())
}

fn print_host_help() {
    println!("Usage: host hostname");
    println!();
    println!("Simple DNS lookup utility");
    println!();
    println!("Options:");
    println!("  -h, --help    Show this help message");
    println!();
    println!("Examples:");
    println!("  host example.com");
    println!("  host google.com");
}
