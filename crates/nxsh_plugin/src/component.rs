use anyhow::Result;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;
// Note: Simplified Pure Rust implementation replacing wasmtime components
// Component model support is limited, using wasmi for basic plugin functionality
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasi-runtime")]
use wasmi::{Engine, Linker, Module, Store};

use crate::{PluginError, PluginMetadata, PluginResult};

/// Component Model registry for managing WebAssembly components
/// Simplified Pure Rust implementation using wasmi
pub struct ComponentRegistry {
    engine: Engine,
    linker: Arc<RwLock<Linker<ComponentState>>>,
    components: Arc<RwLock<HashMap<String, RegisteredComponent>>>,
    // Simplified component type information
    component_types: Arc<RwLock<HashMap<String, String>>>, // name -> type info JSON
    bindings: Arc<RwLock<HashMap<String, ComponentBinding>>>,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Result<Self> {
        // Simple wasmi engine configuration (Pure Rust)
        let engine = Engine::default();
        let linker = Linker::new(&engine);

        Ok(Self {
            engine,
            linker: Arc::new(RwLock::new(linker)),
            components: Arc::new(RwLock::new(HashMap::new())),
            component_types: Arc::new(RwLock::new(HashMap::new())),
            bindings: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Initialize the component registry with host bindings
    pub async fn initialize(&mut self) -> Result<()> {
        let mut linker = self.linker.write().await;

        // Add custom host functions for NexusShell
        // Note: WASI functions need to be implemented separately for wasmi
        self.add_nexus_host_functions(&mut linker).await?;
        // Minimal WASI-like shims (subset) to improve compatibility
        self.add_minimal_wasi_shims(&mut linker).await?;

        log::info!("Component registry initialized successfully");
        Ok(())
    }

    /// Register a WebAssembly component
    pub async fn register_component<P: AsRef<Path>>(
        &self,
        component_id: String,
        path: P,
        metadata: PluginMetadata,
    ) -> PluginResult<()> {
        let component_bytes = tokio::fs::read(&path)
            .await
            .map_err(|e| PluginError::LoadError(format!("Failed to read component file: {e}")))?;

        // Validate module format (wasmi instead of component)
        let module = Module::new(&self.engine, &component_bytes)
            .map_err(|e| PluginError::LoadError(format!("Invalid WASM module format: {e}")))?;

        // Extract simplified type information
        let type_info = self.extract_module_type_info(&module)?;

        // Create component binding
        let binding = self
            .create_component_binding(&component_id, &type_info, &metadata)
            .await?;

        // Register component
        let registered_component = RegisteredComponent {
            id: component_id.clone(),
            module,
            metadata,
            path: path.as_ref().to_path_buf(),
            registered_at: chrono::Utc::now(),
        };

        // Store in registry
        {
            let mut components = self.components.write().await;
            components.insert(component_id.clone(), registered_component);
        }

        {
            let mut component_types = self.component_types.write().await;
            component_types.insert(component_id.clone(), type_info.clone());
        }

        {
            let mut bindings = self.bindings.write().await;
            bindings.insert(component_id.clone(), binding);
        }

        log::info!("Component '{component_id}' registered successfully");
        Ok(())
    }

    /// Unregister a WebAssembly component
    pub async fn unregister_component(&self, component_id: &str) -> PluginResult<()> {
        {
            let mut components = self.components.write().await;
            components.remove(component_id);
        }

        {
            let mut component_types = self.component_types.write().await;
            component_types.remove(component_id);
        }

        {
            let mut bindings = self.bindings.write().await;
            bindings.remove(component_id);
        }

        log::info!("Component '{component_id}' unregistered successfully");
        Ok(())
    }

    /// Execute a function in a WebAssembly component
    pub async fn execute_component_function(
        &self,
        component_id: &str,
        function_name: &str,
        args: &[ComponentValue],
    ) -> PluginResult<Vec<ComponentValue>> {
        // Simplified implementation for wasmi
        log::debug!(
            "Executing function '{}' in component '{}' with {} args",
            function_name,
            component_id,
            args.len()
        );

        // Check if component exists
        let components = self.components.read().await;
        let component = components.get(component_id).ok_or_else(|| {
            PluginError::NotFound(format!("Component '{component_id}' not found"))
        })?;

        // Complete implementation for component function invocation
        // 1. Create a store with the component state
        let store_data = ComponentState::new()?;
        let engine = Engine::default();
        let mut store = Store::new(&engine, store_data);

        // 2. Create linker and add host functions
        let mut linker = Linker::<ComponentState>::new(&engine);
        self.add_nexus_host_functions(&mut linker).await?;

        // 3. Instantiate the module
        let instance = linker
            .instantiate(&mut store, &component.module)
            .map_err(|e| PluginError::Runtime(format!("Failed to instantiate component: {e:?}")))?
            .start(&mut store)
            .map_err(|e| PluginError::Runtime(format!("Failed to start component: {e:?}")))?;

        // 4. Find and call the requested function
        let func = instance
            .get_func(&mut store, function_name)
            .ok_or_else(|| {
                PluginError::ExecutionError(format!(
                    "Function '{function_name}' not found in component '{component_id}'"
                ))
            })?;

        // 5. Convert arguments to wasmi values
        let wasmi_args: Vec<wasmi::Val> = args
            .iter()
            .map(|arg| self.component_value_to_wasmi(arg))
            .collect();

        // 6. Execute the function
        let mut results = vec![wasmi::Val::I32(0); func.ty(&store).results().len()];
        func.call(&mut store, &wasmi_args, &mut results)
            .map_err(|e| PluginError::Runtime(format!("Failed to call function: {e:?}")))?;

        // 6. Convert results back to ComponentValue
        let component_results: Vec<ComponentValue> = results
            .iter()
            .map(|result| self.wasmi_to_component_value(result))
            .collect();

        Ok(component_results)
    }

    /// Get component metadata
    pub async fn get_component_metadata(&self, component_id: &str) -> Option<PluginMetadata> {
        let components = self.components.read().await;
        components.get(component_id).map(|c| c.metadata.clone())
    }

    /// List all registered components
    pub async fn list_components(&self) -> Vec<String> {
        let components = self.components.read().await;
        components.keys().cloned().collect()
    }

    /// Get component type information
    pub async fn get_component_type(&self, component_id: &str) -> Option<String> {
        let component_types = self.component_types.read().await;
        component_types.get(component_id).cloned()
    }

    /// Create component from WIT definition - simplified for wasmi
    pub async fn create_component_from_wit(
        &self,
        _wit_source: &str,
        wasm_module: &[u8],
    ) -> Result<Vec<u8>> {
        // Simplified implementation - just return the WASM module as-is
        // Full Component Model not supported in wasmi
        Ok(wasm_module.to_vec())
    }

    /// Extract type information from WASM module (simplified)
    fn extract_module_type_info(&self, _module: &Module) -> PluginResult<String> {
        // Simplified type extraction - just return basic info as JSON
        let type_info = serde_json::json!({
            "exports": [],
            "imports": [],
            "functions": []
        });
        Ok(type_info.to_string())
    }

    // Private helper methods

    async fn add_nexus_host_functions(&self, linker: &mut Linker<ComponentState>) -> Result<()> {
        // Add shell-specific host functions for NexusShell plugin API
        // Note: Host functions need to be added to the linker with a proper store context
        // This is a comprehensive implementation with proper wasmi 0.34 API usage

        // Create a store for host function binding
        let engine = wasmi::Engine::default();
        let _store = wasmi::Store::new(&engine, ComponentState::new());

        // Implement proper host function binding with correct wasmi 0.34 API
        // Create comprehensive host function set for NexusShell plugin system

        // Shell command execution host function
        linker
            .func_wrap(
                "shell",
                "execute",
                |mut caller: wasmi::Caller<'_, ComponentState>,
                 command_ptr: i32,
                 command_len: i32|
                 -> Result<i32, wasmi::Error> {
                    let memory = caller
                        .get_export("memory")
                        .and_then(|export| export.into_memory())
                        .ok_or_else(|| wasmi::Error::new("Plugin memory not accessible"))?;

                    // Extract command string from WASM memory
                    let memory_data = memory.data(&caller);
                    let start = command_ptr as usize;
                    let end = (command_ptr + command_len) as usize;

                    if end > memory_data.len() {
                        return Err(wasmi::Error::new("Memory access out of bounds"));
                    }

                    let command_bytes = &memory_data[start..end];
                    let command = String::from_utf8(command_bytes.to_vec())
                        .map_err(|_| wasmi::Error::new("Invalid UTF-8 in command string"))?;

                    // Execute command through component state
                    let state = caller.data_mut();
                    match state.execute_shell_command(&command) {
                        Ok(_) => Ok(0),
                        Err(e) => {
                            log::error!("Plugin command execution failed: {e}");
                            Ok(1)
                        }
                    }
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to register shell.execute: {}", e))?;

        // Environment variable access host function
        linker
            .func_wrap(
                "env",
                "get",
                |mut caller: wasmi::Caller<'_, ComponentState>,
                 key_ptr: i32,
                 key_len: i32,
                 value_ptr: i32,
                 value_max_len: i32|
                 -> Result<i32, wasmi::Error> {
                    let memory = caller
                        .get_export("memory")
                        .and_then(|export| export.into_memory())
                        .ok_or_else(|| wasmi::Error::new("Plugin memory not accessible"))?;

                    // Extract environment variable key
                    let memory_data = memory.data(&caller);
                    let key_start = key_ptr as usize;
                    let key_end = (key_ptr + key_len) as usize;

                    if key_end > memory_data.len() {
                        return Err(wasmi::Error::new("Key memory access out of bounds"));
                    }

                    let key_bytes = &memory_data[key_start..key_end];
                    let key = String::from_utf8(key_bytes.to_vec())
                        .map_err(|_| wasmi::Error::new("Invalid UTF-8 in environment key"))?;

                    // Get environment variable value
                    match std::env::var(&key) {
                        Ok(value) => {
                            let value_bytes = value.as_bytes();
                            let copy_len = std::cmp::min(value_bytes.len(), value_max_len as usize);

                            // Write value to plugin memory
                            let memory_data = memory.data_mut(&mut caller);
                            let value_start = value_ptr as usize;
                            let value_end = value_start + copy_len;

                            if value_end > memory_data.len() {
                                return Err(wasmi::Error::new("Value memory access out of bounds"));
                            }

                            memory_data[value_start..value_end]
                                .copy_from_slice(&value_bytes[..copy_len]);
                            Ok(copy_len as i32)
                        }
                        Err(_) => Ok(-1), // Environment variable not found
                    }
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to register env.get: {}", e))?;

        // File system read host function
        linker
            .func_wrap(
                "fs",
                "read_file",
                |mut caller: wasmi::Caller<'_, ComponentState>,
                 path_ptr: i32,
                 path_len: i32,
                 content_ptr: i32,
                 content_max_len: i32|
                 -> Result<i32, wasmi::Error> {
                    let memory = caller
                        .get_export("memory")
                        .and_then(|export| export.into_memory())
                        .ok_or_else(|| wasmi::Error::new("Plugin memory not accessible"))?;

                    // Extract file path
                    let memory_data = memory.data(&caller);
                    let path_start = path_ptr as usize;
                    let path_end = (path_ptr + path_len) as usize;

                    if path_end > memory_data.len() {
                        return Err(wasmi::Error::new("Path memory access out of bounds"));
                    }

                    let path_bytes = &memory_data[path_start..path_end];
                    let path = String::from_utf8(path_bytes.to_vec())
                        .map_err(|_| wasmi::Error::new("Invalid UTF-8 in file path"))?;

                    // Read file content with security checks
                    let state = caller.data_mut();
                    if !state.is_path_accessible(&path) {
                        return Ok(-2); // Access denied
                    }

                    match std::fs::read_to_string(&path) {
                        Ok(content) => {
                            let content_bytes = content.as_bytes();
                            let copy_len =
                                std::cmp::min(content_bytes.len(), content_max_len as usize);

                            // Write content to plugin memory
                            let memory_data = memory.data_mut(&mut caller);
                            let content_start = content_ptr as usize;
                            let content_end = content_start + copy_len;

                            if content_end > memory_data.len() {
                                return Err(wasmi::Error::new(
                                    "Content memory access out of bounds",
                                ));
                            }

                            memory_data[content_start..content_end]
                                .copy_from_slice(&content_bytes[..copy_len]);
                            Ok(copy_len as i32)
                        }
                        Err(_) => Ok(-1), // File read error
                    }
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to register fs.read_file: {}", e))?;

        // Logging host function for plugin debugging
        linker
            .func_wrap(
                "log",
                "message",
                |caller: wasmi::Caller<'_, ComponentState>,
                 level: i32,
                 message_ptr: i32,
                 message_len: i32|
                 -> Result<(), wasmi::Error> {
                    let memory = caller
                        .get_export("memory")
                        .and_then(|export| export.into_memory())
                        .ok_or_else(|| wasmi::Error::new("Plugin memory not accessible"))?;

                    // Extract log message
                    let memory_data = memory.data(&caller);
                    let msg_start = message_ptr as usize;
                    let msg_end = (message_ptr + message_len) as usize;

                    if msg_end > memory_data.len() {
                        return Err(wasmi::Error::new("Message memory access out of bounds"));
                    }

                    let message_bytes = &memory_data[msg_start..msg_end];
                    let message = String::from_utf8(message_bytes.to_vec())
                        .map_err(|_| wasmi::Error::new("Invalid UTF-8 in log message"))?;

                    // Log message with appropriate level
                    match level {
                        0 => log::error!("[Plugin] {message}"),
                        1 => log::warn!("[Plugin] {message}"),
                        2 => log::info!("[Plugin] {message}"),
                        3 => log::debug!("[Plugin] {message}"),
                        _ => log::trace!("[Plugin] {message}"),
                    }

                    Ok(())
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to register log.message: {}", e))?;

        log::info!("Host functions successfully registered for NexusShell plugin API");
        Ok(())
    }

    /// Add minimal WASI-like shims required by some plugins
    async fn add_minimal_wasi_shims(&self, linker: &mut Linker<ComponentState>) -> Result<()> {
        use wasmi::Caller;
        // clock_time_get(clock_id: u32, precision: u64, result_ptr: i32) -> i32
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "clock_time_get",
            |mut caller: Caller<'_, ComponentState>,
             _id: i32,
             _precision_lo: i32,
             _precision_hi: i32,
             result_ptr: i32|
             -> Result<i32, wasmi::Error> {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                let nanos: u128 = now.as_nanos();
                let lo = (nanos as u64) as u32;
                let hi = ((nanos as u64) >> 32) as u32;
                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| wasmi::Error::new("Plugin memory not accessible"))?;
                let data = memory.data_mut(&mut caller);
                let ptr = result_ptr as usize;
                if ptr + 8 > data.len() {
                    return Ok(28);
                } // EFAULT-like
                data[ptr..ptr + 4].copy_from_slice(&lo.to_le_bytes());
                data[ptr + 4..ptr + 8].copy_from_slice(&hi.to_le_bytes());
                Ok(0)
            },
        )?;

        // random_get(buf_ptr: i32, buf_len: i32) -> i32
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "random_get",
            |mut caller: Caller<'_, ComponentState>,
             buf_ptr: i32,
             buf_len: i32|
             -> Result<i32, wasmi::Error> {
                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| wasmi::Error::new("Plugin memory not accessible"))?;
                let data = memory.data_mut(&mut caller);
                let start = buf_ptr as usize;
                let len = buf_len as usize;
                if start + len > data.len() {
                    return Ok(28);
                }
                getrandom::getrandom(&mut data[start..start + len])
                    .map_err(|_| wasmi::Error::new("random_get failed"))?;
                Ok(0)
            },
        )?;

        Ok(())
    }

    async fn create_component_binding(
        &self,
        component_id: &str,
        type_info: &str,
        metadata: &PluginMetadata,
    ) -> PluginResult<ComponentBinding> {
        let mut exports = HashMap::new();
        let mut imports = HashMap::new();

        // Parse simplified type information
        if let Ok(type_data) = serde_json::from_str::<serde_json::Value>(type_info) {
            if let Some(export_list) = type_data["exports"].as_array() {
                for export in export_list {
                    if let Some(name) = export.as_str() {
                        exports.insert(name.to_string(), "function".to_string());
                    }
                }
            }
            if let Some(import_list) = type_data["imports"].as_array() {
                for import in import_list {
                    if let Some(name) = import.as_str() {
                        imports.insert(name.to_string(), "function".to_string());
                    }
                }
            }
        }

        Ok(ComponentBinding {
            component_id: component_id.to_string(),
            exports,
            imports,
            metadata: metadata.clone(),
        })
    }

    /// Convert ComponentValue to wasmi::Value for function calls
    fn component_value_to_wasmi(&self, value: &ComponentValue) -> wasmi::Val {
        match value {
            ComponentValue::Bool(b) => wasmi::Val::I32(if *b { 1 } else { 0 }),
            ComponentValue::S8(v) => wasmi::Val::I32(*v as i32),
            ComponentValue::U8(v) => wasmi::Val::I32(*v as i32),
            ComponentValue::S16(v) => wasmi::Val::I32(*v as i32),
            ComponentValue::U16(v) => wasmi::Val::I32(*v as i32),
            ComponentValue::S32(v) => wasmi::Val::I32(*v),
            ComponentValue::U32(v) => wasmi::Val::I32(*v as i32),
            ComponentValue::S64(v) => wasmi::Val::I64(*v),
            ComponentValue::U64(v) => wasmi::Val::I64(*v as i64),
            ComponentValue::Float32(v) => wasmi::Val::F32((*v).into()),
            ComponentValue::Float64(v) => wasmi::Val::F64((*v).into()),
            ComponentValue::String(_) => wasmi::Val::I32(0), // String would need memory allocation
            ComponentValue::List(_) => wasmi::Val::I32(0),   // Complex types need special handling
        }
    }

    /// Convert wasmi::Value back to ComponentValue after function execution
    fn wasmi_to_component_value(&self, value: &wasmi::Val) -> ComponentValue {
        match value {
            wasmi::Val::I32(v) => ComponentValue::S32(*v),
            wasmi::Val::I64(v) => ComponentValue::S64(*v),
            wasmi::Val::F32(v) => ComponentValue::Float32((*v).into()),
            wasmi::Val::F64(v) => ComponentValue::Float64((*v).into()),
            wasmi::Val::FuncRef(func_ref) => {
                // Handle function reference based on wasmi 0.34 API
                if func_ref.is_null() {
                    ComponentValue::String("null_function".to_string())
                } else {
                    // Convert FuncRef to Func and extract function metadata
                    // Since FuncRef doesn't directly provide Func access in wasmi 0.34,
                    // we use a simplified identifier approach
                    ComponentValue::String(format!("function_ref:0x{:p}", func_ref as *const _))
                }
            }
            wasmi::Val::ExternRef(extern_ref) => {
                // Handle external reference based on wasmi 0.34 API
                if extern_ref.is_null() {
                    ComponentValue::String("null_extern".to_string())
                } else {
                    // Extract external object information
                    ComponentValue::String("extern_ref:object".to_string())
                }
            }
        }
    }

    /// Get a unique identifier for a function reference
    #[allow(dead_code)]
    fn get_function_id(&self, func: &wasmi::Func) -> String {
        // Create a unique identifier based on function properties
        // In a real implementation, this would use function metadata
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash the function pointer/reference
        // This is a simplified approach - full implementation would use
        // more sophisticated function identification
        (func as *const wasmi::Func).hash(&mut hasher);

        let hash = hasher.finish();
        format!("{hash:016x}")
    }

    /// Check if a function reference is callable
    pub fn is_function_callable(&self, _func_ref: &wasmi::Func) -> bool {
        // Validate function signature and accessibility
        // This would integrate with the component's type system
        true // Simplified - real implementation would do proper validation
    }

    /// Call a function reference with proper error handling
    pub fn call_function_ref(
        &self,
        func: &wasmi::Func,
        store: &mut wasmi::Store<ComponentState>,
        params: &[wasmi::Val],
    ) -> Result<Vec<wasmi::Val>, wasmi::Error> {
        // Prepare function call with proper parameter validation
        let mut results = vec![wasmi::Val::I32(0); func.ty(&*store).results().len()];

        // Execute function with comprehensive error handling
        func.call(&mut *store, params, &mut results)?;

        Ok(results)
    }
}

/// Registered WebAssembly component
#[derive(Debug)]
pub struct RegisteredComponent {
    pub id: String,
    pub module: Module, // wasmi Module instead of wasmtime Component
    pub metadata: PluginMetadata,
    pub path: PathBuf,
    pub registered_at: chrono::DateTime<chrono::Utc>,
}

/// Component binding information
#[derive(Debug, Clone)]
pub struct ComponentBinding {
    pub component_id: String,
    // Simplified type information as strings instead of wasmtime types
    pub exports: HashMap<String, String>, // function_name -> signature
    pub imports: HashMap<String, String>, // function_name -> signature
    pub metadata: PluginMetadata,
}

/// Component execution state - simplified for wasmi
#[derive(Debug)]
pub struct ComponentState {
    plugin_data: HashMap<String, String>,
}

impl ComponentState {
    pub fn new() -> Result<Self> {
        Ok(Self {
            plugin_data: HashMap::new(),
        })
    }

    /// Execute shell command through the plugin system
    pub fn execute_shell_command(&mut self, command: &str) -> Result<String> {
        log::debug!("Plugin executing shell command: {command}");

        // Validate command for security
        if self.is_command_allowed(command) {
            // For now, simulate command execution
            // In a full implementation, this would integrate with nxsh_core::executor
            match command.trim() {
                "pwd" => Ok(std::env::current_dir()?.to_string_lossy().to_string()),
                cmd if cmd.starts_with("echo ") => {
                    Ok(cmd.strip_prefix("echo ").unwrap_or("").to_string())
                }
                "date" => {
                    use std::time::SystemTime;
                    let now = SystemTime::now();
                    Ok(format!("{now:?}"))
                }
                "whoami" => Ok(whoami::username()),
                _ => {
                    // Execute through system command for basic commands
                    let output = std::process::Command::new("cmd")
                        .arg("/C")
                        .arg(command)
                        .output()?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if output.status.success() {
                        Ok(stdout.to_string())
                    } else {
                        Err(anyhow::anyhow!("Command failed: {}", stderr))
                    }
                }
            }
        } else {
            Err(anyhow::anyhow!("Command not allowed: {}", command))
        }
    }

    /// Check if a command is allowed to be executed by the plugin
    pub fn is_command_allowed(&self, command: &str) -> bool {
        // Security whitelist for plugin commands
        let allowed_commands = &[
            "pwd", "date", "whoami", "echo", "ls", "dir", "cat", "type", "find", "grep", "wc",
            "sort", "uniq", "head", "tail",
        ];

        let command_name = command.split_whitespace().next().unwrap_or("");
        allowed_commands.contains(&command_name) || command.starts_with("echo ")
    }

    /// Check if a file path is accessible to the plugin
    pub fn is_path_accessible(&self, path: &str) -> bool {
        use std::path::Path;

        let path = Path::new(path);

        // Security restrictions for plugin file access
        let forbidden_dirs = &[
            "/etc/shadow",
            "/etc/passwd",
            "C:\\Windows\\System32",
            "/usr/bin",
            "/sbin",
            "C:\\Program Files",
        ];

        // Convert to absolute path for checking
        if let Ok(abs_path) = path.canonicalize() {
            let path_str = abs_path.to_string_lossy();

            // Check if path is in forbidden directories
            for forbidden in forbidden_dirs {
                if path_str.starts_with(forbidden) {
                    log::warn!("Plugin attempted to access forbidden path: {path_str}");
                    return false;
                }
            }

            // Allow access to user directories and common safe locations
            let safe_dirs = &[
                std::env::var("HOME").unwrap_or_default(),
                std::env::var("USERPROFILE").unwrap_or_default(),
                "/tmp".to_string(),
                "C:\\Temp".to_string(),
                "/var/tmp".to_string(),
            ];

            for safe_dir in safe_dirs {
                if !safe_dir.is_empty() && path_str.starts_with(safe_dir) {
                    return true;
                }
            }

            // Allow access to current working directory
            if let Ok(cwd) = std::env::current_dir() {
                if path_str.starts_with(&cwd.to_string_lossy().to_string()) {
                    return true;
                }
            }
        }

        // Default to deny for security
        log::warn!("Plugin access denied for path: {}", path.display());
        false
    }

    /// Store plugin-specific data
    pub fn set_data(&mut self, key: String, value: String) {
        self.plugin_data.insert(key, value);
    }

    /// Retrieve plugin-specific data
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.plugin_data.get(key)
    }
}

/// Component value types for function calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentValue {
    Bool(bool),
    S8(i8),
    U8(u8),
    S16(i16),
    U16(u16),
    S32(i32),
    U32(u32),
    S64(i64),
    U64(u64),
    Float32(f32),
    Float64(f64),
    String(String),
    List(Vec<ComponentValue>),
}

impl ComponentValue {
    /// Create a string component value
    pub fn string<S: Into<String>>(s: S) -> Self {
        Self::String(s.into())
    }

    /// Create a list component value
    pub fn list(items: Vec<ComponentValue>) -> Self {
        Self::List(items)
    }

    /// Get the type name of the component value
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::S8(_) => "s8",
            Self::U8(_) => "u8",
            Self::S16(_) => "s16",
            Self::U16(_) => "u16",
            Self::S32(_) => "s32",
            Self::U32(_) => "u32",
            Self::S64(_) => "s64",
            Self::U64(_) => "u64",
            Self::Float32(_) => "float32",
            Self::Float64(_) => "float64",
            Self::String(_) => "string",
            Self::List(_) => "list",
        }
    }
}

/// Component interface generator
pub struct ComponentInterfaceGenerator {
    wit_definitions: HashMap<String, String>,
}

impl Default for ComponentInterfaceGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentInterfaceGenerator {
    pub fn new() -> Self {
        Self {
            wit_definitions: HashMap::new(),
        }
    }

    /// Add a WIT world definition
    pub fn add_world_definition(&mut self, name: String, definition: String) {
        self.wit_definitions.insert(name, definition);
    }

    /// Generate component interface bindings
    pub fn generate_bindings(&self, world_name: &str) -> Result<String> {
        let _wit_definition = self
            .wit_definitions
            .get(world_name)
            .ok_or_else(|| anyhow::anyhow!("WIT world '{}' not found", world_name))?;

        // Generate Rust bindings using wit-bindgen
        // For production use, this would call wit-bindgen programmatically
        // Currently provides a simplified binding template
        let bindings = format!(
            r#"// Generated bindings for world '{world_name}'
// Based on WIT definition

#[allow(unused_imports)]
use super::*;

/// Host function interface
pub mod host {{
    use super::*;
    
    pub fn log(level: &str, message: &str) -> anyhow::Result<()> {{
        match level {{
            "info" => log::info!("[Plugin] {{}}", message),
            "warn" => log::warn!("[Plugin] {{}}", message),
            "error" => log::error!("[Plugin] {{}}", message),
            _ => log::debug!("[Plugin] {{}}", message),
        }}
        Ok(())
    }}
    
    pub fn read_file(path: &str) -> anyhow::Result<String> {{
        std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read file: {{}}", e))
    }}
}}

/// Plugin export interface
pub trait PluginExports {{
    fn initialize() -> anyhow::Result<()>;
    fn execute(command: &str, args: &[&str]) -> anyhow::Result<String>;
    fn cleanup() -> anyhow::Result<()>;
}}
"#
        );

        Ok(bindings)
    }

    /// Load WIT definitions from directory
    pub fn load_wit_definitions<P: AsRef<Path>>(&mut self, wit_dir: P) -> Result<()> {
        let wit_dir = wit_dir.as_ref();

        for entry in std::fs::read_dir(wit_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("wit") {
                let content = std::fs::read_to_string(&path)?;
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                self.wit_definitions.insert(name, content);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_component_registry_creation() {
        let registry = ComponentRegistry::new().unwrap();
        assert_eq!(registry.list_components().await.len(), 0);
    }

    #[test]
    fn test_component_interface_generator() {
        let mut generator = ComponentInterfaceGenerator::new();
        generator.add_world_definition(
            "test-world".to_string(),
            "world test-world { export test: func() -> string }".to_string(),
        );

        let bindings = generator.generate_bindings("test-world").unwrap();
        assert!(bindings.contains("test-world"));
    }
}
