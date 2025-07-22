//! `getopts` builtin – POSIX argument parsing helper.
//! Syntax: getopts OPTSTRING NAME [ARGS...]
//! For each option parsed, sets variable NAME to the option character and prints index.
//! If option requires an argument (indicated by ':' in OPTSTRING), the argument is stored in
//! variable `OPTARG` in the shell context.
//! On end of options, returns with status 1.

use anyhow::{anyhow, Result};
use nxsh_core::context::ShellContext;

pub fn getopts_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    if args.len() < 2 {
        return Err(anyhow!("getopts: missing arguments"));
    }
    let optstr = &args[0];
    let name = &args[1];
    let mut argv: Vec<String> = if args.len() > 2 { args[2..].to_vec() } else { vec![] };

    // Pointer index variable maintains current index in argv within context ($OPTIND style)
    let mut optind: usize = ctx
        .get_var("OPTIND")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);

    if optind == 0 { optind = 1; }
    if optind > argv.len() { return Ok(()); }

    let current = &argv[optind - 1];
    if !current.starts_with('-') || current == "-" { return Ok(()); }
    if current == "--" { return Ok(()); }

    let opt_char = current.chars().nth(1).unwrap();
    if !optstr.contains(opt_char) {
        ctx.set_var(name, "?".to_string());
        ctx.set_var("OPTARG", opt_char.to_string());
        return Err(anyhow!("illegal option -- {}", opt_char));
    }

    ctx.set_var(name, opt_char.to_string());
    // option argument if required
    if optstr.contains(&format!("{}:", opt_char)) {
        if current.len() > 2 {
            ctx.set_var("OPTARG", current[2..].to_string());
        } else if optind < argv.len() {
            ctx.set_var("OPTARG", argv[optind].clone());
            optind += 1;
        } else {
            return Err(anyhow!("option requires an argument -- {}", opt_char));
        }
    } else {
        ctx.set_var("OPTARG", String::new());
    }
    optind += 1;
    ctx.set_var("OPTIND", optind.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_getopts() {
        let ctx = ShellContext::new();
        let args = vec!["ab:".into(), "opt".into(), "-a".into(), "-b".into(), "val".into()];
        getopts_cli(&args, &ctx).unwrap();
        assert_eq!(ctx.get_var("opt").unwrap(), "a");
    }
} 