//! `pushd` builtin  Epush directory onto stack and switch to it.
//! Usage: pushd [DIR]
//! Without DIR, swaps the top two directories.

use anyhow::{anyhow, Result};
use dirs_next::home_dir;
use std::env;
use std::path::PathBuf;
use crate::dirs::{pushd as stack_push, popd as stack_pop, dirs_cli};
use nxsh_core::context::ShellContext;

pub fn pushd_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    let target: PathBuf;
    if args.is_empty() {
        // Swap top two
        let first = stack_pop().ok_or_else(|| anyhow!("pushd: directory stack empty"))?;
        let second = stack_pop().ok_or_else(|| anyhow!("pushd: not enough entries"))?;
        // push back in opposite order
        stack_push(first.clone());
        stack_push(second.clone());
        target = second;
    } else {
        let dir_arg = &args[0];
        if dir_arg == "~" {
            target = home_dir().ok_or_else(|| anyhow!("HOME not set"))?;
        } else {
            target = PathBuf::from(dir_arg);
        }
        stack_push(target.clone());
    }
    env::set_current_dir(&target)?;
    ctx.set_var("PWD", target.to_string_lossy());
    dirs_cli(&[])?; // print stack
    Ok(())
} 
