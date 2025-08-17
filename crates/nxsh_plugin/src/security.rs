//! Capability-based security system for NexusShell plugins
//! 
//! This module provides a comprehensive security framework with capability-based
//! access control, sandboxing, and policy enforcement for WASI plugins.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};
#[cfg(feature = "async-support")]
use tokio::sync::RwLock;

use crate::{PluginMetadata, PluginError, PluginResult};

/// Security context for WASM plugin execution
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Allowed host functions
    pub allowed_functions: HashSet<String>,
    /// Resource limits
    pub max_memory: usize,
    /// Maximum execution time in milliseconds
    pub max_execution_time: u64,
    /// Network access policy
    pub network_access: NetworkPolicy,
    /// File system access policy
    pub filesystem_access: FilesystemPolicy,
}

/// Network access policy
#[derive(Debug, Clone)]
pub enum NetworkPolicy {
    /// No network access allowed
    None,
    /// Limited access to specific hosts
    Limited(HashSet<String>),
    /// Full network access
    Full,
}

/// Filesystem access policy
#[derive(Debug, Clone)]
pub enum FilesystemPolicy {
    /// No filesystem access
    None,
    /// Read-only access to specific paths
    ReadOnly(HashSet<String>),
    /// Full filesystem access
    Full,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            allowed_functions: HashSet::new(),
            max_memory: 16 * 1024 * 1024, // 16MB
            max_execution_time: 5000, // 5 seconds
            network_access: NetworkPolicy::None,
            filesystem_access: FilesystemPolicy::None,
        }
    }
}

impl SecurityContext {
    /// Create a new security context with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a restrictive security context
    pub fn restrictive() -> Self {
        Self {
            allowed_functions: HashSet::new(),
            max_memory: 1024 * 1024, // 1MB
            max_execution_time: 1000, // 1 second
            network_access: NetworkPolicy::None,
            filesystem_access: FilesystemPolicy::None,
        }
    }

    /// Create a new restrictive security context (alias for backward compatibility)
    pub fn new_restricted() -> Self {
        Self::restrictive()
    }

    /// Create a permissive security context for trusted plugins
    pub fn permissive() -> Self {
        let mut allowed_functions = HashSet::new();
        allowed_functions.insert("print".to_string());
        allowed_functions.insert("log".to_string());
        allowed_functions.insert("get_env".to_string());

        Self {
            allowed_functions,
            max_memory: 64 * 1024 * 1024, // 64MB
            max_execution_time: 30000, // 30 seconds
            network_access: NetworkPolicy::Limited(HashSet::new()),
            filesystem_access: FilesystemPolicy::ReadOnly(HashSet::new()),
        }
    }

    /// Check if a function call is allowed
    pub fn is_function_allowed(&self, function_name: &str) -> bool {
        self.allowed_functions.contains(function_name)
    }

    /// Allow a specific function
    pub fn allow_function(&mut self, function_name: String) {
        self.allowed_functions.insert(function_name);
    }

    /// Check network access policy
    pub fn check_network_access(&self, host: &str) -> Result<()> {
        match &self.network_access {
            NetworkPolicy::None => Err(anyhow!("Network access denied")),
            NetworkPolicy::Limited(allowed_hosts) => {
                if allowed_hosts.contains(host) {
                    Ok(())
                } else {
                    Err(anyhow!("Access to host '{}' denied", host))
                }
            }
            NetworkPolicy::Full => Ok(()),
        }
    }

    /// Check filesystem access policy
    pub fn check_filesystem_access(&self, path: &str, write: bool) -> Result<()> {
        match &self.filesystem_access {
            FilesystemPolicy::None => Err(anyhow!("Filesystem access denied")),
            FilesystemPolicy::ReadOnly(allowed_paths) => {
                if write {
                    Err(anyhow!("Write access denied"))
                } else if allowed_paths.is_empty() || allowed_paths.iter().any(|p| path.starts_with(p)) {
                    Ok(())
                } else {
                    Err(anyhow!("Access to path '{}' denied", path))
                }
            }
            FilesystemPolicy::Full => Ok(()),
        }
    }
}

/// Capability-based security manager
pub struct CapabilityManager {
    policies: Arc<RwLock<HashMap<String, SecurityPolicy>>>,
    capabilities: Arc<RwLock<CapabilityRegistry>>,
    default_policy: SecurityPolicy,
}

impl Default for CapabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityManager {
    /// Create a new capability manager
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            capabilities: Arc::new(RwLock::new(CapabilityRegistry::new())),
            default_policy: SecurityPolicy::restrictive(),
        }
    }

    /// Initialize the capability manager
    pub async fn initialize(&mut self) -> Result<()> {
        log::info!("Initializing capability-based security manager");
        
        // Load default capabilities
        self.load_default_capabilities().await?;
        
        // Load security policies
        self.load_security_policies().await?;
        
        log::info!("Capability manager initialized successfully");
        Ok(())
    }

    /// Load default capabilities
    async fn load_default_capabilities(&self) -> Result<()> {
        let mut capabilities = self.capabilities.write().await;
        
        // File system capabilities
        capabilities.register_capability(Capability {
            name: "filesystem.read".to_string(),
            description: "Read access to file system".to_string(),
            category: CapabilityCategory::FileSystem,
            risk_level: RiskLevel::Medium,
            required_permissions: vec!["read".to_string()],
        });

        capabilities.register_capability(Capability {
            name: "filesystem.write".to_string(),
            description: "Write access to file system".to_string(),
            category: CapabilityCategory::FileSystem,
            risk_level: RiskLevel::High,
            required_permissions: vec!["write".to_string()],
        });

        capabilities.register_capability(Capability {
            name: "filesystem.execute".to_string(),
            description: "Execute files from file system".to_string(),
            category: CapabilityCategory::FileSystem,
            risk_level: RiskLevel::High,
            required_permissions: vec!["execute".to_string()],
        });

        // Network capabilities
        capabilities.register_capability(Capability {
            name: "network.connect".to_string(),
            description: "Make outbound network connections".to_string(),
            category: CapabilityCategory::Network,
            risk_level: RiskLevel::Medium,
            required_permissions: vec!["connect".to_string()],
        });

        capabilities.register_capability(Capability {
            name: "network.listen".to_string(),
            description: "Listen for incoming network connections".to_string(),
            category: CapabilityCategory::Network,
            risk_level: RiskLevel::High,
            required_permissions: vec!["bind".to_string()],
        });

        // Process capabilities
        capabilities.register_capability(Capability {
            name: "process.spawn".to_string(),
            description: "Spawn new processes".to_string(),
            category: CapabilityCategory::Process,
            risk_level: RiskLevel::Critical,
            required_permissions: vec!["spawn".to_string()],
        });

        capabilities.register_capability(Capability {
            name: "process.signal".to_string(),
            description: "Send signals to processes".to_string(),
            category: CapabilityCategory::Process,
            risk_level: RiskLevel::High,
            required_permissions: vec!["signal".to_string()],
        });

        // Environment capabilities
        capabilities.register_capability(Capability {
            name: "env.read".to_string(),
            description: "Read environment variables".to_string(),
            category: CapabilityCategory::Environment,
            risk_level: RiskLevel::Low,
            required_permissions: vec!["env_read".to_string()],
        });

        capabilities.register_capability(Capability {
            name: "env.write".to_string(),
            description: "Write environment variables".to_string(),
            category: CapabilityCategory::Environment,
            risk_level: RiskLevel::Medium,
            required_permissions: vec!["env_write".to_string()],
        });

        // System capabilities
        capabilities.register_capability(Capability {
            name: "system.time".to_string(),
            description: "Access system time".to_string(),
            category: CapabilityCategory::System,
            risk_level: RiskLevel::Low,
            required_permissions: vec!["time".to_string()],
        });

        capabilities.register_capability(Capability {
            name: "system.random".to_string(),
            description: "Access random number generator".to_string(),
            category: CapabilityCategory::System,
            risk_level: RiskLevel::Low,
            required_permissions: vec!["random".to_string()],
        });

        Ok(())
    }

    /// Load security policies from configuration
    async fn load_security_policies(&self) -> Result<()> {
        // In a real implementation, this would load from configuration files
        let mut policies = self.policies.write().await;
        
        // Default restrictive policy
        policies.insert("default".to_string(), SecurityPolicy::restrictive());
        
        // Trusted policy for verified plugins
        policies.insert("trusted".to_string(), SecurityPolicy::trusted());
        
        // Development policy for testing
        policies.insert("development".to_string(), SecurityPolicy::development());
        
        Ok(())
    }

    /// Validate a plugin's requested capabilities
    pub async fn validate_plugin_security(&self, metadata: &PluginMetadata) -> PluginResult<()> {
        log::debug!("Validating capabilities for plugin: {}", metadata.name);

        let capabilities = self.capabilities.read().await;
        
        for capability_name in &metadata.capabilities {
            // Check if capability exists
            if !capabilities.has_capability(capability_name) {
                return Err(PluginError::ValidationFailed(
                    format!("Unknown capability: {capability_name}")
                ));
            }

            // Check if capability is allowed by policy
            let policy = self.get_policy_for_plugin(metadata).await;
            if !policy.allows_capability(capability_name) {
                return Err(PluginError::CapabilityDenied(
                    format!("Capability {capability_name} denied by policy")
                ));
            }

            // Check risk level
            if let Some(capability) = capabilities.get_capability(capability_name) {
                if !policy.allows_risk_level(capability.risk_level) {
                    return Err(PluginError::CapabilityDenied(
                        format!("Capability {capability_name} risk level too high")
                    ));
                }
            }
        }

        log::debug!("Plugin {} capabilities validated successfully", metadata.name);
        Ok(())
    }

    /// Check if a plugin has permission to execute a specific function
    pub async fn check_capability_permission(&self, plugin_id: &str, function: &str) -> PluginResult<()> {
        // In a real implementation, this would check function-specific permissions
        // For now, allow all executions for loaded plugins
        log::debug!("Checking execution permission for {plugin_id}::{function}");
        Ok(())
    }

    /// Get security policy for a plugin
    async fn get_policy_for_plugin(&self, metadata: &PluginMetadata) -> SecurityPolicy {
        let policies = self.policies.read().await;
        
        // Determine policy based on plugin characteristics
        if self.is_trusted_plugin(metadata) {
            policies.get("trusted").cloned().unwrap_or_else(|| self.default_policy.clone())
        } else if self.is_development_plugin(metadata) {
            policies.get("development").cloned().unwrap_or_else(|| self.default_policy.clone())
        } else {
            policies.get("default").cloned().unwrap_or_else(|| self.default_policy.clone())
        }
    }

    /// Check if a plugin is trusted (e.g., signed by a trusted authority)
    fn is_trusted_plugin(&self, metadata: &PluginMetadata) -> bool {
        // In a real implementation, this would check digital signatures
        metadata.author.contains("nexusshell") || metadata.author.contains("trusted")
    }

    /// Check if a plugin is in development mode
    fn is_development_plugin(&self, metadata: &PluginMetadata) -> bool {
        metadata.version.contains("dev") || metadata.version.contains("alpha") || metadata.version.contains("beta")
    }

    /// Create a sandbox context for a plugin
    pub async fn create_sandbox_context(&self, plugin_id: &str, metadata: &PluginMetadata) -> Result<SandboxContext> {
        log::debug!("Creating sandbox context for plugin: {plugin_id}");

        let policy = self.get_policy_for_plugin(metadata).await;
        let capabilities = self.capabilities.read().await;

        let mut allowed_capabilities = HashSet::new();
        let mut resource_limits = ResourceLimits::default();

        // Process allowed capabilities
        for capability_name in &metadata.capabilities {
            if policy.allows_capability(capability_name) {
                if let Some(capability) = capabilities.get_capability(capability_name) {
                    allowed_capabilities.insert(capability.clone());
                }
            }
        }

        // Set resource limits based on policy
        resource_limits.max_memory = policy.max_memory_bytes;
        resource_limits.max_cpu_time = policy.max_cpu_time_seconds;
        resource_limits.max_file_descriptors = policy.max_file_descriptors;
        resource_limits.max_network_connections = policy.max_network_connections;

        Ok(SandboxContext {
            plugin_id: plugin_id.to_string(),
            allowed_capabilities,
            resource_limits,
            allowed_paths: policy.allowed_file_paths.clone(),
            allowed_hosts: policy.allowed_network_hosts.clone(),
        })
    }

    /// Add a custom security policy
    pub async fn add_policy(&self, name: String, policy: SecurityPolicy) {
        let mut policies = self.policies.write().await;
        policies.insert(name, policy);
    }

    /// Remove a security policy
    pub async fn remove_policy(&self, name: &str) -> bool {
        let mut policies = self.policies.write().await;
        policies.remove(name).is_some()
    }

    /// List all available capabilities
    pub async fn list_capabilities(&self) -> Vec<Capability> {
        let capabilities = self.capabilities.read().await;
        capabilities.list_all()
    }

    /// Get security statistics
    pub async fn get_statistics(&self) -> SecurityStatistics {
        let capabilities = self.capabilities.read().await;
        let policies = self.policies.read().await;

        SecurityStatistics {
            total_capabilities: capabilities.count(),
            total_policies: policies.len(),
            high_risk_capabilities: capabilities.count_by_risk_level(RiskLevel::High),
            critical_capabilities: capabilities.count_by_risk_level(RiskLevel::Critical),
        }
    }
}

/// Security policy for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub name: String,
    pub description: String,
    pub allowed_capabilities: HashSet<String>,
    pub denied_capabilities: HashSet<String>,
    pub max_risk_level: RiskLevel,
    pub max_memory_bytes: usize,
    pub max_cpu_time_seconds: u64,
    pub max_file_descriptors: usize,
    pub max_network_connections: usize,
    pub allowed_file_paths: Vec<PathBuf>,
    pub denied_file_paths: Vec<PathBuf>,
    pub allowed_network_hosts: Vec<String>,
    pub denied_network_hosts: Vec<String>,
    pub require_signature: bool,
    pub allow_native_code: bool,
}

impl SecurityPolicy {
    /// Create a restrictive security policy
    pub fn restrictive() -> Self {
        Self {
            name: "Restrictive".to_string(),
            description: "Highly restrictive policy for untrusted plugins".to_string(),
            allowed_capabilities: ["system.time", "system.random", "env.read"]
                .iter().map(|s| s.to_string()).collect(),
            denied_capabilities: ["process.spawn", "process.signal", "network.listen"]
                .iter().map(|s| s.to_string()).collect(),
            max_risk_level: RiskLevel::Medium,
            max_memory_bytes: 32 * 1024 * 1024, // 32MB
            max_cpu_time_seconds: 10,
            max_file_descriptors: 10,
            max_network_connections: 2,
            allowed_file_paths: vec![],
            denied_file_paths: vec![PathBuf::from("/etc"), PathBuf::from("/sys")],
            allowed_network_hosts: vec![],
            denied_network_hosts: vec!["localhost".to_string(), "127.0.0.1".to_string()],
            require_signature: true,
            allow_native_code: false,
        }
    }

    /// Create a trusted security policy
    pub fn trusted() -> Self {
        Self {
            name: "Trusted".to_string(),
            description: "Permissive policy for trusted plugins".to_string(),
            allowed_capabilities: HashSet::new(), // Allow all by default
            denied_capabilities: ["process.spawn"].iter().map(|s| s.to_string()).collect(),
            max_risk_level: RiskLevel::High,
            max_memory_bytes: 256 * 1024 * 1024, // 256MB
            max_cpu_time_seconds: 300,
            max_file_descriptors: 100,
            max_network_connections: 50,
            allowed_file_paths: vec![],
            denied_file_paths: vec![PathBuf::from("/etc/passwd"), PathBuf::from("/etc/shadow")],
            allowed_network_hosts: vec![],
            denied_network_hosts: vec![],
            require_signature: true,
            allow_native_code: false,
        }
    }

    /// Create a development security policy
    pub fn development() -> Self {
        Self {
            name: "Development".to_string(),
            description: "Permissive policy for development and testing".to_string(),
            allowed_capabilities: HashSet::new(), // Allow all
            denied_capabilities: HashSet::new(),
            max_risk_level: RiskLevel::Critical,
            max_memory_bytes: 512 * 1024 * 1024, // 512MB
            max_cpu_time_seconds: 600,
            max_file_descriptors: 200,
            max_network_connections: 100,
            allowed_file_paths: vec![],
            denied_file_paths: vec![],
            allowed_network_hosts: vec![],
            denied_network_hosts: vec![],
            require_signature: false,
            allow_native_code: true,
        }
    }

    /// Check if a capability is allowed
    pub fn allows_capability(&self, capability: &str) -> bool {
        if self.denied_capabilities.contains(capability) {
            return false;
        }
        
        if self.allowed_capabilities.is_empty() {
            true // Allow all if no specific allowlist
        } else {
            self.allowed_capabilities.contains(capability)
        }
    }

    /// Check if a risk level is allowed
    pub fn allows_risk_level(&self, risk_level: RiskLevel) -> bool {
        risk_level <= self.max_risk_level
    }
}

/// Capability registry
#[derive(Debug)]
pub struct CapabilityRegistry {
    capabilities: HashMap<String, Capability>,
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityRegistry {
    /// Create a new capability registry
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
        }
    }

    /// Register a capability
    pub fn register_capability(&mut self, capability: Capability) {
        self.capabilities.insert(capability.name.clone(), capability);
    }

    /// Check if a capability exists
    pub fn has_capability(&self, name: &str) -> bool {
        self.capabilities.contains_key(name)
    }

    /// Get a capability
    pub fn get_capability(&self, name: &str) -> Option<&Capability> {
        self.capabilities.get(name)
    }

    /// List all capabilities
    pub fn list_all(&self) -> Vec<Capability> {
        self.capabilities.values().cloned().collect()
    }

    /// Count capabilities
    pub fn count(&self) -> usize {
        self.capabilities.len()
    }

    /// Count capabilities by risk level
    pub fn count_by_risk_level(&self, risk_level: RiskLevel) -> usize {
        self.capabilities.values()
            .filter(|cap| cap.risk_level == risk_level)
            .count()
    }
}

/// A specific capability that plugins can request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub category: CapabilityCategory,
    pub risk_level: RiskLevel,
    pub required_permissions: Vec<String>,
}

/// Capability categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CapabilityCategory {
    FileSystem,
    Network,
    Process,
    Environment,
    System,
    Custom(String),
}

/// Risk levels for capabilities
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Sandbox context for plugin execution
#[derive(Debug, Clone)]
pub struct SandboxContext {
    pub plugin_id: String,
    pub allowed_capabilities: HashSet<Capability>,
    pub resource_limits: ResourceLimits,
    pub allowed_paths: Vec<PathBuf>,
    pub allowed_hosts: Vec<String>,
}

impl SandboxContext {
    /// Check if a command can be executed
    pub fn can_execute_command(&self, _command: &str) -> bool {
        // Basic implementation - check if execution capability is allowed
        self.allowed_capabilities.iter().any(|cap| {
            cap.name == "execution" || cap.name.contains("execute")
        })
    }
}

/// Resource limits for plugin execution
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory: usize,
    pub max_cpu_time: u64,
    pub max_file_descriptors: usize,
    pub max_network_connections: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 64 * 1024 * 1024, // 64MB
            max_cpu_time: 30, // 30 seconds
            max_file_descriptors: 20,
            max_network_connections: 5,
        }
    }
}

/// Security statistics
#[derive(Debug, Clone)]
pub struct SecurityStatistics {
    pub total_capabilities: usize,
    pub total_policies: usize,
    pub high_risk_capabilities: usize,
    pub critical_capabilities: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_capability_manager_creation() {
        let manager = CapabilityManager::new();
        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_capabilities, 0);
    }

    #[tokio::test]
    async fn test_capability_manager_initialization() {
        let mut manager = CapabilityManager::new();
        manager.initialize().await.unwrap();
        
        let stats = manager.get_statistics().await;
        assert!(stats.total_capabilities > 0);
        assert!(stats.total_policies > 0);
    }

    #[test]
    fn test_security_policy_creation() {
        let restrictive = SecurityPolicy::restrictive();
        assert_eq!(restrictive.name, "Restrictive");
        assert!(restrictive.require_signature);
        assert!(!restrictive.allow_native_code);

        let trusted = SecurityPolicy::trusted();
        assert_eq!(trusted.name, "Trusted");
        assert!(trusted.max_memory_bytes > restrictive.max_memory_bytes);

        let development = SecurityPolicy::development();
        assert_eq!(development.name, "Development");
        assert!(!development.require_signature);
        assert!(development.allow_native_code);
    }

    #[test]
    fn test_capability_allows() {
        let policy = SecurityPolicy::restrictive();
        
        assert!(policy.allows_capability("system.time"));
        assert!(policy.allows_capability("env.read"));
        assert!(!policy.allows_capability("process.spawn"));
        assert!(!policy.allows_capability("network.listen"));
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }

    #[test]
    fn test_capability_registry() {
        let mut registry = CapabilityRegistry::new();
        
        let capability = Capability {
            name: "test.capability".to_string(),
            description: "Test capability".to_string(),
            category: CapabilityCategory::System,
            risk_level: RiskLevel::Low,
            required_permissions: vec!["test".to_string()],
        };

        registry.register_capability(capability.clone());
        
        assert!(registry.has_capability("test.capability"));
        assert_eq!(registry.get_capability("test.capability"), Some(&capability));
        assert_eq!(registry.count(), 1);
    }

    #[tokio::test]
    async fn test_plugin_validation() {
        let mut manager = CapabilityManager::new();
        manager.initialize().await.unwrap();

        let valid_metadata = PluginMetadata {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "trusted".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            dependencies: HashMap::new(),
            capabilities: vec!["system.time".to_string()],
            exports: vec![],
            min_nexus_version: "0.1.0".to_string(),
            max_nexus_version: None,
        };

        assert!(manager.validate_plugin_security(&valid_metadata).await.is_ok());

        let invalid_metadata = PluginMetadata {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "untrusted".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            dependencies: HashMap::new(),
            capabilities: vec!["unknown.capability".to_string()],
            exports: vec![],
            min_nexus_version: "0.1.0".to_string(),
            max_nexus_version: None,
        };

        assert!(manager.validate_plugin_security(&invalid_metadata).await.is_err());
    }
} 