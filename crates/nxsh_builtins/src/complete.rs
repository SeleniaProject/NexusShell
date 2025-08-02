//! `complete` builtin  Eregister or list completion scripts.
//! Syntax examples:
//!   complete --list                 # list all registered completions
//!   complete CMD SCRIPT             # register SCRIPT for CMD (overwrite if exists)
//!   complete --remove CMD           # delete completion for CMD
//!
//! Completion scripts are stored under:
//!   <config>/nexusshell/completions/<CMD>.comp
//! The script content can be any text; interpretation is handled by the
//! line-editor layer (future work).

use anyhow::{anyhow, Context, Result};
use dirs_next::config_dir;
use std::{env, fs, path::PathBuf};

pub fn complete_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return list_completions();
    }

    match args[0].as_str() {
        "--list" => list_completions(),
        "--remove" if args.len() == 2 => remove_completion(&args[1]),
        cmd => {
            if args.len() < 2 {
                return Err(anyhow!("complete: missing SCRIPT argument"));
            }
            let script = args[1..].join(" ");
            add_completion(cmd, &script)
        }
    }
}

fn completions_dir() -> Result<PathBuf> {
    if let Ok(dir) = env::var("NXSH_CONFIG_DIR") {
        return Ok(PathBuf::from(dir).join("completions"));
    }
    let base = config_dir().context("unable to determine config directory")?;
    Ok(base.join("nexusshell").join("completions"))
}

fn list_completions() -> Result<()> {
    let dir = completions_dir()?;
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
            println!("{}", name);
        }
    }
    Ok(())
}

fn add_completion(cmd: &str, script: &str) -> Result<()> {
    let dir = completions_dir()?;
    fs::create_dir_all(&dir)?;
    let file = dir.join(format!("{}.comp", cmd));
    fs::write(file, script)?;
    Ok(())
}

fn remove_completion(cmd: &str) -> Result<()> {
    let file = completions_dir()?.join(format!("{}.comp", cmd));
    if file.exists() {
        fs::remove_file(file)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn add_list_remove() {
        let dir = tempdir().unwrap();
        std::env::set_var("NXSH_CONFIG_DIR", dir.path());

        add_completion("foo", "echo foo").unwrap();
        let output = list_completions();
        assert!(output.is_ok());

        remove_completion("foo").unwrap();
        let list_after = list_completions();
        assert!(list_after.is_ok());
    }
} 
