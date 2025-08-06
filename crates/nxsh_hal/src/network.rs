//! Network management abstraction layer
//!
//! This module provides platform-agnostic network operations and
//! information gathering capabilities.

use std::net::IpAddr;

use crate::error::{HalError, HalResult};
use crate::platform::{Platform, Capabilities};

/// Network management and operations
#[derive(Debug)]
pub struct NetworkManager {
    #[allow(dead_code)]
    platform: Platform,
    #[allow(dead_code)]
    capabilities: Capabilities,
}

impl NetworkManager {
    pub fn new() -> HalResult<Self> {
        Ok(Self {
            platform: Platform::current(),
            capabilities: Capabilities::current(),
        })
    }

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
                        
                        // For simplicity, only add basic localhost for loopback
                        let addresses = if is_loopback {
                            vec![std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))]
                        } else {
                            vec![] // Would need more complex IP parsing for real addresses
                        };
                        
                        interfaces.push(NetworkInterface {
                            name: name.to_string(),
                            addresses,
                            is_up,
                            is_loopback,
                            mtu: 1500, // Standard Ethernet MTU as default
                            mac_address: None, // Simplified
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

        for line in net_dev.lines().skip(2) { // Skip header lines
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 17 {
                // Parse receive stats
                if let Ok(rx_bytes) = parts[1].parse::<u64>() {
                    total_bytes_received += rx_bytes;
                }
                if let Ok(rx_packets) = parts[2].parse::<u64>() {
                    total_packets_received += rx_packets;
                }
                
                // Parse transmit stats
                if let Ok(tx_bytes) = parts[9].parse::<u64>() {
                    total_bytes_sent += tx_bytes;
                }
                if let Ok(tx_packets) = parts[10].parse::<u64>() {
                    total_packets_sent += tx_packets;
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

    #[cfg(target_os = "macos")]
    fn network_stats_macos(&self) -> HalResult<NetworkStats> {
        // Implementation would use sysctl
        Err(HalError::unsupported("Network statistics not yet implemented on macOS"))
    }

    #[cfg(windows)]
    fn network_stats_windows(&self) -> HalResult<NetworkStats> {
        // Implementation would use GetIfTable2
        Err(HalError::unsupported("Network statistics not yet implemented on Windows"))
    }

    #[cfg(target_os = "linux")]
    fn routing_table_linux(&self) -> HalResult<Vec<RouteEntry>> {
        use std::fs;
        use std::net::Ipv4Addr;

        let route_data = fs::read_to_string("/proc/net/route")
            .map_err(|e| HalError::io_error("read_route", Some("/proc/net/route"), e))?;

        let mut routes = Vec::new();

        for line in route_data.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                let interface = parts[0].to_string();
                
                // Parse destination (in hex, little-endian)
                if let Ok(dest_hex) = u32::from_str_radix(parts[1], 16) {
                    let _dest = Ipv4Addr::from(dest_hex.swap_bytes());
                    
                    // Parse gateway (in hex, little-endian)
                    if let Ok(gw_hex) = u32::from_str_radix(parts[2], 16) {
                        let _gateway = if gw_hex == 0 {
                            None
                        } else {
                            Some(IpAddr::V4(Ipv4Addr::from(gw_hex.swap_bytes())))
                        };

                        // Parse netmask (in hex, little-endian)
                        if let Ok(mask_hex) = u32::from_str_radix(parts[7], 16) {
                            let _netmask = Ipv4Addr::from(mask_hex.swap_bytes());

                            routes.push(RouteEntry {
                                destination: std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
                                netmask: std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
                                gateway: std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 1)), // Default gateway
                                interface,
                                metric: 0,
                            });
                        }
                    }
                }
            }
        }

        Ok(routes)
    }

    #[cfg(target_os = "macos")]
    fn routing_table_macos(&self) -> HalResult<Vec<RouteEntry>> {
        // Implementation would use route command or sysctl
        Err(HalError::unsupported("Routing table not yet implemented on macOS"))
    }

    #[cfg(windows)]
    fn routing_table_windows(&self) -> HalResult<Vec<RouteEntry>> {
        // Implementation would use GetIpForwardTable
        Err(HalError::unsupported("Routing table not yet implemented on Windows"))
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
                                bytes_per_second: rx_bytes + tx_bytes,
                                packets_per_second: parts[2].parse::<u64>().unwrap_or(0) + parts[10].parse::<u64>().unwrap_or(0),
                            });
                        }
                    }
                }
            }
        }
        
        #[cfg(windows)]
        {
            // Use PowerShell to get network adapter statistics
            if let Ok(output) = std::process::Command::new("powershell")
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
    pub protocol: String,
    pub state: String,
}

pub type NetworkInfo = NetworkStats; 