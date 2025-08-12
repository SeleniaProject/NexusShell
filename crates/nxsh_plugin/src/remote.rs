//! Remote Plugin Support for NexusShell
//! 
//! This module provides functionality for downloading, managing, and updating
//! plugins from remote repositories. Uses Pure Rust HTTP client (ureq) to maintain
//! zero C dependencies policy.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, create_dir_all};
use std::io::{Write, Read};
use std::path::{Path, PathBuf};
#[cfg(feature = "remote-plugins")]
use ureq;
#[cfg(feature = "remote-plugins")]
use base64::engine::{Engine, general_purpose::STANDARD as BASE64};
use crate::keys::{load_official_pubkey_b64, load_community_pubkey_b64, is_valid_ed25519_pubkey_b64};

/// Remote plugin repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRepository {
    pub name: String,
    pub base_url: String,
    pub public_key: String,
    pub priority: u32,
    pub enabled: bool,
}

/// Remote plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemotePluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub download_url: String,
    pub checksum: String,
    pub signature: Option<String>,
    pub dependencies: Vec<String>,
    pub platforms: Vec<String>,
    pub size: u64,
}

/// Remote plugin manager
pub struct RemotePluginManager {
    repositories: Vec<RemoteRepository>,
    cache_dir: PathBuf,
    user_agent: String,
}

impl RemotePluginManager {
    /// Create a new remote plugin manager
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        create_dir_all(&cache_dir)
            .with_context(|| format!("Failed to create cache directory: {cache_dir:?}"))?;

        Ok(Self {
            repositories: Vec::new(),
            cache_dir,
            user_agent: "NexusShell-Plugin-Manager/0.1.0".to_string(),
        })
    }

    /// Add a repository
    pub fn add_repository(&mut self, repo: RemoteRepository) {
        self.repositories.push(repo);
        // Sort by priority (higher priority first)
        self.repositories.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Download plugin from remote repository
    pub fn download_plugin(&self, plugin_id: &str, dest_path: &Path) -> Result<RemotePluginInfo> {
        // Try each repository until successful
        for repo in &self.repositories {
            if !repo.enabled {
                continue;
            }

            match self.try_download_from_repo(repo, plugin_id, dest_path) {
                Ok(info) => return Ok(info),
                Err(e) => {
                    log::warn!("Failed to download {} from {}: {}", plugin_id, repo.name, e);
                    continue;
                }
            }
        }

        anyhow::bail!("Plugin '{}' not found in any repository", plugin_id)
    }

    /// Try downloading from specific repository
    fn try_download_from_repo(
        &self,
        repo: &RemoteRepository,
        plugin_id: &str,
        dest_path: &Path,
    ) -> Result<RemotePluginInfo> {
        // Get plugin metadata
        let metadata_url = format!("{}/api/v1/plugins/{}/info", repo.base_url, plugin_id);
        let response = ureq::get(&metadata_url)
            .set("User-Agent", &self.user_agent)
            .call()
            .with_context(|| format!("Failed to fetch metadata from {metadata_url}"))?;

        let body = response.into_string().with_context(|| "Failed to read metadata response")?;
        let plugin_info: RemotePluginInfo = serde_json::from_str(&body)
            .with_context(|| "Failed to parse plugin metadata")?;

        // Verify platform compatibility
        if !self.is_platform_compatible(&plugin_info.platforms)? {
            anyhow::bail!("Plugin {} is not compatible with current platform", plugin_id);
        }

        // Download plugin binary
        let download_response = ureq::get(&plugin_info.download_url)
            .set("User-Agent", &self.user_agent)
            .call()
            .with_context(|| format!("Failed to download plugin from {}", plugin_info.download_url))?; // cannot use shorthand inside braces due to borrow later

        // Read all bytes
        let mut bytes = Vec::new();
        download_response.into_reader().read_to_end(&mut bytes)
            .with_context(|| "Failed to read plugin data")?;

        // Verify checksum
        self.verify_checksum(&bytes, &plugin_info.checksum)
            .with_context(|| "Plugin checksum verification failed")?;

        // Verify signature if present
        if let Some(signature) = &plugin_info.signature {
            self.verify_signature(&bytes, signature, &repo.public_key)
                .with_context(|| "Plugin signature verification failed")?;
        }

        // Write to destination
        let mut file = File::create(dest_path)
            .with_context(|| format!("Failed to create destination file: {dest_path:?}"))?;
        file.write_all(&bytes)
            .with_context(|| "Failed to write plugin data")?;

        Ok(plugin_info)
    }

    /// Check if plugin is compatible with current platform
    fn is_platform_compatible(&self, platforms: &[String]) -> Result<bool> {
        let current_platform = format!("{}-{}", 
            std::env::consts::OS, 
            std::env::consts::ARCH
        );

        Ok(platforms.is_empty() || // Empty means all platforms
           platforms.iter().any(|p| p == "all" || p == &current_platform))
    }

    /// Verify plugin checksum
    fn verify_checksum(&self, data: &[u8], expected_checksum: &str) -> Result<()> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(data);
        let computed_hash = hasher.finalize();
        let computed_hex = hex::encode(computed_hash);

        if computed_hex == expected_checksum {
            Ok(())
        } else {
            anyhow::bail!(
                "Checksum mismatch: expected {}, got {}", 
                expected_checksum, 
                computed_hex
            )
        }
    }

    /// Verify plugin signature
    fn verify_signature(&self, data: &[u8], signature: &str, public_key: &str) -> Result<()> {
        use ed25519_dalek::{Signature, VerifyingKey, Verifier};
        
        // Decode public key and signature
        let public_key_bytes = BASE64.decode(public_key)
            .with_context(|| "Invalid base64 public key")?;
        let signature_bytes = BASE64.decode(signature)
            .with_context(|| "Invalid base64 signature")?;
        
        // Create verifying key
        let verifying_key = VerifyingKey::from_bytes(
            public_key_bytes.as_slice().try_into()
                .map_err(|_| anyhow::anyhow!("Invalid public key length"))?
        ).map_err(|e| anyhow::anyhow!("Invalid Ed25519 public key: {}", e))?;
        
        // Create signature
        let sig = Signature::from_bytes(
            signature_bytes.as_slice().try_into()
                .map_err(|_| anyhow::anyhow!("Invalid signature length"))?
        );
        
        // Verify signature
        verifying_key.verify(data, &sig)
            .map_err(|_| anyhow::anyhow!("Signature verification failed"))
    }

    /// List available plugins from all repositories
    pub fn list_available_plugins(&self) -> Result<HashMap<String, Vec<RemotePluginInfo>>> {
        let mut result = HashMap::new();

        for repo in &self.repositories {
            if !repo.enabled {
                continue;
            }

            match self.fetch_repository_catalog(repo) {
                Ok(plugins) => {
                    result.insert(repo.name.clone(), plugins);
                }
                Err(e) => {
                    log::warn!("Failed to fetch catalog from {}: {}", repo.name, e);
                }
            }
        }

        Ok(result)
    }

    /// Fetch catalog from repository
    fn fetch_repository_catalog(&self, repo: &RemoteRepository) -> Result<Vec<RemotePluginInfo>> {
        let catalog_url = format!("{}/api/v1/plugins/catalog", repo.base_url);
        let response = ureq::get(&catalog_url)
            .set("User-Agent", &self.user_agent)
            .call()
            .with_context(|| format!("Failed to fetch catalog from {catalog_url}"))?;

        let body = response.into_string().with_context(|| "Failed to read catalog response")?;
        let plugins: Vec<RemotePluginInfo> = serde_json::from_str(&body)
            .with_context(|| "Failed to parse plugin catalog")?;

        Ok(plugins)
    }

    /// Search for plugins by name or description
    pub fn search_plugins(&self, query: &str) -> Result<Vec<RemotePluginInfo>> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for repo in &self.repositories {
            if !repo.enabled {
                continue;
            }

            match self.fetch_repository_catalog(repo) {
                Ok(plugins) => {
                    for plugin in plugins {
                        if plugin.name.to_lowercase().contains(&query_lower) ||
                           plugin.description.to_lowercase().contains(&query_lower) {
                            results.push(plugin);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to search in repository {}: {}", repo.name, e);
                }
            }
        }

        Ok(results)
    }

    /// Update repository metadata cache
    pub fn update_cache(&self) -> Result<()> {
        for repo in &self.repositories {
            if !repo.enabled {
                continue;
            }

            let cache_file = self.cache_dir.join(format!("{}-catalog.json", repo.name));
            
            match self.fetch_repository_catalog(repo) {
                Ok(plugins) => {
                    let json = serde_json::to_string_pretty(&plugins)?;
                    std::fs::write(&cache_file, json)
                        .with_context(|| format!("Failed to write cache file: {cache_file:?}"))?;
                    log::info!("Updated cache for repository: {}", repo.name);
                }
                Err(e) => {
                    log::error!("Failed to update cache for {}: {}", repo.name, e);
                }
            }
        }
        Ok(())
    }
}

/// Default repository configurations
impl Default for RemotePluginManager {
    fn default() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nxsh")
            .join("plugin-cache");

        let mut manager = Self::new(cache_dir).unwrap();
        
        // Add official NexusShell plugin repository (key loaded via env/file/built-in)
        let official_key = load_official_pubkey_b64();
        manager.add_repository(RemoteRepository {
            name: "official".to_string(),
            base_url: "https://plugins.nexusshell.org".to_string(),
            public_key: official_key,
            priority: 100,
            enabled: true,
        });

        // Add community repository (key loaded via env/file/built-in)
        let community_key = load_community_pubkey_b64();
        manager.add_repository(RemoteRepository {
            name: "community".to_string(),
            base_url: "https://community-plugins.nexusshell.org".to_string(),
            public_key: community_key,
            priority: 50,
            enabled: true,
        });

        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_remote_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = RemotePluginManager::new(temp_dir.path()).unwrap();
        
        assert_eq!(manager.repositories.len(), 0);
        assert!(manager.cache_dir.exists());
    }

    #[test]
    fn test_repository_management() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = RemotePluginManager::new(temp_dir.path()).unwrap();

        let repo = RemoteRepository {
            name: "test-repo".to_string(),
            base_url: "https://example.com".to_string(),
            public_key: "test-key".to_string(),
            priority: 75,
            enabled: true,
        };

        manager.add_repository(repo.clone());
        assert_eq!(manager.repositories.len(), 1);
        assert_eq!(manager.repositories[0].name, "test-repo");
    }

    #[test]
    fn test_platform_compatibility() {
        let temp_dir = TempDir::new().unwrap();
        let manager = RemotePluginManager::new(temp_dir.path()).unwrap();

        // Test universal compatibility
        assert!(manager.is_platform_compatible(&[]).unwrap());
        assert!(manager.is_platform_compatible(&["all".to_string()]).unwrap());

        // Test specific platform
        let current_platform = format!("{}-{}", 
            std::env::consts::OS, 
            std::env::consts::ARCH
        );
        assert!(manager.is_platform_compatible(&[current_platform]).unwrap());
        assert!(!manager.is_platform_compatible(&["nonexistent-platform".to_string()]).unwrap());
    }
}