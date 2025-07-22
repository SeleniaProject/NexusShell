//! `read` builtin – read a line from standard input into variable.
//! Usage: read [-r] VAR
//!   -r : do not treat backslash as escape, raw read (default raw)
//!
//! If VAR is omitted, read into variable `REPLY`.

use anyhow::{anyhow, Result};
use nxsh_core::context::ShellContext;
use std::io::{self, Read};

pub fn read_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    let mut raw = true; // default raw (bash uses -r to disable escapes but we ignore escapes anyway)
    let mut idx = 0;
    if !args.is_empty() && args[0] == "-r" { raw = true; idx = 1; }
    let var = args.get(idx).map(|s| s.as_str()).unwrap_or("REPLY");

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    if !raw {
        // future escape handling
    }
    if buffer.ends_with('\n') { buffer.pop(); if buffer.ends_with('\r'){buffer.pop();} }
    ctx.set_var(var, buffer);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
} 