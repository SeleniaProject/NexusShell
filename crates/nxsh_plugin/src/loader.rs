//! Pure Rust WASM Plugin Loader
//! 
//! This module provides WASM plugin loading capabilities using Pure Rust components.
//! NO C dependencies - uses wasmi instead of wasmtime for Windows compatibility.

use anyhow::{Result, anyhow, Context};
use std::path::Path;
use std::fs;
#[cfg(feature = "wasi-runtime")]
use wasmi::{Engine, Store, Module, Linker, Caller, Instance, Val};
use crate::registrar::PluginRegistrar;
use crate::security::SecurityContext;
use crate::permissions::PluginPermissions;

/// Resource limiter for WASM execution
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_bytes: usize,
    pub max_execution_time_ms: u64,
    pub max_stack_size: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 16 * 1024 * 1024, // 16MB
            max_execution_time_ms: 5000,        // 5 seconds
            max_stack_size: 1024 * 1024,        // 1MB stack
        }
    }
}

/// Pure Rust WASM Plugin Loader
pub struct WasmPluginLoader {
    engine: Engine,
    linker: Linker<PluginHostState>,
    limits: ResourceLimits,
}

/// Host state passed to WASM instances
pub struct PluginHostState {
    registrar: PluginRegistrar,
    security_context: SecurityContext,
    permissions: PluginPermissions,
    plugin_name: String,
}

/// Loaded plugin instance
pub struct LoadedPlugin {
    instance: Instance,
    store: Store<PluginHostState>,
    plugin_name: String,
}

impl WasmPluginLoader {
    /// Create a new WASM plugin loader with default configuration
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        let limits = ResourceLimits::default();

        // Register host functions for plugin interaction
        Self::register_host_functions(&mut linker)?;

        Ok(Self {
            engine,
            linker,
            limits,
        })
    }

    /// Load and execute a WASM plugin from file
    pub fn load_wasm_plugin<P: AsRef<Path>>(
        &self,
        path: P,
        plugin_name: String,
        permissions: PluginPermissions,
    ) -> Result<LoadedPlugin> {
        let path = path.as_ref();
        
        // Read and validate WASM file
        let wasm_bytes = fs::read(path)
            .with_context(|| format!("Failed to read WASM file: {}", path.display()))?;

        // Parse and validate WASM module
        let module = Module::new(&self.engine, &wasm_bytes)
            .with_context(|| format!("Failed to parse WASM module: {}", path.display()))?;

        // Create host state
        let host_state = PluginHostState {
            registrar: PluginRegistrar::new(),
            security_context: SecurityContext::new_restricted(),
            permissions,
            plugin_name: plugin_name.clone(),
        };

        // Create store with host state
        let mut store = Store::new(&self.engine, host_state);

        // Apply resource limits (commented out - wasmi 0.34 doesn't use limiters like this)
        // store.limiter(|state| &mut state.permissions);

        // Instantiate the module directly
        let instance = self.linker
            .instantiate(&mut store, &module)
            .with_context(|| format!("Failed to instantiate WASM module: {plugin_name}"))?
            .start(&mut store)
            .with_context(|| format!("Failed to start WASM instance: {plugin_name}"))?;

        Ok(LoadedPlugin {
            instance,
            store,
            plugin_name,
        })
    }

    /// Register host functions that plugins can call
    fn register_host_functions(linker: &mut Linker<PluginHostState>) -> Result<()> {
        // Plugin registration function
        linker.func_wrap(
            "nxsh",
            "register_command",
            |caller: Caller<'_, PluginHostState>, name_ptr: i32, name_len: i32, desc_ptr: i32, desc_len: i32| -> Result<i32, wasmi::Error> {
                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| wasmi::Error::new("Missing memory export"))?;

                // Read command name from WASM memory
                let name_bytes = memory.data(&caller)
                    .get(name_ptr as usize..(name_ptr + name_len) as usize)
                    .ok_or_else(|| wasmi::Error::new("Invalid name pointer"))?;
                let name = String::from_utf8_lossy(name_bytes).to_string();

                // Read description from WASM memory
                let desc_bytes = memory.data(&caller)
                    .get(desc_ptr as usize..(desc_ptr + desc_len) as usize)
                    .ok_or_else(|| wasmi::Error::new("Invalid description pointer"))?;
                let description = String::from_utf8_lossy(desc_bytes).to_string();

                // Register command with host
                let host_state = caller.data();
                
                // Create command registration request
                let command_info = crate::registrar::CommandInfo {
                    name: name.clone(),
                    description: description.clone(),
                    plugin_name: host_state.plugin_name.clone(),
                    usage: format!("Usage: {name}"),
                    examples: vec![],
                };

                // Register through the registrar
                if let Err(e) = host_state.registrar.register_command(&command_info) {
                    log::warn!("Failed to register command '{}' from plugin '{}': {}", 
                              name, host_state.plugin_name, e);
                    return Ok(1); // Error code
                }

                log::info!("Plugin '{}' successfully registered command: {} - {}", 
                          host_state.plugin_name, name, description);
                Ok(0) // Success
            },
        )?;

        // Logging functions
        linker.func_wrap(
            "nxsh",
            "log_info",
            |caller: Caller<'_, PluginHostState>, msg_ptr: i32, msg_len: i32| -> Result<(), wasmi::Error> {
                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| wasmi::Error::new("Missing memory export"))?;

                let msg_bytes = memory.data(&caller)
                    .get(msg_ptr as usize..(msg_ptr + msg_len) as usize)
                    .ok_or_else(|| wasmi::Error::new("Invalid message pointer"))?;
                let message = String::from_utf8_lossy(msg_bytes);

                let host_state = caller.data();
                log::info!("[Plugin:{}] {}", host_state.plugin_name, message);
                Ok(())
            },
        )?;

        linker.func_wrap(
            "nxsh",
            "log_error",
            |caller: Caller<'_, PluginHostState>, msg_ptr: i32, msg_len: i32| -> Result<(), wasmi::Error> {
                let memory = caller
                    .get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| wasmi::Error::new("Missing memory export"))?;

                let msg_bytes = memory.data(&caller)
                    .get(msg_ptr as usize..(msg_ptr + msg_len) as usize)
                    .ok_or_else(|| wasmi::Error::new("Invalid message pointer"))?;
                let message = String::from_utf8_lossy(msg_bytes);

                let host_state = caller.data();
                log::error!("[Plugin:{}] {}", host_state.plugin_name, message);
                Ok(())
            },
        )?;

        Ok(())
    }
}

impl LoadedPlugin {
    /// Initialize the plugin by calling its init function
    pub fn initialize(&mut self) -> Result<()> {
        // Try to get the plugin's init function
        let init_func = self.instance
            .get_func(&self.store, "nx_plugin_init")
            .context("Plugin does not export nx_plugin_init function")?;

        // Call the init function
        let mut results = vec![Val::I32(0)];
        init_func
            .call(&mut self.store, &[], &mut results)
            .with_context(|| format!("Failed to initialize plugin: {}", self.plugin_name))?;

        // Check return value
        if let Val::I32(code) = results[0] {
            if code != 0 {
                return Err(anyhow!("Plugin initialization failed with code: {}", code));
            }
        }

        log::info!("Successfully initialized plugin: {}", self.plugin_name);
        Ok(())
    }

    /// Execute a plugin command
    pub fn execute_command(&mut self, command: &str, _args: &[String]) -> Result<i32> {
        // Try to get the plugin's execute function
        let execute_func = self.instance
            .get_func(&self.store, "nx_plugin_execute")
            .ok_or_else(|| anyhow!("Plugin does not export nx_plugin_execute function"))?;

        // For simplicity, we'll pass a dummy command ID
        // In a real implementation, you'd serialize the command and args into WASM memory
        let command_id = command.as_bytes().iter().fold(0i32, |acc, &b| acc.wrapping_add(b as i32));

        let mut results = vec![Val::I32(0)];
        execute_func
            .call(&mut self.store, &[Val::I32(command_id)], &mut results)
            .with_context(|| format!("Failed to execute command '{}' in plugin: {}", command, self.plugin_name))?;

        if let Val::I32(exit_code) = results[0] {
            Ok(exit_code)
        } else {
            Err(anyhow!("Plugin execute function returned unexpected value type"))
        }
    }

    /// Cleanup and shutdown the plugin
    pub fn shutdown(&mut self) -> Result<()> {
        // Try to get the plugin's cleanup function
        match self.instance.get_func(&self.store, "nx_plugin_cleanup") {
            Some(cleanup_func) => {
                let mut results = vec![Val::I32(0)];
                cleanup_func
                    .call(&mut self.store, &[], &mut results)
                    .with_context(|| format!("Failed to cleanup plugin: {}", self.plugin_name))?;
            }
            None => {
                // Plugin doesn't have cleanup function, that's OK
                log::debug!("Plugin '{}' has no cleanup function", self.plugin_name);
            }
        }

        log::info!("Plugin '{}' shutdown complete", self.plugin_name);
        Ok(())
    }

    /// Get plugin name
    pub fn name(&self) -> &str {
        &self.plugin_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_wasm_loader_creation() {
        let loader = WasmPluginLoader::new();
        assert!(loader.is_ok());
    }

    #[test] 
    fn test_invalid_wasm_file() {
        let loader = WasmPluginLoader::new().unwrap();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"invalid wasm content").unwrap();

        let result = loader.load_wasm_plugin(
            temp_file.path(),
            "test_plugin".to_string(),
            PluginPermissions::default(),
        );
        assert!(result.is_err());
    }
} 