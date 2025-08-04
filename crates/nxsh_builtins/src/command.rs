//! `command` builtin  EPATH search & type query
//! Supports a subset of Bash `command` options:
//!   -v name … print the location/type of each name (concise)
//!   -V name … verbose output describing how each name would be interpreted
//!   Without -v/-V and with a command, this builtin simply executes the
//!   external command, bypassing shell aliases (not yet implemented, placeholder).

use anyhow::{anyhow, Result};
use std::{env, path::PathBuf};
use nxsh_core::context::ShellContext;

pub const BUILTIN_NAMES: &[&str] = &[
    "alias", "bg", "bind", "break", "builtin", "command", "continue", "disown", "echo", "eval", "exec", "exit", "getopts", "hash", "let", "local", "pushd", "popd", "pwd", "read", "readonly", "return", "shift", "source", "suspend", "times", "trap", "type", "ulimit", "umask", "unalias", "unset", "cp", "mv", "rm", "mkdir", "rmdir", "ln", "stat", "touch", "tree", "du", "df", "sync", "mount", "umount", "shred", "split", "cat", "more", "less",
    // TODO: add more names as built-ins are implemented
];

/// Entry function.
pub fn command_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("command: missing arguments"));
    }

    let mut verbose = false;
    let mut list_only = false;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "-v" {
            list_only = true;
        } else if arg == "-V" {
            verbose = true;
            list_only = true;
        } else if arg.starts_with('-') {
            return Err(anyhow!("command: unsupported option '{}'.", arg));
        } else {
            // First non-option -> treat as command names list
            let mut names = vec![arg.clone()];
            names.extend(iter.cloned());
            return handle_query(names, ctx, list_only, verbose);
        }
    }
    Ok(())
}

fn handle_query(names: Vec<String>, ctx: &ShellContext, list_only: bool, verbose: bool) -> Result<()> {
    for name in names {
        if let Some(alias) = ctx.get_alias(&name) {
            if verbose {
                println!("{} is an alias for '{}'", name, alias);
            } else {
                println!("{}", alias);
            }
            continue;
        }
        if BUILTIN_NAMES.contains(&name.as_str()) {
            if verbose {
                println!("{} is a shell builtin", name);
            } else {
                println!("{}", name);
            }
            continue;
        }
        match lookup_path(&name) {
            Some(path) => {
                if verbose {
                    println!("{} is {}", name, path.display());
                } else {
                    println!("{}", path.display());
                }
            }
            None => {
                // Not found
                if verbose {
                    println!("{} not found", name);
                }
            }
        }
    }
    Ok(())
}

fn lookup_path(cmd: &str) -> Option<PathBuf> {
    let path_var = env::var("PATH").unwrap_or_default();
    for dir in env::split_paths(&path_var) {
        let p = dir.join(cmd);
        if p.is_file() && is_executable(&p) {
            return Some(p);
        }
    }
    None
}

#[cfg(unix)]
fn is_executable(p: &PathBuf) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
}

#[cfg(windows)]
fn is_executable(p: &PathBuf) -> bool {
    p.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "exe" | "bat" | "cmd"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nxsh_core::context::ShellContext;

    #[test]
    fn alias_lookup() {
        let ctx = ShellContext::new();
        ctx.set_alias("ll", "ls -l").unwrap();
        command_cli(&["-v".into(), "ll".into()], &ctx).unwrap();
    }
} 
