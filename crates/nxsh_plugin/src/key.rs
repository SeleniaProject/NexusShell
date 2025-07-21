use anyhow::Result;
use ring::signature::{Ed25519KeyPair, KeyPair, Signature};
use base64::{engine::general_purpose, Engine as _};

pub fn generate_keypair() -> Result<(String,String)> {
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new())?;
    let kp = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref())?;
    let priv_b64 = general_purpose::STANDARD.encode(pkcs8.as_ref());
    let pub_b64 = general_purpose::STANDARD.encode(kp.public_key().as_ref());
    Ok((pub_b64, priv_b64))
} 