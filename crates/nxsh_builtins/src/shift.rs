//! `shift` builtin  Eshift positional parameters left by N.
//! For initial implementation, positional parameters are stored in
//! `__ARGV` variable within `ShellContext` as a NUL-separated list.
//! `shift [N]` removes first N elements.

use anyhow::{anyhow, Result};
use nxsh_core::context::ShellContext;

const ARGV_KEY: &str = "__ARGV";

pub fn shift_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    let n = if args.is_empty() { 1 } else { args[0].parse::<usize>().unwrap_or(1) };
    let argv_raw = ctx.get_var(ARGV_KEY).unwrap_or_default();
    let mut parts: Vec<&str> = argv_raw.split('\0').filter(|s| !s.is_empty()).collect();
    if n > parts.len() {
        return Err(anyhow!("shift: not enough positional parameters"));
    }
    parts.drain(0..n);
    ctx.set_var(ARGV_KEY, parts.join("\0"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shift_once() {
        let ctx = ShellContext::new();
        ctx.set_var(ARGV_KEY, "a\0b\0c".to_string());
        shift_cli(&[], &ctx).unwrap();
        assert_eq!(ctx.get_var(ARGV_KEY).unwrap(), "b\0c");
    }
} 
