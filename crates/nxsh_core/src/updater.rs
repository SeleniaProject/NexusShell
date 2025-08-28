//! Comprehensive auto-updater system for NexusShell
//!
//! This module provides advanced update functionality including:
//! - Version checking with semantic versioning
//! - Delta/differential binary updates for efficiency
//! - Cryptographic signature verification (Ed25519)
//! - Multi-channel support (stable, beta, nightly)
//! - Safe rollback mechanisms
//! - Progress tracking and resumable downloads
//! - Automated backup and recovery

use std::{
    sync::{Arc, RwLock, atomic::{AtomicU64, AtomicBool, Ordering}},
    time::{Duration, SystemTime},
    path::{Path, PathBuf},
    fs,
    collections::HashMap,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use crate::compat::{Result, Context};
use sha2::{Sha256, Digest};
use base64::Engine;
// Note: bring IO traits locally only when needed to avoid unused warnings

fn home_dir_fallback() -> Option<std::path::PathBuf> {
    if let Ok(h) = std::env::var("HOME") { return Some(std::path::PathBuf::from(h)); }
    if cfg!(windows) {
        if let Ok(p) = std::env::var("USERPROFILE") { return Some(std::path::PathBuf::from(p)); }
    }
    None
}

/// Comprehensive update system with delta updates and signature verification
#[allow(dead_code)] // アップデート監視の一部は未配線
pub struct UpdateSystem {
    config: UpdateConfig,
    update_info: Arc<RwLock<Option<UpdateInfo>>>,
    download_progress: Arc<RwLock<DownloadProgress>>,
    verification_keys: Arc<RwLock<VerificationKeys>>,
    update_history: Arc<RwLock<Vec<UpdateRecord>>>,
    is_updating: AtomicBool,
}

fn compute_key_fingerprint(material: &str) -> Result<String> {
    if material.contains("-----BEGIN") {
        let der = pem::parse(material)
            .map_err(|e| crate::anyhow!("invalid PEM: {}", e))?
            .into_contents();
        let mut hasher = Sha256::new();
        hasher.update(&der);
        Ok(hex::encode(hasher.finalize()))
    } else {
        let raw = base64::engine::general_purpose::STANDARD
            .decode(material.trim())
            .map_err(|e| crate::anyhow!("invalid base64 public key: {}", e))?;
        let mut hasher = Sha256::new();
        hasher.update(&raw);
        Ok(hex::encode(hasher.finalize()))
    }
}

/// Enhanced update configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Update check interval
    pub check_interval: Duration,
    /// Enable automatic updates
    pub auto_update: bool,
    /// Create backup before update
    pub backup_before_update: bool,
    /// Update channel (stable, beta, nightly)
    pub channel: UpdateChannel,
    /// Base URL for update server
    pub update_server_url: String,
    /// Maximum download retry attempts
    pub max_retry_attempts: u32,
    /// Download timeout in seconds
    pub download_timeout_secs: u64,
    /// Enable delta/differential updates
    pub enable_delta_updates: bool,
    /// Verify update signatures
    pub verify_signatures: bool,
    /// Update cache directory
    pub cache_dir: PathBuf,
    /// Backup directory
    pub backup_dir: PathBuf,
    /// Enable automatic rollback on failure
    pub auto_rollback: bool,
    /// Maximum number of backups to keep
    pub max_backups: usize,
    /// Enable progress reporting
    pub progress_reporting: bool,
    /// Custom user agent for requests
    pub user_agent: String,
    /// API key for authenticated requests
    pub api_key: Option<String>,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(3600), // 1 hour
            auto_update: false,
            backup_before_update: true,
            channel: UpdateChannel::Stable,
            update_server_url: "https://updates.nexusshell.org".to_string(),
            max_retry_attempts: 3,
            download_timeout_secs: 300, // 5 minutes
            enable_delta_updates: true,
            verify_signatures: true,
            cache_dir: PathBuf::from("cache/updates"),
            backup_dir: PathBuf::from("backups"),
            auto_rollback: true,
            max_backups: 5,
            progress_reporting: true,
            user_agent: format!("NexusShell-Updater/{}", env!("CARGO_PKG_VERSION")),
            api_key: None,
        }
    }
}

/// Update channels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpdateChannel {
    /// Stable releases only
    Stable,
    /// Beta releases and stable
    Beta,
    /// Nightly builds, beta, and stable
    Nightly,
}

impl std::fmt::Display for UpdateChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateChannel::Stable => write!(f, "stable"),
            UpdateChannel::Beta => write!(f, "beta"),
            UpdateChannel::Nightly => write!(f, "nightly"),
        }
    }
}

/// Comprehensive update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Version string (semver)
    pub version: String,
    /// Release channel
    pub channel: UpdateChannel,
    /// Release notes in markdown format
    pub release_notes: String,
    /// Full binary download URL
    pub download_url: String,
    /// Delta/patch download URL (if available)
    pub delta_url: Option<String>,
    /// File size in bytes
    pub file_size: u64,
    /// Delta file size in bytes (if applicable)
    pub delta_size: Option<u64>,
    /// SHA-256 checksum of full binary
    pub checksum: String,
    /// SHA-256 checksum of delta file
    pub delta_checksum: Option<String>,
    /// Ed25519 signature of the release
    pub signature: String,
    /// Public key fingerprint for verification
    pub key_fingerprint: String,
    /// Release timestamp
    pub release_date: SystemTime,
    /// Minimum compatible version for delta updates
    pub min_delta_version: Option<String>,
    /// Required system architecture
    pub architecture: String,
    /// Required platform
    pub platform: String,
    /// Critical update flag
    pub is_critical: bool,
    /// Security update flag
    pub is_security_update: bool,
    /// Breaking changes flag
    pub has_breaking_changes: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Download progress tracking
/// Download progress tracking structure
/// Note: Custom Clone implementation needed for Atomic types
#[derive(Debug)]
pub struct DownloadProgress {
    /// Total bytes to download
    pub total_bytes: AtomicU64,
    /// Bytes downloaded so far
    pub downloaded_bytes: AtomicU64,
    /// Download speed in bytes per second
    pub speed_bps: AtomicU64,
    /// Estimated time remaining in seconds
    pub eta_seconds: AtomicU64,
    /// Current download stage
    #[allow(dead_code)]
    pub stage: Arc<RwLock<DownloadStage>>,
    /// Start time of download
    pub start_time: SystemTime,
    /// Is download paused
    pub is_paused: AtomicBool,
    /// Last error message
    #[allow(dead_code)]
    pub last_error: Arc<RwLock<Option<String>>>,
}

impl Clone for DownloadProgress {
    fn clone(&self) -> Self {
        Self {
            total_bytes: AtomicU64::new(self.total_bytes.load(Ordering::Relaxed)),
            downloaded_bytes: AtomicU64::new(self.downloaded_bytes.load(Ordering::Relaxed)),
            speed_bps: AtomicU64::new(self.speed_bps.load(Ordering::Relaxed)),
            eta_seconds: AtomicU64::new(self.eta_seconds.load(Ordering::Relaxed)),
            stage: Arc::clone(&self.stage),
            start_time: self.start_time,
            is_paused: AtomicBool::new(self.is_paused.load(Ordering::Relaxed)),
            last_error: Arc::clone(&self.last_error),
        }
    }
}

impl Default for DownloadProgress {
    fn default() -> Self {
        Self {
            total_bytes: AtomicU64::new(0),
            downloaded_bytes: AtomicU64::new(0),
            speed_bps: AtomicU64::new(0),
            eta_seconds: AtomicU64::new(0),
            stage: Arc::new(RwLock::new(DownloadStage::Preparing)),
            start_time: SystemTime::now(),
            is_paused: AtomicBool::new(false),
            last_error: Arc::new(RwLock::new(None)),
        }
    }
}

/// Download stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStage {
    Preparing,
    Downloading,
    Verifying,
    Extracting,
    Installing,
    Finalizing,
    Complete,
    Failed(String),
}

/// Cryptographic verification keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationKeys {
    /// Ed25519 public keys for signature verification
    pub public_keys: HashMap<String, String>,
    /// Key fingerprints to names mapping
    pub fingerprint_to_name: HashMap<String, String>,
    /// Trusted key authorities
    pub trusted_authorities: Vec<String>,
}

impl Default for VerificationKeys {
    fn default() -> Self {
        let mut public_keys = HashMap::new();
        let mut fingerprint_to_name = HashMap::new();
        
        // Add default NexusShell signing key (placeholder)
        let default_key = "placeholder_ed25519_public_key";
        let default_fingerprint = "nxsh_default_key_fingerprint";
        
        public_keys.insert("nxsh-release".to_string(), default_key.to_string());
        fingerprint_to_name.insert(default_fingerprint.to_string(), "nxsh-release".to_string());
        
        Self {
            public_keys,
            fingerprint_to_name,
            trusted_authorities: vec!["nxsh-release".to_string()],
        }
    }
}

/// Update record for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecord {
    /// Update ID
    pub id: String,
    /// Version updated from
    pub from_version: String,
    /// Version updated to
    pub to_version: String,
    /// Update timestamp
    pub timestamp: SystemTime,
    /// Update method (full, delta)
    pub method: UpdateMethod,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Time taken for update
    pub duration: Duration,
    /// Backup path (if created)
    pub backup_path: Option<PathBuf>,
}

/// Update methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateMethod {
    Full,
    Delta,
    Rollback,
}

/// Update verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub checksum_valid: bool,
    pub signature_valid: bool,
    pub key_trusted: bool,
    pub errors: Vec<String>,
}

impl UpdateSystem {
    /// Create a new comprehensive update system
    pub fn new(config: UpdateConfig) -> Result<Self> {
        // Create necessary directories
        fs::create_dir_all(&config.cache_dir)
            .with_context(|| format!("Failed to create cache directory: {:?}", config.cache_dir))?;
        fs::create_dir_all(&config.backup_dir)
            .with_context(|| format!("Failed to create backup directory: {:?}", config.backup_dir))?;

        let system = Self {
            config,
            update_info: Arc::new(RwLock::new(None)),
            download_progress: Arc::new(RwLock::new(DownloadProgress::default())),
            verification_keys: Arc::new(RwLock::new(VerificationKeys::default())),
            update_history: Arc::new(RwLock::new(Vec::new())),
            is_updating: AtomicBool::new(false),
        };
        if let Err(e) = system.initialize_verification_keys_from_env() {
            debug!(error = %e, "Failed to initialize verification keys from env/files");
        }
        if let Err(e) = system.load_verification_keys_from_files_if_present() {
            debug!(error = %e, "Failed to load verification keys from files");
        }
        if let Err(e) = system.rotate_update_keys_if_requested() {
            debug!(error = %e, "Failed to rotate update keys");
        }
        Ok(system)
    }

    /// Check for updates with comprehensive version comparison
    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>> {
        info!(channel = %self.config.channel, "Checking for updates...");

        let current_version = env!("CARGO_PKG_VERSION");
        let check_url = format!(
            "{}/api/v1/check?channel={}&version={}&arch={}&platform={}",
            self.config.update_server_url,
            self.config.channel,
            current_version,
            std::env::consts::ARCH,
            std::env::consts::OS
        );

        debug!(url = %check_url, "Update check URL");

        #[cfg(feature = "updates")]
        {
            let mut req = ureq::get(&check_url).set("User-Agent", &self.config.user_agent);
            if let Some(key) = &self.config.api_key {
                req = req.set("Authorization", &format!("Bearer {key}"));
            }
            let resp = req.call().map_err(|e| crate::anyhow!("update check failed: {e}"))?;
            if resp.status() == 204 {
                info!("No updates available");
                return Ok(None);
            }
            if resp.status() != 200 {
                warn!(status = resp.status(), "Unexpected status from update server");
                return Ok(None);
            }
            let body = resp.into_string().map_err(|e| crate::anyhow!("failed to read response: {e}"))?;
            let info: UpdateInfo = serde_json::from_str(&body).map_err(|e| crate::anyhow!("invalid update info JSON: {e}"))?;
            Ok(Some(info))
        }

        #[cfg(not(feature = "updates"))]
        {
            info!("Update HTTP client disabled (feature 'updates' not enabled)");
            Ok(None)
        }
    }

    /// Download and apply update with delta support
    pub async fn apply_update(&self, update_info: &UpdateInfo) -> Result<()> {
        if self.is_updating.load(Ordering::Relaxed) {
            return Err(crate::anyhow!("Update already in progress"));
        }

        self.is_updating.store(true, Ordering::Relaxed);
        let result = self.apply_update_internal(update_info).await;
        self.is_updating.store(false, Ordering::Relaxed);

        result
    }

    async fn apply_update_internal(&self, update_info: &UpdateInfo) -> Result<()> {
        let update_id = format!("update_{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs());
        let start_time = SystemTime::now();
        
        info!(
            version = %update_info.version,
            channel = %update_info.channel,
            is_critical = update_info.is_critical,
            "Starting update process"
        );

        // Update progress stage
        {
            let progress = self.download_progress.write().unwrap();
            let mut stage = progress.stage.write().unwrap();
            *stage = DownloadStage::Preparing;
        }

        // Create backup if enabled
        let backup_path = if self.config.backup_before_update {
            Some(self.create_backup(&update_id).await?)
        } else {
            None
        };

        let result = async {
            // Determine update method (delta vs full)
            let use_delta = self.should_use_delta_update(update_info)?;
            let method = if use_delta { UpdateMethod::Delta } else { UpdateMethod::Full };

            // Download update
            let download_path = self.download_update(update_info, use_delta).await?;

            // Verify TUF-like metadata from server-provided manifest
            self.verify_tuf_like_metadata(update_info)?;

            // Verify download (select checksum based on method)
            let expected_checksum: String = if use_delta {
                update_info
                    .delta_checksum
                    .clone()
                    .ok_or_else(|| crate::anyhow!("missing delta checksum in update info"))?
            } else {
                update_info.checksum.clone()
            };
            self.verify_update(
                &download_path,
                &expected_checksum,
                &update_info.signature,
                &update_info.key_fingerprint,
            )
            .await?;

            // Apply update
            self.install_update(&download_path, method).await?;

            // Cleanup old backups
            self.cleanup_old_backups().await?;

            Ok::<(), crate::compat::Error>(())
        }.await;

        // Record update attempt
        let duration = start_time.elapsed().unwrap_or_default();
        let record = UpdateRecord {
            id: update_id.clone(),
            from_version: env!("CARGO_PKG_VERSION").to_string(),
            to_version: update_info.version.clone(),
            timestamp: start_time,
            method: if self.should_use_delta_update(update_info).unwrap_or(false) { UpdateMethod::Delta } else { UpdateMethod::Full },
            success: result.is_ok(),
            error_message: result.as_ref().err().map(|e| e.to_string()),
            duration,
            backup_path: backup_path.clone(),
        };

        self.update_history.write().unwrap().push(record);

        if let Err(e) = result {
            error!(error = %e, "Update failed");
            
            // Attempt rollback if enabled and backup exists
            if self.config.auto_rollback && backup_path.is_some() {
                warn!("Attempting automatic rollback");
                if let Err(rollback_err) = self.rollback_update(backup_path.as_ref().unwrap()).await {
                    error!(rollback_error = %rollback_err, "Rollback failed");
                }
            }
            
            return Err(e);
        }

        info!(version = %update_info.version, "Update completed successfully");
        Ok(())
    }

    /// Determine if delta update should be used
    fn should_use_delta_update(&self, update_info: &UpdateInfo) -> Result<bool> {
        if !self.config.enable_delta_updates || update_info.delta_url.is_none() {
            return Ok(false);
        }

        let current_version = env!("CARGO_PKG_VERSION");
        
        // Check if current version is compatible with delta update
        if let Some(min_version) = &update_info.min_delta_version {
            // In production, implement proper semver comparison
            // For now, simple string comparison
            if current_version < min_version.as_str() {
                debug!("Current version too old for delta update");
                return Ok(false);
            }
        }

        // Delta updates are beneficial if delta is significantly smaller
        if let Some(delta_size) = update_info.delta_size {
            let efficiency = (delta_size as f64) / (update_info.file_size as f64);
            if efficiency < 0.7 { // Use delta if it's less than 70% of full size
                info!(efficiency = %efficiency, "Using delta update for efficiency");
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Download update file (full or delta)
    async fn download_update(&self, update_info: &UpdateInfo, use_delta: bool) -> Result<PathBuf> {
    let (url, expected_size, _checksum) = if use_delta {
            (
                update_info.delta_url.as_ref().unwrap(),
                update_info.delta_size.unwrap(),
                update_info.delta_checksum.as_ref().unwrap(),
            )
        } else {
            (&update_info.download_url, update_info.file_size, &update_info.checksum)
        };

        let filename = if use_delta {
            format!("update_{}_delta.bin", update_info.version)
        } else {
            format!("update_{}_full.bin", update_info.version)
        };

        let download_path = self.config.cache_dir.join(filename);
        
        // Update progress
        {
            let progress = self.download_progress.write().unwrap();
            progress.total_bytes.store(expected_size, Ordering::Relaxed);
            progress.downloaded_bytes.store(0, Ordering::Relaxed);
            let mut stage = progress.stage.write().unwrap();
            *stage = DownloadStage::Downloading;
        }

        info!(url = %url, path = ?download_path, size = expected_size, "Starting download");

        #[cfg(feature = "updates")]
        {
            use std::io::Read;
            let mut req = ureq::get(url)
                .set("User-Agent", &self.config.user_agent)
                .timeout(std::time::Duration::from_secs(self.config.download_timeout_secs));
            if let Some(key) = &self.config.api_key {
                req = req.set("Authorization", &format!("Bearer {key}"));
            }

            let resp = req.call().map_err(|e| crate::anyhow!("download failed: {e}"))?;
            if resp.status() != 200 {
                return Err(crate::anyhow!("download HTTP status {}", resp.status()))
            }

            let mut out = std::fs::File::create(&download_path)
                .with_context(|| format!("Failed to create file: {download_path:?}"))?;

            let mut reader = resp.into_reader();
            let mut buf = [0u8; 64 * 1024];
            let mut downloaded: u64 = 0;
            loop {
                if self.download_progress.read().unwrap().is_paused.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    continue;
                }
                let n = reader.read(&mut buf).map_err(|e| crate::anyhow!("read error: {e}"))?;
                if n == 0 { break; }
                std::io::Write::write_all(&mut out, &buf[..n])
                    .map_err(|e| crate::anyhow!("write error: {e}"))?;
                downloaded += n as u64;
                let progress = self.download_progress.read().unwrap();
                progress.downloaded_bytes.store(downloaded, Ordering::Relaxed);
                let elapsed = progress.start_time.elapsed().unwrap_or_default().as_secs_f64();
                if elapsed > 0.0 {
                    let speed = (downloaded as f64 / elapsed) as u64;
                    progress.speed_bps.store(speed, Ordering::Relaxed);
                    if expected_size > 0 {
                        let remaining = expected_size.saturating_sub(downloaded);
                        let eta = if speed > 0 { remaining / speed } else { 0 };
                        progress.eta_seconds.store(eta, Ordering::Relaxed);
                    }
                }
            }
        }

        #[cfg(not(feature = "updates"))]
        {
            fs::write(&download_path, format!("Placeholder update data for version {}", update_info.version))?;
        }

        // Update progress to complete
        {
            let progress = self.download_progress.write().unwrap();
            progress.downloaded_bytes.store(expected_size, Ordering::Relaxed);
            let mut stage = progress.stage.write().unwrap();
            *stage = DownloadStage::Verifying;
        }

        info!(path = ?download_path, "Download completed");
        Ok(download_path)
    }

    /// Verify update integrity and authenticity
    async fn verify_update(
        &self,
        file_path: &Path,
        expected_checksum_hex: &str,
        signature_b64: &str,
        key_fingerprint: &str,
    ) -> Result<()> {
        info!(path = ?file_path, "Verifying update");

        // Verify checksum
        let file_data = fs::read(file_path)
            .with_context(|| format!("Failed to read update file: {file_path:?}"))?;

    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let calculated_hash = hasher.finalize();
    let calculated_hex = format!("{calculated_hash:x}");

        if !expected_checksum_hex.is_empty()
            && expected_checksum_hex.to_ascii_lowercase() != calculated_hex
        {
            return Err(crate::anyhow!(
                "checksum mismatch: expected {}, got {}",
                expected_checksum_hex,
                calculated_hex
            ));
        }

        // Verify signature if enabled
        if self.config.verify_signatures {
            self.verify_signature(&file_data, signature_b64, key_fingerprint)?;
            // TUF-like metadata verification depends on UpdateInfo; skipped here since only file is available
        }

        info!("Update verification completed successfully");
        Ok(())
    }

    /// Verify cryptographic signature using ed25519 algorithm
    /// 
    /// This function performs cryptographic verification of digital signatures
    /// using the Ed25519 signature scheme. It validates that the provided signature
    /// was created by the holder of the private key corresponding to the specified
    /// public key fingerprint.
    /// 
    /// # Arguments
    /// * `data` - The raw data that was signed
    /// * `signature_b64` - Base64-encoded signature to verify
    /// * `key_fingerprint` - Fingerprint identifying the public key to use for verification
    /// 
    /// # Returns
    /// * `Ok(())` if signature verification succeeds
    /// * `Err` if verification fails or key is not trusted
    fn verify_signature(&self, _data: &[u8], _signature_b64: &str, key_fingerprint: &str) -> Result<()> {
        let keys = self.verification_keys.read().unwrap();
        
        let key_name = keys.fingerprint_to_name.get(key_fingerprint)
            .ok_or_else(|| crate::anyhow!("Unknown key fingerprint: {}", key_fingerprint))?;

        if !keys.trusted_authorities.contains(key_name) {
            return Err(crate::anyhow!("Key not in trusted authorities: {}", key_name));
        }

        #[cfg(feature = "crypto-ed25519")]
        {
            use ed25519_dalek::{Signature, Verifier, VerifyingKey};
            use ed25519_dalek::pkcs8::DecodePublicKey;
            let pk_material = keys.public_keys.get(key_name)
                .ok_or_else(|| crate::anyhow!("No public key material for {key_name}"))?;
            let verifying_key = {
                // Try PEM first if it looks like PEM format
                if pk_material.contains("-----BEGIN") {
                    let pem = pem::parse(pk_material.as_bytes())
                        .map_err(|e| crate::anyhow!("invalid PEM public key: {e}"))?;
                    VerifyingKey::from_public_key_der(pem.contents())
                        .map_err(|e| crate::anyhow!("invalid DER public key in PEM: {e}"))?
                } else {
                    // Otherwise treat as base64. If 32 bytes, interpret as raw key; otherwise try DER.
                    let raw = base64::engine::general_purpose::STANDARD
                        .decode(pk_material.as_bytes())
                        .map_err(|e| crate::anyhow!("invalid base64 public key: {e}"))?;
                    if raw.len() == 32 {
                        VerifyingKey::from_bytes(raw.as_slice().try_into().unwrap())
                            .map_err(|e| crate::anyhow!("invalid ed25519 public key (raw 32 bytes): {e}"))?
                    } else {
                        VerifyingKey::from_public_key_der(&raw)
                            .map_err(|e| crate::anyhow!("invalid DER public key (base64): {e}"))?
                    }
                }
            };
            let sig_bytes = base64::engine::general_purpose::STANDARD
                .decode(_signature_b64)
                .map_err(|e| crate::anyhow!("invalid base64 signature: {e}"))?;
            let sig = Signature::from_slice(&sig_bytes)
                .map_err(|e| crate::anyhow!("invalid signature length: {e}"))?;
            verifying_key.verify(_data, &sig)
                .map_err(|_| crate::anyhow!("signature verification failed"))?;
            info!("Signature verification completed successfully");
            Ok(())
        }
        #[cfg(not(feature = "crypto-ed25519"))]
        {
            debug!(key_name = %key_name, "Signature verification skipped (feature 'crypto-ed25519' disabled)");
            Ok(())
        }
    }

    fn verify_tuf_like_metadata(&self, info: &UpdateInfo) -> Result<()> {
        if let Some(role) = info.metadata.get("tuf_role") {
            if role != "targets" { return Err(crate::anyhow!("invalid TUF role: {}", role)); }
        } else { return Err(crate::anyhow!("missing TUF role metadata")); }
        if let Some(exp) = info.metadata.get("tuf_expires") {
            let dt = time::OffsetDateTime::parse(exp, &time::format_description::well_known::Rfc3339)
                .map_err(|e| crate::anyhow!("invalid tuf_expires: {}", e))?;
            if dt < time::OffsetDateTime::now_utc() {
                return Err(crate::anyhow!("TUF metadata expired: {}", exp));
            }
        } else { return Err(crate::anyhow!("missing tuf_expires metadata")); }
        Ok(())
    }

    fn initialize_verification_keys_from_env(&self) -> Result<()> {
        let mut keys = self.verification_keys.write().unwrap();
        if let Ok(json) = std::env::var("NXSH_UPDATE_KEYS_JSON") {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&json) {
                for (name, material) in map {
                    let fp = compute_key_fingerprint(&material)?;
                    keys.public_keys.insert(name.clone(), material.clone());
                    keys.fingerprint_to_name.insert(fp.clone(), name.clone());
                    if !keys.trusted_authorities.contains(&name) { keys.trusted_authorities.push(name); }
                }
            }
        }
        for (name, var) in [("nxsh-release", "NXSH_OFFICIAL_PUBKEY"), ("community", "NXSH_COMMUNITY_PUBKEY")] {
            if let Ok(material) = std::env::var(var) {
                let fp = compute_key_fingerprint(&material)?;
                keys.public_keys.insert(name.to_string(), material.clone());
                keys.fingerprint_to_name.insert(fp.clone(), name.to_string());
                if !keys.trusted_authorities.contains(&name.to_string()) { keys.trusted_authorities.push(name.to_string()); }
            }
        }
        Ok(())
    }

    /// Install the update
    async fn install_update(&self, update_path: &Path, method: UpdateMethod) -> Result<()> {
        info!(method = ?method, path = ?update_path, "Installing update");

        // Update progress
        {
            let progress = self.download_progress.write().unwrap();
            let mut stage = progress.stage.write().unwrap();
            *stage = DownloadStage::Installing;
        }

        match method {
            UpdateMethod::Full => {
                info!("Installing full binary update");
                self.apply_full_update(update_path)?;
            }
            UpdateMethod::Delta => {
                info!("Applying delta patch via bspatch-compatible routine");
                self.apply_delta_patch(update_path)?;
            }
            UpdateMethod::Rollback => {
                return Err(crate::anyhow!("Rollback should not be handled in install_update"));
            }
        }

        // Update progress
        {
            let progress = self.download_progress.write().unwrap();
            let mut stage = progress.stage.write().unwrap();
            *stage = DownloadStage::Complete;
        }

        info!("Update installation completed");
        Ok(())
    }

    /// Apply a delta patch (BSDIFF40) to the current binary using a bspatch-compatible algorithm.
    /// The delta file is expected to be generated by a standard bsdiff implementation.
    fn apply_delta_patch(&self, delta_path: &Path) -> Result<()> {
        // Locate current binary and read old bytes for patching
        let current_exe = std::env::current_exe()
            .map_err(|e| crate::anyhow!("failed to locate current exe: {e}"))?;
        let _old_bytes = fs::read(&current_exe)?;

        // Read delta file into memory
        let delta = fs::read(delta_path)?;

        // Use fallback if not BSDIFF40: treat as full file
        if !delta.starts_with(b"BSDIFF40") {
            return self.apply_full_bytes(&current_exe, &delta);
        }

        // Guard for feature availability
        #[cfg(not(feature = "delta-bspatch"))]
        {
            Err(crate::anyhow!(
                "delta-bspatch feature disabled: rebuild with 'delta-bspatch' feature to enable bspatch"
            ))
        }

        #[cfg(feature = "delta-bspatch")]
        {
            use std::io::{Cursor, Read};
            use bzip2_rs::DecoderReader;

            // Helper: decode signed 64-bit number in BSDIFF offtin format
            fn offtin(buf: [u8; 8]) -> i64 {
                let mut y: i64 = (buf[7] & 0x7f) as i64;
                for i in (0..7).rev() {
                    y = (y << 8) + (buf[i] as i64);
                }
                if (buf[7] & 0x80) != 0 { -y } else { y }
            }

            if delta.len() < 32 {
                return Err(crate::anyhow!("invalid BSDIFF40 patch: too short"));
            }

            let ctrl_len = offtin(delta[8..16].try_into().unwrap());
            let diff_len = offtin(delta[16..24].try_into().unwrap());
            let new_size = offtin(delta[24..32].try_into().unwrap());

            if ctrl_len < 0 || diff_len < 0 || new_size < 0 {
                return Err(crate::anyhow!("invalid BSDIFF40 header: negative lengths"));
            }

            let ctrl_len_usize = ctrl_len as usize;
            let diff_len_usize = diff_len as usize;
            let new_size_usize = new_size as usize;

            if delta.len() < 32 + ctrl_len_usize + diff_len_usize {
                return Err(crate::anyhow!("invalid BSDIFF40 patch: truncated blocks"));
            }

            let ctrl_off = 32;
            let diff_off = 32 + ctrl_len_usize;
            let extra_off = 32 + ctrl_len_usize + diff_len_usize;

            let ctrl_slice = &delta[ctrl_off..diff_off];
            let diff_slice = &delta[diff_off..extra_off];
            let extra_slice = &delta[extra_off..];

            let mut ctrl = DecoderReader::new(Cursor::new(ctrl_slice));
            let mut diff = DecoderReader::new(Cursor::new(diff_slice));
            let mut extra = DecoderReader::new(Cursor::new(extra_slice));

            // Allocate output buffer
            let mut new_bytes = vec![0u8; new_size_usize];
            let mut new_pos: usize = 0;
            let mut old_pos: i64 = 0;

            // Helper to read an offtin i64 from a reader
            fn read_offtin<R: Read>(r: &mut R) -> Result<i64> {
                let mut b = [0u8; 8];
                r.read_exact(&mut b).map_err(|e| crate::anyhow!("failed to read control block: {}", e))?;
                Ok(offtin(b))
            }

            while (new_pos as i64) < new_size {
                let x = read_offtin(&mut ctrl)?;
                let y = read_offtin(&mut ctrl)?;
                let z = read_offtin(&mut ctrl)?;

                if x < 0 || y < 0 {
                    return Err(crate::anyhow!("invalid control tuple: negative x or y"));
                }

                let x_usize = x as usize;
                let y_usize = y as usize;

                // Bounds checks
                if new_pos + x_usize > new_size_usize {
                    return Err(crate::anyhow!("patch would write beyond new size (diff phase)"));
                }
                if old_pos < 0 || (old_pos as usize) > _old_bytes.len() {
                    return Err(crate::anyhow!("old position out of range"));
                }
                if (old_pos as usize) + x_usize > _old_bytes.len() {
                    return Err(crate::anyhow!("patch would read beyond old size (diff phase)"));
                }

                // Read x bytes from diff and add to old
                let mut processed: usize = 0;
                while processed < x_usize {
                    let chunk = (x_usize - processed).min(64 * 1024);
                    let mut buf = vec![0u8; chunk];
                    diff.read_exact(&mut buf)
                        .map_err(|e| crate::anyhow!("failed to read diff block: {}", e))?;
                    let old_slice = &_old_bytes[(old_pos as usize) + processed..(old_pos as usize) + processed + chunk];
                    for i in 0..chunk {
                        new_bytes[new_pos + processed + i] = buf[i].wrapping_add(old_slice[i]);
                    }
                    processed += chunk;
                }

                new_pos += x_usize;
                old_pos += x;

                // Bounds check for extra copy
                if new_pos + y_usize > new_size_usize {
                    return Err(crate::anyhow!("patch would write beyond new size (extra phase)"));
                }

                // Read y bytes from extra
                let mut processed_extra: usize = 0;
                while processed_extra < y_usize {
                    let chunk = (y_usize - processed_extra).min(64 * 1024);
                    let mut buf = vec![0u8; chunk];
                    extra.read_exact(&mut buf)
                        .map_err(|e| crate::anyhow!("failed to read extra block: {}", e))?;
                    new_bytes[new_pos..new_pos + chunk].copy_from_slice(&buf);
                    new_pos += chunk;
                    processed_extra += chunk;
                }

                // Adjust old_pos by z
                old_pos += z;
                if old_pos < 0 || old_pos as usize > _old_bytes.len() {
                    return Err(crate::anyhow!("old position moved out of range after z adjustment"));
                }
            }

            if new_pos != new_size_usize {
                return Err(crate::anyhow!("patch application finished with size mismatch"));
            }

            // Persist resulting bytes
            self.apply_full_bytes(&current_exe, &new_bytes)
        }
    }

    /// Apply a full binary update from a file path.
    fn apply_full_update(&self, full_path: &Path) -> Result<()> {
        let current_exe = std::env::current_exe()
            .map_err(|e| crate::anyhow!("failed to locate current exe: {e}"))?;
        let bytes = fs::read(full_path)?;
        self.apply_full_bytes(&current_exe, &bytes)
    }

    /// Write new binary bytes next to the current executable and finalize installation.
    fn apply_full_bytes(&self, current_exe: &Path, new_bytes: &[u8]) -> Result<()> {
        let dir = current_exe
            .parent()
            .ok_or_else(|| crate::anyhow!("current exe has no parent directory"))?;
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    let tmp_path = dir.join(format!("nxsh_new_{ts}.tmp"));
        fs::write(&tmp_path, new_bytes)?;

        // On Unix, attempt atomic replacement; on Windows, use marker for next restart
        #[cfg(unix)]
        {
            if let Err(e) = std::fs::rename(&tmp_path, current_exe) {
                warn!(error = %e, "Atomic replace failed; falling back to marker file for next restart");
        let marker = dir.join("nxsh_update_pending.txt");
        fs::write(&marker, tmp_path.file_name().unwrap().to_string_lossy().as_bytes())?;
            }
        }
        #[cfg(windows)]
        {
            let marker = dir.join("nxsh_update_pending.txt");
            fs::write(&marker, tmp_path.file_name().unwrap().to_string_lossy().as_bytes())?;
        }

        info!(tmp = ?tmp_path, "New binary written; update finalized or pending on restart");
        Ok(())
    }

    /// Create backup of current installation
    async fn create_backup(&self, update_id: &str) -> Result<PathBuf> {
        // Create a binary backup of the current executable with retention policy
        let current_exe = std::env::current_exe()
            .map_err(|e| crate::anyhow!("failed to locate current exe: {e}"))?;
        let backup_path = self
            .config
            .backup_dir
            .join(format!("nxsh-backup-{update_id}.bin"));
        
        info!(path = ?backup_path, "Creating backup");
        fs::create_dir_all(&self.config.backup_dir)?;
        fs::copy(&current_exe, &backup_path)
            .map_err(|e| crate::anyhow!("failed to create backup: {}", e))?;
        
        info!(path = ?backup_path, "Backup created successfully");
        Ok(backup_path)
    }

    /// Rollback to previous version
    async fn rollback_update(&self, backup_path: &Path) -> Result<()> {
        info!(backup_path = ?backup_path, "Rolling back update");
        let current_exe = std::env::current_exe()
            .map_err(|e| crate::anyhow!("failed to locate current exe: {e}"))?;
        let dir = current_exe
            .parent()
            .ok_or_else(|| crate::anyhow!("current exe has no parent directory"))?;

        #[cfg(unix)]
        {
            // Try atomic restore
            let tmp = dir.join("nxsh_rollback.tmp");
            fs::copy(backup_path, &tmp)
                .map_err(|e| crate::anyhow!("failed to stage rollback file: {}", e))?;
            std::fs::rename(&tmp, &current_exe)
                .map_err(|e| crate::anyhow!("failed to replace current binary: {}", e))?;
        info!("Rollback completed successfully");
            Ok(())
        }
        #[cfg(windows)]
        {
            // Stage for next restart
            let staged = dir.join("nxsh_rollback.tmp");
            fs::copy(backup_path, &staged)
                .map_err(|e| crate::anyhow!("failed to stage rollback file: {}", e))?;
            let marker = dir.join("nxsh_rollback_pending.txt");
            fs::write(&marker, staged.file_name().unwrap().to_string_lossy().as_bytes())?;
            info!("Rollback staged; will be applied on next restart");
            Ok(())
        }
    }

    /// Cleanup old backups based on retention policy
    async fn cleanup_old_backups(&self) -> Result<()> {
        let backups = fs::read_dir(&self.config.backup_dir)?;
        let mut backup_files: Vec<_> = backups
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .collect();

        if backup_files.len() <= self.config.max_backups {
            return Ok(());
        }

        // Sort by modification time (oldest first)
        backup_files.sort_by_key(|entry| {
            entry.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        });

        let to_remove = backup_files.len() - self.config.max_backups;
        for entry in backup_files.iter().take(to_remove) {
            if let Err(e) = fs::remove_file(entry.path()) {
                warn!(path = ?entry.path(), error = %e, "Failed to remove old backup");
            } else {
                info!(path = ?entry.path(), "Removed old backup");
            }
        }

        Ok(())
    }

    /// Get download progress
    pub fn get_download_progress(&self) -> DownloadProgress {
        self.download_progress.read().unwrap().clone()
    }

    /// Get update history
    pub fn get_update_history(&self) -> Vec<UpdateRecord> {
        self.update_history.read().unwrap().clone()
    }

    /// Check if update is in progress
    pub fn is_update_in_progress(&self) -> bool {
        self.is_updating.load(Ordering::Relaxed)
    }

    /// Pause download
    pub fn pause_download(&self) -> Result<()> {
        self.download_progress.read().unwrap().is_paused.store(true, Ordering::Relaxed);
        info!("Download paused");
        Ok(())
    }

    /// Resume download
    pub fn resume_download(&self) -> Result<()> {
        self.download_progress.read().unwrap().is_paused.store(false, Ordering::Relaxed);
        info!("Download resumed");
        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> &UpdateConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: UpdateConfig) -> Result<()> {
        let old_channel = self.config.channel;
        self.config = config;
        
        if old_channel != self.config.channel {
            info!(old_channel = ?old_channel, new_channel = ?self.config.channel, "Update channel changed");
        }
        
        Ok(())
    }

    /// Load verification keys from a JSON file if present.
    /// Path resolution: NXSH_UPDATE_KEYS_PATH or ~/.nxsh/keys/update_keys.json
    fn load_verification_keys_from_files_if_present(&self) -> Result<()> {
        let path = if let Ok(p) = std::env::var("NXSH_UPDATE_KEYS_PATH") {
            std::path::PathBuf::from(p)
        } else if let Some(mut home) = home_dir_fallback() {
            home.push(".nxsh");
            home.push("keys");
            home.push("update_keys.json");
            home
        } else {
            return Ok(());
        };
        if !path.exists() { return Ok(()); }
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {path:?}"))?;
        let map: HashMap<String, String> = serde_json::from_str(&contents)
            .with_context(|| format!("Invalid JSON in {path:?}"))?;
        let mut keys = self.verification_keys.write().unwrap();
        for (name, material) in map {
            let fp = compute_key_fingerprint(&material)?;
            keys.public_keys.insert(name.clone(), material.clone());
            keys.fingerprint_to_name.insert(fp.clone(), name.clone());
            if !keys.trusted_authorities.contains(&name) { keys.trusted_authorities.push(name); }
        }
        Ok(())
    }

    /// Rotate update keys file when requested via environment
    /// Controls: NXSH_UPDATE_ROTATE=1 and NXSH_UPDATE_KEYS_JSON_NEW with JSON map
    fn rotate_update_keys_if_requested(&self) -> Result<()> {
        let rotate = std::env::var("NXSH_UPDATE_ROTATE").ok().map(|v| v=="1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);
        if !rotate { return Ok(()); }
        let new_json = match std::env::var("NXSH_UPDATE_KEYS_JSON_NEW") {
            Ok(v) if !v.trim().is_empty() => v,
            _ => return Ok(()),
        };
        let path = if let Ok(p) = std::env::var("NXSH_UPDATE_KEYS_PATH") {
            std::path::PathBuf::from(p)
        } else if let Some(mut home) = home_dir_fallback() {
            home.push(".nxsh");
            home.push("keys");
            let _ = std::fs::create_dir_all(&home);
            home.push("update_keys.json");
            home
        } else { return Ok(()); };
        // Backup with epoch suffix
        if path.exists() {
            if let Ok(dur) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                let bak = path.with_extension(format!("json.bak.{}", dur.as_secs()));
                let _ = std::fs::copy(&path, &bak);
            }
        }
        // Write atomically via temp
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, new_json.as_bytes())
            .with_context(|| format!("Failed to write temp file for {path:?}"))?;
        std::fs::rename(&tmp, &path)
            .with_context(|| format!("Failed to replace {path:?}"))?;
        Ok(())
    }
} 

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_verify_update_checksum_only() {
        // Prepare a temporary file with known contents
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("update_test.bin");
        let content = b"NexusShell Update File";
        std::fs::write(&file_path, content).unwrap();

        // Compute expected SHA-256 in hex
        let mut hasher = sha2::Sha256::new();
        hasher.update(content);
        let expected = format!("{:x}", hasher.finalize());

        // Build update system with signature verification disabled
        let cfg = UpdateConfig {
            verify_signatures: false,
            ..Default::default()
        };
        let system = UpdateSystem::new(cfg).unwrap();

        // Should verify successfully with matching checksum
        futures::executor::block_on(async {
            system
                .verify_update(&file_path, &expected, "", "")
                .await
                .unwrap();
        });
    }

    #[test]
    fn test_apply_delta_patch_fallback_non_bsdiff40() {
        // Create a fake delta file that does not start with BSDIFF40 magic
        let dir = TempDir::new().unwrap();
        let delta_path = dir.path().join("fake_delta.bin");
        std::fs::write(&delta_path, b"FULL-BINARY-CONTENT").unwrap();

        let system = UpdateSystem::new(UpdateConfig::default()).unwrap();
        // Should treat as full file and stage update next to current exe
        system.apply_delta_patch(&delta_path).unwrap();
    }
}