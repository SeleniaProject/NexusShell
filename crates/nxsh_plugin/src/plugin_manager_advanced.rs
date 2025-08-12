#[cfg(feature = "plugin-management")]
use anyhow::{Result, Context};
#[cfg(feature = "plugin-management")]
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};
#[cfg(feature = "plugin-management")]
use tokio::sync::RwLock;
#[cfg(feature = "plugin-management")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "plugin-management")]
use log::info;

/// Plugin store for managing plugin storage and distribution
#[cfg(feature = "plugin-management")]
#[derive(Debug)]
pub struct PluginStore {
    store_directory: PathBuf,
    index: Arc<RwLock<PluginIndex>>,
    cache: Arc<RwLock<HashMap<String, CachedPlugin>>>,
}

#[cfg(feature = "plugin-management")]
impl PluginStore {
    /// Create a new plugin store
    pub async fn new(store_directory: &Path) -> Result<Self> {
        tokio::fs::create_dir_all(store_directory).await
            .context("Failed to create store directory")?;
        
        let index_path = store_directory.join("index.json");
        let index = if index_path.exists() {
            let data = tokio::fs::read_to_string(&index_path).await?;
            serde_json::from_str(&data)?
        } else {
            PluginIndex::new()
        };

        Ok(Self {
            store_directory: store_directory.to_path_buf(),
            index: Arc::new(RwLock::new(index)),
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Store a plugin in the store
    pub async fn store_plugin(&self, source_path: &Path, plugin_id: &str) -> Result<PathBuf> {
        let plugin_dir = self.store_directory.join(plugin_id);
        tokio::fs::create_dir_all(&plugin_dir).await?;
        
        let stored_path = plugin_dir.join(format!("{plugin_id}.wasm"));
        tokio::fs::copy(source_path, &stored_path).await?;
        
        // Update index and cache
        let metadata = self.extract_plugin_metadata(&stored_path).await?;
        {
            let mut index = self.index.write().await;
            index.plugins.insert(plugin_id.to_string(), PluginIndexEntry {
                id: plugin_id.to_string(),
                metadata,
                stored_path: stored_path.clone(),
                installed_at: SystemTime::now(),
            });
        }
        {
            let mut cache = self.cache.write().await;
            cache.insert(plugin_id.to_string(), CachedPlugin { metadata: self.index.read().await.plugins.get(plugin_id).unwrap().metadata.clone(), cached_at: SystemTime::now() });
        }
        
        self.save_index().await?;
        
        Ok(stored_path)
    }

    /// Remove a plugin from the store (cleans index and cache)
    pub async fn remove_plugin(&self, plugin_id: &str) -> Result<()> {
        let plugin_dir = self.store_directory.join(plugin_id);
        if plugin_dir.exists() {
            tokio::fs::remove_dir_all(&plugin_dir).await?;
        }

        // Update index and cache
        {
            let mut index = self.index.write().await;
            index.plugins.remove(plugin_id);
        }
        {
            let mut cache = self.cache.write().await;
            cache.remove(plugin_id);
        }
        
        self.save_index().await?;
        
        Ok(())
    }

    /// Search for plugins
    pub async fn search_plugins(&self, query: &str) -> Result<Vec<PluginSearchResult>> {
        let index = self.index.read().await;
        let results = index.plugins.values()
            .filter(|entry| {
                entry.metadata.name.contains(query) ||
                entry.metadata.description.contains(query) ||
                entry.metadata.capabilities.iter().any(|cap| cap.contains(query))
            })
            .map(|entry| PluginSearchResult {
                id: entry.id.clone(),
                metadata: entry.metadata.clone(),
                relevance_score: self.calculate_relevance_score(&entry.metadata, query),
            })
            .collect();

        Ok(results)
    }

    /// Get plugin path (uses cache for quick existence check)
    pub async fn get_plugin_path(&self, plugin_id: &str) -> Result<PathBuf> {
        // Fast path via index
        let index = self.index.read().await;
        let entry = index.plugins.get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found in store", plugin_id))?;
        Ok(entry.stored_path.clone())
    }

    /// Backup a plugin
    pub async fn backup_plugin(&self, plugin_id: &str) -> Result<PathBuf> {
        let plugin_dir = self.store_directory.join(plugin_id);
        let backup_dir = self.store_directory.join("backups").join(plugin_id);
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();
        let backup_path = backup_dir.join(format!("backup_{timestamp}.wasm"));
        
        tokio::fs::create_dir_all(&backup_dir).await?;
        
        let source_path = plugin_dir.join(format!("{plugin_id}.wasm"));
        tokio::fs::copy(&source_path, &backup_path).await?;
        
        Ok(backup_path)
    }

    /// Replace a plugin with an updated version
    pub async fn replace_plugin(&self, plugin_id: &str, new_plugin_path: &Path) -> Result<()> {
        let plugin_dir = self.store_directory.join(plugin_id);
        let stored_path = plugin_dir.join(format!("{plugin_id}.wasm"));
        
        tokio::fs::copy(new_plugin_path, &stored_path).await?;
        
        // Update index
        let metadata = self.extract_plugin_metadata(&stored_path).await?;
        {
            let mut index = self.index.write().await;
            if let Some(entry) = index.plugins.get_mut(plugin_id) {
                entry.metadata = metadata;
            }
        }
        
        self.save_index().await?;
        
        Ok(())
    }

    async fn extract_plugin_metadata(&self, plugin_path: &Path) -> Result<PluginMetadata> {
        // This is a simplified metadata extraction
        // In a real implementation, this would parse the WASM module
        Ok(PluginMetadata {
            name: plugin_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            version: "1.0.0".to_string(),
            description: "WASM Plugin".to_string(),
            author: "Unknown".to_string(),
            capabilities: vec!["basic".to_string()],
        })
    }

    async fn save_index(&self) -> Result<()> {
        let index = self.index.read().await;
        let index_path = self.store_directory.join("index.json");
        let data = serde_json::to_string_pretty(&*index)?;
        tokio::fs::write(&index_path, data).await?;
        Ok(())
    }

    fn calculate_relevance_score(&self, metadata: &PluginMetadata, query: &str) -> f32 {
        let mut score = 0.0;
        
        if metadata.name.contains(query) {
            score += 2.0;
        }
        
        if metadata.description.contains(query) {
            score += 1.0;
        }
        
        if metadata.capabilities.iter().any(|cap| cap.contains(query)) {
            score += 1.5;
        }
        
        score
    }
}

#[cfg(feature = "plugin-management")]
impl PluginStore {
    /// Get plugin info summary (placeholder 刷新: 実体実装)
    pub async fn get_plugin_info(&self, plugin_id: &str) -> Result<PluginInfo> {
        let index = self.index.read().await;
        let entry = index.plugins.get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found", plugin_id))?;
        Ok(PluginInfo {
            id: entry.id.clone(),
            metadata: entry.metadata.clone(),
            status: PluginStatus::Installed,
            install_time: Instant::now(),
            last_used: None,
            usage_count: 0,
        })
    }
}

/// Plugin lifecycle manager
#[derive(Debug)]
pub struct PluginLifecycleManager {
    lifecycle_states: Arc<RwLock<HashMap<String, PluginLifecycleState>>>,
}

impl Default for PluginLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginLifecycleManager {
    pub fn new() -> Self {
        Self {
            lifecycle_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn initialize_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut states = self.lifecycle_states.write().await;
        states.insert(plugin_id.to_string(), PluginLifecycleState {
            current_state: LifecycleState::Initialized,
            state_history: vec![(LifecycleState::Initialized, SystemTime::now())],
        });
        
        info!("Initialized plugin lifecycle: {plugin_id}");
        Ok(())
    }

    pub async fn start_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut states = self.lifecycle_states.write().await;
        if let Some(state) = states.get_mut(plugin_id) {
            state.current_state = LifecycleState::Running;
            state.state_history.push((LifecycleState::Running, SystemTime::now()));
        }
        
        info!("Started plugin: {plugin_id}");
        Ok(())
    }

    pub async fn stop_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut states = self.lifecycle_states.write().await;
        if let Some(state) = states.get_mut(plugin_id) {
            state.current_state = LifecycleState::Stopped;
            state.state_history.push((LifecycleState::Stopped, SystemTime::now()));
        }
        
        info!("Stopped plugin: {plugin_id}");
        Ok(())
    }

    pub async fn cleanup_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut states = self.lifecycle_states.write().await;
        states.remove(plugin_id);
        
        info!("Cleaned up plugin lifecycle: {plugin_id}");
        Ok(())
    }
}

/// Update manager for plugin updates
#[derive(Debug)]
pub struct UpdateManager {
    update_sources: Vec<UpdateSource>,
    update_cache: Arc<RwLock<HashMap<String, CachedUpdate>>>,
}

impl Default for UpdateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateManager {
    pub fn new() -> Self {
        Self {
            update_sources: vec![
                UpdateSource {
                    name: "Official Repository".to_string(),
                    url: "https://plugins.nexusshell.org".to_string(),
                    priority: 1,
                },
            ],
            update_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_for_update(&self, plugin_id: &str) -> Result<Option<PluginUpdate>> {
        // Check cache first
        {
            let cache = self.update_cache.read().await;
            if let Some(cached) = cache.get(plugin_id) {
                if cached.expires_at > SystemTime::now() {
                    return Ok(cached.update.clone());
                }
            }
        }

        // Check update sources
        for source in &self.update_sources {
            if let Some(update) = self.check_source_for_update(source, plugin_id).await? {
                // Cache the result
                {
                    let mut cache = self.update_cache.write().await;
                    cache.insert(plugin_id.to_string(), CachedUpdate {
                        update: Some(update.clone()),
                        expires_at: SystemTime::now() + Duration::from_secs(3600), // 1 hour cache
                    });
                }
                
                return Ok(Some(update));
            }
        }

        Ok(None)
    }

    pub async fn download_update(&self, update: &PluginUpdate) -> Result<PathBuf> {
        // This is a simplified implementation
        // In reality, would download from the update URL
        // Simplified temp directory creation
        let temp_dir = std::env::temp_dir().join(format!("nxsh_plugin_{}", update.plugin_id));
        std::fs::create_dir_all(&temp_dir)?;
        let update_path = temp_dir.join(format!("{}.wasm", update.plugin_id));
        
        // Simulate download
        tokio::fs::write(&update_path, b"fake wasm content").await?;
        
        Ok(update_path)
    }

    async fn check_source_for_update(&self, source: &UpdateSource, plugin_id: &str) -> Result<Option<PluginUpdate>> {
        // Basic skeleton for remote metadata check (disabled when feature missing)
        #[cfg(feature = "remote-plugins")]
        {
            let url = format!("{}/api/v1/plugins/{}/latest", source.url, plugin_id);
            match ureq::get(&url).timeout(std::time::Duration::from_secs(5)).call() {
                Ok(resp) => {
                    let body = resp.into_string().unwrap_or_default();
                    if body.is_empty() { return Ok(None); }
                    match serde_json::from_str::<crate::remote::RemotePluginInfo>(&body) {
                        Ok(info) => Ok(Some(PluginUpdate {
                            plugin_id: plugin_id.to_string(),
                            current_version: "unknown".to_string(),
                            new_version: info.version,
                            download_url: info.download_url,
                            changelog: String::new(),
                        })),
                        Err(_) => Ok(None),
                    }
                }
                Err(_) => Ok(None),
            }
        }
        #[cfg(not(feature = "remote-plugins"))]
        {
            let _ = (source, plugin_id);
            Ok(None)
        }
    }
}

// Supporting data structures

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginIndex {
    pub plugins: HashMap<String, PluginIndexEntry>,
    pub last_updated: SystemTime,
}

impl PluginIndex {
    fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            last_updated: SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginIndexEntry {
    pub id: String,
    pub metadata: PluginMetadata,
    pub stored_path: PathBuf,
    pub installed_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PluginSearchResult {
    pub id: String,
    pub metadata: PluginMetadata,
    pub relevance_score: f32,
}

#[derive(Debug)]
struct CachedPlugin {
    metadata: PluginMetadata,
    cached_at: SystemTime,
}

#[derive(Debug)]
pub struct PluginEntry {
    pub id: String,
    pub handle: crate::wasi_advanced::PluginHandle,
    pub file_path: PathBuf,
    pub status: PluginStatus,
    pub install_time: Instant,
    pub last_used: Option<Instant>,
    pub usage_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginStatus {
    Installed,
    Running,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub metadata: PluginMetadata,
    pub status: PluginStatus,
    pub install_time: Instant,
    pub last_used: Option<Instant>,
    pub usage_count: u64,
}

#[derive(Debug)]
pub struct PluginLifecycleState {
    pub current_state: LifecycleState,
    pub state_history: Vec<(LifecycleState, SystemTime)>,
}

#[derive(Debug, Clone)]
pub enum LifecycleState {
    Initialized,
    Running,
    Stopped,
    Error,
}

#[derive(Debug, Clone)]
pub struct UpdateSource {
    pub name: String,
    pub url: String,
    pub priority: u32,
}

#[derive(Debug, Clone)]
pub struct PluginUpdate {
    pub plugin_id: String,
    pub current_version: String,
    pub new_version: String,
    pub download_url: String,
    pub changelog: String,
}

#[derive(Debug)]
struct CachedUpdate {
    update: Option<PluginUpdate>,
    expires_at: SystemTime,
}

#[derive(Debug, Clone)]
pub struct PluginManagerConfig {
    pub store_directory: PathBuf,
    pub plugins_data_dir: PathBuf,
    pub default_max_memory: u64,
    pub default_max_cpu_time: Duration,
    pub default_max_file_handles: u64,
    pub auto_update_enabled: bool,
    pub update_check_interval: Duration,
}

impl Default for PluginManagerConfig {
    fn default() -> Self {
        Self {
            store_directory: PathBuf::from("~/.nxsh/plugins"),
            plugins_data_dir: PathBuf::from("~/.nxsh/plugin_data"),
            default_max_memory: 64 * 1024 * 1024, // 64MB
            default_max_cpu_time: Duration::from_secs(30),
            default_max_file_handles: 100,
            auto_update_enabled: true,
            update_check_interval: Duration::from_secs(3600), // 1 hour
        }
    }
}

#[derive(Debug)]
pub struct PluginStats {
    pub usage_count: u64,
    pub last_used: Option<Instant>,
    pub install_time: Instant,
    pub resource_usage: crate::security_sandbox::ResourceUsage,
    pub performance_metrics: crate::wasi_advanced::PerformanceStats,
}

#[derive(Debug)]
pub struct SecurityInfo {
    pub total_plugins: usize,
    pub security_violations: usize,
    pub threat_level: crate::security_sandbox::ThreatLevel,
}
