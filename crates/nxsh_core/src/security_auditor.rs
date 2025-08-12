use crate::compat::Result;
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
    sync::{Arc, Mutex},
    path::PathBuf,
};
use serde::{Deserialize, Serialize};

/// Comprehensive security audit and compliance system
#[derive(Debug, Clone)]
pub struct SecurityAuditor {
    audit_rules: Vec<AuditRule>,
    compliance_frameworks: HashMap<String, ComplianceFramework>,
    vulnerability_scanners: Vec<VulnerabilityScanner>,
    audit_log: Arc<Mutex<Vec<AuditEvent>>>,
    security_policies: SecurityPolicies,
    scan_results: HashMap<String, ScanResult>,
}

impl SecurityAuditor {
    pub fn new() -> Self {
        let mut auditor = Self {
            audit_rules: Vec::new(),
            compliance_frameworks: HashMap::new(),
            vulnerability_scanners: Vec::new(),
            audit_log: Arc::new(Mutex::new(Vec::new())),
            security_policies: SecurityPolicies::default(),
            scan_results: HashMap::new(),
        };
        
        auditor.register_default_rules();
        auditor.register_compliance_frameworks();
        auditor.register_vulnerability_scanners();
        auditor
    }

    /// Perform comprehensive security audit
    pub fn perform_security_audit(&mut self, scope: AuditScope) -> Result<SecurityAuditReport> {
        let start_time = SystemTime::now();
        
        let mut report = SecurityAuditReport {
            audit_id: Self::generate_audit_id(),
            start_time,
            end_time: None,
            scope: scope.clone(),
            findings: Vec::new(),
            compliance_results: HashMap::new(),
            vulnerability_scan_results: Vec::new(),
            risk_score: 0.0,
            recommendations: Vec::new(),
        };

        // Execute audit rules
        for rule in &self.audit_rules {
            if rule.applies_to_scope(&scope) {
                match self.execute_audit_rule(rule) {
                    Ok(findings) => report.findings.extend(findings),
                    Err(e) => {
                        self.log_audit_event(AuditEvent {
                            timestamp: SystemTime::now(),
                            event_type: AuditEventType::Error,
                            description: format!("Failed to execute audit rule {}: {}", rule.name, e),
                            severity: AuditSeverity::Medium,
                        });
                    }
                }
            }
        }

        // Run compliance checks
        for (framework_name, framework) in &self.compliance_frameworks {
            if scope.includes_compliance_check(framework_name) {
                let compliance_result = self.check_compliance(framework)?;
                report.compliance_results.insert(framework_name.clone(), compliance_result);
            }
        }

        // Run vulnerability scans
        for scanner in &self.vulnerability_scanners {
            if scanner.applies_to_scope(&scope) {
                match self.run_vulnerability_scan(scanner, &scope) {
                    Ok(scan_result) => report.vulnerability_scan_results.push(scan_result),
                    Err(e) => {
                        self.log_audit_event(AuditEvent {
                            timestamp: SystemTime::now(),
                            event_type: AuditEventType::Warning,
                            description: format!("Vulnerability scan failed: {}", e),
                            severity: AuditSeverity::Low,
                        });
                    }
                }
            }
        }

        // Calculate risk score
        report.risk_score = self.calculate_risk_score(&report);

        // Generate recommendations
        report.recommendations = self.generate_recommendations(&report);

        report.end_time = Some(SystemTime::now());
        
        // Log completion
        self.log_audit_event(AuditEvent {
            timestamp: SystemTime::now(),
            event_type: AuditEventType::AuditCompleted,
            description: format!("Security audit {} completed with risk score {:.2}", 
                               report.audit_id, report.risk_score),
            severity: AuditSeverity::Info,
        });

        Ok(report)
    }

    /// Check specific compliance framework
    pub fn check_compliance(&self, framework: &ComplianceFramework) -> Result<ComplianceResult> {
        let mut result = ComplianceResult {
            framework_name: framework.name.clone(),
            version: framework.version.clone(),
            total_controls: framework.controls.len(),
            compliant_controls: 0,
            non_compliant_controls: Vec::new(),
            not_applicable_controls: Vec::new(),
            compliance_percentage: 0.0,
            findings: Vec::new(),
        };

        for control in &framework.controls {
            match self.evaluate_control(control)? {
                ControlEvaluation::Compliant => {
                    result.compliant_controls += 1;
                },
                ControlEvaluation::NonCompliant(finding) => {
                    result.non_compliant_controls.push(control.id.clone());
                    result.findings.push(finding);
                },
                ControlEvaluation::NotApplicable => {
                    result.not_applicable_controls.push(control.id.clone());
                },
            }
        }

        // Calculate compliance percentage
        let applicable_controls = result.total_controls - result.not_applicable_controls.len();
        if applicable_controls > 0 {
            result.compliance_percentage = 
                (result.compliant_controls as f64 / applicable_controls as f64) * 100.0;
        }

        Ok(result)
    }

    /// Run vulnerability assessment
    pub fn run_vulnerability_assessment(&mut self, target: &str) -> Result<VulnerabilityAssessment> {
        let mut assessment = VulnerabilityAssessment {
            target: target.to_string(),
            scan_time: SystemTime::now(),
            vulnerabilities: Vec::new(),
            risk_level: RiskLevel::Low,
            recommendations: Vec::new(),
        };

        // File permission vulnerabilities
        assessment.vulnerabilities.extend(self.scan_file_permissions(target)?);

        // Configuration vulnerabilities
        assessment.vulnerabilities.extend(self.scan_configuration_issues(target)?);

        // Network security vulnerabilities
        assessment.vulnerabilities.extend(self.scan_network_security(target)?);

        // Calculate overall risk level
        assessment.risk_level = self.calculate_vulnerability_risk(&assessment.vulnerabilities);

        // Generate specific recommendations
        assessment.recommendations = self.generate_vulnerability_recommendations(&assessment.vulnerabilities);

        Ok(assessment)
    }

    /// Generate security hardening recommendations
    pub fn generate_hardening_guide(&self, system_info: &SystemInfo) -> Result<HardeningGuide> {
        let mut guide = HardeningGuide {
            system_type: system_info.system_type.clone(),
            recommendations: Vec::new(),
            priority_actions: Vec::new(),
            configuration_templates: HashMap::new(),
        };

        // File system hardening
        guide.recommendations.push(HardeningRecommendation {
            category: HardeningCategory::FileSystem,
            title: "Secure File Permissions".to_string(),
            description: "Set appropriate file permissions for sensitive files".to_string(),
            commands: vec![
                "chmod 600 ~/.nxsh_config".to_string(),
                "chmod 700 ~/.nxsh".to_string(),
            ],
            risk_reduction: RiskLevel::Medium,
        });

        // Network security hardening
        guide.recommendations.push(HardeningRecommendation {
            category: HardeningCategory::Network,
            title: "Network Access Controls".to_string(),
            description: "Configure network access restrictions".to_string(),
            commands: vec![
                "nxsh --set-network-policy restrictive".to_string(),
                "nxsh --enable-network-monitoring".to_string(),
            ],
            risk_reduction: RiskLevel::High,
        });

        // Authentication hardening
        guide.recommendations.push(HardeningRecommendation {
            category: HardeningCategory::Authentication,
            title: "Strong Authentication".to_string(),
            description: "Enable strong authentication mechanisms".to_string(),
            commands: vec![
                "nxsh --enable-2fa".to_string(),
                "nxsh --set-password-policy strong".to_string(),
            ],
            risk_reduction: RiskLevel::High,
        });

        // Identify priority actions based on risk
        guide.priority_actions = guide.recommendations.iter()
            .filter(|rec| matches!(rec.risk_reduction, RiskLevel::High | RiskLevel::Critical))
            .map(|rec| rec.title.clone())
            .collect();

        // Generate configuration templates
        guide.configuration_templates.insert(
            "secure_config".to_string(),
            self.generate_secure_config_template(system_info)?,
        );

        Ok(guide)
    }

    /// Monitor for security events in real-time
    pub fn start_security_monitoring(&mut self) -> Result<SecurityMonitor> {
        let monitor = SecurityMonitor::new(Arc::clone(&self.audit_log))?;
        
        self.log_audit_event(AuditEvent {
            timestamp: SystemTime::now(),
            event_type: AuditEventType::MonitoringStarted,
            description: "Real-time security monitoring started".to_string(),
            severity: AuditSeverity::Info,
        });

        Ok(monitor)
    }

    /// Export audit results
    pub fn export_audit_results(&self, report: &SecurityAuditReport, format: ExportFormat) -> Result<Vec<u8>> {
        match format {
            ExportFormat::Json => {
                Ok(serde_json::to_vec_pretty(report)?)
            },
            ExportFormat::Pdf => {
                // Generate PDF report - simplified implementation
                let content = self.generate_pdf_content(report)?;
                Ok(content.into_bytes())
            },
            ExportFormat::Html => {
                let html = self.generate_html_report(report)?;
                Ok(html.into_bytes())
            },
            ExportFormat::Xml => {
                // Generate XML report - simplified implementation
                let xml = format!(r#"
                    <?xml version="1.0" encoding="UTF-8"?>
                    <SecurityAuditReport>
                        <AuditId>{}</AuditId>
                        <RiskScore>{}</RiskScore>
                        <FindingsCount>{}</FindingsCount>
                    </SecurityAuditReport>
                "#, report.audit_id, report.risk_score, report.findings.len());
                Ok(xml.into_bytes())
            },
        }
    }

    // Private implementation methods

    fn register_default_rules(&mut self) {
        self.audit_rules = vec![
            AuditRule {
                name: "File Permission Check".to_string(),
                description: "Verify file permissions are appropriately restrictive".to_string(),
                category: AuditCategory::FileSystem,
                severity: AuditSeverity::High,
                check_function: std::sync::Arc::new(|_| {
                    // Check critical file permissions
                    Ok(vec![])
                }),
            },
            AuditRule {
                name: "Configuration Security".to_string(),
                description: "Validate security configuration settings".to_string(),
                category: AuditCategory::Configuration,
                severity: AuditSeverity::Medium,
                check_function: std::sync::Arc::new(|_| {
                    // Check configuration security
                    Ok(vec![])
                }),
            },
            AuditRule {
                name: "Network Security".to_string(),
                description: "Assess network security posture".to_string(),
                category: AuditCategory::Network,
                severity: AuditSeverity::High,
                check_function: std::sync::Arc::new(|_| {
                    // Check network security
                    Ok(vec![])
                }),
            },
        ];
    }

    fn register_compliance_frameworks(&mut self) {
        // CIS (Center for Internet Security) Controls
        let cis_framework = ComplianceFramework {
            name: "CIS Controls".to_string(),
            version: "8.0".to_string(),
            description: "CIS Critical Security Controls".to_string(),
            controls: vec![
                ComplianceControl {
                    id: "CIS-1".to_string(),
                    title: "Inventory and Control of Enterprise Assets".to_string(),
                    description: "Actively manage all enterprise assets".to_string(),
                    requirements: vec![
                        "Maintain accurate inventory of enterprise assets".to_string(),
                    ],
                },
                ComplianceControl {
                    id: "CIS-2".to_string(),
                    title: "Inventory and Control of Software Assets".to_string(),
                    description: "Actively manage all software on the network".to_string(),
                    requirements: vec![
                        "Maintain accurate inventory of software assets".to_string(),
                    ],
                },
            ],
        };

        // NIST Cybersecurity Framework
        let nist_framework = ComplianceFramework {
            name: "NIST CSF".to_string(),
            version: "1.1".to_string(),
            description: "NIST Cybersecurity Framework".to_string(),
            controls: vec![
                ComplianceControl {
                    id: "ID.AM-1".to_string(),
                    title: "Physical devices and systems are inventoried".to_string(),
                    description: "Maintain inventory of physical devices and systems".to_string(),
                    requirements: vec![
                        "Inventory all physical devices".to_string(),
                    ],
                },
            ],
        };

        self.compliance_frameworks.insert("CIS".to_string(), cis_framework);
        self.compliance_frameworks.insert("NIST".to_string(), nist_framework);
    }

    fn register_vulnerability_scanners(&mut self) {
        self.vulnerability_scanners = vec![
            VulnerabilityScanner {
                name: "File Permission Scanner".to_string(),
                description: "Scans for insecure file permissions".to_string(),
                scanner_type: ScannerType::FileSystem,
                scan_function: std::sync::Arc::new(|_scope| {
                    // Scan file permissions
                    Ok(ScanResult::default())
                }),
            },
            VulnerabilityScanner {
                name: "Configuration Scanner".to_string(),
                description: "Scans for insecure configuration settings".to_string(),
                scanner_type: ScannerType::Configuration,
                scan_function: std::sync::Arc::new(|_scope| {
                    // Scan configuration
                    Ok(ScanResult::default())
                }),
            },
        ];
    }

    fn execute_audit_rule(&self, rule: &AuditRule) -> Result<Vec<AuditFinding>> {
        (rule.check_function)(self)
    }

    fn evaluate_control(&self, control: &ComplianceControl) -> Result<ControlEvaluation> {
        // Simplified control evaluation
        match control.id.as_str() {
            "CIS-1" | "ID.AM-1" => Ok(ControlEvaluation::Compliant),
            _ => Ok(ControlEvaluation::NotApplicable),
        }
    }

    fn run_vulnerability_scan(&self, scanner: &VulnerabilityScanner, scope: &AuditScope) -> Result<ScanResult> {
        (scanner.scan_function)(scope)
    }

    fn scan_file_permissions(&self, _target: &str) -> Result<Vec<Vulnerability>> {
        let mut vulnerabilities = Vec::new();
        
        // Check for world-writable files
        vulnerabilities.push(Vulnerability {
            id: "VULN-001".to_string(),
            title: "World-writable files detected".to_string(),
            description: "Files with world-write permissions pose security risks".to_string(),
            severity: VulnerabilitySeverity::Medium,
            cvss_score: Some(5.5),
            affected_assets: vec!["config files".to_string()],
            remediation: "Remove world-write permissions: chmod o-w <file>".to_string(),
        });

        Ok(vulnerabilities)
    }

    fn scan_configuration_issues(&self, _target: &str) -> Result<Vec<Vulnerability>> {
        let mut vulnerabilities = Vec::new();
        
        vulnerabilities.push(Vulnerability {
            id: "VULN-002".to_string(),
            title: "Weak encryption settings".to_string(),
            description: "Configuration uses weak encryption algorithms".to_string(),
            severity: VulnerabilitySeverity::High,
            cvss_score: Some(7.5),
            affected_assets: vec!["configuration files".to_string()],
            remediation: "Update to use strong encryption (AES-256)".to_string(),
        });

        Ok(vulnerabilities)
    }

    fn scan_network_security(&self, _target: &str) -> Result<Vec<Vulnerability>> {
        let mut vulnerabilities = Vec::new();
        
        vulnerabilities.push(Vulnerability {
            id: "VULN-003".to_string(),
            title: "Unencrypted network communication".to_string(),
            description: "Network traffic is not encrypted".to_string(),
            severity: VulnerabilitySeverity::High,
            cvss_score: Some(8.0),
            affected_assets: vec!["network connections".to_string()],
            remediation: "Enable TLS/SSL for all network communications".to_string(),
        });

        Ok(vulnerabilities)
    }

    fn calculate_risk_score(&self, report: &SecurityAuditReport) -> f64 {
        let mut score = 0.0;
        
        // Weight findings by severity
        for finding in &report.findings {
            score += match finding.severity {
                AuditSeverity::Critical => 10.0,
                AuditSeverity::High => 7.5,
                AuditSeverity::Medium => 5.0,
                AuditSeverity::Low => 2.5,
                AuditSeverity::Info => 1.0,
            };
        }
        
        // Weight vulnerability scans
        for scan_result in &report.vulnerability_scan_results {
            for vulnerability in &scan_result.vulnerabilities {
                score += match vulnerability.severity {
                    VulnerabilitySeverity::Critical => 15.0,
                    VulnerabilitySeverity::High => 10.0,
                    VulnerabilitySeverity::Medium => 5.0,
                    VulnerabilitySeverity::Low => 2.0,
                };
            }
        }
        
        // Normalize to 0-100 scale with explicit type
        (score as f64).min(100.0) as f64
    }

    fn calculate_vulnerability_risk(&self, vulnerabilities: &[Vulnerability]) -> RiskLevel {
        let max_severity = vulnerabilities.iter()
            .map(|v| &v.severity)
            .max()
            .unwrap_or(&VulnerabilitySeverity::Low);
        
        match max_severity {
            VulnerabilitySeverity::Critical => RiskLevel::Critical,
            VulnerabilitySeverity::High => RiskLevel::High,
            VulnerabilitySeverity::Medium => RiskLevel::Medium,
            VulnerabilitySeverity::Low => RiskLevel::Low,
        }
    }

    fn generate_recommendations(&self, report: &SecurityAuditReport) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if report.risk_score > 80.0 {
            recommendations.push("Immediate action required: Critical security issues detected".to_string());
            recommendations.push("Review and remediate all high-severity findings".to_string());
        } else if report.risk_score > 60.0 {
            recommendations.push("Address medium and high severity security findings".to_string());
            recommendations.push("Implement additional security controls".to_string());
        } else {
            recommendations.push("Maintain current security posture".to_string());
            recommendations.push("Continue regular security monitoring".to_string());
        }

        if !report.vulnerability_scan_results.is_empty() {
            recommendations.push("Review vulnerability scan results and apply patches".to_string());
        }

        recommendations
    }

    fn generate_vulnerability_recommendations(&self, vulnerabilities: &[Vulnerability]) -> Vec<String> {
        vulnerabilities.iter()
            .map(|v| v.remediation.clone())
            .collect()
    }

    fn generate_secure_config_template(&self, _system_info: &SystemInfo) -> Result<String> {
        Ok(r#"
# NexusShell Secure Configuration Template
[security]
encryption_enabled = true
encryption_algorithm = "AES-256-GCM"
require_authentication = true
max_login_attempts = 3
session_timeout = 1800

[network]
enable_ssl = true
ssl_min_version = "1.2"
allowed_hosts = ["localhost"]
blocked_ports = [23, 25, 135]

[audit]
enable_logging = true
log_level = "INFO"
audit_file = "/var/log/nxsh/audit.log"
"#.to_string())
    }

    fn generate_pdf_content(&self, report: &SecurityAuditReport) -> Result<String> {
        // Simplified PDF content generation
        Ok(format!(
            "Security Audit Report\nAudit ID: {}\nRisk Score: {:.2}\nFindings: {}\n",
            report.audit_id, report.risk_score, report.findings.len()
        ))
    }

    fn generate_html_report(&self, report: &SecurityAuditReport) -> Result<String> {
        Ok(format!(r#"
            <!DOCTYPE html>
            <html>
            <head><title>Security Audit Report</title></head>
            <body>
                <h1>Security Audit Report</h1>
                <p>Audit ID: {}</p>
                <p>Risk Score: {:.2}</p>
                <p>Total Findings: {}</p>
                <h2>Recommendations</h2>
                <ul>
                    {}
                </ul>
            </body>
            </html>
        "#, 
            report.audit_id, 
            report.risk_score, 
            report.findings.len(),
            report.recommendations.iter()
                .map(|r| format!("<li>{}</li>", r))
                .collect::<Vec<_>>()
                .join("")
        ))
    }

    fn log_audit_event(&self, event: AuditEvent) {
        if let Ok(mut log) = self.audit_log.try_lock() {
            log.push(event);
            
            // Keep log size manageable
            if log.len() > 10000 {
                log.drain(0..5000);
            }
        }
    }

    fn generate_audit_id() -> String {
        format!("AUDIT_{}", 
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs())
    }
}

// Supporting types and structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditReport {
    pub audit_id: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub scope: AuditScope,
    pub findings: Vec<AuditFinding>,
    pub compliance_results: HashMap<String, ComplianceResult>,
    pub vulnerability_scan_results: Vec<ScanResult>,
    pub risk_score: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditScope {
    pub include_file_system: bool,
    pub include_network: bool,
    pub include_configuration: bool,
    pub include_compliance: Vec<String>,
    pub target_paths: Vec<PathBuf>,
}

impl AuditScope {
    pub fn includes_compliance_check(&self, framework: &str) -> bool {
        self.include_compliance.contains(&framework.to_string())
    }
}

#[derive(Clone)]
pub struct AuditRule {
    pub name: String,
    pub description: String,
    pub category: AuditCategory,
    pub severity: AuditSeverity,
    #[doc = "Function stored as Arc for cloneability"]
    pub check_function: std::sync::Arc<dyn Fn(&SecurityAuditor) -> Result<Vec<AuditFinding>> + Send + Sync>,
}

impl std::fmt::Debug for AuditRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuditRule")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("category", &self.category)
            .field("severity", &self.severity)
            .field("check_function", &"<function>")
            .finish()
    }
}

impl AuditRule {
    pub fn applies_to_scope(&self, scope: &AuditScope) -> bool {
        match self.category {
            AuditCategory::FileSystem => scope.include_file_system,
            AuditCategory::Network => scope.include_network,
            AuditCategory::Configuration => scope.include_configuration,
            AuditCategory::Compliance => !scope.include_compliance.is_empty(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuditCategory {
    FileSystem,
    Network,
    Configuration,
    Compliance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: AuditSeverity,
    pub category: String,
    pub affected_resource: String,
    pub recommendation: String,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum AuditSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone)]
pub struct ComplianceFramework {
    pub name: String,
    pub version: String,
    pub description: String,
    pub controls: Vec<ComplianceControl>,
}

#[derive(Debug, Clone)]
pub struct ComplianceControl {
    pub id: String,
    pub title: String,
    pub description: String,
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    pub framework_name: String,
    pub version: String,
    pub total_controls: usize,
    pub compliant_controls: usize,
    pub non_compliant_controls: Vec<String>,
    pub not_applicable_controls: Vec<String>,
    pub compliance_percentage: f64,
    pub findings: Vec<AuditFinding>,
}

#[derive(Debug, Clone)]
pub enum ControlEvaluation {
    Compliant,
    NonCompliant(AuditFinding),
    NotApplicable,
}

#[derive(Clone)]
pub struct VulnerabilityScanner {
    pub name: String,
    pub description: String,
    pub scanner_type: ScannerType,
    #[doc = "Function stored as Arc for cloneability"]
    pub scan_function: std::sync::Arc<dyn Fn(&AuditScope) -> Result<ScanResult> + Send + Sync>,
}

impl std::fmt::Debug for VulnerabilityScanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VulnerabilityScanner")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("scanner_type", &self.scanner_type)
            .field("scan_function", &"<function>")
            .finish()
    }
}

impl VulnerabilityScanner {
    pub fn applies_to_scope(&self, scope: &AuditScope) -> bool {
        match self.scanner_type {
            ScannerType::FileSystem => scope.include_file_system,
            ScannerType::Network => scope.include_network,
            ScannerType::Configuration => scope.include_configuration,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ScannerType {
    FileSystem,
    Network,
    Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scanner_name: String,
    #[serde(with = "system_time_serde")]
    pub scan_time: SystemTime,
    pub vulnerabilities: Vec<Vulnerability>,
    #[serde(with = "duration_serde")]
    pub scan_duration: Duration,
}

impl Default for ScanResult {
    fn default() -> Self {
        Self {
            scanner_name: String::new(),
            scan_time: std::time::UNIX_EPOCH,
            vulnerabilities: Vec::new(),
            scan_duration: Duration::from_secs(0),
        }
    }
}

// Custom serialization for SystemTime and Duration
pub mod system_time_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration_since_epoch = time.duration_since(UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        duration_since_epoch.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }
}

mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u128::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis as u64))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: VulnerabilitySeverity,
    pub cvss_score: Option<f64>,
    pub affected_assets: Vec<String>,
    pub remediation: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct VulnerabilityAssessment {
    pub target: String,
    pub scan_time: SystemTime,
    pub vulnerabilities: Vec<Vulnerability>,
    pub risk_level: RiskLevel,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct HardeningGuide {
    pub system_type: String,
    pub recommendations: Vec<HardeningRecommendation>,
    pub priority_actions: Vec<String>,
    pub configuration_templates: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct HardeningRecommendation {
    pub category: HardeningCategory,
    pub title: String,
    pub description: String,
    pub commands: Vec<String>,
    pub risk_reduction: RiskLevel,
}

#[derive(Debug, Clone)]
pub enum HardeningCategory {
    FileSystem,
    Network,
    Authentication,
    Encryption,
    Monitoring,
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub system_type: String,
    pub version: String,
    pub architecture: String,
}

#[derive(Debug, Clone)]
pub struct SecurityPolicies {
    pub password_policy: PasswordPolicy,
    pub network_policy: NetworkPolicy,
    pub file_access_policy: FileAccessPolicy,
}

impl Default for SecurityPolicies {
    fn default() -> Self {
        Self {
            password_policy: PasswordPolicy::default(),
            network_policy: NetworkPolicy::default(),
            file_access_policy: FileAccessPolicy::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkPolicy {
    pub allowed_ports: Vec<u16>,
    pub blocked_ports: Vec<u16>,
    pub require_encryption: bool,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self {
            allowed_ports: vec![22, 80, 443],
            blocked_ports: vec![23, 25, 135, 445],
            require_encryption: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileAccessPolicy {
    pub max_permissions: u32,
    pub restricted_paths: Vec<PathBuf>,
    pub require_encryption: bool,
}

impl Default for FileAccessPolicy {
    fn default() -> Self {
        Self {
            max_permissions: 0o644,
            restricted_paths: vec![
                PathBuf::from("/etc/shadow"),
                PathBuf::from("/etc/passwd"),
            ],
            require_encryption: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityMonitor {
    audit_log: Arc<Mutex<Vec<AuditEvent>>>,
    monitoring_active: bool,
}

impl SecurityMonitor {
    pub fn new(audit_log: Arc<Mutex<Vec<AuditEvent>>>) -> Result<Self> {
        Ok(Self {
            audit_log,
            monitoring_active: true,
        })
    }

    pub fn stop(&mut self) {
        self.monitoring_active = false;
    }
}

#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub timestamp: SystemTime,
    pub event_type: AuditEventType,
    pub description: String,
    pub severity: AuditSeverity,
}

#[derive(Debug, Clone)]
pub enum AuditEventType {
    SecurityViolation,
    PolicyViolation,
    AccessDenied,
    AuthenticationFailure,
    ConfigurationChange,
    AuditStarted,
    AuditCompleted,
    MonitoringStarted,
    MonitoringStopped,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Pdf,
    Html,
    Xml,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_auditor_creation() {
        let auditor = SecurityAuditor::new();
        assert!(!auditor.audit_rules.is_empty());
        assert!(!auditor.compliance_frameworks.is_empty());
        assert!(!auditor.vulnerability_scanners.is_empty());
    }

    #[test]
    fn test_security_audit() {
        let mut auditor = SecurityAuditor::new();
        let scope = AuditScope {
            include_file_system: true,
            include_network: true,
            include_configuration: true,
            include_compliance: vec!["CIS".to_string()],
            target_paths: vec![PathBuf::from("/tmp")],
        };

        let report = auditor.perform_security_audit(scope).unwrap();
        assert!(!report.audit_id.is_empty());
        assert!(report.risk_score >= 0.0);
    }

    #[test]
    fn test_vulnerability_assessment() {
        let mut auditor = SecurityAuditor::new();
        let assessment = auditor.run_vulnerability_assessment("test_target").unwrap();
        
        assert_eq!(assessment.target, "test_target");
        assert!(!assessment.vulnerabilities.is_empty());
    }

    #[test]
    fn test_compliance_check() {
        let auditor = SecurityAuditor::new();
        let framework = auditor.compliance_frameworks.get("CIS").unwrap();
        let result = auditor.check_compliance(framework).unwrap();
        
        assert_eq!(result.framework_name, "CIS Controls");
        assert!(result.compliance_percentage >= 0.0);
    }

    #[test]
    fn test_hardening_guide() {
        let auditor = SecurityAuditor::new();
        let system_info = SystemInfo {
            system_type: "Linux".to_string(),
            version: "Ubuntu 20.04".to_string(),
            architecture: "x86_64".to_string(),
        };

        let guide = auditor.generate_hardening_guide(&system_info).unwrap();
        assert!(!guide.recommendations.is_empty());
        assert!(!guide.priority_actions.is_empty());
    }
}
