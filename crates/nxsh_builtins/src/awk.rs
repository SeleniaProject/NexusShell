//! `awk` command – pattern scanning & processing language wrapper.
//!
//! This built-in defers to an external implementation (frawk > gawk > awk)
//! to provide full POSIX awk functionality with maximum performance while
//! avoiding heavy compile-time dependencies inside NexusShell.
//!
//! Usage is identical to normal awk:
//!   awk 'PROGRAM' FILE...
//!   awk -f SCRIPT.awk FILE...
//!
//! If none of the known back-end executables are found in PATH, an error is
//! returned. Users can override detection by setting the environment variable
//! `NXSH_AWK_CMD` to an absolute path or command name.

use anyhow::{anyhow, Result};
use std::env;
use std::process::Command;

/// Entry point exposed to NexusShell.
pub fn awk_cli(args: &[String]) -> Result<()> {
    let candidates = candidate_commands();
    let backend = candidates
        .into_iter()
        .find(|cmd| is_executable(cmd))
        .ok_or_else(|| anyhow!("awk: no compatible backend (frawk/gawk/awk) found in PATH"))?;

    // Spawn the backend inheriting stdio so that interactive programs work.
    let status = Command::new(&backend)
        .args(args)
        .status()
        .map_err(|e| anyhow!("awk: failed to launch '{}': {}", backend, e))?;

    if !status.success() {
        return Err(anyhow!("awk: backend exited with status {:?}", status.code()));
    }
    Ok(())
}

/// Returns list of command names to try, prioritised.
fn candidate_commands() -> Vec<String> {
    if let Ok(cmd) = env::var("NXSH_AWK_CMD") {
        return vec![cmd];
    }
    vec!["frawk", "gawk", "awk"].into_iter().map(String::from).collect()
}

/// Simple PATH lookup to verify command existence & executability.
fn is_executable(cmd: &str) -> bool {
    // If cmd contains a path separator, test it directly.
    if cmd.contains(std::path::MAIN_SEPARATOR) {
        return std::fs::metadata(cmd).map(|m| m.is_file()).unwrap_or(false);
    }
    if let Ok(path_var) = env::var("PATH") {
        for dir in env::split_paths(&path_var) {
            let candidate = dir.join(cmd);
            if cfg!(windows) {
                // On Windows search .exe explicitly
                let exe = candidate.with_extension("exe");
                if exe.is_file() {
                    return true;
                }
            }
            if candidate.is_file() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_contains_default() {
        let list = candidate_commands();
        assert!(list.contains(&"awk".to_string()));
    }
} 