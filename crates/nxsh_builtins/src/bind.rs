//! `bind` builtin – configure key bindings for the NexusShell line editor.
//! Syntax: `bind KEYSEQ:COMMAND [KEYSEQ:COMMAND ...]`
//!
//! Behavior:
//! • No arguments     → print current bindings in KEYSEQ:COMMAND form.
//! • KEYSEQ:COMMAND   → add/update binding in config file.
//!
//! Bindings are persisted to `keymap` file inside the config directory:
//!   $NXSH_CONFIG_DIR/keymap  (if env var set)
//!   otherwise: ~/.config/nexusshell/keymap
//!
//! The file format is plain UTF-8 with one binding per line: `KeySeq:Command`.
//! Duplicate keys are overridden by the last entry.

use anyhow::{anyhow, Context, Result};
use dirs_next::config_dir;
use std::{collections::HashMap, env, fs, path::PathBuf};

/// Entry point for the `bind` builtin.
pub fn bind_cli(args: &[String]) -> Result<()> {
    let mut bindings = load_bindings()?;

    if args.is_empty() {
        // List
        for (k, v) in &bindings {
            println!("{}:{}", k, v);
        }
        return Ok(());
    }

    for arg in args {
        let (keyseq, cmd) = arg
            .split_once(':')
            .ok_or_else(|| anyhow!("bind: argument must be KEYSEQ:COMMAND"))?;
        if keyseq.is_empty() || cmd.is_empty() {
            return Err(anyhow!("bind: key sequence and command cannot be empty"));
        }
        bindings.insert(keyseq.to_string(), cmd.to_string());
    }

    save_bindings(&bindings)?;
    Ok(())
}

/// Returns the concrete path to the keymap file.
fn keymap_path() -> Result<PathBuf> {
    if let Ok(dir) = env::var("NXSH_CONFIG_DIR") {
        return Ok(PathBuf::from(dir).join("keymap"));
    }
    let base = config_dir().context("unable to determine config directory")?;
    Ok(base.join("nexusshell").join("keymap"))
}

fn load_bindings() -> Result<HashMap<String, String>> {
    let path = keymap_path()?;
    let content = fs::read_to_string(&path).unwrap_or_default();
    let mut map = HashMap::new();
    for line in content.lines() {
        if let Some((k, v)) = line.split_once(':') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Ok(map)
}

fn save_bindings(map: &HashMap<String, String>) -> Result<()> {
    let path = keymap_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by_key(|(k, _)| *k);
    let content: String = entries
        .into_iter()
        .map(|(k, v)| format!("{}:{}\n", k, v))
        .collect();
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_bind_add_and_list() {
        let dir = tempdir().unwrap();
        std::env::set_var("NXSH_CONFIG_DIR", dir.path());

        // Add binding
        bind_cli(&["C-S:split-pane-horizontal".to_string()]).unwrap();

        // Load again and assert
        let map = load_bindings().unwrap();
        assert_eq!(map.get("C-S").unwrap(), "split-pane-horizontal");

        // List should output the binding (capture stdout)
        // Simple smoke test – ensure no error
        bind_cli(&[]).unwrap();
    }
} 