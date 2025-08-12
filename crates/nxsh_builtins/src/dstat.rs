use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

/// Entry point for the `dstat` builtin
pub fn dstat_cli(args: &[String]) -> Result<()> {
    // Try external binary first
    if let Ok(path) = which("dstat") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("dstat: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // Basic internal fallback
    if args.is_empty() || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        println!("dstat: Versatile system resource statistics");
        println!("Usage: dstat [options] [delay [count]]");
        println!("Options:");
        println!("  -c    CPU statistics");
        println!("  -d    Disk statistics");
        println!("  -n    Network statistics");
        println!("  -m    Memory statistics");
        println!("  -l    Load average");
        println!("  -s    Swap statistics");
        println!("  -a    All statistics (same as -cdnm)");
        println!("  -h    Show this help");
        return Ok(());
    }

    println!("dstat: System resource statistics (internal fallback)");
    println!("dstat: Install dstat package for full functionality");
    
    // Show basic resource information
    println!("\nSystem Resource Overview:");
    
    #[cfg(unix)]
    {
        println!("CPU and Load:");
        let _ = Command::new("uptime").status();
        
        println!("\nMemory:");
        let _ = Command::new("free").args(&["-h"]).status();
        
        println!("\nDisk Usage:");
        let _ = Command::new("df").args(&["-h"]).status();
        
        println!("\nNetwork Interfaces:");
        let _ = Command::new("ip").args(&["addr", "show"]).status();
    }
    
    #[cfg(windows)]
    {
        println!("CPU Usage:");
        let _ = Command::new("wmic")
            .args(&["cpu", "get", "loadpercentage", "/value"])
            .status();
        
        println!("\nMemory Usage:");
        let _ = Command::new("wmic")
            .args(&["OS", "get", "TotalVisibleMemorySize,FreePhysicalMemory", "/value"])
            .status();
        
        println!("\nDisk Usage:");
        let _ = Command::new("wmic")
            .args(&["logicaldisk", "get", "size,freespace,caption"])
            .status();
        
        println!("\nNetwork Adapters:");
        let _ = Command::new("ipconfig").args(&["/all"]).status();
    }

    Ok(())
}
