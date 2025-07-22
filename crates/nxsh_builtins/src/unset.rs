//! `unset` builtin – remove variables or functions.
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
            ctx.env.remove(name);
        } else {
            ctx.aliases.remove(name);
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