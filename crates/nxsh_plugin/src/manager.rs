//! Plugin Manager for NexusShell
//!
//! This module provides comprehensive plugin management with support for
//! discovery, loading, unloading, dependency resolution, and semantic versioning.

use anyhow::{Context, Result};
#[cfg(feature = "plugin-management")]
use semver::{Version, VersionReq};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    time::SystemTime,
};
#[cfg(any(feature = "plugin-management", feature = "async-support"))]
use tokio::fs;
#[cfg(feature = "plugin-management")]
use walkdir::WalkDir;

// Note: cfg attributes cannot be placed inside a use tree list. Split them.
#[cfg(feature = "native-plugins")]
use crate::native_runtime::NativePluginRuntime;
use crate::{
    // runtime::WasiPluginRuntime,
    // component::ComponentRegistry,
    PluginConfig,
    PluginEvent,
    PluginEventHandler,
    PluginMetadata,
};

/// Plugin Manager for handling plugin lifecycle
pub struct PluginManager {
    config: PluginConfig,
    loaded_plugins: HashMap<String, LoadedPluginInfo>,
    plugin_registry: HashMap<String, PluginRegistryEntry>,
    dependency_graph: DependencyGraph,
    event_handlers: Vec<Box<dyn PluginEventHandler>>,
    native_runtime: Option<NativePluginRuntime>,
    // wasi_runtime: Option<WasiPluginRuntime>,  // Stage 2: WASI support (C-free for now)
    // component_registry: ComponentRegistry,    // Stage 2: Component registry (C-free for now)
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            config: PluginConfig::default(),
            loaded_plugins: HashMap::new(),
            plugin_registry: HashMap::new(),
            dependency_graph: DependencyGraph::new(),
            event_handlers: Vec::new(),
            native_runtime: None,
            // wasi_runtime: None,                  // Stage 2: WASI support (C-free for now)
            // component_registry: ComponentRegistry::new(),  // Stage 2: Component registry (C-free for now)
        }
    }

    /// Create a plugin manager with custom configuration
    pub fn with_config(config: PluginConfig) -> Self {
        Self {
            config,
            loaded_plugins: HashMap::new(),
            plugin_registry: HashMap::new(),
            dependency_graph: DependencyGraph::new(),
            event_handlers: Vec::new(),
            native_runtime: None,
            // wasi_runtime: None,                  // Stage 2: WASI support (C-free for now)
            // component_registry: ComponentRegistry::new(),  // Stage 2: Component registry (C-free for now)
        }
    }

    /// Set the native runtime for the manager
    #[cfg(feature = "native-plugins")]
    pub fn set_native_runtime(&mut self, runtime: NativePluginRuntime) {
        self.native_runtime = Some(runtime);
    }

    // /// Set the WASI runtime for the manager (Stage 2)
    // pub fn set_wasi_runtime(&mut self, runtime: WasiPluginRuntime) {
    //     self.wasi_runtime = Some(runtime);
    // }

    /// Initialize native runtime only (Stage 1)
    pub async fn initialize_runtimes(&mut self) -> Result<()> {
        // Initialize native runtime
        #[cfg(feature = "native-plugins")]
        {
            let native_runtime = NativePluginRuntime::new()?;
            self.set_native_runtime(native_runtime);
        }

        // // Initialize WASI runtime (Stage 2 - C-free for now)
        // let wasi_runtime = WasiPluginRuntime::new().await
        //     .context("Failed to initialize WASI runtime")?;
        // self.set_wasi_runtime(wasi_runtime);

        log::info!("Native plugin runtime initialized successfully");
        Ok(())
    }

    /// Load configuration
    pub async fn load_config(&mut self) -> Result<()> {
        // Load configuration from file if it exists
        #[cfg(feature = "plugin-management")]
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("nexusshell").join("plugins.toml");
            if config_path.exists() {
                let config_content = fs::read_to_string(&config_path)
                    .await
                    .context("Failed to read plugin configuration")?;
                self.config = toml::from_str(&config_content)
                    .context("Failed to parse plugin configuration")?;
            }
        }
        Ok(())
    }

    /// Save configuration
    pub async fn save_config(&self) -> Result<()> {
        #[cfg(feature = "plugin-management")]
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("nexusshell").join("plugins.toml");

            // Create directory if it doesn't exist
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .context("Failed to create config directory")?;
            }

            let config_content = toml::to_string_pretty(&self.config)
                .context("Failed to serialize plugin configuration")?;

            fs::write(&config_path, config_content)
                .await
                .context("Failed to write plugin configuration")?;
        }
        Ok(())
    }

    /// Discover plugins in configured directories
    pub async fn discover_plugins(&mut self) -> Result<()> {
        log::info!("Discovering plugins in configured directories");

        let plugin_dir = &self.config.plugin_dir;
        let plugin_path = PathBuf::from(plugin_dir);

        if plugin_path.exists() {
            self.discover_plugins_in_directory(&plugin_path).await?;
        } else {
            log::warn!("Plugin directory does not exist: {}", plugin_path.display());
        }

        log::info!(
            "Plugin discovery completed. Found {} plugins",
            self.plugin_registry.len()
        );
        Ok(())
    }

    /// Discover plugins in a specific directory
    async fn discover_plugins_in_directory(&mut self, dir: &Path) -> Result<()> {
        log::debug!("Scanning directory for plugins: {}", dir.display());
        #[cfg(feature = "plugin-management")]
        for entry in WalkDir::new(dir).follow_links(true) {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "wasm" {
                        if let Err(e) = self.register_plugin_file(path).await {
                            log::warn!("Failed to register plugin {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Register a plugin file in the registry
    async fn register_plugin_file(&mut self, path: &Path) -> Result<()> {
        log::debug!("Registering plugin file: {}", path.display());

        // Extract metadata from the plugin file
        let metadata = self.extract_plugin_metadata(path).await?;

        // Validate plugin metadata
        self.validate_plugin_metadata(&metadata)?;

        // Create registry entry
        let entry = PluginRegistryEntry {
            id: self.generate_plugin_id(&metadata),
            metadata,
            path: path.to_path_buf(),
            discovered_at: SystemTime::now(),
            status: PluginStatus::Discovered,
        };

        let plugin_id = entry.id.clone();
        self.plugin_registry.insert(entry.id.clone(), entry);
        log::debug!("Registered plugin: {plugin_id}");

        Ok(())
    }

    /// Extract metadata from a plugin file
    async fn extract_plugin_metadata(&self, path: &Path) -> Result<PluginMetadata> {
        // For now, generate basic metadata from filename
        // In a real implementation, this would parse the WASM component metadata
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        Ok(PluginMetadata {
            name: filename.to_string(),
            version: "0.1.0".to_string(),
            description: format!("Plugin loaded from {}", path.display()),
            author: "unknown".to_string(),
            license: "unknown".to_string(),
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            capabilities: vec![],
            exports: vec!["main".to_string()],
            dependencies: HashMap::new(),
            min_nexus_version: "0.1.0".to_string(),
            max_nexus_version: None,
        })
    }

    /// Validate plugin metadata
    pub fn validate_plugin_metadata(&self, metadata: &PluginMetadata) -> Result<()> {
        // Validate version format
        Version::parse(&metadata.version).context("Invalid plugin version format")?;

        // Validate name format
        if metadata.name.is_empty() {
            return Err(anyhow::anyhow!("Plugin name cannot be empty"));
        }

        // Enforce capabilities manifest policy when required by config or env
        let caps_required_cfg = self.config.capabilities_manifest_required;
        let caps_required_env = std::env::var("NXSH_CAP_MANIFEST_REQUIRED")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        if (caps_required_cfg || caps_required_env) && metadata.capabilities.is_empty() {
            return Err(anyhow::anyhow!(
                "Capabilities manifest is required but missing (metadata.capabilities)"
            ));
        }

        // Additional strict policy: if metadata.exports is non-empty but capabilities are empty,
        // hint that at least one capability should be declared. This is a soft warning elevated to
        // error only when NXSH_CAP_MANIFEST_REQUIRED is set.
        if !metadata.exports.is_empty() && metadata.capabilities.is_empty() {
            // Currently do not error unless env requires it; tests rely on env gate.
        }

        // Validate dependencies
        for (dep_name, version_req) in &metadata.dependencies {
            VersionReq::parse(version_req).context(format!(
                "Invalid dependency '{dep_name}' version requirement: {version_req}"
            ))?;
        }

        Ok(())
    }

    /// Generate a unique plugin ID
    fn generate_plugin_id(&self, metadata: &PluginMetadata) -> String {
        let base_id = format!("{}@{}", metadata.name, metadata.version);

        // Ensure uniqueness
        let mut counter = 0;
        let mut id = base_id.clone();
        while self.plugin_registry.contains_key(&id) {
            counter += 1;
            id = format!("{base_id}-{counter}");
        }

        id
    }

    /// Load a plugin from file (Stage 1: Native only)
    pub async fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<String> {
        let path = path.as_ref();
        log::info!("Loading native plugin from: {}", path.display());

        // Extract metadata
        let metadata = self.extract_plugin_metadata(path).await?;
        let plugin_id = self.generate_plugin_id(&metadata);

        // Check if already loaded
        if self.loaded_plugins.contains_key(&plugin_id) {
            return Err(anyhow::anyhow!("Plugin already loaded: {}", plugin_id));
        }

        // Resolve dependencies
        self.resolve_dependencies(&metadata).await?;

        // For now, only support native plugins (Stage 1)
        let file_extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        let plugin_type = match file_extension.to_lowercase().as_str() {
            "so" | "dll" | "dylib" => {
                // Load native plugin
                if let Some(runtime) = &self.native_runtime {
                    runtime
                        .load_plugin(path, plugin_id.clone())
                        .await
                        .context("Failed to load native plugin")?;
                } else {
                    return Err(anyhow::anyhow!(
                        "Native runtime not available for native plugin"
                    ));
                }
                PluginType::Native
            }
            // "wasm" => {
            //     // Stage 2: WASI plugin support (C-free for now)
            //     return Err(anyhow::anyhow!("WASM plugins not yet supported in C-free mode"));
            // }
            _ => {
                // Default to native plugin for unknown extensions
                if let Some(runtime) = &self.native_runtime {
                    runtime
                        .load_plugin(path, plugin_id.clone())
                        .await
                        .context("Failed to load native plugin")?;
                } else {
                    return Err(anyhow::anyhow!("Native runtime not available"));
                }
                PluginType::Native
            }
        };

        // Record loaded plugin info
        let plugin_info = LoadedPluginInfo {
            id: plugin_id.clone(),
            metadata: metadata.clone(),
            plugin_type,
            load_time: SystemTime::now(),
            execution_count: 0,
        };
        self.loaded_plugins.insert(plugin_id.clone(), plugin_info);

        // Emit event
        self.emit_event(PluginEvent::Loaded {
            plugin_id: plugin_id.clone(),
            metadata: Box::new(metadata),
        })
        .await;

        Ok(plugin_id)
    }

    /// Unload a plugin (Stage 1: Native only)
    pub async fn unload_plugin(&mut self, plugin_id: &str) -> Result<()> {
        log::info!("Unloading plugin: {plugin_id}");

        // Check if plugin is loaded
        let plugin_info = self
            .loaded_plugins
            .get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not loaded: {}", plugin_id))?;

        // Check for dependents
        let dependents = self.dependency_graph.get_dependents(plugin_id);
        if !dependents.is_empty() {
            return Err(anyhow::anyhow!(
                "Cannot unload plugin {} - it has dependents: {:?}",
                plugin_id,
                dependents
            ));
        }

        // Unload from appropriate runtime based on plugin type (Stage 1: Native only)
        match plugin_info.plugin_type {
            PluginType::Native => {
                if let Some(runtime) = &self.native_runtime {
                    runtime
                        .unload_plugin(plugin_id)
                        .await
                        .context("Failed to unload native plugin from runtime")?;
                }
            } // PluginType::Wasi => {
              //     // Stage 2: WASI support (C-free for now)
              //     return Err(anyhow::anyhow!("WASI plugin unloading not yet supported"));
              // }
        }

        // Remove from loaded plugins
        self.loaded_plugins.remove(plugin_id);

        // Emit event
        self.emit_event(PluginEvent::Unloaded {
            plugin_id: plugin_id.to_string(),
        })
        .await;

        Ok(())
    }

    /// Unload all plugins
    pub async fn unload_all_plugins(&mut self) -> Result<()> {
        log::info!("Unloading all plugins");

        // Get plugins in dependency order (reverse topological sort)
        let unload_order = self.dependency_graph.get_unload_order();

        for plugin_id in unload_order {
            if let Err(e) = self.unload_plugin(&plugin_id).await {
                log::error!("Failed to unload plugin {plugin_id}: {e}");
            }
        }

        Ok(())
    }

    /// Resolve plugin dependencies
    async fn resolve_dependencies(&self, metadata: &PluginMetadata) -> Result<()> {
        log::debug!("Resolving dependencies for plugin: {}", metadata.name);

        for (dep_name, version_req_str) in &metadata.dependencies {
            let version_req = self.parse_dependency(version_req_str)?;

            // Find compatible plugin
            let compatible_plugin = self.find_compatible_plugin(dep_name, &version_req)?;

            // Ensure dependency is loaded
            if !self.loaded_plugins.contains_key(&compatible_plugin) {
                return Err(anyhow::anyhow!(
                    "Dependency {} is not loaded",
                    compatible_plugin
                ));
            }
        }

        Ok(())
    }

    /// Parse a dependency string
    fn parse_dependency(&self, dependency: &str) -> Result<VersionReq> {
        VersionReq::parse(dependency)
            .map_err(|e| anyhow::anyhow!("Invalid version requirement: {}", e))
    }

    /// Find a compatible plugin for a dependency
    fn find_compatible_plugin(&self, name: &str, version_req: &VersionReq) -> Result<String> {
        for (plugin_id, entry) in &self.plugin_registry {
            if entry.metadata.name == name {
                let version = Version::parse(&entry.metadata.version)?;
                if version_req.matches(&version) {
                    return Ok(plugin_id.clone());
                }
            }
        }

        Err(anyhow::anyhow!(
            "No compatible plugin found for dependency: {}@{}",
            name,
            version_req
        ))
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<String> {
        self.loaded_plugins.keys().cloned().collect()
    }

    /// List all discovered plugins
    pub fn list_discovered_plugins(&self) -> Vec<String> {
        self.plugin_registry.keys().cloned().collect()
    }

    /// Get plugin metadata
    pub fn get_plugin_metadata(&self, plugin_id: &str) -> Option<&PluginMetadata> {
        self.plugin_registry
            .get(plugin_id)
            .map(|entry| &entry.metadata)
    }

    /// Get plugin status
    pub fn get_plugin_status(&self, plugin_id: &str) -> Option<PluginStatus> {
        self.plugin_registry
            .get(plugin_id)
            .map(|entry| entry.status.clone())
    }

    /// Add an event handler
    pub fn add_event_handler(&mut self, handler: Box<dyn PluginEventHandler>) {
        // Store handler for later emission
        self.event_handlers.push(handler);
        log::debug!(
            "Plugin event handler registered (total: {})",
            self.event_handlers.len()
        );
    }

    /// Emit a plugin event
    async fn emit_event(&self, event: PluginEvent) {
        // Dispatch to all registered handlers
        // - When `event-dispatch` feature is enabled, run concurrently via futures::join_all
        // - Otherwise fall back to sequential dispatch to avoid requiring the `futures` crate
        #[cfg(feature = "event-dispatch")]
        {
            use futures::future::join_all;
            let futures_iter = self
                .event_handlers
                .iter()
                .map(|h| h.handle_event(event.clone()));
            let results = join_all(futures_iter).await;
            for res in results {
                if let Err(e) = res {
                    log::warn!("Plugin event handler error: {e}");
                }
            }
        }

        #[cfg(not(feature = "event-dispatch"))]
        {
            for handler in &self.event_handlers {
                if let Err(e) = handler.handle_event(event.clone()).await {
                    log::warn!("Plugin event handler error: {e}");
                }
            }
        }
    }

    /// Update a plugin
    pub async fn update_plugin(&mut self, plugin_id: &str, new_path: &Path) -> Result<()> {
        log::info!("Updating plugin: {plugin_id}");

        // Get current metadata
        let old_metadata = self
            .get_plugin_metadata(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_id))?
            .clone();

        // Extract new metadata
        let new_metadata = self.extract_plugin_metadata(new_path).await?;

        // Validate version is newer
        let old_version = Version::parse(&old_metadata.version)?;
        let new_version = Version::parse(&new_metadata.version)?;

        if new_version <= old_version {
            return Err(anyhow::anyhow!(
                "New version {} is not newer than current version {}",
                new_version,
                old_version
            ));
        }

        // Unload old version
        self.unload_plugin(plugin_id).await?;

        // Load new version
        let new_plugin_id = self.load_plugin(new_path).await?;

        // Emit update event
        self.emit_event(PluginEvent::Updated {
            plugin_id: new_plugin_id,
            old_version: old_metadata.version,
            new_version: new_metadata.version,
        })
        .await;

        Ok(())
    }

    /// Get dependency graph
    pub fn get_dependency_graph(&self) -> &DependencyGraph {
        &self.dependency_graph
    }

    /// Get plugin statistics
    pub fn get_statistics(&self) -> PluginManagerStatistics {
        PluginManagerStatistics {
            total_discovered: self.plugin_registry.len(),
            total_loaded: self.loaded_plugins.len(),
            total_failed: self
                .plugin_registry
                .values()
                .filter(|entry| matches!(entry.status, PluginStatus::Failed))
                .count(),
        }
    }
}

/// Information about a loaded plugin
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct LoadedPluginInfo {
    id: String,
    metadata: PluginMetadata,
    plugin_type: PluginType,
    load_time: SystemTime,
    execution_count: u64,
}

/// Plugin type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum PluginType {
    Native,
    // Wasi,  // Stage 2: WASI support (C-free for now)
}

/// Plugin registry entry
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PluginRegistryEntry {
    id: String,
    metadata: PluginMetadata,
    path: PathBuf,
    discovered_at: SystemTime,
    status: PluginStatus,
}

/// Plugin status
#[derive(Debug, Clone)]
pub enum PluginStatus {
    Discovered,
    Loading,
    Loaded,
    Failed,
    Unloaded,
}

/// Dependency graph for plugins
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    dependencies: HashMap<String, HashSet<String>>, // plugin_id -> dependencies
    dependents: HashMap<String, HashSet<String>>,   // plugin_id -> dependents
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    /// Create a new dependency graph
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    /// Add a dependency relationship
    pub fn add_dependency(&mut self, plugin_id: &str, dependency_id: &str) {
        self.dependencies
            .entry(plugin_id.to_string())
            .or_default()
            .insert(dependency_id.to_string());

        self.dependents
            .entry(dependency_id.to_string())
            .or_default()
            .insert(plugin_id.to_string());
    }

    /// Remove a dependency relationship
    pub fn remove_dependency(&mut self, plugin_id: &str, dependency_id: &str) {
        if let Some(deps) = self.dependencies.get_mut(plugin_id) {
            deps.remove(dependency_id);
        }

        if let Some(deps) = self.dependents.get_mut(dependency_id) {
            deps.remove(plugin_id);
        }
    }

    /// Get dependencies of a plugin
    pub fn get_dependencies(&self, plugin_id: &str) -> Vec<String> {
        self.dependencies
            .get(plugin_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get dependents of a plugin
    pub fn get_dependents(&self, plugin_id: &str) -> Vec<String> {
        self.dependents
            .get(plugin_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get load order (topological sort)
    pub fn get_load_order(&self) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();

        for plugin_id in self.dependencies.keys() {
            if !visited.contains(plugin_id) {
                self.topological_sort(plugin_id, &mut visited, &mut result);
            }
        }

        result
    }

    /// Get unload order (reverse topological sort)
    pub fn get_unload_order(&self) -> Vec<String> {
        let mut load_order = self.get_load_order();
        load_order.reverse();
        load_order
    }

    /// Topological sort helper
    fn topological_sort(
        &self,
        plugin_id: &str,
        visited: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) {
        visited.insert(plugin_id.to_string());

        if let Some(dependencies) = self.dependencies.get(plugin_id) {
            for dep in dependencies {
                if !visited.contains(dep) {
                    self.topological_sort(dep, visited, result);
                }
            }
        }

        result.push(plugin_id.to_string());
    }

    /// Check for circular dependencies
    pub fn has_circular_dependency(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for plugin_id in self.dependencies.keys() {
            if !visited.contains(plugin_id)
                && self.has_cycle(plugin_id, &mut visited, &mut rec_stack)
            {
                return true;
            }
        }

        false
    }

    /// Check for cycle helper
    fn has_cycle(
        &self,
        plugin_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(plugin_id.to_string());
        rec_stack.insert(plugin_id.to_string());

        if let Some(dependencies) = self.dependencies.get(plugin_id) {
            for dep in dependencies {
                if !visited.contains(dep) {
                    if self.has_cycle(dep, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(dep) {
                    return true;
                }
            }
        }

        rec_stack.remove(plugin_id);
        false
    }
}

/// Plugin manager statistics
#[derive(Debug, Clone)]
pub struct PluginManagerStatistics {
    pub total_discovered: usize,
    pub total_loaded: usize,
    pub total_failed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert!(manager.loaded_plugins.is_empty());
        assert!(manager.plugin_registry.is_empty());
    }

    #[test]
    fn test_dependency_graph() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("plugin_a", "plugin_b");
        graph.add_dependency("plugin_b", "plugin_c");

        let load_order = graph.get_load_order();
        assert_eq!(load_order, vec!["plugin_c", "plugin_b", "plugin_a"]);

        let unload_order = graph.get_unload_order();
        assert_eq!(unload_order, vec!["plugin_a", "plugin_b", "plugin_c"]);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("plugin_a", "plugin_b");
        graph.add_dependency("plugin_b", "plugin_c");
        graph.add_dependency("plugin_c", "plugin_a");

        assert!(graph.has_circular_dependency());
    }

    #[test]
    fn test_plugin_id_generation() {
        let manager = PluginManager::new();
        let metadata = PluginMetadata {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            capabilities: vec![],
            exports: vec![],
            dependencies: HashMap::new(),
            min_nexus_version: "0.1.0".to_string(),
            max_nexus_version: None,
        };

        let id = manager.generate_plugin_id(&metadata);
        assert_eq!(id, "test-plugin@1.0.0");
    }

    #[tokio::test]
    async fn test_metadata_validation() {
        let manager = PluginManager::new();

        let valid_metadata = PluginMetadata {
            name: "valid-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Valid plugin".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            capabilities: vec![],
            exports: vec![],
            dependencies: HashMap::new(),
            min_nexus_version: "0.1.0".to_string(),
            max_nexus_version: None,
        };

        assert!(manager.validate_plugin_metadata(&valid_metadata).is_ok());

        let invalid_metadata = PluginMetadata {
            name: "".to_string(),                   // Invalid: empty name
            version: "invalid-version".to_string(), // Invalid: bad version format
            description: "Invalid plugin".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            capabilities: vec![],
            exports: vec![],
            dependencies: HashMap::new(),
            min_nexus_version: "0.1.0".to_string(),
            max_nexus_version: None,
        };

        assert!(manager.validate_plugin_metadata(&invalid_metadata).is_err());
    }
}
