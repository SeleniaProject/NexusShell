use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

/// Entry point for the `sar` builtin
pub fn sar_cli(args: &[String]) -> Result<()> {
    // Try external binary first
    if let Ok(path) = which("sar") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("sar: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // Basic internal fallback
    if args.is_empty() || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        println!("sar: System Activity Reporter");
        println!("Usage: sar [options] [interval [count]]");
        println!("Options:");
        println!("  -u    CPU utilization");
        println!("  -r    Memory utilization");
        println!("  -d    Disk activity");
        println!("  -n    Network statistics");
        println!("  -q    Queue length and load averages");
        println!("  -A    All statistics");
        println!("  -h    Show this help");
        return Ok(());
    }

    println!("sar: System Activity Reporter (internal fallback)");
    println!("sar: Install sysstat package for full functionality");
    
    // Show basic system information
    println!("\nSystem Overview:");
    
    #[cfg(unix)]
    {
        println!("Load Average:");
        let _ = Command::new("uptime").status();
        
        println!("\nCPU Info:");
        let _ = Command::new("lscpu").status();
        
        println!("\nMemory Usage:");
        let _ = Command::new("free").args(&["-h"]).status();
    }
    
    #[cfg(windows)]
    {
        println!("System Information:");
        let _ = Command::new("systeminfo").status();
    }

    Ok(())
}
