use anyhow::{Result, Context};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;
use wasmtime::{
    component::{Component, ComponentType, Instance, Linker, ResourceTable, Val},
    Config, Engine, Store,
};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};
use serde::{Deserialize, Serialize};
use wit_component::ComponentEncoder;
use wit_bindgen::generate;

use crate::{PluginError, PluginMetadata, PluginResult};

/// Component Model registry for managing WebAssembly components
pub struct ComponentRegistry {
    engine: Engine,
    linker: Arc<RwLock<Linker<ComponentState>>>,
    components: Arc<RwLock<HashMap<String, RegisteredComponent>>>,
    component_types: Arc<RwLock<HashMap<String, ComponentType>>>,
    bindings: Arc<RwLock<HashMap<String, ComponentBinding>>>,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Result<Self> {
        // Configure engine for component model
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        config.epoch_interruption(true);
        config.consume_fuel(true);
        
        let engine = Engine::new(&config)
            .context("Failed to create WebAssembly engine")?;
        
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
        
        // Add WASI host functions
        wasmtime_wasi::add_to_linker_async(&mut linker)
            .context("Failed to add WASI host functions")?;
        
        // Add custom host functions for NexusShell
        self.add_nexus_host_functions(&mut linker).await?;
        
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
        let component_bytes = tokio::fs::read(&path).await
            .map_err(|e| PluginError::LoadError(format!("Failed to read component file: {}", e)))?;
        
        // Validate component format
        let component = Component::from_binary(&self.engine, &component_bytes)
            .map_err(|e| PluginError::LoadError(format!("Invalid component format: {}", e)))?;
        
        // Extract component type information
        let component_type = component.component_type();
        
        // Create component binding
        let binding = self.create_component_binding(&component_id, &component_type, &metadata).await?;
        
        // Register component
        let registered_component = RegisteredComponent {
            id: component_id.clone(),
            component,
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
            component_types.insert(component_id.clone(), component_type);
        }
        
        {
            let mut bindings = self.bindings.write().await;
            bindings.insert(component_id.clone(), binding);
        }
        
        log::info!("Component '{}' registered successfully", component_id);
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
        
        log::info!("Component '{}' unregistered successfully", component_id);
        Ok(())
    }
    
    /// Execute a function in a WebAssembly component
    pub async fn execute_component_function(
        &self,
        component_id: &str,
        function_name: &str,
        args: &[ComponentValue],
    ) -> PluginResult<Vec<ComponentValue>> {
        let (component, binding) = {
            let components = self.components.read().await;
            let bindings = self.bindings.read().await;
            
            let component = components.get(component_id)
                .ok_or_else(|| PluginError::NotFound(format!("Component '{}' not found", component_id)))?;
            let binding = bindings.get(component_id)
                .ok_or_else(|| PluginError::NotFound(format!("Binding for component '{}' not found", component_id)))?;
            
            (component.component.clone(), binding.clone())
        };
        
        // Create store with component state
        let mut store = Store::new(&self.engine, ComponentState::new()?);
        store.set_fuel(1_000_000)
            .map_err(|e| PluginError::ExecutionError(format!("Failed to set fuel: {}", e)))?;
        
        // Instantiate component
        let linker = self.linker.read().await;
        let instance = linker.instantiate_async(&mut store, &component).await
            .map_err(|e| PluginError::ExecutionError(format!("Failed to instantiate component: {}", e)))?;
        
        // Execute function
        let result = self.execute_function(&mut store, &instance, &binding, function_name, args).await?;
        
        Ok(result)
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
    pub async fn get_component_type(&self, component_id: &str) -> Option<ComponentType> {
        let component_types = self.component_types.read().await;
        component_types.get(component_id).cloned()
    }
    
    /// Create component from WIT definition
    pub async fn create_component_from_wit(
        &self,
        wit_source: &str,
        wasm_module: &[u8],
    ) -> Result<Vec<u8>> {
        let encoder = ComponentEncoder::default()
            .validate(true)
            .module(wasm_module)
            .context("Failed to set module")?;
        
        // Add WIT world definition
        let component_bytes = encoder
            .encode()
            .context("Failed to encode component")?;
        
        Ok(component_bytes)
    }
    
    // Private helper methods
    
    async fn add_nexus_host_functions(
        &self,
        linker: &mut Linker<ComponentState>,
    ) -> Result<()> {
        // Add shell-specific host functions
        linker.func_wrap_async(
            "nexus:shell/command",
            "execute",
            |mut caller: wasmtime::Caller<'_, ComponentState>, cmd: String| {
                Box::new(async move {
                    // Execute shell command
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
        
        linker.func_wrap_async(
            "nexus:shell/fs",
            "read-file",
            |mut caller: wasmtime::Caller<'_, ComponentState>, path: String| {
                Box::new(async move {
                    let content = tokio::fs::read_to_string(&path).await
                        .map_err(|e| wasmtime::Error::msg(format!("Failed to read file: {}", e)))?;
                    Ok(content)
                })
            },
        )?;
        
        linker.func_wrap_async(
            "nexus:shell/fs",
            "write-file",
            |mut caller: wasmtime::Caller<'_, ComponentState>, path: String, content: String| {
                Box::new(async move {
                    tokio::fs::write(&path, &content).await
                        .map_err(|e| wasmtime::Error::msg(format!("Failed to write file: {}", e)))?;
                    Ok(())
                })
            },
        )?;
        
        Ok(())
    }
    
    async fn create_component_binding(
        &self,
        component_id: &str,
        component_type: &ComponentType,
        metadata: &PluginMetadata,
    ) -> PluginResult<ComponentBinding> {
        let mut exports = HashMap::new();
        let mut imports = HashMap::new();
        
        // Extract export information
        for (name, export_type) in component_type.exports(&self.engine) {
            exports.insert(name.to_string(), export_type);
        }
        
        // Extract import information
        for (name, import_type) in component_type.imports(&self.engine) {
            imports.insert(name.to_string(), import_type);
        }
        
        Ok(ComponentBinding {
            component_id: component_id.to_string(),
            exports,
            imports,
            metadata: metadata.clone(),
        })
    }
    
    async fn execute_function(
        &self,
        store: &mut Store<ComponentState>,
        instance: &Instance,
        binding: &ComponentBinding,
        function_name: &str,
        args: &[ComponentValue],
    ) -> PluginResult<Vec<ComponentValue>> {
        // Get function export
        let func = instance.get_func(store, function_name)
            .ok_or_else(|| PluginError::ExecutionError(format!("Function '{}' not found", function_name)))?;
        
        // Convert arguments to wasmtime values
        let wasmtime_args: Vec<Val> = args.iter()
            .map(|arg| self.component_value_to_val(arg))
            .collect::<Result<Vec<_>, _>>()?;
        
        // Prepare result buffer
        let mut results = vec![Val::Bool(false); func.ty(store).results().len()];
        
        // Execute function
        func.call_async(store, &wasmtime_args, &mut results).await
            .map_err(|e| PluginError::ExecutionError(format!("Function execution failed: {}", e)))?;
        
        // Convert results back to component values
        let component_results: Vec<ComponentValue> = results.iter()
            .map(|val| self.val_to_component_value(val))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(component_results)
    }
    
    fn component_value_to_val(&self, value: &ComponentValue) -> Result<Val> {
        match value {
            ComponentValue::Bool(b) => Ok(Val::Bool(*b)),
            ComponentValue::S8(i) => Ok(Val::S8(*i)),
            ComponentValue::U8(u) => Ok(Val::U8(*u)),
            ComponentValue::S16(i) => Ok(Val::S16(*i)),
            ComponentValue::U16(u) => Ok(Val::U16(*u)),
            ComponentValue::S32(i) => Ok(Val::S32(*i)),
            ComponentValue::U32(u) => Ok(Val::U32(*u)),
            ComponentValue::S64(i) => Ok(Val::S64(*i)),
            ComponentValue::U64(u) => Ok(Val::U64(*u)),
            ComponentValue::Float32(f) => Ok(Val::Float32(*f)),
            ComponentValue::Float64(f) => Ok(Val::Float64(*f)),
            ComponentValue::String(s) => Ok(Val::String(s.clone().into_boxed_str())),
            ComponentValue::List(items) => {
                // Convert list items
                let converted_items: Result<Vec<Val>> = items.iter()
                    .map(|item| self.component_value_to_val(item))
                    .collect();
                Ok(Val::List(converted_items?))
            }
        }
    }
    
    fn val_to_component_value(&self, val: &Val) -> Result<ComponentValue> {
        match val {
            Val::Bool(b) => Ok(ComponentValue::Bool(*b)),
            Val::S8(i) => Ok(ComponentValue::S8(*i)),
            Val::U8(u) => Ok(ComponentValue::U8(*u)),
            Val::S16(i) => Ok(ComponentValue::S16(*i)),
            Val::U16(u) => Ok(ComponentValue::U16(*u)),
            Val::S32(i) => Ok(ComponentValue::S32(*i)),
            Val::U32(u) => Ok(ComponentValue::U32(*u)),
            Val::S64(i) => Ok(ComponentValue::S64(*i)),
            Val::U64(u) => Ok(ComponentValue::U64(*u)),
            Val::Float32(f) => Ok(ComponentValue::Float32(*f)),
            Val::Float64(f) => Ok(ComponentValue::Float64(*f)),
            Val::String(s) => Ok(ComponentValue::String(s.to_string())),
            Val::List(items) => {
                let converted_items: Result<Vec<ComponentValue>> = items.iter()
                    .map(|item| self.val_to_component_value(item))
                    .collect();
                Ok(ComponentValue::List(converted_items?))
            }
            _ => Err(anyhow::anyhow!("Unsupported component value type")),
        }
    }
}

/// Registered WebAssembly component
#[derive(Debug, Clone)]
pub struct RegisteredComponent {
    pub id: String,
    pub component: Component,
    pub metadata: PluginMetadata,
    pub path: PathBuf,
    pub registered_at: chrono::DateTime<chrono::Utc>,
}

/// Component binding information
#[derive(Debug, Clone)]
pub struct ComponentBinding {
    pub component_id: String,
    pub exports: HashMap<String, wasmtime::component::types::ComponentItem>,
    pub imports: HashMap<String, wasmtime::component::types::ComponentItem>,
    pub metadata: PluginMetadata,
}

/// Component execution state
pub struct ComponentState {
    wasi_ctx: WasiCtx,
    resource_table: ResourceTable,
}

impl ComponentState {
    pub fn new() -> Result<Self> {
        let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_env()
            .build();
        
        Ok(Self {
            wasi_ctx,
            resource_table: ResourceTable::new(),
        })
    }
}

impl WasiView for ComponentState {
    fn ctx(&self) -> &WasiCtx {
        &self.wasi_ctx
    }
    
    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
    
    fn table(&self) -> &ResourceTable {
        &self.resource_table
    }
    
    fn table_mut(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
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
        let wit_definition = self.wit_definitions.get(world_name)
            .ok_or_else(|| anyhow::anyhow!("WIT world '{}' not found", world_name))?;
        
        // Generate Rust bindings using wit-bindgen
        // This would typically involve calling wit-bindgen programmatically
        // For now, we'll return a placeholder
        Ok(format!("// Generated bindings for world '{}'\n{}", world_name, wit_definition))
    }
    
    /// Load WIT definitions from directory
    pub fn load_wit_definitions<P: AsRef<Path>>(&mut self, wit_dir: P) -> Result<()> {
        let wit_dir = wit_dir.as_ref();
        
        for entry in std::fs::read_dir(wit_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("wit") {
                let content = std::fs::read_to_string(&path)?;
                let name = path.file_stem()
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
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_component_registry_creation() {
        let registry = ComponentRegistry::new().unwrap();
        assert_eq!(registry.list_components().await.len(), 0);
    }
    
    #[tokio::test]
    async fn test_component_value_conversion() {
        let registry = ComponentRegistry::new().unwrap();
        
        let value = ComponentValue::String("test".to_string());
        let val = registry.component_value_to_val(&value).unwrap();
        let converted_back = registry.val_to_component_value(&val).unwrap();
        
        match converted_back {
            ComponentValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Unexpected value type"),
        }
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