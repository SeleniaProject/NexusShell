//! `dig` builtin - DNS lookup utility for detailed DNS queries.
//!
//! Delegates to the system `dig` binary when available to provide complete
//! DNS functionality. When the binary is unavailable, falls back to an internal
//! implementation using hickory_resolver for common DNS queries.

use anyhow::{anyhow, Result};
use std::process::Command;
use std::net::IpAddr;
use which::which;
use trust_dns_resolver::{Resolver, config::{ResolverConfig, ResolverOpts}};
use trust_dns_resolver::proto::rr::RecordType;

#[derive(Debug, Clone)]
pub struct DigOptions {
    domain: String,
    record_type: RecordType,
    server: Option<String>,
    port: Option<u16>,
    verbose: bool,
    short: bool,
    reverse: bool,
    use_internal: bool,
}

impl Default for DigOptions {
    fn default() -> Self {
        Self {
            domain: String::new(),
            record_type: RecordType::A,
            server: None,
            port: Some(53),
            verbose: false,
            short: false,
            reverse: false,
            use_internal: false,
        }
    }
}

/// Entry point for the `dig` builtin.
pub fn dig_cli(args: &[String]) -> Result<()> {
    let options = parse_dig_args(args)?;
    
    // Prefer the full-featured system implementation when present (unless forced internal).
    if !options.use_internal {
        if let Ok(result) = try_external_dig(args) {
            return result;
        }
        
        if options.verbose {
            println!("; dig: external binary not found, using internal implementation");
        }
    }
    
    // Use internal implementation
    run_internal_dig(&options)
}

fn try_external_dig(args: &[String]) -> Result<Result<()>> {
    if let Ok(path) = which("dig") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("dig: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    
    Err(anyhow!("dig: backend not found"))
}

fn parse_dig_args(args: &[String]) -> Result<DigOptions> {
    let mut options = DigOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_dig_help();
                std::process::exit(0);
            }
            "-v" | "--verbose" => {
                options.verbose = true;
            }
            "+short" => {
                options.short = true;
            }
            "-x" => {
                options.reverse = true;
            }
            "--internal" => {
                options.use_internal = true;
            }
            "-p" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow!("dig: -p requires a port number"));
                }
                options.port = Some(args[i].parse()
                    .map_err(|_| anyhow!("dig: invalid port: {}", args[i]))?);
            }
            arg if arg.starts_with('@') => {
                options.server = Some(arg[1..].to_string());
            }
            arg if !arg.starts_with('-') && !arg.starts_with('+') => {
                if options.domain.is_empty() {
                    options.domain = arg.to_string();
                } else {
                    // Check if it's a record type
                    match arg.to_uppercase().as_str() {
                        "A" => options.record_type = RecordType::A,
                        "AAAA" => options.record_type = RecordType::AAAA,
                        "CNAME" => options.record_type = RecordType::CNAME,
                        "MX" => options.record_type = RecordType::MX,
                        "NS" => options.record_type = RecordType::NS,
                        "PTR" => options.record_type = RecordType::PTR,
                        "SOA" => options.record_type = RecordType::SOA,
                        "TXT" => options.record_type = RecordType::TXT,
                        "SRV" => options.record_type = RecordType::SRV,
                        "ANY" => options.record_type = RecordType::ANY,
                        _ => return Err(anyhow!("dig: unknown record type or too many domains: {}", arg)),
                    }
                }
            }
            _ => {
                return Err(anyhow!("dig: unknown option: {}", args[i]));
            }
        }
        i += 1;
    }
    
    if options.domain.is_empty() {
        return Err(anyhow!("dig: no domain specified"));
    }
    
    Ok(options)
}

fn print_dig_help() {
    println!("Usage: dig [@server] [domain] [type] [options]");
    println!();
    println!("Options:");
    println!("  -h, --help        Show this help message");
    println!("  -v, --verbose     Enable verbose output");
    println!("  -x                Reverse lookup (PTR record)");
    println!("  -p PORT           Use specific port (default: 53)");
    println!("  @SERVER           Use specific DNS server");
    println!("  +short            Short answer format");
    println!("  --internal        Force use of internal implementation");
    println!();
    println!("Record Types:");
    println!("  A, AAAA, CNAME, MX, NS, PTR, SOA, TXT, SRV, ANY");
    println!();
    println!("Examples:");
    println!("  dig example.com");
    println!("  dig example.com MX");
    println!("  dig @8.8.8.8 example.com");
    println!("  dig +short example.com");
    println!("  dig -x 8.8.8.8");
}

fn run_internal_dig(options: &DigOptions) -> Result<()> {
    let resolver = if let Some(server) = &options.server {
        // Use custom DNS server
        let server_addr: IpAddr = server.parse()
            .map_err(|_| anyhow!("dig: invalid DNS server address: {}", server))?;
        
        let config = ResolverConfig::from_parts(
            None,
            vec![],
            vec![(trust_dns_resolver::config::NameServerConfig {
                socket_addr: std::net::SocketAddr::new(server_addr, options.port.unwrap_or(53)),
                protocol: trust_dns_resolver::config::Protocol::Udp,
                tls_dns_name: None,
                trust_negative_responses: false,
                bind_addr: None,
            })],
        );
        Resolver::new(config, ResolverOpts::default())
    } else {
        // Use system resolver
        Resolver::from_system_conf()
    }.map_err(|e| anyhow!("dig: failed to create resolver: {}", e))?;
    
    if !options.short {
        // Print header similar to dig
            println!("; <<>> dig 1.0 (NexusShell internal) <<>> {} {}", 
                options.domain, format_record_type(options.record_type));
        println!(";; global options: +cmd");
    }
    
    let result = if options.reverse {
        // Reverse lookup
        let addr: IpAddr = options.domain.parse()
            .map_err(|_| anyhow!("dig: invalid IP address for reverse lookup: {}", options.domain))?;
        
        match resolver.reverse_lookup(addr) {
            Ok(response) => {
                if options.short {
                    for name in response.iter() {
                        println!("{name}");
                    }
                } else {
                    print_reverse_response(&options.domain, &response);
                }
                Ok(())
            }
            Err(e) => Err(anyhow!("dig: reverse lookup failed: {}", e)),
        }
    } else {
        // Forward lookup
        match options.record_type {
            RecordType::A => {
                match resolver.lookup_ip(&options.domain) {
                    Ok(response) => {
                        if options.short {
                            for ip in response.iter() {
                                if ip.is_ipv4() {
                                    println!("{ip}");
                                }
                            }
                        } else {
                            print_lookup_response(&options.domain, "A", &response.iter().filter(|ip| ip.is_ipv4()).collect::<Vec<_>>());
                        }
                        Ok(())
                    }
                    Err(e) => Err(anyhow!("dig: A record lookup failed: {}", e)),
                }
            }
            RecordType::AAAA => {
                match resolver.lookup_ip(&options.domain) {
                    Ok(response) => {
                        if options.short {
                            for ip in response.iter() {
                                if ip.is_ipv6() {
                                    println!("{ip}");
                                }
                            }
                        } else {
                            print_lookup_response(&options.domain, "AAAA", &response.iter().filter(|ip| ip.is_ipv6()).collect::<Vec<_>>());
                        }
                        Ok(())
                    }
                    Err(e) => Err(anyhow!("dig: AAAA record lookup failed: {}", e)),
                }
            }
            RecordType::MX => {
                match resolver.mx_lookup(&options.domain) {
                    Ok(response) => {
                        if options.short {
                            for mx in response.iter() {
                                println!("{} {}", mx.preference(), mx.exchange());
                            }
                        } else {
                            let mx_records: Vec<String> = response.iter()
                                .map(|mx| format!("{} {}", mx.preference(), mx.exchange()))
                                .collect();
                            print_lookup_response(&options.domain, "MX", &mx_records);
                        }
                        Ok(())
                    }
                    Err(e) => Err(anyhow!("dig: MX record lookup failed: {}", e)),
                }
            }
            RecordType::TXT => {
                match resolver.txt_lookup(&options.domain) {
                    Ok(response) => {
                        if options.short {
                            for txt in response.iter() {
                                for data in txt.iter() {
                                    println!("{}", String::from_utf8_lossy(data));
                                }
                            }
                        } else {
                            let txt_records: Vec<String> = response.iter()
                                .flat_map(|txt| txt.iter())
                                .map(|data| format!("\"{}\"", String::from_utf8_lossy(data)))
                                .collect();
                            print_lookup_response(&options.domain, "TXT", &txt_records);
                        }
                        Ok(())
                    }
                    Err(e) => Err(anyhow!("dig: TXT record lookup failed: {}", e)),
                }
            }
            _ => {
                Err(anyhow!("dig: record type {} not supported in internal implementation", format_record_type(options.record_type)))
            }
        }
    };
    
    if !options.short && result.is_ok() {
        println!();
        println!(";; Query time: 0 msec");
        if let Some(server) = &options.server {
            println!(";; SERVER: {}#{}", server, options.port.unwrap_or(53));
        } else {
            println!(";; SERVER: system resolver");
        }
        println!(";; WHEN: {}", chrono::Local::now().format("%a %b %d %H:%M:%S %Z %Y"));
    }
    
    result
}

fn format_record_type(record_type: RecordType) -> &'static str {
    match record_type {
        RecordType::A => "A",
        RecordType::AAAA => "AAAA",
        RecordType::CNAME => "CNAME",
        RecordType::MX => "MX",
        RecordType::NS => "NS",
        RecordType::PTR => "PTR",
        RecordType::SOA => "SOA",
        RecordType::TXT => "TXT",
        RecordType::SRV => "SRV",
        RecordType::ANY => "ANY",
        _ => "UNKNOWN",
    }
}

fn print_lookup_response<T: std::fmt::Display>(domain: &str, record_type: &str, records: &[T]) {
    println!(";; Got answer:");
    println!(";; ->>HEADER<<- opcode: QUERY, status: NOERROR, id: 1");
    println!(";; flags: qr rd ra; QUERY: 1, ANSWER: {}, AUTHORITY: 0, ADDITIONAL: 0", records.len());
    println!();
    println!(";; QUESTION SECTION:");
    println!(";{domain}\t\tIN\t{record_type}");
    println!();
    println!(";; ANSWER SECTION:");
    
    for record in records {
    println!("{domain}\t300\tIN\t{record_type}\t{record}");
    }
}

fn print_reverse_response(ip: &str, response: &trust_dns_resolver::lookup::ReverseLookup) {
    println!(";; Got answer:");
    println!(";; ->>HEADER<<- opcode: QUERY, status: NOERROR, id: 1");
    println!(";; flags: qr rd ra; QUERY: 1, ANSWER: {}, AUTHORITY: 0, ADDITIONAL: 0", response.iter().count());
    println!();
    println!(";; QUESTION SECTION:");
    println!(";{ip}.in-addr.arpa.\t\tIN\tPTR");
    println!();
    println!(";; ANSWER SECTION:");
    
    for name in response.iter() {
    println!("{ip}.in-addr.arpa.\t300\tIN\tPTR\t{name}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_dig_args() {
        let args = vec!["example.com".to_string()];
        let options = parse_dig_args(&args).unwrap();
        assert_eq!(options.domain, "example.com");
        assert_eq!(options.record_type, RecordType::A);
        
        let args = vec!["example.com".to_string(), "MX".to_string()];
        let options = parse_dig_args(&args).unwrap();
        assert_eq!(options.record_type, RecordType::MX);
    }
}
