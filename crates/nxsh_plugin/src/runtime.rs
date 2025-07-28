//! WASI Plugin Runtime for NexusShell
//! 
//! This module provides a high-performance, secure runtime for executing WASI plugins
//! with support for the WebAssembly Component Model, async execution, and capability-based security.

use anyhow::{Result, Context};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::{RwLock, Semaphore};
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};
use log::{info, warn, error};

use crate::{
    PluginConfig, PluginMetadata, PluginError,
    security::{CapabilityManager, SandboxContext},
    component::{ComponentRegistry, ComponentState, ComponentValue, RegisteredComponent},
};

// Type alias for plugin results to avoid naming conflicts
pub type PluginResult2<T> = std::result::Result<T, PluginError>;

/// WASI Plugin Runtime with WebAssembly Component Model support
pub struct WasiPluginRuntime {
    engine: Engine,
    linker: Arc<RwLock<Linker<PluginState>>>,
    plugins: Arc<RwLock<HashMap<String, LoadedPlugin>>>,
    capability_manager: CapabilityManager,
    component_registry: ComponentRegistry,
    config: PluginConfig,
    execution_semaphore: Arc<Semaphore>,
}

impl WasiPluginRuntime {
    /// Create a new WASI plugin runtime with default configuration
    pub fn new() -> Result<Self> {
        let config = PluginConfig::default();
        Self::with_config(config)
    }
    
    /// Create a new WASI plugin runtime with custom configuration
    pub fn with_config(config: PluginConfig) -> Result<Self> {
        // Configure WebAssembly engine
        let mut engine_config = Config::new();
        engine_config.wasm_component_model(true);
        engine_config.async_support(true);
        engine_config.epoch_interruption(true);
        engine_config.consume_fuel(true);
        engine_config.max_wasm_stack(config.max_stack_size);
        
        // Security configurations
        if config.enable_multi_memory {
            engine_config.wasm_multi_memory(true);
        }
        if config.enable_threads {
            engine_config.wasm_threads(true);
        }
        
        let engine = Engine::new(&engine_config)
            .context("Failed to create WebAssembly engine")?;
        
        let linker = Linker::new(&engine);
        
        let capability_manager = CapabilityManager::new();
        let component_registry = ComponentRegistry::new()
            .context("Failed to create component registry")?;
        
        let max_concurrent_executions = config.max_concurrent_executions.unwrap_or(10);
        let execution_semaphore = Arc::new(Semaphore::new(max_concurrent_executions));
        
        Ok(Self {
            engine,
            linker: Arc::new(RwLock::new(linker)),
            plugins: Arc::new(RwLock::new(HashMap::new())),
            capability_manager,
            component_registry,
            config,
            execution_semaphore,
        })
    }
    
    /// Initialize the runtime with host functions and capabilities
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing WASI plugin runtime");
        
        // Initialize capability manager
        self.capability_manager.initialize().await
            .context("Failed to initialize capability manager")?;
        
        // Initialize component registry
        let mut component_registry = self.component_registry.clone();
        component_registry.initialize().await
            .context("Failed to initialize component registry")?;
        
        // Setup host functions
        self.setup_host_functions().await
            .context("Failed to setup host functions")?;
        
        info!("WASI plugin runtime initialized successfully");
        Ok(())
    }
    
    /// Load a plugin from a WebAssembly file
    pub async fn load_plugin<P: AsRef<Path>>(
        &self,
        path: P,
        plugin_id: String,
    ) -> PluginResult2<PluginMetadata> {
        let path = path.as_ref();
        info!("Loading plugin '{}' from {:?}", plugin_id, path);
        
        // Read plugin file
        let plugin_bytes = tokio::fs::read(path).await
            .map_err(|e| PluginError::LoadError(format!("Failed to read plugin file: {}", e)))?;
        
        // Parse plugin metadata
        let metadata = self.extract_plugin_metadata(&plugin_bytes, path).await?;
        
        // Validate plugin security
        self.capability_manager.validate_plugin(&metadata).await?;
        
        // Create sandbox context
        let sandbox_context = self.capability_manager
            .create_sandbox_context(&plugin_id, &metadata).await
            .map_err(|e| PluginError::SecurityError(format!("Failed to create sandbox: {}", e)))?;
        
        // Load as WebAssembly component
        let component = Component::from_binary(&self.engine, &plugin_bytes)
            .map_err(|e| PluginError::LoadError(format!("Invalid WebAssembly component: {}", e)))?;
        
        // Register component
        self.component_registry.register_component(
            plugin_id.clone(),
            path,
            metadata.clone(),
        ).await?;
        
        // Create loaded plugin entry
        let loaded_plugin = LoadedPlugin {
            id: plugin_id.clone(),
            metadata: metadata.clone(),
            component,
            sandbox_context,
            loaded_at: chrono::Utc::now(),
            execution_count: 0,
        };
        
        // Store in runtime
        {
            let mut plugins = self.plugins.write().await;
            plugins.insert(plugin_id.clone(), loaded_plugin);
        }
        
        info!("Plugin '{}' loaded successfully", plugin_id);
        Ok(metadata)
    }
    
    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &str) -> PluginResult2<()> {
        info!("Unloading plugin '{}'", plugin_id);
        
        // Remove from component registry
        self.component_registry.unregister_component(plugin_id).await?;
        
        // Remove from runtime
        {
            let mut plugins = self.plugins.write().await;
            plugins.remove(plugin_id)
                .ok_or_else(|| PluginError::NotFound(format!("Plugin '{}' not found", plugin_id)))?;
        }
        
        info!("Plugin '{}' unloaded successfully", plugin_id);
        Ok(())
    }
    
    /// Execute a function in a loaded plugin
    pub async fn execute_plugin(
        &self,
        plugin_id: &str,
        function: &str,
        args: &[u8],
    ) -> PluginResult2<Vec<u8>> {
        // Acquire execution permit
        let _permit = self.execution_semaphore.acquire().await
            .map_err(|e| PluginError::ExecutionError(format!("Failed to acquire execution permit: {}", e)))?;
        
        info!("Executing function '{}' in plugin '{}'", function, plugin_id);
        
        // Get plugin
        let plugin = {
            let plugins = self.plugins.read().await;
            plugins.get(plugin_id)
                .ok_or_else(|| PluginError::NotFound(format!("Plugin '{}' not found", plugin_id)))?
                .clone()
        };
        
        // Convert args to component values
        let component_args = vec![ComponentValue::string(String::from_utf8_lossy(args))];
        
        // Execute function
        let start_time = std::time::Instant::now();
        let result = tokio::time::timeout(
            Duration::from_millis(self.config.execution_timeout_ms),
            self.component_registry.execute_component_function(plugin_id, function, &component_args)
        ).await
            .map_err(|_| PluginError::ExecutionError("Plugin execution timeout".to_string()))?;
        
        let execution_time = start_time.elapsed();
        
        match result {
            Ok(component_results) => {
                // Update execution statistics
                {
                    let mut plugins = self.plugins.write().await;
                    if let Some(plugin) = plugins.get_mut(plugin_id) {
                        plugin.execution_count += 1;
                    }
                }
                
                // Convert results back to bytes
                let result_bytes = if let Some(ComponentValue::String(s)) = component_results.first() {
                    s.as_bytes().to_vec()
                } else {
                    serde_json::to_vec(&component_results)
                        .map_err(|e| PluginError::ExecutionError(format!("Failed to serialize results: {}", e)))?
                };
                
                info!("Plugin '{}' function '{}' executed successfully in {:?}", 
                      plugin_id, function, execution_time);
                Ok(result_bytes)
            }
            Err(e) => {
                error!("Plugin '{}' function '{}' execution failed: {:?}", plugin_id, function, e);
                Err(e)
            }
        }
    }
    
    /// Get metadata for a loaded plugin
    pub async fn get_plugin_metadata(&self, plugin_id: &str) -> Option<PluginMetadata> {
        let plugins = self.plugins.read().await;
        plugins.get(plugin_id).map(|p| p.metadata.clone())
    }
    
    /// List all loaded plugins
    pub async fn list_plugins(&self) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins.keys().cloned().collect()
    }
    
    /// Get plugin execution statistics
    pub async fn get_plugin_stats(&self, plugin_id: &str) -> Option<PluginStats> {
        let plugins = self.plugins.read().await;
        plugins.get(plugin_id).map(|p| PluginStats {
            plugin_id: p.id.clone(),
            loaded_at: p.loaded_at,
            execution_count: p.execution_count,
            memory_usage: 0, // TODO: Implement memory tracking
        })
    }
    
    /// Get runtime configuration
    pub fn config(&self) -> &PluginConfig {
        &self.config
    }
    
    /// Update runtime configuration
    pub async fn update_config(&mut self, config: PluginConfig) -> Result<()> {
        self.config = config;
        // TODO: Apply configuration changes to running plugins
        Ok(())
    }
    
    // Private helper methods
    
    async fn setup_host_functions(&self) -> Result<()> {
        let mut linker = self.linker.write().await;
        
        // Add WASI host functions
        wasmtime_wasi::add_to_linker_async(&mut linker)
            .context("Failed to add WASI host functions")?;
        
        // Add custom NexusShell host functions
        self.add_nexus_host_functions(&mut linker).await?;
        
        Ok(())
    }
    
    async fn add_nexus_host_functions(
        &self,
        linker: &mut Linker<PluginState>,
    ) -> Result<()> {
        // Shell command execution
        linker.func_wrap_async(
            "nexus:shell/command",
            "execute",
            |mut caller: wasmtime::Caller<'_, PluginState>, cmd: String| {
                Box::new(async move {
                    // Check if command execution is allowed
                    let state = caller.data();
                    if !state.sandbox_context.can_execute_commands() {
                        return Err(wasmtime::Error::msg("Command execution not allowed"));
                    }
                    
                    let output = tokio::process::Command::new("sh")
                        .arg("-c")
                        .arg(&cmd)
                        .output()
                        .await
                        .map_err(|e| wasmtime::Error::msg(format!("Command execution failed: {}", e)))?;
                    
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                })
            },
        )?;
        
        // File system operations
        linker.func_wrap_async(
            "nexus:shell/fs",
            "read-file",
            |mut caller: wasmtime::Caller<'_, PluginState>, path: String| {
                Box::new(async move {
                    let state = caller.data();
                    if !state.sandbox_context.can_read_file(&path) {
                        return Err(wasmtime::Error::msg("File read access denied"));
                    }
                    
                    let content = tokio::fs::read_to_string(&path).await
                        .map_err(|e| wasmtime::Error::msg(format!("Failed to read file: {}", e)))?;
                    Ok(content)
                })
            },
        )?;
        
        linker.func_wrap_async(
            "nexus:shell/fs",
            "write-file",
            |mut caller: wasmtime::Caller<'_, PluginState>, path: String, content: String| {
                Box::new(async move {
                    let state = caller.data();
                    if !state.sandbox_context.can_write_file(&path) {
                        return Err(wasmtime::Error::msg("File write access denied"));
                    }
                    
                    tokio::fs::write(&path, &content).await
                        .map_err(|e| wasmtime::Error::msg(format!("Failed to write file: {}", e)))?;
                    Ok(())
                })
            },
        )?;
        
        // Environment variable access
        linker.func_wrap(
            "nexus:shell/env",
            "get-var",
            |mut caller: wasmtime::Caller<'_, PluginState>, name: String| {
                let state = caller.data();
                if !state.sandbox_context.can_access_env_var(&name) {
                    return Err(wasmtime::Error::msg("Environment variable access denied"));
                }
                
                Ok(std::env::var(&name).unwrap_or_default())
            },
        )?;
        
        // Network operations
        linker.func_wrap_async(
            "nexus:shell/net",
            "http-request",
            |mut caller: wasmtime::Caller<'_, PluginState>, url: String| {
                Box::new(async move {
                    let state = caller.data();
                    if !state.sandbox_context.can_make_network_request(&url) {
                        return Err(wasmtime::Error::msg("Network access denied"));
                    }
                    
                    let response = reqwest::get(&url).await
                        .map_err(|e| wasmtime::Error::msg(format!("HTTP request failed: {}", e)))?;
                    
                    let text = response.text().await
                        .map_err(|e| wasmtime::Error::msg(format!("Failed to read response: {}", e)))?;
                    
                    Ok(text)
                })
            },
        )?;
        
        Ok(())
    }
    
    async fn extract_plugin_metadata(
        &self,
        plugin_bytes: &[u8],
        path: &Path,
    ) -> PluginResult2<PluginMetadata> {
        // Try to extract metadata from WebAssembly custom sections
        // For now, we'll create basic metadata from the file
        Ok(PluginMetadata {
            name: path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            version: "0.1.0".to_string(),
            description: "WebAssembly plugin".to_string(),
            author: "Unknown".to_string(),
            license: "Unknown".to_string(),
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            dependencies: HashMap::new(),
            capabilities: vec![],
            min_nexus_version: "0.1.0".to_string(),
            max_nexus_version: None,
        })
    }
}

/// Plugin execution state with WASI context
pub struct PluginState {
    wasi_ctx: WasiCtx,
    sandbox_context: SandboxContext,
}

impl PluginState {
    pub fn new(sandbox_context: SandboxContext) -> Result<Self> {
        let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_env()
            .build();
        
        Ok(Self {
            wasi_ctx,
            sandbox_context,
        })
    }
}

impl WasiView for PluginState {
    fn ctx(&self) -> &WasiCtx {
        &self.wasi_ctx
    }
    
    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
    
    fn table(&self) -> &wasmtime::component::ResourceTable {
        // TODO: Implement resource table
        unimplemented!("Resource table not implemented")
    }
    
    fn table_mut(&mut self) -> &mut wasmtime::component::ResourceTable {
        // TODO: Implement resource table
        unimplemented!("Resource table not implemented")
    }
}

/// Loaded plugin information
#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    pub id: String,
    pub metadata: PluginMetadata,
    pub component: Component,
    pub sandbox_context: SandboxContext,
    pub loaded_at: chrono::DateTime<chrono::Utc>,
    pub execution_count: u64,
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
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_runtime_creation() {
        let runtime = WasiPluginRuntime::new().unwrap();
        assert_eq!(runtime.list_plugins().await.len(), 0);
    }
    
    #[tokio::test]
    async fn test_runtime_initialization() {
        let mut runtime = WasiPluginRuntime::new().unwrap();
        runtime.initialize().await.unwrap();
        assert_eq!(runtime.list_plugins().await.len(), 0);
    }
    
    #[test]
    fn test_plugin_config() {
        let config = PluginConfig::default();
        let runtime = WasiPluginRuntime::with_config(config).unwrap();
        assert!(runtime.config().execution_timeout_ms > 0);
    }
} 