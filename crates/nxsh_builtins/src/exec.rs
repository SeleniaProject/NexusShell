//! `exec` builtin  Ereplace the current shell process with the specified command.
//! Usage: `exec CMD [ARGS...]`
//! On Unix this calls `nix::unistd::execvp`. On Windows it spawns the process
//! and exits with its status (best-effort emulation).

use anyhow::{anyhow, Result};

pub fn exec_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("exec: missing command"));
    }
    #[cfg(unix)]
    {
        use nix::unistd::{execvp, ForkResult};
        use nix::unistd::fork;
        use std::ffi::CString;

        let c_cmd = CString::new(args[0].as_str())?;
        let c_args: Vec<CString> = args.iter().map(|s| CString::new(s.as_str()).unwrap()).collect();
        // Replace process image
        execvp(&c_cmd, &c_args)?;
        unreachable!();
    }
    #[cfg(windows)]
    {
        use std::process::Command;
        let status = Command::new(&args[0]).args(&args[1..]).status()?;
        std::process::exit(status.code().unwrap_or(1));
    }
} 
