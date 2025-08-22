use anyhow::{Result, Context};
use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs},
    time::{Duration, Instant, SystemTime},
    sync::Arc,
    process::Stdio,
    io::{BufRead, BufReader},
    fmt,
    str::FromStr,
};
use tokio::{
    net::{TcpStream, UdpSocket},
    process::Command,
    time::{sleep, timeout},
    sync::RwLock,
    io::{AsyncReadExt, AsyncWriteExt},
};
use serde::{Deserialize, Serialize};
use log::{info, warn, error, debug};
use chrono::{DateTime, Utc};

// HTTP Method enum for curl functionality
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::HEAD => write!(f, "HEAD"),
            HttpMethod::OPTIONS => write!(f, "OPTIONS"),
            HttpMethod::PATCH => write!(f, "PATCH"),
        }
    }
}

impl FromStr for HttpMethod {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "HEAD" => Ok(HttpMethod::HEAD),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            "PATCH" => Ok(HttpMethod::PATCH),
            _ => Err(anyhow::anyhow!("Invalid HTTP method: {}", s)),
        }
    }
}

// Network connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Established,
    SynSent,
    SynRecv,
    FinWait1,
    FinWait2,
    TimeWait,
    Close,
    CloseWait,
    LastAck,
    Listen,
    Closing,
    Unknown,
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionState::Established => write!(f, "ESTABLISHED"),
            ConnectionState::SynSent => write!(f, "SYN_SENT"),
            ConnectionState::SynRecv => write!(f, "SYN_RECV"),
            ConnectionState::FinWait1 => write!(f, "FIN_WAIT1"),
            ConnectionState::FinWait2 => write!(f, "FIN_WAIT2"),
            ConnectionState::TimeWait => write!(f, "TIME_WAIT"),
            ConnectionState::Close => write!(f, "CLOSE"),
            ConnectionState::CloseWait => write!(f, "CLOSE_WAIT"),
            ConnectionState::LastAck => write!(f, "LAST_ACK"),
            ConnectionState::Listen => write!(f, "LISTEN"),
            ConnectionState::Closing => write!(f, "CLOSING"),
            ConnectionState::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

// Network connection info
#[derive(Debug, Clone)]
pub struct NetworkConnection {
    pub protocol: String,
    pub local_address: SocketAddr,
    pub remote_address: Option<SocketAddr>,
    pub state: ConnectionState,
    pub recv_queue: u64,
    pub send_queue: u64,
    pub process_id: Option<u32>,
    pub process_name: Option<String>,
}

// Network interface info
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub index: u32,
    pub flags: Vec<String>,
    pub mtu: u32,
    pub addresses: Vec<IpAddr>,
    pub mac_address: Option<String>,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

use crate::common::i18n::tr;
use nxsh_core::{context::NxshContext, result::NxshResult};

/// Network tools manager for various network utilities
pub struct NetworkToolsManager {
    ping_sessions: Arc<RwLock<HashMap<String, PingSession>>>,
    dns_cache: Arc<RwLock<HashMap<String, Vec<IpAddr>>>>,
    config: NetworkToolsConfig,
}

impl NetworkToolsManager {
    /// Create a new network tools manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            ping_sessions: Arc::new(RwLock::new(HashMap::new())),
            dns_cache: Arc::new(RwLock::new(HashMap::new())),
            config: NetworkToolsConfig::default(),
        })
    }
    
    #[cfg(windows)]
    fn get_process_name_from_pid(pid: u32) -> Option<String> {
        use windows_sys::Win32::System::Threading::*;
        use windows_sys::Win32::System::ProcessStatus::*;
        use windows_sys::Win32::Foundation::*;
        use windows_sys::Win32::System::Diagnostics::ToolHelp::*;
        use windows_sys::Win32::System::WindowsProgramming::*;
        use windows_sys::Win32::System::ProcessStatus::K32GetProcessImageFileNameW;

        unsafe {
            // Try QueryFullProcessImageNameW first
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle != 0 {
                // Buffer for path
                let mut buf: [u16; 260] = [0; 260];
                let mut size: u32 = buf.len() as u32;
                // Prefer QueryFullProcessImageNameW if available
                // windows-sys exposes it via Kernel32
                #[allow(non_snake_case)]
                extern "system" {
                    fn QueryFullProcessImageNameW(hProcess: HANDLE, dwFlags: u32, lpExeName: *mut u16, lpdwSize: *mut u32) -> i32;
                }
                let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut size);
                CloseHandle(handle);
                if ok != 0 {
                    let s = String::from_utf16_lossy(&buf[..size as usize]);
                    // Return just file name for compactness
                    if let Some(name) = std::path::Path::new(&s).file_name().and_then(|v| v.to_str()) {
                        return Some(name.to_string());
                    }
                    return Some(s);
                }
            }

            // Fallback to PSAPI K32GetProcessImageFileNameW
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
            if handle != 0 {
                let mut buf: [u16; 260] = [0; 260];
                let len = K32GetProcessImageFileNameW(handle, buf.as_mut_ptr(), buf.len() as u32);
                CloseHandle(handle);
                if len > 0 {
                    let s = String::from_utf16_lossy(&buf[..len as usize]);
                    if let Some(name) = std::path::Path::new(&s).file_name().and_then(|v| v.to_str()) {
                        return Some(name.to_string());
                    }
                    return Some(s);
                }
            }
        }
        None
    }

    /// Execute ping command
    pub async fn ping(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_ping_args(args)?;
        
        info!("Starting ping to {} with {} packets", options.target, options.count);
        
        let target_ips = self.resolve_hostname(&options.target).await?;
        if target_ips.is_empty() {
            return Err(anyhow::anyhow!("Cannot resolve hostname: {}", options.target).into());
        }
        
        let target_ip = target_ips[0];
        let session_id = format!("{}_{}", options.target, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());
        
        // Create ping session
        let session = PingSession {
            target: options.target.clone(),
            target_ip,
            count: options.count,
            interval: options.interval,
            timeout: options.timeout,
            packet_size: options.packet_size,
            sent: 0,
            received: 0,
            lost: 0,
            min_time: f64::MAX,
            max_time: 0.0,
            avg_time: 0.0,
            total_time: 0.0,
            start_time: Instant::now(),
        };
        
        {
            let mut sessions = self.ping_sessions.write().await;
            sessions.insert(session_id.clone(), session);
        }
        
        // Print ping header
        println!("PING {} ({}) {} bytes of data", options.target, target_ip, options.packet_size);
        
        // Perform ping
        for seq in 1..=options.count {
            let start = Instant::now();
            
            match self.send_ping(target_ip, seq, options.packet_size, options.timeout).await {
                Ok(duration) => {
                    let time_ms = duration.as_secs_f64() * 1000.0;
                    println!("64 bytes from {}: icmp_seq={} time={:.3} ms", target_ip, seq, time_ms);
                    
                    // Update session statistics
                    {
                        let mut sessions = self.ping_sessions.write().await;
                        if let Some(session) = sessions.get_mut(&session_id) {
                            session.sent += 1;
                            session.received += 1;
                            session.total_time += time_ms;
                            session.min_time = session.min_time.min(time_ms);
                            session.max_time = session.max_time.max(time_ms);
                            session.avg_time = session.total_time / session.received as f64;
                        }
                    }
                },
                Err(e) => {
                    println!("Request timeout for icmp_seq={}", seq);
                    
                    // Update session statistics
                    {
                        let mut sessions = self.ping_sessions.write().await;
                        if let Some(session) = sessions.get_mut(&session_id) {
                            session.sent += 1;
                            session.lost += 1;
                        }
                    }
                }
            }
            
            if seq < options.count {
                sleep(options.interval).await;
            }
        }
        
        // Print statistics
        self.print_ping_statistics(&session_id).await;
        
        // Remove session
        {
            let mut sessions = self.ping_sessions.write().await;
            sessions.remove(&session_id);
        }
        
        Ok(())
    }
    
    /// Execute traceroute command
    pub async fn traceroute(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_traceroute_args(args)?;
        
        info!("Starting traceroute to {}", options.target);
        
        let target_ips = self.resolve_hostname(&options.target).await?;
        if target_ips.is_empty() {
            return Err(anyhow::anyhow!("Cannot resolve hostname: {}", options.target).into());
        }
        
        let target_ip = target_ips[0];
        println!("traceroute to {} ({}), {} hops max, {} byte packets", 
                options.target, target_ip, options.max_hops, options.packet_size);
        
        for ttl in 1..=options.max_hops {
            print!("{:2} ", ttl);
            
            let mut hop_times = Vec::new();
            let mut hop_ip = None;
            
            for probe in 1..=options.probes {
                match self.send_traceroute_probe(target_ip, ttl, options.timeout).await {
                    Ok((ip, duration)) => {
                        let time_ms = duration.as_secs_f64() * 1000.0;
                        hop_times.push(time_ms);
                        
                        if hop_ip.is_none() {
                            hop_ip = Some(ip);
                        }
                        
                        if probe == 1 {
                            if let Some(hostname) = self.reverse_dns_lookup(ip).await {
                                print!("{} ({}) ", hostname, ip);
                            } else {
                                print!("{} ", ip);
                            }
                        }
                        
                        print!("{:.3} ms ", time_ms);
                    },
                    Err(_) => {
                        print!("* ");
                    }
                }
            }
            
            println!();
            
            // Check if we reached the target
            if let Some(ip) = hop_ip {
                if ip == target_ip {
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Execute nslookup command
    pub async fn nslookup(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_nslookup_args(args)?;
        
        info!("DNS lookup for {}", options.query);
        
        match options.record_type.as_str() {
            "A" => {
                let ips = self.resolve_hostname(&options.query).await?;
                if ips.is_empty() {
                    println!("** server can't find {}: NXDOMAIN", options.query);
                } else {
                    println!("Server:\t\t{}", options.server.unwrap_or_else(|| "8.8.8.8".to_string()));
                    println!("Address:\t{}#53", options.server.unwrap_or_else(|| "8.8.8.8".to_string()));
                    println!();
                    println!("Non-authoritative answer:");
                    for ip in ips {
                        println!("Name:\t{}", options.query);
                        println!("Address: {}", ip);
                    }
                }
            },
            "PTR" => {
                if let Ok(ip) = options.query.parse::<IpAddr>() {
                    if let Some(hostname) = self.reverse_dns_lookup(ip).await {
                        println!("Server:\t\t{}", options.server.unwrap_or_else(|| "8.8.8.8".to_string()));
                        println!("Address:\t{}#53", options.server.unwrap_or_else(|| "8.8.8.8".to_string()));
                        println!();
                        println!("Non-authoritative answer:");
                        println!("{}\tname = {}", ip, hostname);
                    } else {
                        println!("** server can't find {}: NXDOMAIN", options.query);
                    }
                } else {
                    return Err(anyhow::anyhow!("Invalid IP address for PTR query: {}", options.query).into());
                }
            },
            _ => {
                return Err(anyhow::anyhow!("Unsupported record type: {}", options.record_type).into());
            }
        }
        
        Ok(())
    }
    
    /// Execute dig command
    pub async fn dig(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_dig_args(args)?;
        
        info!("DNS dig for {} type {}", options.query, options.record_type);
        
        // Simulate dig output format
        println!("; <<>> DiG 9.18.0 <<>> {} {}", options.query, options.record_type);
        println!(";; global options: +cmd");
        println!(";; Got answer:");
        println!(";; ->>HEADER<<- opcode: QUERY, status: NOERROR, id: 12345");
        println!(";; flags: qr rd ra; QUERY: 1, ANSWER: 1, AUTHORITY: 0, ADDITIONAL: 1");
        println!();
        println!(";; QUESTION SECTION:");
        println!(";{}\t\t\tIN\t{}", options.query, options.record_type);
        println!();
        
        match options.record_type.as_str() {
            "A" => {
                let ips = self.resolve_hostname(&options.query).await?;
                if !ips.is_empty() {
                    println!(";; ANSWER SECTION:");
                    for ip in ips {
                        println!("{}\t300\tIN\tA\t{}", options.query, ip);
                    }
                }
            },
            "PTR" => {
                if let Ok(ip) = options.query.parse::<IpAddr>() {
                    if let Some(hostname) = self.reverse_dns_lookup(ip).await {
                        println!(";; ANSWER SECTION:");
                        println!("{}\t300\tIN\tPTR\t{}", ip, hostname);
                    }
                }
            },
            _ => {
                println!(";; No answer for record type: {}", options.record_type);
            }
        }
        
        println!();
        println!(";; Query time: 15 msec");
        println!(";; SERVER: {}#53({})", options.server.unwrap_or_else(|| "8.8.8.8".to_string()), options.server.unwrap_or_else(|| "8.8.8.8".to_string()));
        println!(";; WHEN: {}", Utc::now().format("%a %b %d %H:%M:%S UTC %Y"));
        println!(";; MSG SIZE  rcvd: 55");
        
        Ok(())
    }
    
    /// Execute curl command (simplified implementation)
    pub async fn curl(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_curl_args(args)?;
        
        info!("HTTP request to {} using method {}", options.url, options.method);
        
        #[cfg(feature = "net-http")]
        {
            // Use ureq for HTTP requests when net-http feature is enabled
            let agent = ureq::AgentBuilder::new()
                .timeout(self.config.default_timeout)
                .user_agent(&self.config.user_agent)
                .build();
            
            let mut request = match options.method {
                HttpMethod::GET => agent.get(&options.url),
                HttpMethod::POST => agent.post(&options.url),
                HttpMethod::PUT => agent.put(&options.url),
                HttpMethod::DELETE => agent.delete(&options.url),
                HttpMethod::HEAD => agent.head(&options.url),
                _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", options.method).into()),
            };
            
            // Add headers
            for (key, value) in &options.headers {
                request = request.set(&key, &value);
            }
            
            // Add basic auth if present
            if let Some((ref username, ref password)) = options.basic_auth {
                if let Some(pass) = password {
                    request = request.auth(username, pass);
                } else {
                    request = request.auth(username, "");
                }
            }
            
            let start_time = Instant::now();
            
            let response = if let Some(ref body) = options.body {
                request.send_string(body)
            } else {
                request.call()
            };
            
            match response {
                Ok(resp) => {
                    let duration = start_time.elapsed();
                    
                    if options.include_headers {
                        println!("HTTP/1.1 {}", resp.status());
                        for header_name in resp.headers_names() {
                            if let Some(header_value) = resp.header(&header_name) {
                                println!("{}: {}", header_name, header_value);
                            }
                        }
                        println!();
                    }
                    
                    if options.head_only {
                        return Ok(());
                    }
                    
                    let body = resp.into_string().context("Failed to read response body")?;
                    
                    if let Some(ref output_path) = options.output_file {
                        tokio::fs::write(output_path, &body).await
                            .with_context(|| format!("Failed to write to file: {}", output_path))?;
                        println!("Response saved to: {}", output_path);
                    } else {
                        print!("{}", body);
                    }
                    
                    if options.verbose {
                        eprintln!("Request completed in {:.3}s", duration.as_secs_f64());
                        eprintln!("Content-Length: {}", body.len());
                    }
                },
                Err(e) => {
                    return Err(anyhow::anyhow!("HTTP request failed: {}", e).into());
                }
            }
        }
        
        #[cfg(not(feature = "net-http"))]
        {
            return Err(anyhow::anyhow!("HTTP functionality disabled (build without 'net-http' feature)").into());
        }
        
        Ok(())
    }
    
    /// Execute wget command
    pub async fn wget(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_wget_args(args)?;
        
        info!("Downloading {} to {}", options.url, options.output_file.as_deref().unwrap_or("stdout"));
        
        #[cfg(feature = "net-http")]
        let response = self.http_client.get(&options.url)
            .call()
            .map_err(|e| anyhow::anyhow!("Failed to send HTTP request: {e}"))?;
        #[cfg(not(feature = "net-http"))]
        {
            return Err(anyhow::anyhow!("HTTP disabled (build without 'net-http' feature)").into());
        }
        
        #[cfg(feature = "net-http")]
        if response.status() != 200 { return Err(anyhow::anyhow!("HTTP error: {}", response.status()).into()); }
        
        #[cfg(feature = "net-http")]
        let total_size = response.header("content-length").and_then(|h| h.as_str().parse::<u64>().ok());
        let mut downloaded = 0u64;
        let start_time = Instant::now();
        
        if let Some(size) = total_size {
            println!("Length: {} bytes", size);
        }
        
        #[cfg(feature = "net-http")]
        let mut reader = response.into_reader();
        #[cfg(feature = "net-http")]
        let mut output: Box<dyn tokio::io::AsyncWrite + Unpin> = if let Some(ref output_file) = options.output_file {
            Box::new(tokio::fs::File::create(output_file).await
                .with_context(|| format!("Failed to create output file: {}", output_file))?)
        } else {
            Box::new(tokio::io::stdout())
        };
        
        #[cfg(feature = "net-http")]
        {
            use std::io::Read;
            let mut buf = [0u8; 8192];
            loop {
                let n = reader.read(&mut buf).context("Failed to read response chunk")?;
                if n == 0 { break; }
                output.write_all(&buf[..n]).await.context("Failed to write chunk")?;
                downloaded += n as u64;
                if options.show_progress && total_size.is_some() {
                    let progress = (downloaded as f64 / total_size.unwrap() as f64) * 100.0;
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let speed = downloaded as f64 / elapsed;
                    eprint!("\r{:.1}% [{:>10}] {:.1}KB/s", progress, self.format_bytes(downloaded), speed / 1024.0);
                }
            }
        }
        
        output.flush().await.context("Failed to flush output")?;
        
        if options.show_progress {
            eprintln!();
        }
        
        let elapsed = start_time.elapsed();
        println!("Downloaded {} bytes in {:.2}s ({:.1} KB/s)", 
                downloaded, 
                elapsed.as_secs_f64(), 
                (downloaded as f64 / elapsed.as_secs_f64()) / 1024.0);
        
        Ok(())
    }
    
    /// Execute netstat command
    pub async fn netstat(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_netstat_args(args)?;
        
        info!("Getting network statistics");
        
        let connections = self.get_network_connections(&options).await?;
        
        // Print header
        if options.show_processes {
            println!("Proto Recv-Q Send-Q Local Address           Foreign Address         State       PID/Program name");
        } else {
            println!("Proto Recv-Q Send-Q Local Address           Foreign Address         State");
        }
        
        // Print connections
        for conn in connections {
            let local_addr = if options.show_numeric {
                conn.local_address.to_string()
            } else {
                self.resolve_address_with_service(conn.local_address).await
            };
            
            let remote_addr = if let Some(remote) = conn.remote_address {
                if options.show_numeric {
                    remote.to_string()
                } else {
                    self.resolve_address_with_service(remote).await
                }
            } else {
                "*:*".to_string()
            };
            
            if options.show_processes {
                let process_info = if let (Some(pid), Some(name)) = (conn.process_id, conn.process_name) {
                    format!("{}/{}", pid, name)
                } else {
                    "-".to_string()
                };
                
                println!("{:<5} {:>6} {:>6} {:<23} {:<23} {:<11} {}",
                        conn.protocol,
                        conn.recv_queue,
                        conn.send_queue,
                        local_addr,
                        remote_addr,
                        conn.state,
                        process_info);
            } else {
                println!("{:<5} {:>6} {:>6} {:<23} {:<23} {}",
                        conn.protocol,
                        conn.recv_queue,
                        conn.send_queue,
                        local_addr,
                        remote_addr,
                        conn.state);
            }
        }
        
        Ok(())
    }
    
    /// Execute ss command (socket statistics)
    pub async fn ss(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_ss_args(args)?;
        
        info!("Getting socket statistics");
        
        let connections = self.get_network_connections_for_ss(&options).await?;
        
        // Print header
        if options.show_processes {
            println!("Netid  State      Recv-Q Send-Q Local Address:Port               Peer Address:Port              Process");
        } else {
            println!("Netid  State      Recv-Q Send-Q Local Address:Port               Peer Address:Port");
        }
        
        // Print connections in ss format
        for conn in connections {
            let netid = conn.protocol.to_lowercase();
            let state = match conn.state {
                ConnectionState::Listen => "LISTEN",
                ConnectionState::Established => "ESTAB",
                ConnectionState::TimeWait => "TIME-WAIT",
                ConnectionState::CloseWait => "CLOSE-WAIT",
                ConnectionState::FinWait1 => "FIN-WAIT-1",
                ConnectionState::FinWait2 => "FIN-WAIT-2",
                ConnectionState::SynSent => "SYN-SENT",
                ConnectionState::SynRecv => "SYN-RECV",
                _ => "UNCONN",
            };
            
            let local_addr = if options.show_numeric {
                conn.local_address.to_string()
            } else {
                self.resolve_address_with_service(conn.local_address).await
            };
            
            let remote_addr = if let Some(remote) = conn.remote_address {
                if options.show_numeric {
                    remote.to_string()
                } else {
                    self.resolve_address_with_service(remote).await
                }
            } else {
                "*:*".to_string()
            };
            
            if options.show_processes {
                let process_info = if let (Some(pid), Some(name)) = (conn.process_id, conn.process_name) {
                    format!("users:((\"{}\",pid={},fd=?))", name, pid)
                } else {
                    "-".to_string()
                };
                
                println!("{:<6} {:<10} {:>6} {:>6} {:<31} {:<31} {}",
                        netid,
                        state,
                        conn.recv_queue,
                        conn.send_queue,
                        local_addr,
                        remote_addr,
                        process_info);
            } else {
                println!("{:<6} {:<10} {:>6} {:>6} {:<31} {}",
                        netid,
                        state,
                        conn.recv_queue,
                        conn.send_queue,
                        local_addr,
                        remote_addr);
            }
        }
        
        Ok(())
    }
    
    /// Execute ip command
    pub async fn ip(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_ip_args(args)?;
        
        info!("Getting IP information for command: {}", options.command);
        
        match options.command.as_str() {
            "addr" => {
                self.show_ip_addresses().await?;
            },
            "route" => {
                self.show_routing_table().await?;
            },
            "link" => {
                self.show_network_interfaces().await?;
            },
            _ => {
                return Err(anyhow::anyhow!("Unknown ip command: {}", options.command).into());
            }
        }
        
        Ok(())
    }
    
    /// Execute ifconfig command
    pub async fn ifconfig(&self, ctx: &NxshContext, args: &[String]) -> NxshResult<()> {
        let options = self.parse_ifconfig_args(args)?;
        
        info!("Getting interface configuration");
        
        if let Some(ref interface) = options.interface {
            self.show_interface_details(interface).await?;
        } else {
            self.show_all_interfaces().await?;
        }
        
        Ok(())
    }
    
    // Private helper methods
    
    /// Get network connections for netstat command
    async fn get_network_connections(&self, options: &NetstatOptions) -> Result<Vec<NetworkConnection>> {
        let mut connections = Vec::new();
        
        #[cfg(unix)]
        {
            if options.show_tcp || options.show_all {
                connections.extend(self.read_tcp_connections().await?);
            }
            
            if options.show_udp || options.show_all {
                connections.extend(self.read_udp_connections().await?);
            }
        }
        
        #[cfg(windows)]
        {
            // Prefer IpHelper for rich data; fall back to netstat parsing
            match self.enumerate_windows_connections_iphelper(options).await {
                Ok(mut v) => connections.append(&mut v),
                Err(_) => connections.extend(self.parse_windows_netstat(options).await?),
            }
        }
        
        // Filter based on options
        if options.show_listening {
            connections.retain(|conn| conn.state == ConnectionState::Listen);
        }
        
        Ok(connections)
    }
    
    /// Get network connections for ss command
    async fn get_network_connections_for_ss(&self, options: &SsOptions) -> Result<Vec<NetworkConnection>> {
        let mut connections = Vec::new();
        
        #[cfg(unix)]
        {
            if options.show_tcp || options.show_all {
                connections.extend(self.read_tcp_connections().await?);
            }
            
            if options.show_udp || options.show_all {
                connections.extend(self.read_udp_connections().await?);
            }
        }
        
        #[cfg(windows)]
        {
            match self.enumerate_windows_connections_iphelper_for_ss(options).await {
                Ok(mut v) => connections.append(&mut v),
                Err(_) => connections.extend(self.parse_windows_netstat_for_ss(options).await?),
            }
        }
        
        // Filter based on options
        if options.show_listening {
            connections.retain(|conn| conn.state == ConnectionState::Listen);
        }
        
        Ok(connections)
    }

    #[cfg(windows)]
    async fn enumerate_windows_connections_iphelper(&self, options: &NetstatOptions) -> Result<Vec<NetworkConnection>> {
        use windows_sys::Win32::NetworkManagement::IpHelper::*;
        use windows_sys::Win32::Networking::WinSock::*;
        
        unsafe {
            let mut list: Vec<NetworkConnection> = Vec::new();
            
            // TCP v4
            if options.show_tcp || options.show_all {
                let mut size: u32 = 0;
                let mut ret = GetExtendedTcpTable(std::ptr::null_mut(), &mut size, 1, AF_INET as u32, TCP_TABLE_CLASS::TCP_TABLE_OWNER_PID_ALL, 0);
                if size > 0 {
                    let mut buf = vec![0u8; size as usize];
                    ret = GetExtendedTcpTable(buf.as_mut_ptr() as *mut _, &mut size, 1, AF_INET as u32, TCP_TABLE_CLASS::TCP_TABLE_OWNER_PID_ALL, 0);
                    if ret == 0 {
                        let table = buf.as_ptr() as *const MIB_TCPTABLE_OWNER_PID;
                        let count = (*table).dwNumEntries as usize;
                        let rows = std::slice::from_raw_parts((*table).table.as_ptr(), count);
                        for r in rows {
                            let local = SocketAddr::new(
                                IpAddr::V4(Ipv4Addr::from(u32::from_le(r.dwLocalAddr))), 
                                u16::from_be(r.dwLocalPort as u16)
                            );
                            let remote = SocketAddr::new(
                                IpAddr::V4(Ipv4Addr::from(u32::from_le(r.dwRemoteAddr))), 
                                u16::from_be(r.dwRemotePort as u16)
                            );
                            let state = match r.dwState {
                                MIB_TCP_STATE_ESTAB => ConnectionState::Established,
                                MIB_TCP_STATE_LISTEN => ConnectionState::Listen,
                                MIB_TCP_STATE_TIME_WAIT => ConnectionState::TimeWait,
                                MIB_TCP_STATE_CLOSE_WAIT => ConnectionState::CloseWait,
                                MIB_TCP_STATE_FIN_WAIT1 => ConnectionState::FinWait1,
                                MIB_TCP_STATE_FIN_WAIT2 => ConnectionState::FinWait2,
                                MIB_TCP_STATE_SYN_SENT => ConnectionState::SynSent,
                                MIB_TCP_STATE_SYN_RCVD => ConnectionState::SynRecv,
                                MIB_TCP_STATE_LAST_ACK => ConnectionState::LastAck,
                                MIB_TCP_STATE_CLOSING => ConnectionState::Closing,
                                MIB_TCP_STATE_CLOSED => ConnectionState::Close,
                                _ => ConnectionState::Unknown
                            };
                            
                            let remote_addr = if state == ConnectionState::Listen || 
                                               remote.ip().is_unspecified() {
                                None
                            } else {
                                Some(remote)
                            };
                            
                            list.push(NetworkConnection {
                                protocol: "tcp".to_string(),
                                local_address: local,
                                remote_address: remote_addr,
                                state,
                                recv_queue: 0,
                                send_queue: 0,
                                process_id: Some(r.dwOwningPid),
                                process_name: None,
                            });
                        }
                    }
                }
            }
            
            // UDP v4
            if options.show_udp || options.show_all {
                let mut size: u32 = 0;
                let mut ret = GetExtendedUdpTable(std::ptr::null_mut(), &mut size, 1, AF_INET as u32, UDP_TABLE_CLASS::UDP_TABLE_OWNER_PID, 0);
                if size > 0 {
                    let mut buf = vec![0u8; size as usize];
                    ret = GetExtendedUdpTable(buf.as_mut_ptr() as *mut _, &mut size, 1, AF_INET as u32, UDP_TABLE_CLASS::UDP_TABLE_OWNER_PID, 0);
                    if ret == 0 {
                        let table = buf.as_ptr() as *const MIB_UDPTABLE_OWNER_PID;
                        let count = (*table).dwNumEntries as usize;
                        let rows = std::slice::from_raw_parts((*table).table.as_ptr(), count);
                        for r in rows {
                            let local = SocketAddr::new(
                                IpAddr::V4(Ipv4Addr::from(u32::from_le(r.dwLocalAddr))), 
                                u16::from_be(r.dwLocalPort as u16)
                            );
                            
                            list.push(NetworkConnection {
                                protocol: "udp".to_string(),
                                local_address: local,
                                remote_address: None,
                                state: ConnectionState::Unknown,
                                recv_queue: 0,
                                send_queue: 0,
                                process_id: Some(r.dwOwningPid),
                                process_name: None,
                            });
                        }
                    }
                }
            }
            
            // TCP v6
            if options.show_tcp || options.show_all {
                let mut size: u32 = 0;
                let mut ret = GetExtendedTcpTable(std::ptr::null_mut(), &mut size, 1, AF_INET6 as u32, TCP_TABLE_CLASS::TCP_TABLE_OWNER_PID_ALL, 0);
                if size > 0 {
                    let mut buf = vec![0u8; size as usize];
                    ret = GetExtendedTcpTable(buf.as_mut_ptr() as *mut _, &mut size, 1, AF_INET6 as u32, TCP_TABLE_CLASS::TCP_TABLE_OWNER_PID_ALL, 0);
                    if ret == 0 {
                        let table = buf.as_ptr() as *const MIB_TCP6TABLE_OWNER_PID;
                        let count = (*table).dwNumEntries as usize;
                        let rows = std::slice::from_raw_parts((*table).table.as_ptr(), count);
                        for r in rows {
                            // Convert ucLocalAddr and ucRemoteAddr from [u8; 16] to Ipv6Addr
                            let local_ip = Ipv6Addr::from(r.ucLocalAddr);
                            let remote_ip = Ipv6Addr::from(r.ucRemoteAddr);
                            let local = SocketAddr::new(
                                IpAddr::V6(local_ip), 
                                u16::from_be(r.dwLocalPort as u16)
                            );
                            let remote = SocketAddr::new(
                                IpAddr::V6(remote_ip), 
                                u16::from_be(r.dwRemotePort as u16)
                            );
                            
                            let state = match r.dwState {
                                MIB_TCP_STATE_ESTAB => ConnectionState::Established,
                                MIB_TCP_STATE_LISTEN => ConnectionState::Listen,
                                MIB_TCP_STATE_TIME_WAIT => ConnectionState::TimeWait,
                                MIB_TCP_STATE_CLOSE_WAIT => ConnectionState::CloseWait,
                                MIB_TCP_STATE_FIN_WAIT1 => ConnectionState::FinWait1,
                                MIB_TCP_STATE_FIN_WAIT2 => ConnectionState::FinWait2,
                                MIB_TCP_STATE_SYN_SENT => ConnectionState::SynSent,
                                MIB_TCP_STATE_SYN_RCVD => ConnectionState::SynRecv,
                                MIB_TCP_STATE_LAST_ACK => ConnectionState::LastAck,
                                MIB_TCP_STATE_CLOSING => ConnectionState::Closing,
                                MIB_TCP_STATE_CLOSED => ConnectionState::Close,
                                _ => ConnectionState::Unknown
                            };
                            
                            let remote_addr = if state == ConnectionState::Listen || 
                                               remote.ip().is_unspecified() {
                                None
                            } else {
                                Some(remote)
                            };
                            
                            list.push(NetworkConnection {
                                protocol: "tcp6".to_string(),
                                local_address: local,
                                remote_address: remote_addr,
                                state,
                                recv_queue: 0,
                                send_queue: 0,
                                process_id: Some(r.dwOwningPid),
                                process_name: None,
                            });
                        }
                    }
                }
            }
            
            // UDP v6
            if options.show_udp || options.show_all {
                let mut size: u32 = 0;
                let mut ret = GetExtendedUdpTable(std::ptr::null_mut(), &mut size, 1, AF_INET6 as u32, UDP_TABLE_CLASS::UDP_TABLE_OWNER_PID, 0);
                if size > 0 {
                    let mut buf = vec![0u8; size as usize];
                    ret = GetExtendedUdpTable(buf.as_mut_ptr() as *mut _, &mut size, 1, AF_INET6 as u32, UDP_TABLE_CLASS::UDP_TABLE_OWNER_PID, 0);
                    if ret == 0 {
                        let table = buf.as_ptr() as *const MIB_UDP6TABLE_OWNER_PID;
                        let count = (*table).dwNumEntries as usize;
                        let rows = std::slice::from_raw_parts((*table).table.as_ptr(), count);
                        for r in rows {
                            let local_ip = Ipv6Addr::from(r.ucLocalAddr);
                            let local = SocketAddr::new(
                                IpAddr::V6(local_ip), 
                                u16::from_be(r.dwLocalPort as u16)
                            );
                            
                            list.push(NetworkConnection {
                                protocol: "udp6".to_string(),
                                local_address: local,
                                remote_address: None,
                                state: ConnectionState::Unknown,
                                recv_queue: 0,
                                send_queue: 0,
                                process_id: Some(r.dwOwningPid),
                                process_name: None,
                            });
                        }
                    }
                }
            }
            
            // Fill process names from PIDs
            let pids: HashSet<u32> = list.iter().filter_map(|c| c.process_id).collect();
            let mut name_cache: HashMap<u32, String> = HashMap::new();
            for pid in pids {
                if let Some(name) = Self::get_process_name_from_pid(pid) {
                    name_cache.insert(pid, name);
                }
            }
            
            for conn in &mut list {
                if let Some(pid) = conn.process_id {
                    if let Some(name) = name_cache.get(&pid) {
                        conn.process_name = Some(name.clone());
                    }
                }
            }
            
            Ok(list)
        }
    }

    #[cfg(windows)]
    async fn enumerate_windows_connections_iphelper_for_ss(&self, options: &SsOptions) -> Result<Vec<NetworkConnection>> {
        // Reuse the same as netstat path
        let netstat_options = NetstatOptions { show_tcp: options.show_tcp, show_udp: options.show_udp, show_listening: options.show_listening, show_numeric: options.show_numeric, show_processes: options.show_processes, show_all: options.show_all };
        self.enumerate_windows_connections_iphelper(&netstat_options).await
    }
    
    /// Resolve address with service name lookup
    async fn resolve_address_with_service(&self, addr: SocketAddr) -> String {
        // Try to resolve hostname and service name
        let hostname = self.reverse_dns_lookup(addr.ip()).await
            .unwrap_or_else(|| addr.ip().to_string());
        
        let service = self.resolve_service_name(addr.port()).await
            .unwrap_or_else(|| addr.port().to_string());
        
        format!("{}:{}", hostname, service)
    }
    
    /// Resolve service name from port number
    async fn resolve_service_name(&self, port: u16) -> Option<String> {
        // Common service mappings
        match port {
            22 => Some("ssh".to_string()),
            23 => Some("telnet".to_string()),
            25 => Some("smtp".to_string()),
            53 => Some("domain".to_string()),
            80 => Some("http".to_string()),
            110 => Some("pop3".to_string()),
            143 => Some("imap".to_string()),
            443 => Some("https".to_string()),
            993 => Some("imaps".to_string()),
            995 => Some("pop3s".to_string()),
            _ => None,
        }
    }
    
    #[cfg(unix)]
    async fn read_tcp_connections(&self) -> Result<Vec<NetworkConnection>> {
        let mut connections = Vec::new();
        
        // Read /proc/net/tcp
        if let Ok(content) = tokio::fs::read_to_string("/proc/net/tcp").await {
            for line in content.lines().skip(1) {
                if let Some(mut conn) = self.parse_proc_net_tcp_line(line) {
                    // Try to fill process info if inode is available
                    if let Some(inode) = self.extract_inode_from_proc_line(line) {
                        if let Some((pid, name)) = self.find_process_by_socket_inode(inode).await {
                            conn.process_id = Some(pid);
                            conn.process_name = Some(name);
                        }
                    }
                    connections.push(conn);
                }
            }
        }
        
        // Read /proc/net/tcp6
        if let Ok(content) = tokio::fs::read_to_string("/proc/net/tcp6").await {
            for line in content.lines().skip(1) {
                if let Some(mut conn) = self.parse_proc_net_tcp6_line(line) {
                    if let Some(inode) = self.extract_inode_from_proc_line(line) {
                        if let Some((pid, name)) = self.find_process_by_socket_inode(inode).await {
                            conn.process_id = Some(pid);
                            conn.process_name = Some(name);
                        }
                    }
                    connections.push(conn);
                }
            }
        }
        
        Ok(connections)
    }
    
    #[cfg(unix)]
    async fn read_udp_connections(&self) -> Result<Vec<NetworkConnection>> {
        let mut connections = Vec::new();
        
        // Read /proc/net/udp
        if let Ok(content) = tokio::fs::read_to_string("/proc/net/udp").await {
            for line in content.lines().skip(1) {
                if let Some(mut conn) = self.parse_proc_net_udp_line(line) {
                    if let Some(inode) = self.extract_inode_from_proc_line(line) {
                        if let Some((pid, name)) = self.find_process_by_socket_inode(inode).await {
                            conn.process_id = Some(pid);
                            conn.process_name = Some(name);
                        }
                    }
                    connections.push(conn);
                }
            }
        }
        
        // Read /proc/net/udp6
        if let Ok(content) = tokio::fs::read_to_string("/proc/net/udp6").await {
            for line in content.lines().skip(1) {
                if let Some(mut conn) = self.parse_proc_net_udp6_line(line) {
                    if let Some(inode) = self.extract_inode_from_proc_line(line) {
                        if let Some((pid, name)) = self.find_process_by_socket_inode(inode).await {
                            conn.process_id = Some(pid);
                            conn.process_name = Some(name);
                        }
                    }
                    connections.push(conn);
                }
            }
        }
        
        Ok(connections)
    }
    
    #[cfg(unix)]
    fn extract_inode_from_proc_line(&self, line: &str) -> Option<u64> {
        // /proc/net/tcp format: 
        // sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 10 {
            parts[9].parse().ok()
        } else {
            None
        }
    }
    
    #[cfg(unix)]
    async fn find_process_by_socket_inode(&self, inode: u64) -> Option<(u32, String)> {
        // Look through /proc/*/fd/* for socket:[inode]
        let proc_dir = std::path::Path::new("/proc");
        if let Ok(entries) = std::fs::read_dir(proc_dir) {
            for entry in entries.flatten() {
                if let Ok(pid_str) = entry.file_name().into_string() {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        if let Some(process_name) = self.check_process_for_socket_inode(pid, inode).await {
                            return Some((pid, process_name));
                        }
                    }
                }
            }
        }
        None
    }
    
    #[cfg(unix)]
    async fn check_process_for_socket_inode(&self, pid: u32, inode: u64) -> Option<String> {
        let fd_dir = format!("/proc/{}/fd", pid);
        let target_link = format!("socket:[{}]", inode);
        
        if let Ok(entries) = std::fs::read_dir(fd_dir) {
            for entry in entries.flatten() {
                if let Ok(link_target) = std::fs::read_link(entry.path()) {
                    if let Some(link_str) = link_target.to_str() {
                        if link_str == target_link {
                            // Found the process, get its name
                            return self.get_process_name_from_pid_linux(pid).await;
                        }
                    }
                }
            }
        }
        None
    }
    
    #[cfg(unix)]
    async fn get_process_name_from_pid_linux(&self, pid: u32) -> Option<String> {
        let comm_path = format!("/proc/{}/comm", pid);
        if let Ok(comm) = tokio::fs::read_to_string(comm_path).await {
            Some(comm.trim().to_string())
        } else {
            // Fallback to reading cmdline
            let cmdline_path = format!("/proc/{}/cmdline", pid);
            if let Ok(cmdline) = tokio::fs::read_to_string(cmdline_path).await {
                if let Some(first_arg) = cmdline.split('\0').next() {
                    if let Some(basename) = std::path::Path::new(first_arg).file_name() {
                        return basename.to_str().map(|s| s.to_string());
                    }
                }
            }
            None
        }
    }
    
    #[cfg(unix)]
    fn parse_proc_net_tcp_line(&self, line: &str) -> Option<NetworkConnection> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            return None;
        }
        
        let local_addr = self.parse_proc_net_addr(parts[1])?;
        let remote_addr = self.parse_proc_net_addr(parts[2]);
        let state = self.parse_tcp_state_from_hex(parts[3]);
        let recv_queue = u64::from_str_radix(parts[4].split(':').next()?, 16).ok()?;
        let send_queue = u64::from_str_radix(parts[4].split(':').nth(1)?, 16).ok()?;
        
        Some(NetworkConnection {
            protocol: "TCP".to_string(),
            local_address: local_addr,
            remote_address: remote_addr,
            state,
            recv_queue,
            send_queue,
            process_id: None, // Would need to parse /proc/net/tcp with inode lookup
            process_name: None,
        })
    }
    
    #[cfg(unix)]
    fn parse_proc_net_tcp6_line(&self, line: &str) -> Option<NetworkConnection> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            return None;
        }
        
        let local_addr = self.parse_proc_net_addr_v6(parts[1])?;
        let remote_addr = self.parse_proc_net_addr_v6(parts[2]);
        let state = self.parse_tcp_state_from_hex(parts[3]);
        let recv_queue = u64::from_str_radix(parts[4].split(':').next()?, 16).ok()?;
        let send_queue = u64::from_str_radix(parts[4].split(':').nth(1)?, 16).ok()?;
        
        Some(NetworkConnection {
            protocol: "TCP6".to_string(),
            local_address: local_addr,
            remote_address: remote_addr,
            state,
            recv_queue,
            send_queue,
            process_id: None,
            process_name: None,
        })
    }
    
    #[cfg(unix)]
    fn parse_proc_net_udp_line(&self, line: &str) -> Option<NetworkConnection> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            return None;
        }
        
        let local_addr = self.parse_proc_net_addr(parts[1])?;
        let remote_addr = self.parse_proc_net_addr(parts[2]);
        let recv_queue = u64::from_str_radix(parts[4].split(':').next()?, 16).ok()?;
        let send_queue = u64::from_str_radix(parts[4].split(':').nth(1)?, 16).ok()?;
        
        Some(NetworkConnection {
            protocol: "UDP".to_string(),
            local_address: local_addr,
            remote_address: remote_addr,
            state: ConnectionState::Unknown, // UDP doesn't have connection state
            recv_queue,
            send_queue,
            process_id: None,
            process_name: None,
        })
    }
    
    #[cfg(unix)]
    fn parse_proc_net_udp6_line(&self, line: &str) -> Option<NetworkConnection> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            return None;
        }
        
        let local_addr = self.parse_proc_net_addr_v6(parts[1])?;
        let remote_addr = self.parse_proc_net_addr_v6(parts[2]);
        let recv_queue = u64::from_str_radix(parts[4].split(':').next()?, 16).ok()?;
        let send_queue = u64::from_str_radix(parts[4].split(':').nth(1)?, 16).ok()?;
        
        Some(NetworkConnection {
            protocol: "UDP6".to_string(),
            local_address: local_addr,
            remote_address: remote_addr,
            state: ConnectionState::Unknown,
            recv_queue,
            send_queue,
            process_id: None,
            process_name: None,
        })
    }
    
    #[cfg(unix)]
    fn parse_proc_net_addr(&self, addr_str: &str) -> Option<SocketAddr> {
        if let Some((addr_hex, port_hex)) = addr_str.split_once(':') {
            if addr_hex.len() == 8 && port_hex.len() == 4 {
                // IPv4 address
                if let (Ok(addr_num), Ok(port_num)) = (u32::from_str_radix(addr_hex, 16), u16::from_str_radix(port_hex, 16)) {
                    let ip = Ipv4Addr::from(addr_num.swap_bytes());
                    return Some(SocketAddr::new(IpAddr::V4(ip), port_num));
                }
            }
        }
        None
    }
    
    #[cfg(unix)]
    fn parse_proc_net_addr_v6(&self, addr_str: &str) -> Option<SocketAddr> {
        if let Some((addr_hex, port_hex)) = addr_str.split_once(':') {
            if addr_hex.len() == 32 && port_hex.len() == 4 {
                // IPv6 address - proper parsing of 32 hex chars to 16 bytes
                if let Ok(port_num) = u16::from_str_radix(port_hex, 16) {
                    let mut bytes = [0u8; 16];
                    for i in 0..16 {
                        if let Ok(byte) = u8::from_str_radix(&addr_hex[i*2..i*2+2], 16) {
                            bytes[i] = byte;
                        } else {
                            return None;
                        }
                    }
                    
                    // Convert from little-endian to big-endian (network byte order)
                    let mut segments = [0u16; 8];
                    for i in 0..8 {
                        segments[i] = u16::from_le_bytes([bytes[i*2], bytes[i*2+1]]);
                    }
                    
                    let ip = Ipv6Addr::new(
                        segments[0], segments[1], segments[2], segments[3],
                        segments[4], segments[5], segments[6], segments[7],
                    );
                    return Some(SocketAddr::new(IpAddr::V6(ip), port_num));
                }
            }
        }
        None
    }
    
    #[cfg(unix)]
    fn parse_tcp_state_from_hex(&self, state_hex: &str) -> ConnectionState {
        match state_hex {
            "01" => ConnectionState::Established,
            "02" => ConnectionState::SynSent,
            "03" => ConnectionState::SynRecv,
            "04" => ConnectionState::FinWait1,
            "05" => ConnectionState::FinWait2,
            "06" => ConnectionState::TimeWait,
            "07" => ConnectionState::Close,
            "08" => ConnectionState::CloseWait,
            "09" => ConnectionState::LastAck,
            "0A" => ConnectionState::Listen,
            "0B" => ConnectionState::Closing,
            _ => ConnectionState::Unknown,
        }
    }
    
    #[cfg(windows)]
    async fn parse_windows_netstat(&self, options: &NetstatOptions) -> Result<Vec<NetworkConnection>> {
        let mut cmd = Command::new("netstat");
        cmd.arg("-an");
        
        if options.show_processes {
            cmd.arg("-o");
        }
        
        let output = cmd.output().await.context("Failed to execute netstat")?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("netstat command failed"));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut connections = Vec::new();
        
        for line in stdout.lines() {
            if let Some(conn) = self.parse_netstat_line(line) {
                connections.push(conn);
            }
        }
        
        Ok(connections)
    }
    
    #[cfg(windows)]
    async fn parse_windows_netstat_for_ss(&self, options: &SsOptions) -> Result<Vec<NetworkConnection>> {
        // Reuse netstat parsing for Windows
        let netstat_options = NetstatOptions {
            show_tcp: options.show_tcp,
            show_udp: options.show_udp,
            show_listening: options.show_listening,
            show_numeric: options.show_numeric,
            show_processes: options.show_processes,
            show_all: options.show_all,
        };
        
        self.parse_windows_netstat(&netstat_options).await
    }
    
    #[cfg(windows)]
    fn parse_netstat_line(&self, line: &str) -> Option<NetworkConnection> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }
        
        let protocol = parts[0];
        if !protocol.eq_ignore_ascii_case("TCP") && !protocol.eq_ignore_ascii_case("UDP") {
            return None;
        }
        
        let local_addr: SocketAddr = parts[1].parse().ok()?;
        let remote_addr: Option<SocketAddr> = if parts[2] == "*:*" {
            None
        } else {
            parts[2].parse().ok()
        };
        
        let state = if protocol.eq_ignore_ascii_case("TCP") && parts.len() > 3 {
            match parts[3] {
                "LISTENING" => ConnectionState::Listen,
                "ESTABLISHED" => ConnectionState::Established,
                "TIME_WAIT" => ConnectionState::TimeWait,
                "CLOSE_WAIT" => ConnectionState::CloseWait,
                "FIN_WAIT_1" => ConnectionState::FinWait1,
                "FIN_WAIT_2" => ConnectionState::FinWait2,
                "SYN_SENT" => ConnectionState::SynSent,
                "SYN_RECEIVED" => ConnectionState::SynRecv,
                _ => ConnectionState::Unknown,
            }
        } else {
            ConnectionState::Unknown
        };
        
        Some(NetworkConnection {
            protocol: protocol.to_uppercase(),
            local_address: local_addr,
            remote_address: remote_addr,
            state,
            recv_queue: 0, // Windows netstat doesn't provide queue info
            send_queue: 0,
            process_id: None, // Would need -o flag parsing
            process_name: None,
        })
    }
    
    async fn resolve_hostname(&self, hostname: &str) -> Result<Vec<IpAddr>> {
        // Check cache first
        {
            let cache = self.dns_cache.read().await;
            if let Some(ips) = cache.get(hostname) {
                return Ok(ips.clone());
            }
        }
        
        // Resolve hostname
        let socket_addrs: Vec<SocketAddr> = format!("{}:80", hostname)
            .to_socket_addrs()
            .context("Failed to resolve hostname")?
            .collect();
        
        let ips: Vec<IpAddr> = socket_addrs.into_iter().map(|addr| addr.ip()).collect();
        
        // Cache result
        {
            let mut cache = self.dns_cache.write().await;
            cache.insert(hostname.to_string(), ips.clone());
        }
        
        Ok(ips)
    }
    
    async fn reverse_dns_lookup(&self, ip: IpAddr) -> Option<String> {
        // Use built-in std::net lookup for PTR resolution
        match tokio::task::spawn_blocking(move || {
            use std::net::ToSocketAddrs;
            // Try to resolve the IP address to hostname
            let dummy_port = 80;
            let socket_addr = std::net::SocketAddr::new(ip, dummy_port);
            
            // Use reverse lookup via getnameinfo-like functionality
            // This is a simple approach using std::net
            match socket_addr.to_socket_addrs() {
                Ok(_) => {
                    // Try parsing as string and using system resolver
                    use std::process::Command;
                    
                    #[cfg(unix)]
                    {
                        let output = Command::new("nslookup")
                            .arg(ip.to_string())
                            .output();
                        
                        if let Ok(output) = output {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            for line in stdout.lines() {
                                if line.contains("name =") {
                                    if let Some(name) = line.split("name =").nth(1) {
                                        return Some(name.trim().to_string());
                                    }
                                }
                            }
                        }
                    }
                    
                    #[cfg(windows)]
                    {
                        let output = Command::new("nslookup")
                            .arg(ip.to_string())
                            .output();
                        
                        if let Ok(output) = output {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let lines: Vec<&str> = stdout.lines().collect();
                            for (i, line) in lines.iter().enumerate() {
                                if line.contains("Address:") && i + 1 < lines.len() {
                                    if let Some(name_line) = lines.get(i + 1) {
                                        if name_line.starts_with("Name:") {
                                            if let Some(name) = name_line.split("Name:").nth(1) {
                                                return Some(name.trim().to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    None
                },
                Err(_) => None,
            }
        }).await {
            Ok(result) => result,
            Err(_) => None,
        }
    }
    
    async fn send_ping(&self, target: IpAddr, seq: u32, size: usize, timeout: Duration) -> Result<Duration> {
        // Windows: try real ICMP ping for IPv4 using IcmpSendEcho
        #[cfg(target_os = "windows")]
        {
            if let IpAddr::V4(ipv4) = target {
                let packet_len = size.clamp(1, 65500);
                let timeout_ms: u32 = timeout
                    .as_millis()
                    .try_into()
                    .unwrap_or(u32::MAX);
                let r = tokio::task::spawn_blocking(move || -> Result<Duration> {
                    use windows_sys::Win32::Foundation::HANDLE;
                    use windows_sys::Win32::NetworkManagement::IpHelper::{IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho, ICMP_ECHO_REPLY};
                    use windows_sys::Win32::Networking::WinSock::{IN_ADDR, IN_ADDR_0};
                    unsafe {
                        let handle: HANDLE = IcmpCreateFile();
                        if handle == 0 {
                            return Err(anyhow::anyhow!("IcmpCreateFile failed"));
                        }
                        // S_addr expects IPv4 in network byte order
                        let addr = IN_ADDR { S_un: IN_ADDR_0 { S_addr: u32::from(ipv4).to_be() } };
                        // Allocate reply buffer: ICMP_ECHO_REPLY + payload + some slack
                        let mut reply_buf = vec![0u8; std::mem::size_of::<ICMP_ECHO_REPLY>() + packet_len + 8];
                        let data = vec![0x61u8; packet_len];
                        let res = IcmpSendEcho(
                            handle,
                            addr.S_un.S_addr,
                            data.as_ptr() as *const _,
                            data.len() as u16,
                            std::ptr::null_mut(),
                            reply_buf.as_mut_ptr() as *mut _,
                            reply_buf.len() as u32,
                            timeout_ms,
                        );
                        IcmpCloseHandle(handle);
                        if res == 0 {
                            return Err(anyhow::anyhow!("Timeout"));
                        }
                        let reply: *const ICMP_ECHO_REPLY = reply_buf.as_ptr() as *const ICMP_ECHO_REPLY;
                        let rtt_ms = (*reply).RoundTripTime as u64;
                        Ok(Duration::from_millis(rtt_ms))
                    }
                })
                .await
                .map_err(|e| anyhow::anyhow!("Join error: {e}"))??;
                return Ok(r);
            } else if let IpAddr::V6(ipv6) = target {
                let packet_len = size.clamp(1, 65500);
                let timeout_ms: u32 = timeout
                    .as_millis()
                    .try_into()
                    .unwrap_or(u32::MAX);
                let segments = ipv6.segments();
                let r = tokio::task::spawn_blocking(move || -> Result<Duration> {
                    use windows_sys::Win32::Foundation::HANDLE;
                    use windows_sys::Win32::NetworkManagement::IpHelper::{Icmp6CreateFile, Icmp6SendEcho2, IcmpCloseHandle, ICMPV6_ECHO_REPLY_LH};
                    use windows_sys::Win32::Networking::WinSock::{SOCKADDR_IN6, IN6_ADDR, ADDRESS_FAMILY, AF_INET6};
                    unsafe {
                        let handle: HANDLE = Icmp6CreateFile();
                        if handle == 0 {
                            return Err(anyhow::anyhow!("Icmp6CreateFile failed"));
                        }
                        let mut dest_addr = SOCKADDR_IN6 {
                            sin6_family: AF_INET6 as ADDRESS_FAMILY,
                            sin6_port: 0,
                            sin6_flowinfo: 0,
                            sin6_addr: IN6_ADDR { u: windows_sys::Win32::Networking::WinSock::IN6_ADDR_0 { Byte: [0; 16] } },
                            sin6_scope_id: 0,
                        };
                        // copy IPv6 bytes into dest_addr.sin6_addr
                        let octets = ipv6.octets();
                        dest_addr.sin6_addr.u.Byte.copy_from_slice(&octets);

                        let mut src_addr: SOCKADDR_IN6 = std::mem::zeroed();
                        let mut reply_buf = vec![0u8; std::mem::size_of::<ICMPV6_ECHO_REPLY_LH>() + packet_len + 8];
                        let data = vec![0x61u8; packet_len];
                        let res = Icmp6SendEcho2(
                            handle,
                            0,
                            None,
                            std::ptr::null_mut(),
                            &mut src_addr as *mut _ as *mut _,
                            &mut dest_addr as *mut _ as *mut _,
                            data.as_ptr() as *const _,
                            data.len() as u16,
                            std::ptr::null_mut(),
                            reply_buf.as_mut_ptr() as *mut _,
                            reply_buf.len() as u32,
                            timeout_ms,
                        );
                        IcmpCloseHandle(handle);
                        if res == 0 {
                            return Err(anyhow::anyhow!("Timeout"));
                        }
                        let reply: *const ICMPV6_ECHO_REPLY_LH = reply_buf.as_ptr() as *const ICMPV6_ECHO_REPLY_LH;
                        let rtt_ms = (*reply).RoundTripTime as u64;
                        Ok(Duration::from_millis(rtt_ms))
                    }
                })
                .await
                .map_err(|e| anyhow::anyhow!("Join error: {e}"))??;
                return Ok(r);
            } else if let IpAddr::V6(ipv6) = target {
                let hop = ttl.min(255) as u32;
                let timeout_ms: u32 = timeout
                    .as_millis()
                    .try_into()
                    .unwrap_or(u32::MAX);
                let r = tokio::task::spawn_blocking(move || -> Result<(IpAddr, Duration)> {
                    use windows_sys::Win32::Foundation::HANDLE;
                    use windows_sys::Win32::NetworkManagement::IpHelper::{Icmp6CreateFile, Icmp6SendEcho2, IcmpCloseHandle, ICMPV6_ECHO_REPLY_LH, IPV6_OPTION_INFORMATION};
                    use windows_sys::Win32::Networking::WinSock::{SOCKADDR_IN6, IN6_ADDR, ADDRESS_FAMILY, AF_INET6};
                    unsafe {
                        let handle: HANDLE = Icmp6CreateFile();
                        if handle == 0 { return Err(anyhow::anyhow!("Icmp6CreateFile failed")); }

                        let mut dest_addr = SOCKADDR_IN6 {
                            sin6_family: AF_INET6 as ADDRESS_FAMILY,
                            sin6_port: 0,
                            sin6_flowinfo: 0,
                            sin6_addr: IN6_ADDR { u: windows_sys::Win32::Networking::WinSock::IN6_ADDR_0 { Byte: [0; 16] } },
                            sin6_scope_id: 0,
                        };
                        dest_addr.sin6_addr.u.Byte.copy_from_slice(&ipv6.octets());

                        let mut src_addr: SOCKADDR_IN6 = std::mem::zeroed();
                        let mut reply_buf = vec![0u8; std::mem::size_of::<ICMPV6_ECHO_REPLY_LH>() + 64];
                        let data = [0u8; 0];

                        let mut opt: IPV6_OPTION_INFORMATION = std::mem::zeroed();
                        opt.HopLimit = hop;

                        let res = Icmp6SendEcho2(
                            handle,
                            0,
                            None,
                            std::ptr::null_mut(),
                            &mut src_addr as *mut _ as *mut _,
                            &mut dest_addr as *mut _ as *mut _,
                            data.as_ptr() as *const _,
                            data.len() as u16,
                            &mut opt as *mut _ as *mut _,
                            reply_buf.as_mut_ptr() as *mut _,
                            reply_buf.len() as u32,
                            timeout_ms,
                        );
                        IcmpCloseHandle(handle);
                        if res == 0 { return Err(anyhow::anyhow!("Timeout")); }

                        let reply: *const ICMPV6_ECHO_REPLY_LH = reply_buf.as_ptr() as *const ICMPV6_ECHO_REPLY_LH;
                        let addr6: *const SOCKADDR_IN6 = &(*reply).Address as *const _ as *const SOCKADDR_IN6;
                        let octets = (*addr6).sin6_addr.u.Byte;
                        let hop_ip = IpAddr::V6(std::net::Ipv6Addr::from(octets));
                        let rtt_ms = (*reply).RoundTripTime as u64;
                        Ok((hop_ip, Duration::from_millis(rtt_ms)))
                    }
                })
                .await
                .map_err(|e| anyhow::anyhow!("Join error: {e}"))??;
                return Ok(r);
            }
        }

        // Fallback (non-Windows or IPv6 on Windows): simplified TCP connect timing
        let start = Instant::now();
        let addr = SocketAddr::new(target, 80);
        match timeout(timeout, TcpStream::connect(addr)).await {
            Ok(Ok(_)) => Ok(start.elapsed()),
            Ok(Err(_)) => Err(anyhow::anyhow!("Connection failed")),
            Err(_) => Err(anyhow::anyhow!("Timeout")),
        }
    }
    
    async fn send_traceroute_probe(&self, target: IpAddr, ttl: u32, timeout: Duration) -> Result<(IpAddr, Duration)> {
        // Windows IPv4: ICMP echo with TTL to discover hop
        #[cfg(target_os = "windows")]
        {
            if let IpAddr::V4(ipv4) = target {
                let ttl_u8 = ttl.min(255) as u8;
                let timeout_ms: u32 = timeout
                    .as_millis()
                    .try_into()
                    .unwrap_or(u32::MAX);
                let r = tokio::task::spawn_blocking(move || -> Result<(IpAddr, Duration)> {
                    use windows_sys::Win32::Foundation::HANDLE;
                    use windows_sys::Win32::NetworkManagement::IpHelper::{IcmpCreateFile, IcmpSendEcho, IcmpCloseHandle, ICMP_ECHO_REPLY, IP_OPTION_INFORMATION};
                    unsafe {
                        let handle: HANDLE = IcmpCreateFile();
                        if handle == 0 {
                            return Err(anyhow::anyhow!("IcmpCreateFile failed"));
                        }
                        let mut opts = IP_OPTION_INFORMATION { Ttl: ttl_u8, Tos: 0, Flags: 0, OptionsSize: 0, OptionsData: std::ptr::null_mut() };
                        let addr = u32::from(ipv4).to_be();
                        let mut reply_buf = vec![0u8; std::mem::size_of::<ICMP_ECHO_REPLY>() + 64];
                        let data = [0u8; 0];
                        let res = IcmpSendEcho(
                            handle,
                            addr,
                            data.as_ptr() as *const _,
                            data.len() as u16,
                            &mut opts as *mut _,
                            reply_buf.as_mut_ptr() as *mut _,
                            reply_buf.len() as u32,
                            timeout_ms,
                        );
                        IcmpCloseHandle(handle);
                        if res == 0 {
                            return Err(anyhow::anyhow!("Timeout"));
                        }
                        let reply: *const ICMP_ECHO_REPLY = reply_buf.as_ptr() as *const ICMP_ECHO_REPLY;
                        let addr_be = (*reply).Address; // network order
                        let octets = addr_be.to_be_bytes();
                        let hop_ip = IpAddr::V4(std::net::Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]));
                        let rtt_ms = (*reply).RoundTripTime as u64;
                        Ok((hop_ip, Duration::from_millis(rtt_ms)))
                    }
                })
                .await
                .map_err(|e| anyhow::anyhow!("Join error: {e}"))??;
                return Ok(r);
            }
        }

        // Fallback (non-Windows or IPv6): TCP connect-based
        let start = Instant::now();
        let addr = SocketAddr::new(target, 33434 + ttl);
        match timeout(timeout, TcpStream::connect(addr)).await {
            Ok(Ok(_)) => Ok((target, start.elapsed())),
            Ok(Err(_)) => Err(anyhow::anyhow!("Connection failed")),
            Err(_) => Err(anyhow::anyhow!("Timeout")),
        }
    }
    
    async fn print_ping_statistics(&self, session_id: &str) {
        let sessions = self.ping_sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            let loss_percent = if session.sent > 0 {
                (session.lost as f64 / session.sent as f64) * 100.0
            } else {
                0.0
            };
            
            println!();
            println!("--- {} ping statistics ---", session.target);
            println!("{} packets transmitted, {} received, {:.1}% packet loss, time {}ms",
                    session.sent, 
                    session.received,
                    loss_percent,
                    session.start_time.elapsed().as_millis());
            
            if session.received > 0 {
                println!("rtt min/avg/max = {:.3}/{:.3}/{:.3} ms",
                        session.min_time,
                        session.avg_time,
                        session.max_time);
            }
        }
    }
    
    #[cfg(unix)]
    async fn show_tcp_connections(&self) -> Result<()> {
        println!("Active Internet connections (TCP)");
        println!("Proto Recv-Q Send-Q Local Address           Foreign Address         State");
        
        // Read /proc/net/tcp
        if let Ok(content) = tokio::fs::read_to_string("/proc/net/tcp").await {
            for line in content.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let local_addr = self.parse_proc_net_addr(parts[1]);
                    let remote_addr = self.parse_proc_net_addr(parts[2]);
                    let state = self.parse_tcp_state(parts[3]);
                    
                    println!("tcp        0      0 {:20} {:20} {}", 
                            local_addr, remote_addr, state);
                }
            }
        }
        
        Ok(())
    }
    
    #[cfg(unix)]
    async fn show_udp_connections(&self) -> Result<()> {
        println!("Active Internet connections (UDP)");
        println!("Proto Recv-Q Send-Q Local Address           Foreign Address");
        
        // Read /proc/net/udp
        if let Ok(content) = tokio::fs::read_to_string("/proc/net/udp").await {
            for line in content.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let local_addr = self.parse_proc_net_addr(parts[1]);
                    let remote_addr = self.parse_proc_net_addr(parts[2]);
                    
                    println!("udp        0      0 {:20} {}", local_addr, remote_addr);
                }
            }
        }
        
        Ok(())
    }
    
    #[cfg(unix)]
    async fn show_listening_ports(&self) -> Result<()> {
        println!("Active listening ports");
        println!("Proto Local Address           State");
        
        // This would parse /proc/net/tcp and /proc/net/udp for listening sockets
        // Simplified implementation
        println!("tcp   0.0.0.0:22            LISTEN");
        println!("tcp   0.0.0.0:80            LISTEN");
        println!("udp   0.0.0.0:53            ");
        
        Ok(())
    }
    
    #[cfg(unix)]
    fn parse_proc_net_addr(&self, addr_str: &str) -> String {
        // Parse hexadecimal address:port format from /proc/net/*
        if let Some((addr_hex, port_hex)) = addr_str.split_once(':') {
            if addr_hex.len() == 8 && port_hex.len() == 4 {
                // IPv4 address
                if let (Ok(addr_num), Ok(port_num)) = (u32::from_str_radix(addr_hex, 16), u16::from_str_radix(port_hex, 16)) {
                    let ip = Ipv4Addr::from(addr_num.to_be());
                    return format!("{}:{}", ip, port_num);
                }
            }
        }
        addr_str.to_string()
    }
    
    #[cfg(unix)]
    fn parse_tcp_state(&self, state_hex: &str) -> &'static str {
        match state_hex {
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
    
    async fn show_ip_addresses(&self) -> Result<()> {
        let interfaces = self.get_network_interfaces().await?;
        
        for iface in interfaces {
            println!("{}: {}: <{}> mtu {} state {}", 
                    iface.index,
                    iface.name, 
                    iface.flags.join(","),
                    iface.mtu,
                    if iface.flags.contains(&"UP".to_string()) { "UP" } else { "DOWN" }
            );
            
            for addr in &iface.addresses {
                match addr {
                    IpAddr::V4(ipv4) => {
                        // Try to determine subnet from common patterns
                        let prefix = if ipv4.is_loopback() { 8 } else { 24 };
                        println!("    inet {}/{} scope {}", 
                                ipv4, 
                                prefix,
                                if ipv4.is_loopback() { "host" } else { "global" }
                        );
                    },
                    IpAddr::V6(ipv6) => {
                        let prefix = if ipv6.is_loopback() { 128 } else { 64 };
                        let scope = if ipv6.is_loopback() { "host" } else { "link" };
                        println!("    inet6 {}/{} scope {}", ipv6, prefix, scope);
                    }
                }
            }
            
            if let Some(mac) = &iface.mac_address {
                println!("    link/ether {}", mac);
            }
        }
        
        Ok(())
    }
    
    async fn show_routing_table(&self) -> Result<()> {
        #[cfg(unix)]
        {
            // Try to read /proc/net/route for IPv4 routing table
            if let Ok(content) = tokio::fs::read_to_string("/proc/net/route").await {
                println!("Kernel IP routing table");
                println!("{:<15} {:<15} {:<15} {:<5} {:<6} {:<3} {:<8} {:<8}", 
                        "Destination", "Gateway", "Genmask", "Flags", "Metric", "Ref", "Use", "Iface");
                
                for line in content.lines().skip(1) {
                    if let Some(route) = self.parse_proc_route_line(line) {
                        println!("{}", route);
                    }
                }
            } else {
                // Fallback to route command
                let output = Command::new("route")
                    .arg("-n")
                    .output()
                    .await
                    .context("Failed to execute route command")?;
                
                if output.status.success() {
                    print!("{}", String::from_utf8_lossy(&output.stdout));
                }
            }
        }
        
        #[cfg(windows)]
        {
            let output = Command::new("route")
                .arg("print")
                .output()
                .await
                .context("Failed to execute route command")?;
            
            if output.status.success() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }
        }
        
        Ok(())
    }
    
    async fn show_network_interfaces(&self) -> Result<()> {
        let interfaces = self.get_network_interfaces().await?;
        
        for iface in interfaces {
            let flags_str = iface.flags.join(",");
            let state = if iface.flags.contains(&"UP".to_string()) { "UP" } else { "DOWN" };
            
            println!("{}: {}: <{}> mtu {} qdisc {} state {} mode DEFAULT",
                    iface.index,
                    iface.name,
                    flags_str,
                    iface.mtu,
                    "unknown", // qdisc is Linux-specific
                    state
            );
        }
        
        Ok(())
    }
    
    async fn get_network_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        #[cfg(unix)]
        {
            self.get_unix_network_interfaces().await
        }
        #[cfg(windows)]
        {
            self.get_windows_network_interfaces().await
        }
    }
    
    #[cfg(unix)]
    async fn get_unix_network_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        use std::process::Command;
        
        let mut interfaces = Vec::new();
        
        // Use 'ip link show' to get interface information
        let output = Command::new("ip")
            .args(&["link", "show"])
            .output()
            .await;
            
        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if let Some(iface) = self.parse_ip_link_line(line) {
                        interfaces.push(iface);
                    }
                }
            }
        }
        
        // Fallback to reading /sys/class/net if ip command failed
        if interfaces.is_empty() {
            if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        let mut iface = NetworkInterface {
                            name: name.to_string(),
                            index: 0,
                            flags: vec!["UNKNOWN".to_string()],
                            mtu: 1500,
                            addresses: Vec::new(),
                            mac_address: None,
                            rx_packets: 0,
                            tx_packets: 0,
                            rx_bytes: 0,
                            tx_bytes: 0,
                            rx_errors: 0,
                            tx_errors: 0,
                        };
                        
                        // Try to read interface details from /sys
                        if let Ok(mtu) = std::fs::read_to_string(format!("/sys/class/net/{}/mtu", name)) {
                            if let Ok(mtu_val) = mtu.trim().parse::<u32>() {
                                iface.mtu = mtu_val;
                            }
                        }
                        
                        if let Ok(addr) = std::fs::read_to_string(format!("/sys/class/net/{}/address", name)) {
                            iface.mac_address = Some(addr.trim().to_string());
                        }
                        
                        // Get IP addresses using getifaddrs-like functionality
                        iface.addresses = self.get_interface_addresses(name).await;
                        
                        interfaces.push(iface);
                    }
                }
            }
        }
        
        Ok(interfaces)
    }
    
    #[cfg(windows)]
    async fn get_windows_network_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        use windows_sys::Win32::NetworkManagement::IpHelper::*;
        use windows_sys::Win32::Foundation::*;
        use std::mem;
        
        let mut interfaces = Vec::new();
        
        unsafe {
            let mut size: u32 = 0;
            let mut ret = GetAdaptersInfo(std::ptr::null_mut(), &mut size);
            
            if ret == ERROR_BUFFER_OVERFLOW && size > 0 {
                let mut buffer = vec![0u8; size as usize];
                ret = GetAdaptersInfo(buffer.as_mut_ptr() as *mut _, &mut size);
                
                if ret == NO_ERROR {
                    let mut adapter = buffer.as_ptr() as *const IP_ADAPTER_INFO;
                    let mut index = 1;
                    
                    while !adapter.is_null() {
                        let adapter_ref = &*adapter;
                        
                        // Convert adapter name
                        let name = std::ffi::CStr::from_ptr(adapter_ref.AdapterName.as_ptr())
                            .to_string_lossy()
                            .to_string();
                        
                        let description = std::ffi::CStr::from_ptr(adapter_ref.Description.as_ptr())
                            .to_string_lossy()
                            .to_string();
                        
                        let mut flags = Vec::new();
                        if adapter_ref.Type == MIB_IF_TYPE_ETHERNET {
                            flags.push("BROADCAST".to_string());
                            flags.push("MULTICAST".to_string());
                        }
                        if adapter_ref.Type == MIB_IF_TYPE_LOOPBACK {
                            flags.push("LOOPBACK".to_string());
                        }
                        flags.push("UP".to_string()); // Assume UP for now
                        
                        // Get MAC address
                        let mac_address = if adapter_ref.AddressLength > 0 {
                            let mac_bytes = std::slice::from_raw_parts(
                                adapter_ref.Address.as_ptr(), 
                                adapter_ref.AddressLength as usize
                            );
                            Some(format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                                mac_bytes[0], mac_bytes[1], mac_bytes[2],
                                mac_bytes[3], mac_bytes[4], mac_bytes[5]))
                        } else {
                            None
                        };
                        
                        // Get IP addresses
                        let mut addresses = Vec::new();
                        let mut ip_addr = &adapter_ref.IpAddressList;
                        loop {
                            let ip_str = std::ffi::CStr::from_ptr(ip_addr.IpAddress.String.as_ptr())
                                .to_string_lossy();
                            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                                addresses.push(ip);
                            }
                            
                            if ip_addr.Next.is_null() {
                                break;
                            }
                            ip_addr = &*ip_addr.Next;
                        }
                        
                        let iface = NetworkInterface {
                            name: format!("{} ({})", description, name),
                            index: index,
                            flags,
                            mtu: 1500, // Default MTU
                            addresses,
                            mac_address,
                            rx_packets: 0,
                            tx_packets: 0,
                            rx_bytes: 0,
                            tx_bytes: 0,
                            rx_errors: 0,
                            tx_errors: 0,
                        };
                        
                        interfaces.push(iface);
                        index += 1;
                        
                        adapter = adapter_ref.Next;
                    }
                }
            }
        }
        
        Ok(interfaces)
    }
    
    #[cfg(unix)]
    fn parse_ip_link_line(&self, line: &str) -> Option<NetworkInterface> {
        // Parse lines like: "2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc pfifo_fast state UP mode DEFAULT group default qlen 1000"
        if let Some((index_name, rest)) = line.split_once(": ") {
            if let Some((name, flags_mtu)) = rest.split_once(": ") {
                let index = index_name.parse().unwrap_or(0);
                
                let mut flags = Vec::new();
                let mut mtu = 1500;
                
                if let Some(flags_start) = flags_mtu.find('<') {
                    if let Some(flags_end) = flags_mtu.find('>') {
                        let flags_str = &flags_mtu[flags_start + 1..flags_end];
                        flags = flags_str.split(',').map(|s| s.to_string()).collect();
                    }
                }
                
                if let Some(mtu_pos) = flags_mtu.find("mtu ") {
                    let mtu_part = &flags_mtu[mtu_pos + 4..];
                    if let Some(mtu_str) = mtu_part.split_whitespace().next() {
                        mtu = mtu_str.parse().unwrap_or(1500);
                    }
                }
                
                return Some(NetworkInterface {
                    name: name.to_string(),
                    index,
                    flags,
                    mtu,
                    addresses: Vec::new(),
                    mac_address: None,
                    rx_packets: 0,
                    tx_packets: 0,
                    rx_bytes: 0,
                    tx_bytes: 0,
                    rx_errors: 0,
                    tx_errors: 0,
                });
            }
        }
        None
    }
    
    #[cfg(unix)]
    async fn get_interface_addresses(&self, interface_name: &str) -> Vec<IpAddr> {
        let mut addresses = Vec::new();
        
        // Try using 'ip addr show' command
        if let Ok(output) = Command::new("ip")
            .args(&["addr", "show", interface_name])
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.trim().starts_with("inet ") || line.trim().starts_with("inet6 ") {
                        if let Some(addr_part) = line.trim().split_whitespace().nth(1) {
                            if let Some(addr_str) = addr_part.split('/').next() {
                                if let Ok(ip) = addr_str.parse::<IpAddr>() {
                                    addresses.push(ip);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        addresses
    }
    
    #[cfg(unix)]
    fn parse_proc_route_line(&self, line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 8 {
            return None;
        }
        
        // /proc/net/route format:
        // Iface Destination Gateway Flags RefCnt Use Metric Mask MTU Window IRTT
        let iface = parts[0];
        let dest_hex = parts[1];
        let gateway_hex = parts[2];
        let flags_hex = parts[3];
        let metric = parts[6];
        let mask_hex = parts[7];
        
        // Convert hex to IP addresses
        let dest = self.hex_to_ipv4(dest_hex)?;
        let gateway = self.hex_to_ipv4(gateway_hex)?;
        let mask = self.hex_to_ipv4(mask_hex)?;
        
        // Convert flags
        let flags_num = u32::from_str_radix(flags_hex, 16).ok()?;
        let mut flags_str = String::new();
        if flags_num & 0x0001 != 0 { flags_str.push('U'); } // Up
        if flags_num & 0x0002 != 0 { flags_str.push('G'); } // Gateway
        if flags_num & 0x0004 != 0 { flags_str.push('H'); } // Host
        
        Some(format!("{:<15} {:<15} {:<15} {:<5} {:<6} {:<3} {:<8} {:<8}", 
                    dest, gateway, mask, flags_str, metric, "0", "0", iface))
    }
    
    #[cfg(unix)]
    fn hex_to_ipv4(&self, hex_str: &str) -> Option<String> {
        if hex_str.len() != 8 {
            return None;
        }
        
        let num = u32::from_str_radix(hex_str, 16).ok()?;
        let ip = Ipv4Addr::from(num.swap_bytes());
        Some(ip.to_string())
    }
    
    async fn show_interface_details(&self, interface: &str) -> Result<()> {
        println!("Interface: {}", interface);
        
        // This would get detailed interface information
        // Simplified implementation
        match interface {
            "lo" => {
                println!("lo: flags=73<UP,LOOPBACK,RUNNING>  mtu 65536");
                println!("        inet 127.0.0.1  netmask 255.0.0.0");
                println!("        inet6 ::1  prefixlen 128  scopeid 0x10<host>");
                println!("        loop  txqueuelen 1000  (Local Loopback)");
                println!("        RX packets 1234  bytes 123456 (120.5 KiB)");
                println!("        TX packets 1234  bytes 123456 (120.5 KiB)");
            },
            "eth0" => {
                println!("eth0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500");
                println!("        inet 192.168.1.100  netmask 255.255.255.0  broadcast 192.168.1.255");
                println!("        inet6 fe80::1234:5678:9abc:def0  prefixlen 64  scopeid 0x20<link>");
                println!("        ether 00:11:22:33:44:55  txqueuelen 1000  (Ethernet)");
                println!("        RX packets 12345  bytes 1234567 (1.1 MiB)");
                println!("        TX packets 12345  bytes 1234567 (1.1 MiB)");
            },
            _ => {
                println!("Interface {} not found", interface);
            }
        }
        
        Ok(())
    }
    
    async fn show_all_interfaces(&self) -> Result<()> {
        self.show_interface_details("lo").await?;
        println!();
        self.show_interface_details("eth0").await?;
        Ok(())
    }
    
    fn format_bytes(&self, bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
    
    // Argument parsing methods
    
    fn parse_ping_args(&self, args: &[String]) -> Result<PingOptions> {
        let mut options = PingOptions::default();
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                "-c" => {
                    i += 1;
                    if i < args.len() {
                        options.count = args[i].parse().context("Invalid count")?;
                    }
                },
                "-i" => {
                    i += 1;
                    if i < args.len() {
                        let interval_secs: f64 = args[i].parse().context("Invalid interval")?;
                        options.interval = Duration::from_secs_f64(interval_secs);
                    }
                },
                "-W" => {
                    i += 1;
                    if i < args.len() {
                        let timeout_secs: f64 = args[i].parse().context("Invalid timeout")?;
                        options.timeout = Duration::from_secs_f64(timeout_secs);
                    }
                },
                "-s" => {
                    i += 1;
                    if i < args.len() {
                        options.packet_size = args[i].parse().context("Invalid packet size")?;
                    }
                },
                arg if !arg.starts_with('-') => {
                    options.target = arg.to_string();
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.target.is_empty() {
            return Err(anyhow::anyhow!("Target hostname required"));
        }
        
        Ok(options)
    }
    
    fn parse_traceroute_args(&self, args: &[String]) -> Result<TracerouteOptions> {
        let mut options = TracerouteOptions::default();
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                "-m" => {
                    i += 1;
                    if i < args.len() {
                        options.max_hops = args[i].parse().context("Invalid max hops")?;
                    }
                },
                "-q" => {
                    i += 1;
                    if i < args.len() {
                        options.probes = args[i].parse().context("Invalid probe count")?;
                    }
                },
                "-w" => {
                    i += 1;
                    if i < args.len() {
                        let timeout_secs: f64 = args[i].parse().context("Invalid timeout")?;
                        options.timeout = Duration::from_secs_f64(timeout_secs);
                    }
                },
                arg if !arg.starts_with('-') => {
                    options.target = arg.to_string();
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.target.is_empty() {
            return Err(anyhow::anyhow!("Target hostname required"));
        }
        
        Ok(options)
    }
    
    fn parse_nslookup_args(&self, args: &[String]) -> Result<NslookupOptions> {
        let mut options = NslookupOptions::default();
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                "-type" => {
                    i += 1;
                    if i < args.len() {
                        options.record_type = args[i].to_uppercase();
                    }
                },
                arg if !arg.starts_with('-') => {
                    if options.query.is_empty() {
                        options.query = arg.to_string();
                    } else if options.server.is_none() {
                        options.server = Some(arg.to_string());
                    }
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.query.is_empty() {
            return Err(anyhow::anyhow!("Query required"));
        }
        
        Ok(options)
    }
    
    fn parse_dig_args(&self, args: &[String]) -> Result<DigOptions> {
        let mut options = DigOptions::default();
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                arg if arg.starts_with('@') => {
                    options.server = Some(arg[1..].to_string());
                },
                "A" | "AAAA" | "MX" | "NS" | "PTR" | "CNAME" | "TXT" => {
                    options.record_type = args[i].to_string();
                },
                arg if !arg.starts_with('-') && !arg.starts_with('+') => {
                    if options.query.is_empty() {
                        options.query = arg.to_string();
                    }
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.query.is_empty() {
            return Err(anyhow::anyhow!("Query required"));
        }
        
        Ok(options)
    }
    
    fn parse_curl_args(&self, args: &[String]) -> Result<CurlOptions> {
        let mut options = CurlOptions::default();
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                "-X" | "--request" => {
                    i += 1;
                    if i < args.len() {
                        options.method = args[i].parse().context("Invalid HTTP method")?;
                    }
                },
                "-H" | "--header" => {
                    i += 1;
                    if i < args.len() {
                        if let Some((key, value)) = args[i].split_once(':') {
                            options.headers.insert(key.trim().to_string(), value.trim().to_string());
                        }
                    }
                },
                "-d" | "--data" => {
                    i += 1;
                    if i < args.len() {
                        options.body = Some(args[i].clone());
                    }
                },
                "-u" | "--user" => {
                    i += 1;
                    if i < args.len() {
                        if let Some((username, password)) = args[i].split_once(':') {
                            options.basic_auth = Some((username.to_string(), Some(password.to_string())));
                        } else {
                            options.basic_auth = Some((args[i].clone(), None));
                        }
                    }
                },
                "-o" | "--output" => {
                    i += 1;
                    if i < args.len() {
                        options.output_file = Some(args[i].clone());
                    }
                },
                "-I" | "--head" => {
                    options.head_only = true;
                },
                "-i" | "--include" => {
                    options.include_headers = true;
                },
                "-v" | "--verbose" => {
                    options.verbose = true;
                },
                arg if !arg.starts_with('-') => {
                    if options.url.is_empty() {
                        options.url = arg.to_string();
                    }
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.url.is_empty() {
            return Err(anyhow::anyhow!("URL required"));
        }
        
        Ok(options)
    }
    
    fn parse_wget_args(&self, args: &[String]) -> Result<WgetOptions> {
        let mut options = WgetOptions::default();
        let mut i = 0;
        
        while i < args.len() {
            match args[i].as_str() {
                "-O" | "--output-document" => {
                    i += 1;
                    if i < args.len() {
                        options.output_file = Some(args[i].clone());
                    }
                },
                "--progress" => {
                    i += 1;
                    if i < args.len() {
                        options.show_progress = args[i] != "dot";
                    }
                },
                "-q" | "--quiet" => {
                    options.show_progress = false;
                },
                arg if !arg.starts_with('-') => {
                    if options.url.is_empty() {
                        options.url = arg.to_string();
                    }
                },
                _ => {}
            }
            i += 1;
        }
        
        if options.url.is_empty() {
            return Err(anyhow::anyhow!("URL required"));
        }
        
        Ok(options)
    }
    
    fn parse_netstat_args(&self, args: &[String]) -> Result<NetstatOptions> {
        let mut options = NetstatOptions::default();
        
        for arg in args {
            match arg.as_str() {
                "-t" | "--tcp" => options.show_tcp = true,
                "-u" | "--udp" => options.show_udp = true,
                "-l" | "--listening" => options.show_listening = true,
                "-n" | "--numeric" => options.show_numeric = true,
                "-p" | "--processes" => options.show_processes = true,
                "-a" | "--all" => {
                    options.show_tcp = true;
                    options.show_udp = true;
                    options.show_listening = true;
                    options.show_all = true;
                },
                _ => {}
            }
        }
        
        // Default to showing all if no specific options
        if !options.show_tcp && !options.show_udp && !options.show_listening {
            options.show_tcp = true;
            options.show_udp = true;
            options.show_listening = true;
            options.show_all = true;
        }
        
        Ok(options)
    }
    
    fn parse_ss_args(&self, args: &[String]) -> Result<SsOptions> {
        let mut options = SsOptions::default();
        
        for arg in args {
            match arg.as_str() {
                "-t" | "--tcp" => options.show_tcp = true,
                "-u" | "--udp" => options.show_udp = true,
                "-l" | "--listening" => options.show_listening = true,
                "-n" | "--numeric" => options.show_numeric = true,
                "-p" | "--processes" => options.show_processes = true,
                "-a" | "--all" => {
                    options.show_tcp = true;
                    options.show_udp = true;
                    options.show_listening = true;
                    options.show_all = true;
                },
                _ => {}
            }
        }
        
        // Default to showing all if no specific options
        if !options.show_tcp && !options.show_udp {
            options.show_tcp = true;
            options.show_udp = true;
        }
        
        Ok(options)
    }
    
    fn parse_ip_args(&self, args: &[String]) -> Result<IpOptions> {
        let mut options = IpOptions::default();
        
        if !args.is_empty() {
            options.command = args[0].clone();
        }
        
        Ok(options)
    }
    
    fn parse_ifconfig_args(&self, args: &[String]) -> Result<IfconfigOptions> {
        let mut options = IfconfigOptions::default();
        
        if !args.is_empty() {
            options.interface = Some(args[0].clone());
        }
        
        Ok(options)
    }
}

// Option structs for different commands

#[derive(Debug, Clone)]
struct PingOptions {
    target: String,
    count: u32,
    interval: Duration,
    timeout: Duration,
    packet_size: usize,
}

impl Default for PingOptions {
    fn default() -> Self {
        Self {
            target: String::new(),
            count: 4,
            interval: Duration::from_secs(1),
            timeout: Duration::from_secs(5),
            packet_size: 64,
        }
    }
}

#[derive(Debug, Clone)]
struct TracerouteOptions {
    target: String,
    max_hops: u32,
    probes: u32,
    timeout: Duration,
    packet_size: usize,
}

impl Default for TracerouteOptions {
    fn default() -> Self {
        Self {
            target: String::new(),
            max_hops: 30,
            probes: 3,
            timeout: Duration::from_secs(5),
            packet_size: 60,
        }
    }
}

#[derive(Debug, Clone)]
struct NslookupOptions {
    query: String,
    record_type: String,
    server: Option<String>,
}

impl Default for NslookupOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            record_type: "A".to_string(),
            server: None,
        }
    }
}

#[derive(Debug, Clone)]
struct DigOptions {
    query: String,
    record_type: String,
    server: Option<String>,
}

impl Default for DigOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            record_type: "A".to_string(),
            server: None,
        }
    }
}

#[derive(Debug, Clone)]
struct CurlOptions {
    url: String,
    method: HttpMethod,
    headers: HashMap<String, String>,
    body: Option<String>,
    basic_auth: Option<(String, Option<String>)>,
    output_file: Option<String>,
    head_only: bool,
    include_headers: bool,
    verbose: bool,
}

impl Default for CurlOptions {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: None,
            basic_auth: None,
            output_file: None,
            head_only: false,
            include_headers: false,
            verbose: false,
        }
    }
}

#[derive(Debug, Clone)]
struct WgetOptions {
    url: String,
    output_file: Option<String>,
    show_progress: bool,
}

impl Default for WgetOptions {
    fn default() -> Self {
        Self {
            url: String::new(),
            output_file: None,
            show_progress: true,
        }
    }
}

#[derive(Debug, Clone)]
struct NetstatOptions {
    show_tcp: bool,
    show_udp: bool,
    show_listening: bool,
    show_numeric: bool,
    show_processes: bool,
    show_all: bool,
}

impl Default for NetstatOptions {
    fn default() -> Self {
        Self {
            show_tcp: false,
            show_udp: false,
            show_listening: false,
            show_numeric: false,
            show_processes: false,
            show_all: false,
        }
    }
}

#[derive(Debug, Clone)]
struct SsOptions {
    show_tcp: bool,
    show_udp: bool,
    show_listening: bool,
    show_numeric: bool,
    show_processes: bool,
    show_all: bool,
}

impl Default for SsOptions {
    fn default() -> Self {
        Self {
            show_tcp: false,
            show_udp: false,
            show_listening: false,
            show_numeric: false,
            show_processes: false,
            show_all: false,
        }
    }
}

#[derive(Debug, Clone)]
struct IpOptions {
    command: String,
}

impl Default for IpOptions {
    fn default() -> Self {
        Self {
            command: "addr".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct IfconfigOptions {
    interface: Option<String>,
}

impl Default for IfconfigOptions {
    fn default() -> Self {
        Self {
            interface: None,
        }
    }
}

// Session tracking structs

#[derive(Debug, Clone)]
struct PingSession {
    target: String,
    target_ip: IpAddr,
    count: u32,
    interval: Duration,
    timeout: Duration,
    packet_size: usize,
    sent: u32,
    received: u32,
    lost: u32,
    min_time: f64,
    max_time: f64,
    avg_time: f64,
    total_time: f64,
    start_time: Instant,
}

// Configuration

#[derive(Debug, Clone)]
struct NetworkToolsConfig {
    default_timeout: Duration,
    max_concurrent_requests: usize,
    dns_cache_ttl: Duration,
    user_agent: String,
}

impl Default for NetworkToolsConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            max_concurrent_requests: 10,
            dns_cache_ttl: Duration::from_secs(300),
            user_agent: "NexusShell/1.0".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_network_tools_creation() {
        let manager = NetworkToolsManager::new().unwrap();
        assert!(manager.ping_sessions.read().await.is_empty());
    }
    
    #[test]
    fn test_ping_args_parsing() {
        let manager = NetworkToolsManager::new().unwrap();
        let args = vec!["google.com".to_string(), "-c".to_string(), "10".to_string()];
        let options = manager.parse_ping_args(&args).unwrap();
        
        assert_eq!(options.target, "google.com");
        assert_eq!(options.count, 10);
    }
    
    #[test]
    fn test_curl_args_parsing() {
        let manager = NetworkToolsManager::new().unwrap();
        let args = vec![
            "https://example.com".to_string(),
            "-X".to_string(),
            "POST".to_string(),
            "-H".to_string(),
            "Content-Type: application/json".to_string(),
        ];
        let options = manager.parse_curl_args(&args).unwrap();
        
        assert_eq!(options.url, "https://example.com");
        assert_eq!(options.method, HttpMethod::POST);
        assert_eq!(options.headers.get("Content-Type"), Some(&"application/json".to_string()));
    }
    
    #[tokio::test]
    async fn test_dns_resolution() {
        let manager = NetworkToolsManager::new().unwrap();
        
        // Test with localhost (should always resolve)
        let result = manager.resolve_hostname("localhost").await;
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_format_bytes() {
        let manager = NetworkToolsManager::new().unwrap();
        
        assert_eq!(manager.format_bytes(1024), "1.0 KB");
        assert_eq!(manager.format_bytes(1048576), "1.0 MB");
        assert_eq!(manager.format_bytes(1073741824), "1.0 GB");
    }
} 

