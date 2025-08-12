use std::process;
use nxsh_core::{ShellError, ErrorKind}; use nxsh_core::error::SystemErrorKind;

pub fn signal_cli(args: &[String]) -> Result<(), ShellError> {
    if args.is_empty() || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let mut list_signals = false;
    let mut signal_name = None;
    let mut pids = Vec::new();
    
    for arg in args {
        match arg.as_str() {
            "-l" | "--list" => list_signals = true,
            arg if arg.starts_with("-") && arg.len() > 1 => {
                signal_name = Some(arg[1..].to_string());
            },
            _ => {
                if let Ok(pid) = arg.parse::<u32>() {
                    pids.push(pid);
                } else {
                    signal_name = Some(arg.clone());
                }
            }
        }
    }
    
    if list_signals {
        list_available_signals();
        return Ok(());
    }
    
    let sig = signal_name.unwrap_or_else(|| "TERM".to_string());
    
    for pid in pids {
        send_signal_to_process(pid, &sig)?;
    }
    
    Ok(())
}

fn print_help() {
    println!("signal - send signals to processes

USAGE:
    signal [-l] [-SIGNAL] PID...
    signal --list
    signal SIGNAL PID...

DESCRIPTION:
    Send a signal to one or more processes by PID.

OPTIONS:
    -l, --list     List available signals
    -SIGNAL        Signal to send (default: TERM)

SIGNALS:
    TERM    Terminate (15)
    KILL    Kill (9) 
    HUP     Hangup (1)
    INT     Interrupt (2)
    QUIT    Quit (3)
    STOP    Stop (19)
    CONT    Continue (18)
    USR1    User signal 1 (10)
    USR2    User signal 2 (12)

EXAMPLES:
    signal 1234           # Send SIGTERM to PID 1234
    signal -KILL 1234     # Send SIGKILL to PID 1234
    signal HUP 1234 5678  # Send SIGHUP to PIDs 1234 and 5678");
}

fn list_available_signals() {
    println!("Available signals:");
    let signals = [
        ("HUP", 1, "Hangup"),
        ("INT", 2, "Interrupt"),
        ("QUIT", 3, "Quit"),
        ("TERM", 15, "Terminate"),
        ("KILL", 9, "Kill"),
        ("USR1", 10, "User signal 1"),
        ("USR2", 12, "User signal 2"),
        ("STOP", 19, "Stop"),
        ("CONT", 18, "Continue"),
    ];
    
    for (name, num, desc) in &signals {
        println!("{num:2}) SIG{name:<4} {desc}");
    }
}

fn send_signal_to_process(pid: u32, signal_name: &str) -> Result<(), ShellError> {
    // On Windows, we use taskkill for terminating processes
    #[cfg(windows)]
    {
        match signal_name.to_uppercase().as_str() {
            "TERM" | "KILL" | "9" | "15" => {
                match process::Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/F"])
                    .status()
                {
                    Ok(status) if status.success() => {
                        println!("Signal sent to process {pid}");
                        Ok(())
                    },
                    Ok(_) => Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::ProcessError), format!("Failed to signal process {pid}"))),
                    Err(e) => Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::ProcessError), format!("Failed to execute taskkill: {e}"))),
                }
            },
            _ => {
                println!("Warning: Signal {signal_name} not supported on Windows, using SIGTERM");
                send_signal_to_process(pid, "TERM")
            }
        }
    }
    
    // On Unix-like systems, we use the kill command
    #[cfg(unix)]
    {
        let signal_arg = match signal_name.to_uppercase().as_str() {
            "HUP" | "1" => "-1",
            "INT" | "2" => "-2",
            "QUIT" | "3" => "-3",
            "KILL" | "9" => "-9",
            "USR1" | "10" => "-10",
            "USR2" | "12" => "-12",
            "TERM" | "15" => "-15",
            "STOP" | "19" => "-19",
            "CONT" | "18" => "-18",
            other => return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), format!("Unknown signal: {}", other))),
        };
        
        match process::Command::new("kill")
            .args(&[signal_arg, &pid.to_string()])
            .status()
        {
            Ok(status) if status.success() => {
                println!("Signal {} sent to process {}", signal_name, pid);
                Ok(())
            },
            Ok(_) => Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::ProcessError), format!("Failed to signal process {}", pid))),
            Err(e) => Err(ShellError::new(ErrorKind::SystemError(SystemErrorKind::ProcessError), format!("Failed to execute kill: {}", e))),
        }
    }
}

// Helper function to validate signal names
pub fn is_valid_signal(signal: &str) -> bool {
    matches!(signal.to_uppercase().as_str(), 
        "HUP" | "INT" | "QUIT" | "TERM" | "KILL" | "USR1" | "USR2" | "STOP" | "CONT" |
        "1" | "2" | "3" | "9" | "10" | "12" | "15" | "18" | "19"
    )
}

// Helper function to get signal number from name
pub fn signal_name_to_number(signal: &str) -> Option<i32> {
    match signal.to_uppercase().as_str() {
        "HUP" | "1" => Some(1),
        "INT" | "2" => Some(2),
        "QUIT" | "3" => Some(3),
        "KILL" | "9" => Some(9),
        "USR1" | "10" => Some(10),
        "USR2" | "12" => Some(12),
        "TERM" | "15" => Some(15),
        "CONT" | "18" => Some(18),
        "STOP" | "19" => Some(19),
        _ => None,
    }
}

