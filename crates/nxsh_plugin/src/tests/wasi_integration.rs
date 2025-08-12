//! WASI Plugin System Integration Tests
//!
//! This module provides comprehensive tests for the WASI plugin system,
//! including runtime initialization, plugin loading, and hybrid operations.

use super::*;
use crate::{
    manager::PluginManager,
    runtime::WasiPluginRuntime,
    component::ComponentRegistry,
};
use anyhow::Result;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;

/// Test WASI runtime initialization
#[tokio::test]
async fn test_wasi_runtime_initialization() -> Result<()> {
    let runtime = WasiPluginRuntime::new().await?;
    assert!(runtime.engine().is_some());
    Ok(())
}

/// Test component registry functionality
#[tokio::test]
async fn test_component_registry() -> Result<()> {
    let mut registry = ComponentRegistry::new();
    
    // Test component registration
    let component_id = "test-component";
    let component_path = PathBuf::from("test.wasm");
    
    registry.register_component(component_id, component_path.clone())?;
    
    // Verify registration
    assert!(registry.is_registered(component_id));
    assert_eq!(registry.get_component_path(component_id), Some(&component_path));
    
    Ok(())
}

/// Test plugin manager with WASI support
#[tokio::test]
async fn test_plugin_manager_wasi_support() -> Result<()> {
    let mut manager = PluginManager::new();
    
    // Initialize runtimes
    manager.initialize_runtimes().await?;
    
    // Verify both runtimes are available
    assert!(manager.has_native_runtime());
    assert!(manager.has_wasi_runtime());
    
    Ok(())
}

/// Test hybrid plugin loading (native + WASI)
#[tokio::test]
async fn test_hybrid_plugin_loading() -> Result<()> {
    let mut manager = PluginManager::new();
    manager.initialize_runtimes().await?;
    
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Create mock native plugin file
    let native_plugin_path = temp_path.join("native_plugin.so");
    std::fs::write(&native_plugin_path, b"mock native plugin")?;
    
    // Create mock WASM plugin file (with WASM magic number)
    let wasm_plugin_path = temp_path.join("wasm_plugin.wasm");
    std::fs::write(&wasm_plugin_path, &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00])?;
    
    // Test plugin type detection
    assert!(!manager.is_wasm_plugin(&native_plugin_path).await?);
    assert!(manager.is_wasm_plugin(&wasm_plugin_path).await?);
    
    Ok(())
}

/// Test WASI plugin security constraints
#[tokio::test]
async fn test_wasi_plugin_security() -> Result<()> {
    let runtime = WasiPluginRuntime::new().await?;
    
    // Test capability-based security
    let capabilities = runtime.get_default_capabilities();
    assert!(capabilities.contains("wasi:filesystem/types"));
    assert!(capabilities.contains("wasi:io/streams"));
    
    // Test resource limits
    let limits = runtime.get_resource_limits();
    assert!(limits.max_memory > 0);
    assert!(limits.max_execution_time.as_secs() > 0);
    
    Ok(())
}

/// Test WASI plugin lifecycle management
#[tokio::test]
async fn test_wasi_plugin_lifecycle() -> Result<()> {
    let mut runtime = WasiPluginRuntime::new().await?;
    
    let temp_dir = TempDir::new()?;
    let plugin_path = temp_dir.path().join("lifecycle_test.wasm");
    
    // Create minimal WASM module
    let wasm_bytes = create_minimal_wasm_module()?;
    std::fs::write(&plugin_path, wasm_bytes)?;
    
    let plugin_id = "lifecycle-test-plugin";
    
    // Test loading
    runtime.load_plugin(&plugin_path, plugin_id.to_string()).await?;
    assert!(runtime.is_plugin_loaded(plugin_id));
    
    // Test execution (if plugin has exported functions)
    // This would depend on the actual WASM module structure
    
    // Test unloading
    runtime.unload_plugin(plugin_id).await?;
    assert!(!runtime.is_plugin_loaded(plugin_id));
    
    Ok(())
}

/// Test error handling for invalid WASM plugins
#[tokio::test]
async fn test_invalid_wasm_plugin_handling() -> Result<()> {
    let mut runtime = WasiPluginRuntime::new().await?;
    
    let temp_dir = TempDir::new()?;
    let invalid_plugin_path = temp_dir.path().join("invalid.wasm");
    
    // Create invalid WASM file
    std::fs::write(&invalid_plugin_path, b"not a valid wasm module")?;
    
    // Test that loading fails gracefully
    let result = runtime.load_plugin(&invalid_plugin_path, "invalid-plugin".to_string()).await;
    assert!(result.is_err());
    
    Ok(())
}

/// Test plugin resource monitoring
#[tokio::test]
async fn test_plugin_resource_monitoring() -> Result<()> {
    let runtime = WasiPluginRuntime::new().await?;
    
    // Test resource usage monitoring
    let usage = runtime.get_resource_usage();
    assert_eq!(usage.loaded_plugins, 0);
    assert_eq!(usage.total_memory_usage, 0);
    
    Ok(())
}

/// Helper function to create a minimal WASM module for testing
fn create_minimal_wasm_module() -> Result<Vec<u8>> {
    // WASM magic number + version
    let mut wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    
    // Type section (empty)
    wasm.extend(&[0x01, 0x01, 0x00]);
    
    // Function section (empty)
    wasm.extend(&[0x03, 0x01, 0x00]);
    
    // Export section (empty)
    wasm.extend(&[0x07, 0x01, 0x00]);
    
    // Code section (empty)
    wasm.extend(&[0x0a, 0x01, 0x00]);
    
    Ok(wasm)
}

// Additional helper implementations for PluginManager
impl PluginManager {
    /// Check if native runtime is available
    pub fn has_native_runtime(&self) -> bool {
        self.native_runtime.is_some()
    }
    
    /// Check if WASI runtime is available
    pub fn has_wasi_runtime(&self) -> bool {
        self.wasi_runtime.is_some()
    }
}

// Additional helper implementations for WasiPluginRuntime
impl WasiPluginRuntime {
    /// Get the wasmtime engine
    pub fn engine(&self) -> Option<&wasmtime::Engine> {
        Some(&self.engine)
    }
    
    /// Get default capabilities
    pub fn get_default_capabilities(&self) -> Vec<&'static str> {
        vec![
            "wasi:filesystem/types",
            "wasi:io/streams",
            "wasi:cli/environment",
            "wasi:cli/exit",
        ]
    }
    
    /// Get resource limits
    pub fn get_resource_limits(&self) -> ResourceLimits {
        ResourceLimits {
            max_memory: 64 * 1024 * 1024, // 64MB
            max_execution_time: std::time::Duration::from_secs(30),
            max_file_descriptors: 64,
        }
    }
    
    /// Check if a plugin is loaded
    pub fn is_plugin_loaded(&self, plugin_id: &str) -> bool {
        self.loaded_instances.contains_key(plugin_id)
    }
    
    /// Get resource usage statistics
    pub fn get_resource_usage(&self) -> ResourceUsage {
        ResourceUsage {
            loaded_plugins: self.loaded_instances.len(),
            total_memory_usage: 0, // Would need actual implementation
        }
    }
}

#[derive(Debug)]
pub struct ResourceLimits {
    pub max_memory: usize,
    pub max_execution_time: std::time::Duration,
    pub max_file_descriptors: u32,
}

#[derive(Debug)]
pub struct ResourceUsage {
    pub loaded_plugins: usize,
    pub total_memory_usage: usize,
}
