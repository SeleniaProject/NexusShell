use std::process::Command;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use anyhow::anyhow;
use nxsh_core::{
    context::ShellContext,
    error::{ErrorKind, RuntimeErrorKind, ShellError, ShellResult},
    ExecutionResult,
};

#[derive(Debug, Clone)]
pub struct NohupOptions {
    pub command: String,
    pub args: Vec<String>,
    pub output_file: Option<String>,
}

pub fn nohup(_ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
    if args.is_empty() {
        return Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            "nohup: missing command",
        ));
    }

    let options = parse_nohup_args(args)?;
    execute_nohup(&options)
}

fn parse_nohup_args(args: &[String]) -> ShellResult<NohupOptions> {
    if args.is_empty() {
        return Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            "nohup: missing command",
        ));
    }

    let options = NohupOptions {
        command: args[0].clone(),
        args: args[1..].to_vec(),
        output_file: None,
    };

    Ok(options)
}

fn execute_nohup(options: &NohupOptions) -> ShellResult<ExecutionResult> {
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        #[cfg(unix)]
        use std::os::unix::io::{AsRawFd, FromRawFd};

        let output_file = options.output_file.as_deref().unwrap_or("nohup.out");
        
        // Open or create the output file
        let file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(output_file) 
        {
            Ok(f) => f,
            Err(e) => {
                return Err(ShellError::new(
                    ErrorKind::PermissionDenied,
                    &format!("nohup: cannot open '{}': {}", output_file, e),
                    "",
                    0,
                ));
            }
        };

        let file_fd = file.as_raw_fd();

        let mut cmd = Command::new(&options.command);
        cmd.args(&options.args);
        
        // Redirect stdout and stderr to the output file
        unsafe {
            cmd.stdout(Stdio::from_raw_fd(file_fd));
            cmd.stderr(Stdio::from_raw_fd(file_fd));
        }
        
        // Make the process ignore SIGHUP
        unsafe {
            cmd.pre_exec(|| {
                libc::signal(libc::SIGHUP, libc::SIG_IGN);
                Ok(())
            });
        }

        println!("nohup: ignoring input and appending output to '{}'", output_file);

        match cmd.spawn() {
            Ok(child) => {
                println!("nohup: process started with PID {}", child.id());
                Ok(ExecutionResult::success(0))
            }
            Err(e) => Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::ExitError),
                &format!("nohup: failed to execute '{}': {}", options.command, e),
                "",
                0,
            )),
        }
    }

    #[cfg(windows)]
    {
        // Windows implementation using job objects or similar
        let mut cmd = Command::new(&options.command);
        cmd.args(&options.args);
        
    #[cfg(windows)]
    {
        // On Windows, we can't ignore SIGHUP in the same way
        // but we can use process creation flags
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x00000008); // DETACHED_PROCESS

        println!("nohup: starting detached process (Windows)");

        match cmd.spawn() {
            Ok(child) => {
                println!("nohup: process started with PID {}", child.id());
                Ok(ExecutionResult::success(0))
            }
            Err(e) => Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                format!("nohup: failed to execute '{}': {}", options.command, e),
            )),
        }
    }
    }
    
    #[cfg(unix)]
    {
        // Unix implementation would use signal handling
        println!("nohup: Unix signal handling not implemented yet");
        Ok(ExecutionResult::success(0))
    }
}

/// CLI wrapper function for nohup command
pub fn nohup_cli(args: &[String]) -> anyhow::Result<()> {
    let mut ctx = nxsh_core::context::ShellContext::new();
    match nohup(&mut ctx, args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("nohup command failed: {}", e)),
    }
}
