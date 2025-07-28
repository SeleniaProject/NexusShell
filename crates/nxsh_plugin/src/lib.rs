//! WASI-based plugin support for NexusShell.
//! 
//! This module provides a comprehensive plugin system using WebAssembly and WASI,
//! with support for the Component Model, capability-based security, and dynamic loading.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

pub mod json;
pub mod registrar;
pub mod loader;
pub mod key;
pub mod remote;
pub mod runtime;
pub mod manager;
pub mod security;
pub mod component;
pub mod signature;
pub mod permissions;

use crate::runtime::WasiPluginRuntime;
use crate::manager::PluginManager;

static PLUGIN_SYSTEM: Lazy<Arc<RwLock<PluginSystem>>> =
    Lazy::new(|| Arc::new(RwLock::new(PluginSystem::new())));

/// Global plugin system state
pub struct PluginSystem {
    runtime: Option<WasiPluginRuntime>,
    manager: Option<PluginManager>,
    initialized: bool,
}

impl PluginSystem {
    fn new() -> Self {
        Self {
            runtime: None,
            manager: None,
            initialized: false,
        }
    }
    
    async fn initialize_internal(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        // Initialize runtime
        let mut runtime = WasiPluginRuntime::new()?;
        runtime.initialize().await?;
        self.runtime = Some(runtime);
        
        // Initialize manager
        let manager = PluginManager::new();
        self.manager = Some(manager);
        
        self.initialized = true;
        log::info!("Plugin system initialized successfully");
        Ok(())
    }
    
    fn runtime(&self) -> Option<&WasiPluginRuntime> {
        self.runtime.as_ref()
    }
    
    fn manager(&self) -> Option<&PluginManager> {
        self.manager.as_ref()
    }
    
    fn runtime_mut(&mut self) -> Option<&mut WasiPluginRuntime> {
        self.runtime.as_mut()
    }
    
    fn manager_mut(&mut self) -> Option<&mut PluginManager> {
        self.manager.as_mut()
    }
}

/// Initialize the global plugin system
pub async fn initialize() -> Result<()> {
    let system = PLUGIN_SYSTEM.clone();
    let mut system = system.write().await;
    system.initialize_internal().await
}

/// Get reference to the global plugin system
pub async fn get_system() -> Arc<RwLock<PluginSystem>> {
    PLUGIN_SYSTEM.clone()
}

/// Shutdown the global plugin system
pub async fn shutdown() -> Result<()> {
    let system = PLUGIN_SYSTEM.clone();
    let mut system = system.write().await;
    
    if let Some(manager) = system.manager_mut() {
        manager.unload_all_plugins().await?;
    }
    
    system.runtime = None;
    system.manager = None;
    system.initialized = false;
    
    log::info!("Plugin system shutdown complete");
    Ok(())
}

/// Load a plugin from a file path
pub async fn load_plugin<P: AsRef<std::path::Path>>(path: P) -> Result<String> {
    let system = PLUGIN_SYSTEM.clone();
    let system = system.read().await;
    
    if let Some(manager) = system.manager() {
        manager.load_plugin(path).await
    } else {
        Err(anyhow::anyhow!("Plugin system not initialized"))
    }
}

/// Unload a plugin by ID
pub async fn unload_plugin(plugin_id: &str) -> Result<()> {
    let system = PLUGIN_SYSTEM.clone();
    let system = system.read().await;
    
    if let Some(manager) = system.manager() {
        manager.unload_plugin(plugin_id).await
    } else {
        Err(anyhow::anyhow!("Plugin system not initialized"))
    }
}

/// List all loaded plugins
pub async fn list_plugins() -> Vec<String> {
    let system = PLUGIN_SYSTEM.clone();
    let system = system.read().await;
    
    if let Some(runtime) = system.runtime() {
        runtime.list_plugins().await
    } else {
        vec![]
    }
}

/// Execute a plugin function
pub async fn execute_plugin(plugin_id: &str, function: &str, args: &[u8]) -> Result<Vec<u8>> {
    let system = PLUGIN_SYSTEM.clone();
    let system = system.read().await;
    
    if let Some(runtime) = system.runtime() {
        runtime.execute_plugin(plugin_id, function, args).await
            .map_err(|e| anyhow::anyhow!("Plugin execution failed: {:?}", e))
    } else {
        Err(anyhow::anyhow!("Plugin system not initialized"))
    }
}

// Plugin configuration and metadata types
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugin_dir: String,
    pub cache_dir: String,
    pub max_concurrent_executions: Option<usize>,
    pub execution_timeout_ms: u64,
    pub max_memory_mb: u64,
    pub max_stack_size: usize,
    pub enable_multi_memory: bool,
    pub enable_threads: bool,
    pub enable_component_model: bool,
    pub security_policy: String,
    pub require_signatures: bool,
    pub enable_encryption: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            plugin_dir: "plugins".to_string(),
            cache_dir: "plugin_cache".to_string(),
            max_concurrent_executions: Some(10),
            execution_timeout_ms: 30000,
            max_memory_mb: 100,
            max_stack_size: 1024 * 1024, // 1MB
            enable_multi_memory: false,
            enable_threads: false,
            enable_component_model: true,
            security_policy: "restrictive".to_string(),
            require_signatures: true,
            enable_encryption: true,
        }
    }
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub dependencies: HashMap<String, String>,
    pub capabilities: Vec<String>,
    pub min_nexus_version: String,
    pub max_nexus_version: Option<String>,
}

/// Plugin execution result
pub type PluginResult<T> = std::result::Result<T, PluginError>;

/// Plugin events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
    Loaded { plugin_id: String, metadata: PluginMetadata },
    Unloaded { plugin_id: String },
    Executed { plugin_id: String, function: String, duration_ms: u64 },
    Error { plugin_id: String, error: String },
    SignatureVerified { plugin_id: String, key_id: String },
    SignatureVerificationFailed { plugin_id: String, reason: String },
    PermissionGranted { plugin_id: String, capability: String },
    PermissionDenied { plugin_id: String, capability: String, reason: String },
}

/// Plugin event handler trait
pub trait PluginEventHandler: Send + Sync {
    async fn handle_event(&self, event: PluginEvent) -> Result<()>;
}

/// Plugin system errors
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Plugin load error: {0}")]
    LoadError(String),
    
    #[error("Plugin execution error: {0}")]
    ExecutionError(String),
    
    #[error("Plugin security error: {0}")]
    SecurityError(String),
    
    #[error("Plugin dependency error: {0}")]
    DependencyError(String),
    
    #[error("Plugin version error: {0}")]
    VersionError(String),
    
    #[error("Plugin configuration error: {0}")]
    ConfigError(String),
    
    #[error("Plugin signature error: {0}")]
    SignatureError(String),
    
    #[error("Plugin permission error: {0}")]
    PermissionError(String),
    
    #[error("Plugin encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Plugin I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Plugin serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Security integration utilities
pub mod security_integration {
    use super::*;
    use crate::{signature::SignatureVerifier, permissions::PermissionManager};
    
    /// Integrated security manager for plugins
    pub struct IntegratedSecurityManager {
        signature_verifier: SignatureVerifier,
        permission_manager: PermissionManager,
    }
    
    impl IntegratedSecurityManager {
        pub async fn new() -> Result<Self> {
            let mut signature_verifier = SignatureVerifier::new()?;
            signature_verifier.initialize().await?;
            
            let mut permission_manager = PermissionManager::new()?;
            permission_manager.initialize().await?;
            
            Ok(Self {
                signature_verifier,
                permission_manager,
            })
        }
        
        /// Perform complete security validation of a plugin
        pub async fn validate_plugin<P: AsRef<std::path::Path>>(
            &self,
            plugin_path: P,
            metadata: &PluginMetadata,
        ) -> Result<SecurityValidationResult> {
            // Verify signature
            let signature_result = self.signature_verifier
                .verify_plugin(&plugin_path, metadata).await
                .map_err(|e| anyhow::anyhow!("Signature verification failed: {:?}", e))?;
            
            // Create execution context with minimal privileges
            let execution_context = self.permission_manager
                .create_execution_context(
                    &metadata.name,
                    metadata,
                    &metadata.capabilities,
                ).await
                .map_err(|e| anyhow::anyhow!("Permission context creation failed: {:?}", e))?;
            
            Ok(SecurityValidationResult {
                signature_valid: signature_result.valid,
                signature_key_id: signature_result.key_id,
                permission_context: execution_context,
                validation_timestamp: std::time::SystemTime::now(),
            })
        }
    }
    
    /// Result of security validation
    #[derive(Debug, Clone)]
    pub struct SecurityValidationResult {
        pub signature_valid: bool,
        pub signature_key_id: Option<String>,
        pub permission_context: crate::permissions::ExecutionContext,
        pub validation_timestamp: std::time::SystemTime,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_plugin_system_initialization() {
        let system = get_system().await;
        let system_guard = system.read().await;
        assert!(!system_guard.initialized);
    }
    
    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        assert_eq!(config.plugin_dir, "plugins");
        assert!(config.require_signatures);
        assert!(config.enable_encryption);
        assert_eq!(config.security_policy, "restrictive");
    }
    
    #[test]
    fn test_plugin_error_types() {
        let error = PluginError::NotFound("test-plugin".to_string());
        assert!(error.to_string().contains("test-plugin"));
        
        let error = PluginError::SecurityError("signature invalid".to_string());
        assert!(error.to_string().contains("security"));
    }
    
    #[tokio::test]
    async fn test_security_integration() {
        // This would require proper setup in a real test environment
        // let security_manager = security_integration::IntegratedSecurityManager::new().await;
        // assert!(security_manager.is_ok());
    }
} 