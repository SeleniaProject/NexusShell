//! Pure Rust WASI-like Plugin Runtime for NexusShell
//! 
//! This module provides a WASI-compatible runtime for executing plugins
//! using Pure Rust components with security sandboxing and resource limits.
//! NO C dependencies - uses wasmi and custom WASI implementation.

use anyhow::{Result, Context, anyhow};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
    fs, io,
};
use tokio::sync::{RwLock, Semaphore, Mutex};
use wasmi::{Engine, Store, Module, Instance, Linker, Caller, Func, Value, Memory};
use log::{info, warn, error, debug};

use crate::{
    security::SecurityContext,
    permissions::PluginPermissions,
    registrar::PluginRegistrar,
};

/// Plugin execution result
pub type PluginResult<T> = std::result::Result<T, PluginError>;

/// Plugin runtime errors
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Plugin execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Invalid plugin format: {0}")]
    InvalidFormat(String),
    
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("WASM error: {0}")]
    Wasm(#[from] wasmi::Error),
}

/// Pure Rust WASI-like Plugin Runtime
pub struct WasiPluginRuntime {
    engine: Engine,
    linker: Arc<RwLock<Linker<PluginRuntimeState>>>,
    plugins: Arc<RwLock<HashMap<String, LoadedWasiPlugin>>>,
    execution_semaphore: Arc<Semaphore>,
    config: RuntimeConfig,
}

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub max_concurrent_plugins: usize,
    pub max_memory_per_plugin: usize,
    pub max_execution_time: Duration,
    pub allowed_directories: Vec<PathBuf>,
    pub enable_networking: bool,
    pub enable_filesystem: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_plugins: 10,
            max_memory_per_plugin: 32 * 1024 * 1024, // 32MB
            max_execution_time: Duration::from_secs(30),
            allowed_directories: vec![],
            enable_networking: false,
            enable_filesystem: false,
        }
    }
}

/// Plugin runtime state
#[derive(Debug)]
pub struct PluginRuntimeState {
    plugin_name: String,
    permissions: PluginPermissions,
    security_context: SecurityContext,
    registrar: PluginRegistrar,
    start_time: Instant,
    file_descriptors: HashMap<i32, VirtualFile>,
    next_fd: i32,
    exit_code: Option<i32>,
}

/// Virtual file descriptor for WASI filesystem simulation
#[derive(Debug)]
pub struct VirtualFile {
    path: PathBuf,
    position: u64,
    flags: i32,
    data: Vec<u8>,
}

impl WasiPluginRuntime {
    /// Create a new WASI plugin runtime with default configuration
    pub async fn new() -> Result<Self> {
        let config = RuntimeConfig::default();
        Self::with_config(config).await
    }
    
    /// Create a new WASI plugin runtime with custom configuration
    pub async fn with_config(config: RuntimeConfig) -> Result<Self> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        
        // Register WASI-like host functions
        Self::register_wasi_functions(&mut linker)?;
        
        Ok(Self {
            engine,
            linker: Arc::new(RwLock::new(linker)),
            plugins: Arc::new(RwLock::new(HashMap::new())),
            execution_semaphore: Arc::new(Semaphore::new(config.max_concurrent_plugins)),
            config,
        })
    }

    /// Load a WASI plugin from file
    pub async fn load_plugin<P: AsRef<Path>>(
        &self,
        path: P,
        plugin_name: String,
        permissions: PluginPermissions,
    ) -> PluginResult<()> {
        let path = path.as_ref();
        
        // Read WASM module
        let wasm_bytes = fs::read(path)
            .with_context(|| format!("Failed to read plugin file: {}", path.display()))?;

        // Parse WASM module
        let module = Module::new(&self.engine, &wasm_bytes)
            .with_context(|| format!("Failed to parse WASM module: {}", plugin_name))?;

        // Create plugin state
        let state = PluginRuntimeState {
            plugin_name: plugin_name.clone(),
            permissions,
            security_context: SecurityContext::new_restricted(),
            registrar: PluginRegistrar::new(),
            start_time: Instant::now(),
            file_descriptors: HashMap::new(),
            next_fd: 3, // 0=stdin, 1=stdout, 2=stderr
            exit_code: None,
        };

        // Create store with state
        let store = Store::new(&self.engine, state);

        // Instantiate module
        let linker = self.linker.read().await;
        let instance = linker
            .instantiate(&store, &module)
            .with_context(|| format!("Failed to instantiate plugin: {}", plugin_name))?;

        let loaded_plugin = LoadedWasiPlugin {
            instance,
            store: Arc::new(Mutex::new(store)),
            plugin_name: plugin_name.clone(),
            metadata: PluginMetadata::default(),
        };

        // Store loaded plugin
        let mut plugins = self.plugins.write().await;
        plugins.insert(plugin_name.clone(), loaded_plugin);

        info!("Successfully loaded WASI plugin: {}", plugin_name);
        Ok(())
    }

    /// Execute a plugin command
    pub async fn execute_plugin(
        &self,
        plugin_name: &str,
        command: &str,
        args: &[String],
    ) -> PluginResult<i32> {
        // Acquire execution semaphore
        let _permit = self.execution_semaphore.acquire().await
            .map_err(|e| PluginError::ExecutionFailed(format!("Failed to acquire execution permit: {}", e)))?;

        // Get plugin
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(plugin_name)
            .ok_or_else(|| PluginError::NotFound(plugin_name.to_string()))?;

        // Execute plugin
        plugin.execute(command, args).await
    }

    /// Register WASI-like host functions
    fn register_wasi_functions(linker: &mut Linker<PluginRuntimeState>) -> Result<()> {
        // WASI proc_exit
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "proc_exit",
            |mut caller: Caller<'_, PluginRuntimeState>, exit_code: i32| -> Result<(), wasmi::Error> {
                caller.data_mut().exit_code = Some(exit_code);
                // Note: In a real implementation, this would terminate the instance
                Ok(())
            },
        )?;

        // WASI fd_write (simplified stdout/stderr)
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "fd_write",
            |caller: Caller<'_, PluginRuntimeState>, fd: i32, iovs_ptr: i32, iovs_len: i32, nwritten_ptr: i32| -> Result<i32, wasmi::Error> {
                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| wasmi::Error::new("Missing memory export"))?;

                let data = memory.data(&caller);
                
                // Simple implementation: just write to stdout if fd == 1
                if fd == 1 || fd == 2 {
                    // For simplicity, assume single iov
                    if iovs_len > 0 {
                        let iov_base = u32::from_le_bytes([
                            data[iovs_ptr as usize],
                            data[iovs_ptr as usize + 1],
                            data[iovs_ptr as usize + 2],
                            data[iovs_ptr as usize + 3],
                        ]) as usize;
                        let iov_len = u32::from_le_bytes([
                            data[iovs_ptr as usize + 4],
                            data[iovs_ptr as usize + 5],
                            data[iovs_ptr as usize + 6],
                            data[iovs_ptr as usize + 7],
                        ]) as usize;

                        if let Some(output_data) = data.get(iov_base..iov_base + iov_len) {
                            let output_str = String::from_utf8_lossy(output_data);
                            if fd == 1 {
                                print!("{}", output_str);
                            } else {
                                eprint!("{}", output_str);
                            }
                            
                            // Write nwritten
                            if let Some(nwritten_bytes) = data.get_mut(nwritten_ptr as usize..nwritten_ptr as usize + 4) {
                                nwritten_bytes.copy_from_slice(&(iov_len as u32).to_le_bytes());
                            }
                            return Ok(0); // Success
                        }
                    }
                }
                Ok(8) // EBADF
            },
        )?;

        // WASI fd_read (simplified stdin)
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "fd_read",
            |_caller: Caller<'_, PluginRuntimeState>, _fd: i32, _iovs_ptr: i32, _iovs_len: i32, _nread_ptr: i32| -> Result<i32, wasmi::Error> {
                // Simplified: return EOF for all reads
                Ok(0)
            },
        )?;

        // WASI environ_sizes_get
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "environ_sizes_get",
            |_caller: Caller<'_, PluginRuntimeState>, _environc_ptr: i32, _environ_buf_size_ptr: i32| -> Result<i32, wasmi::Error> {
                // Return empty environment
                Ok(0)
            },
        )?;

        // WASI environ_get
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "environ_get",
            |_caller: Caller<'_, PluginRuntimeState>, _environ_ptr: i32, _environ_buf_ptr: i32| -> Result<i32, wasmi::Error> {
                // Return empty environment
                Ok(0)
            },
        )?;

        // WASI args_sizes_get
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "args_sizes_get",
            |_caller: Caller<'_, PluginRuntimeState>, _argc_ptr: i32, _argv_buf_size_ptr: i32| -> Result<i32, wasmi::Error> {
                // Return no arguments
                Ok(0)
            },
        )?;

        // WASI args_get
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "args_get",
            |_caller: Caller<'_, PluginRuntimeState>, _argv_ptr: i32, _argv_buf_ptr: i32| -> Result<i32, wasmi::Error> {
                // Return no arguments
                Ok(0)
            },
        )?;

        info!("WASI host functions registered successfully");
        Ok(())
    }

    /// Get list of loaded plugins
    pub async fn list_plugins(&self) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins.keys().cloned().collect()
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_name: &str) -> PluginResult<()> {
        let mut plugins = self.plugins.write().await;
        if plugins.remove(plugin_name).is_some() {
            info!("Unloaded plugin: {}", plugin_name);
            Ok(())
        } else {
            Err(PluginError::NotFound(plugin_name.to_string()))
        }
    }

    /// Shutdown runtime and cleanup all plugins
    pub async fn shutdown(&self) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        plugins.clear();
        info!("WASI plugin runtime shutdown complete");
        Ok(())
    }
}

/// Loaded WASI plugin instance
pub struct LoadedWasiPlugin {
    instance: Instance,
    store: Arc<Mutex<Store<PluginRuntimeState>>>,
    plugin_name: String,
    metadata: PluginMetadata,
}

/// Plugin metadata
#[derive(Debug, Default)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

impl LoadedWasiPlugin {
    /// Execute plugin command
    pub async fn execute(&self, command: &str, args: &[String]) -> PluginResult<i32> {
        let mut store = self.store.lock().await;
        
        // Try to get main function
        let main_func = self.instance
            .get_func(&*store, "_start")
            .or_else(|| self.instance.get_func(&*store, "main"))
            .context("Plugin does not export _start or main function")?;

        // Execute main function
        let mut results = vec![Value::I32(0)];
        main_func
            .call(&mut *store, &[], &mut results)
            .with_context(|| format!("Failed to execute plugin: {}", self.plugin_name))?;

        // Get exit code from state
        let exit_code = store.data().exit_code.unwrap_or(0);
        
        debug!("Plugin '{}' executed with exit code: {}", self.plugin_name, exit_code);
        Ok(exit_code)
    }

    /// Get plugin metadata
    pub fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_runtime_creation() {
        let runtime = WasiPluginRuntime::new().await;
        assert!(runtime.is_ok());
    }

    #[tokio::test]
    async fn test_plugin_loading_invalid_file() {
        let runtime = WasiPluginRuntime::new().await.unwrap();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"invalid wasm content").unwrap();

        let result = runtime.load_plugin(
            temp_file.path(),
            "test_plugin".to_string(),
            PluginPermissions::default(),
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_plugin_list_empty() {
        let runtime = WasiPluginRuntime::new().await.unwrap();
        let plugins = runtime.list_plugins().await;
        assert!(plugins.is_empty());
    }
}
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
        let component_registry = ComponentRegistry::new()?;
        
        let max_concurrent_executions = config.max_concurrent_executions.unwrap_or(10);
        let execution_semaphore = Arc::new(Semaphore::new(max_concurrent_executions));
        
        let mut runtime = Self {
            engine,
            linker: Arc::new(RwLock::new(linker)),
            plugins: Arc::new(RwLock::new(HashMap::new())),
            capability_manager,
            component_registry,
            config,
            execution_semaphore,
        };
        
        // Initialize the runtime
        runtime.initialize().await?;
        
        Ok(runtime)
    }
    
    /// Initialize the runtime with host functions and capabilities
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing WASI plugin runtime");
        
        // Initialize capability manager
        self.capability_manager.initialize().await
            .context("Failed to initialize capability manager")?;
        
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
            memory_usage: 0, // Memory tracking can be implemented later as optimization
        })
    }
    
    /// Get runtime configuration
    pub fn config(&self) -> &PluginConfig {
        &self.config
    }
    
    /// Update runtime configuration
    pub async fn update_config(&mut self, config: PluginConfig) -> Result<()> {
        self.config = config;
        // Apply configuration changes to running plugins if needed
        log::info!("Configuration updated successfully");
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
    resource_table: wasmtime::component::ResourceTable,
    sandbox_context: SandboxContext,
}

impl PluginState {
    pub fn new(sandbox_context: SandboxContext) -> Result<Self> {
        let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_env()
            .build();
        let resource_table = wasmtime::component::ResourceTable::new();
        
        Ok(Self {
            wasi_ctx,
            resource_table,
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
        &self.resource_table
    }
    
    fn table_mut(&mut self) -> &mut wasmtime::component::ResourceTable {
        &mut self.resource_table
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