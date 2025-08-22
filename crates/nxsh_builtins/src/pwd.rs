//! `pwd` builtin  Eprint current working directory.
//! Supports options:
//!   -L : logical path from $PWD (default)
//!   -P : physical path with symlink resolution

use anyhow::Result;
use nxsh_core::context::ShellContext;
use std::env;
use super::ui_design::{Colorize, ColorPalette, Icons};

pub fn pwd_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    let mut physical = false;
    if !args.is_empty() {
        match args[0].as_str() {
            "-P" => physical = true,
            "-L" => physical = false,
            _ => {}
        }
    }
    let path = if physical {
        env::current_dir()?
    } else {
        ctx.get_var("PWD").map(|s| s.into()).unwrap_or(env::current_dir()?)
    };
    
    let palette = ColorPalette::new();
    println!("{} {}", 
        Icons::FOLDER, 
        path.display().to_string().colorize(&palette.success)
    );
    Ok(())
}

/// Execute the pwd builtin command
pub fn execute(args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    let mut physical = false;
    
    for arg in args {
        match arg.as_str() {
            "-P" => physical = true,
            "-L" => physical = false,
            "-h" | "--help" => {
                println!("Usage: pwd [OPTION]...");
                println!("Print the full filename of the current working directory.");
                println!();
                println!("Options:");
                println!("  -L     use PWD from environment, even if it contains symlinks");
                println!("  -P     avoid all symlinks");
                println!("  --help display this help and exit");
                return Ok(0);
            }
            _ if arg.starts_with('-') => {
                eprintln!("pwd: invalid option '{}'", arg);
                return Ok(1);
            }
            _ => {
                eprintln!("pwd: too many arguments");
                return Ok(1);
            }
        }
    }

    // Get current directory
    let path = if physical {
        match std::env::current_dir() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("❌ pwd error: {}", e);
                return Ok(1);
            }
        }
    } else {
        // Try to get PWD from environment first
        match std::env::var("PWD") {
            Ok(pwd_str) => std::path::PathBuf::from(pwd_str),
            Err(_) => match std::env::current_dir() {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("❌ pwd error: {}", e);
                    return Ok(1);
                }
            }
        }
    };

    // Stylish output with cyberpunk theme colors
    let cyan = "\x1b[38;2;0;245;255m";     // #00f5ff - cyberpunk cyan
    let purple = "\x1b[38;2;153;69;255m";  // #9945ff - deep purple
    let reset = "\x1b[0m";
    
    // Format path components with alternating colors
    let path_str = path.display().to_string();
    let components: Vec<&str> = path_str.split(['/', '\\']).collect();
    
    print!("📁 ");
    for (i, component) in components.iter().enumerate() {
        if !component.is_empty() {
            let color = if i % 2 == 0 { cyan } else { purple };
            print!("{}{}{}", color, component, reset);
            if i < components.len() - 1 && !component.is_empty() {
                print!("{}/{}", purple, reset);
            }
        }
    }
    println!();
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_pwd() {
        let ctx = ShellContext::new();
        pwd_cli(&[], &ctx).unwrap();
    }
} 

