//! Dynamic Plugin Loading System
//!
//! This module provides advanced dynamic loading capabilities for WebAssembly plugins,
//! including hot reloading, dependency resolution, and version management.

use anyhow::{Result, Context};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, Duration},
    fs,
};
use tokio::{
    sync::{RwLock, Mutex, Semaphore},
    fs as async_fs,
};
use serde::{Deserialize, Serialize};
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use semver::{Version, VersionReq};
use sha2::{Sha256, Digest};

use crate::{
    PluginMetadata, PluginError, PluginResult,
    runtime::WasiPluginRuntime,
    resource_table::AdvancedResourceTable,
};

/// Dynamic plugin loader with hot reloading and dependency management
#[derive(Debug)]
pub struct DynamicPluginLoader {
    /// Plugin loading configuration
    config: LoaderConfig,
    /// Currently loaded plugins
    loaded_plugins: Arc<RwLock<HashMap<String, LoadedPluginInfo>>>,
    /// Plugin dependency graph
    dependency_graph: Arc<RwLock<DependencyGraph>>,
    /// File system watcher for hot reloading
    file_watcher: Arc<Mutex<Option<notify::RecommendedWatcher>>>,
    /// Plugin registry with versions
    plugin_registry: Arc<RwLock<PluginRegistry>>,
    /// Loading semaphore to prevent concurrent loads
    loading_semaphore: Arc<Semaphore>,
    /// Runtime reference
    runtime: Arc<RwLock<Option<Arc<WasiPluginRuntime>>>>,
    /// Hot reload callback
    reload_callbacks: Arc<RwLock<Vec<Box<dyn ReloadCallback + Send + Sync>>>>,
}

/// Plugin loading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderConfig {
    /// Plugin search directories
    pub plugin_directories: Vec<PathBuf>,
    /// Enable hot reloading
    pub enable_hot_reload: bool,
    /// File watch debounce duration
    pub watch_debounce: Duration,
    /// Maximum concurrent loads
    pub max_concurrent_loads: usize,
    /// Plugin cache directory
    pub cache_directory: Option<PathBuf>,
    /// Enable dependency resolution
    pub enable_dependency_resolution: bool,
    /// Plugin validation settings
    pub validation: ValidationConfig,
    /// Version compatibility rules
    pub version_compatibility: VersionCompatibility,
}

/// Plugin validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Require digital signatures
    pub require_signature: bool,
    /// Maximum plugin file size (bytes)
    pub max_file_size: u64,
    /// Allowed plugin extensions
    pub allowed_extensions: Vec<String>,
    /// Blocked plugin patterns
    pub blocked_patterns: Vec<String>,
    /// Minimum security version
    pub min_security_version: String,
}

/// Version compatibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionCompatibility {
    /// Strict semver compatibility
    pub strict_semver: bool,
    /// Allow major version upgrades
    pub allow_major_upgrades: bool,
    /// Allow downgrades
    pub allow_downgrades: bool,
    /// Version range requirements
    pub version_requirements: HashMap<String, String>,
}

/// Loaded plugin information with extended metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedPluginInfo {
    pub plugin_id: String,
    pub metadata: PluginMetadata,
    pub file_path: PathBuf,
    pub file_hash: String,
    pub load_time: SystemTime,
    pub last_reload: Option<SystemTime>,
    pub reload_count: u32,
    pub version: Version,
    pub dependencies: Vec<PluginDependency>,
    pub dependents: Vec<String>,
    pub load_status: LoadStatus,
    pub performance_metrics: PerformanceMetrics,
}

/// Plugin dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub plugin_id: String,
    pub version_req: VersionReq,
    pub optional: bool,
    pub features: Vec<String>,
    pub resolved_version: Option<Version>,
}

/// Plugin loading status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadStatus {
    /// Plugin is loading
    Loading,
    /// Plugin loaded successfully
    Loaded,
    /// Plugin failed to load
    Failed(String),
    /// Plugin is being reloaded
    Reloading,
    /// Plugin is unloading
    Unloading,
    /// Plugin dependency not met
    DependencyFailed(String),
}

/// Plugin performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub load_duration: Duration,
    pub initialization_duration: Duration,
    pub memory_usage: u64,
    pub cpu_usage_percent: f64,
    pub api_call_count: u64,
    pub error_count: u64,
    pub last_activity: SystemTime,
}

/// Plugin registry for version management
#[derive(Debug, Clone)]
pub struct PluginRegistry {
    /// Available plugin versions
    plugins: HashMap<String, Vec<PluginVersion>>,
    /// Plugin aliases
    aliases: HashMap<String, String>,
    /// Registry cache
    cache: HashMap<String, CachedPluginInfo>,
}

/// Plugin version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginVersion {
    pub version: Version,
    pub file_path: PathBuf,
    pub metadata: PluginMetadata,
    pub file_hash: String,
    pub discovered_at: SystemTime,
    pub compatibility_info: CompatibilityInfo,
}

/// Compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    pub min_nexus_version: Version,
    pub max_nexus_version: Option<Version>,
    pub supported_features: Vec<String>,
    pub deprecated_features: Vec<String>,
    pub breaking_changes: Vec<String>,
}

/// Cached plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPluginInfo {
    pub metadata: PluginMetadata,
    pub file_hash: String,
    pub cache_time: SystemTime,
    pub validation_result: ValidationResult,
}

/// Plugin validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub signature_valid: bool,
    pub size_valid: bool,
    pub format_valid: bool,
    pub security_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Dependency graph for plugin relationships
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Direct dependencies
    dependencies: HashMap<String, HashSet<String>>,
    /// Reverse dependencies (dependents)
    dependents: HashMap<String, HashSet<String>>,
    /// Dependency resolution cache
    resolution_cache: HashMap<String, Vec<String>>,
}

/// Hot reload callback trait
pub trait ReloadCallback {
    /// Called before plugin reload
    fn before_reload(&self, plugin_id: &str) -> Result<()>;
    
    /// Called after successful reload
    fn after_reload(&self, plugin_id: &str, old_version: &Version, new_version: &Version) -> Result<()>;
    
    /// Called when reload fails
    fn reload_failed(&self, plugin_id: &str, error: &str) -> Result<()>;
}

/// Plugin discovery result
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    pub discovered_plugins: Vec<DiscoveredPlugin>,
    pub failed_discoveries: Vec<DiscoveryError>,
    pub scan_duration: Duration,
}

/// Discovered plugin information
#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    pub file_path: PathBuf,
    pub metadata: PluginMetadata,
    pub file_hash: String,
    pub file_size: u64,
    pub discovery_time: SystemTime,
}

/// Discovery error information
#[derive(Debug, Clone)]
pub struct DiscoveryError {
    pub file_path: PathBuf,
    pub error: String,
    pub error_type: DiscoveryErrorType,
}

/// Types of discovery errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryErrorType {
    InvalidFormat,
    ValidationFailed,
    IoError,
    MetadataError,
    SignatureError,
}

impl DynamicPluginLoader {
    /// Create a new dynamic plugin loader
    pub fn new(config: LoaderConfig) -> Result<Self> {
        Ok(Self {
            config,
            loaded_plugins: Arc::new(RwLock::new(HashMap::new())),
            dependency_graph: Arc::new(RwLock::new(DependencyGraph::new())),
            file_watcher: Arc::new(Mutex::new(None)),
            plugin_registry: Arc::new(RwLock::new(PluginRegistry::new())),
            loading_semaphore: Arc::new(Semaphore::new(config.max_concurrent_loads)),
            runtime: Arc::new(RwLock::new(None)),
            reload_callbacks: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Initialize the loader with runtime
    pub async fn initialize(&self, runtime: Arc<WasiPluginRuntime>) -> Result<()> {
        {
            let mut runtime_guard = self.runtime.write().await;
            *runtime_guard = Some(runtime);
        }

        // Create cache directory if specified
        if let Some(cache_dir) = &self.config.cache_directory {
            async_fs::create_dir_all(cache_dir).await
                .context("Failed to create cache directory")?;
        }

        // Start file watcher if hot reload is enabled
        if self.config.enable_hot_reload {
            self.start_file_watcher().await?;
        }

        // Perform initial plugin discovery
        self.discover_plugins().await?;

        log::info!("Dynamic plugin loader initialized successfully");
        Ok(())
    }

    /// Discover plugins in configured directories
    pub async fn discover_plugins(&self) -> Result<DiscoveryResult> {
        let start_time = SystemTime::now();
        let mut discovered_plugins = Vec::new();
        let mut failed_discoveries = Vec::new();

        for plugin_dir in &self.config.plugin_directories {
            if !plugin_dir.exists() {
                log::warn!("Plugin directory does not exist: {}", plugin_dir.display());
                continue;
            }

            match self.scan_directory(plugin_dir).await {
                Ok(mut plugins) => discovered_plugins.append(&mut plugins),
                Err(e) => {
                    failed_discoveries.push(DiscoveryError {
                        file_path: plugin_dir.clone(),
                        error: e.to_string(),
                        error_type: DiscoveryErrorType::IoError,
                    });
                }
            }
        }

        // Update plugin registry
        {
            let mut registry = self.plugin_registry.write().await;
            for plugin in &discovered_plugins {
                registry.add_plugin_version(plugin).await?;
            }
        }

        let scan_duration = start_time.elapsed().unwrap_or(Duration::ZERO);
        
        log::info!("Discovered {} plugins, {} failures in {:?}", 
                  discovered_plugins.len(), failed_discoveries.len(), scan_duration);

        Ok(DiscoveryResult {
            discovered_plugins,
            failed_discoveries,
            scan_duration,
        })
    }

    /// Load a plugin with dependency resolution
    pub async fn load_plugin(&self, plugin_id: &str, version_req: Option<&VersionReq>) -> Result<()> {
        let _permit = self.loading_semaphore.acquire().await
            .context("Failed to acquire loading semaphore")?;

        // Check if already loaded
        {
            let loaded = self.loaded_plugins.read().await;
            if let Some(info) = loaded.get(plugin_id) {
                if info.load_status == LoadStatus::Loaded {
                    return Ok(());
                }
            }
        }

        // Find best version
        let plugin_version = self.find_best_version(plugin_id, version_req).await?;
        
        // Resolve dependencies
        if self.config.enable_dependency_resolution {
            self.resolve_dependencies(&plugin_version).await?;
        }

        // Validate plugin
        let validation_result = self.validate_plugin(&plugin_version).await?;
        if !validation_result.is_valid {
            return Err(anyhow::anyhow!("Plugin validation failed: {:?}", validation_result.errors));
        }

        // Load plugin
        let load_start = SystemTime::now();
        let result = self.load_plugin_internal(&plugin_version).await;
        let load_duration = load_start.elapsed().unwrap_or(Duration::ZERO);

        match result {
            Ok(()) => {
                // Update plugin info
                {
                    let mut loaded = self.loaded_plugins.write().await;
                    if let Some(info) = loaded.get_mut(plugin_id) {
                        info.load_status = LoadStatus::Loaded;
                        info.performance_metrics.load_duration = load_duration;
                        info.last_reload = Some(SystemTime::now());
                    }
                }
                
                log::info!("Successfully loaded plugin {} v{}", plugin_id, plugin_version.version);
                Ok(())
            }
            Err(e) => {
                // Update failure status
                {
                    let mut loaded = self.loaded_plugins.write().await;
                    if let Some(info) = loaded.get_mut(plugin_id) {
                        info.load_status = LoadStatus::Failed(e.to_string());
                    }
                }
                
                log::error!("Failed to load plugin {}: {}", plugin_id, e);
                Err(e)
            }
        }
    }

    /// Unload a plugin and its dependents
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        // Find dependents
        let dependents = {
            let graph = self.dependency_graph.read().await;
            graph.get_dependents(plugin_id)
        };

        // Unload dependents first
        for dependent in dependents {
            if dependent != plugin_id {
                self.unload_plugin(&dependent).await?;
            }
        }

        // Update status
        {
            let mut loaded = self.loaded_plugins.write().await;
            if let Some(info) = loaded.get_mut(plugin_id) {
                info.load_status = LoadStatus::Unloading;
            }
        }

        // Unload from runtime
        if let Some(runtime) = self.get_runtime().await {
            runtime.unload_plugin(plugin_id).await
                .context("Failed to unload plugin from runtime")?;
        }

        // Remove from loaded plugins
        {
            let mut loaded = self.loaded_plugins.write().await;
            loaded.remove(plugin_id);
        }

        // Update dependency graph
        {
            let mut graph = self.dependency_graph.write().await;
            graph.remove_plugin(plugin_id);
        }

        log::info!("Successfully unloaded plugin {}", plugin_id);
        Ok(())
    }

    /// Hot reload a plugin
    pub async fn reload_plugin(&self, plugin_id: &str) -> Result<()> {
        let old_info = {
            let loaded = self.loaded_plugins.read().await;
            loaded.get(plugin_id).cloned()
        };

        let old_version = old_info.as_ref().map(|info| info.version.clone());

        // Execute before reload callbacks
        self.execute_reload_callbacks(|cb| cb.before_reload(plugin_id)).await?;

        // Update status
        {
            let mut loaded = self.loaded_plugins.write().await;
            if let Some(info) = loaded.get_mut(plugin_id) {
                info.load_status = LoadStatus::Reloading;
            }
        }

        // Unload current version
        if old_info.is_some() {
            self.unload_plugin(plugin_id).await?;
        }

        // Load new version
        match self.load_plugin(plugin_id, None).await {
            Ok(()) => {
                let new_version = {
                    let loaded = self.loaded_plugins.read().await;
                    loaded.get(plugin_id).map(|info| info.version.clone())
                };

                // Update reload count
                {
                    let mut loaded = self.loaded_plugins.write().await;
                    if let Some(info) = loaded.get_mut(plugin_id) {
                        info.reload_count += 1;
                        info.last_reload = Some(SystemTime::now());
                    }
                }

                // Execute after reload callbacks
                if let (Some(old_ver), Some(new_ver)) = (old_version, new_version) {
                    self.execute_reload_callbacks(|cb| cb.after_reload(plugin_id, &old_ver, &new_ver)).await?;
                }

                log::info!("Successfully reloaded plugin {}", plugin_id);
                Ok(())
            }
            Err(e) => {
                // Execute failed reload callbacks
                self.execute_reload_callbacks(|cb| cb.reload_failed(plugin_id, &e.to_string())).await?;
                
                log::error!("Failed to reload plugin {}: {}", plugin_id, e);
                Err(e)
            }
        }
    }

    /// Add hot reload callback
    pub async fn add_reload_callback(&self, callback: Box<dyn ReloadCallback + Send + Sync>) -> Result<()> {
        let mut callbacks = self.reload_callbacks.write().await;
        callbacks.push(callback);
        Ok(())
    }

    /// Get loaded plugin information
    pub async fn get_loaded_plugin_info(&self, plugin_id: &str) -> Option<LoadedPluginInfo> {
        let loaded = self.loaded_plugins.read().await;
        loaded.get(plugin_id).cloned()
    }

    /// List all loaded plugins
    pub async fn list_loaded_plugins(&self) -> Vec<String> {
        let loaded = self.loaded_plugins.read().await;
        loaded.keys().cloned().collect()
    }

    /// Get plugin dependency graph
    pub async fn get_dependency_graph(&self) -> DependencyGraph {
        let graph = self.dependency_graph.read().await;
        graph.clone()
    }

    // Private implementation methods

    async fn scan_directory(&self, dir: &Path) -> Result<Vec<DiscoveredPlugin>> {
        let mut plugins = Vec::new();
        let mut read_dir = async_fs::read_dir(dir).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            
            // Check if file has allowed extension
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                if !self.config.validation.allowed_extensions.contains(&extension.to_string()) {
                    continue;
                }
            }

            // Check file size
            if let Ok(metadata) = entry.metadata().await {
                if metadata.len() > self.config.validation.max_file_size {
                    log::warn!("Plugin file too large: {}", path.display());
                    continue;
                }
            }

            // Try to discover plugin
            match self.discover_plugin(&path).await {
                Ok(plugin) => plugins.push(plugin),
                Err(e) => log::warn!("Failed to discover plugin {}: {}", path.display(), e),
            }
        }

        Ok(plugins)
    }

    async fn discover_plugin(&self, path: &Path) -> Result<DiscoveredPlugin> {
        // Calculate file hash
        let file_content = async_fs::read(path).await?;
        let file_hash = format!("{:x}", Sha256::digest(&file_content));
        
        // Check cache first
        if let Some(cached) = self.get_cached_plugin_info(&file_hash).await {
            return Ok(DiscoveredPlugin {
                file_path: path.to_path_buf(),
                metadata: cached.metadata,
                file_hash,
                file_size: file_content.len() as u64,
                discovery_time: SystemTime::now(),
            });
        }

        // Extract metadata (simplified for now)
        let metadata = self.extract_plugin_metadata(&file_content, path).await?;

        Ok(DiscoveredPlugin {
            file_path: path.to_path_buf(),
            metadata,
            file_hash,
            file_size: file_content.len() as u64,
            discovery_time: SystemTime::now(),
        })
    }

    async fn extract_plugin_metadata(&self, content: &[u8], path: &Path) -> Result<PluginMetadata> {
        // Improved metadata extraction from WebAssembly custom sections
        let mut metadata = PluginMetadata {
            name: path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            version: "0.1.0".to_string(),
            description: "WebAssembly plugin".to_string(),
            author: "Unknown".to_string(),
            license: "Unknown".to_string(),
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            dependencies: HashMap::new(),
            capabilities: vec![],
            min_nexus_version: "0.1.0".to_string(),
            max_nexus_version: None,
        };

        // Try to parse WebAssembly format and extract custom sections
        if let Ok(wasm_metadata) = self.parse_wasm_metadata(content) {
            if let Some(name) = wasm_metadata.get("name") {
                metadata.name = name.clone();
            }
            if let Some(version) = wasm_metadata.get("version") {
                metadata.version = version.clone();
            }
            if let Some(description) = wasm_metadata.get("description") {
                metadata.description = description.clone();
            }
            if let Some(author) = wasm_metadata.get("author") {
                metadata.author = author.clone();
            }
            if let Some(license) = wasm_metadata.get("license") {
                metadata.license = license.clone();
            }
            if let Some(homepage) = wasm_metadata.get("homepage") {
                metadata.homepage = Some(homepage.clone());
            }
            if let Some(repository) = wasm_metadata.get("repository") {
                metadata.repository = Some(repository.clone());
            }
        }

        // Try to extract from embedded JSON manifest
        if let Ok(json_metadata) = self.extract_json_manifest(content) {
            metadata = self.merge_metadata(metadata, json_metadata);
        }

        Ok(metadata)
    }

    /// Parse WebAssembly custom sections for metadata
    fn parse_wasm_metadata(&self, content: &[u8]) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();

        // Simple WASM format validation - check magic number
        if content.len() < 8 || &content[0..4] != b"\0asm" {
            return Ok(metadata); // Not a valid WASM file
        }

        // Look for custom sections (section type 0)
        let mut offset = 8; // Skip magic number and version
        while offset < content.len() {
            if offset + 1 >= content.len() {
                break;
            }

            let section_type = content[offset];
            offset += 1;

            // Parse section size (simplified LEB128 parsing)
            let (section_size, size_bytes) = self.parse_leb128(&content[offset..])?;
            offset += size_bytes;

            if section_type == 0 && offset + section_size <= content.len() {
                // Custom section - try to extract metadata
                let section_data = &content[offset..offset + section_size];
                if let Ok(section_metadata) = self.parse_custom_section(section_data) {
                    metadata.extend(section_metadata);
                }
            }

            offset += section_size;
        }

        Ok(metadata)
    }

    /// Parse LEB128 encoded integer (simplified version)
    fn parse_leb128(&self, data: &[u8]) -> Result<(usize, usize)> {
        let mut result = 0;
        let mut shift = 0;
        let mut bytes_read = 0;

        for &byte in data.iter().take(5) { // Max 5 bytes for 32-bit LEB128
            bytes_read += 1;
            result |= ((byte & 0x7F) as usize) << shift;
            
            if byte & 0x80 == 0 {
                return Ok((result, bytes_read));
            }
            
            shift += 7;
        }

        Err(anyhow::anyhow!("Invalid LEB128 encoding"))
    }

    /// Parse custom section content for metadata
    fn parse_custom_section(&self, data: &[u8]) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();

        // Parse section name
        if data.is_empty() {
            return Ok(metadata);
        }

        let name_len = data[0] as usize;
        if data.len() < 1 + name_len {
            return Ok(metadata);
        }

        let section_name = String::from_utf8_lossy(&data[1..1 + name_len]);
        
        // Look for "nexus-plugin" custom section
        if section_name == "nexus-plugin" {
            let payload = &data[1 + name_len..];
            
            // Try to parse as JSON
            if let Ok(json_str) = std::str::from_utf8(payload) {
                if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(obj) = json_data.as_object() {
                        for (key, value) in obj {
                            if let Some(str_value) = value.as_str() {
                                metadata.insert(key.clone(), str_value.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(metadata)
    }

    /// Extract JSON manifest embedded in the binary
    fn extract_json_manifest(&self, content: &[u8]) -> Result<PluginMetadata> {
        // Look for JSON manifest markers
        let start_marker = b"NEXUS_PLUGIN_MANIFEST_START";
        let end_marker = b"NEXUS_PLUGIN_MANIFEST_END";

        if let Some(start_pos) = self.find_pattern(content, start_marker) {
            if let Some(end_pos) = self.find_pattern(&content[start_pos + start_marker.len()..], end_marker) {
                let manifest_start = start_pos + start_marker.len();
                let manifest_end = manifest_start + end_pos;
                
                let manifest_data = &content[manifest_start..manifest_end];
                if let Ok(manifest_str) = std::str::from_utf8(manifest_data) {
                    if let Ok(metadata) = serde_json::from_str::<PluginMetadata>(manifest_str) {
                        return Ok(metadata);
                    }
                }
            }
        }

        Err(anyhow::anyhow!("No JSON manifest found"))
    }

    /// Find byte pattern in content
    fn find_pattern(&self, content: &[u8], pattern: &[u8]) -> Option<usize> {
        content.windows(pattern.len()).position(|window| window == pattern)
    }

    /// Merge metadata from different sources
    fn merge_metadata(&self, mut base: PluginMetadata, overlay: PluginMetadata) -> PluginMetadata {
        // Overlay takes precedence for non-empty fields
        if !overlay.name.is_empty() && overlay.name != "unknown" {
            base.name = overlay.name;
        }
        if overlay.version != "0.1.0" {
            base.version = overlay.version;
        }
        if overlay.description != "WebAssembly plugin" {
            base.description = overlay.description;
        }
        if overlay.author != "Unknown" {
            base.author = overlay.author;
        }
        if overlay.license != "Unknown" {
            base.license = overlay.license;
        }
        if overlay.homepage.is_some() {
            base.homepage = overlay.homepage;
        }
        if overlay.repository.is_some() {
            base.repository = overlay.repository;
        }
        if !overlay.keywords.is_empty() {
            base.keywords = overlay.keywords;
        }
        if !overlay.categories.is_empty() {
            base.categories = overlay.categories;
        }
        if !overlay.dependencies.is_empty() {
            base.dependencies = overlay.dependencies;
        }
        if !overlay.capabilities.is_empty() {
            base.capabilities = overlay.capabilities;
        }

        base
    }
    }

    async fn find_best_version(&self, plugin_id: &str, version_req: Option<&VersionReq>) -> Result<PluginVersion> {
        let registry = self.plugin_registry.read().await;
        
        let versions = registry.plugins.get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} not found in registry", plugin_id))?;

        let best_version = if let Some(req) = version_req {
            versions.iter()
                .filter(|v| req.matches(&v.version))
                .max_by(|a, b| a.version.cmp(&b.version))
                .ok_or_else(|| anyhow::anyhow!("No version satisfies requirement"))?
        } else {
            versions.iter()
                .max_by(|a, b| a.version.cmp(&b.version))
                .ok_or_else(|| anyhow::anyhow!("No versions available"))?
        };

        Ok(best_version.clone())
    }

    async fn resolve_dependencies(&self, plugin_version: &PluginVersion) -> Result<()> {
        // Improved dependency resolution with cycle detection and proper ordering
        let mut resolution_graph = HashMap::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        
        // Build dependency resolution order
        let resolution_order = self.build_dependency_order(
            &plugin_version.metadata.name,
            plugin_version,
            &mut resolution_graph,
            &mut visited,
            &mut visiting
        ).await?;

        // Load dependencies in correct order
        for dep_name in resolution_order {
            if let Some(dep_version) = resolution_graph.get(&dep_name) {
                // Check if dependency is already loaded with compatible version
                let loaded = self.loaded_plugins.read().await;
                if let Some(dep_info) = loaded.get(&dep_name) {
                    if dep_version.version == dep_info.version {
                        continue; // Dependency satisfied
                    } else {
                        // Version conflict - need to handle gracefully
                        return Err(anyhow::anyhow!(
                            "Version conflict for dependency {}: loaded {} but need {}",
                            dep_name, dep_info.version, dep_version.version
                        ));
                    }
                }
                drop(loaded);

                // Load dependency
                self.load_plugin_version(dep_version).await
                    .context(format!("Failed to load dependency {}", dep_name))?;
            }
        }

        Ok(())
    }

    /// Build dependency resolution order with cycle detection
    async fn build_dependency_order(
        &self,
        plugin_name: &str,
        plugin_version: &PluginVersion,
        resolution_graph: &mut HashMap<String, PluginVersion>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>
    ) -> Result<Vec<String>> {
        // Cycle detection
        if visiting.contains(plugin_name) {
            return Err(anyhow::anyhow!("Circular dependency detected involving {}", plugin_name));
        }

        if visited.contains(plugin_name) {
            return Ok(vec![]); // Already processed
        }

        visiting.insert(plugin_name.to_string());
        resolution_graph.insert(plugin_name.to_string(), plugin_version.clone());

        let mut order = Vec::new();

        // Process dependencies first
        for (dep_name, dep_version_req) in &plugin_version.metadata.dependencies {
            let dep_version = self.find_best_version(dep_name, Some(&VersionReq::parse(dep_version_req)?)).await?;
            
            let sub_order = self.build_dependency_order(
                dep_name,
                &dep_version,
                resolution_graph,
                visited,
                visiting
            ).await?;
            
            order.extend(sub_order);
        }

        visiting.remove(plugin_name);
        visited.insert(plugin_name.to_string());
        
        // Add current plugin after its dependencies
        if plugin_name != &plugin_version.metadata.name {
            order.push(plugin_name.to_string());
        }

        Ok(order)
    }

    /// Load a specific plugin version
    async fn load_plugin_version(&self, plugin_version: &PluginVersion) -> Result<()> {
        let version_req = VersionReq::parse(&format!("={}", plugin_version.version))?;
        self.load_plugin(&plugin_version.metadata.name, Some(&version_req)).await
    }

    async fn validate_plugin(&self, plugin_version: &PluginVersion) -> Result<ValidationResult> {
        let mut result = ValidationResult {
            is_valid: true,
            signature_valid: true,
            size_valid: true,
            format_valid: true,
            security_valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
        };

        // Check file size
        if let Ok(metadata) = fs::metadata(&plugin_version.file_path) {
            if metadata.len() > self.config.validation.max_file_size {
                result.size_valid = false;
                result.errors.push("Plugin file exceeds maximum size".to_string());
            }
        }

        // Check extension
        if let Some(extension) = plugin_version.file_path.extension().and_then(|ext| ext.to_str()) {
            if !self.config.validation.allowed_extensions.contains(&extension.to_string()) {
                result.format_valid = false;
                result.errors.push("Plugin file extension not allowed".to_string());
            }
        }

        // Check blocked patterns
        let path_str = plugin_version.file_path.to_string_lossy();
        for pattern in &self.config.validation.blocked_patterns {
            if path_str.contains(pattern) {
                result.security_valid = false;
                result.errors.push(format!("Plugin path matches blocked pattern: {}", pattern));
            }
        }

        // Update overall validity
        result.is_valid = result.signature_valid && result.size_valid && result.format_valid && result.security_valid;

        Ok(result)
    }

    async fn load_plugin_internal(&self, plugin_version: &PluginVersion) -> Result<()> {
        // Get runtime
        let runtime = self.get_runtime().await
            .ok_or_else(|| anyhow::anyhow!("Runtime not available"))?;

        // Load plugin in runtime
        let metadata = runtime.load_plugin(&plugin_version.file_path, plugin_version.metadata.name.clone()).await
            .context("Failed to load plugin in runtime")?;

        // Create loaded plugin info
        let plugin_info = LoadedPluginInfo {
            plugin_id: plugin_version.metadata.name.clone(),
            metadata,
            file_path: plugin_version.file_path.clone(),
            file_hash: plugin_version.file_hash.clone(),
            load_time: SystemTime::now(),
            last_reload: None,
            reload_count: 0,
            version: plugin_version.version.clone(),
            dependencies: Vec::new(), // Will be populated during dependency resolution
            dependents: Vec::new(),
            load_status: LoadStatus::Loading,
            performance_metrics: PerformanceMetrics {
                load_duration: Duration::ZERO,
                initialization_duration: Duration::ZERO,
                memory_usage: 0,
                cpu_usage_percent: 0.0,
                api_call_count: 0,
                error_count: 0,
                last_activity: SystemTime::now(),
            },
        };

        // Store plugin info
        {
            let mut loaded = self.loaded_plugins.write().await;
            loaded.insert(plugin_version.metadata.name.clone(), plugin_info);
        }

        Ok(())
    }

    async fn start_file_watcher(&self) -> Result<()> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = watcher(tx, self.config.watch_debounce)?;

        // Watch plugin directories
        for dir in &self.config.plugin_directories {
            if dir.exists() {
                watcher.watch(dir, RecursiveMode::Recursive)?;
            }
        }

        // Store watcher
        {
            let mut file_watcher = self.file_watcher.lock().await;
            *file_watcher = Some(watcher);
        }

        // Start event processing
        let loader = Arc::new(self);
        tokio::spawn(async move {
            loop {
                match rx.recv() {
                    Ok(event) => {
                        if let Err(e) = loader.handle_file_event(event).await {
                            log::error!("Failed to handle file event: {}", e);
                        }
                    }
                    Err(_) => {
                        log::info!("File watcher channel closed");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn handle_file_event(&self, event: DebouncedEvent) -> Result<()> {
        match event {
            DebouncedEvent::Write(path) | DebouncedEvent::Create(path) => {
                // Check if this is a plugin file
                if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                    if self.config.validation.allowed_extensions.contains(&extension.to_string()) {
                        // Find plugin by file path
                        let plugin_id = self.find_plugin_by_path(&path).await;
                        
                        if let Some(id) = plugin_id {
                            log::info!("Hot reloading plugin {} due to file change", id);
                            if let Err(e) = self.reload_plugin(&id).await {
                                log::error!("Hot reload failed for plugin {}: {}", id, e);
                            }
                        }
                    }
                }
            }
            DebouncedEvent::Remove(path) => {
                // Handle plugin removal
                if let Some(plugin_id) = self.find_plugin_by_path(&path).await {
                    log::info!("Unloading plugin {} due to file removal", plugin_id);
                    if let Err(e) = self.unload_plugin(&plugin_id).await {
                        log::error!("Failed to unload removed plugin {}: {}", plugin_id, e);
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    async fn find_plugin_by_path(&self, path: &Path) -> Option<String> {
        let loaded = self.loaded_plugins.read().await;
        for (plugin_id, info) in loaded.iter() {
            if info.file_path == path {
                return Some(plugin_id.clone());
            }
        }
        None
    }

    async fn get_runtime(&self) -> Option<Arc<WasiPluginRuntime>> {
        let runtime_guard = self.runtime.read().await;
        runtime_guard.clone()
    }

    async fn get_cached_plugin_info(&self, file_hash: &str) -> Option<CachedPluginInfo> {
        let registry = self.plugin_registry.read().await;
        registry.cache.get(file_hash).cloned()
    }

    async fn execute_reload_callbacks<F>(&self, operation: F) -> Result<()>
    where
        F: Fn(&dyn ReloadCallback) -> Result<()>,
    {
        let callbacks = self.reload_callbacks.read().await;
        for callback in callbacks.iter() {
            if let Err(e) = operation(callback.as_ref()) {
                log::warn!("Reload callback failed: {}", e);
            }
        }
        Ok(())
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            aliases: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    pub async fn add_plugin_version(&mut self, plugin: &DiscoveredPlugin) -> Result<()> {
        let version = Version::parse(&plugin.metadata.version)?;
        
        let plugin_version = PluginVersion {
            version,
            file_path: plugin.file_path.clone(),
            metadata: plugin.metadata.clone(),
            file_hash: plugin.file_hash.clone(),
            discovered_at: plugin.discovery_time,
            compatibility_info: CompatibilityInfo {
                min_nexus_version: Version::parse(&plugin.metadata.min_nexus_version)?,
                max_nexus_version: plugin.metadata.max_nexus_version.as_ref()
                    .map(|v| Version::parse(v))
                    .transpose()?,
                supported_features: plugin.metadata.capabilities.clone(),
                deprecated_features: vec![],
                breaking_changes: vec![],
            },
        };

        self.plugins.entry(plugin.metadata.name.clone())
            .or_insert_with(Vec::new)
            .push(plugin_version);

        Ok(())
    }
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
            resolution_cache: HashMap::new(),
        }
    }

    pub fn get_dependents(&self, plugin_id: &str) -> Vec<String> {
        self.dependents.get(plugin_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn remove_plugin(&mut self, plugin_id: &str) {
        self.dependencies.remove(plugin_id);
        self.dependents.remove(plugin_id);
        self.resolution_cache.clear();
    }
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            plugin_directories: vec![PathBuf::from("plugins")],
            enable_hot_reload: false,
            watch_debounce: Duration::from_millis(500),
            max_concurrent_loads: 4,
            cache_directory: Some(PathBuf::from("cache/plugins")),
            enable_dependency_resolution: true,
            validation: ValidationConfig::default(),
            version_compatibility: VersionCompatibility::default(),
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            require_signature: false,
            max_file_size: 50 * 1024 * 1024, // 50MB
            allowed_extensions: vec!["wasm".to_string(), "wat".to_string()],
            blocked_patterns: vec![],
            min_security_version: "0.1.0".to_string(),
        }
    }
}

impl Default for VersionCompatibility {
    fn default() -> Self {
        Self {
            strict_semver: true,
            allow_major_upgrades: false,
            allow_downgrades: false,
            version_requirements: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_plugin_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_path = temp_dir.path().join("test_plugin.wasm");
        std::fs::write(&plugin_path, b"fake wasm content").unwrap();

        let mut config = LoaderConfig::default();
        config.plugin_directories = vec![temp_dir.path().to_path_buf()];

        let loader = DynamicPluginLoader::new(config).unwrap();
        let result = loader.discover_plugins().await.unwrap();

        assert_eq!(result.discovered_plugins.len(), 1);
        assert_eq!(result.discovered_plugins[0].file_path, plugin_path);
    }

    #[tokio::test]
    async fn test_version_parsing() {
        let version = Version::parse("1.2.3").unwrap();
        let req = VersionReq::parse("^1.2.0").unwrap();
        assert!(req.matches(&version));
    }

    #[tokio::test]
    async fn test_dependency_graph() {
        let mut graph = DependencyGraph::new();
        
        // Test empty graph
        assert_eq!(graph.get_dependents("test"), Vec::<String>::new());
        
        // Test removal
        graph.remove_plugin("nonexistent");
        assert_eq!(graph.dependencies.len(), 0);
    }
}
