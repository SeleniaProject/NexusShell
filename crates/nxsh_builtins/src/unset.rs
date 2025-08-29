//! `unset` builtin  Eremove variables or functions.
//! For now supports variables (-v) and aliases (-f maps to alias removal).
//! Usage: unset [-v|-f] NAME ...

use anyhow::Result;
use nxsh_core::context::ShellContext;

pub fn unset_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    let mut mode_var = true; // default variable
    let mut names_start = 0;
    if args[0] == "-f" {
        mode_var = false;
        names_start = 1;
    }
    if args[0] == "-v" {
        mode_var = true;
        names_start = 1;
    }

    for name in &args[names_start..] {
        if mode_var {
            if let Ok(mut env_guard) = ctx.env.write() {
                env_guard.remove(name);
            }
            if let Ok(mut vars_guard) = ctx.vars.write() {
                vars_guard.remove(name);
            }
        } else if let Ok(mut aliases_guard) = ctx.aliases.write() {
            aliases_guard.remove(name);
        }
    }
    Ok(())
}

/// Execute the unset builtin command
pub fn execute(
    args: &[String],
    _context: &crate::common::BuiltinContext,
) -> crate::common::BuiltinResult<i32> {
    if args.is_empty() {
        eprintln!("unset: not enough arguments");
        return Ok(1);
    }

    for var_name in args {
        if var_name.starts_with('-') {
            match var_name.as_str() {
                "-h" | "--help" => {
                    println!("Usage: unset [name ...]");
                    println!("Unset environment variables.");
                    println!();
                    println!("Examples:");
                    println!("  unset PATH        Unset PATH variable");
                    println!("  unset VAR1 VAR2   Unset multiple variables");
                    return Ok(0);
                }
                _ => {
                    eprintln!("unset: invalid option '{var_name}'");
                    return Ok(1);
                }
            }
        }

        std::env::remove_var(var_name);
    }

    Ok(0)
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
