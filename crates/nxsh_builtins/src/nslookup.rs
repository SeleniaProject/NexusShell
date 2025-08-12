use anyhow::{Result, anyhow};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use std::collections::HashMap;
use trust_dns_resolver::{TokioAsyncResolver, config::{ResolverConfig, ResolverOpts}};
use trust_dns_resolver::proto::rr::RecordType;
use nxsh_core::{context::Context, ExecutionResult};

/// DNS lookup utility providing nslookup-like functionality
pub struct NslookupCommand;

impl NslookupCommand {
    pub fn execute(&self, _ctx: &mut Context, args: Vec<String>) -> Result<ExecutionResult> {
        tokio::runtime::Handle::current().block_on(async {
            nslookup_cli(&args).await
        })?;
        Ok(ExecutionResult::success(0))
    }
}

// Query options for DNS lookups
#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub record_type: Option<RecordType>,
    pub server: Option<String>,
    pub port: u16,
    pub timeout: Duration,
    pub debug: bool,
    pub tcp: bool,
    pub recursion: bool,
    pub class: String,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            record_type: None,
            server: None,
            port: 53,
            timeout: Duration::from_secs(5),
            debug: false,
            tcp: false,
            recursion: true,
            class: "IN".to_string(),
        }
    }
}

// DNS record information
#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub name: String,
    pub record_type: RecordType,
    pub ttl: u32,
    pub data: String,
}

// DNS response container
#[derive(Debug)]
pub struct DnsResponse {
    pub query: String,
    pub query_type: RecordType,
    pub server: String,
    pub status: String,
    pub records: Vec<DnsRecord>,
    pub additional_info: HashMap<String, String>,
}

/// Main nslookup CLI function
pub async fn nslookup_cli(args: &[String]) -> Result<()> {
    let mut options = QueryOptions::default();
    let mut query_target = String::new();
    let mut i = 0;

    // Parse command line arguments
    while i < args.len() {
        match args[i].as_str() {
            "-type" | "-t" => {
                if i + 1 < args.len() {
                    options.record_type = Some(parse_record_type(&args[i + 1])?);
                    i += 1;
                } else {
                    return Err(anyhow!("Option {} requires an argument", args[i]));
                }
            },
            "-server" | "-s" => {
                if i + 1 < args.len() {
                    options.server = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err(anyhow!("Option {} requires an argument", args[i]));
                }
            },
            "-port" | "-p" => {
                if i + 1 < args.len() {
                    options.port = args[i + 1].parse()
                        .map_err(|_| anyhow!("Invalid port number: {}", args[i + 1]))?;
                    i += 1;
                } else {
                    return Err(anyhow!("Option {} requires an argument", args[i]));
                }
            },
            "-timeout" => {
                if i + 1 < args.len() {
                    let timeout_secs: u64 = args[i + 1].parse()
                        .map_err(|_| anyhow!("Invalid timeout: {}", args[i + 1]))?;
                    options.timeout = Duration::from_secs(timeout_secs);
                    i += 1;
                } else {
                    return Err(anyhow!("Option {} requires an argument", args[i]));
                }
            },
            "-debug" | "-d" => {
                options.debug = true;
            },
            "-tcp" => {
                options.tcp = true;
            },
            "-norecurse" => {
                options.recursion = false;
            },
            "-class" | "-c" => {
                if i + 1 < args.len() {
                    options.class = args[i + 1].clone();
                    i += 1;
                } else {
                    return Err(anyhow!("Option {} requires an argument", args[i]));
                }
            },
            "-help" | "--help" | "-h" => {
                print_help();
                return Ok(());
            },
            arg if !arg.starts_with('-') => {
                if query_target.is_empty() {
                    query_target = arg.to_string();
                } else {
                    // If we already have a target, this might be a server specification
                    if options.server.is_none() {
                        options.server = Some(arg.to_string());
                    }
                }
            },
            _ => {
                return Err(anyhow!("Unknown option: {}", args[i]));
            }
        }
        i += 1;
    }

    if query_target.is_empty() {
        return Err(anyhow!("Missing hostname to lookup"));
    }

    // Perform the DNS lookup
    let response = perform_dns_lookup(&query_target, &options).await?;

    // Display results
    display_dns_response(&response);

    Ok(())
}

/// Parse record type from string
fn parse_record_type(type_str: &str) -> Result<RecordType> {
    match type_str.to_uppercase().as_str() {
        "A" => Ok(RecordType::A),
        "AAAA" => Ok(RecordType::AAAA),
        "CNAME" => Ok(RecordType::CNAME),
        "MX" => Ok(RecordType::MX),
        "NS" => Ok(RecordType::NS),
        "PTR" => Ok(RecordType::PTR),
        "SOA" => Ok(RecordType::SOA),
        "TXT" => Ok(RecordType::TXT),
        "SRV" => Ok(RecordType::SRV),
        "ANY" => Ok(RecordType::ANY),
        _ => Err(anyhow!("Unknown record type: {}", type_str)),
    }
}

/// Perform DNS lookup using hickory-resolver
async fn perform_dns_lookup(target: &str, options: &QueryOptions) -> Result<DnsResponse> {
    let resolver = create_resolver(options).await?;
    
    let query_type = options.record_type.unwrap_or(RecordType::A);
    let server_info = options.server.clone().unwrap_or_else(|| "default".to_string());

    if options.debug {
        println!("Querying {} for {} records on server {}", target, query_type, server_info);
    }

    let mut records = Vec::new();
    let mut additional_info = HashMap::new();

    match query_type {
        RecordType::A => {
            let lookup = resolver.lookup_ip(target).await
                .map_err(|e| anyhow!("DNS lookup failed: {}", e))?;
            
            for ip in lookup.iter() {
                if let IpAddr::V4(ipv4) = ip {
                    records.push(DnsRecord {
                        name: target.to_string(),
                        record_type: RecordType::A,
                        ttl: 300, // Default TTL
                        data: ipv4.to_string(),
                    });
                }
            }
        },
        RecordType::AAAA => {
            let lookup = resolver.lookup_ip(target).await
                .map_err(|e| anyhow!("DNS lookup failed: {}", e))?;
            
            for ip in lookup.iter() {
                if let IpAddr::V6(ipv6) = ip {
                    records.push(DnsRecord {
                        name: target.to_string(),
                        record_type: RecordType::AAAA,
                        ttl: 300, // Default TTL
                        data: ipv6.to_string(),
                    });
                }
            }
        },
        RecordType::MX => {
            let lookup = resolver.mx_lookup(target).await
                .map_err(|e| anyhow!("MX lookup failed: {}", e))?;
            
            for mx in lookup.iter() {
                records.push(DnsRecord {
                    name: target.to_string(),
                    record_type: RecordType::MX,
                    ttl: 300, // Default TTL
                    data: format!("{} {}", mx.preference(), mx.exchange()),
                });
            }
        },
        RecordType::TXT => {
            let lookup = resolver.txt_lookup(target).await
                .map_err(|e| anyhow!("TXT lookup failed: {}", e))?;
            
            for txt in lookup.iter() {
                let txt_data = txt.txt_data().iter()
                    .map(|data| String::from_utf8_lossy(data))
                    .collect::<Vec<_>>()
                    .join(" ");
                
                records.push(DnsRecord {
                    name: target.to_string(),
                    record_type: RecordType::TXT,
                    ttl: 300, // Default TTL
                    data: txt_data,
                });
            }
        },
        RecordType::PTR => {
            // PTR (reverse DNS) lookup
            if let Ok(ip) = target.parse::<IpAddr>() {
                let lookup = resolver.reverse_lookup(ip).await
                    .map_err(|e| anyhow!("PTR lookup failed: {}", e))?;
                
                for ptr in lookup.iter() {
                    records.push(DnsRecord {
                        name: target.to_string(),
                        record_type: RecordType::PTR,
                        ttl: 300, // Default TTL
                        data: ptr.to_string(),
                    });
                }
            } else {
                return Err(anyhow!("PTR queries require an IP address"));
            }
        },
        RecordType::SOA => {
            let lookup = resolver.soa_lookup(target).await
                .map_err(|e| anyhow!("SOA lookup failed: {}", e))?;
            
            for soa in lookup.iter() {
                records.push(DnsRecord {
                    name: target.to_string(),
                    record_type: RecordType::SOA,
                    ttl: 300, // Default TTL
                    data: format!("{} {} {} {} {} {} {}", 
                        soa.mname(), soa.rname(), soa.serial(), 
                        soa.refresh(), soa.retry(), soa.expire(), soa.minimum()),
                });
            }
        },
        RecordType::SRV => {
            let lookup = resolver.srv_lookup(target).await
                .map_err(|e| anyhow!("SRV lookup failed: {}", e))?;
            
            for srv in lookup.iter() {
                records.push(DnsRecord {
                    name: target.to_string(),
                    record_type: RecordType::SRV,
                    ttl: 300, // Default TTL
                    data: format!("{} {} {} {}", srv.priority(), srv.weight(), srv.port(), srv.target()),
                });
            }
        },
        _ => {
            // Generic lookup for other record types
            let lookup = resolver.lookup(target, query_type).await
                .map_err(|e| anyhow!("DNS lookup failed: {}", e))?;
            
            for record in lookup.record_iter() {
                if let Some(data) = record.data() {
                    records.push(DnsRecord {
                        name: target.to_string(),
                        record_type: query_type,
                        ttl: record.ttl(),
                        data: format!("{:?}", data),
                    });
                }
            }
        }
    }

    if records.is_empty() {
        additional_info.insert("status".to_string(), "NXDOMAIN".to_string());
    }

    Ok(DnsResponse {
        query: target.to_string(),
        query_type,
        server: server_info,
        status: if records.is_empty() { "NXDOMAIN".to_string() } else { "NOERROR".to_string() },
        records,
        additional_info,
    })
}

/// Create a DNS resolver with the specified options
async fn create_resolver(options: &QueryOptions) -> Result<TokioAsyncResolver> {
    let mut config = ResolverConfig::default();
    let mut opts = ResolverOpts::default();

    // Configure resolver options
    opts.timeout = options.timeout;
    opts.recursion_desired = options.recursion;
    opts.use_hosts_file = true;

    // Configure custom DNS server if specified
    if let Some(server) = &options.server {
        config = ResolverConfig::new();
        let server_addr = resolve_server_address(server, options.port).await?;
        config.add_name_server(trust_dns_resolver::config::NameServerConfig::new(
            server_addr,
            trust_dns_resolver::config::Protocol::Udp,
        ));
        
        if options.tcp {
            config.add_name_server(trust_dns_resolver::config::NameServerConfig::new(
                server_addr,
                trust_dns_resolver::config::Protocol::Tcp,
            ));
        }
    }

    let resolver = TokioAsyncResolver::tokio(config, opts);
    Ok(resolver)
}

/// Resolve server address from hostname or IP
async fn resolve_server_address(server: &str, port: u16) -> Result<SocketAddr> {
    if let Ok(ip) = server.parse::<IpAddr>() {
        return Ok(SocketAddr::new(ip, port));
    }

    // Try to resolve hostname
    let addr_str = format!("{}:{}", server, port);
    let mut addrs = tokio::net::lookup_host(addr_str).await
        .map_err(|e| anyhow!("Failed to resolve server address '{}': {}", server, e))?;

    addrs.next()
        .ok_or_else(|| anyhow!("No addresses found for server '{}'", server))
}

/// Display DNS response in nslookup format
fn display_dns_response(response: &DnsResponse) {
    println!("Server:\t\t{}", response.server);
    println!("Address:\t{}", response.server);
    println!();

    if response.records.is_empty() {
        println!("** server can't find {}: {}", response.query, response.status);
        return;
    }

    println!("Non-authoritative answer:");
    for record in &response.records {
        match record.record_type {
            RecordType::A | RecordType::AAAA => {
                println!("Name:\t{}", record.name);
                println!("Address: {}", record.data);
            },
            RecordType::MX => {
                println!("{}\tmail exchanger = {}", record.name, record.data);
            },
            RecordType::CNAME => {
                println!("{}\tcanonical name = {}", record.name, record.data);
            },
            RecordType::TXT => {
                println!("{}\ttext = \"{}\"", record.name, record.data);
            },
            RecordType::NS => {
                println!("{}\tnameserver = {}", record.name, record.data);
            },
            RecordType::PTR => {
                println!("{}\tname = {}", record.name, record.data);
            },
            RecordType::SOA => {
                println!("{}\tstart of authority = {}", record.name, record.data);
            },
            RecordType::SRV => {
                println!("{}\tSRV service location = {}", record.name, record.data);
            },
            _ => {
                println!("{}\t{} = {}", record.name, record.record_type, record.data);
            }
        }
        println!();
    }

    // Display additional information if available
    for (key, value) in &response.additional_info {
        println!("{}: {}", key, value);
    }
}

/// Print help information
fn print_help() {
    println!("nslookup - DNS lookup utility");
    println!();
    println!("Usage:");
    println!("  nslookup [options] hostname [server]");
    println!();
    println!("Options:");
    println!("  -type TYPE, -t TYPE    Query for specific record type (A, AAAA, MX, etc.)");
    println!("  -server SERVER, -s     Use specific DNS server");
    println!("  -port PORT, -p PORT    Use specific port (default: 53)");
    println!("  -timeout SECONDS       Set query timeout");
    println!("  -debug, -d             Enable debug output");
    println!("  -tcp                   Use TCP instead of UDP");
    println!("  -norecurse             Disable recursive queries");
    println!("  -class CLASS, -c       Query class (default: IN)");
    println!("  -help, --help, -h      Show this help message");
    println!();
    println!("Record Types:");
    println!("  A       IPv4 address");
    println!("  AAAA    IPv6 address");
    println!("  CNAME   Canonical name");
    println!("  MX      Mail exchange");
    println!("  NS      Name server");
    println!("  PTR     Pointer (reverse DNS)");
    println!("  SOA     Start of authority");
    println!("  TXT     Text record");
    println!("  SRV     Service record");
    println!("  ANY     Any available records");
    println!();
    println!("Examples:");
    println!("  nslookup google.com");
    println!("  nslookup -type MX google.com");
    println!("  nslookup -type TXT google.com 8.8.8.8");
    println!("  nslookup -type PTR 8.8.8.8");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_record_type() {
        assert_eq!(parse_record_type("A").expect("Should parse A record"), RecordType::A);
        assert_eq!(parse_record_type("mx").expect("Should parse MX record"), RecordType::MX);
        assert_eq!(parse_record_type("AAAA").expect("Should parse AAAA record"), RecordType::AAAA);
        
        assert!(parse_record_type("INVALID").is_err());
    }

    #[test]
    fn test_query_options_default() {
        let options = QueryOptions::default();
        assert_eq!(options.record_type, None);
        assert_eq!(options.server, None);
        assert_eq!(options.port, 53);
        assert_eq!(options.timeout, Duration::from_secs(5));
        assert_eq!(options.debug, false);
        assert_eq!(options.tcp, false);
        assert_eq!(options.recursion, true);
        assert_eq!(options.class, "IN");
    }

    #[tokio::test]
    async fn test_dns_resolver_creation() {
        let options = QueryOptions::default();
        let result = create_resolver(&options).await;
        assert!(result.is_ok());
    }
}
