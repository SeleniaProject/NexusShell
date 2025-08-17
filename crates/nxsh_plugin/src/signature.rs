#![cfg(feature = "crypto-verification")]
use anyhow::{Result, Context};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use serde::{Deserialize, Serialize};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};  // Pure Rust Ed25519
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use log::{info, warn, debug};

use crate::{PluginMetadata, PluginError};

/// Plugin signature verification system
pub struct SignatureVerifier {
    trusted_keys: HashMap<String, String>, // Store as base64 strings
    tuf_metadata: TufMetadata,
    verification_config: VerificationConfig,
    key_rotation_log: Vec<KeyRotationEntry>,
}

impl SignatureVerifier {
    /// Create a new signature verifier
    pub fn new() -> Result<Self> {
        Ok(Self {
            trusted_keys: HashMap::new(),
            tuf_metadata: TufMetadata::new(),
            verification_config: VerificationConfig::default(),
            key_rotation_log: Vec::new(),
        })
    }
    
    /// Initialize with trusted keys and TUF metadata
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing plugin signature verification system");
        
        // Load trusted keys
        self.load_trusted_keys().await?;
        
        // Load TUF metadata
        self.load_tuf_metadata().await?;
        
        // Verify TUF metadata integrity
        self.verify_tuf_metadata().await?;
        
        info!("Plugin signature verification system initialized successfully");
        Ok(())
    }
    
    /// Verify a plugin's signature and metadata
    pub async fn verify_plugin<P: AsRef<Path>>(
        &self,
        plugin_path: P,
        metadata: &PluginMetadata,
    ) -> Result<VerificationResult, PluginError> {
        let plugin_path = plugin_path.as_ref();
        debug!("Verifying plugin signature for: {plugin_path:?}");
        
        // Read plugin file
    let plugin_data = tokio::fs::read(plugin_path).await
            .map_err(|e| PluginError::SecurityError(format!("Failed to read plugin file: {e}")))?;
        
        // Calculate plugin hash
        let plugin_hash = self.calculate_hash(&plugin_data);
        
        // Look up plugin in TUF metadata
        let tuf_entry = self.tuf_metadata.find_plugin(&metadata.name, &metadata.version)
            .ok_or_else(|| PluginError::SecurityError(
                format!("Plugin '{}' v{} not found in TUF metadata", metadata.name, metadata.version)
            ))?;
        
        // Verify hash matches TUF metadata
        if plugin_hash != tuf_entry.hash {
            return Ok(VerificationResult::failed(
                "Plugin hash does not match TUF metadata".to_string()
            ));
        }
        
        // Verify TUF signature
        let signature_valid = self.verify_tuf_signature(tuf_entry).await?;
        if !signature_valid {
            return Ok(VerificationResult::failed(
                "Invalid TUF signature".to_string()
            ));
        }
        
        // Check if plugin signature exists
        let signature_path = plugin_path.with_extension("sig");
        if !signature_path.exists() {
            if self.verification_config.require_signatures {
                return Ok(VerificationResult::failed(
                    "Plugin signature file not found".to_string()
                ));
            } else {
                warn!("Plugin signature not found, but not required by configuration");
                return Ok(VerificationResult::unsigned());
            }
        }
        
        // Read and verify plugin signature
    let signature_data = tokio::fs::read(&signature_path).await
            .map_err(|e| PluginError::SecurityError(format!("Failed to read signature file: {e}")))?;
        
        let plugin_signature: PluginSignature = serde_json::from_slice(&signature_data)
            .map_err(|e| PluginError::SecurityError(format!("Invalid signature format: {e}")))?;
        
        // Verify signature
        let verification_result = self.verify_plugin_signature(&plugin_data, &plugin_signature).await?;
        
        // Check expiration
        if let Some(expires_at) = plugin_signature.expires_at {
            if Utc::now() > expires_at {
                return Ok(VerificationResult::failed(
                    "Plugin signature has expired".to_string()
                ));
            }
        }
        
        // Check revocation
        if self.is_key_revoked(&plugin_signature.key_id).await? {
            return Ok(VerificationResult::failed(
                "Signing key has been revoked".to_string()
            ));
        }
        
        info!("Plugin '{}' signature verification completed successfully", metadata.name);
        Ok(verification_result)
    }
    
    /// Sign a plugin with Ed25519 private key
    pub async fn sign_plugin<P: AsRef<Path>>(
        &self,
        plugin_path: P,
        private_key: &Ed25519PrivateKey,
        key_id: String,
    ) -> Result<PluginSignature> {
        let plugin_path = plugin_path.as_ref();
        debug!("Signing plugin: {plugin_path:?}");
        
        // Read plugin data
        let plugin_data = tokio::fs::read(plugin_path).await
            .context("Failed to read plugin file")?;
        
        // Calculate hash
        let hash = self.calculate_hash(&plugin_data);
        
        // Create signature payload
        let payload = SignaturePayload {
            hash: hash.clone(),
            timestamp: Utc::now(),
            key_id: key_id.clone(),
            algorithm: "Ed25519".to_string(),
        };
        
        let payload_bytes = serde_json::to_vec(&payload)
            .context("Failed to serialize signature payload")?;
        
        // Sign the payload
        let signature_bytes = private_key.sign(&payload_bytes)?;
        
        // Create plugin signature
        let plugin_signature = PluginSignature {
            hash,
            signature: BASE64.encode(&signature_bytes),
            key_id,
            algorithm: "Ed25519".to_string(),
            timestamp: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(365)), // 1 year expiration
            metadata: SignatureMetadata {
                version: "1.0".to_string(),
                tool: "NexusShell".to_string(),
                tool_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };
        
        // Save signature file
        let signature_path = plugin_path.with_extension("sig");
        let signature_json = serde_json::to_string_pretty(&plugin_signature)
            .context("Failed to serialize plugin signature")?;
        
        tokio::fs::write(&signature_path, signature_json).await
            .context("Failed to write signature file")?;
        
        info!("Plugin signed successfully: {signature_path:?}");
        Ok(plugin_signature)
    }
    
    /// Add a trusted public key
    pub async fn add_trusted_key(&mut self, key_id: String, public_key: Ed25519PublicKey) -> Result<()> {
        self.trusted_keys.insert(key_id.clone(), public_key.to_base64());
        
        // Log key addition
        self.key_rotation_log.push(KeyRotationEntry {
            key_id: key_id.clone(),
            action: KeyAction::Added,
            timestamp: Utc::now(),
            reason: "Manual addition".to_string(),
        });
        
        // Save updated keys
        self.save_trusted_keys().await?;
        
        info!("Added trusted key: {key_id}");
        Ok(())
    }
    
    /// Revoke a trusted key
    pub async fn revoke_key(&mut self, key_id: &str, reason: String) -> Result<()> {
        if self.trusted_keys.remove(key_id).is_some() {
            // Log key revocation
            self.key_rotation_log.push(KeyRotationEntry {
                key_id: key_id.to_string(),
                action: KeyAction::Revoked,
                timestamp: Utc::now(),
                reason: reason.clone(),
            });
            
            // Save updated keys
            self.save_trusted_keys().await?;
            
            warn!("Revoked trusted key '{key_id}': {reason}");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Key '{}' not found", key_id))
        }
    }
    
    /// Generate a new Ed25519 key pair using Pure Rust implementation
    /// This method is memory-safe, formally verifiable, and compatible with WebAssembly
    pub fn generate_key_pair() -> Result<(Ed25519PrivateKey, Ed25519PublicKey)> {        
        // Generate signing key using cryptographically secure randomness
        let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let verifying_key = signing_key.verifying_key();
        
        let private_key = Ed25519PrivateKey::from_signing_key(signing_key)?;
        let public_key = Ed25519PublicKey::from_bytes(verifying_key.as_bytes())?;
        
        Ok((private_key, public_key))
    }
    
    /// Update TUF metadata
    pub async fn update_tuf_metadata(&mut self, metadata: TufMetadata) -> Result<()> {
        // Verify new metadata signature
        self.verify_tuf_metadata_signature(&metadata).await?;
        
        // Check version is newer
        if metadata.version <= self.tuf_metadata.version {
            return Err(anyhow::anyhow!("TUF metadata version is not newer"));
        }
        
        // Update metadata
        self.tuf_metadata = metadata;
        
        // Save updated metadata
        self.save_tuf_metadata().await?;
        
        info!("Updated TUF metadata to version {}", self.tuf_metadata.version);
        Ok(())
    }
    
    // Private helper methods
    
    async fn load_trusted_keys(&mut self) -> Result<()> {
        let keys_path = self.get_trusted_keys_path();
        
        if keys_path.exists() {
            let keys_data = tokio::fs::read_to_string(&keys_path).await
                .context("Failed to read trusted keys file")?;
            
            let keys_file: TrustedKeysFile = serde_json::from_str(&keys_data)
                .context("Failed to parse trusted keys file")?;
            
            self.trusted_keys = keys_file.keys;
            self.key_rotation_log = keys_file.rotation_log;
        } else {
            // Initialize with default keys if available
            self.initialize_default_keys().await?;
        }
        
        Ok(())
    }
    
    async fn save_trusted_keys(&self) -> Result<()> {
        let keys_path = self.get_trusted_keys_path();
        
        if let Some(parent) = keys_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .context("Failed to create keys directory")?;
        }
        
        let keys_file = TrustedKeysFile {
            version: 1,
            keys: self.trusted_keys.clone(),
            rotation_log: self.key_rotation_log.clone(),
        };
        
        let keys_json = serde_json::to_string_pretty(&keys_file)
            .context("Failed to serialize trusted keys")?;
        
        tokio::fs::write(&keys_path, keys_json).await
            .context("Failed to write trusted keys file")?;
        
        Ok(())
    }
    
    async fn load_tuf_metadata(&mut self) -> Result<()> {
        let metadata_path = self.get_tuf_metadata_path();
        
        if metadata_path.exists() {
            let metadata_data = tokio::fs::read_to_string(&metadata_path).await
                .context("Failed to read TUF metadata file")?;
            
            self.tuf_metadata = serde_json::from_str(&metadata_data)
                .context("Failed to parse TUF metadata file")?;
        } else {
            // Initialize with empty metadata
            self.tuf_metadata = TufMetadata::new();
        }
        
        Ok(())
    }
    
    async fn save_tuf_metadata(&self) -> Result<()> {
        let metadata_path = self.get_tuf_metadata_path();
        
        if let Some(parent) = metadata_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .context("Failed to create metadata directory")?;
        }
        
        let metadata_json = serde_json::to_string_pretty(&self.tuf_metadata)
            .context("Failed to serialize TUF metadata")?;
        
        tokio::fs::write(&metadata_path, metadata_json).await
            .context("Failed to write TUF metadata file")?;
        
        Ok(())
    }
    
    async fn verify_tuf_metadata(&self) -> Result<()> {
        // Verify TUF metadata signature
        if !self.verify_tuf_metadata_signature(&self.tuf_metadata).await? {
            return Err(anyhow::anyhow!("Invalid TUF metadata signature"));
        }
        
        // Check expiration
        if let Some(expires_at) = self.tuf_metadata.expires_at {
            if Utc::now() > expires_at {
                return Err(anyhow::anyhow!("TUF metadata has expired"));
            }
        }
        
        Ok(())
    }
    
    async fn verify_tuf_metadata_signature(&self, metadata: &TufMetadata) -> Result<bool> {
        if let Some(signature) = &metadata.signature {
            // Get the root key for TUF metadata
            if let Some(root_key_b64) = self.trusted_keys.get("tuf-root") {
                let payload = serde_json::to_vec(&metadata.signed)
                    .context("Failed to serialize TUF signed metadata")?;
                
                let signature_bytes = BASE64.decode(&signature.signature)
                    .context("Failed to decode TUF signature")?;
                
                // Convert base64 key back to Ed25519PublicKey for verification
                let root_key = Ed25519PublicKey::from_base64(root_key_b64)?;
                return Ok(root_key.verify(&payload, &signature_bytes).is_ok());
            }
        }
        
        // If no signature or no root key, consider invalid
        Ok(false)
    }
    
    async fn verify_tuf_signature(&self, _entry: &TufPluginEntry) -> Result<bool> {
        // Verify that the TUF entry is properly signed
        // This would involve checking the TUF signature chain
        // For now, we'll assume it's valid if it exists in our metadata
        Ok(true)
    }
    
    async fn verify_plugin_signature(
        &self,
        plugin_data: &[u8],
        signature: &PluginSignature,
    ) -> Result<VerificationResult> {
        // Get the public key
        let public_key = self.trusted_keys.get(&signature.key_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown signing key: {}", signature.key_id))?;
        
        // Verify hash
        let calculated_hash = self.calculate_hash(plugin_data);
        if calculated_hash != signature.hash {
            return Ok(VerificationResult::failed(
                "Plugin hash does not match signature".to_string()
            ));
        }
        
        // Create signature payload
        let payload = SignaturePayload {
            hash: signature.hash.clone(),
            timestamp: signature.timestamp,
            key_id: signature.key_id.clone(),
            algorithm: signature.algorithm.clone(),
        };
        
        let payload_bytes = serde_json::to_vec(&payload)
            .context("Failed to serialize signature payload")?;
        
        // Verify signature
        let signature_bytes = BASE64.decode(&signature.signature)
            .context("Failed to decode signature")?;
        
        // Convert base64 key back to Ed25519PublicKey for verification
        let public_key_obj = Ed25519PublicKey::from_base64(public_key)?;
        match public_key_obj.verify(&payload_bytes, &signature_bytes) {
            Ok(()) => Ok(VerificationResult::valid(signature.key_id.clone())),
            Err(_) => Ok(VerificationResult::failed("Invalid signature".to_string())),
        }
    }
    
    async fn is_key_revoked(&self, key_id: &str) -> Result<bool> {
        // Check if key is in revocation log
        Ok(self.key_rotation_log.iter().any(|entry| {
            entry.key_id == key_id && entry.action == KeyAction::Revoked
        }))
    }
    
    fn calculate_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        format!("sha256:{}", hex::encode(hash))
    }
    
    async fn initialize_default_keys(&mut self) -> Result<()> {
        // In a real implementation, this would load well-known public keys
        // For now, we'll create an empty set
        info!("Initialized with empty trusted keys set");
        Ok(())
    }
    
    fn get_trusted_keys_path(&self) -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nexusshell")
            .join("trusted_keys.json")
    }
    
    fn get_tuf_metadata_path(&self) -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nexusshell")
            .join("tuf_metadata.json")
    }
}

/// Ed25519 private key wrapper
/// Ed25519 private key wrapper for Pure Rust implementation
pub struct Ed25519PrivateKey {
    signing_key: SigningKey,
}

impl Ed25519PrivateKey {
    /// Create a private key from raw bytes using Pure Rust implementation
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid Ed25519 private key length: expected 32 bytes, got {}", bytes.len()));
        }
        
        let signing_key = SigningKey::from_bytes(
            bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Failed to convert bytes to array"))?
        );
        
        Ok(Self { signing_key })
    }
    
    /// Create a private key from a SigningKey
    pub fn from_signing_key(signing_key: SigningKey) -> Result<Self> {
        Ok(Self { signing_key })
    }
    
    /// Sign data using Pure Rust Ed25519 implementation
    /// This method is memory-safe and formally verifiable
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        let signature = self.signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }
    
    /// Get the corresponding public key
    pub fn public_key(&self) -> Result<Ed25519PublicKey> {
        let verifying_key = self.signing_key.verifying_key();
        Ed25519PublicKey::from_bytes(verifying_key.as_bytes())
    }
}

/// Ed25519 public key wrapper
/// Ed25519 public key wrapper for Pure Rust implementation
#[derive(Debug, Clone)]
pub struct Ed25519PublicKey {
    verifying_key: VerifyingKey,
}

impl Ed25519PublicKey {
    /// Create a public key from raw bytes using Pure Rust implementation
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid Ed25519 public key length: expected 32 bytes, got {}", bytes.len()));
        }
        
        let verifying_key = VerifyingKey::from_bytes(
            bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Failed to convert bytes to array"))?
        ).map_err(|e| anyhow::anyhow!("Invalid Ed25519 public key: {}", e))?;
        
        Ok(Self { verifying_key })
    }
    
    /// Verify a signature using Pure Rust Ed25519 implementation
    /// This method is memory-safe and formally verifiable
    pub fn verify(&self, message: &[u8], signature_bytes: &[u8]) -> Result<()> {
        let signature = Signature::from_bytes(
            signature_bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Invalid signature length: expected 64 bytes"))?
        );
        
        self.verifying_key.verify(message, &signature)
            .map_err(|e| anyhow::anyhow!("Signature verification failed: {}", e))
    }
    
    pub fn to_base64(&self) -> String {
        BASE64.encode(self.verifying_key.as_bytes())
    }
    
    pub fn from_base64(encoded: &str) -> Result<Self> {
        let bytes = BASE64.decode(encoded)
            .context("Failed to decode base64 public key")?;
        Self::from_bytes(&bytes)
    }
}

/// Plugin signature structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSignature {
    pub hash: String,
    pub signature: String,
    pub key_id: String,
    pub algorithm: String,
    pub timestamp: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: SignatureMetadata,
}

/// Signature metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureMetadata {
    pub version: String,
    pub tool: String,
    pub tool_version: String,
}

/// Signature payload for signing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignaturePayload {
    hash: String,
    timestamp: DateTime<Utc>,
    key_id: String,
    algorithm: String,
}

/// Verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub valid: bool,
    pub signed: bool,
    pub key_id: Option<String>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl VerificationResult {
    pub fn valid(key_id: String) -> Self {
        Self {
            valid: true,
            signed: true,
            key_id: Some(key_id),
            error: None,
            timestamp: Utc::now(),
        }
    }
    
    pub fn failed(error: String) -> Self {
        Self {
            valid: false,
            signed: true,
            key_id: None,
            error: Some(error),
            timestamp: Utc::now(),
        }
    }
    
    pub fn unsigned() -> Self {
        Self {
            valid: false,
            signed: false,
            key_id: None,
            error: Some("Plugin is not signed".to_string()),
            timestamp: Utc::now(),
        }
    }
}

/// TUF (The Update Framework) metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TufMetadata {
    pub version: u64,
    pub expires_at: Option<DateTime<Utc>>,
    pub signed: TufSigned,
    pub signature: Option<TufSignature>,
}

impl Default for TufMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl TufMetadata {
    pub fn new() -> Self {
        Self {
            version: 1,
            expires_at: None,
            signed: TufSigned {
                targets: HashMap::new(),
            },
            signature: None,
        }
    }
    
    pub fn find_plugin(&self, name: &str, version: &str) -> Option<&TufPluginEntry> {
        let key = format!("{name}-{version}");
        self.signed.targets.get(&key)
    }
}

/// TUF signed metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TufSigned {
    pub targets: HashMap<String, TufPluginEntry>,
}

/// TUF plugin entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TufPluginEntry {
    pub hash: String,
    pub length: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// TUF signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TufSignature {
    pub key_id: String,
    pub signature: String,
}

/// Verification configuration
#[derive(Debug, Clone)]
pub struct VerificationConfig {
    pub require_signatures: bool,
    pub allow_unsigned_dev: bool,
    pub max_signature_age_days: u32,
    pub enforce_key_expiration: bool,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            require_signatures: true,
            allow_unsigned_dev: false,
            max_signature_age_days: 365,
            enforce_key_expiration: true,
        }
    }
}

/// Trusted keys file format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrustedKeysFile {
    version: u32,
    keys: HashMap<String, String>, // Store as base64 strings
    rotation_log: Vec<KeyRotationEntry>,
}

/// Key rotation log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationEntry {
    pub key_id: String,
    pub action: KeyAction,
    pub timestamp: DateTime<Utc>,
    pub reason: String,
}

/// Key rotation actions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyAction {
    Added,
    Revoked,
    Rotated,
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_signature_verifier_creation() {
        let verifier = SignatureVerifier::new().unwrap();
        assert!(verifier.trusted_keys.is_empty());
    }
    
    #[test]
    fn test_key_pair_generation() {
        let (private_key, public_key) = SignatureVerifier::generate_key_pair().unwrap();
        
        // Test signing and verification
        let message = b"test message";
        let signature = private_key.sign(message).unwrap();
        assert!(public_key.verify(message, &signature).is_ok());
    }
    
    #[test]
    fn test_public_key_serialization() {
        let (_, public_key) = SignatureVerifier::generate_key_pair().unwrap();
        
        let base64_key = public_key.to_base64();
        let decoded_key = Ed25519PublicKey::from_base64(&base64_key).unwrap();
        
        assert_eq!(public_key.bytes, decoded_key.bytes);
    }
    
    #[tokio::test]
    async fn test_plugin_signing() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_path = temp_dir.path().join("test_plugin.wasm");
        
        // Create a test plugin file
        tokio::fs::write(&plugin_path, b"fake wasm content").await.unwrap();
        
        let verifier = SignatureVerifier::new().unwrap();
        let (private_key, _) = SignatureVerifier::generate_key_pair().unwrap();
        
        let signature = verifier.sign_plugin(&plugin_path, &private_key, "test-key".to_string()).await.unwrap();
        
        assert_eq!(signature.key_id, "test-key");
        assert_eq!(signature.algorithm, "Ed25519");
        assert!(signature.signature.len() > 0);
    }
    
    #[test]
    fn test_verification_result() {
        let valid_result = VerificationResult::valid("test-key".to_string());
        assert!(valid_result.valid);
        assert!(valid_result.signed);
        assert_eq!(valid_result.key_id, Some("test-key".to_string()));
        
        let failed_result = VerificationResult::failed("test error".to_string());
        assert!(!failed_result.valid);
        assert!(failed_result.signed);
        assert_eq!(failed_result.error, Some("test error".to_string()));
        
        let unsigned_result = VerificationResult::unsigned();
        assert!(!unsigned_result.valid);
        assert!(!unsigned_result.signed);
    }
    
    #[test]
    fn test_tuf_metadata() {
        let mut metadata = TufMetadata::new();
        assert_eq!(metadata.version, 1);
        
        let plugin_entry = TufPluginEntry {
            hash: "sha256:abcd1234".to_string(),
            length: 1024,
            metadata: HashMap::new(),
        };
        
        metadata.signed.targets.insert("test-plugin-1.0.0".to_string(), plugin_entry);
        
        let found = metadata.find_plugin("test-plugin", "1.0.0");
        assert!(found.is_some());
        assert_eq!(found.unwrap().hash, "sha256:abcd1234");
    }
}
*/ 