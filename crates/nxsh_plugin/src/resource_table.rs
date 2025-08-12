//! Pure Rust Resource Table for Plugin Runtime
//!
//! This module provides comprehensive resource management for WebAssembly plugins
//! using Pure Rust components without wasmtime dependencies.

use anyhow::{Result, anyhow};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
    fmt,
    any::{Any, TypeId},
};
use tokio::sync::{RwLock, Mutex};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use log::{info, warn, error, debug};

/// Pure Rust resource table with memory tracking and lifecycle management
pub struct ResourceTable {
    /// Resource storage by ID
    resources: Arc<RwLock<HashMap<ResourceId, Box<dyn ResourceEntry>>>>,
    /// Resource tracking by type
    type_tracker: Arc<RwLock<HashMap<TypeId, Vec<ResourceId>>>>,
    /// Memory allocations tracking
    memory_tracker: Arc<Mutex<MemoryTracker>>,
    /// Resource limits configuration
    limits: ResourceLimits,
    /// Resource lifecycle callbacks
    callbacks: Arc<RwLock<Vec<Box<dyn ResourceCallback + Send + Sync>>>>,
    /// Global statistics
    statistics: Arc<RwLock<ResourceStatistics>>,
    /// Resource cleanup queue
    cleanup_queue: Arc<Mutex<Vec<ResourceId>>>,
}

/// Unique resource identifier
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceId(Uuid);

impl Default for ResourceId {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceId {
    /// Create a new resource ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID
    pub fn uuid(&self) -> &Uuid {
        &self.0
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Resource entry trait for type-erased storage
pub trait ResourceEntry: Send + Sync + std::fmt::Debug {
    /// Get resource info
    fn info(&self) -> &ResourceInfo;
    
    /// Get mutable resource info
    fn info_mut(&mut self) -> &mut ResourceInfo;
    
    /// Get the underlying resource as Any
    fn as_any(&self) -> &dyn Any;
    
    /// Get the underlying resource as mutable Any
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    /// Check if resource can be dropped
    fn can_drop(&self) -> bool;
    
    /// Prepare resource for cleanup
    fn prepare_cleanup(&mut self) -> Result<()>;
}

/// Concrete resource entry implementation with proper Arc management
#[derive(Debug)]
pub struct ConcreteResourceEntry<T: Send + Sync + 'static> {
    pub info: ResourceInfo,
    pub resource: Arc<T>,
}

impl<T: Send + Sync + 'static + std::fmt::Debug> ResourceEntry for ConcreteResourceEntry<T> {
    fn info(&self) -> &ResourceInfo {
        &self.info
    }
    
    fn info_mut(&mut self) -> &mut ResourceInfo {
        &mut self.info
    }
    
    fn as_any(&self) -> &dyn Any {
        self.resource.as_ref()
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        // Note: Arc doesn't allow mutable access to inner data directly
        // This is a limitation when using Arc for shared ownership
        Arc::get_mut(&mut self.resource).unwrap_or_else(|| {
            panic!("Cannot get mutable reference to Arc with multiple references")
        })
    }
    
    fn can_drop(&self) -> bool {
        Arc::strong_count(&self.resource) <= 1
    }
    
    fn prepare_cleanup(&mut self) -> Result<()> {
        // Prepare the resource for cleanup - custom cleanup logic can be added here
        info!("Preparing resource {} for cleanup", self.info.id);
        self.info.state = ResourceState::Cleaned;
        Ok(())
    }
}

impl<T: Send + Sync + 'static + std::fmt::Debug> ConcreteResourceEntry<T> {
    /// Get a cloned Arc to the resource - this is the safe way to share resources
    pub fn get_arc(&self) -> Arc<T> {
        Arc::clone(&self.resource)
    }
}

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

/// Resource types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Memory,
    File,
    Socket,
    Thread,
    Process,
    Timer,
    Handle,
    Buffer,
    Stream,
    Custom(String),
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::Memory => write!(f, "memory"),
            ResourceType::File => write!(f, "file"),
            ResourceType::Socket => write!(f, "socket"),
            ResourceType::Thread => write!(f, "thread"),
            ResourceType::Process => write!(f, "process"),
            ResourceType::Timer => write!(f, "timer"),
            ResourceType::Handle => write!(f, "handle"),
            ResourceType::Buffer => write!(f, "buffer"),
            ResourceType::Stream => write!(f, "stream"),
            ResourceType::Custom(name) => write!(f, "custom:{name}"),
        }
    }
}

/// Resource states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceState {
    Creating,
    Active,
    Idle,
    MarkedForCleanup,
    Cleaning,
    Cleaned,
    Error(String),
}

/// Memory tracking for resources
#[derive(Debug, Default, Clone)]
pub struct MemoryTracker {
    /// Total allocated memory
    pub total_allocated: u64,
    /// Total deallocated memory
    pub total_deallocated: u64,
    /// Current memory usage
    pub current_usage: u64,
    /// Memory by resource type
    pub by_type: HashMap<ResourceType, u64>,
    /// Memory by plugin
    pub by_plugin: HashMap<String, u64>,
    /// Peak memory usage
    pub peak_usage: u64,
    /// Allocation history
    pub allocations: Vec<AllocationRecord>,
}

/// Memory allocation record
#[derive(Debug, Clone)]
pub struct AllocationRecord {
    pub resource_id: ResourceId,
    pub size: u64,
    pub allocated_at: SystemTime,
    pub freed_at: Option<SystemTime>,
    pub plugin_id: String,
}

/// Resource limits configuration
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum total memory usage
    pub max_memory: u64,
    /// Maximum memory per plugin
    pub max_memory_per_plugin: u64,
    /// Maximum number of resources
    pub max_resources: u32,
    /// Maximum number of resources per type
    pub max_resources_per_type: u32,
    /// Resource lifetime limits
    pub max_lifetime: Duration,
    /// Maximum idle time before cleanup
    pub max_idle_time: Duration,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 1024 * 1024 * 1024, // 1GB
            max_memory_per_plugin: 256 * 1024 * 1024, // 256MB
            max_resources: 10000,
            max_resources_per_type: 1000,
            max_lifetime: Duration::from_secs(3600), // 1 hour
            max_idle_time: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Resource lifecycle callback trait
pub trait ResourceCallback: Send + Sync {
    /// Called when a resource is created
    fn on_created(&self, resource_id: &ResourceId, resource_type: &ResourceType) -> Result<()>;
    
    /// Called when a resource is accessed
    fn on_accessed(&self, resource_id: &ResourceId) -> Result<()>;
    
    /// Called before a resource is cleaned up
    fn on_cleanup(&self, resource_id: &ResourceId) -> Result<()>;
    
    /// Called when a resource encounters an error
    fn on_error(&self, resource_id: &ResourceId, error: &str) -> Result<()>;
}

/// Resource statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ResourceStatistics {
    /// Total resources created
    pub total_created: u64,
    /// Total resources cleaned up
    pub total_cleaned: u64,
    /// Current active resources
    pub active_count: u32,
    /// Resources by type
    pub by_type: HashMap<ResourceType, u32>,
    /// Resources by plugin
    pub by_plugin: HashMap<String, u32>,
    /// Memory statistics
    pub memory_stats: MemoryStatistics,
    /// Error statistics
    pub error_count: u64,
    /// Performance metrics
    pub perf_metrics: PerformanceMetrics,
}

/// Memory statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemoryStatistics {
    pub current_usage: u64,
    pub peak_usage: u64,
    pub total_allocated: u64,
    pub total_freed: u64,
    pub allocation_count: u64,
    pub free_count: u64,
}

/// Performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_allocation_time: Duration,
    pub avg_cleanup_time: Duration,
    pub total_operations: u64,
    pub operations_per_second: f64,
}

impl ResourceTable {
    /// Create a new resource table
    pub fn new() -> Self {
        Self::with_limits(ResourceLimits::default())
    }

    /// Create a new resource table with custom limits
    pub fn with_limits(limits: ResourceLimits) -> Self {
        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            type_tracker: Arc::new(RwLock::new(HashMap::new())),
            memory_tracker: Arc::new(Mutex::new(MemoryTracker::default())),
            limits,
            callbacks: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(ResourceStatistics::default())),
            cleanup_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a resource to the table
    pub async fn add_resource<T: Send + Sync + 'static + std::fmt::Debug>(
        &self,
        resource: T,
        resource_type: ResourceType,
        plugin_id: String,
        memory_usage: u64,
    ) -> Result<ResourceId> {
        let resource_id = ResourceId::new();
        
        // Check limits
        self.check_limits(&resource_type, &plugin_id, memory_usage).await?;
        
        // Create resource info
        let resource_info = ResourceInfo {
            id: resource_id.clone(),
            resource_type: resource_type.clone(),
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            access_count: 0,
            memory_usage,
            metadata: HashMap::new(),
            plugin_id: plugin_id.clone(),
            state: ResourceState::Creating,
        };
        
        // Create resource entry with Arc
        let resource_arc = Arc::new(resource);
        let entry = ConcreteResourceEntry {
            info: resource_info,
            resource: Arc::clone(&resource_arc),
        };
        
        // Add to storage
        {
            let mut resources = self.resources.write().await;
            resources.insert(resource_id.clone(), Box::new(entry));
        }
        
        // Update type tracker
        {
            let mut type_tracker = self.type_tracker.write().await;
            type_tracker.entry(TypeId::of::<T>())
                .or_insert_with(Vec::new)
                .push(resource_id.clone());
        }
        
        // Update memory tracking
        {
            let mut memory_tracker = self.memory_tracker.lock().await;
            memory_tracker.total_allocated += memory_usage;
            *memory_tracker.by_type.entry(resource_type.clone()).or_insert(0) += memory_usage;
            *memory_tracker.by_plugin.entry(plugin_id.clone()).or_insert(0) += memory_usage;
            
            if memory_tracker.total_allocated > memory_tracker.peak_usage {
                memory_tracker.peak_usage = memory_tracker.total_allocated;
            }
            
            memory_tracker.allocations.push(AllocationRecord {
                resource_id: resource_id.clone(),
                size: memory_usage,
                allocated_at: SystemTime::now(),
                freed_at: None,
                plugin_id: plugin_id.clone(),
            });
        }
        
        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.total_created += 1;
            stats.active_count += 1;
            *stats.by_type.entry(resource_type.clone()).or_insert(0) += 1;
            *stats.by_plugin.entry(plugin_id.clone()).or_insert(0) += 1;
            stats.memory_stats.current_usage += memory_usage;
            stats.memory_stats.total_allocated += memory_usage;
            stats.memory_stats.allocation_count += 1;
        }
        
        // Mark as active
        {
            let mut resources = self.resources.write().await;
            if let Some(entry) = resources.get_mut(&resource_id) {
                entry.info_mut().state = ResourceState::Active;
            }
        }
        
        // Notify callbacks
        self.notify_created(&resource_id, &resource_type).await?;
        
        info!("Resource {resource_id} ({resource_type}) created for plugin {plugin_id}");
        Ok(resource_id)
    }

    /// Get a resource from the table with proper Arc management
    pub async fn get_resource<T: Send + Sync + std::fmt::Debug + 'static>(&self, resource_id: &ResourceId) -> Result<Option<Arc<T>>> {
        let mut resources = self.resources.write().await;
        
        if let Some(entry) = resources.get_mut(resource_id) {
            // Update access information
            entry.info_mut().last_accessed = SystemTime::now();
            entry.info_mut().access_count += 1;
            
            // Notify callbacks
            self.notify_accessed(resource_id).await?;
            
            // Try to downcast to the concrete type
            let entry_any = entry.as_any();
            if let Some(concrete_entry) = entry_any.downcast_ref::<ConcreteResourceEntry<T>>() {
                debug!("Resource {resource_id} accessed successfully");
                return Ok(Some(concrete_entry.get_arc()));
            }
            
            // If direct downcast fails, try generic approach
            warn!("Failed to downcast resource {resource_id} to requested type");
        }
        
        Ok(None)
    }

    /// Remove a resource from the table
    pub async fn remove_resource(&self, resource_id: &ResourceId) -> Result<bool> {
        let removed_entry = {
            let mut resources = self.resources.write().await;
            resources.remove(resource_id)
        };
        
        if let Some(mut entry) = removed_entry {
            let resource_info = entry.info().clone();
            
            // Prepare for cleanup
            entry.prepare_cleanup()?;
            
            // Update type tracker
            {
                let mut type_tracker = self.type_tracker.write().await;
                for (_, ids) in type_tracker.iter_mut() {
                    ids.retain(|id| id != resource_id);
                }
            }
            
            // Update memory tracking
            {
                let mut memory_tracker = self.memory_tracker.lock().await;
                memory_tracker.total_allocated = memory_tracker.total_allocated.saturating_sub(resource_info.memory_usage);
                
                if let Some(type_usage) = memory_tracker.by_type.get_mut(&resource_info.resource_type) {
                    *type_usage = type_usage.saturating_sub(resource_info.memory_usage);
                }
                
                if let Some(plugin_usage) = memory_tracker.by_plugin.get_mut(&resource_info.plugin_id) {
                    *plugin_usage = plugin_usage.saturating_sub(resource_info.memory_usage);
                }
                
                // Update allocation record
                if let Some(record) = memory_tracker.allocations.iter_mut()
                    .find(|r| r.resource_id == *resource_id && r.freed_at.is_none()) {
                    record.freed_at = Some(SystemTime::now());
                }
            }
            
            // Update statistics
            {
                let mut stats = self.statistics.write().await;
                stats.total_cleaned += 1;
                stats.active_count = stats.active_count.saturating_sub(1);
                
                if let Some(type_count) = stats.by_type.get_mut(&resource_info.resource_type) {
                    *type_count = type_count.saturating_sub(1);
                }
                
                if let Some(plugin_count) = stats.by_plugin.get_mut(&resource_info.plugin_id) {
                    *plugin_count = plugin_count.saturating_sub(1);
                }
                
                stats.memory_stats.current_usage = stats.memory_stats.current_usage.saturating_sub(resource_info.memory_usage);
                stats.memory_stats.total_freed += resource_info.memory_usage;
                stats.memory_stats.free_count += 1;
            }
            
            // Notify callbacks
            self.notify_cleanup(resource_id).await?;
            
            info!("Resource {} ({}) removed for plugin {}", 
                  resource_id, resource_info.resource_type, resource_info.plugin_id);
            return Ok(true);
        }
        
        Ok(false)
    }

    /// Get resources by type
    pub async fn get_resources_by_type<T: 'static>(&self) -> Vec<ResourceId> {
        let type_tracker = self.type_tracker.read().await;
        type_tracker.get(&TypeId::of::<T>())
            .cloned()
            .unwrap_or_default()
    }

    /// Get resources by plugin
    pub async fn get_resources_by_plugin(&self, plugin_id: &str) -> Vec<ResourceId> {
        let resources = self.resources.read().await;
        resources.iter()
            .filter(|(_, entry)| entry.info().plugin_id == plugin_id)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Clean up idle resources
    pub async fn cleanup_idle_resources(&self) -> Result<usize> {
        let now = SystemTime::now();
        let mut to_cleanup = Vec::new();
        
        {
            let resources = self.resources.read().await;
            for (id, entry) in resources.iter() {
                let info = entry.info();
                if let Ok(idle_duration) = now.duration_since(info.last_accessed) {
                    if idle_duration > self.limits.max_idle_time && 
                       matches!(info.state, ResourceState::Active | ResourceState::Idle) {
                        to_cleanup.push(id.clone());
                    }
                }
            }
        }
        
        let cleanup_count = to_cleanup.len();
        for resource_id in to_cleanup {
            if let Err(e) = self.remove_resource(&resource_id).await {
                error!("Failed to cleanup idle resource {resource_id}: {e}");
            }
        }
        
        info!("Cleaned up {cleanup_count} idle resources");
        Ok(cleanup_count)
    }

    /// Check resource limits
    async fn check_limits(
        &self,
        resource_type: &ResourceType,
        plugin_id: &str,
        memory_usage: u64,
    ) -> Result<()> {
        let memory_tracker = self.memory_tracker.lock().await;
        let stats = self.statistics.read().await;
        
        // Check total memory limit
        if memory_tracker.total_allocated + memory_usage > self.limits.max_memory {
            return Err(anyhow!("Total memory limit exceeded"));
        }
        
        // Check per-plugin memory limit
        let plugin_usage = memory_tracker.by_plugin.get(plugin_id).copied().unwrap_or(0);
        if plugin_usage + memory_usage > self.limits.max_memory_per_plugin {
            return Err(anyhow!("Per-plugin memory limit exceeded"));
        }
        
        // Check total resource count
        if stats.active_count >= self.limits.max_resources {
            return Err(anyhow!("Total resource count limit exceeded"));
        }
        
        // Check per-type resource count
        let type_count = stats.by_type.get(resource_type).copied().unwrap_or(0);
        if type_count >= self.limits.max_resources_per_type {
            return Err(anyhow!("Per-type resource count limit exceeded"));
        }
        
        Ok(())
    }

    /// Add resource callback
    pub async fn add_callback(&self, callback: Box<dyn ResourceCallback + Send + Sync>) {
        let mut callbacks = self.callbacks.write().await;
        callbacks.push(callback);
    }

    /// Notify callbacks about resource creation
    async fn notify_created(&self, resource_id: &ResourceId, resource_type: &ResourceType) -> Result<()> {
        let callbacks = self.callbacks.read().await;
        for callback in callbacks.iter() {
            if let Err(e) = callback.on_created(resource_id, resource_type) {
                warn!("Resource callback failed on creation: {e}");
            }
        }
        Ok(())
    }

    /// Notify callbacks about resource access
    async fn notify_accessed(&self, resource_id: &ResourceId) -> Result<()> {
        let callbacks = self.callbacks.read().await;
        for callback in callbacks.iter() {
            if let Err(e) = callback.on_accessed(resource_id) {
                warn!("Resource callback failed on access: {e}");
            }
        }
        Ok(())
    }

    /// Notify callbacks about resource cleanup
    async fn notify_cleanup(&self, resource_id: &ResourceId) -> Result<()> {
        let callbacks = self.callbacks.read().await;
        for callback in callbacks.iter() {
            if let Err(e) = callback.on_cleanup(resource_id) {
                warn!("Resource callback failed on cleanup: {e}");
            }
        }
        Ok(())
    }

    /// Get current statistics
    pub async fn get_statistics(&self) -> ResourceStatistics {
        self.statistics.read().await.clone()
    }

    /// Get memory usage information
    pub async fn get_memory_usage(&self) -> MemoryTracker {
        self.memory_tracker.lock().await.clone()
    }

    /// Force cleanup all resources for a plugin
    pub async fn cleanup_plugin_resources(&self, plugin_id: &str) -> Result<usize> {
        let resource_ids = self.get_resources_by_plugin(plugin_id).await;
        let cleanup_count = resource_ids.len();
        
        for resource_id in resource_ids {
            if let Err(e) = self.remove_resource(&resource_id).await {
                error!("Failed to cleanup plugin resource {resource_id}: {e}");
            }
        }
        
        info!("Cleaned up {cleanup_count} resources for plugin {plugin_id}");
        Ok(cleanup_count)
    }
}

impl Default for ResourceTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    #[derive(Debug)]
    struct TestResource {
        data: String,
    }
    
    #[tokio::test]
    async fn test_resource_table_creation() {
        let table = ResourceTable::new();
        let stats = table.get_statistics().await;
        assert_eq!(stats.active_count, 0);
    }

    #[tokio::test]
    async fn test_add_and_remove_resource() {
        let table = ResourceTable::new();
        
        let resource = TestResource {
            data: "test".to_string(),
        };
        
        let resource_id = table.add_resource(
            resource,
            ResourceType::Custom("test".to_string()),
            "test_plugin".to_string(),
            100,
        ).await.unwrap();
        
        let stats = table.get_statistics().await;
        assert_eq!(stats.active_count, 1);
        assert_eq!(stats.memory_stats.current_usage, 100);
        
        let removed = table.remove_resource(&resource_id).await.unwrap();
        assert!(removed);
        
        let stats = table.get_statistics().await;
        assert_eq!(stats.active_count, 0);
        assert_eq!(stats.memory_stats.current_usage, 0);
    }

    #[tokio::test]
    async fn test_memory_limits() {
        let limits = ResourceLimits {
            max_memory: 200,
            ..Default::default()
        };
        let table = ResourceTable::with_limits(limits);
        
        let resource1 = TestResource { data: "test1".to_string() };
        let _id1 = table.add_resource(
            resource1,
            ResourceType::Memory,
            "test_plugin".to_string(),
            100,
        ).await.unwrap();
        
        let resource2 = TestResource { data: "test2".to_string() };
        let result = table.add_resource(
            resource2,
            ResourceType::Memory,
            "test_plugin".to_string(),
            150, // This should exceed the limit
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_plugin_resource_cleanup() {
        let table = ResourceTable::new();
        
        // Add resources for different plugins
        let resource1 = TestResource { data: "test1".to_string() };
        let _id1 = table.add_resource(
            resource1,
            ResourceType::Memory,
            "plugin1".to_string(),
            100,
        ).await.unwrap();
        
        let resource2 = TestResource { data: "test2".to_string() };
        let _id2 = table.add_resource(
            resource2,
            ResourceType::Memory,
            "plugin2".to_string(),
            100,
        ).await.unwrap();
        
        let stats = table.get_statistics().await;
        assert_eq!(stats.active_count, 2);
        
        // Cleanup plugin1 resources
        let cleaned = table.cleanup_plugin_resources("plugin1").await.unwrap();
        assert_eq!(cleaned, 1);
        
        let stats = table.get_statistics().await;
        assert_eq!(stats.active_count, 1);
    }
}
