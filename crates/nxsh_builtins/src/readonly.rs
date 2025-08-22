//! `readonly` builtin  Emark shell variables as immutable.
//! Syntax: readonly NAME[=VALUE] ...
//! For simplicity, this implementation records readonly keys in a hidden
//! variable `__READONLY_SET` stored as a comma-separated list inside
//! `ShellContext`. Future attempts to modify these variables via `set_var`
//! should consult this list (not yet enforced globally).

use anyhow::{anyhow, Result};
use nxsh_core::context::ShellContext;

const READONLY_SET_KEY: &str = "__READONLY_SET";

pub fn readonly_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    if args.is_empty() {
        // list readonly vars
        if let Some(list) = ctx.get_var(READONLY_SET_KEY) {
            for name in list.split(',').filter(|s| !s.is_empty()) {
                if let Some(val) = ctx.get_var(name) {
                    println!("readonly {name}={val}");
                } else {
                    println!("readonly {name}");
                }
            }
        }
        return Ok(());
    }

    let mut readonly_set: Vec<String> = ctx
        .get_var(READONLY_SET_KEY)
        .map(|s| s.split(',').filter(|v| !v.is_empty()).map(|v| v.to_string()).collect())
        .unwrap_or_default();

    for arg in args {
        if let Some((name, val)) = arg.split_once('=') {
            if readonly_set.contains(&name.to_string()) {
                return Err(anyhow!("{}: readonly variable", name));
            }
            ctx.set_var(name, val);
            readonly_set.push(name.to_string());
        } else if !readonly_set.contains(arg) {
            readonly_set.push(arg.clone());
        }
    }
    ctx.set_var(READONLY_SET_KEY, readonly_set.join(","));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn set_and_list() {
        let ctx = ShellContext::new();
        readonly_cli(&["foo=bar".into()], &ctx).unwrap();
        readonly_cli(&[], &ctx).unwrap();
    }
} 


/// Execute function stub
pub fn execute(_args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}
