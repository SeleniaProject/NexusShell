//! `unset` builtin  Eremove variables or functions.
//! For now supports variables (-v) and aliases (-f maps to alias removal).
//! Usage: unset [-v|-f] NAME ...

use anyhow::{Result};
use nxsh_core::context::ShellContext;

pub fn unset_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    if args.is_empty() { return Ok(()); }
    let mut mode_var = true; // default variable
    let mut names_start = 0;
    if args[0] == "-f" { mode_var = false; names_start = 1; }
    if args[0] == "-v" { mode_var = true; names_start = 1; }

    for name in &args[names_start..] {
        if mode_var {
            if let Ok(mut env_guard) = ctx.env.write() { env_guard.remove(name); }
            if let Ok(mut vars_guard) = ctx.vars.write() { vars_guard.remove(name); }
        } else if let Ok(mut aliases_guard) = ctx.aliases.write() {
            aliases_guard.remove(name);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unset_var() {
        let ctx = ShellContext::new();
        ctx.set_var("FOO", "bar");
        unset_cli(&["FOO".into()], &ctx).unwrap();
        assert!(ctx.get_var("FOO").is_none());
    }
} 
