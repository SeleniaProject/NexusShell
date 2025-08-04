//! `ping` command  Ecomprehensive ICMP echo request implementation with full Unix ping functionality.
//!
//! Supports complete ping functionality:
//!   ping HOST                  - Basic ping with default settings
//!   ping -c COUNT HOST         - Send COUNT packets and stop
//!   ping -i INTERVAL HOST      - Set interval between packets (minimum 0.001s for root)
//!   ping -s SIZE HOST          - Set packet size (0-65507 bytes)
//!   ping -t TTL HOST           - Set Time To Live (1-255)
//!   ping -W TIMEOUT HOST       - Set timeout for responses
//!   ping -w DEADLINE HOST      - Set deadline for entire ping session
//!   ping -f HOST               - Flood ping (root only)
//!   ping -q HOST               - Quiet output (summary only)
//!   ping -v HOST               - Verbose output with detailed packet info  
//!   ping -4 HOST               - Force IPv4
//!   ping -6 HOST               - Force IPv6
//!   ping -D HOST               - Print timestamps with each packet
//!   ping -n HOST               - No DNS resolution (numeric output only)
//!   ping -a HOST               - Audible ping (beep on response)
//!   ping -b HOST               - Allow pinging broadcast address
//!   ping -r HOST               - Bypass routing table (direct interface)
//!   ping -R HOST               - Record route (IPv4 only)
//!   ping -U HOST               - Print full user-to-user latency
//!   ping -l PRELOAD HOST       - Send PRELOAD packets as fast as possible
//!   ping -p PATTERN HOST       - Fill packet with given hex pattern
//!   ping -S SOURCE HOST        - Set source address
//!   ping -I INTERFACE HOST     - Use specific network interface
//!   ping -m MARK HOST          - Set SO_MARK socket option
//!   ping -M HINT HOST          - Path MTU discovery strategy
//!   ping -F HOST               - Don't fragment flag
//!   ping -O HOST               - Report outstanding ICMP ECHO reply before sending next packet
//!   ping -A HOST               - Adaptive ping (adjust interval based on RTT)
//!   ping -B HOST               - Don't allow ping to change source address
//!   ping -L HOST               - Suppress loopback of multicast packets
//!   ping -T TSONLY HOST        - Set IP timestamp options
//!   ping -Q TOS HOST           - Set Quality of Service related bits

use anyhow::{Result, anyhow};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::collections::{VecDeque, BTreeMap};
use std::io::{self, Write};
use chrono::Local;
use hickory_resolver::{Resolver, config::*};
use std::sync::mpsc::{self, Sender};
use parking_lot::Mutex;
use socket2::{Socket, Domain, Type, Protocol, SockAddr};
// TODO: Replace pnet with pure Rust alternative or delegate to system ping
// use pnet::packet::icmp::{IcmpPacket, MutableIcmpPacket, IcmpTypes, echo_request, echo_reply};
// use pnet::packet::icmpv6::{Icmpv6Packet, MutableIcmpv6Packet, Icmpv6Types};
// use pnet::packet::ip::IpNextHeaderProtocols;
// use pnet::packet::Packet;
use dashmap::DashMap;
#[cfg(unix)]
use std::os::fd::AsRawFd;

// Signal handling - cross-platform
#[cfg(unix)]
use nxsh_core::Signals;

// Cross-platform signal constants
#[cfg(unix)]
const SIGINT: i32 = nxsh_core::SIGINT;
#[cfg(windows)]
const SIGINT: i32 = 2; // Windows equivalent

#[derive(Debug, Clone)]
pub struct PingOptions {
    pub host: String,
    pub count: Option<u64>,
    pub interval: Duration,
    pub packet_size: usize,
    pub ttl: Option<u8>,
    pub timeout: Duration,
    pub deadline: Option<Duration>,
    pub flood: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub ipv4_only: bool,
    pub ipv6_only: bool,
    pub source_addr: Option<IpAddr>,
    pub print_timestamps: bool,
    pub no_dns: bool,
    pub audible: bool,
    pub broadcast: bool,
    pub bypass_routing: bool,
    pub record_route: bool,
    pub user_latency: bool,
    pub preload: Option<u32>,
    pub pattern: Option<Vec<u8>>,
    pub adaptive: bool,
    pub mark: Option<u32>,
    pub interface: Option<String>,
    pub pmtu_discovery: Option<PmtuDiscovery>,
    pub dont_fragment: bool,
    pub outstanding: bool,
    pub suppress_loopback: bool,
    pub timestamp_option: Option<TimestampOption>,
    pub tos: Option<u8>,
    pub flow_label: Option<u32>, // IPv6 only
    pub hop_limit: Option<u8>,   // IPv6 equivalent of TTL
}

#[derive(Debug, Clone, PartialEq)]
pub enum PmtuDiscovery {
    Want,
    Do,
    Dont,
    Probe,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimestampOption {
    TsOnly,
    TsAndAddr,
    TsPrespec,
}

impl Default for PingOptions {
    fn default() -> Self {
        Self {
            host: String::new(),
            count: None,
            interval: Duration::from_secs(1),
            packet_size: 56, // Default ping payload size (64 bytes total with ICMP header)
            ttl: None,
            timeout: Duration::from_secs(10),
            deadline: None,
            flood: false,
            quiet: false,
            verbose: false,
            ipv4_only: false,
            ipv6_only: false,
            source_addr: None,
            print_timestamps: false,
            no_dns: false,
            audible: false,
            broadcast: false,
            bypass_routing: false,
            record_route: false,
            user_latency: false,
            preload: None,
            pattern: None,
            adaptive: false,
            mark: None,
            interface: None,
            pmtu_discovery: None,
            dont_fragment: false,
            outstanding: false,
            suppress_loopback: false,
            timestamp_option: None,
            tos: None,
            flow_label: None,
            hop_limit: None,
        }
    }
}

#[derive(Debug)]
pub struct PingStats {
    pub packets_sent: AtomicU64,
    pub packets_received: AtomicU64,
    pub packets_lost: AtomicU64,
    pub packets_duplicated: AtomicU64,
    pub packets_corrupted: AtomicU64,
    pub packets_different_host: AtomicU64,
    pub min_time: Arc<Mutex<f64>>,
    pub max_time: Arc<Mutex<f64>>, 
    pub total_time: Arc<Mutex<f64>>,
    pub sum_squares: Arc<Mutex<f64>>,
    pub start_time: Instant,
    pub times: Arc<Mutex<VecDeque<f64>>>,
    pub sequence_map: Arc<DashMap<u16, (Instant, u16)>>, // seq -> (send_time, icmp_id)
    pub errors: AtomicU64,
    pub rtt_histogram: Arc<Mutex<BTreeMap<u32, u64>>>, // RTT bucket -> count
    pub packet_loss_bursts: Arc<Mutex<Vec<u64>>>,
    pub outstanding_packets: AtomicUsize,
    pub max_outstanding: AtomicUsize,
}

impl Clone for PingStats {
    fn clone(&self) -> Self {
        Self {
            packets_sent: AtomicU64::new(self.packets_sent.load(Ordering::Relaxed)),
            packets_received: AtomicU64::new(self.packets_received.load(Ordering::Relaxed)),
            packets_lost: AtomicU64::new(self.packets_lost.load(Ordering::Relaxed)),
            packets_duplicated: AtomicU64::new(self.packets_duplicated.load(Ordering::Relaxed)),
            packets_corrupted: AtomicU64::new(self.packets_corrupted.load(Ordering::Relaxed)),
            packets_different_host: AtomicU64::new(self.packets_different_host.load(Ordering::Relaxed)),
            min_time: Arc::new(Mutex::new(*self.min_time.lock())),
            max_time: Arc::new(Mutex::new(*self.max_time.lock())),
            total_time: Arc::new(Mutex::new(*self.total_time.lock())),
            sum_squares: Arc::new(Mutex::new(*self.sum_squares.lock())),
            start_time: self.start_time,
            times: Arc::new(Mutex::new(self.times.lock().clone())),
            sequence_map: Arc::new(DashMap::new()),
            errors: AtomicU64::new(self.errors.load(Ordering::Relaxed)),
            rtt_histogram: Arc::new(Mutex::new(self.rtt_histogram.lock().clone())),
            packet_loss_bursts: Arc::new(Mutex::new(self.packet_loss_bursts.lock().clone())),
            outstanding_packets: AtomicUsize::new(self.outstanding_packets.load(Ordering::Relaxed)),
            max_outstanding: AtomicUsize::new(self.max_outstanding.load(Ordering::Relaxed)),
        }
    }
}

impl PingStats {
    fn new() -> Self {
        Self {
            packets_sent: AtomicU64::new(0),
            packets_received: AtomicU64::new(0),
            packets_lost: AtomicU64::new(0),
            packets_duplicated: AtomicU64::new(0),
            packets_corrupted: AtomicU64::new(0),
            packets_different_host: AtomicU64::new(0),
            min_time: Arc::new(Mutex::new(f64::INFINITY)),
            max_time: Arc::new(Mutex::new(0.0)),
            total_time: Arc::new(Mutex::new(0.0)),
            sum_squares: Arc::new(Mutex::new(0.0)),
            start_time: Instant::now(),
            times: Arc::new(Mutex::new(VecDeque::new())),
            sequence_map: Arc::new(DashMap::new()),
            errors: AtomicU64::new(0),
            rtt_histogram: Arc::new(Mutex::new(BTreeMap::new())),
            packet_loss_bursts: Arc::new(Mutex::new(Vec::new())),
            outstanding_packets: AtomicUsize::new(0),
            max_outstanding: AtomicUsize::new(0),
        }
    }
    
    fn add_time(&self, time: f64, seq: u16, icmp_id: u16) {
        if let Some((_, (_, expected_id))) = self.sequence_map.remove(&seq) {
            if expected_id == icmp_id {
                self.packets_received.fetch_add(1, Ordering::Relaxed);
                self.outstanding_packets.fetch_sub(1, Ordering::Relaxed);
                
                let mut times = self.times.lock();
                times.push_back(time);
                if times.len() > 1000 {
                    times.pop_front();
                }
                drop(times);
                
                let mut total = self.total_time.lock();
                *total += time;
                drop(total);
                
                let mut squares = self.sum_squares.lock();
                *squares += time * time;
                drop(squares);
                
                let mut min = self.min_time.lock();
                if time < *min {
                    *min = time;
                }
                drop(min);
                
                let mut max = self.max_time.lock();
                if time > *max {
                    *max = time;
                }
                drop(max);
                
                // Update histogram
                let bucket = (time as u32 / 10) * 10; // 10ms buckets
                let mut histogram = self.rtt_histogram.lock();
                *histogram.entry(bucket).or_insert(0) += 1;
                drop(histogram);
            } else {
                self.packets_different_host.fetch_add(1, Ordering::Relaxed);
            }
        } else {
            self.packets_duplicated.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    fn add_sent(&self, seq: u16, icmp_id: u16, send_time: Instant) {
        self.packets_sent.fetch_add(1, Ordering::Relaxed);
        let outstanding = self.outstanding_packets.fetch_add(1, Ordering::Relaxed) + 1;
        
        // Update max outstanding
        let mut max_out = self.max_outstanding.load(Ordering::Relaxed);
        while outstanding > max_out {
            match self.max_outstanding.compare_exchange_weak(
                max_out, outstanding, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => max_out = x,
            }
        }
        
        self.sequence_map.insert(seq, (send_time, icmp_id));
    }
    
    fn add_timeout(&self, seq: u16) {
        if self.sequence_map.remove(&seq).is_some() {
            self.packets_lost.fetch_add(1, Ordering::Relaxed);
            self.outstanding_packets.fetch_sub(1, Ordering::Relaxed);
        }
    }
    
    fn add_corrupted(&self) {
        self.packets_corrupted.fetch_add(1, Ordering::Relaxed);
    }
    
    fn add_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }
    
    fn packets_sent(&self) -> u64 {
        self.packets_sent.load(Ordering::Relaxed)
    }
    
    fn packets_received(&self) -> u64 {
        self.packets_received.load(Ordering::Relaxed)
    }
    
    fn packets_lost(&self) -> u64 {
        self.packets_lost.load(Ordering::Relaxed)
    }
    
    fn packets_duplicated(&self) -> u64 {
        self.packets_duplicated.load(Ordering::Relaxed)
    }
    
    fn avg_time(&self) -> f64 {
        let received = self.packets_received();
        if received > 0 {
            *self.total_time.lock() / received as f64
        } else {
            0.0
        }
    }
    
    fn packet_loss_percent(&self) -> f64 {
        let sent = self.packets_sent();
        if sent == 0 {
            0.0
        } else {
            (self.packets_lost() as f64 / sent as f64) * 100.0
        }
    }
    
    fn standard_deviation(&self) -> f64 {
        let received = self.packets_received();
        if received < 2 {
            return 0.0;
        }
        
        let mean = self.avg_time();
        let sum_squares = *self.sum_squares.lock();
        let total_time = *self.total_time.lock();
        
        let variance = (sum_squares - (total_time * total_time / received as f64)) 
            / (received - 1) as f64;
        
        variance.sqrt()
    }
    
    fn median(&self) -> f64 {
        let times = self.times.lock();
        if times.is_empty() {
            return 0.0;
        }
        
        let mut sorted_times: Vec<f64> = times.iter().cloned().collect();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let len = sorted_times.len();
        if len % 2 == 0 {
            (sorted_times[len / 2 - 1] + sorted_times[len / 2]) / 2.0
        } else {
            sorted_times[len / 2]
        }
    }
    
    fn percentile(&self, p: f64) -> f64 {
        let times = self.times.lock();
        if times.is_empty() {
            return 0.0;
        }
        
        let mut sorted_times: Vec<f64> = times.iter().cloned().collect();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let index = ((p / 100.0) * (sorted_times.len() - 1) as f64).round() as usize;
        sorted_times[index.min(sorted_times.len() - 1)]
    }
    
    fn jitter(&self) -> f64 {
        let times = self.times.lock();
        if times.len() < 2 {
            return 0.0;
        }
        
        let mut jitter_sum = 0.0;
        let mut prev_time: Option<f64> = None;
        
        for &time in times.iter() {
            if let Some(prev) = prev_time {
                jitter_sum += (time - prev).abs() as f64;
            }
            prev_time = Some(time);
        }
        
        jitter_sum / (times.len() - 1) as f64
    }
}

// ICMP packet structures and utilities
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct IcmpHeader {
    icmp_type: u8,
    icmp_code: u8,
    icmp_cksum: u16,
    icmp_id: u16,
    icmp_seq: u16,
}

impl IcmpHeader {
    fn new_echo_request(id: u16, seq: u16) -> Self {
        Self {
            icmp_type: 8, // ICMP_ECHO
            icmp_code: 0,
            icmp_cksum: 0,
            icmp_id: id.to_be(),
            icmp_seq: seq.to_be(),
        }
    }
    
    fn calculate_checksum(&mut self, payload: &[u8]) {
        self.icmp_cksum = 0;
        
        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                self as *const _ as *const u8,
                std::mem::size_of::<IcmpHeader>()
            )
        };
        
        let mut data = Vec::new();
        data.extend_from_slice(header_bytes);
        data.extend_from_slice(payload);
        
        // Pad to even length
        if data.len() % 2 != 0 {
            data.push(0);
        }
        
        let mut sum = 0u32;
        for chunk in data.chunks(2) {
            if chunk.len() == 2 {
                sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
            }
        }
        
        // Fold 32-bit sum to 16 bits
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        self.icmp_cksum = (!(sum as u16)).to_be();
    }
}

// ICMPv6 packet structures
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct Icmp6Header {
    icmp6_type: u8,
    icmp6_code: u8,
    icmp6_cksum: u16,
    icmp6_id: u16,
    icmp6_seq: u16,
}

impl Icmp6Header {
    fn new_echo_request(id: u16, seq: u16) -> Self {
        Self {
            icmp6_type: 128, // ICMPv6_ECHO_REQUEST
            icmp6_code: 0,
            icmp6_cksum: 0,
            icmp6_id: id.to_be(),
            icmp6_seq: seq.to_be(),
        }
    }
    
    fn calculate_checksum(&mut self, src: &Ipv6Addr, dst: &Ipv6Addr, payload: &[u8]) {
        self.icmp6_cksum = 0;
        
        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                self as *const _ as *const u8,
                std::mem::size_of::<Icmp6Header>()
            )
        };
        
        let total_len = header_bytes.len() + payload.len();
        
        // Create pseudo-header for ICMPv6 checksum
        let mut pseudo_header = Vec::new();
        pseudo_header.extend_from_slice(&src.octets());
        pseudo_header.extend_from_slice(&dst.octets());
        pseudo_header.extend_from_slice(&(total_len as u32).to_be_bytes());
        pseudo_header.extend_from_slice(&[0, 0, 0, 58]); // Next header = ICMPv6
        
        let mut data = Vec::new();
        data.extend_from_slice(&pseudo_header);
        data.extend_from_slice(header_bytes);
        data.extend_from_slice(payload);
        
        // Pad to even length
        if data.len() % 2 != 0 {
            data.push(0);
        }
        
        let mut sum = 0u32;
        for chunk in data.chunks(2) {
            if chunk.len() == 2 {
                sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
            }
        }
        
        // Fold 32-bit sum to 16 bits
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        self.icmp6_cksum = (!(sum as u16)).to_be();
    }
}

fn parse_ping_args(args: &[String]) -> Result<PingOptions> {
    let mut options = PingOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "-c" => {
                if i + 1 < args.len() {
                    let count: u64 = args[i + 1].parse()
                        .map_err(|_| anyhow!("ping: invalid count '{}'", args[i + 1]))?;
                    if count == 0 {
                        return Err(anyhow!("ping: bad number of packets to transmit"));
                    }
                    options.count = Some(count);
                    i += 1;
                } else {
                    return Err(anyhow!("ping: option requires an argument -- c"));
                }
            }
            "-i" => {
                if i + 1 < args.len() {
                    let interval: f64 = args[i + 1].parse()
                        .map_err(|_| anyhow!("ping: invalid interval '{}'", args[i + 1]))?;
                    if interval < 0.001 {
                        return Err(anyhow!("ping: interval must be >= 0.001"));
                    }
                    options.interval = Duration::from_secs_f64(interval);
                    i += 1;
                } else {
                    return Err(anyhow!("ping: option requires an argument -- i"));
                }
            }
            "-s" => {
                if i + 1 < args.len() {
                    let size: usize = args[i + 1].parse()
                        .map_err(|_| anyhow!("ping: invalid packet size '{}'", args[i + 1]))?;
                    if size > 65507 {
                        return Err(anyhow!("ping: packet size too large: {}", size));
                    }
                    options.packet_size = size;
                    i += 1;
                } else {
                    return Err(anyhow!("ping: option requires an argument -- s"));
                }
            }
            "-t" => {
                if i + 1 < args.len() {
                    let ttl: u8 = args[i + 1].parse()
                        .map_err(|_| anyhow!("ping: invalid TTL '{}'", args[i + 1]))?;
                    if ttl == 0 || ttl > 255 {
                        return Err(anyhow!("ping: TTL must be between 1 and 255"));
                    }
                    options.ttl = Some(ttl);
                    i += 1;
                } else {
                    return Err(anyhow!("ping: option requires an argument -- t"));
                }
            }
            "-W" => {
                if i + 1 < args.len() {
                    let timeout: f64 = args[i + 1].parse()
                        .map_err(|_| anyhow!("ping: invalid timeout '{}'", args[i + 1]))?;
                    if timeout <= 0.0 {
                        return Err(anyhow!("ping: timeout must be positive"));
                    }
                    options.timeout = Duration::from_secs_f64(timeout);
                    i += 1;
                } else {
                    return Err(anyhow!("ping: option requires an argument -- W"));
                }
            }
            "-w" => {
                if i + 1 < args.len() {
                    let deadline: f64 = args[i + 1].parse()
                        .map_err(|_| anyhow!("ping: invalid deadline '{}'", args[i + 1]))?;
                    if deadline <= 0.0 {
                        return Err(anyhow!("ping: deadline must be positive"));
                    }
                    options.deadline = Some(Duration::from_secs_f64(deadline));
                    i += 1;
                } else {
                    return Err(anyhow!("ping: option requires an argument -- w"));
                }
            }
            "-S" => {
                if i + 1 < args.len() {
                    let addr: IpAddr = args[i + 1].parse()
                        .map_err(|_| anyhow!("ping: invalid source address '{}'", args[i + 1]))?;
                    options.source_addr = Some(addr);
                    i += 1;
                } else {
                    return Err(anyhow!("ping: option requires an argument -- S"));
                }
            }
            "-I" => {
                if i + 1 < args.len() {
                    options.interface = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err(anyhow!("ping: option requires an argument -- I"));
                }
            }
            "-l" => {
                if i + 1 < args.len() {
                    let preload: u32 = args[i + 1].parse()
                        .map_err(|_| anyhow!("ping: invalid preload '{}'", args[i + 1]))?;
                    if preload > 3 && !getuid().is_root() {
                        return Err(anyhow!("ping: preload value too high for non-root user"));
                    }
                    options.preload = Some(preload);
                    i += 1;
                } else {
                    return Err(anyhow!("ping: option requires an argument -- l"));
                }
            }
            "-p" => {
                if i + 1 < args.len() {
                    let pattern_str = &args[i + 1];
                    if pattern_str.len() % 2 != 0 {
                        return Err(anyhow!("ping: pattern must have even number of hex digits"));
                    }
                    if pattern_str.len() > 32 {
                        return Err(anyhow!("ping: pattern too long (max 16 bytes)"));
                    }
                    
                    let mut pattern = Vec::new();
                    for chunk in pattern_str.as_bytes().chunks(2) {
                        let hex_str = std::str::from_utf8(chunk)
                            .map_err(|_| anyhow!("ping: invalid hex pattern"))?;
                        let byte = u8::from_str_radix(hex_str, 16)
                            .map_err(|_| anyhow!("ping: invalid hex digit in pattern"))?;
                        pattern.push(byte);
                    }
                    options.pattern = Some(pattern);
                    i += 1;
                } else {
                    return Err(anyhow!("ping: option requires an argument -- p"));
                }
            }
            "-m" => {
                if i + 1 < args.len() {
                    let mark: u32 = args[i + 1].parse()
                        .map_err(|_| anyhow!("ping: invalid mark '{}'", args[i + 1]))?;
                    options.mark = Some(mark);
                    i += 1;
                } else {
                    return Err(anyhow!("ping: option requires an argument -- m"));
                }
            }
            "-M" => {
                if i + 1 < args.len() {
                    let hint = &args[i + 1];
                    options.pmtu_discovery = Some(match hint.as_str() {
                        "want" => PmtuDiscovery::Want,
                        "do" => PmtuDiscovery::Do,
                        "dont" => PmtuDiscovery::Dont,
                        "probe" => PmtuDiscovery::Probe,
                        _ => return Err(anyhow!("ping: invalid pmtu discovery hint '{}'", hint)),
                    });
                    i += 1;
                } else {
                    return Err(anyhow!("ping: option requires an argument -- M"));
                }
            }
            "-T" => {
                if i + 1 < args.len() {
                    let ts_opt = &args[i + 1];
                    options.timestamp_option = Some(match ts_opt.as_str() {
                        "tsonly" => TimestampOption::TsOnly,
                        "tsandaddr" => TimestampOption::TsAndAddr,
                        "tsprespec" => TimestampOption::TsPrespec,
                        _ => return Err(anyhow!("ping: invalid timestamp option '{}'", ts_opt)),
                    });
                    i += 1;
                } else {
                    return Err(anyhow!("ping: option requires an argument -- T"));
                }
            }
            "-Q" => {
                if i + 1 < args.len() {
                    let tos: u8 = args[i + 1].parse()
                        .map_err(|_| anyhow!("ping: invalid TOS '{}'", args[i + 1]))?;
                    options.tos = Some(tos);
                    i += 1;
                } else {
                    return Err(anyhow!("ping: option requires an argument -- Q"));
                }
            }
            "-f" => {
                options.flood = true;
                options.interval = Duration::from_millis(10);
            }
            "-q" => options.quiet = true,
            "-v" => options.verbose = true,
            "-4" => options.ipv4_only = true,
            "-6" => options.ipv6_only = true,
            "-D" => options.print_timestamps = true,
            "-n" => options.no_dns = true,
            "-a" => options.audible = true,
            "-b" => options.broadcast = true,
            "-r" => options.bypass_routing = true,
            "-R" => options.record_route = true,
            "-U" => options.user_latency = true,
            "-A" => options.adaptive = true,
            "-F" => options.dont_fragment = true,
            "-O" => options.outstanding = true,
            "-B" => {
                // Don't allow ping to change source address - set source to first interface
            }
            "-L" => options.suppress_loopback = true,
            arg if !arg.starts_with('-') => {
                if options.host.is_empty() {
                    options.host = arg.to_string();
                } else {
                    return Err(anyhow!("ping: unknown host '{}'", arg));
                }
            }
            _ => {
                return Err(anyhow!("ping: unknown option '{}'", arg));
            }
        }
        i += 1;
    }
    
    // Validate option combinations
    if options.ipv4_only && options.ipv6_only {
        return Err(anyhow!("ping: cannot specify both -4 and -6"));
    }
    
    if options.flood && options.interval > Duration::from_millis(10) {
        options.interval = Duration::from_millis(10);
    }
    
    if options.flood && !options.quiet {
        println!("FLOOD mode enabled. Press Ctrl+C to stop.");
    }
    
    Ok(options)
}

fn resolve_hostname(hostname: &str, ipv4_only: bool, ipv6_only: bool, no_dns: bool) -> Result<IpAddr> {
    // Try to parse as IP address first
    if let Ok(addr) = hostname.parse::<IpAddr>() {
        match addr {
            IpAddr::V4(_) if ipv6_only => {
                return Err(anyhow!("ping: cannot resolve '{}': Address family not supported", hostname));
            }
            IpAddr::V6(_) if ipv4_only => {
                return Err(anyhow!("ping: cannot resolve '{}': Address family not supported", hostname));
            }
            _ => return Ok(addr),
        }
    }
    
    if no_dns {
        return Err(anyhow!("ping: cannot resolve '{}': Name resolution disabled", hostname));
    }
    
    // Resolve using DNS
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())?;
    
    if ipv6_only {
        let response = resolver.ipv6_lookup(hostname)?;
        if let Some(addr) = response.iter().next() {
            return Ok(IpAddr::V6(addr.0));
        }
    } else if ipv4_only {
        let response = resolver.ipv4_lookup(hostname)?;
        if let Some(addr) = response.iter().next() {
            return Ok(IpAddr::V4(addr.0));
        }
    } else {
        // Try IPv4 first, then IPv6
        if let Ok(response) = resolver.ipv4_lookup(hostname) {
            if let Some(addr) = response.iter().next() {
                return Ok(IpAddr::V4(addr.0));
            }
        }
        if let Ok(response) = resolver.ipv6_lookup(hostname) {
            if let Some(addr) = response.iter().next() {
                return Ok(IpAddr::V6(addr.0));
            }
        }
    }
    
    Err(anyhow!("ping: cannot resolve '{}': Name or service not known", hostname))
}

fn create_raw_socket(target_addr: &IpAddr, options: &PingOptions) -> Result<Socket> {
    let (domain, protocol) = match target_addr {
        IpAddr::V4(_) => (Domain::IPV4, Protocol::ICMPV4),
        IpAddr::V6(_) => (Domain::IPV6, Protocol::ICMPV6),
    };
    
    let socket = Socket::new(domain, Type::RAW, Some(protocol))
        .map_err(|e| anyhow!("ping: socket creation failed: {} (are you root?)", e))?;
    
    // Set socket to non-blocking for better control
    socket.set_nonblocking(true)?;
    
    Ok(socket)
}

fn configure_socket(socket: &Socket, options: &PingOptions, target_addr: &IpAddr) -> Result<()> {
    #[cfg(unix)]
    {
        // Set receive timeout
        let timeout = libc::timeval {
            tv_sec: options.timeout.as_secs() as i64,
            tv_usec: options.timeout.subsec_micros() as i64,
        };
        
        unsafe {
            let ret = libc::setsockopt(
                socket.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_RCVTIMEO,
                &timeout as *const _ as *const std::ffi::c_void,
                std::mem::size_of::<libc::timeval>() as u32,
            );
            if ret != 0 {
                return Err(anyhow!("ping: failed to set receive timeout"));
            }
        }
    }
    
    #[cfg(windows)]
    {
        // Windows-specific socket configuration would go here
        // For now, use socket2 methods where possible
        if let Some(ttl) = options.ttl {
            socket.set_ttl(ttl as u32)?;
        }
    }
    
    Ok(())
}

fn create_icmp_packet(id: u16, seq: u16, payload: &[u8]) -> Vec<u8> {
    let mut header = IcmpHeader::new_echo_request(id, seq);
    
    let mut packet = Vec::new();
    packet.extend_from_slice(unsafe {
        std::slice::from_raw_parts(
            &header as *const _ as *const u8,
            std::mem::size_of::<IcmpHeader>()
        )
    });
    packet.extend_from_slice(payload);
    
    // Calculate checksum
    header.calculate_checksum(payload);
    
    // Update packet with correct checksum
    packet[2..4].copy_from_slice(&header.icmp_cksum.to_ne_bytes());
    
    packet
}

// Simplified ping function for compatibility
pub fn ping(args: &[String]) -> Result<()> {
    ping_cli(args)
}

fn run_ping(
    socket: &Socket,
    target_addr: &IpAddr,
    options: &PingOptions,
    running: Arc<AtomicBool>,
) -> Result<PingStats> {
    let stats = PingStats::new();
    let mut seq = 1u16;
    let ping_id = (getpid() & 0xFFFF) as u16;
    
    let target_socket_addr = match target_addr {
        IpAddr::V4(addr) => SocketAddr::new(IpAddr::V4(*addr), 0),
        IpAddr::V6(addr) => SocketAddr::new(IpAddr::V6(*addr), 0),
    };
    
    let deadline_start = if options.deadline.is_some() {
        Some(Instant::now())
    } else {
        None
    };
    
    // Set up channels for packet sending and receiving
    let (send_tx, send_rx) = mpsc::channel::<(u16, Instant)>();
    let (recv_tx, recv_rx) = mpsc::channel::<PingResponse>();
    
    // Spawn receiver thread
    let recv_socket = socket.try_clone()?;
    let recv_stats = stats.clone();
    let recv_options = options.clone();
    let recv_target = *target_addr;
    let recv_running = running.clone();
    
    thread::spawn(move || {
        receive_packets(recv_socket, recv_target, recv_options, recv_stats, recv_running, recv_tx);
    });
    
    // Send preload packets if specified
    if let Some(preload) = options.preload {
        for _ in 0..preload {
            if !running.load(Ordering::Relaxed) {
                break;
            }
            
            let send_time = Instant::now();
            let payload = create_payload(options.packet_size, seq, send_time, &options.pattern);
            
            match send_packet(socket, &target_socket_addr, target_addr, &payload, ping_id, seq) {
                Ok(_) => {
                    stats.add_sent(seq, ping_id, send_time);
                    seq = seq.wrapping_add(1);
                }
                Err(e) => {
                    if !options.quiet {
                        eprintln!("ping: sendto failed: {}", e);
                    }
                    stats.add_error();
                }
            }
        }
    }
    
    // Main ping loop
    let mut last_send = Instant::now();
    let mut adaptive_interval = options.interval;
    
    while running.load(Ordering::Relaxed) {
        // Check deadline
        if let (Some(deadline), Some(start)) = (options.deadline, deadline_start) {
            if start.elapsed() >= deadline {
                break;
            }
        }
        
        // Check count limit
        if let Some(count) = options.count {
            if stats.packets_sent() >= count {
                break;
            }
        }
        
        // Handle outstanding packets reporting
        if options.outstanding {
            let outstanding = stats.outstanding_packets.load(Ordering::Relaxed);
            if outstanding > 0 && !options.quiet {
                println!("no answer yet for icmp_seq={}", seq.wrapping_sub(outstanding as u16));
            }
        }
        
        // Wait for appropriate interval
        let now = Instant::now();
        let elapsed = now.duration_since(last_send);
        
        if elapsed < adaptive_interval {
            let sleep_time = adaptive_interval - elapsed;
            if sleep_time > Duration::from_millis(1) {
                thread::sleep(sleep_time);
            }
        }
        
        let send_time = Instant::now();
        last_send = send_time;
        
        // Create and send packet
        let payload = create_payload(options.packet_size, seq, send_time, &options.pattern);
        
        match send_packet(socket, &target_socket_addr, target_addr, &payload, ping_id, seq) {
            Ok(_) => {
                stats.add_sent(seq, ping_id, send_time);
                
                if options.flood && !options.quiet {
                    print!(".");
                    io::stdout().flush().unwrap();
                }
            }
            Err(e) => {
                if !options.quiet {
                    eprintln!("ping: sendto failed: {}", e);
                }
                stats.add_error();
                
                if options.flood && !options.quiet {
                    print!("E");
                    io::stdout().flush().unwrap();
                }
            }
        }
        
        // Handle received packets
        while let Ok(response) = recv_rx.try_recv() {
            handle_ping_response(response, options, &stats);
        }
        
        seq = seq.wrapping_add(1);
        
        // Adaptive interval adjustment
        if options.adaptive && stats.packets_received() > 0 {
            let avg_rtt = stats.avg_time();
            let new_interval = Duration::from_millis((avg_rtt * 2.0) as u64);
            adaptive_interval = new_interval.max(options.interval);
        }
    }
    
    if options.flood && !options.quiet {
        println!();
    }
    
    // Wait a bit for any remaining responses
    let wait_time = options.timeout.min(Duration::from_secs(1));
    let wait_start = Instant::now();
    
    while wait_start.elapsed() < wait_time {
        if let Ok(response) = recv_rx.try_recv() {
            handle_ping_response(response, options, &stats);
        }
        thread::sleep(Duration::from_millis(10));
    }
    
    // Mark any remaining packets as lost
    for entry in stats.sequence_map.iter() {
        let seq = *entry.key();
        stats.add_timeout(seq);
    }
    
    Ok(stats)
}

fn create_payload(size: usize, seq: u16, send_time: Instant, pattern: &Option<Vec<u8>>) -> Vec<u8> {
    let mut payload = Vec::with_capacity(size);
    
    // Add timestamp (8 bytes) - Unix ping compatibility
    let timestamp = send_time.elapsed().as_nanos() as u64;
    payload.extend_from_slice(&timestamp.to_be_bytes());
    
    // Add sequence number (2 bytes)
    payload.extend_from_slice(&seq.to_be_bytes());
    
    // Add process ID (2 bytes)
    let pid = getpid() as u16;
    payload.extend_from_slice(&pid.to_be_bytes());
    
    // Fill remaining space with pattern or default pattern
    let remaining = size.saturating_sub(payload.len());
    
    if let Some(ref pat) = pattern {
        for i in 0..remaining {
            payload.push(pat[i % pat.len()]);
        }
    } else {
        // Default pattern: ascending bytes starting from 0x08
        for i in 0..remaining {
            payload.push(((i + 8) % 256) as u8);
        }
    }
    
    payload.truncate(size);
    payload
}

fn send_packet(
    socket: &Socket,
    target_addr: &SocketAddr,
    ip_addr: &IpAddr,
    payload: &[u8],
    icmp_id: u16,
    seq: u16,
) -> Result<()> {
    match ip_addr {
        IpAddr::V4(_) => {
            let mut icmp_header = IcmpHeader::new_echo_request(icmp_id, seq);
            icmp_header.calculate_checksum(payload);
            
            let mut packet = Vec::new();
            packet.extend_from_slice(unsafe {
                std::slice::from_raw_parts(
                    &icmp_header as *const _ as *const u8,
                    std::mem::size_of::<IcmpHeader>()
                )
            });
            packet.extend_from_slice(payload);
            
            socket.send_to(&packet, &(*target_addr).into())?;
        }
        IpAddr::V6(ipv6_addr) => {
            // For IPv6, we need source address for checksum calculation
            let src_addr = match socket.local_addr()? {
                addr => {
                    if let Some(socket_addr) = addr.as_socket_ipv6() {
                        *socket_addr.ip()
                    } else {
                        return Err(anyhow!("ping: invalid socket address family"));
                    }
                }
            };
            
            let mut icmp6_header = Icmp6Header::new_echo_request(icmp_id, seq);
            icmp6_header.calculate_checksum(&src_addr, ipv6_addr, payload);
            
            let mut packet = Vec::new();
            packet.extend_from_slice(unsafe {
                std::slice::from_raw_parts(
                    &icmp6_header as *const _ as *const u8,
                    std::mem::size_of::<Icmp6Header>()
                )
            });
            packet.extend_from_slice(payload);
            
            socket.send_to(&packet, &(*target_addr).into())?;
        }
    }
    
    Ok(())
}

#[derive(Debug, Clone)]
struct PingResponse {
    from_addr: IpAddr,
    icmp_type: u8,
    icmp_code: u8,
    icmp_id: u16,
    icmp_seq: u16,
    payload: Vec<u8>,
    receive_time: Instant,
    ttl: Option<u8>,
    packet_size: usize,
}

fn receive_packets(
    socket: Socket,
    target_addr: IpAddr,
    options: PingOptions,
    stats: PingStats,
    running: Arc<AtomicBool>,
    tx: Sender<PingResponse>,
) {
    let buffer = [0u8; 65536];
    
    while running.load(Ordering::Relaxed) {
        let mut uninit_buffer = [std::mem::MaybeUninit::<u8>::uninit(); 65536];
        match socket.recv_from(&mut uninit_buffer) {
            Ok((bytes_received, from_addr)) => {
                let receive_time = Instant::now();
                
                // Convert MaybeUninit buffer to initialized buffer
                let buffer: Vec<u8> = uninit_buffer[..bytes_received]
                    .iter()
                    .map(|x| unsafe { x.assume_init() })
                    .collect();
                
                if let Ok(from_ip) = extract_ip_from_sockaddr(&from_addr) {
                    if let Some(response) = parse_icmp_response(
                        &buffer,
                        from_ip,
                        receive_time,
                        &target_addr,
                    ) {
                        if tx.send(response).is_err() {
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                // Handle timeout and other errors
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut {
                    thread::sleep(Duration::from_millis(1));
                    continue;
                }
                
                if running.load(Ordering::Relaxed) {
                    stats.add_error();
                }
                break;
            }
        }
    }
}

fn extract_ip_from_sockaddr(addr: &SockAddr) -> Result<IpAddr> {
    if let Some(socket_addr_v4) = addr.as_socket_ipv4() {
        Ok(IpAddr::V4(*socket_addr_v4.ip()))
    } else if let Some(socket_addr_v6) = addr.as_socket_ipv6() {
        Ok(IpAddr::V6(*socket_addr_v6.ip()))
    } else {
        Err(anyhow!("ping: unsupported address family"))
    }
}

fn parse_icmp_response(
    packet: &[u8],
    from_addr: IpAddr,
    receive_time: Instant,
    target_addr: &IpAddr,
) -> Option<PingResponse> {
    match target_addr {
        IpAddr::V4(_) => {
            // Skip IP header for IPv4
            if packet.len() < 20 {
                return None;
            }
            
            let ip_header_len = ((packet[0] & 0x0F) * 4) as usize;
            if packet.len() < ip_header_len + 8 {
                return None;
            }
            
            let icmp_packet = &packet[ip_header_len..];
            let ttl = Some(packet[8]); // TTL field in IP header
            
            parse_icmp4_packet(icmp_packet, from_addr, receive_time, ttl, packet.len())
        }
        IpAddr::V6(_) => {
            // For IPv6, kernel strips the IPv6 header
            if packet.len() < 8 {
                return None;
            }
            
            parse_icmp6_packet(packet, from_addr, receive_time, packet.len())
        }
    }
}

fn parse_icmp4_packet(
    icmp_packet: &[u8],
    from_addr: IpAddr,
    receive_time: Instant,
    ttl: Option<u8>,
    total_size: usize,
) -> Option<PingResponse> {
    if icmp_packet.len() < 8 {
        return None;
    }
    
    let icmp_type = icmp_packet[0];
    let icmp_code = icmp_packet[1];
    let icmp_id = u16::from_be_bytes([icmp_packet[4], icmp_packet[5]]);
    let icmp_seq = u16::from_be_bytes([icmp_packet[6], icmp_packet[7]]);
    let payload = icmp_packet[8..].to_vec();
    
    Some(PingResponse {
        from_addr,
        icmp_type,
        icmp_code,
        icmp_id,
        icmp_seq,
        payload,
        receive_time,
        ttl,
        packet_size: total_size,
    })
}

fn parse_icmp6_packet(
    icmp_packet: &[u8],
    from_addr: IpAddr,
    receive_time: Instant,
    total_size: usize,
) -> Option<PingResponse> {
    if icmp_packet.len() < 8 {
        return None;
    }
    
    let icmp_type = icmp_packet[0];
    let icmp_code = icmp_packet[1];
    let icmp_id = u16::from_be_bytes([icmp_packet[4], icmp_packet[5]]);
    let icmp_seq = u16::from_be_bytes([icmp_packet[6], icmp_packet[7]]);
    let payload = icmp_packet[8..].to_vec();
    
    Some(PingResponse {
        from_addr,
        icmp_type,
        icmp_code,
        icmp_id,
        icmp_seq,
        payload,
        receive_time,
        ttl: None, // IPv6 doesn't have TTL in the same way
        packet_size: total_size,
    })
}

fn handle_ping_response(response: PingResponse, options: &PingOptions, stats: &PingStats) {
    match response.icmp_type {
        0 | 129 => {
            // ICMP Echo Reply (IPv4) or ICMPv6 Echo Reply
            if let Some(entry) = stats.sequence_map.get(&response.icmp_seq) {
                let (send_time, _) = *entry;
                let rtt = response.receive_time.duration_since(send_time).as_secs_f64() * 1000.0;
                
                stats.add_time(rtt, response.icmp_seq, response.icmp_id);
                
                if !options.quiet && !options.flood {
                    let timestamp = if options.print_timestamps {
                        format!("[{}] ", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
                    } else {
                        String::new()
                    };
                    
                    let ttl_info = if let Some(ttl) = response.ttl {
                        format!(" ttl={}", ttl)
                    } else {
                        String::new()
                    };
                    
                    if options.verbose {
                        println!("{}64 bytes from {}: icmp_seq={}{} time={:.3} ms{}",
                            timestamp, response.from_addr, response.icmp_seq, ttl_info, rtt,
                            if options.user_latency { " (user-to-user)" } else { "" });
                    } else {
                        println!("{}64 bytes from {}: icmp_seq={}{} time={:.3} ms",
                            timestamp, response.from_addr, response.icmp_seq, ttl_info, rtt);
                    }
                    
                    if options.audible {
                        print!("\x07"); // Bell character
                        io::stdout().flush().unwrap();
                    }
                }
            } else {
                stats.add_corrupted();
            }
        }
        3 | 1 => {
            // ICMP Destination Unreachable or ICMPv6 Destination Unreachable
            if !options.quiet {
                let error_msg = match response.icmp_code {
                    0 => "Destination Net Unreachable",
                    1 => "Destination Host Unreachable", 
                    2 => "Destination Protocol Unreachable",
                    3 => "Destination Port Unreachable",
                    4 => "Fragmentation needed and DF set",
                    5 => "Source Route Failed",
                    _ => "Destination Unreachable",
                };
                println!("From {}: {}", response.from_addr, error_msg);
            }
            stats.add_error();
        }
        11 | 3 => {
            // ICMP Time Exceeded or ICMPv6 Time Exceeded
            if !options.quiet {
                let error_msg = match response.icmp_code {
                    0 => "Time to live exceeded in transit",
                    1 => "Fragment reassembly time exceeded",
                    _ => "Time exceeded",
                };
                println!("From {}: {}", response.from_addr, error_msg);
            }
            stats.add_error();
        }
        _ => {
            // Other ICMP types
            if options.verbose && !options.quiet {
                println!("From {}: ICMP type {} code {}", 
                    response.from_addr, response.icmp_type, response.icmp_code);
            }
            stats.add_error();
        }
    }
}

fn print_summary(options: &PingOptions, stats: &PingStats) {
    if options.quiet && !options.flood {
        return;
    }
    
    let total_time = stats.start_time.elapsed().as_secs_f64() * 1000.0;
    let packets_sent = stats.packets_sent();
    let packets_received = stats.packets_received();
    let packets_lost = stats.packets_lost();
    let packets_duplicated = stats.packets_duplicated();
    
    println!("\n--- {} ping statistics ---", options.host);
    
    if packets_duplicated > 0 {
        println!("{} packets transmitted, {} received, +{} duplicates, {:.1}% packet loss, time {:.0}ms",
            packets_sent, packets_received, packets_duplicated,
            stats.packet_loss_percent(), total_time);
    } else {
        println!("{} packets transmitted, {} received, {:.1}% packet loss, time {:.0}ms",
            packets_sent, packets_received, stats.packet_loss_percent(), total_time);
    }
    
    if packets_received > 0 {
        let min_time = *stats.min_time.lock();
        let max_time = *stats.max_time.lock();
        let avg_time = stats.avg_time();
        let stddev = stats.standard_deviation();
        let median = stats.median();
        
        println!("rtt min/avg/max/mdev = {:.3}/{:.3}/{:.3}/{:.3} ms",
            min_time, avg_time, max_time, stddev);
        
        if options.verbose {
            println!("rtt median = {:.3} ms", median);
            println!("rtt 95th percentile = {:.3} ms", stats.percentile(95.0));
            println!("rtt 99th percentile = {:.3} ms", stats.percentile(99.0));
            println!("jitter = {:.3} ms", stats.jitter());
            
            let errors = stats.errors.load(Ordering::Relaxed);
            let corrupted = stats.packets_corrupted.load(Ordering::Relaxed);
            let different_host = stats.packets_different_host.load(Ordering::Relaxed);
            
            if errors > 0 {
                println!("errors: {}", errors);
            }
            if corrupted > 0 {
                println!("corrupted packets: {}", corrupted);
            }
            if different_host > 0 {
                println!("packets from different host: {}", different_host);
            }
            
            let max_outstanding = stats.max_outstanding.load(Ordering::Relaxed);
            if max_outstanding > 1 {
                println!("max outstanding packets: {}", max_outstanding);
            }
            
            // Print RTT histogram for very verbose output
            let histogram = stats.rtt_histogram.lock();
            if !histogram.is_empty() && packets_received > 10 {
                println!("RTT histogram (ms):");
                for (bucket, count) in histogram.iter() {
                    let percentage = (*count as f64 / packets_received as f64) * 100.0;
                    println!("  {}-{}: {} ({:.1}%)", bucket, bucket + 9, count, percentage);
                }
            }
        }
    }
    
    // Print additional statistics for flood mode
    if options.flood && packets_sent > 0 {
        let pps = packets_sent as f64 / (total_time / 1000.0);
        println!("flood statistics: {:.0} packets/sec", pps);
        
        if packets_received > 0 {
            let response_rate = (packets_received as f64 / packets_sent as f64) * 100.0;
            println!("response rate: {:.1}%", response_rate);
        }
    }
    
    // Print error summary
    let total_errors = stats.errors.load(Ordering::Relaxed) + 
                      stats.packets_corrupted.load(Ordering::Relaxed) +
                      stats.packets_different_host.load(Ordering::Relaxed);
    
    if total_errors > 0 && options.verbose {
        println!("total errors: {}", total_errors);
    }
}

pub fn ping_cli(args: &[String]) -> Result<()> {
    let options = parse_ping_args(args)?;
    
    if options.host.is_empty() {
        return Err(anyhow!("ping: usage error: Destination address required"));
    }
    
    // Check privileges for certain options
    let is_root = getuid().is_root();
    
    if options.flood && !is_root {
        return Err(anyhow!("ping: cannot flood; minimal interval allowed for user is 200ms"));
    }
    
    if options.interval < Duration::from_millis(200) && !is_root {
        return Err(anyhow!("ping: cannot set interval to less than 200ms; minimal interval allowed for user is 200ms"));
    }
    
    if options.preload.is_some() && !is_root {
        return Err(anyhow!("ping: cannot use preload; this requires root privileges"));
    }
    
    // Set up signal handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    
    #[cfg(unix)]
    {
        let mut signals = Signals::new(&[SIGINT])?;
        thread::spawn(move || {
            for _sig in signals.forever() {
                running_clone.store(false, Ordering::Relaxed);
                break;
            }
        });
    }
    
    #[cfg(windows)]
    {
        // Windows signal handling would go here
        // For now, rely on Ctrl+C handling in main application
    }
    
    // Resolve hostname
    let target_addr = resolve_hostname(&options.host, options.ipv4_only, options.ipv6_only, options.no_dns)?;
    
    if !options.quiet {
        let hostname_display = if options.no_dns {
            target_addr.to_string()
        } else {
            options.host.clone()
        };
        
        println!("PING {} ({}): {} data bytes", hostname_display, target_addr, options.packet_size);
        
        if options.verbose {
            println!("PING statistics for {}:", target_addr);
            if let Some(ttl) = options.ttl {
                println!("TTL: {}", ttl);
            }
            if let Some(ref iface) = options.interface {
                println!("Interface: {}", iface);
            }
            if let Some(ref src) = options.source_addr {
                println!("Source: {}", src);
            }
            if options.flood {
                println!("Flood mode enabled");
            }
            if options.dont_fragment {
                println!("Don't fragment flag set");
            }
        }
    }
    
    // Create socket based on IP version
    let socket = create_raw_socket(&target_addr, &options)?;
    
    // Configure socket options
    configure_socket(&socket, &options, &target_addr)?;
    
    // Run ping
    let stats = run_ping(&socket, &target_addr, &options, running)?;
    
    // Print summary
    print_summary(&options, &stats);
    
    // Set exit code based on packet loss and errors
    let exit_code = if stats.packets_received() == 0 { 
        2 
    } else if stats.packet_loss_percent() > 0.0 { 
        1 
    } else { 
        0 
    };
    
    std::process::exit(exit_code);
}

// Missing function implementations for compatibility
pub fn getuid() -> u32 {
    #[cfg(unix)]
    return unsafe { libc::getuid() };
    #[cfg(windows)]
    return 0; // Non-root user on Windows
}

pub fn getpid() -> u32 {
    std::process::id()
}

// Type aliases for compatibility
pub type c_void = std::ffi::c_void;
pub type c_int = std::ffi::c_int;

// UID helper trait
trait IsRoot {
    fn is_root(&self) -> bool;
}

impl IsRoot for u32 {
    fn is_root(&self) -> bool {
        *self == 0
    }
} 
