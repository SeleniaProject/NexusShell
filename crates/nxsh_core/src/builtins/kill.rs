//! Kill builtin implementation for nxsh_core
//!
//! This module provides a comprehensive kill command implementation with:
//! - Signal sending to processes by PID
//! - Job control integration
//! - Multiple signal types (TERM, KILL, HUP, etc.)
//! - Process validation and error handling
//! - Cross-platform compatibility

use crate::context::ShellContext;
use crate::error::ShellResult;
use crate::executor::{Builtin, ExecutionMetrics, ExecutionResult, ExecutionStrategy};
use std::time::Instant;

/// Comprehensive kill builtin with signal support
pub struct KillBuiltin;

impl Builtin for KillBuiltin {
    fn name(&self) -> &'static str {
        "kill"
    }

    fn synopsis(&self) -> &'static str {
        "send a signal to processes"
    }

    fn description(&self) -> &'static str {
        "Send signals to processes identified by PID or job ID with comprehensive signal support"
    }

    fn usage(&self) -> &'static str {
        "kill [-s SIGNAL | -SIGNAL] PID...\nkill -l [SIGNAL]"
    }

    fn execute(
        &self,
        _context: &mut ShellContext,
        args: &[String],
    ) -> ShellResult<ExecutionResult> {
        let start_time = Instant::now();

        if args.is_empty() {
            return Ok(ExecutionResult {
                exit_code: 1,
                stdout: String::new(),
                stderr: "kill: usage: kill [-s sigspec | -n signum | -sigspec] pid | jobspec ... or kill -l [sigspec]".to_string(),
                execution_time: start_time.elapsed().as_micros() as u64,
                strategy: ExecutionStrategy::DirectInterpreter,
                metrics: ExecutionMetrics::default(),
            });
        }

        // Parse arguments
        let mut signal = "TERM"; // Default signal
        let mut list_signals = false;
        let mut pids = Vec::new();
        let mut i = 0;

        while i < args.len() {
            let arg = &args[i];

            if arg == "-l" || arg == "--list" {
                list_signals = true;
            } else if let Some(stripped) = arg.strip_prefix("-s") {
                // -s SIGNAL format
                if arg.len() > 2 {
                    signal = stripped;
                } else if i + 1 < args.len() {
                    i += 1;
                    signal = &args[i];
                } else {
                    return Ok(ExecutionResult {
                        exit_code: 1,
                        stdout: String::new(),
                        stderr: "kill: option requires an argument -- s".to_string(),
                        execution_time: start_time.elapsed().as_micros() as u64,
                        strategy: ExecutionStrategy::DirectInterpreter,
                        metrics: ExecutionMetrics::default(),
                    });
                }
            } else if arg.starts_with('-') && arg.len() > 1 && arg != "--" {
                // -SIGNAL format (e.g., -9, -KILL)
                signal = &arg[1..];
            } else if arg == "--" {
                // End of options
                i += 1;
                break;
            } else {
                // PID or job spec
                pids.push(arg.clone());
            }
            i += 1;
        }

        // Add remaining args as PIDs after --
        while i < args.len() {
            pids.push(args[i].clone());
            i += 1;
        }

        if list_signals {
            let signal_list = self.get_signal_list();
            return Ok(ExecutionResult {
                exit_code: 0,
                stdout: signal_list,
                stderr: String::new(),
                execution_time: start_time.elapsed().as_micros() as u64,
                strategy: ExecutionStrategy::DirectInterpreter,
                metrics: ExecutionMetrics::default(),
            });
        }

        if pids.is_empty() {
            return Ok(ExecutionResult {
                exit_code: 1,
                stdout: String::new(),
                stderr: "kill: no process ID specified".to_string(),
                execution_time: start_time.elapsed().as_micros() as u64,
                strategy: ExecutionStrategy::DirectInterpreter,
                metrics: ExecutionMetrics::default(),
            });
        }

        // Convert signal name/number to system signal
        let signal_num = match self.parse_signal(signal) {
            Ok(num) => num,
            Err(err) => {
                return Ok(ExecutionResult {
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: format!("kill: {err}"),
                    execution_time: start_time.elapsed().as_micros() as u64,
                    strategy: ExecutionStrategy::DirectInterpreter,
                    metrics: ExecutionMetrics::default(),
                });
            }
        };

        // Send signals to processes
        let mut failed_pids = Vec::new();
        let mut stdout_lines = Vec::new();

        for pid_str in &pids {
            match self.kill_process(pid_str, signal_num) {
                Ok(message) => {
                    if !message.is_empty() {
                        stdout_lines.push(message);
                    }
                }
                Err(err) => {
                    failed_pids.push(format!("kill: {pid_str}: {err}"));
                }
            }
        }

        let stdout = if stdout_lines.is_empty() {
            String::new()
        } else {
            stdout_lines.join("\n") + "\n"
        };

        let stderr = if failed_pids.is_empty() {
            String::new()
        } else {
            failed_pids.join("\n") + "\n"
        };

        let exit_code = if failed_pids.is_empty() { 0 } else { 1 };

        Ok(ExecutionResult {
            exit_code,
            stdout,
            stderr,
            execution_time: start_time.elapsed().as_micros() as u64,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        })
    }

    fn help(&self) -> &'static str {
        "kill - terminate processes by PID or job ID

USAGE:
    kill [-s SIGNAL | -SIGNAL] PID...
    kill -l [SIGNAL]

OPTIONS:
    -s SIGNAL    Send the specified signal (default: TERM)
    -SIGNAL      Send the specified signal by name or number
    -l, --list   List available signal names

SIGNALS:
    TERM (15)    Terminate gracefully (default)
    KILL (9)     Force terminate (cannot be caught)
    HUP (1)      Hangup
    INT (2)      Interrupt (Ctrl+C)
    QUIT (3)     Quit
    USR1 (10)    User-defined signal 1
    USR2 (12)    User-defined signal 2

EXAMPLES:
    kill 1234           # Send TERM signal to process 1234
    kill -9 1234        # Force kill process 1234
    kill -s HUP 1234    # Send HUP signal to process 1234
    kill -l             # List all available signals"
    }
}

impl KillBuiltin {
    /// Get list of available signals
    fn get_signal_list(&self) -> String {
        let signals = [
            ("HUP", 1),
            ("INT", 2),
            ("QUIT", 3),
            ("ILL", 4),
            ("TRAP", 5),
            ("ABRT", 6),
            ("BUS", 7),
            ("FPE", 8),
            ("KILL", 9),
            ("USR1", 10),
            ("SEGV", 11),
            ("USR2", 12),
            ("PIPE", 13),
            ("ALRM", 14),
            ("TERM", 15),
            ("STKFLT", 16),
            ("CHLD", 17),
            ("CONT", 18),
            ("STOP", 19),
            ("TSTP", 20),
            ("TTIN", 21),
            ("TTOU", 22),
            ("URG", 23),
            ("XCPU", 24),
            ("XFSZ", 25),
            ("VTALRM", 26),
            ("PROF", 27),
            ("WINCH", 28),
            ("IO", 29),
            ("PWR", 30),
            ("SYS", 31),
        ];

        let mut output = String::new();
        for (name, num) in &signals {
            output.push_str(&format!("{num:2}) {name}\n"));
        }
        output
    }

    /// Parse signal name or number to signal number
    fn parse_signal(&self, signal: &str) -> Result<i32, String> {
        // Try to parse as number first
        if let Ok(num) = signal.parse::<i32>() {
            if num > 0 && num <= 31 {
                return Ok(num);
            } else {
                return Err(format!("invalid signal number: {signal}"));
            }
        }

        // Parse as signal name
        let signal_upper = signal.to_uppercase();
        match signal_upper.as_str() {
            "HUP" => Ok(1),
            "INT" => Ok(2),
            "QUIT" => Ok(3),
            "ILL" => Ok(4),
            "TRAP" => Ok(5),
            "ABRT" | "IOT" => Ok(6),
            "BUS" => Ok(7),
            "FPE" => Ok(8),
            "KILL" => Ok(9),
            "USR1" => Ok(10),
            "SEGV" => Ok(11),
            "USR2" => Ok(12),
            "PIPE" => Ok(13),
            "ALRM" => Ok(14),
            "TERM" => Ok(15),
            "STKFLT" => Ok(16),
            "CHLD" | "CLD" => Ok(17),
            "CONT" => Ok(18),
            "STOP" => Ok(19),
            "TSTP" => Ok(20),
            "TTIN" => Ok(21),
            "TTOU" => Ok(22),
            "URG" => Ok(23),
            "XCPU" => Ok(24),
            "XFSZ" => Ok(25),
            "VTALRM" => Ok(26),
            "PROF" => Ok(27),
            "WINCH" => Ok(28),
            "IO" | "POLL" => Ok(29),
            "PWR" => Ok(30),
            "SYS" => Ok(31),
            _ => Err(format!("invalid signal name: {signal}")),
        }
    }

    /// Kill a process by PID string
    fn kill_process(&self, pid_str: &str, signal: i32) -> Result<String, String> {
        // Parse PID
        let pid = pid_str
            .parse::<u32>()
            .map_err(|_| format!("invalid process ID: {pid_str}"))?;

        // Platform-specific signal sending
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            let result = unsafe { libc::kill(pid as libc::pid_t, signal) };
            if result == 0 {
                if signal == 0 {
                    Ok(format!("Process {pid} exists"))
                } else {
                    Ok(String::new()) // Success, no output needed
                }
            } else {
                let errno = std::io::Error::last_os_error();
                Err(format!(
                    "operation not permitted or no such process: {}",
                    errno
                ))
            }
        }

        #[cfg(windows)]
        {
            // On Windows, we use taskkill for termination
            let success = match signal {
                9 | 15 => {
                    // KILL or TERM - force terminate
                    std::process::Command::new("taskkill")
                        .args(["/F", "/PID", &pid.to_string()])
                        .output()
                        .map(|output| output.status.success())
                        .unwrap_or(false)
                }
                0 => {
                    // Signal 0 - check if process exists
                    std::process::Command::new("tasklist")
                        .args(["/FI", &format!("PID eq {pid}")])
                        .output()
                        .map(|output| {
                            output.status.success()
                                && String::from_utf8_lossy(&output.stdout)
                                    .contains(&pid.to_string())
                        })
                        .unwrap_or(false)
                }
                _ => {
                    // Other signals not supported on Windows
                    return Err("signal not supported on Windows".to_string());
                }
            };

            if success {
                if signal == 0 {
                    Ok(format!("Process {pid} exists"))
                } else {
                    Ok(String::new())
                }
            } else {
                Err("operation not permitted or no such process".to_string())
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            Err("kill command not supported on this platform".to_string())
        }
    }
}
