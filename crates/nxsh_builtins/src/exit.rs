//! `exit` builtin  Eterminate shell with optional status code.
//! Usage: `exit [N]`.

use anyhow::Result;

pub fn exit_cli(args: &[String]) -> Result<()> {
    let code = if args.is_empty() { 0 } else { args[0].parse::<i32>().unwrap_or(1) };
    std::process::exit(code);
}

/// Execute exit command
pub fn execute(args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    match exit_cli(args) {
        Ok(_) => Ok(0),
        Err(e) => {
            eprintln!("exit: {e}");
            Ok(1)
        }
    }
} 

