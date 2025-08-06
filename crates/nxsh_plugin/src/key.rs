use anyhow::Result;
use ed25519_dalek::{SigningKey, VerifyingKey};  // Pure Rust Ed25519 implementation
use base64::{engine::general_purpose, Engine as _};
use rand::rngs::OsRng;  // Cryptographically secure random number generator

/// Generate an Ed25519 keypair using Pure Rust implementation
/// 
/// Returns a tuple of (public_key_base64, private_key_base64)
/// This implementation is memory-safe, formally verifiable, and compatible with WebAssembly
pub fn generate_keypair() -> Result<(String, String)> {
    // Generate a new signing key using cryptographically secure randomness
    let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
    
    // Derive the verifying key from the signing key
    let verifying_key = signing_key.verifying_key();
    
    // Encode keys as base64 for storage and transmission
    let private_key_b64 = general_purpose::STANDARD.encode(signing_key.to_bytes());
    let public_key_b64 = general_purpose::STANDARD.encode(verifying_key.to_bytes());
    
    Ok((public_key_b64, private_key_b64))
} 