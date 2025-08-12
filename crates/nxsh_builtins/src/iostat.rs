use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

/// Entry point for the `iostat` builtin
pub fn iostat_cli(args: &[String]) -> Result<()> {
    // Try external binary first
    if let Ok(path) = which("iostat") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("iostat: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // Basic internal fallback
    if args.is_empty() || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        println!("iostat: I/O and CPU statistics");
        println!("Usage: iostat [options] [interval] [count]");
        println!("Options:");
        println!("  -c    Show CPU utilization");
        println!("  -d    Show device utilization"); 
        println!("  -x    Extended statistics");
        println!("  -h    Show this help");
        return Ok(());
    }

    println!("iostat: Basic I/O statistics (internal fallback)");
    println!("iostat: Install sysstat package for full functionality");
    
    // Show basic disk usage information
    #[cfg(unix)]
    {
        let _ = Command::new("df")
            .args(&["-h"])
            .status();
    }
    
    #[cfg(windows)]
    {
        println!("Device utilization information:");
        let _ = Command::new("wmic")
            .args(&["logicaldisk", "get", "size,freespace,caption"])
            .status();
    }

    Ok(())
}
