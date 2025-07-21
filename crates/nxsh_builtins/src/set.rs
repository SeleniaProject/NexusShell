use anyhow::Result;
use nxsh_core::context::{ShellContext, ShellOptions};

/// Handle `set` builtin for flags -e, -x, -o pipefail.
pub fn set_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    if args.is_empty() {
        let opts = ctx.get_options();
        println!("-e {}", opts.errexit);
        println!("-x {}", opts.xtrace);
        println!("pipefail {}", opts.pipefail);
        return Ok(());
    }

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-e" => ctx.set_option(|o| o.errexit = true),
            "+e" => ctx.set_option(|o| o.errexit = false),
            "-x" => ctx.set_option(|o| o.xtrace = true),
            "+x" => ctx.set_option(|o| o.xtrace = false),
            "-o" => {
                if let Some(name) = iter.next() {
                    if name == "pipefail" {
                        ctx.set_option(|o| o.pipefail = true);
                    }
                }
            }
            "+o" => {
                if let Some(name) = iter.next() {
                    if name == "pipefail" {
                        ctx.set_option(|o| o.pipefail = false);
                    }
                }
            }
            _ => println!("unknown option {}", arg),
        }
    }
    Ok(())
} 