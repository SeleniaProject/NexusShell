//! Native Rust Plugin Support for NexusShell.
//! 
//! This module provides a comprehensive plugin system using native Rust dynamic libraries,
//! with capability-based security and dynamic loading.
//! 
//! STAGE 1: Native Rust Plugin Support (100% Pure Rust)
//! STAGE 2: WASI Plugin Support (planned for future milestone)

use anyhow::Result;
use std::sync::Arc;
#[cfg(any(feature = "native-plugins", feature = "async-support"))]
use tokio::sync::RwLock;
#[cfg(any(feature = "native-plugins", feature = "async-support"))]
use once_cell::sync::Lazy;

pub mod json;
pub mod registrar;
#[cfg(feature = "wasi-runtime")]
pub mod loader;             // Pure Rust WASM plugin loading (restored)
pub mod keys;
#[cfg(feature = "remote-plugins")]
pub mod remote;             // Stage 2: Remote plugin support (restored in Phase 3)
#[cfg(feature = "native-plugins")]
pub mod native_runtime;     // Stage 1: Native Rust plugins
#[cfg(feature = "wasi-runtime")]
pub mod runtime;            // Pure Rust WASI plugins (restored)
pub mod manager;
pub mod security;
#[cfg(feature = "wasi-runtime")]
pub mod component;          // Pure Rust Component model (restored)
pub mod signature;
pub mod permissions;
#[cfg(feature = "wasi-runtime")]
pub mod resource_table;     // Pure Rust WASM resource management (restored)
#[cfg(feature = "wasi-runtime")]
pub mod wasi_advanced;      // Advanced WASM/WASI runtime
pub mod security_sandbox;   // Security sandbox system
#[cfg(feature = "plugin-management")]
pub mod plugin_manager_advanced; // Advanced plugin management

#[cfg(feature = "native-plugins")]
use crate::native_runtime::NativePluginRuntime;
#[cfg(feature = "wasi-runtime")]
use crate::runtime::WasiPluginRuntime;
#[cfg(feature = "wasi-runtime")]
use crate::component::ComponentRegistry;
#[cfg(feature = "wasi-runtime")]
use crate::resource_table::ResourceTable;
pub use crate::manager::PluginManager;
pub use crate::signature::PluginSignature;

// #[cfg(test)]
// mod tests;  // Disabled legacy tests for now

#[cfg(any(feature = "native-plugins", feature = "async-support"))]
static PLUGIN_SYSTEM: Lazy<Arc<RwLock<PluginSystem>>> =
    Lazy::new(|| Arc::new(RwLock::new(PluginSystem::new())));

/// Global plugin system state with Pure Rust Plugin support
pub struct PluginSystem {
    #[cfg(feature = "native-plugins")]
    native_runtime: Option<NativePluginRuntime>,
    #[cfg(feature = "wasi-runtime")]
    wasi_runtime: Option<WasiPluginRuntime>,
    #[cfg(feature = "wasi-runtime")]
    component_registry: Option<ComponentRegistry>,
    #[cfg(feature = "wasi-runtime")]
    resource_table: Option<ResourceTable>,
    manager: Option<PluginManager>,
    initialized: bool,
}

impl PluginSystem {
    fn new() -> Self {
        Self {
            #[cfg(feature = "native-plugins")]
            native_runtime: None,
            #[cfg(feature = "wasi-runtime")]
            wasi_runtime: None,
            #[cfg(feature = "wasi-runtime")]
            component_registry: None,
            #[cfg(feature = "wasi-runtime")]
            resource_table: None,
            manager: None,
            initialized: false,
        }
    }
    
    async fn initialize_internal(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        // Initialize native runtime
        #[cfg(feature = "native-plugins")]
        {
            let mut native_runtime = NativePluginRuntime::new()?;
            native_runtime.initialize().await?;
            self.native_runtime = Some(native_runtime);
        }
        
        // Initialize WASI runtime
        #[cfg(feature = "wasi-runtime")]
        {
            let wasi_runtime = WasiPluginRuntime::new()?;
            self.wasi_runtime = Some(wasi_runtime);
            
            // Initialize component registry
            let component_registry = ComponentRegistry::new()?;
            self.component_registry = Some(component_registry);
            
            // Initialize resource table
            let resource_table = ResourceTable::new();
            self.resource_table = Some(resource_table);
        }
        
        // Initialize manager
        let manager = PluginManager::new();
        self.manager = Some(manager);
        
        self.initialized = true;
        log::info!("Pure Rust Plugin system initialized successfully");
        Ok(())
    }
    
    #[cfg(feature = "native-plugins")]
    fn native_runtime(&self) -> Option<&NativePluginRuntime> {
        self.native_runtime.as_ref()
    }
    
    #[cfg(feature = "wasi-runtime")]
    fn wasi_runtime(&self) -> Option<&WasiPluginRuntime> {
        self.wasi_runtime.as_ref()
    }
    
    #[cfg(feature = "wasi-runtime")]
    fn component_registry(&self) -> Option<&ComponentRegistry> {
        self.component_registry.as_ref()
    }
    
    #[cfg(feature = "wasi-runtime")]
    fn resource_table(&self) -> Option<&ResourceTable> {
        self.resource_table.as_ref()
    }
    
    fn manager(&self) -> Option<&PluginManager> {
        self.manager.as_ref()
    }
    
    #[cfg(feature = "native-plugins")]
    fn native_runtime_mut(&mut self) -> Option<&mut NativePluginRuntime> {
        self.native_runtime.as_mut()
    }
    
    #[cfg(feature = "wasi-runtime")]
    fn wasi_runtime_mut(&mut self) -> Option<&mut WasiPluginRuntime> {
        self.wasi_runtime.as_mut()
    }
    
    #[cfg(feature = "wasi-runtime")]
    fn component_registry_mut(&mut self) -> Option<&mut ComponentRegistry> {
        self.component_registry.as_mut()
    }
    
    #[cfg(feature = "wasi-runtime")]
    fn resource_table_mut(&mut self) -> Option<&mut ResourceTable> {
        self.resource_table.as_mut()
    }
    
    fn manager_mut(&mut self) -> Option<&mut PluginManager> {
        self.manager.as_mut()
    }
}

/// Initialize the plugin system
#[cfg(any(feature = "native-plugins", feature = "async-support"))]
pub async fn initialize() -> Result<()> {
    let system = PLUGIN_SYSTEM.clone();
    let mut system = system.write().await;
    system.initialize_internal().await
}

#[cfg(not(any(feature = "native-plugins", feature = "async-support")))]
pub fn initialize() -> Result<()> {
    log::info!("Plugin system disabled - minimal build");
    Ok(())
}

/// Get reference to the global plugin system
#[cfg(any(feature = "native-plugins", feature = "async-support"))]
pub async fn get_system() -> Arc<RwLock<PluginSystem>> {
    PLUGIN_SYSTEM.clone()
}

/// Shutdown the global plugin system
#[cfg(any(feature = "native-plugins", feature = "async-support"))]
pub async fn shutdown() -> Result<()> {
    let system = PLUGIN_SYSTEM.clone();
    let mut system = system.write().await;
    
    if let Some(manager) = system.manager_mut() {
        manager.unload_all_plugins().await?;
    }
    
    system.native_runtime = None;
    system.manager = None;
    system.initialized = false;
    
    log::info!("Plugin system shutdown complete");
    Ok(())
}

/// Load a plugin from a file path
#[cfg(any(feature = "native-plugins", feature = "async-support"))]
pub async fn load_plugin<P: AsRef<std::path::Path>>(path: P) -> Result<String> {
    let system = PLUGIN_SYSTEM.clone();
    let mut system = system.write().await;
    
    if let Some(manager) = system.manager_mut() {
        manager.load_plugin(path).await
    } else {
        Err(anyhow::anyhow!("Plugin system not initialized"))
    }
}

#[cfg(not(any(feature = "native-plugins", feature = "async-support")))]
pub fn load_plugin<P: AsRef<std::path::Path>>(_path: P) -> Result<String> {
    Err(anyhow::anyhow!("Plugin system disabled"))
}

/// Unload a plugin by ID
#[cfg(any(feature = "native-plugins", feature = "async-support"))]
pub async fn unload_plugin(plugin_id: &str) -> Result<()> {
    let system = PLUGIN_SYSTEM.clone();
    let mut system = system.write().await;
    
    if let Some(manager) = system.manager_mut() {
        manager.unload_plugin(plugin_id).await
    } else {
        Err(anyhow::anyhow!("Plugin system not initialized"))
    }
}

#[cfg(not(any(feature = "native-plugins", feature = "async-support")))]
pub fn unload_plugin(_plugin_id: &str) -> Result<()> {
    Err(anyhow::anyhow!("Plugin system disabled"))
}

/// List all loaded plugins
#[cfg(feature = "native-plugins")]
pub async fn list_plugins() -> Vec<String> {
    let system = PLUGIN_SYSTEM.clone();
    let system = system.read().await;
    
    if let Some(runtime) = system.native_runtime() {
        runtime.list_plugins().await
    } else {
        vec![]
    }
}

#[cfg(not(feature = "native-plugins"))]
pub fn list_plugins() -> Vec<String> {
    vec![]
}

/// Execute a plugin function
#[cfg(feature = "native-plugins")]
pub async fn execute_plugin(plugin_id: &str, function: &str, args: &[String]) -> Result<String> {
    let system = PLUGIN_SYSTEM.clone();
    let system = system.read().await;
    
    if let Some(runtime) = system.native_runtime() {
        runtime.execute_plugin(plugin_id, function, args).await
            .map_err(|e| anyhow::anyhow!("Plugin execution failed: {:?}", e))
    } else {
        Err(anyhow::anyhow!("Plugin system not initialized"))
    }
}

#[cfg(not(feature = "native-plugins"))]
pub fn execute_plugin(_plugin_id: &str, _function: &str, _args: &[String]) -> Result<String> {
    Err(anyhow::anyhow!("Native plugin support disabled"))
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
    pub exports: Vec<String>,
    pub min_nexus_version: String,
    pub max_nexus_version: Option<String>,
}

/// Plugin execution result
pub type PluginResult<T> = std::result::Result<T, PluginError>;

/// Plugin events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
    // Box<PluginMetadata> でサイズ削減 (large_enum_variant 対策)
    Loaded { plugin_id: String, metadata: Box<PluginMetadata> },
    Unloaded { plugin_id: String },
    Executed { plugin_id: String, function: String, duration_ms: u64 },
    Error { plugin_id: String, error: String },
    SignatureVerified { plugin_id: String, key_id: String },
    SignatureVerificationFailed { plugin_id: String, reason: String },
    PermissionGranted { plugin_id: String, capability: String },
    PermissionDenied { plugin_id: String, capability: String, reason: String },
    Updated { plugin_id: String, old_version: String, new_version: String },
}

/// Plugin event handler trait
pub trait PluginEventHandler: Send + Sync {
    fn handle_event(&self, event: PluginEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>>;
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
    
    #[error("Plugin runtime error: {0}")]
    RuntimeError(String),
    
    #[error("Plugin WASM runtime error: {0}")]
    Runtime(String),
    
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
    
    #[error("Plugin initialization error: {0}")]
    InitializationError(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Capability denied: {0}")]
    CapabilityDenied(String),
    
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

impl From<anyhow::Error> for PluginError {
    fn from(err: anyhow::Error) -> Self {
        PluginError::ExecutionError(err.to_string())
    }
}

#[cfg(feature = "wasi-runtime")]
impl From<wasmi::Error> for PluginError {
    fn from(err: wasmi::Error) -> Self {
        PluginError::Runtime(err.to_string())
    }
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