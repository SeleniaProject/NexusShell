//! Command execution and management
//!
//! This module provides abstractions for command execution across different platforms.

use crate::error::{HalError, HalResult};
use std::process::{Command as StdCommand, Stdio, ExitStatus};
use std::ffi::OsStr;
use std::path::Path;

/// Command builder and executor
pub struct Command {
    inner: StdCommand,
}

impl Command {
    /// Create a new command with the given program
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Self {
            inner: StdCommand::new(program),
        }
    }

    /// Add an argument to the command
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    /// Add multiple arguments to the command
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.inner.args(args);
        self
    }

    /// Set the working directory for the command
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.inner.current_dir(dir);
        self
    }

    /// Set environment variable for the command
    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.inner.env(key, val);
        self
    }

    /// Set stdin configuration
    pub fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.inner.stdin(cfg);
        self
    }

    /// Set stdout configuration
    pub fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.inner.stdout(cfg);
        self
    }

    /// Set stderr configuration
    pub fn stderr<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.inner.stderr(cfg);
        self
    }

    /// Execute the command and wait for completion
    pub fn status(&mut self) -> HalResult<ExitStatus> {
        self.inner.status()
            .map_err(|e| HalError::Io(crate::error::IoError {
                operation: "status".to_string(),
                path: None,
                kind: e.kind(),
                message: e.to_string(),
            }))
    }

    /// Execute the command and capture output
    pub fn output(&mut self) -> HalResult<std::process::Output> {
        self.inner.output()
            .map_err(|e| HalError::Io(crate::error::IoError {
                operation: "output".to_string(),
                path: None,
                kind: e.kind(),
                message: e.to_string(),
            }))
    }

    /// Spawn the command without waiting
    pub fn spawn(&mut self) -> HalResult<std::process::Child> {
        self.inner.spawn()
            .map_err(|e| HalError::Io(crate::error::IoError {
                operation: "spawn".to_string(),
                path: None,
                kind: e.kind(),
                message: e.to_string(),
            }))
    }
}

/// Command execution result
pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl CommandResult {
    /// Create a new command result
    pub fn new(exit_code: i32, stdout: Vec<u8>, stderr: Vec<u8>) -> Self {
        Self {
            exit_code,
            stdout,
            stderr,
        }
    }

    /// Create a successful result with output
    pub fn success_with_output(output: String) -> Self {
        Self::new(0, output.into_bytes(), Vec::new())
    }

    /// Check if the command was successful
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Get stdout as string
    pub fn stdout_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.stdout.clone())
    }

    /// Get stderr as string
    pub fn stderr_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.stderr.clone())
    }
}

/// Execute a command with arguments
pub fn execute<P, I, S>(program: P, args: I) -> HalResult<CommandResult>
where
    P: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(program)
        .args(args)
        .output()?;

    Ok(CommandResult::new(
        output.status.code().unwrap_or(-1),
        output.stdout,
        output.stderr,
    ))
}

/// Execute a command in the background
pub fn execute_background<P, I, S>(program: P, args: I) -> HalResult<std::process::Child>
where
    P: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(program)
        .args(args)
        .spawn()
}

/// Check if a command exists in PATH
pub fn command_exists<S: AsRef<OsStr>>(command: S) -> bool {
    let command_str = command.as_ref().to_string_lossy();
    
    #[cfg(windows)]
    {
        // On Windows, try both with and without .exe extension
        let commands = if command_str.ends_with(".exe") {
            vec![command_str.to_string()]
        } else {
            vec![command_str.to_string(), format!("{}.exe", command_str)]
        };
        
        for cmd in commands {
            if let Ok(output) = StdCommand::new("where")
                .arg(&cmd)
                .output()
            {
                if output.status.success() {
                    return true;
                }
            }
        }
        false
    }
    
    #[cfg(not(windows))]
    {
        if let Ok(output) = StdCommand::new("which")
            .arg(command)
            .output()
        {
            output.status.success()
        } else {
            false
        }
    }
}
