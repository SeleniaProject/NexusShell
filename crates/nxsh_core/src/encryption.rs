//! Encryption and security utilities for NexusShell
//!
//! This module provides cryptographic functions for secure data handling,
//! including password storage, secure communication, and data protection.

use crate::error::{ShellError, ErrorKind, ShellResult};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::SystemTime,
};
use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose, Engine as _};
use ring::{aead, digest, pbkdf2, rand};
use std::num::NonZeroU32;

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Key derivation iterations
    pub pbkdf2_iterations: u32,
    /// Salt length for key derivation
    pub salt_length: usize,
    /// Nonce length for AEAD
    pub nonce_length: usize,
    /// Tag length for AEAD
    pub tag_length: usize,
    /// Key rotation interval in seconds
    pub key_rotation_interval: u64,
    /// Maximum number of operations before key rotation
    pub max_operations_per_key: u64,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            pbkdf2_iterations: 100_000,
            salt_length: 32,
            nonce_length: 12,
            tag_length: 16,
            key_rotation_interval: 86400, // 24 hours
            max_operations_per_key: 1_000_000,
        }
    }
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Base64-encoded ciphertext
    pub ciphertext: String,
    /// Base64-encoded nonce
    pub nonce: String,
    /// Base64-encoded salt
    pub salt: String,
    /// Algorithm used for encryption
    pub algorithm: String,
    /// Key derivation parameters
    pub key_params: KeyDerivationParams,
    /// Timestamp of encryption
    pub timestamp: u64,
}

/// Key derivation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationParams {
    /// Number of iterations
    pub iterations: u32,
    /// Salt length
    pub salt_length: usize,
}

/// Encryption service
pub struct EncryptionService {
    config: EncryptionConfig,
    rng: rand::SystemRandom,
    keys: Arc<RwLock<HashMap<String, EncryptionKey>>>,
}

/// Encryption key with metadata
struct EncryptionKey {
    key: aead::UnboundKey,
    created_at: SystemTime,
    operations_count: u64,
}

impl EncryptionService {
    /// Create a new encryption service
    pub fn new(config: EncryptionConfig) -> Self {
        Self {
            config,
            rng: rand::SystemRandom::new(),
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Encrypt data with a password
    pub fn encrypt(&self, data: &[u8], password: &str) -> ShellResult<EncryptedData> {
        // Generate salt
        let mut salt = vec![0u8; self.config.salt_length];
        rand::SecureRandom::fill(&self.rng, &mut salt)
            .map_err(|_| ShellError::new(ErrorKind::CryptoError(crate::error::CryptoErrorKind::KeyGenerationFailed), "Failed to generate salt"))?;

        // Derive key from password
        let mut key = [0u8; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(self.config.pbkdf2_iterations).unwrap(),
            &salt,
            password.as_bytes(),
            &mut key,
        );

        // Generate nonce
        let mut nonce = vec![0u8; self.config.nonce_length];
        rand::SecureRandom::fill(&self.rng, &mut nonce)
            .map_err(|_| ShellError::new(ErrorKind::CryptoError(crate::error::CryptoErrorKind::KeyGenerationFailed), "Failed to generate nonce"))?;

        // Encrypt data
        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &key)
            .map_err(|_| ShellError::new(ErrorKind::CryptoError(crate::error::CryptoErrorKind::EncryptionFailed), "Failed to create encryption key"))?;
        let sealing_key = aead::LessSafeKey::new(unbound_key);
        let nonce_seq = aead::Nonce::try_assume_unique_for_key(&nonce)
            .map_err(|_| ShellError::new(ErrorKind::CryptoError(crate::error::CryptoErrorKind::EncryptionFailed), "Invalid nonce"))?;

        let mut in_out = data.to_vec();
        sealing_key.seal_in_place_append_tag(nonce_seq, aead::Aad::empty(), &mut in_out)
            .map_err(|_| ShellError::new(ErrorKind::CryptoError(crate::error::CryptoErrorKind::EncryptionFailed), "Encryption failed"))?;

        // Encode to base64
        let ciphertext = general_purpose::STANDARD.encode(&in_out);
        let nonce_b64 = general_purpose::STANDARD.encode(&nonce);
        let salt_b64 = general_purpose::STANDARD.encode(&salt);

        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_b64,
            salt: salt_b64,
            algorithm: "AES-256-GCM".to_string(),
            key_params: KeyDerivationParams {
                iterations: self.config.pbkdf2_iterations,
                salt_length: self.config.salt_length,
            },
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Decrypt data with a password
    pub fn decrypt(&self, encrypted_data: &EncryptedData, password: &str) -> ShellResult<Vec<u8>> {
        // Decode from base64
        let ciphertext = general_purpose::STANDARD.decode(&encrypted_data.ciphertext)
            .map_err(|_| ShellError::new(ErrorKind::CryptoError(crate::error::CryptoErrorKind::DecryptionFailed), "Failed to decode ciphertext"))?;
        let nonce = general_purpose::STANDARD.decode(&encrypted_data.nonce)
            .map_err(|_| ShellError::new(ErrorKind::CryptoError(crate::error::CryptoErrorKind::DecryptionFailed), "Failed to decode nonce"))?;
        let salt = general_purpose::STANDARD.decode(&encrypted_data.salt)
            .map_err(|_| ShellError::new(ErrorKind::CryptoError(crate::error::CryptoErrorKind::DecryptionFailed), "Failed to decode salt"))?;

        // Derive key from password
        let mut key = [0u8; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(encrypted_data.key_params.iterations).unwrap(),
            &salt,
            password.as_bytes(),
            &mut key,
        );

        // Decrypt data
        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &key)
            .map_err(|_| ShellError::new(ErrorKind::CryptoError(crate::error::CryptoErrorKind::DecryptionFailed), "Failed to create decryption key"))?;
        let opening_key = aead::LessSafeKey::new(unbound_key);
        let nonce_seq = aead::Nonce::try_assume_unique_for_key(&nonce)
            .map_err(|_| ShellError::new(ErrorKind::CryptoError(crate::error::CryptoErrorKind::DecryptionFailed), "Invalid nonce"))?;

        let mut in_out = ciphertext;
        let plaintext = opening_key.open_in_place(nonce_seq, aead::Aad::empty(), &mut in_out)
            .map_err(|_| ShellError::new(ErrorKind::CryptoError(crate::error::CryptoErrorKind::DecryptionFailed), "Decryption failed"))?;

        Ok(plaintext.to_vec())
    }

    /// Generate a random key
    pub fn generate_key(&self) -> ShellResult<Vec<u8>> {
        let mut key = vec![0u8; 32];
        rand::SecureRandom::fill(&self.rng, &mut key)
            .map_err(|_| ShellError::new(ErrorKind::CryptoError(crate::error::CryptoErrorKind::KeyGenerationFailed), "Failed to generate key"))?;
        Ok(key)
    }

    /// Hash data using SHA-256
    pub fn hash_sha256(&self, data: &[u8]) -> Vec<u8> {
        digest::digest(&digest::SHA256, data).as_ref().to_vec()
    }

    /// Hash data using SHA-512
    pub fn hash_sha512(&self, data: &[u8]) -> Vec<u8> {
        digest::digest(&digest::SHA512, data).as_ref().to_vec()
    }

    /// Securely compare two byte arrays
    pub fn secure_compare(&self, a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut result = 0u8;
        for (byte_a, byte_b) in a.iter().zip(b.iter()) {
            result |= byte_a ^ byte_b;
        }

        result == 0
    }

    /// Get encryption configuration
    pub fn config(&self) -> &EncryptionConfig {
        &self.config
    }
}

impl Default for EncryptionService {
    fn default() -> Self {
        Self::new(EncryptionConfig::default())
    }
} 