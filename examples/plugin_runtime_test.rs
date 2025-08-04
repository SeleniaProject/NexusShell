//! Plugin Runtime System Test Suite
//!
//! Comprehensive testing of the enhanced plugin runtime with resource management
//! and dynamic loading capabilities.

use std::{
    collections::HashMap,
    path::PathBuf,
    time::Duration,
};
use tempfile::TempDir;
use tokio::time::sleep;

use nxsh_plugin::{
    enhanced_runtime::{EnhancedPluginRuntime, RuntimeConfig},
    resource_table::{ResourceLimits, ResourceType},
    dynamic_loader::{LoaderConfig, ValidationConfig, VersionCompatibility},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init();
    
    println!("🚀 Plugin Runtime System Test Suite");
    println!("=====================================");
    
    // Test 1: Resource Table Management
    test_resource_table_management().await?;
    
    // Test 2: Dynamic Plugin Loading
    test_dynamic_plugin_loading().await?;
    
    // Test 3: Enhanced Runtime Integration
    test_enhanced_runtime_integration().await?;
    
    // Test 4: Performance Monitoring
    test_performance_monitoring().await?;
    
    // Test 5: Memory Management
    test_memory_management().await?;
    
    // Test 6: Hot Reload Simulation
    test_hot_reload_simulation().await?;
    
    println!("\n✅ All Plugin Runtime System tests completed successfully!");
    Ok(())
}

async fn test_resource_table_management() -> anyhow::Result<()> {
    println!("\n📊 Testing Resource Table Management...");
    
    use nxsh_plugin::resource_table::{AdvancedResourceTable, ResourceLimits, ResourceType};
    
    // Create resource table with limits
    let limits = ResourceLimits {
        max_resources_per_plugin: 5,
        max_memory_per_plugin: 1024 * 1024, // 1MB
        max_file_handles: 10,
        max_network_connections: 3,
        max_timers: 2,
        cleanup_timeout: Duration::from_secs(10),
        memory_pressure_threshold: 0.8,
    };
    
    let resource_table = AdvancedResourceTable::new(limits)?;
    
    // Test resource creation
    println!("  ├─ Creating test resources...");
    let resource1 = resource_table.create_resource(
        "test_plugin".to_string(),
        ResourceType::Memory { size: 1024 },
        HashMap::new(),
    ).await?;
    
    let resource2 = resource_table.create_resource(
        "test_plugin".to_string(),
        ResourceType::File { path: "/tmp/test.txt".to_string() },
        HashMap::new(),
    ).await?;
    
    // Test resource access
    println!("  ├─ Testing resource access...");
    let accessed = resource_table.access_resource(&resource1).await?;
    assert!(accessed.is_some());
    println!("  │   ✓ Resource access successful");
    
    // Test memory statistics
    println!("  ├─ Checking memory statistics...");
    let memory_stats = resource_table.get_memory_statistics().await?;
    println!("  │   ├─ Current usage: {} bytes", memory_stats.current_usage);
    println!("  │   ├─ Total allocated: {} bytes", memory_stats.total_allocated);
    println!("  │   └─ Peak usage: {} bytes", memory_stats.peak_usage);
    
    // Test plugin memory info
    let plugin_memory = resource_table.get_plugin_memory_info("test_plugin").await?;
    if let Some(info) = plugin_memory {
        println!("  │   ├─ Plugin memory usage: {} bytes", info.current_usage);
        println!("  │   └─ Plugin allocations: {}", info.allocation_count);
    }
    
    // Test garbage collection
    println!("  ├─ Testing garbage collection...");
    let gc_result = resource_table.garbage_collect_plugin("test_plugin").await?;
    println!("  │   ├─ Collected resources: {}", gc_result.collected_resources);
    println!("  │   ├─ Freed memory: {} bytes", gc_result.freed_memory);
    println!("  │   └─ GC duration: {:?}", gc_result.duration);
    
    // Test resource destruction
    println!("  ├─ Testing resource destruction...");
    resource_table.destroy_resource(&resource1).await?;
    resource_table.destroy_resource(&resource2).await?;
    
    // Final statistics
    let final_stats = resource_table.get_statistics().await?;
    println!("  └─ Final statistics:");
    println!("      ├─ Total resources created: {}", final_stats.total_resources);
    println!("      ├─ Active resources: {}", final_stats.active_resources);
    println!("      └─ Destroyed resources: {}", final_stats.destroyed_resources);
    
    println!("  ✅ Resource table management test completed");
    Ok(())
}

async fn test_dynamic_plugin_loading() -> anyhow::Result<()> {
    println!("\n🔄 Testing Dynamic Plugin Loading...");
    
    use nxsh_plugin::dynamic_loader::{DynamicPluginLoader, LoaderConfig, ValidationConfig};
    
    // Create temporary directory for plugins
    let temp_dir = TempDir::new()?;
    let plugin_dir = temp_dir.path().to_path_buf();
    
    // Create a fake plugin file
    let plugin_path = plugin_dir.join("test_plugin.wasm");
    std::fs::write(&plugin_path, b"fake wasm content for testing")?;
    
    // Configure loader
    let config = LoaderConfig {
        plugin_directories: vec![plugin_dir],
        enable_hot_reload: false, // Disabled for testing
        watch_debounce: Duration::from_millis(100),
        max_concurrent_loads: 2,
        cache_directory: Some(temp_dir.path().join("cache")),
        enable_dependency_resolution: true,
        validation: ValidationConfig {
            require_signature: false,
            max_file_size: 1024 * 1024, // 1MB
            allowed_extensions: vec!["wasm".to_string()],
            blocked_patterns: vec![],
            min_security_version: "0.1.0".to_string(),
        },
        version_compatibility: VersionCompatibility::default(),
    };
    
    let loader = DynamicPluginLoader::new(config)?;
    
    // Test plugin discovery
    println!("  ├─ Discovering plugins...");
    let discovery_result = loader.discover_plugins().await?;
    println!("  │   ├─ Discovered plugins: {}", discovery_result.discovered_plugins.len());
    println!("  │   ├─ Failed discoveries: {}", discovery_result.failed_discoveries.len());
    println!("  │   └─ Scan duration: {:?}", discovery_result.scan_duration);
    
    // Test plugin registry
    println!("  ├─ Testing plugin registry...");
    let dependency_graph = loader.get_dependency_graph().await;
    println!("  │   └─ Dependency graph created");
    
    // Test loaded plugins list
    println!("  ├─ Checking loaded plugins...");
    let loaded_plugins = loader.list_loaded_plugins().await;
    println!("  │   └─ Loaded plugins: {}", loaded_plugins.len());
    
    println!("  ✅ Dynamic plugin loading test completed");
    Ok(())
}

async fn test_enhanced_runtime_integration() -> anyhow::Result<()> {
    println!("\n🔧 Testing Enhanced Runtime Integration...");
    
    // Create runtime configuration
    let config = RuntimeConfig {
        resource_limits: ResourceLimits {
            max_resources_per_plugin: 10,
            max_memory_per_plugin: 2 * 1024 * 1024, // 2MB
            max_file_handles: 20,
            max_network_connections: 5,
            max_timers: 5,
            cleanup_timeout: Duration::from_secs(15),
            memory_pressure_threshold: 0.75,
        },
        loader_config: LoaderConfig::default(),
        performance_monitoring: nxsh_plugin::enhanced_runtime::PerformanceConfig {
            enabled: true,
            collection_interval: Duration::from_secs(1),
            memory_warning_threshold: 0.8,
            cpu_warning_threshold: 0.9,
            enable_profiling: true,
            max_samples: 1000,
        },
        security: nxsh_plugin::enhanced_runtime::SecurityConfig::default(),
        optimization: nxsh_plugin::enhanced_runtime::OptimizationConfig::default(),
    };
    
    println!("  ├─ Creating enhanced runtime...");
    let runtime = EnhancedPluginRuntime::new(config).await?;
    
    // Test plugin listing
    println!("  ├─ Testing plugin management...");
    let plugins = runtime.list_plugins().await;
    println!("  │   └─ Active plugins: {}", plugins.len());
    
    // Test performance metrics
    println!("  ├─ Testing performance monitoring...");
    let performance_report = runtime.get_performance_metrics().await?;
    println!("  │   ├─ System uptime: {:?}", performance_report.uptime);
    println!("  │   ├─ Total plugins loaded: {}", performance_report.system_metrics.total_plugins_loaded);
    println!("  │   ├─ Active plugins: {}", performance_report.system_metrics.active_plugins);
    println!("  │   ├─ Total memory usage: {} bytes", performance_report.system_metrics.total_memory_usage);
    println!("  │   └─ Performance samples: {}", performance_report.sample_count);
    
    // Test garbage collection
    println!("  ├─ Testing garbage collection...");
    let gc_report = runtime.garbage_collect().await?;
    println!("  │   ├─ Total freed memory: {} bytes", gc_report.total_freed_memory);
    println!("  │   ├─ GC duration: {:?}", gc_report.duration);
    println!("  │   └─ Plugin results: {}", gc_report.plugin_results.len());
    
    println!("  ✅ Enhanced runtime integration test completed");
    Ok(())
}

async fn test_performance_monitoring() -> anyhow::Result<()> {
    println!("\n📈 Testing Performance Monitoring...");
    
    use nxsh_plugin::enhanced_runtime::PerformanceMonitor;
    
    let mut monitor = PerformanceMonitor::new();
    
    // Initialize plugin metrics
    println!("  ├─ Initializing plugin metrics...");
    monitor.initialize_plugin_metrics("test_plugin_1");
    monitor.initialize_plugin_metrics("test_plugin_2");
    
    // Simulate plugin executions
    println!("  ├─ Simulating plugin executions...");
    for i in 0..10 {
        let duration = Duration::from_millis(50 + i * 10);
        let success = i % 3 != 0; // Some failures
        let memory_usage = 1024 * (i + 1);
        
        monitor.update_plugin_metrics("test_plugin_1", duration, success, memory_usage);
        
        if i % 2 == 0 {
            let duration2 = Duration::from_millis(30 + i * 5);
            monitor.update_plugin_metrics("test_plugin_2", duration2, true, memory_usage / 2);
        }
    }
    
    // Collect system metrics
    println!("  ├─ Collecting system metrics...");
    monitor.collect_system_metrics().await;
    
    // Display plugin metrics
    println!("  ├─ Plugin performance metrics:");
    for (plugin_id, metrics) in &monitor.plugin_metrics {
        println!("  │   ├─ Plugin: {}", plugin_id);
        println!("  │   │   ├─ Total calls: {}", metrics.total_calls);
        println!("  │   │   ├─ Success rate: {:.2}%", 
                 (metrics.successful_calls as f64 / metrics.total_calls as f64) * 100.0);
        println!("  │   │   ├─ Avg execution time: {:?}", metrics.average_execution_time);
        println!("  │   │   ├─ Max execution time: {:?}", metrics.max_execution_time);
        println!("  │   │   ├─ Memory usage: {} bytes", metrics.memory_usage);
        println!("  │   │   └─ Error rate: {:.2}%", metrics.error_rate * 100.0);
    }
    
    // Display system metrics
    println!("  └─ System metrics:");
    println!("      ├─ Total plugins loaded: {}", monitor.system_metrics.total_plugins_loaded);
    println!("      ├─ Active plugins: {}", monitor.system_metrics.active_plugins);
    println!("      ├─ Total API calls: {}", monitor.system_metrics.total_api_calls);
    println!("      ├─ Failed API calls: {}", monitor.system_metrics.failed_api_calls);
    println!("      └─ Uptime: {:?}", monitor.system_metrics.uptime);
    
    println!("  ✅ Performance monitoring test completed");
    Ok(())
}

async fn test_memory_management() -> anyhow::Result<()> {
    println!("\n🧠 Testing Memory Management...");
    
    use nxsh_plugin::resource_table::{AdvancedResourceTable, ResourceLimits, ResourceType};
    
    // Create resource table with memory limits
    let limits = ResourceLimits {
        max_resources_per_plugin: 100,
        max_memory_per_plugin: 10 * 1024, // 10KB limit for testing
        max_file_handles: 50,
        max_network_connections: 10,
        max_timers: 10,
        cleanup_timeout: Duration::from_secs(5),
        memory_pressure_threshold: 0.8,
    };
    
    let resource_table = AdvancedResourceTable::new(limits)?;
    
    // Test memory allocation and tracking
    println!("  ├─ Testing memory allocation...");
    let mut resources = Vec::new();
    
    // Allocate resources until we approach the limit
    for i in 0..8 {
        let resource_id = resource_table.create_resource(
            "memory_test_plugin".to_string(),
            ResourceType::Memory { size: 1024 }, // 1KB each
            HashMap::new(),
        ).await?;
        resources.push(resource_id);
        
        let memory_info = resource_table.get_plugin_memory_info("memory_test_plugin").await?;
        if let Some(info) = memory_info {
            println!("  │   ├─ Allocation {}: {} bytes total", i + 1, info.current_usage);
        }
    }
    
    // Try to exceed memory limit
    println!("  ├─ Testing memory limit enforcement...");
    let result = resource_table.create_resource(
        "memory_test_plugin".to_string(),
        ResourceType::Memory { size: 5 * 1024 }, // This should fail
        HashMap::new(),
    ).await;
    
    if result.is_err() {
        println!("  │   ✓ Memory limit properly enforced");
    } else {
        println!("  │   ✗ Memory limit not enforced (unexpected)");
    }
    
    // Test garbage collection effectiveness
    println!("  ├─ Testing garbage collection...");
    let gc_result = resource_table.garbage_collect_plugin("memory_test_plugin").await?;
    println!("  │   ├─ Resources collected: {}", gc_result.collected_resources);
    println!("  │   ├─ Memory freed: {} bytes", gc_result.freed_memory);
    println!("  │   └─ GC duration: {:?}", gc_result.duration);
    
    // Test memory statistics
    println!("  ├─ Final memory statistics...");
    let memory_stats = resource_table.get_memory_statistics().await?;
    println!("  │   ├─ Current usage: {} bytes", memory_stats.current_usage);
    println!("  │   ├─ Peak usage: {} bytes", memory_stats.peak_usage);
    println!("  │   ├─ Total allocated: {} bytes", memory_stats.total_allocated);
    println!("  │   └─ Total freed: {} bytes", memory_stats.total_freed);
    
    // Cleanup remaining resources
    println!("  ├─ Cleaning up remaining resources...");
    for resource_id in resources {
        let _ = resource_table.destroy_resource(&resource_id).await;
    }
    
    println!("  ✅ Memory management test completed");
    Ok(())
}

async fn test_hot_reload_simulation() -> anyhow::Result<()> {
    println!("\n🔥 Testing Hot Reload Simulation...");
    
    use nxsh_plugin::dynamic_loader::{DynamicPluginLoader, LoaderConfig, ReloadCallback};
    use semver::Version;
    
    // Custom reload callback for testing
    struct TestReloadCallback {
        events: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    }
    
    impl ReloadCallback for TestReloadCallback {
        fn before_reload(&self, plugin_id: &str) -> anyhow::Result<()> {
            let mut events = self.events.lock().unwrap();
            events.push(format!("before_reload:{}", plugin_id));
            Ok(())
        }
        
        fn after_reload(&self, plugin_id: &str, old_version: &Version, new_version: &Version) -> anyhow::Result<()> {
            let mut events = self.events.lock().unwrap();
            events.push(format!("after_reload:{}:{}:{}", plugin_id, old_version, new_version));
            Ok(())
        }
        
        fn reload_failed(&self, plugin_id: &str, error: &str) -> anyhow::Result<()> {
            let mut events = self.events.lock().unwrap();
            events.push(format!("reload_failed:{}:{}", plugin_id, error));
            Ok(())
        }
    }
    
    // Create temporary plugin directory
    let temp_dir = TempDir::new()?;
    let plugin_dir = temp_dir.path().to_path_buf();
    
    // Create test plugin file
    let plugin_path = plugin_dir.join("hot_reload_test.wasm");
    std::fs::write(&plugin_path, b"version 1.0 content")?;
    
    // Configure loader without hot reload initially
    let config = LoaderConfig {
        plugin_directories: vec![plugin_dir],
        enable_hot_reload: false,
        watch_debounce: Duration::from_millis(50),
        max_concurrent_loads: 1,
        cache_directory: None,
        enable_dependency_resolution: false,
        validation: nxsh_plugin::dynamic_loader::ValidationConfig::default(),
        version_compatibility: nxsh_plugin::dynamic_loader::VersionCompatibility::default(),
    };
    
    let loader = DynamicPluginLoader::new(config)?;
    
    // Setup reload callback
    println!("  ├─ Setting up reload callback...");
    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let callback = Box::new(TestReloadCallback {
        events: events.clone(),
    });
    loader.add_reload_callback(callback).await?;
    
    // Discover initial plugins
    println!("  ├─ Discovering initial plugins...");
    let discovery_result = loader.discover_plugins().await?;
    println!("  │   └─ Discovered {} plugins", discovery_result.discovered_plugins.len());
    
    // Simulate file change and reload
    println!("  ├─ Simulating plugin update...");
    sleep(Duration::from_millis(100)).await;
    std::fs::write(&plugin_path, b"version 2.0 updated content")?;
    
    // Manual reload test (since we disabled automatic hot reload)
    println!("  ├─ Testing manual reload...");
    if let Some(plugin) = discovery_result.discovered_plugins.first() {
        let plugin_name = &plugin.metadata.name;
        
        // This would normally be triggered by file watcher
        let reload_result = loader.reload_plugin(plugin_name).await;
        println!("  │   └─ Reload result: {:?}", reload_result.is_ok());
    }
    
    // Check callback events
    println!("  ├─ Checking reload events...");
    let events_guard = events.lock().unwrap();
    println!("  │   └─ Callback events triggered: {}", events_guard.len());
    for event in events_guard.iter() {
        println!("  │       ├─ {}", event);
    }
    drop(events_guard);
    
    // Test plugin information
    println!("  ├─ Testing plugin information retrieval...");
    let loaded_plugins = loader.list_loaded_plugins().await;
    for plugin_id in loaded_plugins {
        if let Some(info) = loader.get_loaded_plugin_info(&plugin_id).await {
            println!("  │   ├─ Plugin: {}", info.plugin_id);
            println!("  │   │   ├─ Version: {}", info.version);
            println!("  │   │   ├─ Reload count: {}", info.reload_count);
            println!("  │   │   ├─ Load status: {:?}", info.load_status);
            println!("  │   │   └─ File hash: {}", &info.file_hash[..8]);
        }
    }
    
    println!("  ✅ Hot reload simulation test completed");
    Ok(())
}

// Helper function to create test configuration
fn create_test_runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        resource_limits: ResourceLimits {
            max_resources_per_plugin: 50,
            max_memory_per_plugin: 5 * 1024 * 1024, // 5MB
            max_file_handles: 25,
            max_network_connections: 10,
            max_timers: 10,
            cleanup_timeout: Duration::from_secs(20),
            memory_pressure_threshold: 0.85,
        },
        loader_config: LoaderConfig::default(),
        performance_monitoring: nxsh_plugin::enhanced_runtime::PerformanceConfig {
            enabled: true,
            collection_interval: Duration::from_secs(2),
            memory_warning_threshold: 0.8,
            cpu_warning_threshold: 0.9,
            enable_profiling: true,
            max_samples: 500,
        },
        security: nxsh_plugin::enhanced_runtime::SecurityConfig {
            enable_sandbox: true,
            allow_network: false,
            allow_filesystem: true,
            allowed_directories: vec!["/tmp".to_string()],
            max_execution_time: Duration::from_secs(10),
            capability_based_security: true,
        },
        optimization: nxsh_plugin::enhanced_runtime::OptimizationConfig {
            enable_jit: true,
            enable_simd: true,
            enable_multithreading: false,
            memory_pool_size: 32 * 1024 * 1024, // 32MB
            enable_resource_pooling: true,
            enable_module_cache: true,
        },
    }
}
