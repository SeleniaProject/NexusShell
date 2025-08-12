//! Built-in command traits and utilities

use anyhow::{anyhow, Result};

pub use nxsh_core::Builtin;

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
            let _id = rest.first().and_then(|s| s.parse::<u32>().ok());
            crate::bg::bg_cli(&rest)
        }
        "bind" => crate::bind::bind_cli(&rest),
        "break" => crate::break_builtin::break_cli(&rest),
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
