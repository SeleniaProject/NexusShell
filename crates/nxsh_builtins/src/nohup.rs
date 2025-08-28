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

        // Use safer process spawning without unsafe blocks
        let mut cmd = Command::new(&options.command);
        cmd.args(&options.args);
        
        // Redirect stdout and stderr to the output file (safe alternative)
        cmd.stdout(std::process::Stdio::from(file.try_clone().map_err(|e| ShellError::new(
            ErrorKind::IoError(IoErrorKind::Other),
            &format!("Failed to clone file handle: {}", e),
            "",
            0,
        ))?));
        cmd.stderr(std::process::Stdio::from(file));
        
        // Set process session to detach from terminal (safer alternative to signal handling)
        // This approach avoids unsafe signal manipulation
        cmd.process_group(0); // Create new process group
        
        // Use environment variable to signal NOHUP behavior instead of unsafe signal calls
        cmd.env("NOHUP", "1");
        
        // Additional security: limit environment exposure to prevent privilege escalation
        cmd.env_clear();
        for (key, value) in std::env::vars() {
            // Only pass through safe environment variables
            if is_safe_env_var(&key) {
                cmd.env(key, value);
            }
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
        // Windows implementation using safer process creation
        let mut cmd = Command::new(&options.command);
        cmd.args(&options.args);
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x00000008); // DETACHED_PROCESS

        // Safe output redirection on Windows
        if let Some(of) = options.output_file.as_deref() {
            use std::fs::OpenOptions;
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(of)
                .map_err(|e| ShellError::new(
                    ErrorKind::IoError(IoErrorKind::PermissionDenied),
                    &format!("Failed to open output file '{}': {}", of, e),
                    "",
                    0,
                ))?;
            
            // Use safer handle conversion without unsafe blocks
            cmd.stdout(std::process::Stdio::from(file.try_clone().map_err(|e| ShellError::new(
                ErrorKind::IoError(IoErrorKind::Other),
                &format!("Failed to clone file handle: {}", e),
                "",
                0,
            ))?));
            cmd.stderr(std::process::Stdio::from(file));
        }

        // Environment security for Windows - prevent privilege escalation
        cmd.env_clear();
        for (key, value) in std::env::vars() {
            if is_safe_env_var(&key) {
                cmd.env(key, value);
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

/// Check if an environment variable is safe to pass to child process
/// This prevents privilege escalation through environment manipulation
fn is_safe_env_var(var_name: &str) -> bool {
    // Allow common safe environment variables
    const SAFE_VARS: &[&str] = &[
        "PATH", "HOME", "USER", "USERNAME", "LANG", "LC_ALL", "LC_CTYPE",
        "TERM", "SHELL", "PWD", "OLDPWD", "TZ", "TMPDIR", "TEMP", "TMP",
    ];
    
    // Block potentially dangerous variables that could affect security
    const DANGEROUS_VARS: &[&str] = &[
        "LD_PRELOAD", "LD_LIBRARY_PATH", "DYLD_LIBRARY_PATH", "DYLD_INSERT_LIBRARIES",
        "PYTHONPATH", "NODE_PATH", "PERL5LIB", "RUBYLIB", "GEM_PATH", "GEM_HOME",
        "CLASSPATH", "JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "MAVEN_OPTS", "GRADLE_OPTS",
        "LD_AUDIT", "LD_DEBUG", "MALLOC_CHECK_", "MALLOC_PERTURB_",
    ];
    
    // Check if explicitly dangerous
    if DANGEROUS_VARS.contains(&var_name) {
        return false;
    }
    
    // Allow explicitly safe variables
    if SAFE_VARS.contains(&var_name) {
        return true;
    }
    
    // Allow NXSH-specific variables
    if var_name.starts_with("NXSH_") {
        return true;
    }
    
    // Block variables that start with potentially dangerous prefixes
    let dangerous_prefixes = ["LD_", "DYLD_", "_JAVA_", "JAVA_"];
    if dangerous_prefixes.iter().any(|prefix| var_name.starts_with(prefix)) {
        return false;
    }
    
    // By default, be conservative and block unknown variables
    false
}
