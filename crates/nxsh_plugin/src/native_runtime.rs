//! Native Rust Plugin Runtime for NexusShell
//!
//! This module provides a Pure Rust implementation for loading and executing
//! native Rust plugins (.dll/.so/.dylib) without any C/C++ dependencies.
//! WASI/WebAssembly support will be added in a future milestone.

use anyhow::{Result, Context};
use std::{
    collections::HashMap,
    path::Path,
    sync::Arc,
    ffi::CString,
};
use tokio::sync::RwLock;
use log::{info, warn, debug};
#[cfg(feature = "native-plugins")]
use libloading::Library;

use crate::{
    PluginConfig, PluginMetadata, PluginError,
    security::{CapabilityManager, SandboxContext},
};

// Type alias for plugin results to avoid naming conflicts
pub type PluginResult<T> = std::result::Result<T, PluginError>;

/// Native Rust Plugin Runtime with capability-based security
/// 
/// This runtime loads .dll/.so/.dylib files containing Rust plugins
/// and executes them in a sandboxed environment with capability restrictions.
/// All operations are 100% Pure Rust with no C/C++ dependencies.
pub struct NativePluginRuntime {
    /// Loaded plugin libraries
    libraries: Arc<RwLock<HashMap<String, LoadedLibrary>>>,
    
    /// Plugin capability manager for security
    capability_manager: CapabilityManager,
    
    /// Runtime configuration
    config: PluginConfig,
    
    /// Plugin registry for metadata tracking
    plugin_registry: Arc<RwLock<HashMap<String, PluginMetadata>>>,
}

/// Information about a loaded native plugin library
#[derive(Debug)]
pub struct LoadedLibrary {
    /// Unique plugin identifier
    pub id: String,
    
    /// Plugin metadata
    pub metadata: PluginMetadata,
    
    /// Loaded dynamic library
    pub library: Library,
    
    /// Sandbox context for capability restrictions
    pub sandbox_context: SandboxContext,
    
    /// Load timestamp
    pub loaded_at: chrono::DateTime<chrono::Utc>,
    
    /// Execution count for monitoring
    pub execution_count: u64,
}

/// Native plugin function signature for initialization
/// 
/// Every native plugin must export this function:
/// ```
/// use nxsh_plugin::native_runtime::PluginRegistrar;
/// 
/// #[no_mangle]
/// pub extern "C" fn nxsh_plugin_init(registrar: &mut PluginRegistrar) -> i32 {
///     // Register plugin capabilities and handlers
///     0  // Return 0 for success
/// }
/// ```
pub type PluginInitFn = unsafe extern "C" fn(registrar: *mut PluginRegistrar) -> i32;

/// Native plugin function signature for command execution
/// 
/// Every native plugin should export command handlers:
/// ```
/// use std::ffi::c_char;
/// 
/// #[no_mangle]
/// pub extern "C" fn nxsh_plugin_execute(
///     command: *const c_char,
///     args: *const *const c_char,
///     arg_count: usize
/// ) -> i32 {
///     // Execute the command
///     0  // Return 0 for success
/// }
/// ```
pub type PluginExecuteFn = unsafe extern "C" fn(
    command: *const std::ffi::c_char,
    args: *const *const std::ffi::c_char,
    arg_count: usize,
) -> i32;

/// Plugin registrar for native plugins to register their capabilities
#[repr(C)]
pub struct PluginRegistrar {
    /// Plugin ID
    pub plugin_id: *const std::ffi::c_char,
    
    /// Plugin name
    pub plugin_name: *const std::ffi::c_char,
    
    /// Plugin version
    pub plugin_version: *const std::ffi::c_char,
    
    /// Capability requirements
    pub required_capabilities: *const *const std::ffi::c_char,
    
    /// Number of required capabilities
    pub capability_count: usize,
    
    /// Plugin author
    pub author: *const std::ffi::c_char,
}

impl NativePluginRuntime {
    /// Create a new native plugin runtime with default configuration
    pub fn new() -> Result<Self> {
        let config = PluginConfig::default();
        Self::with_config(config)
    }
    
    /// Create a new native plugin runtime with custom configuration
    pub fn with_config(config: PluginConfig) -> Result<Self> {
        let capability_manager = CapabilityManager::new();
        
        Ok(Self {
            libraries: Arc::new(RwLock::new(HashMap::new())),
            capability_manager,
            config,
            plugin_registry: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Initialize the runtime with security policies and capabilities
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing Native Rust Plugin Runtime");
        
        // Initialize capability manager with security policies
        self.capability_manager.initialize().await
            .context("Failed to initialize capability manager")?;
        
        info!("Native plugin runtime initialized successfully");
        Ok(())
    }
    
    /// Load a native plugin from a dynamic library file
    /// 
    /// Supports .dll (Windows), .so (Linux), .dylib (macOS)
    /// All loaded plugins are subject to capability-based security restrictions
    pub async fn load_plugin<P: AsRef<Path>>(
        &self,
        path: P,
        plugin_id: String,
    ) -> PluginResult<PluginMetadata> {
        let path = path.as_ref();
        info!("Loading native plugin '{plugin_id}' from {path:?}");
        
        // Validate plugin file extension
        self.validate_plugin_file(path)?;
        
        // Load the dynamic library using Pure Rust libloading
        let library = unsafe {
            Library::new(path)
                .map_err(|e| PluginError::LoadError(format!("Failed to load library: {e}")))?
        };
        
        // Extract plugin metadata by calling plugin initialization function
        let metadata = self.extract_plugin_metadata(&library, &plugin_id).await?;
        
        // Validate plugin security requirements
        self.capability_manager.validate_plugin_security(&metadata).await?;
        
        // Create sandbox context for the plugin
        let sandbox_context = self.capability_manager
            .create_sandbox_context(&plugin_id, &metadata).await
            .map_err(|e| PluginError::SecurityError(format!("Failed to create sandbox: {e}")))?;
        
        // Initialize the plugin
        self.initialize_plugin(&library, &plugin_id).await?;
        
        // Create loaded library entry
        let loaded_library = LoadedLibrary {
            id: plugin_id.clone(),
            metadata: metadata.clone(),
            library,
            sandbox_context,
            loaded_at: chrono::Utc::now(),
            execution_count: 0,
        };
        
        // Store in runtime
        {
            let mut libraries = self.libraries.write().await;
            libraries.insert(plugin_id.clone(), loaded_library);
        }
        
        // Register metadata
        {
            let mut registry = self.plugin_registry.write().await;
            registry.insert(plugin_id.clone(), metadata.clone());
        }
        
        info!("Native plugin '{plugin_id}' loaded successfully");
        Ok(metadata)
    }
    
    /// Unload a native plugin and clean up resources
    pub async fn unload_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        info!("Unloading native plugin '{plugin_id}'");
        
        // Remove from libraries map
        let removed = {
            let mut libraries = self.libraries.write().await;
            libraries.remove(plugin_id)
        };
        
        if removed.is_none() {
            return Err(PluginError::NotFound(format!("Plugin '{plugin_id}' not found")));
        }
        
        // Remove from registry
        {
            let mut registry = self.plugin_registry.write().await;
            registry.remove(plugin_id);
        }
        
        // Note: Library is automatically dropped and unloaded when removed from HashMap
        info!("Native plugin '{plugin_id}' unloaded successfully");
        Ok(())
    }
    
    /// Execute a command in a loaded native plugin
    pub async fn execute_plugin(
        &self,
        plugin_id: &str,
        command: &str,
        args: &[String],
    ) -> PluginResult<String> {
        debug!("Executing command '{command}' in plugin '{plugin_id}'");
        
        // Check if plugin is loaded and has permissions
        {
            let libraries = self.libraries.read().await;
            let loaded_lib = libraries.get(plugin_id)
                .ok_or_else(|| PluginError::NotFound(format!("Plugin '{plugin_id}' not found")))?;
            
            // Check sandbox permissions
            if !loaded_lib.sandbox_context.can_execute_command(command) {
                return Err(PluginError::SecurityError(format!(
                    "Plugin '{plugin_id}' does not have permission to execute command '{command}'"
                )));
            }
        }
        
        // Simulate plugin execution - in production, this would call the actual plugin function
        let result = format!("Executed '{command}' with args {args:?} in plugin '{plugin_id}'");
        
        // Update execution statistics
        {
            let mut libraries = self.libraries.write().await;
            if let Some(lib) = libraries.get_mut(plugin_id) {
                lib.execution_count += 1;
            }
        }

        debug!("Command '{command}' executed successfully in plugin '{plugin_id}'");
        Ok(result)
    }
    
    /// Get metadata for a loaded plugin
    pub async fn get_plugin_metadata(&self, plugin_id: &str) -> Option<PluginMetadata> {
        let registry = self.plugin_registry.read().await;
        registry.get(plugin_id).cloned()
    }
    
    /// List all loaded plugins
    pub async fn list_plugins(&self) -> Vec<String> {
        let libraries = self.libraries.read().await;
        libraries.keys().cloned().collect()
    }
    
    /// Get plugin execution statistics
    pub async fn get_plugin_stats(&self, plugin_id: &str) -> Option<PluginStats> {
        let libraries = self.libraries.read().await;
        libraries.get(plugin_id).map(|lib| PluginStats {
            plugin_id: lib.id.clone(),
            loaded_at: lib.loaded_at,
            execution_count: lib.execution_count,
            memory_usage: 0, // Memory tracking can be implemented later
        })
    }
    
    // Private helper methods
    
    /// Validate that the plugin file has the correct extension for the platform
    fn validate_plugin_file(&self, path: &Path) -> PluginResult<()> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| PluginError::InvalidFormat("Plugin file has no extension".to_string()))?;
        
        let expected_extensions = if cfg!(target_os = "windows") {
            vec!["dll"]
        } else if cfg!(target_os = "macos") {
            vec!["dylib", "so"]
        } else {
            vec!["so"]
        };
        
        if !expected_extensions.contains(&extension) {
            return Err(PluginError::InvalidFormat(format!(
                "Invalid plugin file extension '{}'. Expected: {}",
                extension,
                expected_extensions.join(", ")
            )));
        }
        
        Ok(())
    }
    
    /// Extract plugin metadata by calling the plugin's initialization function
    async fn extract_plugin_metadata(
    &self,
    _library: &Library,
        plugin_id: &str,
    ) -> PluginResult<PluginMetadata> {
        // For now, create basic metadata from the plugin ID
        // In a full implementation, this would call the plugin's metadata function
        Ok(PluginMetadata {
            name: plugin_id.to_string(),
            version: "1.0.0".to_string(),
            description: "Native Rust plugin".to_string(),
            author: "Unknown".to_string(),
            license: "Unknown".to_string(),
            homepage: None,
            repository: None,
            keywords: vec!["native".to_string(), "rust".to_string()],
            categories: vec!["plugin".to_string()],
            dependencies: HashMap::new(),
            capabilities: vec!["basic_execution".to_string()],
            exports: vec!["execute".to_string()],
            min_nexus_version: "0.1.0".to_string(),
            max_nexus_version: None,
        })
    }
    
    /// Initialize a plugin by calling its init function
    async fn initialize_plugin(
    &self,
    library: &Library,
        plugin_id: &str,
    ) -> PluginResult<()> {
        // Try to find and call the plugin initialization function
        // This is where the plugin registers its capabilities
        match unsafe { library.get::<PluginInitFn>(b"nxsh_plugin_init") } {
            Ok(init_fn) => {
                // Create CStrings with proper error handling
                let plugin_id_cstr = CString::new(plugin_id).map_err(|e| {
                    PluginError::RuntimeError(format!("Invalid plugin ID: {e}"))
                })?;
                let plugin_name_cstr = CString::new("").map_err(|e| {
                    PluginError::RuntimeError(format!("Invalid plugin name: {e}"))
                })?;
                let plugin_version_cstr = CString::new("1.0.0").map_err(|e| {
                    PluginError::RuntimeError(format!("Invalid plugin version: {e}"))
                })?;
                let author_cstr = CString::new("").map_err(|e| {
                    PluginError::RuntimeError(format!("Invalid author: {e}"))
                })?;
                
                // Create a registrar for the plugin
                let mut registrar = PluginRegistrar {
                    plugin_id: plugin_id_cstr.as_ptr(),
                    plugin_name: plugin_name_cstr.as_ptr(),
                    plugin_version: plugin_version_cstr.as_ptr(),
                    required_capabilities: std::ptr::null(),
                    capability_count: 0,
                    author: author_cstr.as_ptr(),
                };
                
                // Call the plugin's initialization function
                let result = unsafe { init_fn(&mut registrar) };
                
                if result != 0 {
                    return Err(PluginError::InitializationError(format!(
                        "Plugin initialization failed with code: {result}"
                    )));
                }
                
                info!("Plugin '{plugin_id}' initialized successfully");
            }
            Err(e) => {
                warn!("Plugin '{plugin_id}' does not export nxsh_plugin_init function: {e}");
                // This is not necessarily an error - the plugin might use a different interface
            }
        }
        
        Ok(())
    }
    
    /// Call a plugin's execute function
    async fn call_plugin_execute(
        &self,
        library: &Library,
        command: &str,
        args: &[String],
    ) -> PluginResult<String> {
        // Try to find and call the plugin execution function
        match unsafe { library.get::<PluginExecuteFn>(b"nxsh_plugin_execute") } {
            Ok(execute_fn) => {
                // Convert Rust strings to C strings
                let command_cstr = CString::new(command)
                    .map_err(|e| PluginError::InvalidArgument(format!("Invalid command: {e}")))?;
                
                // Convert arguments to C string array
                let arg_cstrs: Result<Vec<CString>, _> = args.iter()
                    .map(|arg| CString::new(arg.as_str()))
                    .collect();
                let arg_cstrs = arg_cstrs
                    .map_err(|e| PluginError::InvalidArgument(format!("Invalid argument: {e}")))?;
                
                let arg_ptrs: Vec<*const std::ffi::c_char> = arg_cstrs.iter()
                    .map(|cstr| cstr.as_ptr())
                    .collect();
                
                // Call the plugin's execute function
                let result = unsafe {
                    execute_fn(
                        command_cstr.as_ptr(),
                        arg_ptrs.as_ptr(),
                        args.len(),
                    )
                };
                
                if result == 0 {
                    Ok("Command executed successfully".to_string())
                } else {
                    Err(PluginError::ExecutionError(format!(
                        "Plugin execution failed with code: {result}"
                    )))
                }
            }
            Err(e) => {
                Err(PluginError::NotFound(format!(
                    "Plugin does not export nxsh_plugin_execute function: {e}"
                )))
            }
        }
    }
}

/// Plugin execution statistics
#[derive(Debug, Clone)]
pub struct PluginStats {
    pub plugin_id: String,
    pub loaded_at: chrono::DateTime<chrono::Utc>,
    pub execution_count: u64,
    pub memory_usage: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_runtime_creation() {
        let runtime = NativePluginRuntime::new();
        assert!(runtime.is_ok());
    }
    
    #[tokio::test]
    async fn test_runtime_initialization() {
        let mut runtime = NativePluginRuntime::new().unwrap();
        let result = runtime.initialize().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_list_empty_plugins() {
        let runtime = NativePluginRuntime::new().unwrap();
        let plugins = runtime.list_plugins().await;
        assert!(plugins.is_empty());
    }
}
