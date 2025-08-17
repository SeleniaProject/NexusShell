use anyhow::Result;
use aes_gcm::{Aes256Gcm, KeyInit};
use aes_gcm::aead::{Aead, OsRng};
use rand::RngCore;

const MAGIC: &[u8; 8] = b"NXSHHIST"; // Magic header for encrypted history
const VERSION: u8 = 1; // Format version
const SALT_LEN: usize = 16; // Argon2 salt size
const NONCE_LEN: usize = 12; // AES-GCM nonce size

fn derive_key_argon2id(passphrase: &str, salt: &[u8]) -> Result<[u8; 32]> {
    use argon2::Argon2;
    // Tuneable parameters via env (optional)
    let m: u32 = std::env::var("NXSH_HISTORY_ARGON2_M_COST").ok().and_then(|v| v.parse().ok()).unwrap_or(19456);
    let t: u32 = std::env::var("NXSH_HISTORY_ARGON2_T_COST").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
    let p: u32 = std::env::var("NXSH_HISTORY_ARGON2_P_COST").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let params = argon2::Params::new(m, t, p, None)
        .map_err(|e| anyhow::anyhow!("Invalid Argon2 params: {}", e))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2.hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow::anyhow!("Argon2id key derivation failed: {}", e))?;
    Ok(key)
}

pub fn encrypt_history(passphrase: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);

    let key = derive_key_argon2id(passphrase, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("AES-GCM init failed: {}", e))?;

    let ciphertext = cipher.encrypt((&nonce).into(), plaintext)
        .map_err(|e| anyhow::anyhow!("AES-GCM encryption failed: {}", e))?;

    let mut out = Vec::with_capacity(8 + 1 + SALT_LEN + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

pub fn is_encrypted_history(data: &[u8]) -> bool {
    data.len() >= 8 + 1 + SALT_LEN + NONCE_LEN && &data[..8] == MAGIC
}

pub fn decrypt_history(passphrase: &str, data: &[u8]) -> Result<Vec<u8>> {
    if !is_encrypted_history(data) {
        anyhow::bail!("Not an encrypted history file");
    }
    let version = data[8];
    if version != VERSION {
        anyhow::bail!("Unsupported history format version: {}", version);
    }
    let salt_start = 9;
    let nonce_start = salt_start + SALT_LEN;
    let ct_start = nonce_start + NONCE_LEN;
    let salt = &data[salt_start..salt_start + SALT_LEN];
    let nonce = &data[nonce_start..nonce_start + NONCE_LEN];
    let ciphertext = &data[ct_start..];

    let key = derive_key_argon2id(passphrase, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("AES-GCM init failed: {}", e))?;

    let plaintext = cipher.decrypt(nonce.into(), ciphertext)
        .map_err(|_| anyhow::anyhow!("History decryption failed (wrong passphrase or tampered data)"))?;
    Ok(plaintext)
}


