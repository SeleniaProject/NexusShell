//! `eval` builtin – re-evaluate the given arguments as a command line.
//! This implementation concatenates the arguments with spaces and invokes the
//! platform default shell (`sh -c` on Unix, `cmd /C` on Windows). In the future
//! this will integrate directly with NexusShell's parser/executor.

use anyhow::{anyhow, Result};
use std::process::Command;

pub fn eval_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    let cmdline = args.join(" ");

    #[cfg(unix)]
    let status = Command::new("sh").arg("-c").arg(&cmdline).status()?;

    #[cfg(windows)]
    let status = Command::new("cmd").arg("/C").arg(&cmdline).status()?;

    if !status.success() {
        return Err(anyhow!("eval: command exited with status {:?}", status.code()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eval_echo() {
        eval_cli(&["echo".into(), "ok".into()]).unwrap();
    }
} 