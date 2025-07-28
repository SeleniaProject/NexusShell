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
use dashmap::DashMap;

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
            ppid: 0, // TODO: Get actual parent PID via HAL
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
    pub env: Arc<DashMap<String, String>>,
    /// Shell variables
    pub vars: Arc<DashMap<String, ShellVariable>>,
    /// Aliases
    pub aliases: Arc<DashMap<String, String>>,
    /// Functions
    pub functions: Arc<DashMap<String, String>>,
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
            .field("env", &"Arc<DashMap<String, String>>")
            .field("vars", &"Arc<DashMap<String, ShellVariable>>")
            .field("aliases", &"Arc<DashMap<String, String>>")
            .field("functions", &"Arc<DashMap<String, String>>")
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
            env: Arc::new(DashMap::new()),
            vars: Arc::new(DashMap::new()),
            aliases: Arc::new(DashMap::new()),
            functions: Arc::new(DashMap::new()),
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
            interactive: atty::is(atty::Stream::Stdin),
            login_shell: false, // TODO: Detect login shell properly
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
        if let Some(var) = self.vars.get(key) {
            return Some(var.value.clone());
        }
        
        // Then check environment variables
        self.env.get(key).map(|v| v.clone())
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
        self.env.insert(key_str.clone(), val_str.clone());
        
        // Also set as shell variable if not already present
        if !self.vars.contains_key(&key_str) {
            self.vars.insert(key_str, ShellVariable::new(val_str));
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
            self.env.insert(key_str.clone(), var.value.clone());
        }
        
        self.vars.insert(key_str, var);
    }

    /// Get alias value
    pub fn get_alias(&self, key: &str) -> Option<String> {
        self.aliases.get(key).map(|v| v.clone())
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
        
        self.aliases.insert(key_str, val_str);
        Ok(())
    }

    /// Remove alias
    pub fn unset_alias(&self, key: &str) -> bool {
        self.aliases.remove(key).is_some()
    }

    /// Get function body
    pub fn get_function(&self, name: &str) -> Option<String> {
        self.functions.get(name).map(|v| v.clone())
    }

    /// Set function
    pub fn set_function<K, V>(&self, name: K, body: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.functions.insert(name.into(), body.into());
    }

    /// Remove function
    pub fn unset_function(&self, name: &str) -> bool {
        self.functions.remove(name).is_some()
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
        self.history.lock().unwrap_or_else(|_| panic!("History lock poisoned")).clone()
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
        self.dir_stack.lock().unwrap_or_else(|_| panic!("Directory stack lock poisoned")).clone()
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
        self.last_exit_status.lock().unwrap_or_else(|_| panic!("Exit status lock poisoned")).clone()
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
}

impl Default for ShellContext {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export for backward compatibility
// pub use ShellContext as Context; // Commented out to avoid naming conflict 