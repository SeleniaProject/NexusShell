use anyhow::Result;
use ansi_term::Colour;
use nxsh_core::context::ShellContext;

/// Handle `export` builtin.
/// - `export NAME=VALUE` to set env variable
/// - `export -p` to print all env vars with colorized output
pub fn export_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    if args.is_empty() || args[0] == "-p" {
        for (k, v) in ctx.env.iter() {
            println!("{}={}", Colour::Cyan.paint(k.key()), v.value());
        }
        return Ok(());
    }

    for arg in args {
        if let Some((name, value)) = arg.split_once('=') {
            ctx.set_var(name, value);
        } else {
            println!("invalid export argument: {}", arg);
        }
    }
    Ok(())
} 