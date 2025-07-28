//! Process management abstraction for NexusShell HAL
//!
//! This module provides cross-platform process management capabilities
//! including process creation, monitoring, and control.

use crate::error::{HalError, HalResult};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Process identifier type
pub type ProcessId = u32;

/// Process group identifier type
pub type ProcessGroupId = u32;

/// Process information structure
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: ProcessId,
    pub parent_pid: Option<ProcessId>,
    pub name: String,
    pub command_line: String,
    pub start_time: std::time::SystemTime,
    pub cpu_time: std::time::Duration,
    pub memory_usage: u64,
    pub status: ProcessStatus,
}

/// Process status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    /// Process is running
    Running,
    /// Process is sleeping
    Sleeping,
    /// Process is stopped
    Stopped,
    /// Process is a zombie (finished but not reaped)
    Zombie,
    /// Process has exited
    Exited(i32),
    /// Process was terminated by signal
    Signaled(i32),
    /// Unknown status
    Unknown,
}

/// Process handle for managing spawned processes
#[derive(Debug)]
pub struct ProcessHandle {
    pub pid: u32,
    #[allow(dead_code)]
    start_time: std::time::Instant,
    info: ProcessInfo,
}

impl ProcessHandle {
    /// Create a new process handle
    pub fn new(child: Child, command: String) -> Self {
        let pid = child.id();
        let start_time = Instant::now();
        
        let info = ProcessInfo {
            pid,
            parent_pid: None,
            name: "unknown".to_string(),
            command_line: command,
            start_time: std::time::SystemTime::now(),
            cpu_time: std::time::Duration::ZERO,
            memory_usage: 0,
            status: ProcessStatus::Running,
        };

        Self {
            pid,
            start_time,
            info,
        }
    }

    /// Get process ID
    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    /// Get process information
    pub fn info(&self) -> &ProcessInfo {
        &self.info
    }

    /// Wait for process to complete
    pub fn wait(&mut self) -> HalResult<ExitStatus> {
        // This method needs to be implemented or removed if not used
        // For now, it will cause a compilation error
        unimplemented!("ProcessHandle::wait() needs implementation")
    }

    /// Try to wait for process (non-blocking)
    pub fn try_wait(&mut self) -> HalResult<Option<ExitStatus>> {
        // This method needs to be implemented or removed if not used
        // For now, it will cause a compilation error
        unimplemented!("ProcessHandle::try_wait() needs implementation")
    }

    /// Kill the process
    pub fn kill(&mut self) -> HalResult<()> {
        // This method needs to be implemented or removed if not used
        // For now, it will cause a compilation error
        unimplemented!("ProcessHandle::kill() needs implementation")
    }

    /// Send signal to process (Unix only)
    #[cfg(unix)]
    pub fn signal(&self, signal: i32) -> HalResult<()> {
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;

        let nix_signal = Signal::try_from(signal)
            .map_err(|e| HalError::invalid_input(&format!("Invalid signal: {}", e)))?;

        signal::kill(Pid::from_raw(self.pid as i32), nix_signal)
            .map_err(|e| HalError::process_error("kill", Some(self.pid), &format!("Failed to send signal: {}", e)))
    }

    /// Update process information
    pub fn update_info(&mut self) -> HalResult<()> {
        // Update process status and resource usage
        // This is a simplified implementation - real implementation would
        // query system for actual process information
        
        if let Ok(Some(exit_status)) = self.try_wait() {
            self.info.status = if let Some(code) = exit_status.code() {
                ProcessStatus::Exited(code)
            } else {
                // Process was terminated by signal
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    if let Some(signal) = exit_status.signal() {
                        ProcessStatus::Signaled(signal)
                    } else {
                        ProcessStatus::Unknown
                    }
                }
                #[cfg(not(unix))]
                {
                    ProcessStatus::Unknown
                }
            };
        }

        Ok(())
    }
}

/// Process manager for creating and managing processes
pub struct ProcessManager {
    /// Map of active processes
    processes: Arc<Mutex<HashMap<ProcessId, ProcessHandle>>>,
    /// Process creation statistics
    stats: ProcessStats,
}

/// Process management statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessStats {
    pub processes_created: u64,
    pub processes_completed: u64,
    pub processes_killed: u64,
    pub total_cpu_time: Duration,
    pub peak_memory_usage: u64,
}

impl ProcessManager {
    /// Create a new process manager
    pub fn new() -> HalResult<Self> {
        Ok(Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            stats: ProcessStats::default(),
        })
    }

    /// Spawn a new process
    pub fn spawn<S>(&mut self, program: S, args: &[S]) -> HalResult<ProcessHandle>
    where
        S: AsRef<OsStr> + std::fmt::Display,
    {
        let mut command = Command::new(program.as_ref());
        command.args(args);
        
        let child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
                return Err(HalError::process_error("spawn", None, &e.to_string()));
            }
        };

        let pid = child.id();
        let command_line = format!("{} {}", 
            program, 
            args.iter()
                .map(|s| s.as_ref().to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        );

        let _info = ProcessInfo {
            pid,
            parent_pid: None,
            name: program.to_string(),
            command_line: command_line.clone(),
            start_time: std::time::SystemTime::now(),
            cpu_time: std::time::Duration::ZERO,
            memory_usage: 0,
            status: ProcessStatus::Running,
        };

        let _handle = ProcessHandle::new(child, command_line);
        let pid = _handle.pid();

        // Store the process handle
        {
            let mut processes = self.processes.lock()
                .map_err(|_| HalError::resource_error("Process map lock poisoned"))?;
            processes.insert(pid, _handle);
        }

        self.stats.processes_created += 1;

        // Return a reference to the stored handle
        let processes = self.processes.lock()
            .map_err(|_| HalError::resource_error("Process map lock poisoned"))?;
        let _handle = processes.get(&pid)
            .ok_or_else(|| HalError::process_error("get_process", None, "Process handle disappeared"))?;
        
        // We need to return an owned handle, so we'll create a new one
        // This is a limitation of the current design
        Ok(ProcessHandle::new(
            Command::new("echo").spawn().unwrap(), // Placeholder - this needs redesign
            "placeholder".to_string()
        ))
    }

    /// Spawn a process with custom configuration
    pub fn spawn_with_config<S>(&mut self, config: ProcessConfig<S>) -> HalResult<ProcessHandle>
    where
        S: AsRef<OsStr>,
    {
        let mut command = Command::new(&config.program);
        command.args(&config.args);

        // Set working directory
        if let Some(ref cwd) = config.working_dir {
            command.current_dir(cwd);
        }

        // Set environment variables
        for (key, value) in &config.env {
            command.env(key, value);
        }

        // Set up stdio
        if let Some(stdin) = config.stdin {
            command.stdin(stdin);
        }
        if let Some(stdout) = config.stdout {
            command.stdout(stdout);
        }
        if let Some(stderr) = config.stderr {
            command.stderr(stderr);
        }

        let child = command.spawn()
            .map_err(|e| HalError::process_error("spawn", None, &format!("Failed to spawn process: {}", e)))?;

        let command_line = format!("{} {}", 
            config.program.as_ref().to_string_lossy(),
            config.args.iter()
                .map(|s| s.as_ref().to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        );

        let handle = ProcessHandle::new(child, command_line);
        self.stats.processes_created += 1;

        Ok(handle)
    }

    /// Get process information by PID
    pub fn get_process_info(&self, pid: ProcessId) -> HalResult<Option<ProcessInfo>> {
        let processes = self.processes.lock()
            .map_err(|_| HalError::resource_error("Process map lock poisoned"))?;
        
        Ok(processes.get(&pid).map(|handle| handle.info().clone()))
    }

    /// List all managed processes
    pub fn list_processes(&self) -> HalResult<Vec<ProcessInfo>> {
        let processes = self.processes.lock()
            .map_err(|_| HalError::resource_error("Process map lock poisoned"))?;
        
        Ok(processes.values().map(|handle| handle.info().clone()).collect())
    }

    /// Kill a process by PID
    pub fn kill_process(&mut self, pid: ProcessId) -> HalResult<()> {
        let mut processes = self.processes.lock()
            .map_err(|_| HalError::resource_error("Process map lock poisoned"))?;
        
        if let Some(handle) = processes.get_mut(&pid) {
            handle.kill()?;
            self.stats.processes_killed += 1;
            Ok(())
        } else {
            Err(HalError::process_error("get_process", Some(pid), &format!("Process {} not found", pid)))
        }
    }

    /// Wait for a process to complete
    pub fn wait_for_process(&mut self, pid: ProcessId) -> HalResult<Option<ExitStatus>> {
        let mut processes = self.processes.lock()
            .map_err(|_| HalError::resource_error("Process map lock poisoned"))?;
        
        if let Some(handle) = processes.get_mut(&pid) {
            let status = handle.wait()?;
            self.stats.processes_completed += 1;
            Ok(Some(status))
        } else {
            Ok(None)
        }
    }

    /// Clean up finished processes
    pub fn cleanup_finished(&mut self) -> HalResult<()> {
        let mut processes = self.processes.lock()
            .map_err(|_| HalError::resource_error("Process map lock poisoned"))?;
        
        let mut to_remove = Vec::new();
        
        for (pid, handle) in processes.iter_mut() {
            if let Ok(Some(_)) = handle.try_wait() {
                to_remove.push(*pid);
            }
        }
        
        for pid in to_remove {
            processes.remove(&pid);
        }
        
        Ok(())
    }

    /// Get process management statistics
    pub fn get_stats(&self) -> &ProcessStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = ProcessStats::default();
    }

    /// Send signal to process group (Unix only)
    #[cfg(unix)]
    pub fn signal_process_group(&self, pgid: ProcessGroupId, signal: i32) -> HalResult<()> {
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;

        let nix_signal = Signal::try_from(signal)
            .map_err(|e| HalError::invalid_input(&format!("Invalid signal: {}", e)))?;

        signal::killpg(Pid::from_raw(pgid as i32), nix_signal)
            .map_err(|e| HalError::process_error("killpg", Some(pgid), &format!("Failed to send signal to process group: {}", e)))
    }

    /// Get system process information (all processes)
    pub fn get_system_processes(&self) -> HalResult<Vec<ProcessInfo>> {
        // This would query the system for all running processes
        // For now, return empty list as this requires platform-specific implementation
        Ok(Vec::new())
    }
}

/// Process configuration for spawning
pub struct ProcessConfig<S>
where
    S: AsRef<OsStr>,
{
    /// Program to execute
    pub program: S,
    /// Command line arguments
    pub args: Vec<S>,
    /// Working directory
    pub working_dir: Option<std::path::PathBuf>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Standard input configuration
    pub stdin: Option<Stdio>,
    /// Standard output configuration
    pub stdout: Option<Stdio>,
    /// Standard error configuration
    pub stderr: Option<Stdio>,
}

impl<S> ProcessConfig<S>
where
    S: AsRef<OsStr>,
{
    /// Create a new process configuration
    pub fn new(program: S) -> Self {
        Self {
            program,
            args: Vec::new(),
            working_dir: None,
            env: HashMap::new(),
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    /// Add an argument
    pub fn arg(mut self, arg: S) -> Self {
        self.args.push(arg);
        self
    }

    /// Add multiple arguments
    pub fn args<I>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
    {
        self.args.extend(args);
        self
    }

    /// Set working directory
    pub fn current_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.working_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Set environment variable
    pub fn env<K, V>(mut self, key: K, val: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.env.insert(key.into(), val.into());
        self
    }

    /// Set multiple environment variables
    pub fn envs<I, K, V>(mut self, vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (key, val) in vars {
            self.env.insert(key.into(), val.into());
        }
        self
    }

    /// Set stdin
    pub fn stdin(mut self, stdin: Stdio) -> Self {
        self.stdin = Some(stdin);
        self
    }

    /// Set stdout
    pub fn stdout(mut self, stdout: Stdio) -> Self {
        self.stdout = Some(stdout);
        self
    }

    /// Set stderr
    pub fn stderr(mut self, stderr: Stdio) -> Self {
        self.stderr = Some(stderr);
        self
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default ProcessManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_manager_creation() {
        let manager = ProcessManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_process_config() {
        let config = ProcessConfig::new("echo")
            .arg("hello")
            .arg("world")
            .env("TEST", "value");
        
        assert_eq!(config.args.len(), 2);
        assert_eq!(config.env.get("TEST"), Some(&"value".to_string()));
    }
} 