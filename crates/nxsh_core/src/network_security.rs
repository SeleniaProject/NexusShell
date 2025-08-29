//! Network security and firewall management for NexusShell
//!
//! This module provides network security features including firewall rules,
//! intrusion detection, and secure network communication.

use crate::compat::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, SystemTime},
};

/// Comprehensive network monitoring and security system for NexusShell
#[derive(Debug, Clone)]
pub struct NetworkMonitor {
    connections: Arc<Mutex<HashMap<String, NetworkConnection>>>,
    security_rules: Vec<SecurityRule>,
    traffic_analyzer: TrafficAnalyzer,
    threat_detector: ThreatDetector,
    monitoring_enabled: bool,
    whitelist: Vec<String>,
    blacklist: Vec<String>,
    security_policies: Arc<RwLock<HashMap<String, SecurityPolicy>>>,
}

impl NetworkMonitor {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            security_rules: Self::default_security_rules(),
            traffic_analyzer: TrafficAnalyzer::new(),
            threat_detector: ThreatDetector::new(),
            monitoring_enabled: true,
            whitelist: vec![
                "127.0.0.1".to_string(),
                "::1".to_string(),
                "localhost".to_string(),
            ],
            blacklist: vec![
                "0.0.0.0".to_string(),
                "169.254.0.0/16".to_string(), // Link-local
            ],
            security_policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    // Default implementation declared at module scope below
    /// Start network monitoring
    pub fn start_monitoring(&mut self) -> Result<()> {
        if self.monitoring_enabled {
            return Ok(()); // Already started
        }

        self.monitoring_enabled = true;

        // Start connection monitoring thread
        let connections = Arc::clone(&self.connections);
        std::thread::spawn(move || {
            let mut monitor = ConnectionMonitor::new(connections);
            if let Err(e) = monitor.run() {
                eprintln!("Network monitoring error: {e}");
            }
        });

        println!("Network monitoring started");
        Ok(())
    }

    /// Stop network monitoring
    pub fn stop_monitoring(&mut self) {
        self.monitoring_enabled = false;
        println!("Network monitoring stopped");
    }

    /// Validate network access
    pub fn validate_access(&self, host: &str, port: u16) -> Result<AccessDecision> {
        // Check blacklist first
        for blocked in &self.blacklist {
            if self.matches_pattern(host, blocked) {
                return Ok(AccessDecision::Deny(format!("Host {host} is blacklisted")));
            }
        }

        // Check whitelist
        for allowed in &self.whitelist {
            if self.matches_pattern(host, allowed) {
                return Ok(AccessDecision::Allow);
            }
        }

        // Apply security rules
        for rule in &self.security_rules {
            if rule.matches(host, port) {
                match rule.action {
                    RuleAction::Allow => return Ok(AccessDecision::Allow),
                    RuleAction::Deny => return Ok(AccessDecision::Deny(rule.reason.clone())),
                    RuleAction::Log => {
                        self.log_access_attempt(host, port, &rule.reason);
                    }
                    RuleAction::Monitor => {
                        self.add_monitoring_target(host, port);
                    }
                }
            }
        }

        // Default to allow with monitoring
        self.add_monitoring_target(host, port);
        Ok(AccessDecision::Allow)
    }

    /// Analyze network traffic for threats
    pub fn analyze_traffic(
        &mut self,
        data: &[u8],
        source: &str,
        destination: &str,
    ) -> Result<ThreatAnalysis> {
        let analysis = self
            .threat_detector
            .analyze_packet(data, source, destination)?;

        if analysis.threat_level > ThreatLevel::Low {
            self.handle_threat(&analysis)?;
        }

        self.traffic_analyzer
            .record_traffic(data.len(), source, destination);
        Ok(analysis)
    }

    /// Get network statistics
    pub fn get_network_stats(&self) -> NetworkStatistics {
        let connections = self.connections.lock().unwrap();
        let active_connections = connections.len();

        NetworkStatistics {
            active_connections,
            total_bytes_sent: self.traffic_analyzer.total_bytes_sent(),
            total_bytes_received: self.traffic_analyzer.total_bytes_received(),
            threats_detected: self.threat_detector.threat_count(),
            monitoring_uptime: self.get_uptime(),
            security_events: self.get_security_events(),
        }
    }

    /// Add a host to whitelist
    pub fn whitelist_host(&mut self, host: String) {
        if !self.whitelist.contains(&host) {
            self.whitelist.push(host.clone());
            println!("Added {host} to whitelist");
        }
    }

    /// Add a host to blacklist
    pub fn blacklist_host(&mut self, host: String) {
        if !self.blacklist.contains(&host) {
            self.blacklist.push(host.clone());
            println!("Added {host} to blacklist");
        }
    }

    /// Configure security rule
    pub fn add_security_rule(&mut self, rule: SecurityRule) {
        self.security_rules.push(rule);
    }

    /// Monitor specific network endpoint
    pub fn monitor_endpoint(&mut self, endpoint: &str) -> crate::compat::Result<EndpointMonitor> {
        let parts: Vec<&str> = endpoint.split(':').collect();
        if parts.len() != 2 {
            return Err(crate::anyhow!("Invalid endpoint format. Use host:port"));
        }

        let host = parts[0].to_string();
        let port: u16 = parts[1].parse().context("Invalid port number")?;

        Ok(EndpointMonitor::new(host, port))
    }

    /// Get connection details
    pub fn get_connection_info(&self, connection_id: &str) -> Option<NetworkConnection> {
        let connections = self.connections.lock().unwrap();
        connections.get(connection_id).cloned()
    }

    /// Perform network security audit
    pub fn security_audit(&self) -> SecurityAuditReport {
        let mut report = SecurityAuditReport::new();

        // Check for suspicious patterns
        let connections = self.connections.lock().unwrap();
        for (id, conn) in connections.iter() {
            if self.is_suspicious_connection(conn) {
                report.add_finding(SecurityFinding {
                    severity: FindingSeverity::Medium,
                    description: format!("Suspicious connection detected: {id}"),
                    recommendation: "Review connection and consider blocking".to_string(),
                    connection_id: Some(id.clone()),
                });
            }
        }

        // Check security rule coverage
        if self.security_rules.len() < 5 {
            report.add_finding(SecurityFinding {
                severity: FindingSeverity::Low,
                description: "Limited security rules configured".to_string(),
                recommendation: "Consider adding more specific security rules".to_string(),
                connection_id: None,
            });
        }

        // Check for unencrypted connections
        for (id, conn) in connections.iter() {
            if !conn.encrypted && conn.port != 80 && conn.port != 443 {
                report.add_finding(SecurityFinding {
                    severity: FindingSeverity::High,
                    description: format!("Unencrypted connection on port {0}", conn.port),
                    recommendation: "Use encrypted connections when possible".to_string(),
                    connection_id: Some(id.clone()),
                });
            }
        }

        report
    }

    // Private helper methods

    fn default_security_rules() -> Vec<SecurityRule> {
        vec![
            SecurityRule {
                name: "Block suspicious ports".to_string(),
                host_pattern: "*".to_string(),
                port_range: (1, 1023),
                action: RuleAction::Monitor,
                reason: "System port access".to_string(),
            },
            SecurityRule {
                name: "Allow HTTP/HTTPS".to_string(),
                host_pattern: "*".to_string(),
                port_range: (80, 80),
                action: RuleAction::Allow,
                reason: "Standard web traffic".to_string(),
            },
            SecurityRule {
                name: "Allow HTTPS".to_string(),
                host_pattern: "*".to_string(),
                port_range: (443, 443),
                action: RuleAction::Allow,
                reason: "Secure web traffic".to_string(),
            },
            SecurityRule {
                name: "Block known malicious ranges".to_string(),
                host_pattern: "192.168.0.1/8".to_string(),
                port_range: (1, 65535),
                action: RuleAction::Deny,
                reason: "Private network access blocked".to_string(),
            },
        ]
    }

    fn matches_pattern(&self, host: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.contains('/') {
            // CIDR notation - simplified check
            return host.starts_with(&pattern[..pattern.find('/').unwrap_or(0)]);
        }

        // Exact match or wildcard
        pattern == host || pattern.replace('*', "") == host.replace(&pattern.replace('*', ""), "")
    }

    fn log_access_attempt(&self, host: &str, port: u16, reason: &str) {
        println!("[NETWORK] Access to {host}:{port} - {reason}");
    }

    fn add_monitoring_target(&self, host: &str, port: u16) {
        let connection_id = format!("{host}:{port}");
        let mut connections = self.connections.lock().unwrap();

        connections
            .entry(connection_id.clone())
            .or_insert_with(|| NetworkConnection {
                id: connection_id,
                host: host.to_string(),
                port,
                protocol: Protocol::TCP,
                state: ConnectionState::Monitoring,
                bytes_sent: 0,
                bytes_received: 0,
                start_time: SystemTime::now(),
                last_activity: SystemTime::now(),
                encrypted: port == 443 || port == 22,
            });
    }

    fn handle_threat(&self, analysis: &ThreatAnalysis) -> Result<()> {
        match analysis.threat_level {
            ThreatLevel::Critical => {
                eprintln!("CRITICAL THREAT DETECTED: {}", analysis.description);
                // Could implement automatic blocking here
            }
            ThreatLevel::High => {
                eprintln!("HIGH THREAT DETECTED: {}", analysis.description);
            }
            ThreatLevel::Medium => {
                println!("Medium threat detected: {}", analysis.description);
            }
            ThreatLevel::Low => {
                // Just log
            }
        }
        Ok(())
    }

    fn get_uptime(&self) -> Duration {
        // Placeholder - would track actual start time
        Duration::from_secs(3600)
    }

    fn get_security_events(&self) -> usize {
        // Placeholder - would track actual events
        self.threat_detector.threat_count()
    }

    fn is_suspicious_connection(&self, conn: &NetworkConnection) -> bool {
        // Check for suspicious patterns
        let duration = SystemTime::now()
            .duration_since(conn.start_time)
            .unwrap_or_default();

        // Long-running connections to unusual ports
        if duration > Duration::from_secs(3600) && conn.port > 10000 {
            return true;
        }

        // High traffic volume
        if conn.bytes_sent + conn.bytes_received > 100_000_000 {
            // 100MB
            return true;
        }

        false
    }

    /// Legacy NetworkSecurityManager compatibility
    pub fn add_policy(&self, policy: SecurityPolicy) -> Result<()> {
        let mut policies = self.security_policies.write().unwrap();
        policies.insert(policy.name.clone(), policy);
        Ok(())
    }
}

/// Traffic analysis system
#[derive(Debug, Clone)]
pub struct TrafficAnalyzer {
    total_bytes_sent: Arc<Mutex<u64>>,
    total_bytes_received: Arc<Mutex<u64>>,
    traffic_history: Arc<Mutex<Vec<TrafficSample>>>,
}

impl TrafficAnalyzer {
    pub fn new() -> Self {
        Self {
            total_bytes_sent: Arc::new(Mutex::new(0)),
            total_bytes_received: Arc::new(Mutex::new(0)),
            traffic_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Default implementation declared at module scope below
    pub fn record_traffic(&self, bytes: usize, source: &str, destination: &str) {
        let mut sent = self.total_bytes_sent.lock().unwrap();
        let mut received = self.total_bytes_received.lock().unwrap();

        if source == "localhost" {
            *sent += bytes as u64;
        } else {
            *received += bytes as u64;
        }

        let mut history = self.traffic_history.lock().unwrap();
        history.push(TrafficSample {
            timestamp: SystemTime::now(),
            bytes,
            source: source.to_string(),
            destination: destination.to_string(),
        });

        // Keep only recent samples
        if history.len() > 1000 {
            history.drain(0..500);
        }
    }

    pub fn total_bytes_sent(&self) -> u64 {
        *self.total_bytes_sent.lock().unwrap()
    }

    pub fn total_bytes_received(&self) -> u64 {
        *self.total_bytes_received.lock().unwrap()
    }
}

/// Threat detection system
#[derive(Debug, Clone)]
pub struct ThreatDetector {
    threat_signatures: Vec<ThreatSignature>,
    threat_count: Arc<Mutex<usize>>,
}

impl ThreatDetector {
    pub fn new() -> Self {
        Self {
            threat_signatures: Self::load_signatures(),
            threat_count: Arc::new(Mutex::new(0)),
        }
    }

    // Default implementation declared at module scope below
    pub fn analyze_packet(
        &self,
        data: &[u8],
        source: &str,
        destination: &str,
    ) -> Result<ThreatAnalysis> {
        let mut threat_level = ThreatLevel::Low;
        let mut description = "Normal traffic".to_string();

        // Check for known threat signatures
        for signature in &self.threat_signatures {
            if signature.matches(data) {
                threat_level = signature.threat_level.clone();
                description = signature.description.clone();

                let mut count = self.threat_count.lock().unwrap();
                *count += 1;

                break;
            }
        }

        // Behavioral analysis: never downgrade existing severity.
        // If packet is unusually large, ensure severity is at least Medium.
        if data.len() > 64_000 {
            if threat_level < ThreatLevel::Medium {
                threat_level = ThreatLevel::Medium;
                description = "Large packet detected".to_string();
            } else {
                // Keep existing description indicating the more severe signature-based detection.
                // Optionally we could append a note, but keep output stable for now.
            }
        }

        Ok(ThreatAnalysis {
            threat_level,
            description,
            source: source.to_string(),
            destination: destination.to_string(),
            timestamp: SystemTime::now(),
        })
    }

    pub fn threat_count(&self) -> usize {
        *self.threat_count.lock().unwrap()
    }

    fn load_signatures() -> Vec<ThreatSignature> {
        vec![
            ThreatSignature {
                name: "SQL Injection - OR".to_string(),
                pattern: b"' OR 1=1".to_vec(),
                threat_level: ThreatLevel::High,
                description: "Potential SQL injection attempt".to_string(),
            },
            ThreatSignature {
                name: "SQL Injection - DROP".to_string(),
                pattern: b"DROP TABLE".to_vec(),
                threat_level: ThreatLevel::Critical,
                description: "SQL injection with DROP TABLE attempt".to_string(),
            },
            ThreatSignature {
                name: "SQL Injection - Comment".to_string(),
                pattern: b"; --".to_vec(),
                threat_level: ThreatLevel::High,
                description: "SQL injection with comment bypass".to_string(),
            },
            ThreatSignature {
                name: "XSS Attempt".to_string(),
                pattern: b"<script>".to_vec(),
                threat_level: ThreatLevel::Medium,
                description: "Cross-site scripting attempt".to_string(),
            },
            ThreatSignature {
                name: "Command Injection".to_string(),
                pattern: b"; rm -rf /".to_vec(),
                threat_level: ThreatLevel::Critical,
                description: "Command injection attempt".to_string(),
            },
        ]
    }
}

/// Connection monitoring system
#[derive(Debug)]
pub struct ConnectionMonitor {
    connections: Arc<Mutex<HashMap<String, NetworkConnection>>>,
}

impl ConnectionMonitor {
    pub fn new(connections: Arc<Mutex<HashMap<String, NetworkConnection>>>) -> Self {
        Self { connections }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            std::thread::sleep(Duration::from_secs(1));

            let mut connections = self.connections.lock().unwrap();
            let current_time = SystemTime::now();

            // Update connection states and clean up old connections
            let mut to_remove = Vec::new();

            for (id, conn) in connections.iter_mut() {
                let idle_time = current_time
                    .duration_since(conn.last_activity)
                    .unwrap_or_default();

                if idle_time > Duration::from_secs(300) {
                    // 5 minutes idle
                    to_remove.push(id.clone());
                } else if conn.state == ConnectionState::Monitoring {
                    // Update connection state based on activity
                    conn.state = ConnectionState::Active;
                }
            }

            for id in to_remove {
                connections.remove(&id);
            }
        }
    }
}

// Supporting types and enums

#[derive(Debug, Clone)]
pub struct NetworkConnection {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub protocol: Protocol,
    pub state: ConnectionState,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub start_time: SystemTime,
    pub last_activity: SystemTime,
    pub encrypted: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    HTTP,
    HTTPS,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Active,
    Monitoring,
    Idle,
    Closed,
}

#[derive(Debug, Clone)]
pub struct SecurityRule {
    pub name: String,
    pub host_pattern: String,
    pub port_range: (u16, u16),
    pub action: RuleAction,
    pub reason: String,
}

impl SecurityRule {
    pub fn matches(&self, host: &str, port: u16) -> bool {
        let host_matches = self.matches_host_pattern(host);
        let port_matches = port >= self.port_range.0 && port <= self.port_range.1;

        host_matches && port_matches
    }

    fn matches_host_pattern(&self, host: &str) -> bool {
        if self.host_pattern == "*" {
            return true;
        }

        if self.host_pattern == host {
            return true;
        }

        // Handle wildcard patterns like "*.example.com"
        if self.host_pattern.starts_with("*.") {
            let suffix = &self.host_pattern[2..]; // Remove "*."
            return host.ends_with(suffix) && (host.len() > suffix.len());
        }

        // Handle patterns like "example.*"
        if self.host_pattern.ends_with(".*") {
            let prefix = &self.host_pattern[..self.host_pattern.len() - 2]; // Remove ".*"
            return host.starts_with(prefix);
        }

        // Fallback to simple contains check
        host.contains(&self.host_pattern)
    }
}

#[derive(Debug, Clone)]
pub enum RuleAction {
    Allow,
    Deny,
    Log,
    Monitor,
}

#[derive(Debug, Clone)]
pub enum AccessDecision {
    Allow,
    Deny(String),
}

#[derive(Debug, Clone)]
pub struct ThreatAnalysis {
    pub threat_level: ThreatLevel,
    pub description: String,
    pub source: String,
    pub destination: String,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ThreatSignature {
    pub name: String,
    pub pattern: Vec<u8>,
    pub threat_level: ThreatLevel,
    pub description: String,
}

impl ThreatSignature {
    pub fn matches(&self, data: &[u8]) -> bool {
        // Convert both data and pattern to lowercase for case-insensitive matching
        let data_lower = data.to_ascii_lowercase();
        let pattern_lower = self.pattern.to_ascii_lowercase();

        data_lower
            .windows(pattern_lower.len())
            .any(|window| window == pattern_lower)
    }
}

#[derive(Debug, Clone)]
pub struct NetworkStatistics {
    pub active_connections: usize,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub threats_detected: usize,
    pub monitoring_uptime: Duration,
    pub security_events: usize,
}

#[derive(Debug, Clone)]
pub struct TrafficSample {
    pub timestamp: SystemTime,
    pub bytes: usize,
    pub source: String,
    pub destination: String,
}

#[derive(Debug, Clone)]
pub struct EndpointMonitor {
    pub host: String,
    pub port: u16,
    pub start_time: SystemTime,
}

impl EndpointMonitor {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            start_time: SystemTime::now(),
        }
    }

    pub fn check_connectivity(&self) -> Result<bool> {
        // Simplified connectivity check - would use actual network testing
        use std::net::TcpStream;
        use std::time::Duration;

        match TcpStream::connect_timeout(
            &format!("{}:{}", self.host, self.port).parse()?,
            Duration::from_secs(3),
        ) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityAuditReport {
    pub findings: Vec<SecurityFinding>,
    pub audit_time: SystemTime,
}

impl SecurityAuditReport {
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            audit_time: SystemTime::now(),
        }
    }

    // Default implementation declared at module scope below
    pub fn add_finding(&mut self, finding: SecurityFinding) {
        self.findings.push(finding);
    }

    pub fn severity_summary(&self) -> HashMap<FindingSeverity, usize> {
        let mut summary = HashMap::new();
        for finding in &self.findings {
            *summary.entry(finding.severity.clone()).or_insert(0) += 1;
        }
        summary
    }
}

#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub severity: FindingSeverity,
    pub description: String,
    pub recommendation: String,
    pub connection_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FindingSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Security policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub name: String,
    pub allow_http: bool,
    pub require_auth: bool,
    pub timeout: std::time::Duration,
}

/// Legacy NetworkSecurityManager for compatibility
pub struct NetworkSecurityManager {
    monitor: NetworkMonitor,
}

impl NetworkSecurityManager {
    /// Create a new network security manager
    pub fn new() -> crate::compat::Result<Self> {
        Ok(Self {
            monitor: NetworkMonitor::new(),
        })
    }

    /// Add a security policy
    pub fn add_policy(&self, policy: SecurityPolicy) -> crate::compat::Result<()> {
        self.monitor.add_policy(policy)
    }
}

// Default implementations (module scope)
impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TrafficAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ThreatDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SecurityAuditReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_monitor_creation() {
        let monitor = NetworkMonitor::new();
        assert!(monitor.monitoring_enabled);
        assert!(!monitor.whitelist.is_empty());
        assert!(!monitor.blacklist.is_empty());
    }

    #[test]
    fn test_access_validation() {
        let monitor = NetworkMonitor::new();

        // Test whitelist
        let result = monitor.validate_access("127.0.0.1", 8080).unwrap();
        assert!(matches!(result, AccessDecision::Allow));

        // Test blacklist
        let result = monitor.validate_access("0.0.0.0", 8080).unwrap();
        assert!(matches!(result, AccessDecision::Deny(_)));
    }

    #[test]
    fn test_threat_detection() {
        let detector = ThreatDetector::new();

        let malicious_data = b"'; DROP TABLE users; --";
        let result = detector
            .analyze_packet(malicious_data, "192.168.1.1", "localhost")
            .unwrap();

        assert_ne!(result.threat_level, ThreatLevel::Low);
    }

    #[test]
    fn test_threat_detection_large_packet_priority() {
        // Ensure that signature-based Critical detection is not downgraded
        // by the large-packet behavioral rule.
        let detector = ThreatDetector::new();

        // Build a payload containing a Critical signature and make it large.
        let mut data = Vec::new();
        let signature = b"DROP TABLE"; // Critical per load_signatures
        for _ in 0..7000 {
            // ~77KB when including delimiter
            data.extend_from_slice(signature);
            data.push(b' ');
        }

        let result = detector
            .analyze_packet(&data, "10.0.0.1", "localhost")
            .expect("analysis should succeed");

        assert_eq!(
            result.threat_level,
            ThreatLevel::Critical,
            "Severity must not be downgraded for large malicious packets"
        );
    }

    #[test]
    fn test_security_rule_matching() {
        let rule = SecurityRule {
            name: "Test Rule".to_string(),
            host_pattern: "*.example.com".to_string(),
            port_range: (80, 443),
            action: RuleAction::Allow,
            reason: "Test".to_string(),
        };

        assert!(rule.matches("api.example.com", 80));
        assert!(rule.matches("www.example.com", 443));
        assert!(!rule.matches("example.org", 80));
        assert!(!rule.matches("api.example.com", 8080));
    }

    #[test]
    fn test_security_audit() {
        let monitor = NetworkMonitor::new();
        let report = monitor.security_audit();

        assert!(!report.findings.is_empty());
    }

    #[test]
    fn test_legacy_compatibility() {
        let manager = NetworkSecurityManager::new().unwrap();
        let policy = SecurityPolicy {
            name: "test".to_string(),
            allow_http: true,
            require_auth: false,
            timeout: Duration::from_secs(30),
        };

        assert!(manager.add_policy(policy).is_ok());
    }
}
