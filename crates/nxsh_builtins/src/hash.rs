//! `hash` builtin – command lookup cache management.
//! Supported subset of Bash options:
//!   hash                 # list current cache entries
//!   hash CMD [...]       # search PATH for each CMD and cache its location
//!   hash -r              # reset/clear the cache
//!
//! The cache lives for the process lifetime and accelerates repeated PATH
//! look-ups by the shell.

use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static CACHE: Lazy<Mutex<HashMap<String, PathBuf>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn hash_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        list_cache();
        return Ok(());
    }
    if args.len() == 1 && args[0] == "-r" {
        CACHE.lock().unwrap().clear();
        return Ok(());
    }
    for cmd in args {
        if let Some(path) = lookup_path(cmd) {
            CACHE.lock().unwrap().insert(cmd.clone(), path);
        }
    }
    Ok(())
}

fn list_cache() {
    let cache = CACHE.lock().unwrap();
    for (cmd, path) in cache.iter() {
        println!("{} = {}", cmd, path.display());
    }
}

fn lookup_path(cmd: &str) -> Option<PathBuf> {
    // Check direct cache first
    if let Some(path) = CACHE.lock().unwrap().get(cmd) {
        return Some(path.clone());
    }
    // Absolute or relative path with slash – take as is
    if cmd.contains('/') || cmd.contains('\\') {
        let p = PathBuf::from(cmd);
        if p.is_file() { return Some(p); }
    }
    let path_env = env::var("PATH").unwrap_or_default();
    for dir in env::split_paths(&path_env) {
        let candidate = dir.join(cmd);
        if candidate.is_file() && is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
}

#[cfg(windows)]
fn is_executable(p: &Path) -> bool {
    p.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "exe" | "bat" | "cmd"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_cycle() {
        hash_cli(&["-r".into()]).unwrap(); // clear
        hash_cli(&["echo".into()]).unwrap();
        assert!(!CACHE.lock().unwrap().is_empty());
    }
} 