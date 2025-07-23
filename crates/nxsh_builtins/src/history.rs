use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, OsRng};
use anyhow::{Context as AnyhowContext, Result};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use dirs_next::home_dir;
use crate::logging;

const NONCE_LEN: usize = 12;
const KEY_ENV: &str = "NXSH_HISTORY_KEY";
const DEFAULT_HISTORY_FILE: &str = ".nxsh_history";

fn history_path() -> PathBuf {
    home_dir().unwrap_or_else(|| PathBuf::from("."))
        .join(DEFAULT_HISTORY_FILE)
}

fn load_key() -> Result<aes_gcm::Key<Aes256Gcm>> {
    if let Ok(encoded) = std::env::var(KEY_ENV) {
        let bytes = general_purpose::STANDARD.decode(encoded)?;
        if bytes.len() == 32 {
            return Ok(*aes_gcm::Key::<Aes256Gcm>::from_slice(&bytes));
        }
    }
    // Generate random key and tell user to persist
    let mut key_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key_bytes);
    let encoded = general_purpose::STANDARD.encode(key_bytes);
    logging::info_i18n(
        &format!("履歴暗号化キーを生成しました: {} (環境変数 {} に設定してください)", encoded, KEY_ENV),
        &format!("Generated history encryption key: {} (set env {} to persist)", encoded, KEY_ENV),
    );
    Ok(*aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes))
}

pub fn add(command: &str) -> Result<()> {
    let key = load_key()?;
    let cipher = Aes256Gcm::new(&key);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, command.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(history_path())?;

    writeln!(
        file,
        "{}:{}",
        general_purpose::STANDARD.encode(nonce_bytes),
        general_purpose::STANDARD.encode(ciphertext)
    )?;
    Ok(())
}

pub fn show() -> Result<()> {
    let key = load_key()?;
    let cipher = Aes256Gcm::new(&key);
    let file = match File::open(history_path()) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };

    for line in BufReader::new(file).lines() {
        let line = line?;
        if let Some((nonce_b64, ct_b64)) = line.split_once(':') {
            let nonce_bytes = general_purpose::STANDARD.decode(nonce_b64)?;
            let ct_bytes = general_purpose::STANDARD.decode(ct_b64)?;
            if nonce_bytes.len() != NONCE_LEN { continue; }
            let nonce = Nonce::from_slice(&nonce_bytes);
            match cipher.decrypt(nonce, ct_bytes.as_ref()) {
                Ok(plaintext) => {
                    println!("{}", String::from_utf8_lossy(&plaintext));
                }
                Err(_) => {
                    println!("[corrupt entry]");
                }
            }
        }
    }
    Ok(())
}

/// `history -s <cmd>` support to append command without executing.
pub fn history_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return show();
    }
    if args[0] == "-s" {
        if args.len() < 2 {
            anyhow::bail!("-s requires argument");
        }
        let cmd = args[1..].join(" ");
        add(&cmd)
    } else {
        show()
    }
} 