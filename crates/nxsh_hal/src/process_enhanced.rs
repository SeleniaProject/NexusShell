use anyhow::Result;
use std::{
    collections::HashMap,
    io::Write, // write_all/flush に必要
    process::{Child, Command, Stdio},
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

/// Advanced process management and monitoring
#[derive(Debug)]
pub struct ProcessMonitor {
    processes: Arc<RwLock<HashMap<u32, ProcessInfo>>>,
    stats: Arc<RwLock<ProcessStats>>,
}

impl Default for ProcessMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessMonitor {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ProcessStats::default())),
        }
    }

    /// Register a process for monitoring
    pub fn register_process(&self, pid: u32, info: ProcessInfo) {
        if let Ok(mut processes) = self.processes.write() {
            processes.insert(pid, info);
        }

        if let Ok(mut stats) = self.stats.write() {
            stats.total_processes += 1;
        }
    }

    /// Unregister a process
    pub fn unregister_process(&self, pid: u32) -> Option<ProcessInfo> {
        if let Ok(mut processes) = self.processes.write() {
            return processes.remove(&pid);
        }
        None
    }

    /// Get process information
    pub fn get_process(&self, pid: u32) -> Option<ProcessInfo> {
        if let Ok(processes) = self.processes.read() {
            return processes.get(&pid).cloned();
        }
        None
    }

    /// List all monitored processes
    pub fn list_processes(&self) -> Vec<(u32, ProcessInfo)> {
        if let Ok(processes) = self.processes.read() {
            return processes
                .iter()
                .map(|(&pid, info)| (pid, info.clone()))
                .collect();
        }
        Vec::new()
    }

    /// Get process statistics
    pub fn stats(&self) -> ProcessStats {
        if let Ok(stats) = self.stats.read() {
            stats.clone()
        } else {
            ProcessStats::default()
        }
    }

    /// Update process statistics
    pub fn record_execution(&self, duration: Duration, exit_code: i32) {
        if let Ok(mut stats) = self.stats.write() {
            stats.executions += 1;
            stats.total_execution_time += duration;

            if exit_code == 0 {
                stats.successful_executions += 1;
            } else {
                stats.failed_executions += 1;
            }

            if duration < stats.fastest_execution || stats.fastest_execution == Duration::ZERO {
                stats.fastest_execution = duration;
            }

            if duration > stats.slowest_execution {
                stats.slowest_execution = duration;
            }
        }
    }

    /// Kill process by PID
    pub fn kill_process(&self, _pid: u32, _signal: ProcessSignal) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            let signal_num = match _signal {
                ProcessSignal::Term => 15,
                ProcessSignal::Kill => 9,
                ProcessSignal::Int => 2,
                ProcessSignal::Quit => 3,
                ProcessSignal::Stop => 19,
                ProcessSignal::Cont => 18,
            };

            unsafe {
                if libc::kill(_pid as libc::pid_t, signal_num) != 0 {
                    return Err(anyhow::anyhow!(
                        "Failed to send signal {} to process {}",
                        signal_num,
                        _pid
                    ));
                }
            }
        }

        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
            use windows_sys::Win32::System::Threading::{
                OpenProcess, TerminateProcess, PROCESS_TERMINATE,
            };

            unsafe {
                let handle: HANDLE = OpenProcess(PROCESS_TERMINATE, 0, _pid);
                if handle == 0 {
                    return Err(anyhow::anyhow!(
                        "Failed to open process {} for termination",
                        _pid
                    ));
                }
                let exit_code: u32 = match _signal {
                    ProcessSignal::Kill => 1,         // forceful
                    ProcessSignal::Term => 0,         // graceful intent
                    ProcessSignal::Int => 0xC000013A, // CTRL+C/Break equivalent status code
                    ProcessSignal::Quit => 0,
                    ProcessSignal::Stop => 0,
                    ProcessSignal::Cont => 0,
                };
                let ok = TerminateProcess(handle, exit_code);
                CloseHandle(handle);
                if ok == 0 {
                    return Err(anyhow::anyhow!("TerminateProcess failed for pid {}", _pid));
                }
            }
        }

        #[allow(unreachable_code)]
        Ok(())
    }
}

/// Process information structure
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command_line: String,
    pub start_time: Instant,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub status: ProcessStatus,
    pub parent_pid: Option<u32>,
}

impl ProcessInfo {
    pub fn new(pid: u32, name: String, command_line: String) -> Self {
        Self {
            pid,
            name,
            command_line,
            start_time: Instant::now(),
            cpu_usage: 0.0,
            memory_usage: 0,
            status: ProcessStatus::Running,
            parent_pid: None,
        }
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Process status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Running,
    Sleeping,
    Stopped,
    Zombie,
    Dead,
}

/// Process signals for Unix systems
#[derive(Debug, Clone, Copy)]
pub enum ProcessSignal {
    Term, // SIGTERM
    Kill, // SIGKILL
    Int,  // SIGINT
    Quit, // SIGQUIT
    Stop, // SIGSTOP
    Cont, // SIGCONT
}

/// Process execution statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessStats {
    pub total_processes: u64,
    pub executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_execution_time: Duration,
    pub fastest_execution: Duration,
    pub slowest_execution: Duration,
}

impl ProcessStats {
    pub fn success_rate(&self) -> f64 {
        if self.executions > 0 {
            self.successful_executions as f64 / self.executions as f64
        } else {
            0.0
        }
    }

    pub fn avg_execution_time(&self) -> Duration {
        if self.executions > 0 {
            self.total_execution_time / self.executions as u32
        } else {
            Duration::ZERO
        }
    }
}

/// Enhanced command executor with monitoring
pub struct CommandExecutor {
    monitor: Arc<ProcessMonitor>,
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self {
            monitor: Arc::new(ProcessMonitor::new()),
        }
    }

    pub fn with_monitor(monitor: Arc<ProcessMonitor>) -> Self {
        Self { monitor }
    }

    /// Execute command with monitoring
    pub fn execute(&self, program: &str, args: &[&str]) -> Result<CommandResult> {
        let start = Instant::now();

        let mut command = Command::new(program);
        command.args(args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let child = command.spawn()?;
        let pid = child.id();

        // Register process
        let process_info = ProcessInfo::new(
            pid,
            program.to_string(),
            format!("{} {}", program, args.join(" ")),
        );
        self.monitor.register_process(pid, process_info);

        // Wait for completion
        let output = child.wait_with_output()?;
        let duration = start.elapsed();

        // Record statistics
        self.monitor
            .record_execution(duration, output.status.code().unwrap_or(-1));
        self.monitor.unregister_process(pid);

        Ok(CommandResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
        })
    }

    /// Execute command asynchronously
    pub fn execute_async(&self, program: &str, args: &[&str]) -> Result<AsyncCommandHandle> {
        let mut command = Command::new(program);
        command.args(args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.stdin(Stdio::piped());

        let child = command.spawn()?;
        let pid = child.id();

        // Register process
        let process_info = ProcessInfo::new(
            pid,
            program.to_string(),
            format!("{} {}", program, args.join(" ")),
        );
        self.monitor.register_process(pid, process_info);

        Ok(AsyncCommandHandle {
            child,
            monitor: Arc::clone(&self.monitor),
            start_time: Instant::now(),
        })
    }

    /// Get monitor reference
    pub fn monitor(&self) -> Arc<ProcessMonitor> {
        Arc::clone(&self.monitor)
    }
}

/// Result of command execution
#[derive(Debug)]
pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

impl CommandResult {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Handle for asynchronous command execution
pub struct AsyncCommandHandle {
    child: Child,
    monitor: Arc<ProcessMonitor>,
    start_time: Instant,
}

impl AsyncCommandHandle {
    /// Check if process is still running
    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Wait for process completion
    pub fn wait(self) -> Result<CommandResult> {
        let pid = self.child.id();
        let output = self.child.wait_with_output()?;
        let duration = self.start_time.elapsed();

        // Record statistics and unregister
        self.monitor
            .record_execution(duration, output.status.code().unwrap_or(-1));
        self.monitor.unregister_process(pid);

        Ok(CommandResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
        })
    }

    /// Kill the process
    pub fn kill(&mut self) -> Result<()> {
        self.child.kill()?;
        self.monitor.unregister_process(self.child.id());
        Ok(())
    }

    /// Get process ID
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Send input to process stdin
    pub fn send_input(&mut self, input: &str) -> Result<()> {
        if let Some(ref mut stdin) = self.child.stdin {
            stdin.write_all(input.as_bytes())?;
            stdin.flush()?;
        }
        Ok(())
    }
}

/// System process information collector
pub struct SystemProcessCollector;

impl SystemProcessCollector {
    /// Get list of all system processes (Unix only)
    #[cfg(unix)]
    pub fn list_all_processes() -> Result<Vec<ProcessInfo>> {
        use std::fs;

        let mut processes = Vec::new();

        let proc_dir = fs::read_dir("/proc")?;
        for entry in proc_dir {
            let entry = entry?;
            if let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() {
                if let Ok(info) = Self::read_proc_info(pid) {
                    processes.push(info);
                }
            }
        }

        Ok(processes)
    }

    #[cfg(unix)]
    fn read_proc_info(pid: u32) -> Result<ProcessInfo> {
        use std::fs;

        let stat_path = format!("/proc/{}/stat", pid);
        let cmdline_path = format!("/proc/{}/cmdline", pid);

        let stat_content = fs::read_to_string(stat_path)?;
        let cmdline_content = fs::read_to_string(cmdline_path).unwrap_or_else(|_| String::new());

        let stat_fields: Vec<&str> = stat_content.split_whitespace().collect();
        let name = if stat_fields.len() > 1 {
            stat_fields[1].trim_matches(['(', ')']).to_string()
        } else {
            "unknown".to_string()
        };

        let command_line = cmdline_content.replace('\0', " ").trim().to_string();
        let command_line = if command_line.is_empty() {
            format!("[{}]", name)
        } else {
            command_line
        };

        Ok(ProcessInfo::new(pid, name, command_line))
    }

    #[cfg(windows)]
    pub fn list_all_processes() -> Result<Vec<ProcessInfo>> {
        // Windows implementation would use WMI or Windows API
        // For now, return empty list
        Ok(Vec::new())
    }

    /// Find processes by name
    pub fn find_by_name(name: &str) -> Result<Vec<ProcessInfo>> {
        let all_processes = Self::list_all_processes()?;
        Ok(all_processes
            .into_iter()
            .filter(|p| p.name.contains(name))
            .collect())
    }

    /// Get current process information
    pub fn current_process() -> ProcessInfo {
        let pid = std::process::id();
        ProcessInfo::new(
            pid,
            "nxsh".to_string(),
            std::env::args().collect::<Vec<_>>().join(" "),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_monitor() {
        let monitor = ProcessMonitor::new();
        let info = ProcessInfo::new(12345, "test".to_string(), "test command".to_string());

        monitor.register_process(12345, info.clone());

        let retrieved = monitor.get_process(12345).unwrap();
        assert_eq!(retrieved.pid, 12345);
        assert_eq!(retrieved.name, "test");

        monitor.record_execution(Duration::from_millis(100), 0);
        let stats = monitor.stats();
        assert_eq!(stats.executions, 1);
        assert_eq!(stats.successful_executions, 1);
    }

    #[test]
    fn test_command_executor() {
        let executor = CommandExecutor::new();

        // Test with a simple command that should exist on most systems
        #[cfg(unix)]
        {
            let result = executor.execute("echo", &["hello"]).unwrap();
            assert!(result.success());
            assert_eq!(result.stdout.trim(), "hello");
        }

        #[cfg(windows)]
        {
            let result = executor.execute("cmd", &["/c", "echo hello"]).unwrap();
            assert!(result.success());
            assert_eq!(result.stdout.trim(), "hello");
        }
    }

    #[test]
    fn test_process_stats() {
        let stats = ProcessStats {
            executions: 10,
            successful_executions: 8,
            failed_executions: 2,
            total_execution_time: Duration::from_millis(1000),
            ..Default::default()
        };

        assert!((stats.success_rate() - 0.8).abs() < 0.001);
        assert_eq!(stats.avg_execution_time(), Duration::from_millis(100));
    }
}
