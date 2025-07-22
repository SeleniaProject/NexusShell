//! `builtin` builtin – force execution of a built-in command even if an
//! external command with the same name exists. Mirrors POSIX `builtin`.
//!
//! Usage:
//!   builtin CMD [ARGS...]
//!   builtin              # List available builtins
//!
//! This minimal implementation dispatches to a curated set of built-ins that
//! already exist inside `nxsh_builtins`. As more built-ins adopt a uniform
//! `Builtin` trait, this dispatcher can evolve to dynamic lookup. For now it
//! matches on command names and forwards arguments.

use anyhow::{anyhow, Result};

/// Entry point for the builtin.
pub fn builtin_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        // List supported builtins.
        println!(
            "Available built-ins: bg bind break (more will be auto-detected in future)"
        );
        return Ok(());
    }

    let cmd = &args[0];
    let rest: Vec<String> = args[1..].to_vec();

    match cmd.as_str() {
        "bg" => {
            // bg [JOBID]
            let id = rest.get(0).and_then(|s| s.parse::<u32>().ok());
            crate::bg(id)
        }
        "bind" => crate::bind(&rest),
        "break" => crate::break_cmd(&rest),
        _ => Err(anyhow!("builtin: unsupported command '{}'.", cmd)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn list() {
        builtin_cli(&[]).unwrap();
    }
} 