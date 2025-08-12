//! Pure Rust WASI Plugin Runtime
//!
//! This module provides a comprehensive WASI-like runtime for WebAssembly plugins
//! using Pure Rust components without wasmtime dependencies.

use anyhow::{Result, Context, anyhow};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
    path::{Path, PathBuf},
    fs,
    io::{self, Write, Read},
};
use tokio::{
    sync::{RwLock, Semaphore, Mutex},
    time::timeout,
};
use wasmi::{Engine, Store, Module, Instance, Linker, Caller, Func};
use log::{info, warn, error, debug};
use uuid::Uuid;

use crate::{
    security::SecurityContext,
    permissions::PluginPermissions,
    component::{ComponentRegistry, ComponentValue},
    registrar::PluginRegistrar,
};

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Maximum execution time in milliseconds
    pub execution_timeout_ms: u64,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Maximum number of concurrent executions
    pub max_concurrent_executions: Option<usize>,
    /// Enable debugging features
    pub debug_mode: bool,
    /// Plugin directory for loading
    pub plugin_directory: Option<PathBuf>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            execution_timeout_ms: 30000, // 30 seconds
            max_memory_bytes: 128 * 1024 * 1024, // 128MB
            max_concurrent_executions: Some(10),
            debug_mode: false,
            plugin_directory: None,
        }
    }
}

/// WASI Plugin Runtime using Pure Rust wasmi
pub struct WasiPluginRuntime {
    engine: Engine,
    linker: Arc<RwLock<Linker<RuntimeContext>>>,
    plugins: Arc<RwLock<HashMap<String, LoadedPlugin>>>,
    capability_manager: CapabilityManager,
    component_registry: ComponentRegistry,
    config: RuntimeConfig,
    execution_semaphore: Arc<Semaphore>,
}

/// Runtime context for plugin execution
#[derive(Debug)]
pub struct RuntimeContext {
    pub security_context: SecurityContext,
    pub permissions: PluginPermissions,
    pub registrar: PluginRegistrar,
    pub file_descriptors: HashMap<i32, FileDescriptor>,
    pub environment: HashMap<String, String>,
    pub args: Vec<String>,
    pub start_time: SystemTime,
}

impl RuntimeContext {
    pub fn new() -> Self {
        let mut file_descriptors = HashMap::new();
        
        // Add standard file descriptors
        file_descriptors.insert(0, FileDescriptor::stdin());
        file_descriptors.insert(1, FileDescriptor::stdout());
        file_descriptors.insert(2, FileDescriptor::stderr());
        
        Self {
            security_context: SecurityContext::new_restricted(),
            permissions: PluginPermissions::default(),
            registrar: PluginRegistrar::new(),
            file_descriptors,
            environment: std::env::vars().collect(),
            args: Vec::new(),
            start_time: SystemTime::now(),
        }
    }
}

/// File descriptor for WASI emulation
#[derive(Debug, Clone)]
pub enum FileDescriptor {
    Stdin,
    Stdout,
    Stderr,
    File {
        path: PathBuf,
        readable: bool,
        writable: bool,
    },
}

impl FileDescriptor {
    pub fn stdin() -> Self {
        Self::Stdin
    }
    
    pub fn stdout() -> Self {
        Self::Stdout
    }
    
    pub fn stderr() -> Self {
        Self::Stderr
    }
}

/// Loaded plugin information
#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    pub id: String,
    pub module: Module,
    pub metadata: PluginMetadata,
    pub load_time: SystemTime,
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub permissions: Vec<String>,
}

impl Default for PluginMetadata {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            version: "0.0.0".to_string(),
            description: "No description".to_string(),
            permissions: Vec::new(),
        }
    }
}

impl WasiPluginRuntime {
    /// Create a new WASI plugin runtime with default configuration
    pub fn new() -> Result<Self> {
        let config = RuntimeConfig::default();
        Self::with_config(config)
    }
    
    /// Create a new WASI plugin runtime with custom configuration
    pub fn with_config(config: RuntimeConfig) -> Result<Self> {
        let engine = Engine::default();
        let linker = Linker::new(&engine);
        
        // Initialize capability manager
        let capability_manager = CapabilityManager::new(SecurityContext::new_restricted())?;
        
        // Initialize component registry
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
    
    /// Setup WASI host functions
    async fn setup_host_functions(&mut self) -> Result<()> {
        let mut linker = self.linker.write().await;
        
        // WASI core functions
        linker.func_wrap("wasi_snapshot_preview1", "proc_exit", |_: Caller<'_, RuntimeContext>, exit_code: i32| {
            debug!("proc_exit called with code: {}", exit_code);
            // In a real implementation, this would exit the plugin
        })?;
        
        linker.func_wrap("wasi_snapshot_preview1", "fd_write", |mut caller: Caller<'_, RuntimeContext>, fd: i32, iovs: i32, iovs_len: i32, nwritten: i32| -> Result<i32, wasmi::Error> {
            let memory = match caller.get_export("memory") {
                Some(wasmi::Extern::Memory(mem)) => mem,
                _ => return Ok(8), // EBADF
            };
            
            let mut total_written = 0u32;
            
            for i in 0..iovs_len {
                let iov_base = iovs + i * 8;
                
                // Read iovec structure
                let mut buf = [0u8; 8];
                memory.read(&caller, iov_base as usize, &mut buf).map_err(|_| wasmi::Error::Store)?;
                
                let ptr = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                let len = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
                
                // Read data
                let mut data = vec![0u8; len as usize];
                memory.read(&caller, ptr as usize, &mut data).map_err(|_| wasmi::Error::Store)?;
                
                // Write to appropriate file descriptor
                match fd {
                    1 => {
                        // stdout
                        print!("{}", String::from_utf8_lossy(&data));
                        io::stdout().flush().ok();
                        total_written += len;
                    },
                    2 => {
                        // stderr
                        eprint!("{}", String::from_utf8_lossy(&data));
                        io::stderr().flush().ok();
                        total_written += len;
                    },
                    _ => {
                        // Other file descriptors (simplified)
                        total_written += len;
                    }
                }
            }
            
            // Write total bytes written
            let written_bytes = total_written.to_le_bytes();
            memory.write(&mut caller, nwritten as usize, &written_bytes).map_err(|_| wasmi::Error::Store)?;
            
            Ok(0) // Success
        })?;
        
        linker.func_wrap("wasi_snapshot_preview1", "environ_sizes_get", |mut caller: Caller<'_, RuntimeContext>, environc: i32, environ_buf_size: i32| -> Result<i32, wasmi::Error> {
            let memory = match caller.get_export("memory") {
                Some(wasmi::Extern::Memory(mem)) => mem,
                _ => return Ok(8), // EBADF
            };
            
            let context = caller.data();
            let env_count = context.environment.len() as u32;
            let mut buf_size = 0u32;
            
            for (key, value) in &context.environment {
                buf_size += (key.len() + value.len() + 2) as u32; // key=value\0
            }
            
            // Write environment count
            memory.write(&mut caller, environc as usize, &env_count.to_le_bytes()).map_err(|_| wasmi::Error::Store)?;
            
            // Write buffer size
            memory.write(&mut caller, environ_buf_size as usize, &buf_size.to_le_bytes()).map_err(|_| wasmi::Error::Store)?;
            
            Ok(0) // Success
        })?;
        
        linker.func_wrap("wasi_snapshot_preview1", "environ_get", |mut caller: Caller<'_, RuntimeContext>, environ: i32, environ_buf: i32| -> Result<i32, wasmi::Error> {
            let memory = match caller.get_export("memory") {
                Some(wasmi::Extern::Memory(mem)) => mem,
                _ => return Ok(8), // EBADF
            };
            
            let context = caller.data();
            let mut buf_offset = environ_buf as usize;
            let mut ptr_offset = environ as usize;
            
            for (key, value) in &context.environment {
                let env_string = format!("{}={}\0", key, value);
                let env_bytes = env_string.as_bytes();
                
                // Write pointer to string
                memory.write(&mut caller, ptr_offset, &(buf_offset as u32).to_le_bytes()).map_err(|_| wasmi::Error::Store)?;
                ptr_offset += 4;
                
                // Write string
                memory.write(&mut caller, buf_offset, env_bytes).map_err(|_| wasmi::Error::Store)?;
                buf_offset += env_bytes.len();
            }
            
            Ok(0) // Success
        })?;
        
        linker.func_wrap("wasi_snapshot_preview1", "args_sizes_get", |mut caller: Caller<'_, RuntimeContext>, argc: i32, argv_buf_size: i32| -> Result<i32, wasmi::Error> {
            let memory = match caller.get_export("memory") {
                Some(wasmi::Extern::Memory(mem)) => mem,
                _ => return Ok(8), // EBADF
            };
            
            let context = caller.data();
            let arg_count = context.args.len() as u32;
            let mut buf_size = 0u32;
            
            for arg in &context.args {
                buf_size += (arg.len() + 1) as u32; // arg\0
            }
            
            // Write argument count
            memory.write(&mut caller, argc as usize, &arg_count.to_le_bytes()).map_err(|_| wasmi::Error::Store)?;
            
            // Write buffer size
            memory.write(&mut caller, argv_buf_size as usize, &buf_size.to_le_bytes()).map_err(|_| wasmi::Error::Store)?;
            
            Ok(0) // Success
        })?;
        
        linker.func_wrap("wasi_snapshot_preview1", "clock_time_get", |mut caller: Caller<'_, RuntimeContext>, id: i32, precision: i64, time: i32| -> Result<i32, wasmi::Error> {
            let memory = match caller.get_export("memory") {
                Some(wasmi::Extern::Memory(mem)) => mem,
                _ => return Ok(8), // EBADF
            };
            
            let now = SystemTime::now();
            let timestamp = now.duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            
            // Write timestamp
            memory.write(&mut caller, time as usize, &timestamp.to_le_bytes()).map_err(|_| wasmi::Error::Store)?;
            
            Ok(0) // Success
        })?;
        
        debug!("WASI host functions setup completed");
        Ok(())
    }
    
    /// Load a plugin from WebAssembly bytes
    pub async fn load_plugin_from_bytes(
        &self,
        plugin_id: String,
        wasm_bytes: &[u8],
        metadata: PluginMetadata,
    ) -> Result<()> {
        let module = Module::new(&self.engine, wasm_bytes)
            .context("Failed to compile WebAssembly module")?;
        
        let plugin = LoadedPlugin {
            id: plugin_id.clone(),
            module,
            metadata,
            load_time: SystemTime::now(),
        };
        
        let mut plugins = self.plugins.write().await;
        plugins.insert(plugin_id.clone(), plugin);
        
        info!("Plugin '{}' loaded successfully", plugin_id);
        Ok(())
    }
    
    /// Load a plugin from file
    pub async fn load_plugin_from_file<P: AsRef<Path>>(
        &self,
        plugin_id: String,
        path: P,
        metadata: PluginMetadata,
    ) -> Result<()> {
        let path = path.as_ref();
        let wasm_bytes = fs::read(path)
            .with_context(|| format!("Failed to read plugin file: {}", path.display()))?;
        
        self.load_plugin_from_bytes(plugin_id, &wasm_bytes, metadata).await
    }
    
    /// Execute a function in a loaded plugin
    pub async fn execute_plugin_function(
        &self,
        plugin_id: &str,
        function_name: &str,
        args: &[ComponentValue],
    ) -> Result<Vec<ComponentValue>> {
        let _permit = self.execution_semaphore.acquire().await?;
        
        let plugin = {
            let plugins = self.plugins.read().await;
            plugins.get(plugin_id)
                .ok_or_else(|| anyhow!("Plugin '{}' not found", plugin_id))?
                .clone()
        };
        
        let context = RuntimeContext::new();
        let mut store = Store::new(&self.engine, context);
        
        let linker = self.linker.read().await;
        let instance = linker.instantiate(&mut store, &plugin.module)
            .context("Failed to instantiate plugin")?;
        
        let execution_timeout = Duration::from_millis(self.config.execution_timeout_ms);
        
        let result = timeout(execution_timeout, async {
            self.execute_function(&mut store, &instance, function_name, args).await
        }).await??;
        
        Ok(result)
    }
    
    /// Execute function in instance
    async fn execute_function(
        &self,
        store: &mut Store<RuntimeContext>,
        instance: &Instance,
        function_name: &str,
        args: &[ComponentValue],
    ) -> Result<Vec<ComponentValue>> {
        let func = instance
            .get_func(store, function_name)
            .ok_or_else(|| anyhow!("Function '{}' not found", function_name))?;
        
        // Convert arguments (simplified)
        let wasm_args: Vec<wasmi::Value> = args.iter()
            .map(|arg| match arg {
                ComponentValue::S32(i) => wasmi::Value::I32(*i),
                ComponentValue::S64(i) => wasmi::Value::I64(*i),
                ComponentValue::Float32(f) => wasmi::Value::F32((*f).into()),
                ComponentValue::Float64(f) => wasmi::Value::F64((*f).into()),
                _ => wasmi::Value::I32(0), // Simplified conversion
            })
            .collect();
        
        let mut results = vec![wasmi::Value::I32(0); func.ty(store).results().len()];
        
        func.call(store, &wasm_args, &mut results)
            .context("Function execution failed")?;
        
        // Convert results back (simplified)
        let component_results: Vec<ComponentValue> = results.iter()
            .map(|val| match val {
                wasmi::Value::I32(i) => ComponentValue::S32(*i),
                wasmi::Value::I64(i) => ComponentValue::S64(*i),
                wasmi::Value::F32(f) => ComponentValue::Float32(f.to_float()),
                wasmi::Value::F64(f) => ComponentValue::Float64(f.to_float()),
            })
            .collect();
        
        Ok(component_results)
    }
    
    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<bool> {
        let mut plugins = self.plugins.write().await;
        let removed = plugins.remove(plugin_id).is_some();
        
        if removed {
            info!("Plugin '{}' unloaded successfully", plugin_id);
        }
        
        Ok(removed)
    }
    
    /// List all loaded plugins
    pub async fn list_plugins(&self) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins.keys().cloned().collect()
    }
    
    /// Get plugin metadata
    pub async fn get_plugin_metadata(&self, plugin_id: &str) -> Option<PluginMetadata> {
        let plugins = self.plugins.read().await;
        plugins.get(plugin_id).map(|p| p.metadata.clone())
    }
    
    /// Get runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }
}

/// Capability manager for plugin permissions
#[derive(Debug)]
pub struct CapabilityManager {
    security_context: SecurityContext,
    granted_capabilities: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl CapabilityManager {
    pub fn new(security_context: SecurityContext) -> Result<Self> {
        Ok(Self {
            security_context,
            granted_capabilities: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    pub async fn initialize(&self) -> Result<()> {
        info!("Capability manager initialized");
        Ok(())
    }
    
    pub async fn grant_capability(&self, plugin_id: &str, capability: &str) -> Result<()> {
        let mut capabilities = self.granted_capabilities.lock().await;
        capabilities.entry(plugin_id.to_string())
            .or_insert_with(Vec::new)
            .push(capability.to_string());
        
        debug!("Granted capability '{}' to plugin '{}'", capability, plugin_id);
        Ok(())
    }
    
    pub async fn check_capability(&self, plugin_id: &str, capability: &str) -> bool {
        let capabilities = self.granted_capabilities.lock().await;
        capabilities.get(plugin_id)
            .map(|caps| caps.contains(&capability.to_string()))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
        let config = RuntimeConfig::default();
        let runtime = WasiPluginRuntime::with_config(config).unwrap();
        assert!(runtime.config().execution_timeout_ms > 0);
    }
} 
