//! Job control and process management for NexusShell
//!
//! This module provides job control functionality including background jobs,
//! process groups, signal handling, and job status tracking.

use crate::error::{ShellError, ErrorKind, ShellResult};
use std::collections::HashMap;
use std::fmt;
use std::process::ExitStatus;
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::LazyLock;

/// Job identifier type
pub type JobId = u32;

/// Process identifier type  
pub type ProcessId = u32;

/// Process group identifier type
pub type ProcessGroupId = u32;

/// Job status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    /// Job is currently running
    Running,
    /// Job is stopped (suspended)
    Stopped,
    /// Job completed successfully
    Done(i32), // exit code
    /// Job was terminated by a signal
    Terminated(i32), // signal number
    /// Job failed to start
    Failed(String), // error message
    /// Job is waiting for input/output
    Waiting,
    /// Job is in the background
    Background,
    /// Job is in the foreground
    Foreground,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Stopped => write!(f, "Stopped"),
            JobStatus::Done(code) => write!(f, "Done ({code})"),
            JobStatus::Terminated(sig) => write!(f, "Terminated (signal {sig})"),
            JobStatus::Failed(msg) => write!(f, "Failed: {msg}"),
            JobStatus::Waiting => write!(f, "Waiting"),
            JobStatus::Background => write!(f, "Background"),
            JobStatus::Foreground => write!(f, "Foreground"),
        }
    }
}

/// Process information within a job
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: ProcessId,
    /// Process group ID
    pub pgid: ProcessGroupId,
    /// Command line that started this process
    pub command: String,
    /// Process status
    pub status: JobStatus,
    /// Process start time
    pub start_time: Instant,
    /// Process end time (if finished)
    pub end_time: Option<Instant>,
    /// Exit status (if finished)
    pub exit_status: Option<ExitStatus>,
    /// CPU time used
    pub cpu_time: Duration,
    /// Memory usage (in bytes)
    pub memory_usage: u64,
}

impl ProcessInfo {
    /// Create new process info
    pub fn new(pid: ProcessId, pgid: ProcessGroupId, command: String) -> Self {
        Self {
            pid,
            pgid,
            command,
            status: JobStatus::Running,
            start_time: Instant::now(),
            end_time: None,
            exit_status: None,
            cpu_time: Duration::new(0, 0),
            memory_usage: 0,
        }
    }

    /// Check if process is running
    pub fn is_running(&self) -> bool {
        matches!(self.status, JobStatus::Running | JobStatus::Background | JobStatus::Foreground)
    }

    /// Check if process is finished
    pub fn is_finished(&self) -> bool {
        matches!(self.status, JobStatus::Done(_) | JobStatus::Terminated(_) | JobStatus::Failed(_))
    }

    /// Get runtime duration
    pub fn runtime(&self) -> Duration {
        match self.end_time {
            Some(end) => end.duration_since(self.start_time),
            None => self.start_time.elapsed(),
        }
    }

    /// Update process status
    pub fn update_status(&mut self, status: JobStatus) {
        self.status = status;
        if self.is_finished() && self.end_time.is_none() {
            self.end_time = Some(Instant::now());
        }
    }
}

/// Job represents a collection of processes (pipeline)
#[derive(Debug, Clone)]
pub struct Job {
    /// Unique job identifier
    pub id: JobId,
    /// Job description/command
    pub description: String,
    /// List of processes in this job
    pub processes: Vec<ProcessInfo>,
    /// Job status (derived from process statuses)
    pub status: JobStatus,
    /// Process group ID for the entire job
    pub pgid: ProcessGroupId,
    /// Whether this job is in the foreground
    pub foreground: bool,
    /// Job creation time
    pub created_at: Instant,
    /// Job completion time
    pub completed_at: Option<Instant>,
    /// Working directory when job was started
    pub working_dir: std::path::PathBuf,
    /// Environment variables when job was started
    pub environment: HashMap<String, String>,
}

impl Job {
    /// Create a new job
    pub fn new(id: JobId, description: String) -> Self {
        Self {
            id,
            description,
            processes: Vec::new(),
            status: JobStatus::Running,
            pgid: 0, // Will be set when first process is added
            foreground: false,
            created_at: Instant::now(),
            completed_at: None,
            working_dir: std::env::current_dir().unwrap_or_default(),
            environment: std::env::vars().collect(),
        }
    }

    /// Add a process to this job
    pub fn add_process(&mut self, process: ProcessInfo) {
        if self.pgid == 0 {
            self.pgid = process.pgid;
        }
        self.processes.push(process);
        self.update_status();
    }

    /// Remove a process from this job
    pub fn remove_process(&mut self, pid: ProcessId) -> Option<ProcessInfo> {
        if let Some(pos) = self.processes.iter().position(|p| p.pid == pid) {
            let process = self.processes.remove(pos);
            self.update_status();
            Some(process)
        } else {
            None
        }
    }

    /// Get process by PID
    pub fn get_process(&self, pid: ProcessId) -> Option<&ProcessInfo> {
        self.processes.iter().find(|p| p.pid == pid)
    }

    /// Get mutable process by PID
    pub fn get_process_mut(&mut self, pid: ProcessId) -> Option<&mut ProcessInfo> {
        self.processes.iter_mut().find(|p| p.pid == pid)
    }

    /// Update job status based on process statuses
    pub fn update_status(&mut self) {
        if self.processes.is_empty() {
            self.status = JobStatus::Done(0);
            return;
        }

        let running_count = self.processes.iter().filter(|p| p.is_running()).count();
        let finished_count = self.processes.iter().filter(|p| p.is_finished()).count();
        let stopped_count = self.processes.iter().filter(|p| p.status == JobStatus::Stopped).count();

        self.status = if running_count > 0 {
            if self.foreground {
                JobStatus::Foreground
            } else {
                JobStatus::Background
            }
        } else if stopped_count > 0 {
            JobStatus::Stopped
        } else if finished_count == self.processes.len() {
            // All processes finished - determine overall status
            let failed_processes: Vec<_> = self.processes.iter()
                .filter(|p| matches!(p.status, JobStatus::Failed(_) | JobStatus::Terminated(_)))
                .collect();
            
            if !failed_processes.is_empty() {
                if let JobStatus::Failed(ref msg) = failed_processes[0].status {
                    JobStatus::Failed(msg.clone())
                } else if let JobStatus::Terminated(sig) = failed_processes[0].status {
                    JobStatus::Terminated(sig)
                } else {
                    JobStatus::Done(1)
                }
            } else {
                // Get exit code from last process (traditional shell behavior)
                let exit_code = self.processes.last()
                    .and_then(|p| match p.status {
                        JobStatus::Done(code) => Some(code),
                        _ => None,
                    })
                    .unwrap_or(0);
                JobStatus::Done(exit_code)
            }
        } else {
            JobStatus::Running
        };

        // Update completion time if job is finished
        if self.is_finished() && self.completed_at.is_none() {
            self.completed_at = Some(Instant::now());
        }
    }

    /// Check if job has any running processes
    pub fn has_running_processes(&self) -> bool {
        self.processes.iter().any(|p| p.is_running())
    }

    /// Check if job is running
    pub fn is_running(&self) -> bool {
        matches!(self.status, JobStatus::Running | JobStatus::Background | JobStatus::Foreground)
    }

    /// Check if job is finished
    pub fn is_finished(&self) -> bool {
        matches!(self.status, JobStatus::Done(_) | JobStatus::Terminated(_) | JobStatus::Failed(_))
    }

    /// Check if job is stopped
    pub fn is_stopped(&self) -> bool {
        matches!(self.status, JobStatus::Stopped)
    }

    /// Get job runtime
    pub fn runtime(&self) -> Duration {
        match self.completed_at {
            Some(end) => end.duration_since(self.created_at),
            None => self.created_at.elapsed(),
        }
    }

    /// Get total CPU time used by all processes
    pub fn total_cpu_time(&self) -> Duration {
        self.processes.iter().map(|p| p.cpu_time).sum()
    }

    /// Get total memory usage of all processes
    pub fn total_memory_usage(&self) -> u64 {
        self.processes.iter().map(|p| p.memory_usage).sum()
    }

    /// Move job to foreground
    pub fn move_to_foreground(&mut self) {
        self.foreground = true;
        self.update_status();
    }

    /// Move job to background
    pub fn move_to_background(&mut self) {
        self.foreground = false;
        self.update_status();
    }
}

/// Job control signals
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JobSignal {
    /// Continue execution (SIGCONT)
    Continue,
    /// Stop execution (SIGSTOP)
    Stop,
    /// Terminate (SIGTERM)
    Terminate,
    /// Kill (SIGKILL)
    Kill,
    /// Interrupt (SIGINT)
    Interrupt,
    /// Quit (SIGQUIT)
    Quit,
    /// Hangup (SIGHUP)
    Hangup,
    /// User signal 1 (SIGUSR1)
    User1,
    /// User signal 2 (SIGUSR2)
    User2,
}

impl JobSignal {
    /// Convert to system signal number
    pub fn to_signal_number(self) -> i32 {
        match self {
            JobSignal::Continue => 18,  // SIGCONT
            JobSignal::Stop => 19,      // SIGSTOP
            JobSignal::Terminate => 15, // SIGTERM
            JobSignal::Kill => 9,       // SIGKILL
            JobSignal::Interrupt => 2,  // SIGINT
            JobSignal::Quit => 3,       // SIGQUIT
            JobSignal::Hangup => 1,     // SIGHUP
            JobSignal::User1 => 10,     // SIGUSR1
            JobSignal::User2 => 12,     // SIGUSR2
        }
    }

    /// Convert from system signal number
    pub fn from_signal_number(sig: i32) -> Option<Self> {
        match sig {
            18 => Some(JobSignal::Continue),
            19 => Some(JobSignal::Stop),
            15 => Some(JobSignal::Terminate),
            9 => Some(JobSignal::Kill),
            2 => Some(JobSignal::Interrupt),
            3 => Some(JobSignal::Quit),
            1 => Some(JobSignal::Hangup),
            10 => Some(JobSignal::User1),
            12 => Some(JobSignal::User2),
            _ => None,
        }
    }
}

/// Job manager for handling all jobs in the shell
pub struct JobManager {
    /// Map of job ID to job
    jobs: Arc<RwLock<HashMap<JobId, Job>>>,
    /// Next job ID to assign
    next_job_id: Arc<Mutex<JobId>>,
    /// Currently active (foreground) job
    foreground_job: Arc<Mutex<Option<JobId>>>,
    /// Job notification channel
    notification_tx: mpsc::Sender<JobNotification>,
    /// Job notification receiver
    notification_rx: Arc<Mutex<mpsc::Receiver<JobNotification>>>,
    /// Whether job control is enabled
    job_control_enabled: bool,
    /// Process monitoring thread handle
    monitor_handle: Option<thread::JoinHandle<()>>,
}

// ---------------------------------------------------------------------------
// Global JobManager (singleton) for builtins needing job control access
// ---------------------------------------------------------------------------
static GLOBAL_JOB_MANAGER: LazyLock<Mutex<JobManager>> = LazyLock::new(|| Mutex::new(JobManager::new()));

/// Execute a closure with mutable access to the global JobManager.
/// Keep critical section short to avoid blocking other builtin operations.
pub fn with_global_job_manager<F, R>(f: F) -> R
where
    F: FnOnce(&mut JobManager) -> R
{
    let mut guard = GLOBAL_JOB_MANAGER.lock().expect("GLOBAL_JOB_MANAGER poisoned");
    f(&mut guard)
}


impl fmt::Debug for JobManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JobManager")
            .field("jobs", &"Arc<RwLock<HashMap<JobId, Job>>>")
            .field("next_job_id", &"Arc<Mutex<JobId>>")
            .field("foreground_job", &"Arc<Mutex<Option<JobId>>>")
            .field("job_control_enabled", &self.job_control_enabled)
            .field("monitor_handle", &self.monitor_handle.is_some())
            .finish()
    }
}

/// Job notification types
#[derive(Debug, Clone)]
pub enum JobNotification {
    /// Job status changed
    StatusChanged {
        job_id: JobId,
        old_status: JobStatus,
        new_status: JobStatus,
    },
    /// Process added to job
    ProcessAdded {
        job_id: JobId,
        process_id: ProcessId,
    },
    /// Process removed from job
    ProcessRemoved {
        job_id: JobId,
        process_id: ProcessId,
    },
    /// Job created
    JobCreated {
        job_id: JobId,
        description: String,
    },
    /// Job removed
    JobRemoved {
        job_id: JobId,
    },
}

impl JobManager {
    /// Create a new job manager
    pub fn new() -> Self {
        let (notification_tx, notification_rx) = mpsc::channel();
        
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            next_job_id: Arc::new(Mutex::new(1)),
            foreground_job: Arc::new(Mutex::new(None)),
            notification_tx,
            notification_rx: Arc::new(Mutex::new(notification_rx)),
            job_control_enabled: true,
            monitor_handle: None,
        }
    }

    /// Safely acquire a read lock on jobs
    fn get_jobs_read(&self) -> ShellResult<std::sync::RwLockReadGuard<'_, HashMap<JobId, Job>>> {
        self.jobs.read().map_err(|_| ShellError::new(
            ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
            "Failed to acquire read lock on jobs".to_string()
        ))
    }

    /// Safely acquire a write lock on jobs  
    fn get_jobs_write(&self) -> ShellResult<std::sync::RwLockWriteGuard<'_, HashMap<JobId, Job>>> {
        self.jobs.write().map_err(|_| ShellError::new(
            ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
            "Failed to acquire write lock on jobs".to_string()
        ))
    }

    /// Safely acquire a lock on next job ID
    fn get_next_job_id_lock(&self) -> ShellResult<std::sync::MutexGuard<'_, JobId>> {
        self.next_job_id.lock().map_err(|_| ShellError::new(
            ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
            "Failed to acquire lock on next job ID".to_string()
        ))
    }

    /// Safely acquire a lock on foreground job
    fn get_foreground_job_lock(&self) -> ShellResult<std::sync::MutexGuard<'_, Option<JobId>>> {
        self.foreground_job.lock().map_err(|_| ShellError::new(
            ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
            "Failed to acquire lock on foreground job".to_string()
        ))
    }

    /// Enable or disable job control
    pub fn set_job_control(&mut self, enabled: bool) {
        self.job_control_enabled = enabled;
    }

    /// Check if job control is enabled
    pub fn is_job_control_enabled(&self) -> bool {
        self.job_control_enabled
    }

    /// Create a new job
    pub fn create_job(&mut self, description: String) -> ShellResult<JobId> {
        let job_id = {
            let mut next_id = self.get_next_job_id_lock()?;
            let id = *next_id;
            *next_id += 1;
            id
        };

        let job = Job::new(job_id, description.clone());
        
        {
            let mut jobs = self.get_jobs_write()?;
            jobs.insert(job_id, job);
        }

        // Send notification
        let _ = self.notification_tx.send(JobNotification::JobCreated {
            job_id,
            description,
        });

        Ok(job_id)
    }

    /// Get a job by ID
    pub fn get_job(&self, job_id: JobId) -> ShellResult<Option<Job>> {
        let jobs = self.get_jobs_read()?;
        Ok(jobs.get(&job_id).cloned())
    }

    /// Get a mutable reference to a job by ID
    /// 
    /// This method provides temporary mutable access to a job for updating
    /// its state. The caller receives a closure that can modify the job.
    pub fn with_job_mut<T, F>(&self, job_id: JobId, f: F) -> Option<T>
    where
        F: FnOnce(&mut Job) -> T,
    {
        let mut jobs = self.jobs.write().unwrap();
        jobs.get_mut(&job_id).map(f)
    }

    /// Get a mutable job by ID (alternative implementation)
    /// 
    /// Returns a clone of the job that can be modified and then updated back
    /// using update_job method. This approach avoids holding locks for extended periods.
    pub fn get_job_mut(&self, job_id: JobId) -> Option<Job> {
        let jobs = self.jobs.read().unwrap();
        jobs.get(&job_id).cloned()
    }

    /// Update an existing job
    /// 
    /// This method should be used in conjunction with get_job_mut to update
    /// a job after modification.
    pub fn update_job(&mut self, job: Job) -> bool {
        let mut jobs = self.jobs.write().unwrap();
        use std::collections::hash_map::Entry;
        match jobs.entry(job.id) {
            Entry::Occupied(mut e) => {
                e.insert(job);
                true
            }
            Entry::Vacant(_) => false,
        }
    }

    /// Get all jobs
    pub fn get_all_jobs(&self) -> Vec<Job> {
        let jobs = self.jobs.read().unwrap();
        jobs.values().cloned().collect()
    }

    /// Get running jobs
    pub fn get_running_jobs(&self) -> Vec<Job> {
        let jobs = self.jobs.read().unwrap();
        jobs.values().filter(|job| job.is_running()).cloned().collect()
    }

    /// Get stopped jobs
    pub fn get_stopped_jobs(&self) -> Vec<Job> {
        let jobs = self.jobs.read().unwrap();
        jobs.values().filter(|job| job.is_stopped()).cloned().collect()
    }

    /// Remove a job
    pub fn remove_job(&mut self, job_id: JobId) -> Option<Job> {
        let job = {
            let mut jobs = self.jobs.write().unwrap();
            jobs.remove(&job_id)
        };

        if job.is_some() {
            // Send notification
            let _ = self.notification_tx.send(JobNotification::JobRemoved { job_id });
        }

        job
    }

    /// Add a process to a job
    pub fn add_process_to_job(&mut self, job_id: JobId, process: ProcessInfo) -> ShellResult<()> {
        let mut jobs = self.jobs.write()
            .map_err(|_| ShellError::new(ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState), "Jobs lock poisoned"))?;
        
        if let Some(job) = jobs.get_mut(&job_id) {
            let process_id = process.pid;
            job.add_process(process);
            
            // Send notification
            let _ = self.notification_tx.send(JobNotification::ProcessAdded {
                job_id,
                process_id,
            });
            
            Ok(())
        } else {
            Err(ShellError::new(ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), format!("Job {job_id} not found")))
        }
    }

    /// Remove a process from a job
    pub fn remove_process_from_job(&mut self, job_id: JobId, process_id: ProcessId) -> ShellResult<Option<ProcessInfo>> {
        let mut jobs = self.jobs.write()
            .map_err(|_| ShellError::new(ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState), "Jobs lock poisoned"))?;
        
        if let Some(job) = jobs.get_mut(&job_id) {
            let process = job.remove_process(process_id);
            
            if process.is_some() {
                // Send notification
                let _ = self.notification_tx.send(JobNotification::ProcessRemoved {
                    job_id,
                    process_id,
                });
            }
            
            Ok(process)
        } else {
            Err(ShellError::new(ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), format!("Job {job_id} not found")))
        }
    }

    /// Update job status
    pub fn update_job_status(&mut self, job_id: JobId, new_status: JobStatus) -> ShellResult<()> {
        let mut jobs = self.jobs.write()
            .map_err(|_| ShellError::new(ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState), "Jobs lock poisoned"))?;
        
        if let Some(job) = jobs.get_mut(&job_id) {
            let old_status = job.status.clone();
            job.status = new_status.clone();
            
            // Send notification
            let _ = self.notification_tx.send(JobNotification::StatusChanged {
                job_id,
                old_status,
                new_status,
            });
            
            Ok(())
        } else {
            Err(ShellError::new(ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), format!("Job {job_id} not found")))
        }
    }

    /// Send signal to a job (all processes in the job)
    pub fn send_signal_to_job(&self, job_id: JobId, signal: JobSignal) -> ShellResult<()> {
        let jobs = self.jobs.read()
            .map_err(|_| ShellError::new(ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState), "Jobs lock poisoned"))?;
        
        if let Some(job) = jobs.get(&job_id) {
            // Send signal to process group
            self.send_signal_to_process_group(job.pgid, signal)
        } else {
            Err(ShellError::new(ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), format!("Job {job_id} not found")))
        }
    }

    /// Send signal to a process group
    pub fn send_signal_to_process_group(&self, pgid: ProcessGroupId, signal: JobSignal) -> ShellResult<()> {
        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;
            
            let signal_num = signal.to_signal_number();
            let nix_signal = Signal::try_from(signal_num)
                .map_err(|e| ShellError::new(ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError), format!("Invalid signal: {}", e)))?;
            
            signal::killpg(Pid::from_raw(pgid as i32), nix_signal)
                .map_err(|e| ShellError::new(ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError), format!("Failed to send signal: {}", e)))?;
        }
        
        #[cfg(windows)]
        {
            use std::process::Command;
            use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
            use windows_sys::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Thread32First, Thread32Next, THREADENTRY32, TH32CS_SNAPTHREAD};
            use windows_sys::Win32::System::Threading::{OpenThread, ResumeThread, SuspendThread, THREAD_SUSPEND_RESUME};

            // Helper: suspend all threads of the process identified by pid
            unsafe fn suspend_process_threads(pid: u32) -> Result<(), String> {
                let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
                if snapshot == INVALID_HANDLE_VALUE {
                    return Err("CreateToolhelp32Snapshot failed".to_string());
                }
                let mut entry: THREADENTRY32 = std::mem::zeroed();
                entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
                if Thread32First(snapshot, &mut entry) == 0 {
                    CloseHandle(snapshot);
                    return Err("Thread32First failed".to_string());
                }
                loop {
                    if entry.th32OwnerProcessID == pid {
                        let hthread = OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID);
                        if hthread != 0 {
                            let _ = SuspendThread(hthread);
                            CloseHandle(hthread);
                        }
                    }
                    if Thread32Next(snapshot, &mut entry) == 0 { break; }
                }
                CloseHandle(snapshot);
                Ok(())
            }

            // Helper: resume all threads of the process identified by pid
            unsafe fn resume_process_threads(pid: u32) -> Result<(), String> {
                let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
                if snapshot == INVALID_HANDLE_VALUE {
                    return Err("CreateToolhelp32Snapshot failed".to_string());
                }
                let mut entry: THREADENTRY32 = std::mem::zeroed();
                entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
                if Thread32First(snapshot, &mut entry) == 0 {
                    CloseHandle(snapshot);
                    return Err("Thread32First failed".to_string());
                }
                loop {
                    if entry.th32OwnerProcessID == pid {
                        let hthread = OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID);
                        if hthread != 0 {
                            let _ = ResumeThread(hthread);
                            CloseHandle(hthread);
                        }
                    }
                    if Thread32Next(snapshot, &mut entry) == 0 { break; }
                }
                CloseHandle(snapshot);
                Ok(())
            }

            match signal {
                JobSignal::Terminate | JobSignal::Kill => {
                    // Use taskkill for termination on Windows (works across consoles)
                    let kill_flag = if signal == JobSignal::Kill { "/F" } else { "/T" };
                    Command::new("taskkill")
                        .args([kill_flag, "/PID", &pgid.to_string()])
                        .output()
                        .map_err(|e| ShellError::new(
                            ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                            format!("Failed to send signal on Windows: {e}")
                        ))?;
                }
                JobSignal::Stop => {
                    // Suspend all threads of the target process (best-effort)
                    unsafe { suspend_process_threads(pgid) }
                        .map_err(|e| ShellError::new(
                            ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                            format!("Failed to suspend process {pgid}: {e}")
                        ))?;
                }
                JobSignal::Continue => {
                    // Resume all threads of the target process (best-effort)
                    unsafe { resume_process_threads(pgid) }
                        .map_err(|e| ShellError::new(
                            ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                            format!("Failed to resume process {pgid}: {e}")
                        ))?;
                }
                _ => {
                    return Err(ShellError::new(
                        ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                        format!("Signal {signal:?} not supported on Windows")
                    ));
                }
            }
        }
        
        Ok(())
    }

    /// Spawn a background job
    pub fn spawn_background_job(&mut self, command: String, args: Vec<String>) -> ShellResult<JobId> {
        // Reuse a temporary buffer to avoid intermediate allocations from format!/join in hot path
        let mut cmd_buf = String::with_capacity(command.len() + 1 + args.iter().map(|a| a.len()+1).sum::<usize>());
    cmd_buf.push_str(&command);
        if !args.is_empty() { cmd_buf.push(' '); }
        for (i, a) in args.iter().enumerate() {
            cmd_buf.push_str(a);
            if i + 1 != args.len() { cmd_buf.push(' '); }
        }
        let job_id = self.create_job(cmd_buf)?;
        
        // Spawn the process
        #[cfg(unix)]
        {
            use std::process::{Command, Stdio};
            use nix::unistd::{setpgid, Pid};
            
            let mut cmd = Command::new(&command);
            cmd.args(&args)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            
            let child = cmd.spawn()
                .map_err(|e| ShellError::new(
                    ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                    format!("Failed to spawn background process: {}", e)
                ))?;
            
            let pid = child.id();
            let pgid = pid; // Use PID as PGID for new process group
            
            // Set process group for job control
            if let Err(e) = setpgid(Pid::from_raw(pid as i32), Pid::from_raw(pgid as i32)) {
                eprintln!("Warning: Failed to set process group: {}", e);
            }
            
            let process_info = crate::job::ProcessInfo::new(pid, pgid, format!("{} {}", command, args.join(" ")));
            self.add_process_to_job(job_id, process_info)?;
            
            // Start monitoring thread for this job
            self.start_job_monitor(job_id, child);
        }
        
        #[cfg(windows)]
        {
            use std::process::{Command, Stdio};
            use std::io::ErrorKind as IoErrorKind;
            
            // First attempt direct spawn (works for real executables)
            let mut direct = Command::new(&command);
            direct.args(&args)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());

            let child = match direct.spawn() {
                Ok(c) => c,
                Err(e) => {
                    if e.kind() == IoErrorKind::NotFound {
                        // Fallback for common shell builtins not available as executables
                        let lower = command.to_ascii_lowercase();
                        if lower == "echo" {
                            let mut fb = Command::new("cmd.exe");
                            let mut full = String::from("echo");
                            for a in &args { full.push(' '); full.push_str(a); }
                            fb.args(["/C", &full])
                                .stdin(Stdio::null())
                                .stdout(Stdio::null())
                                .stderr(Stdio::null());
                            fb.spawn().map_err(|e2| ShellError::new(
                                ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                                format!("Failed to spawn background process (fallback echo): {e2}")
                            ))?
                        } else if lower == "sleep" {
                            let seconds = args.first().and_then(|s| s.parse::<u64>().ok()).unwrap_or(1);
                            let mut fb = Command::new("powershell.exe");
                            fb.args(["-NoProfile", "-Command", &format!("Start-Sleep -Seconds {seconds}")])
                                .stdin(Stdio::null())
                                .stdout(Stdio::null())
                                .stderr(Stdio::null());
                            fb.spawn().map_err(|e2| ShellError::new(
                                ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                                format!("Failed to spawn background process (fallback sleep): {e2}")
                            ))?
                        } else {
                            return Err(ShellError::new(
                                ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                                "Failed to spawn background process: program not found".to_string()));
                        }
                    } else {
                        return Err(ShellError::new(
                            ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                            format!("Failed to spawn background process: {e}")));
                    }
                }
            };
            
            let pid = child.id();
            let pgid = pid; // Windows doesn't have process groups like Unix
            
            let process_info = crate::job::ProcessInfo::new(pid, pgid, format!("{} {}", command, args.join(" ")));
            self.add_process_to_job(job_id, process_info)?;
            
            // Start monitoring thread for this job
            self.start_job_monitor(job_id, child);
        }
        
        // Move job to background
        self.move_job_to_background(job_id)?;
        
        Ok(job_id)
    }
    
    /// Start a monitoring thread for a background job
    fn start_job_monitor(&self, job_id: JobId, mut child: std::process::Child) {
        let jobs = Arc::clone(&self.jobs);
        let notification_tx = self.notification_tx.clone();
        
        std::thread::spawn(move || {
            // Wait for process completion
            match child.wait() {
                Ok(exit_status) => {
                    let new_status = if exit_status.success() {
                        JobStatus::Done(exit_status.code().unwrap_or(0))
                    } else {
                        JobStatus::Failed(format!("Process exited with code: {}",
                            exit_status.code().unwrap_or(-1)))
                    };
                    
                    // Update job status
                    if let Ok(mut jobs_guard) = jobs.write() {
                        if let Some(job) = jobs_guard.get_mut(&job_id) {
                            let old_status = job.status.clone();
                            job.status = new_status.clone();
                            job.completed_at = Some(std::time::Instant::now());
                            
                            // Update process status
                            if let Some(process) = job.processes.get_mut(0) {
                                process.status = new_status.clone();
                                process.end_time = Some(std::time::Instant::now());
                                process.exit_status = Some(exit_status);
                            }
                            
                            // Send notification
                            let _ = notification_tx.send(JobNotification::StatusChanged {
                                job_id,
                                old_status,
                                new_status,
                            });
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error waiting for background job {job_id}: {e}");
                    
                    // Update job as failed
                    if let Ok(mut jobs_guard) = jobs.write() {
                        if let Some(job) = jobs_guard.get_mut(&job_id) {
                            let old_status = job.status.clone();
                            let new_status = JobStatus::Failed(format!("Wait error: {e}"));
                            job.status = new_status.clone();
                            job.completed_at = Some(std::time::Instant::now());
                            
                            // Send notification
                            let _ = notification_tx.send(JobNotification::StatusChanged {
                                job_id,
                                old_status,
                                new_status,
                            });
                        }
                    }
                }
            }
        });
    }

    /// Get job notifications channel receiver
    pub fn get_notification_receiver(&self) -> Arc<std::sync::Mutex<std::sync::mpsc::Receiver<JobNotification>>> {
        Arc::clone(&self.notification_rx)
    }

    /// Process pending job notifications
    pub fn process_notifications(&self) -> Vec<JobNotification> {
        let mut notifications = Vec::new();
        
        if let Ok(rx) = self.notification_rx.lock() {
            while let Ok(notification) = rx.try_recv() {
                notifications.push(notification);
            }
        }
        
        notifications
    }

    /// Move job to foreground
    pub fn move_job_to_foreground(&mut self, job_id: JobId) -> ShellResult<()> {
        // Set current foreground job
        {
            let mut fg_job = self.foreground_job.lock()
                .map_err(|_| ShellError::new(ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState), "Foreground job lock poisoned"))?;
            *fg_job = Some(job_id);
        }

        // Update job status
        {
            let mut jobs = self.jobs.write()
                .map_err(|_| ShellError::new(ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState), "Jobs lock poisoned"))?;
            
            if let Some(job) = jobs.get_mut(&job_id) {
                job.move_to_foreground();
                
                // Continue the job if it was stopped
                if job.is_stopped() {
                    self.send_signal_to_job(job_id, JobSignal::Continue)?;
                }
            } else {
                return Err(ShellError::new(ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), format!("Job {job_id} not found")));
            }
        }

        Ok(())
    }

    /// Move job to background
    pub fn move_job_to_background(&mut self, job_id: JobId) -> ShellResult<()> {
        // Clear foreground job if this was it
        {
            let mut fg_job = self.foreground_job.lock()
                .map_err(|_| ShellError::new(ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState), "Foreground job lock poisoned"))?;
            if *fg_job == Some(job_id) {
                *fg_job = None;
            }
        }

        // Update job status
        {
            let mut jobs = self.jobs.write()
                .map_err(|_| ShellError::new(ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState), "Jobs lock poisoned"))?;
            
            if let Some(job) = jobs.get_mut(&job_id) {
                job.move_to_background();
                
                // Continue the job if it was stopped
                if job.is_stopped() {
                    self.send_signal_to_job(job_id, JobSignal::Continue)?;
                }
            } else {
                return Err(ShellError::new(ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), format!("Job {job_id} not found")));
            }
        }

        Ok(())
    }

    /// Get current foreground job
    pub fn get_foreground_job(&self) -> ShellResult<Option<JobId>> {
        let fg_job = self.get_foreground_job_lock()?;
        Ok(*fg_job)
    }

    /// Wait for a job to complete
    pub fn wait_for_job(&self, job_id: JobId) -> ShellResult<JobStatus> {
        loop {
            {
                let jobs = self.jobs.read()
                    .map_err(|_| ShellError::new(ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState), "Jobs lock poisoned"))?;
                
                if let Some(job) = jobs.get(&job_id) {
                    if job.is_finished() {
                        return Ok(job.status.clone());
                    }
                } else {
                    return Err(ShellError::new(ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument), format!("Job {job_id} not found")));
                }
            }
            
            // Sleep briefly before checking again
            thread::sleep(Duration::from_millis(10));
        }
    }

    /// Clean up finished jobs
    pub fn cleanup_finished_jobs(&mut self) -> ShellResult<()> {
        let finished_jobs: Vec<JobId> = {
            let jobs = self.get_jobs_read()?;
            jobs.iter()
                .filter(|(_, job)| job.is_finished())
                .map(|(id, _)| *id)
                .collect()
        };

        for job_id in finished_jobs {
            let _ = self.remove_job(job_id); // Ignore return value since we only want cleanup
        }
        
        Ok(())
    }

    /// Get job statistics
    pub fn get_statistics(&self) -> ShellResult<JobStatistics> {
        let jobs = self.get_jobs_read()?;
        
        let total_jobs = jobs.len();
        let running_jobs = jobs.values().filter(|job| job.is_running()).count();
        let stopped_jobs = jobs.values().filter(|job| job.is_stopped()).count();
        let finished_jobs = jobs.values().filter(|job| job.is_finished()).count();
        
        let total_processes: usize = jobs.values().map(|job| job.processes.len()).sum();
        let total_cpu_time: Duration = jobs.values().map(|job| job.total_cpu_time()).sum();
        let total_memory_usage: u64 = jobs.values().map(|job| job.total_memory_usage()).sum();

        Ok(JobStatistics {
            total_jobs,
            running_jobs,
            stopped_jobs,
            finished_jobs,
            total_processes,
            total_cpu_time,
            total_memory_usage,
        })
    }
}

/// Job manager statistics
#[derive(Debug, Clone)]
pub struct JobStatistics {
    pub total_jobs: usize,
    pub running_jobs: usize,
    pub stopped_jobs: usize,
    pub finished_jobs: usize,
    pub total_processes: usize,
    pub total_cpu_time: Duration,
    pub total_memory_usage: u64,
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize job control system
pub fn init() {
    // This function can be used to set up signal handlers and other
    // job control initialization if needed
    
    #[cfg(unix)]
    {
        // Set up signal handlers for job control
        // This would typically involve setting up handlers for SIGCHLD, SIGTSTP, etc.
        // For now, we'll leave this as a placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let mut manager = JobManager::new();
        let job_id = manager.create_job("test command".to_string()).expect("Failed to create job");
        
        assert_eq!(job_id, 1);
        
        let job = manager.get_job(job_id).expect("Failed to get job").expect("Job not found");
        assert_eq!(job.description, "test command");
        assert_eq!(job.processes.len(), 0);
    }

    #[test]
    fn test_process_management() {
        let mut manager = JobManager::new();
        let job_id = manager.create_job("test command".to_string()).expect("Failed to create job");
        
        let process = ProcessInfo::new(12345, 12345, "test".to_string());
        manager.add_process_to_job(job_id, process).expect("Failed to add process");
        
        let job = manager.get_job(job_id).expect("Failed to get job").expect("Job not found");
        assert_eq!(job.processes.len(), 1);
        assert_eq!(job.processes[0].pid, 12345);
    }

    #[test]
    fn test_job_status_updates() {
        let mut manager = JobManager::new();
        let job_id = manager.create_job("test command".to_string()).expect("Failed to create job");
        
        manager.update_job_status(job_id, JobStatus::Stopped).expect("Failed to update job status");
        
        let job = manager.get_job(job_id).expect("Failed to get job").expect("Job not found");
        assert_eq!(job.status, JobStatus::Stopped);
    }
} 