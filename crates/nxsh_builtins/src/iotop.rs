use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

/// Entry point for the `iotop` builtin
pub fn iotop_cli(args: &[String]) -> Result<()> {
    // Try external binary first
    if let Ok(path) = which("iotop") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("iotop: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // Basic internal fallback
    if args.is_empty() || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        println!("iotop: Display I/O usage by processes");
        println!("Usage: iotop [options]");
        println!("Options:");
        println!("  -o, --only      Only show processes that have I/O activity");
        println!("  -b, --batch     Non-interactive mode");
        println!("  -n NUM          Number of iterations before exit");
        println!("  -d SEC          Delay between iterations");
        println!("  -p PID          Monitor processes with specified PIDs");
        println!("  -u USER         Monitor processes owned by specified user");
        println!("  -a, --accumulated    Show accumulated I/O instead of bandwidth");
        println!("  -k, --kilobytes      Use kilobytes instead of human-friendly units");
        println!("  -t, --time           Add timestamp on each line");
        println!("  -q, --quiet          Suppress some lines of header");
        println!("  -h, --help           Show this help");
        return Ok(());
    }

    println!("iotop: Process I/O monitor (internal fallback)");
    println!("iotop: Install iotop package for real-time I/O monitoring");
    
    // Show basic I/O information
    println!("\nProcess I/O Overview:");
    
    #[cfg(unix)]
    {
        println!("System I/O Statistics:");
        let _ = Command::new("iostat").args(&["-x", "1", "1"]).status();
        
        println!("\nTop I/O Processes (ps with memory/cpu):");
        let _ = Command::new("ps")
            .args(&["aux", "--sort=-%mem"])
            .status();
        
        println!("\nDisk Usage:");
        let _ = Command::new("df").args(&["-h"]).status();
    }
    
    #[cfg(windows)]
    {
        println!("Process List with Resource Usage:");
        let _ = Command::new("tasklist")
            .args(&["/v", "/fo", "table"])
            .status();
        
        println!("\nDisk Performance Counters:");
        let _ = Command::new("wmic")
            .args(&["logicaldisk", "get", "size,freespace,caption"])
            .status();
        
        println!("\nPerformance Data:");
        let _ = Command::new("typeperf")
            .args(&["-sc", "1", "\\LogicalDisk(_Total)\\Disk Bytes/sec"])
            .status();
    }

    Ok(())
}

