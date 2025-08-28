//! `unalias` builtin  Eremove aliases.
//! Usage:
//!   unalias NAME ...  # remove specified aliases
//!   unalias -a        # remove all aliases

use anyhow::{Result, anyhow};
use nxsh_core::context::ShellContext;

pub fn unalias_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("unalias: missing arguments"));
    }
    if args[0] == "-a" {
        if let Ok(mut aliases_guard) = ctx.aliases.write() {
            aliases_guard.clear();
        }
        return Ok(());
    }
    for name in args {
        if let Ok(mut aliases_guard) = ctx.aliases.write() {
            aliases_guard.remove(name);
        }
    }
    Ok(())
} 


/// Execute function stub
pub fn execute(_args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn remove_alias() {
        let ctx = ShellContext::new();
        ctx.set_alias("ll", "ls -l").unwrap();
        unalias_cli(&["ll".into()], &ctx).unwrap();
        assert!(ctx.get_alias("ll").is_none());
    }
}
