//! Enhanced Plugin Runtime Integration
//!
//! This module integrates the advanced resource table and dynamic loading
//! capabilities with the existing WASI plugin runtime.

use anyhow::{Result, Context};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
    path::Path,
};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::{
    runtime::{WasiPluginRuntime, PluginState},
    resource_table::{AdvancedResourceTable, ResourceLimits, ResourceType, ResourceCallback, ResourceInfo, MemoryUsage},
    dynamic_loader::{DynamicPluginLoader, LoaderConfig, ReloadCallback, LoadedPluginInfo},
    PluginMetadata, PluginResult, PluginError,
};

/// Enhanced plugin runtime with advanced resource management and dynamic loading
#[derive(Debug)]
pub struct EnhancedPluginRuntime {
    /// Core WASI runtime
    core_runtime: Arc<WasiPluginRuntime>,
    /// Advanced resource table
    resource_table: Arc<AdvancedResourceTable>,
    /// Dynamic plugin loader
    dynamic_loader: Arc<DynamicPluginLoader>,
    /// Runtime configuration
    config: RuntimeConfig,
    /// Performance monitor
    performance_monitor: Arc<RwLock<PerformanceMonitor>>,
    /// Plugin lifecycle hooks
    lifecycle_hooks: Arc<RwLock<Vec<Box<dyn LifecycleHook + Send + Sync>>>>,
}

/// Enhanced runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Resource management settings
    pub resource_limits: ResourceLimits,
    /// Dynamic loading settings
    pub loader_config: LoaderConfig,
    /// Performance monitoring settings
    pub performance_monitoring: PerformanceConfig,
    /// Security settings
    pub security: SecurityConfig,
    /// Optimization settings
    pub optimization: OptimizationConfig,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable performance monitoring
    pub enabled: bool,
    /// Metrics collection interval
    pub collection_interval: Duration,
    /// Memory usage threshold for warnings
    pub memory_warning_threshold: f64,
    /// CPU usage threshold for warnings
    pub cpu_warning_threshold: f64,
    /// Enable detailed profiling
    pub enable_profiling: bool,
    /// Maximum number of performance samples to keep
    pub max_samples: usize,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable sandbox isolation
    pub enable_sandbox: bool,
    /// Allow network access
    pub allow_network: bool,
    /// Allow file system access
    pub allow_filesystem: bool,
    /// Allowed directories for file access
    pub allowed_directories: Vec<String>,
    /// Maximum execution time per plugin call
    pub max_execution_time: Duration,
    /// Enable capability-based security
    pub capability_based_security: bool,
}

/// Optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Enable JIT compilation
    pub enable_jit: bool,
    /// Enable SIMD optimizations
    pub enable_simd: bool,
    /// Enable multi-threading
    pub enable_multithreading: bool,
    /// Memory pool size for allocations
    pub memory_pool_size: u64,
    /// Enable resource pooling
    pub enable_resource_pooling: bool,
    /// Cache compiled modules
    pub enable_module_cache: bool,
}

/// Performance monitoring system
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Plugin performance metrics
    plugin_metrics: HashMap<String, PluginPerformanceMetrics>,
    /// System-wide metrics
    system_metrics: SystemMetrics,
    /// Performance samples history
    samples: Vec<PerformanceSample>,
    /// Monitoring start time
    start_time: SystemTime,
}

/// Plugin-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPerformanceMetrics {
    pub plugin_id: String,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub average_execution_time: Duration,
    pub max_execution_time: Duration,
    pub min_execution_time: Duration,
    pub memory_usage: u64,
    pub peak_memory_usage: u64,
    pub cpu_usage_percent: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub error_rate: f64,
    pub throughput: f64, // calls per second
}

/// System-wide performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_plugins_loaded: u64,
    pub active_plugins: u64,
    pub total_memory_usage: u64,
    pub available_memory: u64,
    pub cpu_usage_percent: f64,
    pub uptime: Duration,
    pub garbage_collections: u64,
    pub total_api_calls: u64,
    pub failed_api_calls: u64,
}

/// Performance sample point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSample {
    pub timestamp: SystemTime,
    pub plugin_id: Option<String>,
    pub operation: String,
    pub duration: Duration,
    pub memory_before: u64,
    pub memory_after: u64,
    pub success: bool,
    pub metadata: HashMap<String, String>,
}

/// Plugin lifecycle hook trait
pub trait LifecycleHook {
    /// Called before plugin initialization
    fn before_init(&self, plugin_id: &str, metadata: &PluginMetadata) -> Result<()>;
    
    /// Called after plugin initialization
    fn after_init(&self, plugin_id: &str, metadata: &PluginMetadata) -> Result<()>;
    
    /// Called before plugin execution
    fn before_execute(&self, plugin_id: &str, function: &str) -> Result<()>;
    
    /// Called after plugin execution
    fn after_execute(&self, plugin_id: &str, function: &str, duration: Duration) -> Result<()>;
    
    /// Called when plugin fails
    fn on_error(&self, plugin_id: &str, error: &str) -> Result<()>;
    
    /// Called before plugin cleanup
    fn before_cleanup(&self, plugin_id: &str) -> Result<()>;
    
    /// Called after plugin cleanup
    fn after_cleanup(&self, plugin_id: &str) -> Result<()>;
}

/// Resource monitoring callback
struct ResourceMonitorCallback {
    performance_monitor: Arc<RwLock<PerformanceMonitor>>,
}

/// Hot reload monitoring callback
struct HotReloadCallback {
    performance_monitor: Arc<RwLock<PerformanceMonitor>>,
}

impl EnhancedPluginRuntime {
    /// Create a new enhanced plugin runtime
    pub async fn new(config: RuntimeConfig) -> Result<Self> {
        // Create core runtime
        let core_runtime = Arc::new(WasiPluginRuntime::new()?);
        
        // Create advanced resource table
        let resource_table = Arc::new(AdvancedResourceTable::new(config.resource_limits.clone())?);
        
        // Create dynamic loader
        let dynamic_loader = Arc::new(DynamicPluginLoader::new(config.loader_config.clone())?);
        
        // Create performance monitor
        let performance_monitor = Arc::new(RwLock::new(PerformanceMonitor::new()));
        
        let runtime = Self {
            core_runtime,
            resource_table,
            dynamic_loader,
            config,
            performance_monitor,
            lifecycle_hooks: Arc::new(RwLock::new(Vec::new())),
        };
        
        // Initialize components
        runtime.initialize().await?;
        
        Ok(runtime)
    }

    /// Initialize the enhanced runtime
    async fn initialize(&self) -> Result<()> {
        // Initialize core runtime
        let mut core_runtime = self.core_runtime.clone();
        let runtime_ref = Arc::get_mut(&mut core_runtime)
            .ok_or_else(|| anyhow::anyhow!("Failed to get mutable reference to core runtime"))?;
        runtime_ref.initialize().await?;
        
        // Initialize dynamic loader with runtime reference
        self.dynamic_loader.initialize(self.core_runtime.clone()).await?;
        
        // Setup resource monitoring
        self.setup_resource_monitoring().await?;
        
        // Setup hot reload monitoring
        self.setup_hot_reload_monitoring().await?;
        
        // Start performance monitoring
        if self.config.performance_monitoring.enabled {
            self.start_performance_monitoring().await?;
        }
        
        log::info!("Enhanced plugin runtime initialized successfully");
        Ok(())
    }

    /// Load a plugin with enhanced features
    pub async fn load_plugin<P: AsRef<Path>>(&self, path: P, plugin_id: String) -> Result<PluginMetadata> {
        let start_time = SystemTime::now();
        
        // Execute before init hooks
        self.execute_lifecycle_hooks(|hook| hook.before_init(&plugin_id, &PluginMetadata::default())).await?;
        
        // Load plugin using dynamic loader
        self.dynamic_loader.load_plugin(&plugin_id, None).await
            .context("Failed to load plugin with dynamic loader")?;
        
        // Get plugin metadata
        let metadata = self.core_runtime.get_plugin_metadata(&plugin_id).await
            .ok_or_else(|| anyhow::anyhow!("Plugin metadata not found after loading"))?;
        
        // Initialize performance tracking
        {
            let mut monitor = self.performance_monitor.write().await;
            monitor.initialize_plugin_metrics(&plugin_id);
        }
        
        // Record performance sample
        self.record_performance_sample(
            Some(plugin_id.clone()),
            "plugin_load".to_string(),
            start_time.elapsed().unwrap_or(Duration::ZERO),
            true,
            HashMap::new(),
        ).await;
        
        // Execute after init hooks
        self.execute_lifecycle_hooks(|hook| hook.after_init(&plugin_id, &metadata)).await?;
        
        log::info!("Enhanced plugin '{}' loaded successfully", plugin_id);
        Ok(metadata)
    }

    /// Execute a plugin function with enhanced monitoring
    pub async fn execute_plugin(
        &self,
        plugin_id: &str,
        function: &str,
        args: &[u8],
    ) -> Result<Vec<u8>> {
        let start_time = SystemTime::now();
        
        // Execute before execute hooks
        self.execute_lifecycle_hooks(|hook| hook.before_execute(plugin_id, function)).await?;
        
        // Record memory before execution
        let memory_before = self.get_plugin_memory_usage(plugin_id).await.unwrap_or(0);
        
        // Execute plugin function
        let result = self.core_runtime.execute_plugin(plugin_id, function, args).await;
        
        let duration = start_time.elapsed().unwrap_or(Duration::ZERO);
        let success = result.is_ok();
        
        // Record memory after execution
        let memory_after = self.get_plugin_memory_usage(plugin_id).await.unwrap_or(0);
        
        // Update performance metrics
        {
            let mut monitor = self.performance_monitor.write().await;
            monitor.update_plugin_metrics(plugin_id, duration, success, memory_after);
        }
        
        // Record performance sample
        let mut metadata = HashMap::new();
        metadata.insert("function".to_string(), function.to_string());
        metadata.insert("memory_delta".to_string(), 
                        (memory_after as i64 - memory_before as i64).to_string());
        
        self.record_performance_sample(
            Some(plugin_id.to_string()),
            format!("execute_{}", function),
            duration,
            success,
            metadata,
        ).await;
        
        // Execute after execute hooks
        self.execute_lifecycle_hooks(|hook| hook.after_execute(plugin_id, function, duration)).await?;
        
        // Handle errors
        if let Err(ref e) = result {
            self.execute_lifecycle_hooks(|hook| hook.on_error(plugin_id, &e.to_string())).await?;
        }
        
        result
    }

    /// Unload a plugin with cleanup
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        // Execute before cleanup hooks
        self.execute_lifecycle_hooks(|hook| hook.before_cleanup(plugin_id)).await?;
        
        // Cleanup plugin resources
        self.cleanup_plugin_resources(plugin_id).await?;
        
        // Unload from dynamic loader
        self.dynamic_loader.unload_plugin(plugin_id).await?;
        
        // Remove performance metrics
        {
            let mut monitor = self.performance_monitor.write().await;
            monitor.remove_plugin_metrics(plugin_id);
        }
        
        // Execute after cleanup hooks
        self.execute_lifecycle_hooks(|hook| hook.after_cleanup(plugin_id)).await?;
        
        log::info!("Enhanced plugin '{}' unloaded successfully", plugin_id);
        Ok(())
    }

    /// Get comprehensive performance metrics
    pub async fn get_performance_metrics(&self) -> Result<PerformanceReport> {
        let monitor = self.performance_monitor.read().await;
        
        Ok(PerformanceReport {
            system_metrics: monitor.system_metrics.clone(),
            plugin_metrics: monitor.plugin_metrics.clone(),
            uptime: monitor.start_time.elapsed().unwrap_or(Duration::ZERO),
            sample_count: monitor.samples.len(),
            memory_usage: self.resource_table.get_memory_statistics().await?,
        })
    }

    /// Trigger garbage collection for all plugins
    pub async fn garbage_collect(&self) -> Result<GarbageCollectionReport> {
        let start_time = SystemTime::now();
        let mut total_freed = 0;
        let mut plugin_results = HashMap::new();
        
        // Get list of loaded plugins
        let loaded_plugins = self.dynamic_loader.list_loaded_plugins().await;
        
        // Perform garbage collection for each plugin
        for plugin_id in loaded_plugins {
            match self.resource_table.garbage_collect_plugin(&plugin_id).await {
                Ok(result) => {
                    total_freed += result.freed_memory;
                    plugin_results.insert(plugin_id, result);
                }
                Err(e) => {
                    log::warn!("Garbage collection failed for plugin {}: {}", plugin_id, e);
                }
            }
        }
        
        let duration = start_time.elapsed().unwrap_or(Duration::ZERO);
        
        // Update system metrics
        {
            let mut monitor = self.performance_monitor.write().await;
            monitor.system_metrics.garbage_collections += 1;
        }
        
        log::info!("Garbage collection completed: freed {} bytes in {:?}", total_freed, duration);
        
        Ok(GarbageCollectionReport {
            total_freed_memory: total_freed,
            duration,
            plugin_results,
        })
    }

    /// Add lifecycle hook
    pub async fn add_lifecycle_hook(&self, hook: Box<dyn LifecycleHook + Send + Sync>) -> Result<()> {
        let mut hooks = self.lifecycle_hooks.write().await;
        hooks.push(hook);
        Ok(())
    }

    /// Get loaded plugin information
    pub async fn get_plugin_info(&self, plugin_id: &str) -> Option<LoadedPluginInfo> {
        self.dynamic_loader.get_loaded_plugin_info(plugin_id).await
    }

    /// List all loaded plugins
    pub async fn list_plugins(&self) -> Vec<String> {
        self.dynamic_loader.list_loaded_plugins().await
    }

    // Private implementation methods

    async fn setup_resource_monitoring(&self) -> Result<()> {
        let callback = Box::new(ResourceMonitorCallback {
            performance_monitor: self.performance_monitor.clone(),
        });
        
        self.resource_table.add_callback(callback).await?;
        Ok(())
    }

    async fn setup_hot_reload_monitoring(&self) -> Result<()> {
        let callback = Box::new(HotReloadCallback {
            performance_monitor: self.performance_monitor.clone(),
        });
        
        self.dynamic_loader.add_reload_callback(callback).await?;
        Ok(())
    }

    async fn start_performance_monitoring(&self) -> Result<()> {
        let monitor = self.performance_monitor.clone();
        let interval = self.config.performance_monitoring.collection_interval;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                let mut monitor_guard = monitor.write().await;
                monitor_guard.collect_system_metrics().await;
                monitor_guard.cleanup_old_samples();
            }
        });
        
        Ok(())
    }

    async fn get_plugin_memory_usage(&self, plugin_id: &str) -> Result<u64> {
        if let Some(memory_info) = self.resource_table.get_plugin_memory_info(plugin_id).await? {
            Ok(memory_info.current_usage)
        } else {
            Ok(0)
        }
    }

    async fn cleanup_plugin_resources(&self, plugin_id: &str) -> Result<()> {
        // Trigger garbage collection for the specific plugin
        self.resource_table.garbage_collect_plugin(plugin_id).await?;
        Ok(())
    }

    async fn record_performance_sample(
        &self,
        plugin_id: Option<String>,
        operation: String,
        duration: Duration,
        success: bool,
        metadata: HashMap<String, String>,
    ) {
        let sample = PerformanceSample {
            timestamp: SystemTime::now(),
            plugin_id,
            operation,
            duration,
            memory_before: 0, // Will be filled by caller if available
            memory_after: 0,  // Will be filled by caller if available
            success,
            metadata,
        };
        
        let mut monitor = self.performance_monitor.write().await;
        monitor.add_sample(sample);
    }

    async fn execute_lifecycle_hooks<F>(&self, operation: F) -> Result<()>
    where
        F: Fn(&dyn LifecycleHook) -> Result<()>,
    {
        let hooks = self.lifecycle_hooks.read().await;
        for hook in hooks.iter() {
            if let Err(e) = operation(hook.as_ref()) {
                log::warn!("Lifecycle hook failed: {}", e);
            }
        }
        Ok(())
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            plugin_metrics: HashMap::new(),
            system_metrics: SystemMetrics {
                total_plugins_loaded: 0,
                active_plugins: 0,
                total_memory_usage: 0,
                available_memory: 0,
                cpu_usage_percent: 0.0,
                uptime: Duration::ZERO,
                garbage_collections: 0,
                total_api_calls: 0,
                failed_api_calls: 0,
            },
            samples: Vec::new(),
            start_time: SystemTime::now(),
        }
    }

    pub fn initialize_plugin_metrics(&mut self, plugin_id: &str) {
        let metrics = PluginPerformanceMetrics {
            plugin_id: plugin_id.to_string(),
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            average_execution_time: Duration::ZERO,
            max_execution_time: Duration::ZERO,
            min_execution_time: Duration::MAX,
            memory_usage: 0,
            peak_memory_usage: 0,
            cpu_usage_percent: 0.0,
            cache_hits: 0,
            cache_misses: 0,
            error_rate: 0.0,
            throughput: 0.0,
        };
        
        self.plugin_metrics.insert(plugin_id.to_string(), metrics);
        self.system_metrics.total_plugins_loaded += 1;
        self.system_metrics.active_plugins += 1;
    }

    pub fn update_plugin_metrics(&mut self, plugin_id: &str, duration: Duration, success: bool, memory_usage: u64) {
        if let Some(metrics) = self.plugin_metrics.get_mut(plugin_id) {
            metrics.total_calls += 1;
            
            if success {
                metrics.successful_calls += 1;
            } else {
                metrics.failed_calls += 1;
            }
            
            // Update execution time statistics
            if duration > metrics.max_execution_time {
                metrics.max_execution_time = duration;
            }
            if duration < metrics.min_execution_time {
                metrics.min_execution_time = duration;
            }
            
            // Update average execution time
            let total_time = metrics.average_execution_time.as_nanos() * (metrics.total_calls - 1) as u128 + duration.as_nanos();
            metrics.average_execution_time = Duration::from_nanos((total_time / metrics.total_calls as u128) as u64);
            
            // Update memory usage
            metrics.memory_usage = memory_usage;
            if memory_usage > metrics.peak_memory_usage {
                metrics.peak_memory_usage = memory_usage;
            }
            
            // Update error rate
            metrics.error_rate = metrics.failed_calls as f64 / metrics.total_calls as f64;
        }
        
        // Update system metrics
        self.system_metrics.total_api_calls += 1;
        if !success {
            self.system_metrics.failed_api_calls += 1;
        }
    }

    pub fn remove_plugin_metrics(&mut self, plugin_id: &str) {
        self.plugin_metrics.remove(plugin_id);
        self.system_metrics.active_plugins = self.system_metrics.active_plugins.saturating_sub(1);
    }

    pub async fn collect_system_metrics(&mut self) {
        // Update uptime
        self.system_metrics.uptime = self.start_time.elapsed().unwrap_or(Duration::ZERO);
        
        // Update active plugins count
        self.system_metrics.active_plugins = self.plugin_metrics.len() as u64;
        
        // Update total memory usage
        self.system_metrics.total_memory_usage = self.plugin_metrics.values()
            .map(|m| m.memory_usage)
            .sum();
    }

    pub fn add_sample(&mut self, sample: PerformanceSample) {
        self.samples.push(sample);
    }

    pub fn cleanup_old_samples(&mut self) {
        const MAX_SAMPLES: usize = 10000;
        if self.samples.len() > MAX_SAMPLES {
            self.samples.drain(0..self.samples.len() - MAX_SAMPLES);
        }
    }
}

impl ResourceCallback for ResourceMonitorCallback {
    fn on_resource_created(&self, _resource: &ResourceInfo) -> Result<()> {
        // Could update resource creation metrics here
        Ok(())
    }

    fn on_resource_accessed(&self, _resource: &ResourceInfo) -> Result<()> {
        // Could update resource access metrics here
        Ok(())
    }

    fn on_resource_destroyed(&self, _resource: &ResourceInfo) -> Result<()> {
        // Could update resource destruction metrics here
        Ok(())
    }

    fn on_memory_pressure(&self, _usage: &MemoryUsage) -> Result<()> {
        // Could trigger garbage collection or emit warnings
        log::warn!("Memory pressure detected");
        Ok(())
    }
}

impl ReloadCallback for HotReloadCallback {
    fn before_reload(&self, _plugin_id: &str) -> Result<()> {
        // Could pause metrics collection
        Ok(())
    }

    fn after_reload(&self, plugin_id: &str, _old_version: &semver::Version, _new_version: &semver::Version) -> Result<()> {
        log::info!("Plugin {} hot reloaded successfully", plugin_id);
        Ok(())
    }

    fn reload_failed(&self, plugin_id: &str, error: &str) -> Result<()> {
        log::error!("Plugin {} hot reload failed: {}", plugin_id, error);
        Ok(())
    }
}

/// Comprehensive performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub system_metrics: SystemMetrics,
    pub plugin_metrics: HashMap<String, PluginPerformanceMetrics>,
    pub uptime: Duration,
    pub sample_count: usize,
    pub memory_usage: MemoryUsage,
}

/// Garbage collection report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageCollectionReport {
    pub total_freed_memory: u64,
    pub duration: Duration,
    pub plugin_results: HashMap<String, crate::resource_table::GarbageCollectionResult>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            resource_limits: ResourceLimits::default(),
            loader_config: LoaderConfig::default(),
            performance_monitoring: PerformanceConfig::default(),
            security: SecurityConfig::default(),
            optimization: OptimizationConfig::default(),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: Duration::from_secs(10),
            memory_warning_threshold: 0.8,
            cpu_warning_threshold: 0.9,
            enable_profiling: false,
            max_samples: 10000,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_sandbox: true,
            allow_network: false,
            allow_filesystem: false,
            allowed_directories: vec![],
            max_execution_time: Duration::from_secs(30),
            capability_based_security: true,
        }
    }
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            enable_jit: true,
            enable_simd: true,
            enable_multithreading: false,
            memory_pool_size: 64 * 1024 * 1024, // 64MB
            enable_resource_pooling: true,
            enable_module_cache: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_enhanced_runtime_creation() {
        let config = RuntimeConfig::default();
        let runtime = EnhancedPluginRuntime::new(config).await.expect("Failed to create enhanced runtime");
        
        let plugins = runtime.list_plugins().await;
        assert_eq!(plugins.len(), 0);
    }

    #[tokio::test]
    async fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        
        monitor.initialize_plugin_metrics("test_plugin");
        assert_eq!(monitor.plugin_metrics.len(), 1);
        assert_eq!(monitor.system_metrics.active_plugins, 1);
        
        monitor.update_plugin_metrics("test_plugin", Duration::from_millis(100), true, 1024);
        
        let metrics = monitor.plugin_metrics.get("test_plugin").expect("Plugin metrics should exist");
        assert_eq!(metrics.total_calls, 1);
        assert_eq!(metrics.successful_calls, 1);
        assert_eq!(metrics.memory_usage, 1024);
    }

    #[tokio::test]
    async fn test_garbage_collection() {
        let config = RuntimeConfig::default();
        let runtime = EnhancedPluginRuntime::new(config).await.expect("Failed to create enhanced runtime");
        
        let report = runtime.garbage_collect().await.expect("Failed to run garbage collection");
        assert_eq!(report.plugin_results.len(), 0); // No plugins loaded
    }
}
