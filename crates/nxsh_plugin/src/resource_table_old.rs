//! Advanced Resource Table for Plugin Runtime
//!
//! This module provides comprehensive resource management for WebAssembly plugins,
//! including memory tracking, resource limits, and lifecycle management.

use anyhow::{Result, Context};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
    time::{Duration, SystemTime},
};
use tokio::sync::{RwLock, Mutex};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Advanced resource table with memory tracking and lifecycle management
#[derive(Debug)]
pub struct AdvancedResourceTable {
    /// Core WebAssembly resource table
    core_table: Arc<RwLock<wasmtime::component::ResourceTable>>,
    /// Resource tracking by type
    resources: Arc<RwLock<HashMap<ResourceId, ResourceInfo>>>,
    /// Memory allocations tracking
    memory_tracker: Arc<Mutex<MemoryTracker>>,
    /// Resource limits configuration
    limits: ResourceLimits,
    /// Resource lifecycle callbacks
    callbacks: Arc<RwLock<Vec<Box<dyn ResourceCallback + Send + Sync>>>>,
    /// Global statistics
    statistics: Arc<RwLock<ResourceStatistics>>,
}

/// Unique resource identifier
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceId(Uuid);

/// Resource information and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub id: ResourceId,
    pub resource_type: ResourceType,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub access_count: u64,
    pub memory_usage: u64,
    pub metadata: HashMap<String, String>,
    pub plugin_id: String,
    pub state: ResourceState,
}

/// Resource type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    /// File handle resource
    File { path: String },
    /// Network socket resource
    Socket { addr: String },
    /// Memory buffer resource
    Memory { size: u64 },
    /// Timer resource
    Timer { interval: Duration },
    /// Custom plugin-defined resource
    Custom { type_name: String },
    /// WASI resource (generic)
    Wasi { handle: u32 },
}

/// Resource state in lifecycle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceState {
    /// Resource is active and usable
    Active,
    /// Resource is temporarily suspended
    Suspended,
    /// Resource is being cleaned up
    Destroying,
    /// Resource has been destroyed
    Destroyed,
    /// Resource creation failed
    Failed(String),
}

/// Memory tracking and allocation management
#[derive(Debug)]
pub struct MemoryTracker {
    /// Total allocated memory by plugin
    allocations: HashMap<String, PluginMemoryInfo>,
    /// Global memory usage
    global_usage: MemoryUsage,
    /// Memory allocation history
    allocation_history: Vec<AllocationEvent>,
    /// Memory limits per plugin
    plugin_limits: HashMap<String, u64>,
}

/// Plugin memory information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMemoryInfo {
    pub plugin_id: String,
    pub total_allocated: u64,
    pub current_usage: u64,
    pub peak_usage: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub fragmentation_ratio: f64,
    pub last_gc: Option<SystemTime>,
}

/// Global memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub total_allocated: u64,
    pub total_freed: u64,
    pub current_usage: u64,
    pub peak_usage: u64,
    pub allocations_per_second: f64,
    pub gc_frequency: f64,
}

/// Memory allocation event for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationEvent {
    pub plugin_id: String,
    pub resource_id: ResourceId,
    pub size: u64,
    pub operation: AllocationOperation,
    pub timestamp: SystemTime,
    pub stack_trace: Option<String>,
}

/// Memory allocation operation type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocationOperation {
    Allocate,
    Reallocate { old_size: u64 },
    Deallocate,
    GarbageCollect,
}

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum number of resources per plugin
    pub max_resources_per_plugin: u32,
    /// Maximum total memory per plugin (bytes)
    pub max_memory_per_plugin: u64,
    /// Maximum file handles per plugin
    pub max_file_handles: u32,
    /// Maximum network connections per plugin
    pub max_network_connections: u32,
    /// Maximum timer instances per plugin
    pub max_timers: u32,
    /// Resource cleanup timeout
    pub cleanup_timeout: Duration,
    /// Memory pressure threshold (0.0-1.0)
    pub memory_pressure_threshold: f64,
}

/// Resource statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStatistics {
    pub total_resources: u64,
    pub active_resources: u64,
    pub destroyed_resources: u64,
    pub failed_resources: u64,
    pub total_plugins: u64,
    pub memory_usage: MemoryUsage,
    pub resource_types: HashMap<String, u64>,
    pub plugin_usage: HashMap<String, PluginResourceUsage>,
}

/// Plugin resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResourceUsage {
    pub plugin_id: String,
    pub active_resources: u64,
    pub memory_usage: u64,
    pub resource_creation_rate: f64,
    pub error_rate: f64,
    pub last_active: SystemTime,
}

/// Resource lifecycle callback trait
pub trait ResourceCallback {
    /// Called when a resource is created
    fn on_resource_created(&self, resource: &ResourceInfo) -> Result<()>;
    
    /// Called when a resource is accessed
    fn on_resource_accessed(&self, resource: &ResourceInfo) -> Result<()>;
    
    /// Called when a resource is destroyed
    fn on_resource_destroyed(&self, resource: &ResourceInfo) -> Result<()>;
    
    /// Called when memory pressure is detected
    fn on_memory_pressure(&self, usage: &MemoryUsage) -> Result<()>;
}

impl AdvancedResourceTable {
    /// Create a new advanced resource table
    pub fn new(limits: ResourceLimits) -> Result<Self> {
        Ok(Self {
            core_table: Arc::new(RwLock::new(wasmtime::component::ResourceTable::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            memory_tracker: Arc::new(Mutex::new(MemoryTracker::new())),
            limits,
            callbacks: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(ResourceStatistics::new())),
        })
    }

    /// Create a new resource with tracking
    pub async fn create_resource(
        &self,
        plugin_id: String,
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
    ) -> Result<ResourceId> {
        // Check resource limits
        self.check_resource_limits(&plugin_id).await?;
        
        let resource_id = ResourceId(Uuid::new_v4());
        let now = SystemTime::now();
        
        let resource_info = ResourceInfo {
            id: resource_id.clone(),
            resource_type: resource_type.clone(),
            created_at: now,
            last_accessed: now,
            access_count: 0,
            memory_usage: self.calculate_resource_memory_usage(&resource_type),
            metadata,
            plugin_id: plugin_id.clone(),
            state: ResourceState::Active,
        };

        // Update memory tracking
        {
            let mut tracker = self.memory_tracker.lock().await;
            tracker.track_allocation(
                plugin_id.clone(),
                resource_id.clone(),
                resource_info.memory_usage,
            ).await?;
        }

        // Store resource info
        {
            let mut resources = self.resources.write().await;
            resources.insert(resource_id.clone(), resource_info.clone());
        }

        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.total_resources += 1;
            stats.active_resources += 1;
            
            let type_name = self.resource_type_name(&resource_type);
            *stats.resource_types.entry(type_name).or_insert(0) += 1;
            
            let plugin_usage = stats.plugin_usage.entry(plugin_id.clone()).or_insert(
                PluginResourceUsage {
                    plugin_id: plugin_id.clone(),
                    active_resources: 0,
                    memory_usage: 0,
                    resource_creation_rate: 0.0,
                    error_rate: 0.0,
                    last_active: now,
                }
            );
            plugin_usage.active_resources += 1;
            plugin_usage.memory_usage += resource_info.memory_usage;
            plugin_usage.last_active = now;
        }

        // Execute callbacks
        self.execute_callbacks(|cb| cb.on_resource_created(&resource_info)).await?;

        log::debug!("Created resource {} for plugin {}", resource_id.0, plugin_id);
        Ok(resource_id)
    }

    /// Access a resource and update tracking
    pub async fn access_resource(&self, resource_id: &ResourceId) -> Result<Option<ResourceInfo>> {
        let mut resources = self.resources.write().await;
        
        if let Some(resource) = resources.get_mut(resource_id) {
            if resource.state != ResourceState::Active {
                return Err(anyhow::anyhow!("Resource {} is not in active state", resource_id.0));
            }
            
            resource.last_accessed = SystemTime::now();
            resource.access_count += 1;
            
            let resource_clone = resource.clone();
            drop(resources);
            
            // Execute callbacks
            self.execute_callbacks(|cb| cb.on_resource_accessed(&resource_clone)).await?;
            
            Ok(Some(resource_clone))
        } else {
            Ok(None)
        }
    }

    /// Destroy a resource and clean up tracking
    pub async fn destroy_resource(&self, resource_id: &ResourceId) -> Result<()> {
        let resource_info = {
            let mut resources = self.resources.write().await;
            if let Some(mut resource) = resources.get_mut(resource_id) {
                resource.state = ResourceState::Destroying;
                resource.clone()
            } else {
                return Err(anyhow::anyhow!("Resource {} not found", resource_id.0));
            }
        };

        // Update memory tracking
        {
            let mut tracker = self.memory_tracker.lock().await;
            tracker.track_deallocation(
                resource_info.plugin_id.clone(),
                resource_id.clone(),
                resource_info.memory_usage,
            ).await?;
        }

        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.active_resources -= 1;
            stats.destroyed_resources += 1;
            
            if let Some(plugin_usage) = stats.plugin_usage.get_mut(&resource_info.plugin_id) {
                plugin_usage.active_resources -= 1;
                plugin_usage.memory_usage -= resource_info.memory_usage;
            }
        }

        // Execute callbacks
        self.execute_callbacks(|cb| cb.on_resource_destroyed(&resource_info)).await?;

        // Remove from tracking
        {
            let mut resources = self.resources.write().await;
            if let Some(mut resource) = resources.get_mut(resource_id) {
                resource.state = ResourceState::Destroyed;
            }
        }

        log::debug!("Destroyed resource {} from plugin {}", resource_id.0, resource_info.plugin_id);
        Ok(())
    }

    /// Get memory statistics
    pub async fn get_memory_statistics(&self) -> Result<MemoryUsage> {
        let tracker = self.memory_tracker.lock().await;
        Ok(tracker.global_usage.clone())
    }

    /// Get plugin memory information
    pub async fn get_plugin_memory_info(&self, plugin_id: &str) -> Result<Option<PluginMemoryInfo>> {
        let tracker = self.memory_tracker.lock().await;
        Ok(tracker.allocations.get(plugin_id).cloned())
    }

    /// Perform garbage collection for a plugin
    pub async fn garbage_collect_plugin(&self, plugin_id: &str) -> Result<GarbageCollectionResult> {
        let now = SystemTime::now();
        let mut collected_resources = 0;
        let mut freed_memory = 0;

        // Find resources to collect
        let resources_to_collect = {
            let resources = self.resources.read().await;
            resources.iter()
                .filter(|(_, resource)| {
                    resource.plugin_id == plugin_id &&
                    resource.state == ResourceState::Active &&
                    now.duration_since(resource.last_accessed)
                        .unwrap_or(Duration::ZERO) > Duration::from_secs(300) // 5 minutes
                })
                .map(|(id, resource)| (id.clone(), resource.memory_usage))
                .collect::<Vec<_>>()
        };

        // Collect identified resources
        for (resource_id, memory_usage) in resources_to_collect {
            if let Err(e) = self.destroy_resource(&resource_id).await {
                log::warn!("Failed to collect resource {}: {}", resource_id.0, e);
            } else {
                collected_resources += 1;
                freed_memory += memory_usage;
            }
        }

        // Update memory tracker
        {
            let mut tracker = self.memory_tracker.lock().await;
            if let Some(plugin_memory) = tracker.allocations.get_mut(plugin_id) {
                plugin_memory.last_gc = Some(now);
            }
            
            tracker.allocation_history.push(AllocationEvent {
                plugin_id: plugin_id.to_string(),
                resource_id: ResourceId(Uuid::new_v4()),
                size: freed_memory,
                operation: AllocationOperation::GarbageCollect,
                timestamp: now,
                stack_trace: None,
            });
        }

        log::info!("Garbage collected {} resources, freed {} bytes for plugin {}", 
                  collected_resources, freed_memory, plugin_id);

        Ok(GarbageCollectionResult {
            collected_resources,
            freed_memory,
            duration: now.elapsed().unwrap_or(Duration::ZERO),
        })
    }

    /// Add resource lifecycle callback
    pub async fn add_callback(&self, callback: Box<dyn ResourceCallback + Send + Sync>) -> Result<()> {
        let mut callbacks = self.callbacks.write().await;
        callbacks.push(callback);
        Ok(())
    }

    /// Get comprehensive resource statistics
    pub async fn get_statistics(&self) -> Result<ResourceStatistics> {
        let stats = self.statistics.read().await;
        let tracker = self.memory_tracker.lock().await;
        
        let mut result = stats.clone();
        result.memory_usage = tracker.global_usage.clone();
        
        Ok(result)
    }

    /// Check if plugin exceeds resource limits
    async fn check_resource_limits(&self, plugin_id: &str) -> Result<()> {
        let stats = self.statistics.read().await;
        
        if let Some(plugin_usage) = stats.plugin_usage.get(plugin_id) {
            if plugin_usage.active_resources >= self.limits.max_resources_per_plugin as u64 {
                return Err(anyhow::anyhow!("Plugin {} exceeds resource limit", plugin_id));
            }
            
            if plugin_usage.memory_usage >= self.limits.max_memory_per_plugin {
                return Err(anyhow::anyhow!("Plugin {} exceeds memory limit", plugin_id));
            }
        }
        
        Ok(())
    }

    /// Calculate memory usage for a resource type
    fn calculate_resource_memory_usage(&self, resource_type: &ResourceType) -> u64 {
        match resource_type {
            ResourceType::File { .. } => 1024, // 1KB for file handles
            ResourceType::Socket { .. } => 2048, // 2KB for sockets
            ResourceType::Memory { size } => *size,
            ResourceType::Timer { .. } => 512, // 512B for timers
            ResourceType::Custom { .. } => 1024, // 1KB default
            ResourceType::Wasi { .. } => 512, // 512B for WASI handles
        }
    }

    /// Get human-readable resource type name
    fn resource_type_name(&self, resource_type: &ResourceType) -> String {
        match resource_type {
            ResourceType::File { .. } => "file".to_string(),
            ResourceType::Socket { .. } => "socket".to_string(),
            ResourceType::Memory { .. } => "memory".to_string(),
            ResourceType::Timer { .. } => "timer".to_string(),
            ResourceType::Custom { type_name } => type_name.clone(),
            ResourceType::Wasi { .. } => "wasi".to_string(),
        }
    }

    /// Execute all registered callbacks
    async fn execute_callbacks<F>(&self, operation: F) -> Result<()>
    where
        F: Fn(&dyn ResourceCallback) -> Result<()>,
    {
        let callbacks = self.callbacks.read().await;
        for callback in callbacks.iter() {
            if let Err(e) = operation(callback.as_ref()) {
                log::warn!("Resource callback failed: {}", e);
            }
        }
        Ok(())
    }
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            global_usage: MemoryUsage {
                total_allocated: 0,
                total_freed: 0,
                current_usage: 0,
                peak_usage: 0,
                allocations_per_second: 0.0,
                gc_frequency: 0.0,
            },
            allocation_history: Vec::new(),
            plugin_limits: HashMap::new(),
        }
    }

    pub async fn track_allocation(
        &mut self,
        plugin_id: String,
        resource_id: ResourceId,
        size: u64,
    ) -> Result<()> {
        // Update plugin-specific tracking
        let plugin_memory = self.allocations.entry(plugin_id.clone()).or_insert(
            PluginMemoryInfo {
                plugin_id: plugin_id.clone(),
                total_allocated: 0,
                current_usage: 0,
                peak_usage: 0,
                allocation_count: 0,
                deallocation_count: 0,
                fragmentation_ratio: 0.0,
                last_gc: None,
            }
        );

        plugin_memory.total_allocated += size;
        plugin_memory.current_usage += size;
        plugin_memory.allocation_count += 1;
        
        if plugin_memory.current_usage > plugin_memory.peak_usage {
            plugin_memory.peak_usage = plugin_memory.current_usage;
        }

        // Update global tracking
        self.global_usage.total_allocated += size;
        self.global_usage.current_usage += size;
        
        if self.global_usage.current_usage > self.global_usage.peak_usage {
            self.global_usage.peak_usage = self.global_usage.current_usage;
        }

        // Record allocation event
        self.allocation_history.push(AllocationEvent {
            plugin_id,
            resource_id,
            size,
            operation: AllocationOperation::Allocate,
            timestamp: SystemTime::now(),
            stack_trace: None,
        });

        Ok(())
    }

    pub async fn track_deallocation(
        &mut self,
        plugin_id: String,
        resource_id: ResourceId,
        size: u64,
    ) -> Result<()> {
        // Update plugin-specific tracking
        if let Some(plugin_memory) = self.allocations.get_mut(&plugin_id) {
            plugin_memory.current_usage = plugin_memory.current_usage.saturating_sub(size);
            plugin_memory.deallocation_count += 1;
        }

        // Update global tracking
        self.global_usage.total_freed += size;
        self.global_usage.current_usage = self.global_usage.current_usage.saturating_sub(size);

        // Record deallocation event
        self.allocation_history.push(AllocationEvent {
            plugin_id,
            resource_id,
            size,
            operation: AllocationOperation::Deallocate,
            timestamp: SystemTime::now(),
            stack_trace: None,
        });

        Ok(())
    }
}

impl ResourceStatistics {
    pub fn new() -> Self {
        Self {
            total_resources: 0,
            active_resources: 0,
            destroyed_resources: 0,
            failed_resources: 0,
            total_plugins: 0,
            memory_usage: MemoryUsage {
                total_allocated: 0,
                total_freed: 0,
                current_usage: 0,
                peak_usage: 0,
                allocations_per_second: 0.0,
                gc_frequency: 0.0,
            },
            resource_types: HashMap::new(),
            plugin_usage: HashMap::new(),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_resources_per_plugin: 1000,
            max_memory_per_plugin: 100 * 1024 * 1024, // 100MB
            max_file_handles: 100,
            max_network_connections: 50,
            max_timers: 20,
            cleanup_timeout: Duration::from_secs(30),
            memory_pressure_threshold: 0.8,
        }
    }
}

/// Garbage collection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageCollectionResult {
    pub collected_resources: u64,
    pub freed_memory: u64,
    pub duration: Duration,
}

impl ResourceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_resource_creation() {
        let limits = ResourceLimits::default();
        let table = AdvancedResourceTable::new(limits).unwrap();
        
        let resource_id = table.create_resource(
            "test_plugin".to_string(),
            ResourceType::Memory { size: 1024 },
            HashMap::new(),
        ).await.unwrap();
        
        let resource = table.access_resource(&resource_id).await.unwrap();
        assert!(resource.is_some());
        assert_eq!(resource.unwrap().memory_usage, 1024);
    }

    #[tokio::test]
    async fn test_memory_tracking() {
        let limits = ResourceLimits::default();
        let table = AdvancedResourceTable::new(limits).unwrap();
        
        let _resource_id = table.create_resource(
            "test_plugin".to_string(),
            ResourceType::Memory { size: 2048 },
            HashMap::new(),
        ).await.unwrap();
        
        let memory_stats = table.get_memory_statistics().await.unwrap();
        assert_eq!(memory_stats.current_usage, 2048);
        assert_eq!(memory_stats.total_allocated, 2048);
    }

    #[tokio::test]
    async fn test_resource_limits() {
        let mut limits = ResourceLimits::default();
        limits.max_resources_per_plugin = 1;
        
        let table = AdvancedResourceTable::new(limits).unwrap();
        
        // First resource should succeed
        let _resource1 = table.create_resource(
            "test_plugin".to_string(),
            ResourceType::Memory { size: 1024 },
            HashMap::new(),
        ).await.unwrap();
        
        // Second resource should fail due to limit
        let result = table.create_resource(
            "test_plugin".to_string(),
            ResourceType::Memory { size: 1024 },
            HashMap::new(),
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_garbage_collection() {
        let limits = ResourceLimits::default();
        let table = AdvancedResourceTable::new(limits).unwrap();
        
        let resource_id = table.create_resource(
            "test_plugin".to_string(),
            ResourceType::Memory { size: 1024 },
            HashMap::new(),
        ).await.unwrap();
        
        // Simulate old resource (modify last_accessed manually for test)
        {
            let mut resources = table.resources.write().await;
            if let Some(resource) = resources.get_mut(&resource_id) {
                resource.last_accessed = SystemTime::now() - Duration::from_secs(400);
            }
        }
        
        let gc_result = table.garbage_collect_plugin("test_plugin").await.unwrap();
        assert_eq!(gc_result.collected_resources, 1);
        assert_eq!(gc_result.freed_memory, 1024);
    }
}
