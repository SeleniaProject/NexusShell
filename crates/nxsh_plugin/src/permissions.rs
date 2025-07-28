use anyhow::{Result, Context};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use log::{info, warn, error, debug};

use crate::{PluginMetadata, PluginError, security::SandboxContext};

/// Permission management system for plugins
pub struct PermissionManager {
    permission_policies: Arc<RwLock<HashMap<String, PermissionPolicy>>>,
    active_permissions: Arc<RwLock<HashMap<String, ActivePermissionSet>>>,
    capability_definitions: Arc<RwLock<HashMap<String, CapabilityDefinition>>>,
    permission_audit_log: Arc<RwLock<Vec<PermissionAuditEntry>>>,
    config: PermissionConfig,
}

impl PermissionManager {
    /// Create a new permission manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            permission_policies: Arc::new(RwLock::new(HashMap::new())),
            active_permissions: Arc::new(RwLock::new(HashMap::new())),
            capability_definitions: Arc::new(RwLock::new(HashMap::new())),
            permission_audit_log: Arc::new(RwLock::new(Vec::new())),
            config: PermissionConfig::default(),
        })
    }
    
    /// Initialize the permission manager
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing permission management system");
        
        // Load permission policies
        self.load_permission_policies().await?;
        
        // Initialize capability definitions
        self.initialize_capability_definitions().await?;
        
        // Load audit log
        self.load_audit_log().await?;
        
        info!("Permission management system initialized successfully");
        Ok(())
    }
    
    /// Create a minimal privilege execution context for a plugin
    pub async fn create_execution_context(
        &self,
        plugin_id: &str,
        metadata: &PluginMetadata,
        requested_capabilities: &[String],
    ) -> Result<ExecutionContext, PluginError> {
        debug!("Creating execution context for plugin '{}'", plugin_id);
        
        // Get permission policy for plugin
        let policy = self.get_permission_policy(plugin_id, metadata).await?;
        
        // Validate requested capabilities
        let validated_capabilities = self.validate_capabilities(requested_capabilities, &policy).await?;
        
        // Create minimal permission set
        let permission_set = self.create_minimal_permission_set(&validated_capabilities).await?;
        
        // Create sandbox constraints
        let sandbox_constraints = self.create_sandbox_constraints(&policy, &permission_set).await?;
        
        // Create execution context
        let context = ExecutionContext {
            plugin_id: plugin_id.to_string(),
            permission_set,
            sandbox_constraints,
            created_at: SystemTime::now(),
            last_used: SystemTime::now(),
            usage_count: 0,
            resource_limits: policy.resource_limits.clone(),
        };
        
        // Store active permissions
        {
            let mut active = self.active_permissions.write().await;
            active.insert(plugin_id.to_string(), ActivePermissionSet {
                context: context.clone(),
                granted_at: SystemTime::now(),
                expires_at: SystemTime::now() + Duration::from_secs(policy.session_timeout_seconds),
            });
        }
        
        // Log permission grant
        self.log_permission_event(
            plugin_id,
            PermissionAction::Granted,
            format!("Granted {} capabilities", validated_capabilities.len()),
        ).await;
        
        info!("Created execution context for plugin '{}' with {} capabilities", 
              plugin_id, validated_capabilities.len());
        
        Ok(context)
    }
    
    /// Check if a plugin has permission for a specific operation
    pub async fn check_permission(
        &self,
        plugin_id: &str,
        operation: &PermissionOperation,
    ) -> Result<PermissionResult, PluginError> {
        debug!("Checking permission for plugin '{}': {:?}", plugin_id, operation);
        
        // Get active permissions
        let active_permissions = self.active_permissions.read().await;
        let permission_set = active_permissions.get(plugin_id)
            .ok_or_else(|| PluginError::SecurityError(
                format!("No active permissions for plugin '{}'", plugin_id)
            ))?;
        
        // Check if permissions have expired
        if SystemTime::now() > permission_set.expires_at {
            return Ok(PermissionResult::denied(
                "Permission session has expired".to_string()
            ));
        }
        
        // Check specific operation permission
        let result = self.evaluate_permission(&permission_set.context, operation).await?;
        
        // Log permission check
        self.log_permission_event(
            plugin_id,
            PermissionAction::Checked,
            format!("Operation: {:?}, Result: {}", operation, result.allowed),
        ).await;
        
        // Update usage statistics
        if result.allowed {
            let mut active = self.active_permissions.write().await;
            if let Some(perm_set) = active.get_mut(plugin_id) {
                perm_set.context.last_used = SystemTime::now();
                perm_set.context.usage_count += 1;
            }
        }
        
        Ok(result)
    }
    
    /// Revoke permissions for a plugin
    pub async fn revoke_permissions(&self, plugin_id: &str, reason: String) -> Result<()> {
        debug!("Revoking permissions for plugin '{}'", plugin_id);
        
        let mut active = self.active_permissions.write().await;
        if active.remove(plugin_id).is_some() {
            // Log revocation
            self.log_permission_event(
                plugin_id,
                PermissionAction::Revoked,
                reason,
            ).await;
            
            info!("Revoked permissions for plugin '{}'", plugin_id);
        }
        
        Ok(())
    }
    
    /// Update permission policy for a plugin
    pub async fn update_permission_policy(
        &self,
        plugin_id: String,
        policy: PermissionPolicy,
    ) -> Result<()> {
        let mut policies = self.permission_policies.write().await;
        policies.insert(plugin_id.clone(), policy.clone());
        
        // Save updated policies
        self.save_permission_policies().await?;
        
        // Log policy update
        self.log_permission_event(
            &plugin_id,
            PermissionAction::PolicyUpdated,
            format!("Updated to policy level: {:?}", policy.trust_level),
        ).await;
        
        info!("Updated permission policy for plugin '{}'", plugin_id);
        Ok(())
    }
    
    /// Create a capability sandbox for a plugin
    pub async fn create_capability_sandbox(
        &self,
        plugin_id: &str,
        context: &ExecutionContext,
    ) -> Result<CapabilitySandbox> {
        debug!("Creating capability sandbox for plugin '{}'", plugin_id);
        
        let sandbox = CapabilitySandbox {
            plugin_id: plugin_id.to_string(),
            allowed_capabilities: context.permission_set.capabilities.clone(),
            file_system_access: self.create_fs_sandbox(&context.sandbox_constraints).await?,
            network_access: self.create_network_sandbox(&context.sandbox_constraints).await?,
            process_access: self.create_process_sandbox(&context.sandbox_constraints).await?,
            resource_limits: context.resource_limits.clone(),
            created_at: SystemTime::now(),
        };
        
        info!("Created capability sandbox for plugin '{}'", plugin_id);
        Ok(sandbox)
    }
    
    /// Get permission audit log
    pub async fn get_audit_log(&self, plugin_id: Option<&str>) -> Vec<PermissionAuditEntry> {
        let log = self.permission_audit_log.read().await;
        
        if let Some(id) = plugin_id {
            log.iter()
                .filter(|entry| entry.plugin_id == id)
                .cloned()
                .collect()
        } else {
            log.clone()
        }
    }
    
    /// Clean up expired permissions
    pub async fn cleanup_expired_permissions(&self) -> Result<usize> {
        let mut active = self.active_permissions.write().await;
        let now = SystemTime::now();
        
        let expired_plugins: Vec<String> = active
            .iter()
            .filter(|(_, perm_set)| now > perm_set.expires_at)
            .map(|(plugin_id, _)| plugin_id.clone())
            .collect();
        
        for plugin_id in &expired_plugins {
            active.remove(plugin_id);
            
            // Log expiration
            self.log_permission_event(
                plugin_id,
                PermissionAction::Expired,
                "Permission session expired".to_string(),
            ).await;
        }
        
        let count = expired_plugins.len();
        if count > 0 {
            info!("Cleaned up {} expired permission sessions", count);
        }
        
        Ok(count)
    }
    
    // Private helper methods
    
    async fn get_permission_policy(
        &self,
        plugin_id: &str,
        metadata: &PluginMetadata,
    ) -> Result<PermissionPolicy, PluginError> {
        let policies = self.permission_policies.read().await;
        
        // Check for plugin-specific policy
        if let Some(policy) = policies.get(plugin_id) {
            return Ok(policy.clone());
        }
        
        // Check for author-based policy
        let author_key = format!("author:{}", metadata.author);
        if let Some(policy) = policies.get(&author_key) {
            return Ok(policy.clone());
        }
        
        // Use default policy based on plugin metadata
        Ok(self.create_default_policy(metadata))
    }
    
    fn create_default_policy(&self, metadata: &PluginMetadata) -> PermissionPolicy {
        // Create a restrictive default policy
        PermissionPolicy {
            trust_level: TrustLevel::Untrusted,
            allowed_capabilities: HashSet::new(),
            denied_capabilities: HashSet::new(),
            resource_limits: ResourceLimits::strict(),
            session_timeout_seconds: 3600, // 1 hour
            require_user_approval: true,
            audit_all_operations: true,
            network_restrictions: NetworkRestrictions::none(),
            file_system_restrictions: FileSystemRestrictions::none(),
            process_restrictions: ProcessRestrictions::strict(),
        }
    }
    
    async fn validate_capabilities(
        &self,
        requested: &[String],
        policy: &PermissionPolicy,
    ) -> Result<Vec<String>, PluginError> {
        let mut validated = Vec::new();
        let capability_defs = self.capability_definitions.read().await;
        
        for capability in requested {
            // Check if capability exists
            if !capability_defs.contains_key(capability) {
                return Err(PluginError::SecurityError(
                    format!("Unknown capability: {}", capability)
                ));
            }
            
            // Check if explicitly denied
            if policy.denied_capabilities.contains(capability) {
                continue; // Skip denied capabilities
            }
            
            // Check if allowed
            if policy.allowed_capabilities.is_empty() || 
               policy.allowed_capabilities.contains(capability) {
                validated.push(capability.clone());
            }
        }
        
        Ok(validated)
    }
    
    async fn create_minimal_permission_set(
        &self,
        capabilities: &[String],
    ) -> Result<MinimalPermissionSet> {
        let capability_defs = self.capability_definitions.read().await;
        let mut permission_set = MinimalPermissionSet {
            capabilities: capabilities.to_vec(),
            derived_permissions: HashSet::new(),
        };
        
        // Derive minimal required permissions from capabilities
        for capability in capabilities {
            if let Some(def) = capability_defs.get(capability) {
                permission_set.derived_permissions.extend(def.required_permissions.clone());
            }
        }
        
        Ok(permission_set)
    }
    
    async fn create_sandbox_constraints(
        &self,
        policy: &PermissionPolicy,
        permission_set: &MinimalPermissionSet,
    ) -> Result<SandboxConstraints> {
        Ok(SandboxConstraints {
            file_system: policy.file_system_restrictions.clone(),
            network: policy.network_restrictions.clone(),
            process: policy.process_restrictions.clone(),
            memory_limit_mb: policy.resource_limits.max_memory_mb,
            cpu_limit_percent: policy.resource_limits.max_cpu_percent,
            execution_timeout_seconds: policy.resource_limits.max_execution_time_seconds,
            allowed_syscalls: self.derive_allowed_syscalls(permission_set).await?,
        })
    }
    
    async fn derive_allowed_syscalls(
        &self,
        permission_set: &MinimalPermissionSet,
    ) -> Result<HashSet<String>> {
        let mut syscalls = HashSet::new();
        
        // Basic syscalls always allowed
        syscalls.extend([
            "read".to_string(),
            "write".to_string(),
            "exit".to_string(),
            "exit_group".to_string(),
        ]);
        
        // Add syscalls based on capabilities
        for capability in &permission_set.capabilities {
            match capability.as_str() {
                "file_read" => {
                    syscalls.extend([
                        "open".to_string(),
                        "openat".to_string(),
                        "stat".to_string(),
                        "fstat".to_string(),
                    ]);
                }
                "file_write" => {
                    syscalls.extend([
                        "creat".to_string(),
                        "unlink".to_string(),
                        "rename".to_string(),
                    ]);
                }
                "network_request" => {
                    syscalls.extend([
                        "socket".to_string(),
                        "connect".to_string(),
                        "send".to_string(),
                        "recv".to_string(),
                    ]);
                }
                _ => {}
            }
        }
        
        Ok(syscalls)
    }
    
    async fn evaluate_permission(
        &self,
        context: &ExecutionContext,
        operation: &PermissionOperation,
    ) -> Result<PermissionResult> {
        match operation {
            PermissionOperation::FileRead { path } => {
                if context.permission_set.capabilities.contains(&"file_read".to_string()) {
                    if self.is_path_allowed(path, &context.sandbox_constraints.file_system).await? {
                        Ok(PermissionResult::allowed())
                    } else {
                        Ok(PermissionResult::denied("Path access denied".to_string()))
                    }
                } else {
                    Ok(PermissionResult::denied("File read capability not granted".to_string()))
                }
            }
            PermissionOperation::FileWrite { path } => {
                if context.permission_set.capabilities.contains(&"file_write".to_string()) {
                    if self.is_path_allowed(path, &context.sandbox_constraints.file_system).await? {
                        Ok(PermissionResult::allowed())
                    } else {
                        Ok(PermissionResult::denied("Path write access denied".to_string()))
                    }
                } else {
                    Ok(PermissionResult::denied("File write capability not granted".to_string()))
                }
            }
            PermissionOperation::NetworkRequest { url } => {
                if context.permission_set.capabilities.contains(&"network_request".to_string()) {
                    if self.is_url_allowed(url, &context.sandbox_constraints.network).await? {
                        Ok(PermissionResult::allowed())
                    } else {
                        Ok(PermissionResult::denied("Network access denied".to_string()))
                    }
                } else {
                    Ok(PermissionResult::denied("Network capability not granted".to_string()))
                }
            }
            PermissionOperation::ProcessExecute { command } => {
                if context.permission_set.capabilities.contains(&"command_execute".to_string()) {
                    if self.is_command_allowed(command, &context.sandbox_constraints.process).await? {
                        Ok(PermissionResult::allowed())
                    } else {
                        Ok(PermissionResult::denied("Command execution denied".to_string()))
                    }
                } else {
                    Ok(PermissionResult::denied("Process execution capability not granted".to_string()))
                }
            }
            PermissionOperation::EnvironmentRead { variable } => {
                if context.permission_set.capabilities.contains(&"env_read".to_string()) {
                    Ok(PermissionResult::allowed())
                } else {
                    Ok(PermissionResult::denied("Environment read capability not granted".to_string()))
                }
            }
        }
    }
    
    async fn is_path_allowed(
        &self,
        path: &Path,
        restrictions: &FileSystemRestrictions,
    ) -> Result<bool> {
        match restrictions {
            FileSystemRestrictions::None => Ok(false),
            FileSystemRestrictions::ReadOnly { allowed_paths } => {
                Ok(allowed_paths.iter().any(|allowed| path.starts_with(allowed)))
            }
            FileSystemRestrictions::Limited { allowed_paths, .. } => {
                Ok(allowed_paths.iter().any(|allowed| path.starts_with(allowed)))
            }
            FileSystemRestrictions::Full => Ok(true),
        }
    }
    
    async fn is_url_allowed(
        &self,
        url: &str,
        restrictions: &NetworkRestrictions,
    ) -> Result<bool> {
        match restrictions {
            NetworkRestrictions::None => Ok(false),
            NetworkRestrictions::Limited { allowed_domains, allowed_ports } => {
                // Simple domain/port checking
                // In a real implementation, this would be more sophisticated
                Ok(allowed_domains.iter().any(|domain| url.contains(domain)))
            }
            NetworkRestrictions::Full => Ok(true),
        }
    }
    
    async fn is_command_allowed(
        &self,
        command: &str,
        restrictions: &ProcessRestrictions,
    ) -> Result<bool> {
        match restrictions {
            ProcessRestrictions::None => Ok(false),
            ProcessRestrictions::Limited { allowed_commands } => {
                Ok(allowed_commands.contains(&command.to_string()))
            }
            ProcessRestrictions::Full => Ok(true),
        }
    }
    
    async fn create_fs_sandbox(&self, constraints: &SandboxConstraints) -> Result<FileSystemSandbox> {
        Ok(FileSystemSandbox {
            restrictions: constraints.file_system.clone(),
            temp_directory: None, // Could create isolated temp dir
        })
    }
    
    async fn create_network_sandbox(&self, constraints: &SandboxConstraints) -> Result<NetworkSandbox> {
        Ok(NetworkSandbox {
            restrictions: constraints.network.clone(),
            proxy_config: None, // Could set up network proxy
        })
    }
    
    async fn create_process_sandbox(&self, constraints: &SandboxConstraints) -> Result<ProcessSandbox> {
        Ok(ProcessSandbox {
            restrictions: constraints.process.clone(),
            allowed_syscalls: constraints.allowed_syscalls.clone(),
        })
    }
    
    async fn log_permission_event(&self, plugin_id: &str, action: PermissionAction, details: String) {
        let entry = PermissionAuditEntry {
            plugin_id: plugin_id.to_string(),
            action,
            details,
            timestamp: SystemTime::now(),
        };
        
        let mut log = self.permission_audit_log.write().await;
        log.push(entry);
        
        // Keep log size manageable
        if log.len() > self.config.max_audit_entries {
            log.drain(0..self.config.max_audit_entries / 2);
        }
    }
    
    async fn load_permission_policies(&self) -> Result<()> {
        // Load policies from configuration file
        // For now, initialize with default policies
        Ok(())
    }
    
    async fn save_permission_policies(&self) -> Result<()> {
        // Save policies to configuration file
        // For now, this is a no-op
        Ok(())
    }
    
    async fn initialize_capability_definitions(&self) -> Result<()> {
        let mut defs = self.capability_definitions.write().await;
        
        // Define standard capabilities
        defs.insert("file_read".to_string(), CapabilityDefinition {
            name: "file_read".to_string(),
            description: "Read files from the file system".to_string(),
            required_permissions: vec!["fs:read".to_string()],
            risk_level: RiskLevel::Medium,
        });
        
        defs.insert("file_write".to_string(), CapabilityDefinition {
            name: "file_write".to_string(),
            description: "Write files to the file system".to_string(),
            required_permissions: vec!["fs:write".to_string()],
            risk_level: RiskLevel::High,
        });
        
        defs.insert("network_request".to_string(), CapabilityDefinition {
            name: "network_request".to_string(),
            description: "Make network requests".to_string(),
            required_permissions: vec!["net:request".to_string()],
            risk_level: RiskLevel::Medium,
        });
        
        defs.insert("command_execute".to_string(), CapabilityDefinition {
            name: "command_execute".to_string(),
            description: "Execute system commands".to_string(),
            required_permissions: vec!["proc:execute".to_string()],
            risk_level: RiskLevel::Critical,
        });
        
        defs.insert("env_read".to_string(), CapabilityDefinition {
            name: "env_read".to_string(),
            description: "Read environment variables".to_string(),
            required_permissions: vec!["env:read".to_string()],
            risk_level: RiskLevel::Low,
        });
        
        Ok(())
    }
    
    async fn load_audit_log(&self) -> Result<()> {
        // Load audit log from persistent storage
        // For now, start with empty log
        Ok(())
    }
}

/// Execution context for a plugin with minimal privileges
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub plugin_id: String,
    pub permission_set: MinimalPermissionSet,
    pub sandbox_constraints: SandboxConstraints,
    pub created_at: SystemTime,
    pub last_used: SystemTime,
    pub usage_count: u64,
    pub resource_limits: ResourceLimits,
}

/// Minimal permission set with only required capabilities
#[derive(Debug, Clone)]
pub struct MinimalPermissionSet {
    pub capabilities: Vec<String>,
    pub derived_permissions: HashSet<String>,
}

/// Sandbox constraints for plugin execution
#[derive(Debug, Clone)]
pub struct SandboxConstraints {
    pub file_system: FileSystemRestrictions,
    pub network: NetworkRestrictions,
    pub process: ProcessRestrictions,
    pub memory_limit_mb: u64,
    pub cpu_limit_percent: u32,
    pub execution_timeout_seconds: u64,
    pub allowed_syscalls: HashSet<String>,
}

/// Permission policy for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicy {
    pub trust_level: TrustLevel,
    pub allowed_capabilities: HashSet<String>,
    pub denied_capabilities: HashSet<String>,
    pub resource_limits: ResourceLimits,
    pub session_timeout_seconds: u64,
    pub require_user_approval: bool,
    pub audit_all_operations: bool,
    pub network_restrictions: NetworkRestrictions,
    pub file_system_restrictions: FileSystemRestrictions,
    pub process_restrictions: ProcessRestrictions,
}

/// Trust levels for plugins
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrustLevel {
    Untrusted,
    Limited,
    Trusted,
    System,
}

/// Resource limits for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: u32,
    pub max_execution_time_seconds: u64,
    pub max_file_descriptors: u32,
    pub max_network_connections: u32,
}

impl ResourceLimits {
    pub fn strict() -> Self {
        Self {
            max_memory_mb: 10,
            max_cpu_percent: 10,
            max_execution_time_seconds: 5,
            max_file_descriptors: 10,
            max_network_connections: 2,
        }
    }
    
    pub fn moderate() -> Self {
        Self {
            max_memory_mb: 50,
            max_cpu_percent: 25,
            max_execution_time_seconds: 30,
            max_file_descriptors: 50,
            max_network_connections: 10,
        }
    }
    
    pub fn relaxed() -> Self {
        Self {
            max_memory_mb: 200,
            max_cpu_percent: 50,
            max_execution_time_seconds: 300,
            max_file_descriptors: 200,
            max_network_connections: 50,
        }
    }
}

/// File system access restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSystemRestrictions {
    None,
    ReadOnly { allowed_paths: Vec<PathBuf> },
    Limited { allowed_paths: Vec<PathBuf>, denied_paths: Vec<PathBuf> },
    Full,
}

impl FileSystemRestrictions {
    pub fn none() -> Self {
        Self::None
    }
}

/// Network access restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkRestrictions {
    None,
    Limited { allowed_domains: Vec<String>, allowed_ports: Vec<u16> },
    Full,
}

impl NetworkRestrictions {
    pub fn none() -> Self {
        Self::None
    }
}

/// Process execution restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessRestrictions {
    None,
    Limited { allowed_commands: Vec<String> },
    Full,
}

impl ProcessRestrictions {
    pub fn strict() -> Self {
        Self::None
    }
}

/// Permission operations that can be checked
#[derive(Debug, Clone)]
pub enum PermissionOperation {
    FileRead { path: PathBuf },
    FileWrite { path: PathBuf },
    NetworkRequest { url: String },
    ProcessExecute { command: String },
    EnvironmentRead { variable: String },
}

/// Result of a permission check
#[derive(Debug, Clone)]
pub struct PermissionResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub timestamp: SystemTime,
}

impl PermissionResult {
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            reason: None,
            timestamp: SystemTime::now(),
        }
    }
    
    pub fn denied(reason: String) -> Self {
        Self {
            allowed: false,
            reason: Some(reason),
            timestamp: SystemTime::now(),
        }
    }
}

/// Active permission set for a plugin
#[derive(Debug, Clone)]
pub struct ActivePermissionSet {
    pub context: ExecutionContext,
    pub granted_at: SystemTime,
    pub expires_at: SystemTime,
}

/// Capability sandbox for plugin execution
#[derive(Debug, Clone)]
pub struct CapabilitySandbox {
    pub plugin_id: String,
    pub allowed_capabilities: Vec<String>,
    pub file_system_access: FileSystemSandbox,
    pub network_access: NetworkSandbox,
    pub process_access: ProcessSandbox,
    pub resource_limits: ResourceLimits,
    pub created_at: SystemTime,
}

/// File system sandbox
#[derive(Debug, Clone)]
pub struct FileSystemSandbox {
    pub restrictions: FileSystemRestrictions,
    pub temp_directory: Option<PathBuf>,
}

/// Network sandbox
#[derive(Debug, Clone)]
pub struct NetworkSandbox {
    pub restrictions: NetworkRestrictions,
    pub proxy_config: Option<String>,
}

/// Process sandbox
#[derive(Debug, Clone)]
pub struct ProcessSandbox {
    pub restrictions: ProcessRestrictions,
    pub allowed_syscalls: HashSet<String>,
}

/// Capability definition
#[derive(Debug, Clone)]
pub struct CapabilityDefinition {
    pub name: String,
    pub description: String,
    pub required_permissions: Vec<String>,
    pub risk_level: RiskLevel,
}

/// Risk levels for capabilities
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Permission audit entry
#[derive(Debug, Clone)]
pub struct PermissionAuditEntry {
    pub plugin_id: String,
    pub action: PermissionAction,
    pub details: String,
    pub timestamp: SystemTime,
}

/// Permission actions for audit logging
#[derive(Debug, Clone)]
pub enum PermissionAction {
    Granted,
    Denied,
    Revoked,
    Expired,
    Checked,
    PolicyUpdated,
}

/// Permission manager configuration
#[derive(Debug, Clone)]
pub struct PermissionConfig {
    pub max_audit_entries: usize,
    pub default_session_timeout: Duration,
    pub enable_syscall_filtering: bool,
    pub strict_capability_checking: bool,
}

impl Default for PermissionConfig {
    fn default() -> Self {
        Self {
            max_audit_entries: 10000,
            default_session_timeout: Duration::from_secs(3600),
            enable_syscall_filtering: true,
            strict_capability_checking: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_permission_manager_creation() {
        let manager = PermissionManager::new().unwrap();
        assert!(manager.permission_policies.read().await.is_empty());
    }
    
    #[tokio::test]
    async fn test_execution_context_creation() {
        let mut manager = PermissionManager::new().unwrap();
        manager.initialize().await.unwrap();
        
        let metadata = PluginMetadata {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            dependencies: HashMap::new(),
            capabilities: vec!["file_read".to_string()],
            min_nexus_version: "0.1.0".to_string(),
            max_nexus_version: None,
        };
        
        let context = manager.create_execution_context(
            "test-plugin",
            &metadata,
            &["file_read".to_string()],
        ).await.unwrap();
        
        assert_eq!(context.plugin_id, "test-plugin");
        assert!(context.permission_set.capabilities.contains(&"file_read".to_string()));
    }
    
    #[tokio::test]
    async fn test_permission_checking() {
        let mut manager = PermissionManager::new().unwrap();
        manager.initialize().await.unwrap();
        
        let metadata = PluginMetadata {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            dependencies: HashMap::new(),
            capabilities: vec!["file_read".to_string()],
            min_nexus_version: "0.1.0".to_string(),
            max_nexus_version: None,
        };
        
        // Create execution context
        manager.create_execution_context(
            "test-plugin",
            &metadata,
            &["file_read".to_string()],
        ).await.unwrap();
        
        // Check file read permission
        let operation = PermissionOperation::FileRead {
            path: PathBuf::from("/tmp/test.txt"),
        };
        
        let result = manager.check_permission("test-plugin", &operation).await.unwrap();
        assert!(!result.allowed); // Should be denied due to path restrictions
    }
    
    #[test]
    fn test_resource_limits() {
        let strict = ResourceLimits::strict();
        assert_eq!(strict.max_memory_mb, 10);
        assert_eq!(strict.max_cpu_percent, 10);
        
        let relaxed = ResourceLimits::relaxed();
        assert_eq!(relaxed.max_memory_mb, 200);
        assert_eq!(relaxed.max_cpu_percent, 50);
    }
    
    #[test]
    fn test_permission_result() {
        let allowed = PermissionResult::allowed();
        assert!(allowed.allowed);
        assert!(allowed.reason.is_none());
        
        let denied = PermissionResult::denied("Test reason".to_string());
        assert!(!denied.allowed);
        assert_eq!(denied.reason, Some("Test reason".to_string()));
    }
} 