use anyhow::Result;
use nxsh_core::context::{ShellContext, ShellOptions};

/// Handle `set` builtin for flags -e, -x, -o pipefail.
pub fn set_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    if args.is_empty() {
        if let Ok(opts_guard) = ctx.options.read() {
            println!("-e {}", opts_guard.errexit);
            println!("-x {}", opts_guard.xtrace);
            println!("pipefail {}", opts_guard.pipefail);
        }
        return Ok(());
    }

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-e" => {
                if let Ok(mut opts_guard) = ctx.options.write() {
                    opts_guard.errexit = true;
                }
            },
            "+e" => {
                if let Ok(mut opts_guard) = ctx.options.write() {
                    opts_guard.errexit = false;
                }
            },
            "-x" => {
                if let Ok(mut opts_guard) = ctx.options.write() {
                    opts_guard.xtrace = true;
                }
            },
            "+x" => {
                if let Ok(mut opts_guard) = ctx.options.write() {
                    opts_guard.xtrace = false;
                }
            },
            "-o" => {
                if let Some(name) = iter.next() {
                    if name == "pipefail" {
                        if let Ok(mut opts_guard) = ctx.options.write() {
                            opts_guard.pipefail = true;
                        }
                    }
                }
            }
            "+o" => {
                if let Some(name) = iter.next() {
                    if name == "pipefail" {
                        if let Ok(mut opts_guard) = ctx.options.write() {
                            opts_guard.pipefail = false;
                        }
                    }
                }
            }
            _ => println!("unknown option {}", arg),
        }
    }
    Ok(())
} 
