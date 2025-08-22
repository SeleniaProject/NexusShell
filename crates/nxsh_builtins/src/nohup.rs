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

    let mut i = 0;
    let mut output_file: Option<String> = None;
    let mut cmd: Option<String> = None;
    let mut cmd_args: Vec<String> = Vec::new();
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                if i >= args.len() { return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "nohup: -o requires FILE")); }
                output_file = Some(args[i].clone());
            }
            arg if arg.starts_with('-') => {
                // Unknown option
            }
            _ => {
                cmd = Some(args[i].clone());
                cmd_args.extend(args[i+1..].iter().cloned());
                break;
            }
        }
        i += 1;
    }

    let command = cmd.ok_or_else(|| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "nohup: missing command"))?;
    Ok(NohupOptions { command, args: cmd_args, output_file })
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
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x00000008); // DETACHED_PROCESS

        // Optional output redirection on Windows: create or append to nohup.out if specified
        if let Some(of) = options.output_file.as_deref() {
            use std::fs::OpenOptions;
            use std::os::windows::io::{AsRawHandle, FromRawHandle};
            let file = OpenOptions::new().create(true).append(true).open(of)
                .map_err(|_e| ShellError::permission_denied(of))?;
            let handle = file.as_raw_handle();
            unsafe {
                use std::process::Stdio;
                cmd.stdout(Stdio::from_raw_handle(handle));
                cmd.stderr(Stdio::from_raw_handle(handle));
            }
        }

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

/// CLI wrapper function for nohup command
pub fn nohup_cli(args: &[String]) -> anyhow::Result<()> {
    let mut ctx = nxsh_core::context::ShellContext::new();
    match nohup(&mut ctx, args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("nohup command failed: {}", e)),
    }
}


/// Execute function stub
pub fn execute(_args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}
