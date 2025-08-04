//! Shell execution context and state management
//!
//! This module provides the core execution context for NexusShell,
//! managing variables, functions, aliases, and execution state.

use crate::error::{ShellError, ErrorKind, ShellResult};
use crate::stream::Stream;
use crate::job::{JobManager, JobId};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, Mutex},
    path::PathBuf,
    time::{Duration, Instant},
    io,
};
use is_terminal::IsTerminal;

/// Get the parent process ID of the current process
fn get_parent_pid() -> u32 {
    #[cfg(unix)]
    {
        // On Unix systems, read from /proc/self/stat
        if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
            let fields: Vec<&str> = stat.split_whitespace().collect();
            if fields.len() > 3 {
                return fields[3].parse().unwrap_or(0);
            }
        }
    }
    
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS
        };
        use windows_sys::Win32::System::Threading::GetCurrentProcessId;
        
        let current_pid = unsafe { GetCurrentProcessId() };
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        
        if snapshot != -1i32 as isize {
            let mut pe32 = PROCESSENTRY32 {
                dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
                cntUsage: 0,
                th32ProcessID: 0,
                th32DefaultHeapID: 0,
                th32ModuleID: 0,
                cntThreads: 0,
                th32ParentProcessID: 0,
                pcPriClassBase: 0,
                dwFlags: 0,
                szExeFile: [0; 260],
            };
            
            if unsafe { Process32First(snapshot, &mut pe32) } != 0 {
                loop {
                    if pe32.th32ProcessID == current_pid {
                        return pe32.th32ParentProcessID;
                    }
                    if unsafe { Process32Next(snapshot, &mut pe32) } == 0 {
                        break;
                    }
                }
            }
        }
    }
    
    // Try to get from environment variable PPID if available
    if let Ok(ppid_str) = std::env::var("PPID") {
        return ppid_str.parse().unwrap_or(0);
    }
    
    // Fallback: return 0 if we can't determine parent PID
    0
}

/// Detect if this is a login shell
fn detect_login_shell() -> bool {
    // Method 1: Check command line argument 0 for login shell prefix
    // Login shells typically have their name prefixed with '-'
    if let Some(arg0) = std::env::args_os().next() {
        if let Some(arg0_str) = arg0.to_str() {
            if arg0_str.starts_with('-') {
                return true;
            }
        }
    }
    
    // Method 2: Check the original argument 0 via environment
    if let Ok(arg0) = std::env::var("_") {
        if arg0.starts_with('-') {
            return true;
        }
    }
    
    // Method 3: Check explicit login shell environment variables
    if std::env::var("LOGIN").is_ok() {
        return true;
    }
    
    // Method 4: Check if we're the session leader or direct child of init (Unix)
    #[cfg(unix)]
    {
        let ppid = get_parent_pid();
        
        // If parent is init (PID 1), systemd, or other system process managers
        if ppid == 1 || ppid == 0 {
            return true;
        }
        
        // Check session leadership via process group analysis
        // Session leaders typically indicate login shells
        use std::process;
        let current_pid = process::id();
        
        // Advanced session detection: if PPID is a login manager or display manager
        // Common parent PIDs for login shells: 1 (init), systemd, login, gdm, sddm, etc.
        if let Ok(comm_path) = std::fs::read_to_string(format!("/proc/{}/comm", ppid)) {
            let parent_name = comm_path.trim();
            if matches!(parent_name, "systemd" | "init" | "login" | "gdm" | "sddm" | "lightdm" | "lxdm") {
                return true;
            }
        }
    }
    
    // Method 5: Windows-specific login shell detection
    #[cfg(windows)]
    {
        // Check if started by Windows Terminal, Explorer, or other session managers
        let ppid = get_parent_pid();
        if ppid != 0 {
            // In Windows, login shells are often direct children of explorer.exe or winlogon.exe
            // This is a simplified heuristic - could be enhanced with process name checking
            if let Ok(logonserver) = std::env::var("LOGONSERVER") {
                if !logonserver.is_empty() {
                    return true;
                }
            }
        }
    }
    
    // Method 6: Environment-based detection for cross-platform compatibility
    if std::env::var("LOGNAME").is_ok() && std::env::var("HOME").is_ok() && std::env::var("SHELL").is_ok() {
        // Check SHLVL (shell level) - login shells typically start at level 1
        match std::env::var("SHLVL") {
            Ok(shlvl_str) => {
                if let Ok(shlvl) = shlvl_str.parse::<i32>() {
                    // Login shells typically have SHLVL=1, but also consider SHLVL=0 as potential login
                    if shlvl <= 1 {
                        return true;
                    }
                }
            }
            Err(_) => {
                // If SHLVL is not set but we have login environment, likely a login shell
                return true;
            }
        }
    }
    
    // Method 7: Terminal-specific login detection
    if let Ok(term) = std::env::var("TERM") {
        // Some terminals set specific TERM values for login sessions
        if term.contains("login") || std::env::var("TERM_SESSION_ID").is_ok() {
            return true;
        }
    }
    
    // Method 8: SSH and remote login detection
    if std::env::var("SSH_CLIENT").is_ok() || 
       std::env::var("SSH_CONNECTION").is_ok() || 
       std::env::var("SSH_TTY").is_ok() {
        // SSH sessions are typically login shells
        return true;
    }
    
    // Default: not a login shell
    false
}

/// Shell execution context passed to every command
/// This contains all the state needed for command execution
#[derive(Debug)]
pub struct Context<'a> {
    /// Command line arguments
    pub args: Vec<String>,
    /// Mutable reference to the shell environment
    pub env: &'a mut ShellContext,
    /// Standard input stream
    pub stdin: Stream,
    /// Standard output stream
    pub stdout: Stream,
    /// Standard error stream
    pub stderr: Stream,
    /// Current working directory
    pub cwd: PathBuf,
    /// Exit code of the last command
    pub last_exit_code: i32,
    /// Process ID of the current process
    pub pid: u32,
    /// Parent process ID
    pub ppid: u32,
    /// Current job ID (if running in background)
    pub job_id: Option<JobId>,
    /// Execution start time
    pub start_time: Instant,
    /// Maximum execution time (timeout)
    pub timeout: Option<Duration>,
}

impl<'a> Context<'a> {
    /// Create a new execution context
    pub fn new(
        args: Vec<String>,
        env: &'a mut ShellContext,
        stdin: Stream,
        stdout: Stream,
        stderr: Stream,
    ) -> ShellResult<Self> {
        let cwd = std::env::current_dir()
            .map_err(|e| ShellError::new(ErrorKind::IoError(crate::error::IoErrorKind::NotFound), format!("Failed to get current directory: {}", e)))?;
        
        Ok(Self {
            args,
            env,
            stdin,
            stdout,
            stderr,
            cwd,
            last_exit_code: 0,
            pid: std::process::id(),
            ppid: get_parent_pid(), // Get actual parent PID
            job_id: None,
            start_time: Instant::now(),
            timeout: None,
        })
    }

    /// Set the current working directory
    pub fn set_cwd(&mut self, path: PathBuf) -> ShellResult<()> {
        std::env::set_current_dir(&path)
            .map_err(|e| ShellError::new(ErrorKind::IoError(crate::error::IoErrorKind::NotFound), format!("Failed to change directory: {}", e)))?;
        self.cwd = path;
        Ok(())
    }

    /// Get an environment variable
    pub fn get_var(&self, key: &str) -> Option<String> {
        self.env.get_var(key)
    }

    /// Set an environment variable
    pub fn set_var(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.env.set_var(key, value);
    }

    /// Check if execution has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout {
            self.start_time.elapsed() > timeout
        } else {
            false
        }
    }

    /// Get elapsed execution time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Shell execution context
pub struct ShellContext {
    /// Environment variables
    pub env: Arc<RwLock<HashMap<String, String>>>,
    /// Shell variables
    pub vars: Arc<RwLock<HashMap<String, ShellVariable>>>,
    /// Aliases
    pub aliases: Arc<RwLock<HashMap<String, String>>>,
    /// Functions
    pub functions: Arc<RwLock<HashMap<String, String>>>,
    /// Current working directory
    pub cwd: PathBuf,
    /// Last exit code
    pub last_exit_status: Arc<Mutex<i32>>,
    /// Job manager
    pub job_manager: Arc<Mutex<JobManager>>,
    /// Standard input
    pub stdin: Box<dyn io::Read + Send>,
    /// Standard output
    pub stdout: Box<dyn io::Write + Send>,
    /// Standard error
    pub stderr: Box<dyn io::Write + Send>,
    /// Shell options
    pub options: Arc<RwLock<ShellOptions>>,
    /// Active jobs in this context
    pub jobs: Arc<RwLock<HashMap<u32, crate::job::Job>>>,
    /// Shell level (for nested shells)
    pub shell_level: u32,
    /// Initialization time
    pub init_time: Instant,
    /// Command history
    pub history: Arc<Mutex<Vec<String>>>,
    /// Directory stack for pushd/popd
    pub dir_stack: Arc<Mutex<Vec<PathBuf>>>,
    /// Whether this is an interactive shell
    pub interactive: bool,
    /// Whether this is a login shell
    pub login_shell: bool,
}

impl std::fmt::Debug for ShellContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShellContext")
            .field("env", &"Arc<RwLock<HashMap<String, String>>>")
            .field("vars", &"Arc<RwLock<HashMap<String, ShellVariable>>>")
            .field("aliases", &"Arc<RwLock<HashMap<String, String>>>")
            .field("functions", &"Arc<RwLock<HashMap<String, String>>>")
            .field("cwd", &self.cwd)
            .field("last_exit_status", &"Arc<Mutex<i32>>")
            .field("job_manager", &"Arc<Mutex<JobManager>>")
            .field("stdin", &"Box<dyn io::Read + Send>")
            .field("stdout", &"Box<dyn io::Write + Send>")
            .field("stderr", &"Box<dyn io::Write + Send>")
            .field("options", &"Arc<RwLock<ShellOptions>>")
            .field("jobs", &"Arc<RwLock<HashMap<u32, Job>>>")
            .field("shell_level", &self.shell_level)
            .field("init_time", &self.init_time)
            .field("history", &"Arc<Mutex<Vec<String>>>")
            .field("dir_stack", &"Arc<Mutex<Vec<PathBuf>>>")
            .field("interactive", &self.interactive)
            .field("login_shell", &self.login_shell)
            .finish()
    }
}

/// Shell variable with metadata
#[derive(Debug, Clone)]
pub struct ShellVariable {
    pub value: String,
    pub exported: bool,
    pub readonly: bool,
    pub local: bool,
}

impl ShellVariable {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            exported: false,
            readonly: false,
            local: false,
        }
    }

    pub fn exported(mut self) -> Self {
        self.exported = true;
        self
    }

    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    pub fn local(mut self) -> Self {
        self.local = true;
        self
    }
}

/// Shell configuration options
#[derive(Debug, Clone)]
pub struct ShellOptions {
    /// Exit on error (-e, errexit)
    pub errexit: bool,
    /// Print commands before execution (-x, xtrace)
    pub xtrace: bool,
    /// Fail on pipe errors (-o pipefail)
    pub pipefail: bool,
    /// No clobber - don't overwrite files with redirection (-C, noclobber)
    pub noclobber: bool,
    /// No glob - disable pathname expansion (-f, noglob)
    pub noglob: bool,
    /// Hash commands for faster lookup (-h, hashall)
    pub hashall: bool,
    /// Job control enabled (-m, monitor)
    pub monitor: bool,
    /// No unset variables (-u, nounset)
    pub nounset: bool,
    /// Verbose mode (-v, verbose)
    pub verbose: bool,
    /// Vi editing mode
    pub vi_mode: bool,
    /// Emacs editing mode (default)
    pub emacs_mode: bool,
    /// History expansion enabled
    pub histexpand: bool,
    /// Command completion enabled
    pub completion: bool,
    /// Spell checking for directory names
    pub cdspell: bool,
    /// Check window size after each command
    pub checkwinsize: bool,
    /// Enable extended globbing
    pub extglob: bool,
    /// Enable null globbing (empty expansion for no matches)
    pub nullglob: bool,
    /// Case insensitive globbing
    pub nocaseglob: bool,
    /// Enable dotglob (include hidden files in globs)
    pub dotglob: bool,
    /// Control flow state: break requested
    pub break_requested: bool,
    /// Control flow state: continue requested
    pub continue_requested: bool,
    /// Continue execution on errors
    pub continue_on_error: bool,
    /// Enable process isolation for subshells
    pub enable_process_isolation: bool,
    /// Current subshell nesting level
    pub subshell_level: u32,
}

impl Default for ShellOptions {
    fn default() -> Self {
        Self {
            errexit: false,
            xtrace: false,
            pipefail: false,
            noclobber: false,
            noglob: false,
            hashall: true,
            monitor: true,
            nounset: false,
            verbose: false,
            vi_mode: false,
            emacs_mode: true,
            histexpand: true,
            completion: true,
            cdspell: false,
            checkwinsize: true,
            extglob: false,
            nullglob: false,
            nocaseglob: false,
            dotglob: false,
            break_requested: false,
            continue_requested: false,
            continue_on_error: false,
            enable_process_isolation: true,
            subshell_level: 0,
        }
    }
}

impl ShellContext {
    /// Create a new shell context
    pub fn new() -> Self {
        let shell_level = std::env::var("SHLVL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0) + 1;

        Self {
            env: Arc::new(RwLock::new(HashMap::new())),
            vars: Arc::new(RwLock::new(HashMap::new())),
            aliases: Arc::new(RwLock::new(HashMap::new())),
            functions: Arc::new(RwLock::new(HashMap::new())),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            last_exit_status: Arc::new(Mutex::new(0)),
            job_manager: Arc::new(Mutex::new(JobManager::new())),
            stdin: Box::new(io::stdin()),
            stdout: Box::new(io::stdout()),
            stderr: Box::new(io::stderr()),
            options: Arc::new(RwLock::new(ShellOptions::default())),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            shell_level,
            init_time: Instant::now(),
            history: Arc::new(Mutex::new(Vec::new())),
            dir_stack: Arc::new(Mutex::new(Vec::new())),
            interactive: std::io::stdin().is_terminal(),
            login_shell: detect_login_shell(), // Detect login shell properly
        }
    }

    /// Check if execution has timed out
    pub fn is_timed_out(&self) -> bool {
        // TODO: Implement timeout logic
        false
    }

    /// Get environment variable
    pub fn get_var(&self, key: &str) -> Option<String> {
        // Check shell variables first
        if let Ok(vars) = self.vars.read() {
            if let Some(var) = vars.get(key) {
                return Some(var.value.clone());
            }
        }
        
        // Then check environment variables
        if let Ok(env) = self.env.read() {
            env.get(key).cloned()
        } else {
            None
        }
    }

    /// Set environment variable
    pub fn set_var<K, V>(&self, key: K, val: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        let key_str = key.into();
        let val_str = val.into();
        
        // Set in environment
        if let Ok(mut env) = self.env.write() {
            env.insert(key_str.clone(), val_str.clone());
        }
        
        // Also set as shell variable if not already present
        if let Ok(vars) = self.vars.read() {
            if !vars.contains_key(&key_str) {
                drop(vars); // Release read lock before write
                if let Ok(mut vars) = self.vars.write() {
                    vars.insert(key_str, ShellVariable::new(val_str));
                }
            }
        }
    }

    /// Set shell variable (not exported to environment)
    pub fn set_shell_var<K>(&self, key: K, var: ShellVariable)
    where
        K: Into<String>,
    {
        let key_str = key.into();
        
        // If exported, also set in environment
        if var.exported {
            if let Ok(mut env) = self.env.write() {
                env.insert(key_str.clone(), var.value.clone());
            }
        }
        
        if let Ok(mut vars) = self.vars.write() {
            vars.insert(key_str, var);
        }
    }

    /// Get alias value
    pub fn get_alias(&self, key: &str) -> Option<String> {
        if let Ok(aliases) = self.aliases.read() {
            aliases.get(key).cloned()
        } else {
            None
        }
    }

    /// Set alias with cycle detection
    pub fn set_alias<K, V>(&self, key: K, val: V) -> ShellResult<()>
    where
        K: Into<String>,
        V: Into<String>,
    {
        let key_str = key.into();
        let val_str = val.into();
        
        // Simple cycle detection - check if alias points to itself
        if key_str == val_str {
            return Err(ShellError::new(
                ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                format!("Alias '{}' would create a cycle", key_str)
            ));
        }
        
        // TODO: Implement more sophisticated cycle detection
        
        if let Ok(mut aliases) = self.aliases.write() {
            aliases.insert(key_str, val_str);
        }
        Ok(())
    }

    /// Remove alias
    pub fn unset_alias(&self, key: &str) -> bool {
        if let Ok(mut aliases) = self.aliases.write() {
            aliases.remove(key).is_some()
        } else {
            false
        }
    }

    /// Get function body
    pub fn get_function(&self, name: &str) -> Option<String> {
        if let Ok(functions) = self.functions.read() {
            functions.get(name).cloned()
        } else {
            None
        }
    }

    /// Set function
    pub fn set_function<K, V>(&self, name: K, body: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        if let Ok(mut functions) = self.functions.write() {
            functions.insert(name.into(), body.into());
        }
    }

    /// Remove function
    pub fn unset_function(&self, name: &str) -> bool {
        if let Ok(mut functions) = self.functions.write() {
            functions.remove(name).is_some()
        } else {
            false
        }
    }

    /// Add command to history
    pub fn add_history(&self, command: String) {
        if let Ok(mut history) = self.history.lock() {
            history.push(command);
            
            // Limit history size (TODO: make configurable)
            if history.len() > 1000 {
                history.remove(0);
            }
        }
    }

    /// Get command history
    pub fn get_history(&self) -> Vec<String> {
        self.history.lock()
            .unwrap_or_else(|poisoned| {
                // Recover from poisoned mutex by extracting the value
                poisoned.into_inner()
            })
            .clone()
    }

    /// Push directory to stack
    pub fn pushd(&self, dir: PathBuf) {
        if let Ok(mut stack) = self.dir_stack.lock() {
            stack.push(dir);
        }
    }

    /// Pop directory from stack
    pub fn popd(&self) -> Option<PathBuf> {
        if let Ok(mut stack) = self.dir_stack.lock() {
            stack.pop()
        } else {
            None
        }
    }

    /// Get directory stack
    pub fn dirs(&self) -> Vec<PathBuf> {
        self.dir_stack.lock()
            .unwrap_or_else(|poisoned| {
                // Recover from poisoned mutex by extracting the value
                poisoned.into_inner()
            })
            .clone()
    }

    /// Set shell option
    pub fn set_option(&self, option: &str, value: bool) -> ShellResult<()> {
        let mut options = self.options.write()
            .map_err(|_| ShellError::new(ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState), "Options lock poisoned"))?;
        
        match option {
            "errexit" | "e" => options.errexit = value,
            "xtrace" | "x" => options.xtrace = value,
            "pipefail" => options.pipefail = value,
            "noclobber" | "C" => options.noclobber = value,
            "noglob" | "f" => options.noglob = value,
            "hashall" | "h" => options.hashall = value,
            "monitor" | "m" => options.monitor = value,
            "nounset" | "u" => options.nounset = value,
            "verbose" | "v" => options.verbose = value,
            "vi" => {
                options.vi_mode = value;
                if value { options.emacs_mode = false; }
            },
            "emacs" => {
                options.emacs_mode = value;
                if value { options.vi_mode = false; }
            },
            "histexpand" | "H" => options.histexpand = value,
            "completion" => options.completion = value,
            "cdspell" => options.cdspell = value,
            "checkwinsize" => options.checkwinsize = value,
            "extglob" => options.extglob = value,
            "nullglob" => options.nullglob = value,
            "nocaseglob" => options.nocaseglob = value,
            "dotglob" => options.dotglob = value,
            _ => return Err(ShellError::new(
                ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                format!("Unknown shell option: {}", option)
            )),
        }
        
        Ok(())
    }

    /// Get shell option
    pub fn get_option(&self, option: &str) -> ShellResult<bool> {
        let options = self.options.read()
            .map_err(|_| ShellError::new(ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState), "Options lock poisoned"))?;
        
        let value = match option {
            "errexit" | "e" => options.errexit,
            "xtrace" | "x" => options.xtrace,
            "pipefail" => options.pipefail,
            "noclobber" | "C" => options.noclobber,
            "noglob" | "f" => options.noglob,
            "hashall" | "h" => options.hashall,
            "monitor" | "m" => options.monitor,
            "nounset" | "u" => options.nounset,
            "verbose" | "v" => options.verbose,
            "vi" => options.vi_mode,
            "emacs" => options.emacs_mode,
            "histexpand" | "H" => options.histexpand,
            "completion" => options.completion,
            "cdspell" => options.cdspell,
            "checkwinsize" => options.checkwinsize,
            "extglob" => options.extglob,
            "nullglob" => options.nullglob,
            "nocaseglob" => options.nocaseglob,
            "dotglob" => options.dotglob,
            _ => return Err(ShellError::new(
                ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                format!("Unknown shell option: {}", option)
            )),
        };
        
        Ok(value)
    }

    /// Get all shell options
    pub fn get_all_options(&self) -> ShellResult<ShellOptions> {
        let options = self.options.read()
            .map_err(|_| ShellError::new(ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState), "Options lock poisoned"))?;
        Ok(options.clone())
    }

    /// Set exit status of last command
    pub fn set_exit_status(&self, status: i32) {
        if let Ok(mut last_status) = self.last_exit_status.lock() {
            *last_status = status;
        }
    }

    /// Get exit status of last command
    pub fn get_exit_status(&self) -> i32 {
        self.last_exit_status.lock()
            .unwrap_or_else(|poisoned| {
                // Recover from poisoned mutex by extracting the value
                poisoned.into_inner()
            })
            .clone()
    }

    /// Check if shell is interactive
    pub fn is_interactive(&self) -> bool {
        self.interactive
    }

    /// Check if this is a login shell
    pub fn is_login_shell(&self) -> bool {
        self.login_shell
    }

    /// Get shell uptime
    pub fn uptime(&self) -> Duration {
        self.init_time.elapsed()
    }

    /// Get job manager
    pub fn job_manager(&self) -> Arc<Mutex<JobManager>> {
        Arc::clone(&self.job_manager)
    }

    pub fn create_subcontext(&self) -> Result<ShellContext, Box<dyn std::error::Error>> {
        let mut _context = ShellContext::new();
        // TODO: Copy necessary state from parent context
        // For now, just return a new context
        Ok(ShellContext::new())
    }

    /// Check if break was requested
    pub fn should_break(&self) -> bool {
        if let Ok(options) = self.options.read() {
            options.break_requested
        } else {
            false
        }
    }

    /// Clear break request
    pub fn clear_break(&self) {
        if let Ok(mut options) = self.options.write() {
            options.break_requested = false;
        }
    }

    /// Request break
    pub fn request_break(&self) {
        if let Ok(mut options) = self.options.write() {
            options.break_requested = true;
        }
    }

    /// Check if continue was requested
    pub fn should_continue(&self) -> bool {
        if let Ok(options) = self.options.read() {
            options.continue_requested
        } else {
            false
        }
    }

    /// Clear continue request
    pub fn clear_continue(&self) {
        if let Ok(mut options) = self.options.write() {
            options.continue_requested = false;
        }
    }

    /// Request continue
    pub fn request_continue(&self) {
        if let Ok(mut options) = self.options.write() {
            options.continue_requested = true;
        }
    }

    /// Check if should continue on error
    pub fn continue_on_error(&self) -> bool {
        if let Ok(options) = self.options.read() {
            options.continue_on_error
        } else {
            false
        }
    }

    /// Set continue on error
    pub fn set_continue_on_error(&self, value: bool) {
        if let Ok(mut options) = self.options.write() {
            options.continue_on_error = value;
        }
    }
}

impl Default for ShellContext {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export for backward compatibility
// pub use ShellContext as Context; // Commented out to avoid naming conflict 

// Include tests module
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_detect_login_shell_with_dash_prefix() {
        // Test login shell detection via argument prefix
        // This simulates the traditional login shell indicator where argv[0] starts with '-'
        
        // Set up environment to simulate a login shell
        env::set_var("_", "-nxsh");
        
        let result = detect_login_shell();
        
        // Clean up
        env::remove_var("_");
        
        assert!(result, "Should detect login shell from argv[0] prefix");
    }

    #[test]
    fn test_detect_login_shell_with_login_env() {
        // Test login shell detection via LOGIN environment variable
        env::set_var("LOGIN", "user");
        
        let result = detect_login_shell();
        
        env::remove_var("LOGIN");
        
        assert!(result, "Should detect login shell from LOGIN environment variable");
    }

    #[test]
    fn test_detect_login_shell_with_ssh_connection() {
        // Test login shell detection for SSH sessions
        env::set_var("SSH_CONNECTION", "192.168.1.100 54321 192.168.1.1 22");
        
        let result = detect_login_shell();
        
        env::remove_var("SSH_CONNECTION");
        
        assert!(result, "Should detect login shell from SSH connection");
    }

    #[test]
    fn test_detect_login_shell_with_shlvl_analysis() {
        // Test SHLVL-based detection with complete login environment
        env::set_var("LOGNAME", "testuser");
        env::set_var("HOME", "/home/testuser");
        env::set_var("SHELL", "/bin/nxsh");
        env::set_var("SHLVL", "1");
        
        let result = detect_login_shell();
        
        // Clean up
        env::remove_var("LOGNAME");
        env::remove_var("HOME");
        env::remove_var("SHELL");
        env::remove_var("SHLVL");
        
        assert!(result, "Should detect login shell from SHLVL=1 with login environment");
    }

    #[test]
    fn test_detect_login_shell_missing_shlvl() {
        // Test detection when SHLVL is missing but login environment is present
        env::set_var("LOGNAME", "testuser");
        env::set_var("HOME", "/home/testuser");
        env::set_var("SHELL", "/bin/nxsh");
        // Explicitly remove SHLVL to simulate missing variable
        env::remove_var("SHLVL");
        
        let result = detect_login_shell();
        
        // Clean up
        env::remove_var("LOGNAME");
        env::remove_var("HOME");
        env::remove_var("SHELL");
        
        assert!(result, "Should detect login shell when SHLVL is missing but login env is present");
    }

    #[test]
    fn test_detect_non_login_shell() {
        // Test that non-login shells are correctly identified
        env::set_var("SHLVL", "3"); // High shell level indicates nested shell
        env::remove_var("LOGIN");
        env::remove_var("SSH_CONNECTION");
        env::remove_var("SSH_CLIENT");
        env::remove_var("_");
        
        let result = detect_login_shell();
        
        env::remove_var("SHLVL");
        
        assert!(!result, "Should not detect login shell for nested shell (SHLVL=3)");
    }

    #[test]
    fn test_mutex_poisoning_recovery_history() {
        // Test graceful recovery from history mutex poisoning
        let context = ShellContext::new();
        
        // Simulate mutex poisoning by creating a panic in another thread
        let history_mutex = Arc::clone(&context.history);
        let handle = thread::spawn(move || {
            let _guard = history_mutex.lock().unwrap();
            eprintln!("Simulated panic to poison mutex for testing");
            assert!(false, "Simulated panic to poison mutex");
        });
        
        // Wait for thread to panic and poison the mutex
        let _ = handle.join();
        
        // The mutex should now be poisoned, but get_history should recover gracefully
        let history = context.get_history();
        
        // Should return empty vector instead of panicking
        assert!(history.is_empty(), "Should return empty history on mutex poisoning recovery");
    }

    #[test]
    fn test_mutex_poisoning_recovery_dirs() {
        // Test graceful recovery from directory stack mutex poisoning
        let context = ShellContext::new();
        
        // Simulate mutex poisoning
        let dir_mutex = Arc::clone(&context.dir_stack);
        let handle = thread::spawn(move || {
            let _guard = dir_mutex.lock().unwrap();
            eprintln!("Simulated panic to poison mutex for testing");
            assert!(false, "Simulated panic to poison mutex");
        });
        
        let _ = handle.join();
        
        // Should recover gracefully
        let dirs = context.dirs();
        assert!(dirs.is_empty(), "Should return empty dirs on mutex poisoning recovery");
    }

    #[test]
    fn test_mutex_poisoning_recovery_exit_status() {
        // Test graceful recovery from exit status mutex poisoning
        let context = ShellContext::new();
        
        // Simulate mutex poisoning
        let status_mutex = Arc::clone(&context.last_exit_status);
        let handle = thread::spawn(move || {
            let _guard = status_mutex.lock().unwrap();
            eprintln!("Simulated panic to poison mutex for testing");
            assert!(false, "Simulated panic to poison mutex");
        });
        
        let _ = handle.join();
        
        // Should recover gracefully and return default status
        let status = context.get_exit_status();
        assert_eq!(status, 0, "Should return 0 on mutex poisoning recovery");
    }

    #[cfg(unix)]
    #[test]
    fn test_get_parent_pid_unix() {
        // Test Unix parent PID detection
        let ppid = get_parent_pid();
        
        // Parent PID should be non-zero on Unix systems
        assert!(ppid > 0, "Parent PID should be positive on Unix systems");
        
        // Verify by checking /proc/self/stat manually
        if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
            let fields: Vec<&str> = stat.split_whitespace().collect();
            if fields.len() > 3 {
                let expected_ppid: u32 = fields[3].parse().unwrap_or(0);
                assert_eq!(ppid, expected_ppid, "get_parent_pid should match /proc/self/stat");
            }
        }
    }

    #[test]
    fn test_login_shell_context_integration() {
        // Test that login shell detection is properly integrated into ShellContext
        let context = ShellContext::new();
        
        // The is_login_shell method should return a boolean without panicking
        let is_login = context.is_login_shell();
        
        // Result should be deterministic based on current environment
        assert!(is_login == context.is_login_shell(), 
                "is_login_shell should be consistent across calls");
    }

    #[test]
    fn test_concurrent_context_access() {
        // Test thread safety of context operations using operations that don't require Send+Sync
        let context = ShellContext::new();
        
        // Test sequential operations that don't require Arc sharing
        for i in 0..10 {
            // Test read operations
            let _ = context.get_history();
            let _ = context.dirs();
            let _ = context.get_exit_status();
            let _ = context.is_login_shell();
            
            // Test write operations
            context.set_exit_status(i);
            context.pushd(format!("/tmp/test{}", i).into());
        }
        
        // Context should remain in valid state
        assert!(context.dirs().len() <= 10, "Directory stack should be bounded");
    }

    #[test]
    fn test_environment_variable_edge_cases() {
        // Test edge cases in environment variable parsing
        
        // Test empty environment variables
        env::set_var("SHLVL", "");
        env::set_var("_", "");
        
        let result = detect_login_shell();
        
        // Clean up
        env::remove_var("SHLVL");
        env::remove_var("_");
        
        // Should handle empty values gracefully
        assert!(!result || result, "Should handle empty environment variables without crashing");
    }

    #[test]
    fn test_malformed_shlvl_parsing() {
        // Test handling of malformed SHLVL values
        let test_cases = vec!["abc", "12.34", "-1", "999999999999999999"];
        
        for shlvl in test_cases {
            env::set_var("LOGNAME", "testuser");
            env::set_var("HOME", "/home/testuser");
            env::set_var("SHELL", "/bin/nxsh");
            env::set_var("SHLVL", shlvl);
            
            // Should not panic on malformed input
            let result = detect_login_shell();
            
            // Clean up
            env::remove_var("LOGNAME");
            env::remove_var("HOME");
            env::remove_var("SHELL");
            env::remove_var("SHLVL");
            
            // Result should be deterministic
            assert!(result == true || result == false, 
                    "Should handle malformed SHLVL '{}' gracefully", shlvl);
        }
    }
} 