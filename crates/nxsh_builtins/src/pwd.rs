//! `pwd` builtin  Eprint current working directory.
//! Supports options:
//!   -L : logical path from $PWD (default)
//!   -P : physical path with symlink resolution

use anyhow::Result;
use nxsh_core::context::ShellContext;
use std::env;
use super::ui_design::{Colorize, TableFormatter, ColorPalette, Icons};

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
    
    println!("{} {}", 
        Icons::FOLDER, 
        path.display().to_string().colorize(&ColorPalette::SUCCESS)
    );
    Ok(())
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
