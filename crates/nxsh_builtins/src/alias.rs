use anyhow::Result;
use nxsh_core::context::ShellContext;

/// Handle `alias` builtin.
/// - `alias NAME=VALUE` to set
/// - `alias -p` to print all
pub fn alias_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    if args.is_empty() || args[0] == "-p" {
        for (k, v) in ctx.list_aliases() {
            println!("alias {}='{}'", k, v);
        }
        return Ok(());
    }
    // parse NAME=VALUE pairs
    for arg in args {
        if let Some((name, value)) = arg.split_once('=') {
            ctx.set_alias(name, value)?;
        } else {
            println!("invalid alias argument: {}", arg);
        }
    }
    Ok(())
} 