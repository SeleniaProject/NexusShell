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
            // bg [JOBID] - Background job control (not implemented)
            println!("bg: background job control not implemented yet");
            Ok(())
        }
        "bind" => {
            println!("bind: readline key bindings (not implemented yet)");
            Ok(())
        },
        "break" => {
            println!("break: exit from loops (not implemented yet)");
            Ok(())
        },
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


/// Execute function stub
pub fn execute(_args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}
