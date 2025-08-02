//! `popd` builtin  Epop directory stack and change to new top.
//! Usage: popd

use anyhow::{anyhow, Result};
use std::env;
use crate::dirs::{popd as stack_pop, dirs_cli};
use nxsh_core::context::ShellContext;

pub fn popd_cli(_args: &[String], ctx: &ShellContext) -> Result<()> {
    let popped = stack_pop().ok_or_else(|| anyhow!("popd: directory stack empty"))?;
    let new_top = {
        use crate::dirs::DIR_STACK;
        DIR_STACK.lock().unwrap().last().cloned().ok_or_else(|| anyhow!("popd: stack became empty"))?
    };
    env::set_current_dir(&new_top)?;
    ctx.set_var("PWD", new_top.to_string_lossy());
    dirs_cli(&[])?;
    Ok(())
} 
