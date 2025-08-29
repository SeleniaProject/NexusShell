//! Public key management for remote plugin signature verification
//!
//! This module provides a robust way to obtain trusted Ed25519 public keys
//! for official and community plugin repositories. Keys can be provided via:
//! - Environment variables: `NXSH_OFFICIAL_PUBKEY`, `NXSH_COMMUNITY_PUBKEY`
//! - Local files: `~/.nxsh/keys/official_ed25519.pub`, `~/.nxsh/keys/community_ed25519.pub`
//! - Built-in constants (compile-time embedded)
//!
//! The key strings must be base64-encoded raw 32-byte Ed25519 public keys.

#[cfg(feature = "crypto-verification")]
use base64::Engine;
#[cfg(any(feature = "crypto-verification", feature = "plugin-management"))]
use std::fs;

/// Built-in fallback keys (base64). Replace with actual keys for production builds.
/// Leave empty to force external provisioning.
const BUILTIN_OFFICIAL_PUBKEY_B64: &str = ""; // base64-encoded 32-byte Ed25519 key
const BUILTIN_COMMUNITY_PUBKEY_B64: &str = ""; // base64-encoded 32-byte Ed25519 key

/// Load the official repository public key (base64) from env, file, or built-in.
pub fn load_official_pubkey_b64() -> String {
    // 1) Environment variable
    if let Ok(val) = std::env::var("NXSH_OFFICIAL_PUBKEY") {
        if !val.trim().is_empty() {
            return val;
        }
    }

    // 2) Local file under ~/.nxsh/keys/official_ed25519.pub
    #[cfg(feature = "plugin-management")]
    if let Some(mut path) = dirs::home_dir() {
        path.push(".nxsh");
        path.push("keys");
        path.push("official_ed25519.pub");
        if let Ok(contents) = fs::read_to_string(&path) {
            let trimmed = contents.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
    }

    // 3) Built-in constant
    BUILTIN_OFFICIAL_PUBKEY_B64.to_string()
}

/// Load the community repository public key (base64) from env, file, or built-in.
pub fn load_community_pubkey_b64() -> String {
    // 1) Environment variable
    if let Ok(val) = std::env::var("NXSH_COMMUNITY_PUBKEY") {
        if !val.trim().is_empty() {
            return val;
        }
    }

    // 2) Local file under ~/.nxsh/keys/community_ed25519.pub
    #[cfg(feature = "plugin-management")]
    if let Some(mut path) = dirs::home_dir() {
        path.push(".nxsh");
        path.push("keys");
        path.push("community_ed25519.pub");
        if let Ok(contents) = fs::read_to_string(&path) {
            let trimmed = contents.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
    }

    // 3) Built-in constant
    BUILTIN_COMMUNITY_PUBKEY_B64.to_string()
}

/// Helper to validate that a base64 key decodes to 32 bytes.
pub fn is_valid_ed25519_pubkey_b64(key_b64: &str) -> bool {
    if key_b64.trim().is_empty() {
        return false;
    }
    #[cfg(feature = "crypto-verification")]
    let decoded = match base64::engine::general_purpose::STANDARD.decode(key_b64) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    #[cfg(feature = "crypto-verification")]
    return decoded.len() == 32;
    #[cfg(not(feature = "crypto-verification"))]
    {
        false
    }
}

/// Rotate trusted keys by environment-supplied new keys; returns (old_official, old_community).
/// When NXSH_ROTATE_KEYS=1 and NXSH_NEW_OFFICIAL_PUBKEY/NXSH_NEW_COMMUNITY_PUBKEY are provided,
/// this function writes new keys to ~/.nxsh/keys/*.pub atomically, keeping a timestamped backup.
pub fn rotate_trusted_keys_if_requested() -> std::io::Result<(Option<String>, Option<String>)> {
    if std::env::var("NXSH_ROTATE_KEYS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        let new_off = std::env::var("NXSH_NEW_OFFICIAL_PUBKEY").ok();
        let new_com = std::env::var("NXSH_NEW_COMMUNITY_PUBKEY").ok();
        if new_off.is_none() && new_com.is_none() {
            return Ok((None, None));
        }
        #[cfg(feature = "plugin-management")]
        let home = dirs::home_dir().ok_or_else(|| std::io::Error::other("no home dir"))?;
        #[cfg(not(feature = "plugin-management"))]
        let home: std::path::PathBuf = std::env::var_os("HOME")
            .map(Into::into)
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let keys_dir = home.join(".nxsh").join("keys");
        let _ = std::fs::create_dir_all(&keys_dir);

        let mut old_off: Option<String> = None;
        let mut old_com: Option<String> = None;

        fn backup_and_write(
            path: &std::path::Path,
            new_val: &str,
        ) -> std::io::Result<Option<String>> {
            let old = std::fs::read_to_string(path)
                .ok()
                .map(|s| s.trim().to_string());
            if let Some(ref o) = old {
                if o == new_val {
                    return Ok(old);
                }
            }
            if path.exists() {
                let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
                let bak = path.with_extension(format!("pub.bak.{ts}"));
                let _ = std::fs::copy(path, &bak);
            }
            let tmp = path.with_extension("pub.tmp");
            std::fs::write(&tmp, new_val.as_bytes())?;
            std::fs::rename(&tmp, path)?;
            Ok(old)
        }

        if let Some(val) = new_off {
            let p = keys_dir.join("official_ed25519.pub");
            old_off = backup_and_write(&p, val.trim())?;
        }
        if let Some(val) = new_com {
            let p = keys_dir.join("community_ed25519.pub");
            old_com = backup_and_write(&p, val.trim())?;
        }
        return Ok((old_off, old_com));
    }
    Ok((None, None))
}
