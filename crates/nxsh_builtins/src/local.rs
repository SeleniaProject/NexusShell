//! `local` builtin  Edeclare local variables within function scope.
//! As function scoping is not fully implemented yet, this builtin currently
//! behaves similarly to `declare` but sets a special `__local_` prefix which
//! future function frames will interpret as local scope.

use anyhow::Result;
use nxsh_core::context::ShellContext;

pub fn local_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    for arg in args {
        if let Some((name, val)) = arg.split_once('=') {
            ctx.set_var(format!("__local_{name}"), val);
        } else {
            ctx.set_var(format!("__local_{arg}"), "");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn set_local() {
        let ctx = ShellContext::new();
        local_cli(&["foo=bar".into()], &ctx).unwrap();
        assert_eq!(ctx.get_var("__local_foo").unwrap(), "bar");
    }
} 
