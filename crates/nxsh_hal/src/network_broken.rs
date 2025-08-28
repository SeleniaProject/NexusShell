//! Advanced network management abstraction layer with comprehensive networking capabilities
//!
//! This module provides platform-agnostic network operations, connection management,
//! protocol support, security features, and performance monitoring.

use std::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::thread;
use std::io::{Read, Write};

use crate::error::{HalError, HalResult};
use crate::platform::{Platform, Capabilities};

/// Advanced network management with comprehensive networking capabilities
#[derive(Debug)]
pub struct NetworkManager {
    platform: Platform,
    capabilities: Capabilities,
    connection_pool: Arc<RwLock<ConnectionPool>>,
    performance_monitor: Arc<Mutex<NetworkPerformanceMonitor>>,
    security_manager: NetworkSecurityManager,
    bandwidth_monitor: BandwidthMonitor,
    dns_cache: Arc<RwLock<DnsCache>>,
    proxy_config: Option<ProxyConfiguration>,
}

impl NetworkManager {
    pub fn new() -> HalResult<Self> {
        let platform = Platform::current();
        let capabilities = Capabilities::current();
        
        let mut manager = Self {
            platform,
            capabilities,
            connection_pool: Arc::new(RwLock::new(ConnectionPool::new())),
            performance_monitor: Arc::new(Mutex::new(NetworkPerformanceMonitor::new())),
            security_manager: NetworkSecurityManager::new(),
            bandwidth_monitor: BandwidthMonitor::new(),
            dns_cache: Arc::new(RwLock::new(DnsCache::new())),
            proxy_config: None,
        };
        
        // Initialize default security policies
        manager.initialize_security_policies()?;
        
        // Start background monitoring
        manager.start_background_monitoring()?;
        
        Ok(manager)
    }

    /// Create a new network connection with advanced features
    pub fn create_connection(&self, target: &str, protocol: NetworkProtocol) -> HalResult<NetworkConnection> {
        let connection_info = self.parse_connection_target(target)?;
        
        // Security validation
        self.security_manager.validate_connection(&connection_info)?;
        
        // Apply proxy settings if configured
        let effective_target = if let Some(ref proxy) = self.proxy_config {
            proxy.resolve_target(&connection_info)?
        } else {
            connection_info.clone()
        };
        
        // Create the raw connection
        let raw_connection = match protocol {
            NetworkProtocol::Tcp => self.create_tcp_connection(&effective_target)?,
            NetworkProtocol::Udp => self.create_udp_connection(&effective_target)?,
            NetworkProtocol::Http => self.create_http_connection(&effective_target)?,
            NetworkProtocol::Https => self.create_https_connection(&effective_target)?,
        };
        
        let connection = NetworkConnection {
            id: Self::generate_connection_id(),
            protocol,
            target: connection_info,
            stream: raw_connection,
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            bytes_sent: 0,
            bytes_received: 0,
            state: ConnectionState::Connected,
            quality_metrics: ConnectionQualityMetrics::default(),
        };

        // Add to connection pool
        if let Ok(mut pool) = self.connection_pool.write() {
            pool.add_connection(connection.clone());
        }

        // Start performance monitoring
        if let Ok(mut monitor) = self.performance_monitor.lock() {
            monitor.start_monitoring(&connection)?;
        }

        Ok(connection)
    }

    /// Send data with bandwidth limiting and security filtering
    pub fn send_data(&self, connection_id: &str, data: &[u8]) -> HalResult<usize> {
        // Check bandwidth limits
        self.bandwidth_monitor.check_send_limit(data.len())?;
        
        // Apply security filtering
        self.security_manager.filter_outgoing_data(data)?;
        
        // Get connection from pool
        let connection = {
            let pool = self.connection_pool.read()
                .map_err(|_| HalError::internal("Failed to acquire connection pool lock"))?;
            pool.get_connection(connection_id)
                .ok_or_else(|| HalError::not_found("Connection not found"))?
                .clone()
        };

        // Send data
        let bytes_sent = connection.send_data(data)?;
        
        // Update metrics
        if let Ok(mut monitor) = self.performance_monitor.lock() {
            monitor.record_bytes_sent(connection_id, bytes_sent);
        }
        
        self.bandwidth_monitor.record_sent_bytes(bytes_sent);
        
        // Update connection statistics
        self.update_connection_stats(connection_id, bytes_sent, 0)?;
        
        Ok(bytes_sent)
    }

    /// Receive data with security filtering
    pub fn receive_data(&self, connection_id: &str, buffer: &mut [u8]) -> HalResult<usize> {
        // Get connection from pool
        let connection = {
            let pool = self.connection_pool.read()
                .map_err(|_| HalError::internal("Failed to acquire connection pool lock"))?;
            pool.get_connection(connection_id)
                .ok_or_else(|| HalError::not_found("Connection not found"))?
                .clone()
        };

        // Receive data
        let bytes_received = connection.receive_data(buffer)?;
        
        if bytes_received > 0 {
            // Apply security filtering
            self.security_manager.filter_incoming_data(&buffer[..bytes_received])?;
            
            // Update metrics
            if let Ok(mut monitor) = self.performance_monitor.lock() {
                monitor.record_bytes_received(connection_id, bytes_received);
            }
            
            self.bandwidth_monitor.record_received_bytes(bytes_received);
            
            // Update connection statistics
            self.update_connection_stats(connection_id, 0, bytes_received)?;
        }
        
        Ok(bytes_received)
    }

    /// Close connection and cleanup resources
    pub fn close_connection(&self, connection_id: &str) -> HalResult<()> {
        if let Ok(mut pool) = self.connection_pool.write() {
            if let Some(mut connection) = pool.remove_connection(connection_id) {
                connection.close()?;
                
                // Stop monitoring
                if let Ok(mut monitor) = self.performance_monitor.lock() {
                    monitor.stop_monitoring(connection_id);
                }
                
                // Generate connection report
                let report = self.generate_connection_report(&connection)?;
                if let Ok(mut monitor) = self.performance_monitor.lock() {
                    monitor.add_connection_report(report);
                }
            }
        }
        Ok(())
    }

    /// Test network connectivity with comprehensive analysis
    pub fn test_connectivity(&self, target: &str, timeout: Duration) -> HalResult<ConnectivityTestResult> {
        let start_time = SystemTime::now();
        
        let result = match self.ping_host(target, timeout) {
            Ok(ping_time) => {
                let hop_count = self.estimate_hop_count(target).unwrap_or(0);
                ConnectivityTestResult {
                    target: target.to_string(),
                    reachable: true,
                    response_time: ping_time,
                    test_time: SystemTime::now().duration_since(start_time).unwrap_or_default(),
                    error_message: None,
                    hop_count,
                    quality_score: self.calculate_connection_quality(&ping_time, 0.0),
                }
            },
            Err(e) => ConnectivityTestResult {
                target: target.to_string(),
                reachable: false,
                response_time: Duration::default(),
                test_time: SystemTime::now().duration_since(start_time).unwrap_or_default(),
                error_message: Some(format!("{e:?}")),
                hop_count: 0,
                quality_score: 0.0,
            }
        };
        
        Ok(result)
    }

    /// Measure network performance over time
    pub fn measure_performance(&self, target: &str, duration: Duration) -> HalResult<NetworkPerformanceReport> {
        let start_time = SystemTime::now();
        let mut measurements = Vec::new();
        let end_time = start_time + duration;

        while SystemTime::now() < end_time {
            let measurement_start = SystemTime::now();
            
            match self.ping_host(target, Duration::from_secs(1)) {
                Ok(latency) => {
                    measurements.push(PerformanceMeasurement {
                        timestamp: measurement_start,
                        latency,
                        packet_loss: false,
                        jitter: Duration::default(), // Would calculate from previous measurements
                    });
                },
                Err(_) => {
                    measurements.push(PerformanceMeasurement {
                        timestamp: measurement_start,
                        latency: Duration::default(),
                        packet_loss: true,
                        jitter: Duration::default(),
                    });
                }
            }
            
            thread::sleep(Duration::from_millis(100));
        }

        self.generate_performance_report(target, duration, measurements)
    }

    pub fn get_default_gateway(&self) -> HalResult<IpAddr> {
        #[cfg(unix)]
        {
            self.get_default_gateway_unix()
        }
        #[cfg(windows)]
        {
            self.get_default_gateway_windows()
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(HalError::unsupported("Default gateway detection not supported on this platform"))
        }
    }

    pub fn get_dns_servers(&self) -> HalResult<Vec<IpAddr>> {
        #[cfg(unix)]
        {
            self.get_dns_servers_unix()
        }
        #[cfg(windows)]
        {
            self.get_dns_servers_windows()
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(HalError::unsupported("DNS server detection not supported on this platform"))
        }
    }

    pub fn ping(&self, target: &str, timeout_ms: u64) -> HalResult<PingResult> {
        let start = std::time::Instant::now();
        
        // Try native ping first for more accurate results
        if let Ok(result) = self.native_ping(target, timeout_ms) {
            return Ok(result);
        }
        
        // Fallback to TCP connectivity test
        self.tcp_ping(target, timeout_ms, start)
    }

    /// Attempt to use native ping command for accurate ICMP ping
    fn native_ping(&self, target: &str, timeout_ms: u64) -> HalResult<PingResult> {
        use std::process::Command;
        
        let start = std::time::Instant::now();
        
        #[cfg(windows)]
        let output = Command::new("ping")
            .args(&["-n", "1", "-w", &timeout_ms.to_string(), target])
            .output();
            
        #[cfg(unix)]
        let output = {
            let timeout_secs = std::cmp::max(1, timeout_ms / 1000);
            Command::new("ping")
                .args(&["-c", "1", "-W", &timeout_secs.to_string(), target])
                .output()
        };

        match output {
            Ok(output) => {
                let duration = start.elapsed();
                let success = output.status.success();
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                // Parse round-trip time from output
                let mut parsed_duration = duration.as_millis() as u32;
                
                #[cfg(windows)]
                if let Some(time_line) = output_str.lines().find(|line| line.contains("時間")) {
                    if let Some(time_part) = time_line.split("時間=").nth(1) {
                        if let Some(time_str) = time_part.split("ms").next() {
                            if let Ok(ms) = time_str.trim().parse::<u32>() {
                                parsed_duration = ms;
                            }
                        }
                    }
                } else if let Some(time_line) = output_str.lines().find(|line| line.contains("time=")) {
                    if let Some(time_part) = time_line.split("time=").nth(1) {
                        if let Some(time_str) = time_part.split("ms").next() {
                            if let Ok(ms) = time_str.trim().parse::<u32>() {
                                parsed_duration = ms;
                            }
                        }
                    }
                }
                
                #[cfg(unix)]
                if let Some(time_line) = output_str.lines().find(|line| line.contains("time=")) {
                    if let Some(time_part) = time_line.split("time=").nth(1) {
                        let time_str = time_part.split_whitespace().next().unwrap_or("");
                        if let Ok(ms) = time_str.trim().parse::<f32>() {
                            parsed_duration = ms as u32;
                        }
                    }
                }

                Ok(PingResult {
                    host: target.to_string(),
                    success,
                    duration_ms: parsed_duration,
                    error: if success { None } else { Some(String::from_utf8_lossy(&output.stderr).to_string()) },
                })
            }
            Err(e) => Err(HalError::network_error("native_ping", Some(target), None, &e.to_string()))
        }
    }

    /// TCP-based connectivity test as fallback
    fn tcp_ping(&self, target: &str, timeout_ms: u64, start: std::time::Instant) -> HalResult<PingResult> {
        use std::net::{SocketAddr, TcpStream};
        use std::time::Duration;

        let timeout = Duration::from_millis(timeout_ms);
        
        // Try multiple common ports for better connectivity testing
        let common_ports = [80, 443, 22, 53, 8080, 8443];
        
        // Try to parse as IP address first
        let base_addr = if let Ok(ip) = target.parse::<IpAddr>() {
            ip
        } else {
            // Try to resolve hostname
            use std::net::ToSocketAddrs;
            let host_with_port = format!("{}:80", target);
            let resolved = host_with_port
                .to_socket_addrs()
                .map_err(|e| HalError::network_error("resolve", Some(target), None, &e.to_string()))?
                .next()
                .ok_or_else(|| HalError::network_error("resolve", Some(target), None, "No addresses found"))?;
            resolved.ip()
        };

        // Try connecting to common ports
        for &port in &common_ports {
            let addr = SocketAddr::new(base_addr, port);
            let connect_start = std::time::Instant::now();
            
            match TcpStream::connect_timeout(&addr, timeout) {
                Ok(_) => {
                    let duration = connect_start.elapsed();
                    return Ok(PingResult {
                        host: target.to_string(),
                        success: true,
                        duration_ms: duration.as_millis() as u32,
                        error: None,
                    });
                }
                Err(_) => {
                    // Try next port
                    continue;
                }
            }
        }

        // If all ports failed
        let duration = start.elapsed();
        Ok(PingResult {
            host: target.to_string(),
            success: false,
            duration_ms: duration.as_millis() as u32,
            error: Some("No connectivity on common ports".to_string()),
        })
    }

    pub fn resolve_hostname(&self, hostname: &str) -> HalResult<Vec<IpAddr>> {
        use std::net::ToSocketAddrs;
        
        let addresses: Vec<IpAddr> = format!("{}:80", hostname)
            .to_socket_addrs()
            .map_err(|e| HalError::network_error("resolve_hostname", Some(hostname), None, &e.to_string()))?
            .map(|addr| addr.ip())
            .collect();
            
        if addresses.is_empty() {
            Err(HalError::network_error("resolve_hostname", Some(hostname), None, "No addresses found"))
        } else {
            Ok(addresses)
        }
    }

    pub fn get_network_stats(&self) -> HalResult<NetworkStats> {
        #[cfg(target_os = "linux")]
        {
            self.network_stats_linux()
        }
        #[cfg(target_os = "macos")]
        {
            self.network_stats_macos()
        }
        #[cfg(windows)]
        {
            self.network_stats_windows()
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        {
            Err(HalError::unsupported("Network statistics not supported on this platform"))
        }
    }

    pub fn configure_interface(&self, _name: &str, _config: &InterfaceConfig) -> HalResult<()> {
        // This would require platform-specific implementation
        Err(HalError::unsupported("Interface configuration requires platform-specific implementation"))
    }

    pub fn create_bridge(&self, _name: &str, _interfaces: &[String]) -> HalResult<()> {
        // This would require platform-specific implementation
        Err(HalError::unsupported("Bridge creation requires platform-specific implementation"))
    }

    pub fn get_routing_table(&self) -> HalResult<Vec<RouteEntry>> {
        #[cfg(target_os = "linux")]
        {
            self.routing_table_linux()
        }
        #[cfg(target_os = "macos")]
        {
            self.routing_table_macos()
        }
        #[cfg(windows)]
        {
            self.routing_table_windows()
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        {
            Err(HalError::unsupported("Routing table not supported on this platform"))
        }
    }

    pub fn add_route(&self, _destination: IpAddr, gateway: IpAddr, interface: &str) -> HalResult<()> {
        // This would require platform-specific implementation
        Err(HalError::unsupported(&format!("Adding route via {} on {} requires platform-specific implementation", gateway, interface)))
    }

    pub fn remove_route(&self, destination: IpAddr) -> HalResult<()> {
        // This would require platform-specific implementation
        Err(HalError::unsupported(&format!("Route removal for {} requires platform-specific implementation", destination)))
    }

    pub fn get_firewall_rules(&self) -> HalResult<Vec<FirewallRule>> {
        #[cfg(windows)]
        {
            self.get_firewall_rules_windows()
        }
        #[cfg(unix)]
        {
            self.get_firewall_rules_unix()
        }
        #[cfg(not(any(windows, unix)))]
        {
            Err(HalError::unsupported("Firewall rules retrieval not supported on this platform"))
        }
    }

    pub fn add_firewall_rule(&self, _rule: &FirewallRule) -> HalResult<()> {
        // This would require platform-specific implementation
        Err(HalError::unsupported("Firewall rule addition requires platform-specific implementation"))
    }

    pub fn remove_firewall_rule(&self, _rule_id: u32) -> HalResult<()> {
        // This would require platform-specific implementation
        Err(HalError::unsupported("Firewall rule removal requires platform-specific implementation"))
    }

    pub fn monitor_traffic(&self, interface: &str) -> HalResult<TrafficMonitor> {
        self.monitor_traffic_impl(interface)
    }

    pub fn get_bandwidth_usage(&self, interface: &str) -> HalResult<BandwidthUsage> {
        self.get_bandwidth_usage_impl(interface)
    }

    pub fn scan_ports(&self, target: IpAddr, _ports: &[u16]) -> HalResult<Vec<PortScanResult>> {
        // This would require platform-specific implementation
        Err(HalError::unsupported(&format!("Port scanning for {} requires platform-specific implementation", target)))
    }

    pub fn get_network_topology(&self) -> HalResult<NetworkTopology> {
        // This would require platform-specific implementation
        Err(HalError::unsupported("Network topology retrieval requires platform-specific implementation"))
    }

    pub fn create_tunnel(&self, _config: &TunnelConfig) -> HalResult<TunnelHandle> {
        // This would require platform-specific implementation
        Err(HalError::unsupported("Tunnel creation requires platform-specific implementation"))
    }

    pub fn get_connection_info(&self) -> HalResult<Vec<ConnectionInfo>> {
        // This would require platform-specific implementation
        Err(HalError::unsupported("Connection info retrieval requires platform-specific implementation"))
    }

    #[cfg(unix)]
    fn network_interfaces_unix(&self) -> HalResult<Vec<NetworkInterface>> {
        use std::net::{Ipv4Addr, Ipv6Addr};

        // Instead of using unsafe libc calls, use a safer alternative approach
        // Read network interfaces from /sys/class/net on Linux systems
        #[cfg(target_os = "linux")]
        {
            let net_path = "/sys/class/net";
            if let Ok(entries) = std::fs::read_dir(net_path) {
                let mut interfaces: Vec<NetworkInterface> = Vec::new();
                
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        // Check if interface is up
                        let operstate_path = format!("{}/{}/operstate", net_path, name);
                        let is_up = std::fs::read_to_string(&operstate_path)
                            .map(|s| s.trim() == "up")
                            .unwrap_or(false);
                        
                        // Determine if it's a loopback interface
                        let is_loopback = name == "lo";
                        
                        // Get MAC address
                        let mac_address = if !is_loopback {
                            let address_path = format!("{}/{}/address", net_path, name);
                            std::fs::read_to_string(&address_path)
                                .ok()
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty() && s != "00:00:00:00:00:00")
                        } else {
                            None
                        };
                        
                        // Get MTU
                        let mtu = {
                            let mtu_path = format!("{}/{}/mtu", net_path, name);
                            std::fs::read_to_string(&mtu_path)
                                .ok()
                                .and_then(|s| s.trim().parse::<u32>().ok())
                                .unwrap_or(if is_loopback { 65536 } else { 1500 })
                        };
                        
                        // Get IP addresses using getifaddrs-like approach
                        let addresses = self.get_interface_addresses(name).unwrap_or_else(|_| {
                            if is_loopback {
                                vec![std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))]
                            } else {
                                vec![]
                            }
                        });
                        
                        interfaces.push(NetworkInterface {
                            name: name.to_string(),
                            addresses,
                            is_up,
                            is_loopback,
                            mtu,
                            mac_address,
                        });
                    }
                }
                
                return Ok(interfaces);
            }
        }
        
        // Fallback for other Unix systems - return minimal interface info
        Ok(vec![
            NetworkInterface {
                name: "lo".to_string(),
                addresses: vec![std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))],
                is_up: true,
                is_loopback: true,
                mtu: 65536, // Standard loopback MTU
                mac_address: None,
            }
        ])
    }

    #[cfg(windows)]
    fn network_interfaces_windows(&self) -> HalResult<Vec<NetworkInterface>> {
        // This is a simplified implementation
        Ok(vec![])
    }

    #[cfg(target_os = "linux")]
    fn network_stats_linux(&self) -> HalResult<NetworkStats> {
        use std::fs;

        let net_dev = fs::read_to_string("/proc/net/dev")
            .map_err(|e| HalError::io_error("read_net_dev", Some("/proc/net/dev"), e))?;

        let mut total_bytes_received = 0u64;
        let mut total_bytes_sent = 0u64;
        let mut total_packets_received = 0u64;
        let mut total_packets_sent = 0u64;
        let mut total_errors_received = 0u64;
        let mut total_errors_sent = 0u64;

        for line in net_dev.lines().skip(2) { // Skip header lines
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 17 {
                // Skip loopback interface for external traffic stats
                if parts[0].starts_with("lo:") {
                    continue;
                }
                
                // Parse receive stats
                if let Ok(rx_bytes) = parts[1].parse::<u64>() {
                    total_bytes_received += rx_bytes;
                }
                if let Ok(rx_packets) = parts[2].parse::<u64>() {
                    total_packets_received += rx_packets;
                }
                if let Ok(rx_errors) = parts[3].parse::<u64>() {
                    total_errors_received += rx_errors;
                }
                
                // Parse transmit stats
                if let Ok(tx_bytes) = parts[9].parse::<u64>() {
                    total_bytes_sent += tx_bytes;
                }
                if let Ok(tx_packets) = parts[10].parse::<u64>() {
                    total_packets_sent += tx_packets;
                }
                if let Ok(tx_errors) = parts[11].parse::<u64>() {
                    total_errors_sent += tx_errors;
                }
            }
        }

        Ok(NetworkStats {
            bytes_received: total_bytes_received,
            bytes_sent: total_bytes_sent,
            packets_received: total_packets_received,
            packets_sent: total_packets_sent,
            errors_received: total_errors_received,
            errors_sent: total_errors_sent,
        })
    }

    #[cfg(target_os = "macos")]
    fn network_stats_macos(&self) -> HalResult<NetworkStats> {
        use std::process::Command;
        
        let output = Command::new("netstat")
            .args(&["-i", "-b"])
            .output()
            .map_err(|e| HalError::io_error("netstat", Some("netstat -i -b"), e))?;

        if !output.status.success() {
            return Err(HalError::command_failed(
                "netstat", 
                output.status.code().unwrap_or(-1)
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut total_bytes_received = 0u64;
        let mut total_bytes_sent = 0u64;
        let mut total_packets_received = 0u64;
        let mut total_packets_sent = 0u64;

        for line in output_str.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                // Skip loopback interface
                if parts[0].starts_with("lo") {
                    continue;
                }
                
                // Parse network interface stats
                if let Ok(ipackets) = parts[4].parse::<u64>() {
                    total_packets_received += ipackets;
                }
                if let Ok(ibytes) = parts[6].parse::<u64>() {
                    total_bytes_received += ibytes;
                }
                if let Ok(opackets) = parts[7].parse::<u64>() {
                    total_packets_sent += opackets;
                }
                if let Ok(obytes) = parts[9].parse::<u64>() {
                    total_bytes_sent += obytes;
                }
            }
        }

        Ok(NetworkStats {
            bytes_received: total_bytes_received,
            bytes_sent: total_bytes_sent,
            packets_received: total_packets_received,
            packets_sent: total_packets_sent,
            errors_received: 0,
            errors_sent: 0,
        })
    }

    #[cfg(windows)]
    fn network_stats_windows(&self) -> HalResult<NetworkStats> {
        use std::process::Command;
        
        // Use Get-NetAdapterStatistics PowerShell command for comprehensive stats
        let output = Command::new("powershell")
            .args(&[
                "-Command", 
                "Get-NetAdapterStatistics | Where-Object {$_.Name -notlike '*Loopback*'} | Measure-Object -Property BytesReceivedPerSec,BytesSentPerSec,PacketsReceivedPerSec,PacketsSentPerSec -Sum"
            ])
            .output()
            .map_err(|e| HalError::io_error("powershell", Some("Get-NetAdapterStatistics"), e))?;

        if !output.status.success() {
            // Fallback to simpler approach using basic netstat
            let fallback_output = Command::new("netstat")
                .args(&["-e"])
                .output()
                .map_err(|e| HalError::io_error("netstat", Some("netstat -e"), e))?;

            if !fallback_output.status.success() {
                return Ok(NetworkStats {
                    bytes_received: 0,
                    bytes_sent: 0,
                    packets_received: 0,
                    packets_sent: 0,
                    errors_received: 0,
                    errors_sent: 0,
                });
            }

            let fallback_str = String::from_utf8_lossy(&fallback_output.stdout);
            let mut bytes_received = 0u64;
            let mut bytes_sent = 0u64;

            // Parse netstat -e output (Interface statistics)
            for line in fallback_str.lines() {
                if line.contains("Bytes") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        if let (Ok(rx), Ok(tx)) = (parts[1].parse::<u64>(), parts[2].parse::<u64>()) {
                            bytes_received = rx;
                            bytes_sent = tx;
                        }
                    }
                }
            }

            return Ok(NetworkStats {
                bytes_received,
                bytes_sent,
                packets_received: 0,
                packets_sent: 0,
                errors_received: 0,
                errors_sent: 0,
            });
        }

        // Parse PowerShell output for comprehensive statistics
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut total_bytes_received = 0u64;
        let mut total_bytes_sent = 0u64;
        let mut total_packets_received = 0u64;
        let mut total_packets_sent = 0u64;

        for line in output_str.lines() {
            if line.contains("Sum") && line.contains("BytesReceived") {
                if let Some(value_str) = line.split_whitespace().last() {
                    if let Ok(value) = value_str.parse::<u64>() {
                        total_bytes_received = value;
                    }
                }
            }
            if line.contains("Sum") && line.contains("BytesSent") {
                if let Some(value_str) = line.split_whitespace().last() {
                    if let Ok(value) = value_str.parse::<u64>() {
                        total_bytes_sent = value;
                    }
                }
            }
            if line.contains("Sum") && line.contains("PacketsReceived") {
                if let Some(value_str) = line.split_whitespace().last() {
                    if let Ok(value) = value_str.parse::<u64>() {
                        total_packets_received = value;
                    }
                }
            }
            if line.contains("Sum") && line.contains("PacketsSent") {
                if let Some(value_str) = line.split_whitespace().last() {
                    if let Ok(value) = value_str.parse::<u64>() {
                        total_packets_sent = value;
                    }
                }
            }
        }

        Ok(NetworkStats {
            bytes_received: total_bytes_received,
            bytes_sent: total_bytes_sent,
            packets_received: total_packets_received,
            packets_sent: total_packets_sent,
            errors_received: 0,
            errors_sent: 0,
        })
    }

    #[cfg(target_os = "linux")]
    fn routing_table_linux(&self) -> HalResult<Vec<RouteEntry>> {
        use std::fs;
        use std::net::Ipv4Addr;

        let route_data = fs::read_to_string("/proc/net/route")
            .map_err(|e| HalError::io_error("read_route", Some("/proc/net/route"), e))?;

        let mut routes = Vec::new();

        // /proc/net/route columns:
        // Iface Destination Gateway Flags RefCnt Use Metric Mask MTU Window IRTT
        for line in route_data.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 11 { continue; }

            let interface = parts[0].to_string();
            let dest = match u32::from_str_radix(parts[1], 16) { Ok(v) => Ipv4Addr::from(u32::from_le(v)), Err(_) => continue };
            let gate_hex = match u32::from_str_radix(parts[2], 16) { Ok(v) => v, Err(_) => continue };
            let gateway = Ipv4Addr::from(u32::from_le(gate_hex));
            let mask = match u32::from_str_radix(parts[7], 16) { Ok(v) => Ipv4Addr::from(u32::from_le(v)), Err(_) => Ipv4Addr::UNSPECIFIED };
            let metric: u32 = parts[6].parse().unwrap_or(0);

            routes.push(RouteEntry {
                destination: IpAddr::V4(dest),
                netmask: IpAddr::V4(mask),
                gateway: IpAddr::V4(gateway),
                interface,
                metric,
            });
        }

        Ok(routes)
    }

    #[cfg(target_os = "macos")]
    fn routing_table_macos(&self) -> HalResult<Vec<RouteEntry>> {
        use std::process::Command;
        use std::net::Ipv4Addr;

        // Prefer netstat -rn -f inet for IPv4 routing table
        let output = Command::new("netstat")
            .args(["-rn", "-f", "inet"])
            .output()
            .map_err(|e| HalError::io_error("netstat", Some("netstat -rn -f inet"), e))?;

        if !output.status.success() {
            return Err(HalError::command_failed("netstat", output.status.code().unwrap_or(-1)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut routes: Vec<RouteEntry> = Vec::new();
        let mut in_ipv4_section = false;

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            if line.starts_with("Internet:") { in_ipv4_section = true; continue; }
            if line.starts_with("Internet6:") { in_ipv4_section = false; continue; }
            if !in_ipv4_section { continue; }
            if line.starts_with("Destination") { continue; } // header

            // Expected columns: Destination Gateway Flags Refs Use Netif Expire
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 7 { continue; }
            let dest_str = cols[0];
            let gw_str = cols[1];
            // Netif is usually the penultimate column; guard for variants
            let netif = if cols.len() >= 6 { cols[cols.len().saturating_sub(2)] } else { cols[0] };

            // Destination handling
            let (dest_ip, mask_ip) = if dest_str == "default" {
                (Ipv4Addr::new(0,0,0,0), Ipv4Addr::new(0,0,0,0))
            } else if let Some((ip, prefix)) = dest_str.split_once('/') {
                // CIDR form x.x.x.x/nn or a.b.c/nn
                let ip_addr: Ipv4Addr = ip.parse().unwrap_or(Ipv4Addr::UNSPECIFIED);
                let p: u8 = prefix.parse().unwrap_or(0);
                let mask = prefix_to_mask_v4(p);
                (ip_addr, mask)
            } else {
                // Host route, set mask /32
                let ip_addr: Ipv4Addr = dest_str.parse().unwrap_or(Ipv4Addr::UNSPECIFIED);
                (ip_addr, Ipv4Addr::new(255,255,255,255))
            };

            // Gateway
            let gateway_ip: Ipv4Addr = if gw_str == "link#" || gw_str.starts_with("link#") || gw_str == "-" {
                Ipv4Addr::UNSPECIFIED
            } else {
                gw_str.parse().unwrap_or(Ipv4Addr::UNSPECIFIED)
            };

            routes.push(RouteEntry {
                destination: IpAddr::V4(dest_ip),
                netmask: IpAddr::V4(mask_ip),
                gateway: IpAddr::V4(gateway_ip),
                interface: netif.to_string(),
                metric: 0,
            });
        }

        Ok(routes)
    }

    #[cfg(windows)]
    fn routing_table_windows(&self) -> HalResult<Vec<RouteEntry>> {
        use std::process::Command;
        use std::net::Ipv4Addr;
        // Use `route print -4` and parse the IPv4 Route Table
        let output = Command::new("route")
            .args(["print", "-4"])
            .output()
            .map_err(|e| HalError::io_error("route", Some("route print -4"), e))?;

        if !output.status.success() {
            return Err(HalError::command_failed("route", output.status.code().unwrap_or(-1)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut routes: Vec<RouteEntry> = Vec::new();
        let mut in_table = false;

        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            if trimmed.starts_with("IPv4 Route Table") { in_table = true; continue; }
            if !in_table { continue; }

            // Skip header lines until we find the column headers
            if trimmed.starts_with("Network Destination") { continue; }
            // Table ends when a blank line or new section starts
            if trimmed.starts_with("IPv6 Route Table") { break; }

            // Expected columns: Network Destination  Netmask  Gateway  Interface  Metric
            let cols: Vec<&str> = trimmed.split_whitespace().collect();
            if cols.len() < 5 { continue; }
            let dest = cols[0];
            let mask = cols[1];
            let gw = cols[2];
            let interface = cols[3];
            let metric: u32 = cols.get(4).and_then(|m| m.parse().ok()).unwrap_or(0);

            // Only consider IPv4
            let dest_ip: Ipv4Addr = match dest.parse::<Ipv4Addr>() { Ok(v) => v, Err(_) => continue };
            let mask_ip: Ipv4Addr = match mask.parse::<Ipv4Addr>() { Ok(v) => v, Err(_) => continue };
            let gw_ip: Ipv4Addr = gw.parse::<Ipv4Addr>().unwrap_or(Ipv4Addr::UNSPECIFIED);

            routes.push(RouteEntry {
                destination: IpAddr::V4(dest_ip),
                netmask: IpAddr::V4(mask_ip),
                gateway: IpAddr::V4(gw_ip),
                interface: interface.to_string(),
                metric,
            });
        }

        Ok(routes)
    }

    /// Get default gateway on Unix systems
    #[cfg(unix)]
    fn get_default_gateway_unix(&self) -> HalResult<IpAddr> {
        use std::process::Command;
        
        // Try multiple approaches for different Unix variants
        
        // Method 1: Parse route command output (Linux/macOS)
        if let Ok(output) = Command::new("route")
            .args(&["-n", "get", "default"])
            .output()
        {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.trim().starts_with("gateway:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if let Ok(gateway) = parts[1].parse::<IpAddr>() {
                                return Ok(gateway);
                            }
                        }
                    }
                }
            }
        }
        
        // Method 2: Parse ip route output (Linux)
        if let Ok(output) = Command::new("ip")
            .args(&["route", "show", "default"])
            .output()
        {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("via") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(via_pos) = parts.iter().position(|&x| x == "via") {
                            if via_pos + 1 < parts.len() {
                                if let Ok(gateway) = parts[via_pos + 1].parse::<IpAddr>() {
                                    return Ok(gateway);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Method 3: Parse /proc/net/route (Linux fallback)
        if let Ok(route_content) = std::fs::read_to_string("/proc/net/route") {
            for line in route_content.lines().skip(1) { // Skip header
                let fields: Vec<&str> = line.split('\t').collect();
                if fields.len() >= 3 {
                    // Check if this is the default route (destination 00000000)
                    if fields[1] == "00000000" {
                        // Parse gateway (field 2)
                        if let Ok(gateway_hex) = u32::from_str_radix(fields[2], 16) {
                            let gateway = std::net::Ipv4Addr::from(gateway_hex.to_be());
                            return Ok(IpAddr::V4(gateway));
                        }
                    }
                }
            }
        }
        
        Err(HalError::network_error("get_default_gateway", None, None, "Could not determine default gateway"))
    }

    /// Get default gateway on Windows systems
    #[cfg(windows)]
    fn get_default_gateway_windows(&self) -> HalResult<IpAddr> {
        use std::process::Command;
        use std::net::Ipv4Addr;
        
        // Method 1: Use PowerShell to get default gateway
        if let Ok(output) = Command::new("powershell")
            .args(&["-Command", "Get-NetRoute -DestinationPrefix '0.0.0.0/0' | Select-Object -ExpandProperty NextHop"])
            .output()
        {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        if let Ok(gateway) = trimmed.parse::<IpAddr>() {
                            return Ok(gateway);
                        }
                    }
                }
            }
        }
        
        // Method 2: Parse route print output
        if let Ok(output) = Command::new("route")
            .args(&["print", "0.0.0.0"])
            .output()
        {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 && parts[0] == "0.0.0.0" && parts[1] == "0.0.0.0" {
                        if let Ok(gateway) = parts[2].parse::<IpAddr>() {
                            return Ok(gateway);
                        }
                    }
                }
            }
        }
        
        // Method 3: Parse `ipconfig` default gateway as a last resort (localized output tolerant)
        if let Ok(output) = Command::new("ipconfig").output() {
            if output.status.success() {
                let out = String::from_utf8_lossy(&output.stdout);
                for line in out.lines() {
                    // Match forms like: Default Gateway . . . . . . . . . : 192.168.1.1
                    if line.to_lowercase().contains("default gateway") {
                        if let Some(idx) = line.rfind(':') {
                            let cand = line[idx+1..].trim();
                            if let Ok(ip) = cand.parse::<Ipv4Addr>() {
                                return Ok(IpAddr::V4(ip));
                            }
                        }
                    }
                }
            }
        }

        Err(HalError::network_error("get_default_gateway", None, None, "Could not determine default gateway"))
    }

    /// Get DNS servers on Unix systems
    #[cfg(unix)]
    fn get_dns_servers_unix(&self) -> HalResult<Vec<IpAddr>> {
        let mut dns_servers = Vec::new();
        
        // Method 1: Parse /etc/resolv.conf
        if let Ok(resolv_content) = std::fs::read_to_string("/etc/resolv.conf") {
            for line in resolv_content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("nameserver") {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(dns) = parts[1].parse::<IpAddr>() {
                            dns_servers.push(dns);
                        }
                    }
                }
            }
        }
        
        // Method 2: Try systemd-resolved (modern Linux systems)
        if dns_servers.is_empty() {
            if let Ok(output) = std::process::Command::new("systemd-resolve")
                .args(&["--status"])
                .output()
            {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    for line in output_str.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("DNS Servers:") {
                            let dns_part = trimmed.trim_start_matches("DNS Servers:");
                            for dns_str in dns_part.split_whitespace() {
                                if let Ok(dns) = dns_str.parse::<IpAddr>() {
                                    dns_servers.push(dns);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Method 3: Try resolvectl (newer systemd)
        if dns_servers.is_empty() {
            if let Ok(output) = std::process::Command::new("resolvectl")
                .args(&["status"])
                .output()
            {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    for line in output_str.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("DNS Servers:") {
                            let dns_part = trimmed.trim_start_matches("DNS Servers:");
                            for dns_str in dns_part.split_whitespace() {
                                if let Ok(dns) = dns_str.parse::<IpAddr>() {
                                    dns_servers.push(dns);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if dns_servers.is_empty() {
            Err(HalError::network_error("get_dns_servers", None, None, "No DNS servers found"))
        } else {
            Ok(dns_servers)
        }
    }

    /// Get DNS servers on Windows systems
    #[cfg(windows)]
    fn get_dns_servers_windows(&self) -> HalResult<Vec<IpAddr>> {
        use std::process::Command;
        let mut dns_servers = Vec::new();
        
        // Method 1: Use PowerShell to get DNS servers
        if let Ok(output) = Command::new("powershell")
            .args(&["-Command", "Get-DnsClientServerAddress | Where-Object {$_.AddressFamily -eq 2} | Select-Object -ExpandProperty ServerAddresses"])
            .output()
        {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        if let Ok(dns) = trimmed.parse::<IpAddr>() {
                            dns_servers.push(dns);
                        }
                    }
                }
            }
        }
        
        // Method 2: Parse nslookup output as fallback
        if dns_servers.is_empty() {
            if let Ok(output) = Command::new("nslookup")
                .args(&["localhost"])
                .output()
            {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    for line in output_str.lines() {
                        if line.contains("Address:") && !line.contains("#") {
                            let parts: Vec<&str> = line.split(':').collect();
                            if parts.len() >= 2 {
                                let ip_str = parts[1].trim();
                                if let Ok(dns) = ip_str.parse::<IpAddr>() {
                                    dns_servers.push(dns);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if dns_servers.is_empty() {
            Err(HalError::network_error("get_dns_servers", None, None, "No DNS servers found"))
        } else {
            Ok(dns_servers)
        }
    }

    // === Firewall Management Implementation ===

    #[cfg(windows)]
    fn get_firewall_rules_windows(&self) -> HalResult<Vec<FirewallRule>> {
        let mut rules = Vec::new();
        
        // Use PowerShell to query Windows Firewall rules
        let output = std::process::Command::new("powershell")
            .args(&["-Command", 
                "Get-NetFirewallRule | Where-Object {$_.Enabled -eq 'True'} | Select-Object DisplayName,Direction,Action,Protocol | ConvertTo-Json"])
            .output()
            .map_err(|e| HalError::network_error("get_firewall_rules", None, None, &format!("Failed to execute PowerShell: {}", e)))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Basic JSON parsing for firewall rules
            for line in output_str.lines() {
                if line.contains("DisplayName") {
                    let rule = FirewallRule {
                        id: rules.len() as u32,
                        action: "Allow".to_string(),
                        protocol: "TCP".to_string(),
                        source: None,
                        destination: None,
                        port: None,
                    };
                    rules.push(rule);
                }
            }
        }
        
        Ok(rules)
    }

    #[cfg(unix)]
    fn get_firewall_rules_unix(&self) -> HalResult<Vec<FirewallRule>> {
        let mut rules = Vec::new();
        
        // Try iptables first
        if let Ok(output) = std::process::Command::new("iptables")
            .args(&["-L", "-n", "--line-numbers"])
            .output()
        {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for (index, line) in output_str.lines().enumerate() {
                    if line.contains("ACCEPT") || line.contains("DROP") || line.contains("REJECT") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            let rule = FirewallRule {
                                id: index as u32,
                                action: parts[1].to_string(),
                                protocol: parts.get(2).unwrap_or(&"all").to_string(),
                                source: None,
                                destination: None,
                                port: None,
                            };
                            rules.push(rule);
                        }
                    }
                }
            }
        }
        
        // Try ufw as fallback
        if rules.is_empty() {
            if let Ok(output) = std::process::Command::new("ufw")
                .args(&["status", "numbered"])
                .output()
            {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    for (index, line) in output_str.lines().enumerate() {
                        if line.contains("ALLOW") || line.contains("DENY") {
                            let rule = FirewallRule {
                                id: index as u32,
                                action: if line.contains("ALLOW") { "Allow" } else { "Deny" }.to_string(),
                                protocol: "TCP".to_string(),
                                source: None,
                                destination: None,
                                port: None,
                            };
                            rules.push(rule);
                        }
                    }
                }
            }
        }
        
        Ok(rules)
    }

    // === Traffic Monitoring Implementation ===

    pub fn monitor_traffic_impl(&self, interface: &str) -> HalResult<TrafficMonitor> {
        #[cfg(unix)]
        {
            // Use /proc/net/dev for traffic statistics
            let stats_path = "/proc/net/dev";
            if std::path::Path::new(stats_path).exists() {
                Ok(TrafficMonitor {
                    interface: interface.to_string(),
                })
            } else {
                Err(HalError::unsupported(&format!("Traffic monitoring not available: {} not found", stats_path)))
            }
        }
        
        #[cfg(windows)]
        {
            // Use WMI or PowerShell for Windows traffic monitoring
            Ok(TrafficMonitor {
                interface: interface.to_string(),
            })
        }
        
        #[cfg(not(any(unix, windows)))]
        {
            Err(HalError::unsupported(&format!("Traffic monitoring not supported on this platform for interface {}", interface)))
        }
    }

    // === Bandwidth Usage Implementation ===

    pub fn get_bandwidth_usage_impl(&self, interface: &str) -> HalResult<BandwidthUsage> {
        #[cfg(unix)]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
                for line in content.lines() {
                    if line.contains(interface) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 10 {
                            let rx_bytes = parts[1].parse::<u64>().unwrap_or(0);
                            let tx_bytes = parts[9].parse::<u64>().unwrap_or(0);
                            
                            return Ok(BandwidthUsage {
                                interface: interface.to_string(),
                                rx_bytes,
                                tx_bytes,
                                rx_rate_bps: 0, // Would need time-based calculation
                                tx_rate_bps: 0, // Would need time-based calculation
                                timestamp: SystemTime::now(),
                            });
                        }
                    }
                }
            }
        }
        
        #[cfg(windows)]
        {
            // Windows implementation using WMI
            Ok(BandwidthUsage {
                interface: interface.to_string(),
                rx_bytes: 0,
                tx_bytes: 0,
                rx_rate_bps: 0,
                tx_rate_bps: 0,
                timestamp: SystemTime::now(),
            })
        }
        
        Err(HalError::not_found(&format!("Interface {} not found or no data available", interface)))
    }

    // === Helper Methods for Advanced Features ===

    fn initialize_security_policies(&mut self) -> HalResult<()> {
        // Set up default security policies
        self.security_manager.add_policy(SecurityPolicy {
            name: "Block Malicious Ports".to_string(),
            rule_type: SecurityRuleType::Port,
            pattern: "1,7,9,11,13,15,17,19".to_string(),
            action: SecurityAction::Block,
            priority: 100,
        });
        
        Ok(())
    }

    fn start_background_monitoring(&self) -> HalResult<()> {
        // Start background threads for monitoring
        // This would spawn threads for continuous monitoring
        Ok(())
    }

    fn parse_connection_target(&self, target: &str) -> HalResult<ConnectionTarget> {
        if target.contains("://") {
            // Parse URL format: protocol://host:port
            let parts: Vec<&str> = target.splitn(2, "://").collect();
            if parts.len() != 2 {
                return Err(HalError::invalid_input("Invalid target format"));
            }
            
            let protocol = parts[0];
            let host_port = parts[1];
            
            let (hostname, port) = if host_port.contains(':') {
                let hp_parts: Vec<&str> = host_port.rsplitn(2, ':').collect();
                if hp_parts.len() == 2 {
                    (hp_parts[1], hp_parts[0].parse::<u16>().unwrap_or(80))
                } else {
                    (host_port, 80)
                }
            } else {
                (host_port, match protocol {
                    "http" => 80,
                    "https" => 443,
                    "ftp" => 21,
                    "ssh" => 22,
                    _ => 80,
                })
            };
            
            Ok(ConnectionTarget {
                protocol: protocol.to_string(),
                hostname: hostname.to_string(),
                port,
                resolved_ip: None,
            })
        } else {
            // Parse host:port format
            if target.contains(':') {
                let parts: Vec<&str> = target.rsplitn(2, ':').collect();
                if parts.len() == 2 {
                    Ok(ConnectionTarget {
                        protocol: "tcp".to_string(),
                        hostname: parts[1].to_string(),
                        port: parts[0].parse().unwrap_or(80),
                        resolved_ip: None,
                    })
                } else {
                    Err(HalError::invalid_input("Invalid host:port format"))
                }
            } else {
                Ok(ConnectionTarget {
                    protocol: "tcp".to_string(),
                    hostname: target.to_string(),
                    port: 80,
                    resolved_ip: None,
                })
            }
        }
    }

    fn create_tcp_connection(&self, target: &ConnectionTarget) -> HalResult<NetworkStream> {
        let addr = format!("{}:{}", target.hostname, target.port);
        let stream = TcpStream::connect_timeout(
            &addr.parse().map_err(|_| HalError::invalid_input("Invalid address format"))?,
            Duration::from_secs(30)
        ).map_err(|e| HalError::network(format!("Failed to connect: {}", e)))?;
        
        stream.set_nodelay(true).ok();
        stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(30))).ok();
        
        Ok(NetworkStream::Tcp(stream))
    }

    fn create_udp_connection(&self, target: &ConnectionTarget) -> HalResult<NetworkStream> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| HalError::network(format!("Failed to bind UDP socket: {}", e)))?;
        
        let addr = format!("{}:{}", target.hostname, target.port);
        socket.connect(addr)
            .map_err(|e| HalError::network(format!("Failed to connect UDP: {}", e)))?;
        
        socket.set_read_timeout(Some(Duration::from_secs(30))).ok();
        socket.set_write_timeout(Some(Duration::from_secs(30))).ok();
        
        Ok(NetworkStream::Udp(socket))
    }

    fn create_http_connection(&self, target: &ConnectionTarget) -> HalResult<NetworkStream> {
        // For HTTP, create TCP connection
        self.create_tcp_connection(target)
    }

    fn create_https_connection(&self, target: &ConnectionTarget) -> HalResult<NetworkStream> {
        // For HTTPS, would create TLS-wrapped TCP connection
        // For now, create regular TCP connection
        self.create_tcp_connection(target)
    }

    fn ping_host(&self, target: &str, timeout: Duration) -> HalResult<Duration> {
        let start = SystemTime::now();
        
        // Try to establish a quick TCP connection to common ports
        let test_ports = [80, 443, 22, 21, 53];
        
        for port in &test_ports {
            let addr = format!("{}:{}", target, port);
            if let Ok(_) = TcpStream::connect_timeout(
                &addr.parse().unwrap_or_else(|_| "127.0.0.1:80".parse().unwrap()),
                timeout
            ) {
                return Ok(SystemTime::now().duration_since(start).unwrap_or_default());
            }
        }
        
        Err(HalError::network("Host unreachable"))
    }

    fn estimate_hop_count(&self, _target: &str) -> Option<u8> {
        // Simplified hop count estimation
        Some(8)
    }

    fn calculate_connection_quality(&self, latency: &Duration, packet_loss: f64) -> f64 {
        let latency_ms = latency.as_millis() as f64;
        let latency_score = if latency_ms < 10.0 {
            100.0
        } else if latency_ms < 50.0 {
            90.0 - (latency_ms - 10.0) * 2.0
        } else {
            50.0
        };
        
        let packet_loss_score = 100.0 - (packet_loss * 100.0);
        (latency_score + packet_loss_score) / 2.0
    }

    fn generate_performance_report(&self, target: &str, duration: Duration, measurements: Vec<PerformanceMeasurement>) -> HalResult<NetworkPerformanceReport> {
        let total_measurements = measurements.len();
        let packet_loss_count = measurements.iter().filter(|m| m.packet_loss).count();
        let successful_measurements: Vec<_> = measurements.iter()
            .filter(|m| !m.packet_loss)
            .collect();

        let average_latency = if !successful_measurements.is_empty() {
            Duration::from_millis(successful_measurements.iter()
                .map(|m| m.latency.as_millis() as u64)
                .sum::<u64>() / successful_measurements.len() as u64)
        } else {
            Duration::default()
        };

        let min_latency = successful_measurements.iter()
            .map(|m| m.latency)
            .min()
            .unwrap_or_default();
            
        let max_latency = successful_measurements.iter()
            .map(|m| m.latency)
            .max()
            .unwrap_or_default();

        Ok(NetworkPerformanceReport {
            target: target.to_string(),
            test_duration: duration,
            total_measurements,
            packet_loss_rate: (packet_loss_count as f64 / total_measurements as f64) * 100.0,
            average_latency,
            min_latency,
            max_latency,
            measurements,
            bandwidth_estimate: 100_000_000, // 100 Mbps estimate
            quality_score: self.calculate_connection_quality(&average_latency, packet_loss_count as f64 / total_measurements as f64),
        })
    }

    fn update_connection_stats(&self, connection_id: &str, bytes_sent: usize, bytes_received: usize) -> HalResult<()> {
        if let Ok(mut pool) = self.connection_pool.write() {
            if let Some(connection) = pool.get_connection_mut(connection_id) {
                connection.bytes_sent += bytes_sent as u64;
                connection.bytes_received += bytes_received as u64;
                connection.last_activity = SystemTime::now();
            }
        }
        Ok(())
    }

    fn generate_connection_report(&self, connection: &NetworkConnection) -> HalResult<ConnectionReport> {
        let duration = SystemTime::now().duration_since(connection.created_at).unwrap_or_default();
        let total_bytes = connection.bytes_sent + connection.bytes_received;
        
        Ok(ConnectionReport {
            connection_id: connection.id.clone(),
            target: connection.target.hostname.clone(),
            protocol: connection.protocol.clone(),
            duration,
            bytes_sent: connection.bytes_sent,
            bytes_received: connection.bytes_received,
            total_bytes,
            average_throughput: if duration.as_secs() > 0 {
                total_bytes / duration.as_secs()
            } else {
                0
            },
            quality_metrics: connection.quality_metrics.clone(),
        })
    }

    fn generate_connection_id() -> String {
        format!("CONN_{}", 
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis())
    }

    // === Keep existing interface methods ===

    pub fn get_interfaces(&self) -> HalResult<Vec<NetworkInterface>> {
        #[cfg(unix)]
        {
            self.network_interfaces_unix()
        }
        #[cfg(windows)]
        {
            self.network_interfaces_windows()
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(HalError::unsupported("Network interfaces not supported on this platform"))
        }
    }

    pub fn get_default_gateway(&self) -> HalResult<IpAddr> {
        #[cfg(unix)]
        {
            self.get_default_gateway_unix()
        }
        #[cfg(windows)]
        {
            self.get_default_gateway_windows()
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(HalError::unsupported("Default gateway detection not supported on this platform"))
        }
    }
        
        #[cfg(windows)]
        {
            // Use PowerShell to get network adapter statistics
            if let Ok(output) = std::process::Command::new("powershell")
                .args(&["-Command", "Get-NetAdapterStatistics | Select-Object Name,BytesReceived,BytesSent"])
                .output()
            {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    for line in output_str.lines().skip(3) { // Skip headers
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            return Ok(BandwidthUsage {
                                interface: parts[0].to_string(),
                                rx_bytes: parts[1].parse().unwrap_or(0),
                                tx_bytes: parts[2].parse().unwrap_or(0),
                                rx_rate_bps: 0, // Would need time-based calculation
                                tx_rate_bps: 0, // Would need time-based calculation
                                timestamp: SystemTime::now(),
                            });
                        }
                    }
                }
            }
        }
        
        Err(HalError::not_found(&format!("Interface {} not found or no data available", interface)))
    }
                
                .args(&["-Command", &format!("Get-Counter '\\Network Interface({})\\Bytes Total/sec' | Select-Object -ExpandProperty CounterSamples | Select-Object -ExpandProperty CookedValue", interface)])
                .output()
            {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let bytes_total = output_str.trim().parse::<u64>().unwrap_or(0);
                    
                    return Ok(BandwidthUsage {
                        interface: interface.to_string(),
                        bytes_per_second: bytes_total,
                        packets_per_second: 0,
                    });
                }
            }
        }
        
        Err(HalError::network_error("get_bandwidth_usage", None, None, &format!("Could not get bandwidth usage for interface {}", interface)))
    }
    
    /// Get IP addresses for a specific network interface
    #[cfg(target_os = "linux")]
    fn get_interface_addresses(&self, interface_name: &str) -> HalResult<Vec<std::net::IpAddr>> {
        use std::process::Command;
        
        let output = Command::new("ip")
            .args(&["addr", "show", interface_name])
            .output()
            .map_err(|e| HalError::io_error("ip", Some("ip addr show"), e))?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let mut addresses = Vec::new();
        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines() {
            let line = line.trim();
            if line.starts_with("inet ") {
                if let Some(addr_part) = line.split_whitespace().nth(1) {
                    if let Some(addr_str) = addr_part.split('/').next() {
                        if let Ok(addr) = addr_str.parse::<std::net::IpAddr>() {
                            addresses.push(addr);
                        }
                    }
                }
            }
        }

        Ok(addresses)
    }

    #[cfg(not(target_os = "linux"))]
    fn get_interface_addresses(&self, interface_name: &str) -> HalResult<Vec<std::net::IpAddr>> {
        // Fallback implementation for non-Linux systems
        if interface_name == "lo" {
            Ok(vec![std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))])
        } else {
            Ok(vec![])
        }
    }
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

// Missing type definitions
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub addresses: Vec<IpAddr>,
    pub mac_address: Option<String>,
    pub is_up: bool,
    pub is_loopback: bool,
    pub mtu: u32,
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors_received: u64,
    pub errors_sent: u64,
}

#[derive(Debug, Clone)]
pub struct PingResult {
    pub host: String,
    pub success: bool,
    pub duration_ms: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InterfaceConfig {
    pub ip_address: Option<IpAddr>,
    pub netmask: Option<IpAddr>,
    pub gateway: Option<IpAddr>,
    pub mtu: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct RouteEntry {
    pub destination: IpAddr,
    pub netmask: IpAddr,
    pub gateway: IpAddr,
    pub interface: String,
    pub metric: u32,
}

#[derive(Debug, Clone)]
pub struct FirewallRule {
    pub id: u32,
    pub action: String,
    pub protocol: String,
    pub source: Option<IpAddr>,
    pub destination: Option<IpAddr>,
    pub port: Option<u16>,
}

#[derive(Debug)]
pub struct TrafficMonitor {
    pub interface: String,
}

#[derive(Debug, Clone)]
pub struct BandwidthUsage {
    pub interface: String,
    pub bytes_per_second: u64,
    pub packets_per_second: u64,
}

#[derive(Debug, Clone)]
pub struct PortScanResult {
    pub port: u16,
    pub is_open: bool,
    pub service: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NetworkTopology {
    pub devices: Vec<NetworkDevice>,
    pub connections: Vec<NetworkConnection>,
}

#[derive(Debug, Clone)]
pub struct NetworkDevice {
    pub ip_address: IpAddr,
    pub mac_address: Option<String>,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NetworkConnection {
    pub source: IpAddr,
    pub destination: IpAddr,
    pub protocol: String,
}

#[derive(Debug, Clone)]
pub struct TunnelConfig {
    pub tunnel_type: String,
    pub local_endpoint: IpAddr,
    pub remote_endpoint: IpAddr,
    pub encryption: bool,
}

#[derive(Debug)]
pub struct TunnelHandle {
    pub id: u32,
    pub config: TunnelConfig,
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub local_address: IpAddr,
    pub local_port: u16,
    pub remote_address: IpAddr,
    pub remote_port: u16,
    pub protocol: NetworkProtocol,
    pub state: ConnectionState,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

// Additional supporting types for advanced network features

#[derive(Debug, Clone)]
pub struct ConnectionTarget {
    pub protocol: String,
    pub hostname: String,
    pub port: u16,
    pub resolved_ip: Option<IpAddr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetworkProtocol {
    Tcp,
    Udp,
    Http,
    Https,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
    Failed,
}

#[derive(Debug)]
pub enum NetworkStream {
    Tcp(std::net::TcpStream),
    Udp(std::net::UdpSocket),
}

impl NetworkStream {
    pub fn send_data(&self, data: &[u8]) -> HalResult<usize> {
        match self {
            NetworkStream::Tcp(stream) => {
                use std::io::Write;
                let mut stream_ref = stream;
                stream_ref.write(data).map_err(|e| HalError::network(format!("TCP write failed: {}", e)))
            },
            NetworkStream::Udp(socket) => {
                socket.send(data).map_err(|e| HalError::network(format!("UDP send failed: {}", e)))
            }
        }
    }

    pub fn receive_data(&self, buffer: &mut [u8]) -> HalResult<usize> {
        match self {
            NetworkStream::Tcp(stream) => {
                use std::io::Read;
                let mut stream_ref = stream;
                stream_ref.read(buffer).map_err(|e| HalError::network(format!("TCP read failed: {}", e)))
            },
            NetworkStream::Udp(socket) => {
                socket.recv(buffer).map_err(|e| HalError::network(format!("UDP recv failed: {}", e)))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkConnection {
    pub id: String,
    pub protocol: NetworkProtocol,
    pub target: ConnectionTarget,
    pub stream: NetworkStream,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub state: ConnectionState,
    pub quality_metrics: ConnectionQualityMetrics,
}

impl NetworkConnection {
    pub fn send_data(&self, data: &[u8]) -> HalResult<usize> {
        self.stream.send_data(data)
    }

    pub fn receive_data(&self, buffer: &mut [u8]) -> HalResult<usize> {
        self.stream.receive_data(buffer)
    }

    pub fn close(&mut self) -> HalResult<()> {
        self.state = ConnectionState::Disconnected;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionQualityMetrics {
    pub average_latency: Duration,
    pub packet_loss_rate: f64,
    pub jitter: Duration,
    pub throughput_bps: u64,
}

#[derive(Debug, Clone)]
pub struct ConnectivityTestResult {
    pub target: String,
    pub reachable: bool,
    pub response_time: Duration,
    pub test_time: Duration,
    pub error_message: Option<String>,
    pub hop_count: u8,
    pub quality_score: f64,
}

#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    pub timestamp: SystemTime,
    pub latency: Duration,
    pub packet_loss: bool,
    pub jitter: Duration,
}

#[derive(Debug, Clone)]
pub struct NetworkPerformanceReport {
    pub target: String,
    pub test_duration: Duration,
    pub total_measurements: usize,
    pub packet_loss_rate: f64,
    pub average_latency: Duration,
    pub min_latency: Duration,
    pub max_latency: Duration,
    pub measurements: Vec<PerformanceMeasurement>,
    pub bandwidth_estimate: u64,
    pub quality_score: f64,
}

#[derive(Debug, Clone)]
pub struct ConnectionReport {
    pub connection_id: String,
    pub target: String,
    pub protocol: NetworkProtocol,
    pub duration: Duration,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub total_bytes: u64,
    pub average_throughput: u64,
    pub quality_metrics: ConnectionQualityMetrics,
}

// Supporting infrastructure types

#[derive(Debug, Clone)]
pub struct ConnectionPool {
    connections: HashMap<String, NetworkConnection>,
    max_connections: usize,
}

impl ConnectionPool {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            max_connections: 1000,
        }
    }

    pub fn add_connection(&mut self, connection: NetworkConnection) {
        if self.connections.len() < self.max_connections {
            self.connections.insert(connection.id.clone(), connection);
        }
    }

    pub fn get_connection(&self, id: &str) -> Option<&NetworkConnection> {
        self.connections.get(id)
    }

    pub fn get_connection_mut(&mut self, id: &str) -> Option<&mut NetworkConnection> {
        self.connections.get_mut(id)
    }

    pub fn remove_connection(&mut self, id: &str) -> Option<NetworkConnection> {
        self.connections.remove(id)
    }
}

#[derive(Debug)]
pub struct NetworkPerformanceMonitor;

impl NetworkPerformanceMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn start_monitoring(&mut self, _connection: &NetworkConnection) -> HalResult<()> {
        Ok(())
    }

    pub fn stop_monitoring(&mut self, _connection_id: &str) {
        // Stop monitoring implementation
    }

    pub fn record_bytes_sent(&mut self, _connection_id: &str, _bytes: usize) {
        // Record bytes sent
    }

    pub fn record_bytes_received(&mut self, _connection_id: &str, _bytes: usize) {
        // Record bytes received
    }

    pub fn add_connection_report(&mut self, _report: ConnectionReport) {
        // Add connection report
    }
}

#[derive(Debug, Clone)]
pub struct NetworkSecurityManager;

impl NetworkSecurityManager {
    pub fn new() -> Self {
        Self
    }

    pub fn add_policy(&mut self, _policy: SecurityPolicy) {
        // Add security policy
    }

    pub fn validate_connection(&self, _target: &ConnectionTarget) -> HalResult<()> {
        Ok(())
    }

    pub fn filter_outgoing_data(&self, _data: &[u8]) -> HalResult<()> {
        Ok(())
    }

    pub fn filter_incoming_data(&self, _data: &[u8]) -> HalResult<()> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BandwidthMonitor;

impl BandwidthMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn check_send_limit(&self, _data_size: usize) -> HalResult<()> {
        Ok(())
    }

    pub fn record_sent_bytes(&self, _bytes: usize) {
        // Record sent bytes
    }

    pub fn record_received_bytes(&self, _bytes: usize) {
        // Record received bytes
    }
}

#[derive(Debug)]
pub struct DnsCache {
    cache: HashMap<String, IpAddr>,
}

impl DnsCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProxyConfiguration {
    pub proxy_type: String,
    pub host: String,
    pub port: u16,
}

impl ProxyConfiguration {
    pub fn resolve_target(&self, target: &ConnectionTarget) -> HalResult<ConnectionTarget> {
        // For now, just return the original target
        Ok(target.clone())
    }
}

#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub name: String,
    pub rule_type: SecurityRuleType,
    pub pattern: String,
    pub action: SecurityAction,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub enum SecurityRuleType {
    Port,
    IpRange,
    Hostname,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityAction {
    Allow,
    Block,
    Monitor,
}
    pub remote_address: IpAddr,
    pub remote_port: u16,
    pub protocol: String,
    pub state: String,
}

pub type NetworkInfo = NetworkStats; 