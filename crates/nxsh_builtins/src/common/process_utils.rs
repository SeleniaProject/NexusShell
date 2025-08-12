use std::process::Command;
use nxsh_core::{ShellError, ErrorKind, ShellResult};
use nxsh_core::error::SystemErrorKind;

pub fn execute_kill_target(target_pid: u32, _signal: i32) -> ShellResult<()> {
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        
        let pid = Pid::from_raw(target_pid as i32);
        let sig = Signal::try_from(signal).map_err(|_| {
            ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Invalid signal: {}", signal))
        })?;
        
        kill(pid, Some(sig)).map_err(|e| {
            ShellError::new(ErrorKind::SystemError(SystemErrorKind::ProcessError), format!("Failed to send signal: {}", e))
        })?;
    }
    
    #[cfg(windows)]
    {
        let status = Command::new("taskkill")
            .arg("/PID")
            .arg(target_pid.to_string())
            .arg("/F")
            .status()
            .map_err(|e| ShellError::new(ErrorKind::SystemError(SystemErrorKind::ProcessError), format!("Failed to kill process: {e}")))?;
            
        if !status.success() {
            return Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::ProcessError), "Failed to terminate process"));
        }
    }
    
    Ok(())
}

pub fn execute_uptime_command() -> ShellResult<String> {
    #[cfg(unix)]
    {
        let output = Command::new("uptime")
            .output()
            .map_err(|e| ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound), format!("Failed to execute uptime: {}", e)))?;
            
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound), "uptime command failed"))
        }
    }
    
    #[cfg(windows)]
    {
        // Windows: use systeminfo or wmic to get boot time
        let output = Command::new("wmic")
            .args(["os", "get", "lastbootuptime", "/value"])
            .output()
            .map_err(|e| ShellError::new(ErrorKind::SystemError(SystemErrorKind::ProcessError), format!("Failed to get uptime: {e}")))?;
            
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Parse WMIC output and format as uptime
            Ok(format!("System uptime information: {}", output_str.trim()))
        } else {
            Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::ProcessError), "Failed to get system uptime"))
        }
    }
}
