//! Pure Rust WebAssembly Component Model Implementation
//! 
//! This module provides a simplified component model for NexusShell plugins
//! using Pure Rust components without C dependencies.

use anyhow::{Result, Context, anyhow};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    fmt,
};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use wasmi::{Engine, Store, Module, Instance};
use log::{info, warn, error, debug};

use crate::{
    security::SecurityContext,
    permissions::PluginPermissions,
    registrar::PluginRegistrar,
};

/// Component registry error
#[derive(Debug, thiserror::Error)]
pub enum ComponentError {
    #[error("Component not found: {0}")]
    NotFound(String),
    
    #[error("Invalid component: {0}")]
    Invalid(String),
    
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type ComponentResult<T> = std::result::Result<T, ComponentError>;

/// Pure Rust Component Registry
pub struct ComponentRegistry {
    engine: Engine,
    components: Arc<RwLock<HashMap<String, RegisteredComponent>>>,
    component_types: Arc<RwLock<HashMap<String, ComponentType>>>,
    interface_generator: ComponentInterfaceGenerator,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        
        Ok(Self {
            engine,
            components: Arc::new(RwLock::new(HashMap::new())),
            component_types: Arc::new(RwLock::new(HashMap::new())),
            interface_generator: ComponentInterfaceGenerator::new(),
        })
    }

    /// Register a WebAssembly component from bytes
    pub async fn register_component_from_bytes(
        &self,
        component_id: String,
        component_bytes: &[u8],
        metadata: ComponentMetadata,
    ) -> ComponentResult<()> {
        // Parse component as WASM module
        let module = Module::new(&self.engine, component_bytes)
            .map_err(|e| ComponentError::Invalid(format!("Failed to parse component: {}", e)))?;

        // Analyze component exports and imports
        let component_type = self.analyze_component_type(&module)?;

        // Create registered component
        let component = RegisteredComponent {
            id: component_id.clone(),
            module,
            metadata,
            registered_at: chrono::Utc::now(),
        };

        // Store component and type info
        {
            let mut components = self.components.write().await;
            components.insert(component_id.clone(), component);
        }
        
        {
            let mut component_types = self.component_types.write().await;
            component_types.insert(component_id.clone(), component_type);
        }
        
        info!("Component '{}' registered successfully", component_id);
        Ok(())
    }

    /// Register a component from file
    pub async fn register_component_from_file<P: AsRef<Path>>(
        &self,
        component_id: String,
        path: P,
        metadata: ComponentMetadata,
    ) -> ComponentResult<()> {
        let path = path.as_ref();
        let component_bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read component file: {}", path.display()))?;

        self.register_component_from_bytes(component_id, &component_bytes, metadata).await
    }

    /// Unregister a component
    pub async fn unregister_component(&self, component_id: &str) -> ComponentResult<()> {
        {
            let mut components = self.components.write().await;
            components.remove(component_id);
        }
        
        {
            let mut component_types = self.component_types.write().await;
            component_types.remove(component_id);
        }
        
        info!("Component '{}' unregistered successfully", component_id);
        Ok(())
    }

    /// Execute a function in a component
    pub async fn execute_component_function(
        &self,
        component_id: &str,
        function_name: &str,
        args: &[ComponentValue],
    ) -> ComponentResult<Vec<ComponentValue>> {
        let component = {
            let components = self.components.read().await;
            components.get(component_id)
                .ok_or_else(|| ComponentError::NotFound(component_id.to_string()))?
                .clone()
        };

        // Create component state
        let state = ComponentState::new();
        let mut store = Store::new(&self.engine, state);

        // Instantiate component
        let instance = Instance::new(&mut store, &component.module, &[])
            .map_err(|e| ComponentError::ExecutionFailed(format!("Failed to instantiate component: {}", e)))?;

        // Execute function
        self.execute_function(&mut store, &instance, function_name, args).await
    }

    /// Get component metadata
    pub async fn get_component_metadata(&self, component_id: &str) -> Option<ComponentMetadata> {
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

    /// Analyze component type from module
    fn analyze_component_type(&self, module: &Module) -> ComponentResult<ComponentType> {
        let mut exports = HashMap::new();
        let mut imports = HashMap::new();

        // Analyze module exports
        for export in module.exports() {
            let export_type = match export.ty() {
                wasmi::ExternType::Func(func_type) => {
                    ComponentExportType::Function {
                        params: func_type.params().iter().map(|p| self.value_type_to_component_type(*p)).collect(),
                        results: func_type.results().iter().map(|r| self.value_type_to_component_type(*r)).collect(),
                    }
                },
                wasmi::ExternType::Table(_) => ComponentExportType::Table,
                wasmi::ExternType::Memory(_) => ComponentExportType::Memory,
                wasmi::ExternType::Global(_) => ComponentExportType::Global,
            };
            exports.insert(export.name().to_string(), export_type);
        }

        // Analyze module imports
        for import in module.imports() {
            let import_type = match import.ty() {
                wasmi::ExternType::Func(func_type) => {
                    ComponentImportType::Function {
                        params: func_type.params().iter().map(|p| self.value_type_to_component_type(*p)).collect(),
                        results: func_type.results().iter().map(|r| self.value_type_to_component_type(*r)).collect(),
                    }
                },
                wasmi::ExternType::Table(_) => ComponentImportType::Table,
                wasmi::ExternType::Memory(_) => ComponentImportType::Memory,
                wasmi::ExternType::Global(_) => ComponentImportType::Global,
            };
            imports.insert(format!("{}::{}", import.module(), import.name()), import_type);
        }

        Ok(ComponentType {
            exports,
            imports,
        })
    }

    /// Convert WASM value type to component type
    fn value_type_to_component_type(&self, value_type: wasmi::core::ValueType) -> ComponentValueType {
        match value_type {
            wasmi::core::ValueType::I32 => ComponentValueType::S32,
            wasmi::core::ValueType::I64 => ComponentValueType::S64,
            wasmi::core::ValueType::F32 => ComponentValueType::Float32,
            wasmi::core::ValueType::F64 => ComponentValueType::Float64,
        }
    }

    /// Execute function in component instance
    async fn execute_function(
        &self,
        store: &mut Store<ComponentState>,
        instance: &Instance,
        function_name: &str,
        args: &[ComponentValue],
    ) -> ComponentResult<Vec<ComponentValue>> {
        if let Some(func) = instance.get_func(store, function_name) {
            // Convert arguments with formal pointer handling for complex values
            let wasm_args: Vec<wasmi::Value> = self
                .prepare_wasm_args(store, instance, args)
                .map_err(|e| ComponentError::ExecutionFailed(format!("Argument preparation failed: {}", e)))?;

            // Prepare results
            let mut results = vec![wasmi::Value::I32(0); func.ty(store).results().len()];

            // Execute function
            func.call(store, &wasm_args, &mut results)
                .map_err(|e| ComponentError::ExecutionFailed(format!("Function execution failed: {}", e)))?;

            // Convert results back
            let component_results: Vec<ComponentValue> = results.iter()
                .map(|val| self.wasm_value_to_component_value(val))
                .collect();

            return Ok(component_results);
        }

        // Fallback to host-dispatched functions
        self.dispatch_host_function(store, function_name, args).await
    }

    /// Prepare WASM arguments, allocating memory for strings/lists when possible.
    fn prepare_wasm_args(
        &self,
        store: &mut Store<ComponentState>,
        instance: &Instance,
        args: &[ComponentValue],
    ) -> Result<Vec<wasmi::Value>> {
        let mut wasm_args: Vec<wasmi::Value> = Vec::with_capacity(args.len());

        for arg in args {
            match arg {
                ComponentValue::String(text) => {
                    match self.alloc_and_write_bytes(store, instance, text.as_bytes()) {
                        Ok(ptr) => wasm_args.push(wasmi::Value::I32(ptr)),
                        Err(err) => {
                            // As a safe fallback, use 0 pointer placeholder
                            log::warn!("[component] could not allocate for string arg ({}), passing 0 pointer: {}", function_name, err);
                            wasm_args.push(wasmi::Value::I32(0));
                        }
                    }
                }
                ComponentValue::List(items) => {
                    // Encode list as JSON and pass pointer to buffer when possible
                    let json = serde_json::to_vec(items)
                        .map_err(|e| anyhow!("failed to encode list as JSON: {}", e))?;
                    match self.alloc_and_write_bytes(store, instance, &json) {
                        Ok(ptr) => wasm_args.push(wasmi::Value::I32(ptr)),
                        Err(err) => {
                            log::warn!("[component] could not allocate for list arg, passing 0 pointer: {}", err);
                            wasm_args.push(wasmi::Value::I32(0));
                        }
                    }
                }
                other => wasm_args.push(self.component_value_to_wasm_value(other)?),
            }
        }

        Ok(wasm_args)
    }

    /// Allocate memory in guest and write bytes, returning pointer. Tries export `alloc(len)->ptr`,
    /// falls back to writing at scratch area if memory is accessible. Returns error on failure.
    fn alloc_and_write_bytes(
        &self,
        store: &mut Store<ComponentState>,
        instance: &Instance,
        bytes: &[u8],
    ) -> Result<i32> {
        let memory = instance
            .get_export(store, "memory")
            .and_then(|e| e.into_memory())
            .ok_or_else(|| anyhow!("no exported memory found"))?;

        // Prefer an exported `alloc` function: (i32 len) -> i32 ptr
        if let Some(alloc_func) = instance.get_func(store, "alloc") {
            let ty = alloc_func.ty(store);
            if ty.params().len() == 1 && ty.results().len() == 1 {
                if matches!(ty.params()[0], wasmi::core::ValueType::I32)
                    && matches!(ty.results()[0], wasmi::core::ValueType::I32)
                {
                    let mut results = [wasmi::Value::I32(0)];
                    alloc_func
                        .call(store, &[wasmi::Value::I32(bytes.len() as i32)], &mut results)
                        .map_err(|e| anyhow!("alloc call failed: {}", e))?;
                    let ptr = match results[0] { wasmi::Value::I32(p) => p, _ => 0 };
                    if ptr <= 0 { return Err(anyhow!("alloc returned invalid pointer")); }
                    memory
                        .write(store, ptr as usize, bytes)
                        .map_err(|_e| anyhow!("memory write failed"))?;
                    return Ok(ptr);
                }
            }
        }

        // Fallback: write to a fixed scratch region if large enough
        let scratch_ptr: usize = 1024; // leave low memory for runtime
        let data = memory.data_mut(store);
        if scratch_ptr + bytes.len() <= data.len() {
            data[scratch_ptr..scratch_ptr + bytes.len()].copy_from_slice(bytes);
            return Ok(scratch_ptr as i32);
        }

        Err(anyhow!("insufficient memory for write ({} bytes)", bytes.len()))
    }

    /// Built-in host function dispatcher for WASM → host calls
    async fn dispatch_host_function(
        &self,
        _store: &mut Store<ComponentState>,
        function_name: &str,
        args: &[ComponentValue],
    ) -> ComponentResult<Vec<ComponentValue>> {
        match function_name {
            "host.log" => {
                if let Some(ComponentValue::String(msg)) = args.get(0) {
                    log::info!("[plugin host.log] {}", msg);
                    Ok(vec![])
                } else {
                    Err(ComponentError::TypeMismatch { expected: "string".into(), actual: format!("{:?}", args.get(0)) })
                }
            }
            "host.env_get" => {
                if let Some(ComponentValue::String(key)) = args.get(0) {
                    let val = std::env::var(key).unwrap_or_default();
                    Ok(vec![ComponentValue::String(val)])
                } else {
                    Err(ComponentError::TypeMismatch { expected: "string".into(), actual: format!("{:?}", args.get(0)) })
                }
            }
            "host.time_now_ms" => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as i64;
                Ok(vec![ComponentValue::S64(now)])
            }
            other if other.starts_with("host.") => Err(ComponentError::NotFound(other.to_string())),
            other => Err(ComponentError::NotFound(other.to_string())),
        }
    }

    /// Convert component value to WASM value
    fn component_value_to_wasm_value(&self, value: &ComponentValue) -> Result<wasmi::Value> {
        match value {
            ComponentValue::Bool(b) => Ok(wasmi::Value::I32(if *b { 1 } else { 0 })),
            ComponentValue::S8(i) => Ok(wasmi::Value::I32(*i as i32)),
            ComponentValue::U8(u) => Ok(wasmi::Value::I32(*u as i32)),
            ComponentValue::S16(i) => Ok(wasmi::Value::I32(*i as i32)),
            ComponentValue::U16(u) => Ok(wasmi::Value::I32(*u as i32)),
            ComponentValue::S32(i) => Ok(wasmi::Value::I32(*i)),
            ComponentValue::U32(u) => Ok(wasmi::Value::I32(*u as i32)),
            ComponentValue::S64(i) => Ok(wasmi::Value::I64(*i)),
            ComponentValue::U64(u) => Ok(wasmi::Value::I64(*u as i64)),
            ComponentValue::Float32(f) => Ok(wasmi::Value::F32((*f).into())),
            ComponentValue::Float64(f) => Ok(wasmi::Value::F64((*f).into())),
            ComponentValue::String(_) => {
                // String handling would require memory management
                // For simplicity, return 0 (pointer placeholder)
                Ok(wasmi::Value::I32(0))
            },
            ComponentValue::List(_) => {
                // List handling would require memory management
                // For simplicity, return 0 (pointer placeholder)
                Ok(wasmi::Value::I32(0))
            },
        }
    }

    /// Convert WASM value to component value
    fn wasm_value_to_component_value(&self, value: &wasmi::Value) -> ComponentValue {
        match value {
            wasmi::Value::I32(i) => ComponentValue::S32(*i),
            wasmi::Value::I64(i) => ComponentValue::S64(*i),
            wasmi::Value::F32(f) => ComponentValue::Float32(f.to_float()),
            wasmi::Value::F64(f) => ComponentValue::Float64(f.to_float()),
        }
    }
}

/// Registered component
#[derive(Debug, Clone)]
pub struct RegisteredComponent {
    pub id: String,
    pub module: Module,
    pub metadata: ComponentMetadata,
    pub registered_at: chrono::DateTime<chrono::Utc>,
}

/// Component metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: Option<String>,
    pub homepage: Option<String>,
}

impl Default for ComponentMetadata {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            version: "0.0.0".to_string(),
            description: "No description".to_string(),
            author: "Unknown".to_string(),
            license: None,
            homepage: None,
        }
    }
}

/// Component type information
#[derive(Debug, Clone)]
pub struct ComponentType {
    pub exports: HashMap<String, ComponentExportType>,
    pub imports: HashMap<String, ComponentImportType>,
}

/// Component export type
#[derive(Debug, Clone)]
pub enum ComponentExportType {
    Function {
        params: Vec<ComponentValueType>,
        results: Vec<ComponentValueType>,
    },
    Table,
    Memory,
    Global,
}

/// Component import type
#[derive(Debug, Clone)]
pub enum ComponentImportType {
    Function {
        params: Vec<ComponentValueType>,
        results: Vec<ComponentValueType>,
    },
    Table,
    Memory,
    Global,
}

/// Component value types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentValueType {
    Bool,
    S8,
    U8,
    S16,
    U16,
    S32,
    U32,
    S64,
    U64,
    Float32,
    Float64,
    String,
    List(Box<ComponentValueType>),
}

impl fmt::Display for ComponentValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentValueType::Bool => write!(f, "bool"),
            ComponentValueType::S8 => write!(f, "s8"),
            ComponentValueType::U8 => write!(f, "u8"),
            ComponentValueType::S16 => write!(f, "s16"),
            ComponentValueType::U16 => write!(f, "u16"),
            ComponentValueType::S32 => write!(f, "s32"),
            ComponentValueType::U32 => write!(f, "u32"),
            ComponentValueType::S64 => write!(f, "s64"),
            ComponentValueType::U64 => write!(f, "u64"),
            ComponentValueType::Float32 => write!(f, "float32"),
            ComponentValueType::Float64 => write!(f, "float64"),
            ComponentValueType::String => write!(f, "string"),
            ComponentValueType::List(inner) => write!(f, "list<{}>", inner),
        }
    }
}

/// Component values
#[derive(Debug, Clone, PartialEq)]
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

impl fmt::Display for ComponentValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentValue::Bool(b) => write!(f, "{}", b),
            ComponentValue::S8(i) => write!(f, "{}", i),
            ComponentValue::U8(u) => write!(f, "{}", u),
            ComponentValue::S16(i) => write!(f, "{}", i),
            ComponentValue::U16(u) => write!(f, "{}", u),
            ComponentValue::S32(i) => write!(f, "{}", i),
            ComponentValue::U32(u) => write!(f, "{}", u),
            ComponentValue::S64(i) => write!(f, "{}", i),
            ComponentValue::U64(u) => write!(f, "{}", u),
            ComponentValue::Float32(fl) => write!(f, "{}", fl),
            ComponentValue::Float64(fl) => write!(f, "{}", fl),
            ComponentValue::String(s) => write!(f, "\"{}\"", s),
            ComponentValue::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            },
        }
    }
}

/// Component execution state
#[derive(Debug)]
pub struct ComponentState {
    pub security_context: SecurityContext,
    pub permissions: PluginPermissions,
    pub registrar: PluginRegistrar,
}

impl ComponentState {
    pub fn new() -> Self {
        Self {
            security_context: SecurityContext::new_restricted(),
            permissions: PluginPermissions::default(),
            registrar: PluginRegistrar::new(),
        }
    }
}

/// Component interface generator
pub struct ComponentInterfaceGenerator {
    wit_definitions: HashMap<String, String>,
}

impl ComponentInterfaceGenerator {
    /// Create a new interface generator
    pub fn new() -> Self {
        Self {
            wit_definitions: HashMap::new(),
        }
    }

    /// Add WIT world definition
    pub fn add_world_definition(&mut self, world_name: String, definition: String) {
        self.wit_definitions.insert(world_name, definition);
    }

    /// Generate component interface bindings
    pub fn generate_bindings(&self, world_name: &str) -> Result<String> {
        let wit_definition = self.wit_definitions.get(world_name)
            .ok_or_else(|| anyhow!("WIT world '{}' not found", world_name))?;

        // Generate Rust bindings from WIT definition
        // This is a simplified implementation
        let bindings = format!(
            r#"// Generated bindings for world '{}'
// Based on WIT definition:
// {}

#[allow(unused)]
pub mod bindings {{
    use super::*;
    
    // Host functions would be generated here
    pub fn call_host_function(name: &str, _args: &[ComponentValue]) -> Result<Vec<ComponentValue>> {{
        // Generated bindings are not directly wired; real dispatch happens in
        // ComponentRegistry::dispatch_host_function. Keep this as explicit error.
        Err(anyhow!(format!("host function not available in generated bindings: {}", name)))
    }}
    
    // Export functions would be generated here
    pub fn register_exports() -> Vec<(&'static str, fn(&[ComponentValue]) -> Result<Vec<ComponentValue>>)> {{
        // Implementation would return actual exported functions
        vec![]
    }}
}}
"#,
            world_name, wit_definition
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
        let converted_back = registry.wasm_value_to_component_value(&wasmi::Value::I32(0));
        
        // Since string conversion is simplified, we test with integers
        let int_value = ComponentValue::S32(42);
        let wasm_val = registry.component_value_to_wasm_value(&int_value).unwrap();
        let converted_back = registry.wasm_value_to_component_value(&wasm_val);
        
        match converted_back {
            ComponentValue::S32(i) => assert_eq!(i, 42),
            other => panic!("Expected S32 value, got {:?}", other),
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

    #[test]
    fn test_component_value_display() {
        let value = ComponentValue::String("hello".to_string());
        assert_eq!(format!("{}", value), "\"hello\"");
        
        let list_value = ComponentValue::List(vec![
            ComponentValue::S32(1),
            ComponentValue::S32(2),
            ComponentValue::S32(3),
        ]);
        assert_eq!(format!("{}", list_value), "[1, 2, 3]");
    }
}
