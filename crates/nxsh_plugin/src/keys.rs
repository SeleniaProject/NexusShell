//! Public key management for remote plugin signature verification
//!
//! This module provides a robust way to obtain trusted Ed25519 public keys
//! for official and community plugin repositories. Keys can be provided via:
//! - Environment variables: `NXSH_OFFICIAL_PUBKEY`, `NXSH_COMMUNITY_PUBKEY`
//! - Local files: `~/.nxsh/keys/official_ed25519.pub`, `~/.nxsh/keys/community_ed25519.pub`
//! - Built-in constants (compile-time embedded)
//!
//! The key strings must be base64-encoded raw 32-byte Ed25519 public keys.

use std::fs;
use base64::Engine;

/// Built-in fallback keys (base64). Replace with actual keys for production builds.
/// Leave empty to force external provisioning.
const BUILTIN_OFFICIAL_PUBKEY_B64: &str = ""; // base64-encoded 32-byte Ed25519 key
const BUILTIN_COMMUNITY_PUBKEY_B64: &str = ""; // base64-encoded 32-byte Ed25519 key

/// Load the official repository public key (base64) from env, file, or built-in.
pub fn load_official_pubkey_b64() -> String {
    // 1) Environment variable
    if let Ok(val) = std::env::var("NXSH_OFFICIAL_PUBKEY") {
        if !val.trim().is_empty() { return val; }
    }

    // 2) Local file under ~/.nxsh/keys/official_ed25519.pub
    if let Some(mut path) = dirs::home_dir() {
        path.push(".nxsh"); path.push("keys"); path.push("official_ed25519.pub");
        if let Ok(contents) = fs::read_to_string(&path) {
            let trimmed = contents.trim().to_string();
            if !trimmed.is_empty() { return trimmed; }
        }
    }

    // 3) Built-in constant
    BUILTIN_OFFICIAL_PUBKEY_B64.to_string()
}

/// Load the community repository public key (base64) from env, file, or built-in.
pub fn load_community_pubkey_b64() -> String {
    // 1) Environment variable
    if let Ok(val) = std::env::var("NXSH_COMMUNITY_PUBKEY") {
        if !val.trim().is_empty() { return val; }
    }

    // 2) Local file under ~/.nxsh/keys/community_ed25519.pub
    if let Some(mut path) = dirs::home_dir() {
        path.push(".nxsh"); path.push("keys"); path.push("community_ed25519.pub");
        if let Ok(contents) = fs::read_to_string(&path) {
            let trimmed = contents.trim().to_string();
            if !trimmed.is_empty() { return trimmed; }
        }
    }

    // 3) Built-in constant
    BUILTIN_COMMUNITY_PUBKEY_B64.to_string()
}

/// Helper to validate that a base64 key decodes to 32 bytes.
pub fn is_valid_ed25519_pubkey_b64(key_b64: &str) -> bool {
    if key_b64.trim().is_empty() { return false; }
    let decoded = match base64::engine::general_purpose::STANDARD.decode(key_b64) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    decoded.len() == 32
}


