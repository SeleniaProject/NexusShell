use anyhow::Result;
use nxsh_core::context::ShellContext;
use crate::common::{BuiltinResult, BuiltinContext};

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

/// Execute the set builtin command
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    if args.is_empty() {
        // Print all environment variables if no arguments
        for (key, value) in std::env::vars() {
            println!("{}={}", key, value);
        }
        return Ok(0);
    }

    // Handle shell options
    for arg in args {
        match arg.as_str() {
            "-e" => {
                eprintln!("set: -e (errexit) option is not implemented in this context");
            }
            "+e" => {
                eprintln!("set: +e (no errexit) option is not implemented in this context");
            }
            "-x" => {
                eprintln!("set: -x (xtrace) option is not implemented in this context");
            }
            "+x" => {
                eprintln!("set: +x (no xtrace) option is not implemented in this context");
            }
            "-o" => {
                eprintln!("set: -o option requires an argument");
            }
            "+o" => {
                eprintln!("set: +o option requires an argument");
            }
            _ if arg.starts_with("-o") => {
                let option = &arg[2..];
                eprintln!("set: -o {} option is not implemented", option);
            }
            _ if arg.starts_with("+o") => {
                let option = &arg[2..];
                eprintln!("set: +o {} option is not implemented", option);
            }
            _ => {
                eprintln!("set: invalid option '{}'", arg);
                return Ok(1);
            }
        }
    }

    Ok(0)
} 

