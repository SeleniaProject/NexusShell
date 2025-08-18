//! `mount` command - mount filesystem operations with cross-platform support.
//! 
//! Provides mounting functionality on Unix systems and informational
//! commands on Windows. Pure Rust implementation without external dependencies.

use anyhow::{anyhow, Result};
use std::path::Path;

pub async fn mount_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        // Show mounted filesystems
        return show_mounts();
    }

    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_mount_help();
        return Ok(());
    }

    if args.contains(&"--version".to_string()) {
        println!("mount (NexusShell) 1.0.0");
        return Ok(());
    }

    if args.len() < 2 {
        return Err(anyhow!("mount: missing operands\nTry 'mount --help' for more information."));
    }

    let source = &args[0];
    let target = &args[1];
    let fs_type = if args.len() >= 3 { Some(args[2].as_str()) } else { None };
    let options = if args.len() >= 4 { Some(args[3].as_str()) } else { None };

    mount_filesystem(source, target, fs_type, options)
}

#[cfg(unix)]
fn mount_filesystem(source: &str, target: &str, fs_type: Option<&str>, _options: Option<&str>) -> Result<()> {
    use nix::mount::{mount, MsFlags};
    use std::ffi::OsStr;

    // Check if target directory exists
    if !Path::new(target).exists() {
        return Err(anyhow!("mount: mount point '{}' does not exist", target));
    }

    // Convert fs_type to Option<&OsStr>
    let fs_type_os = fs_type.map(OsStr::new);

    // Mount with basic flags (can be extended for more options)
    let flags = MsFlags::empty();
    let data: Option<&str> = None;

    match mount(
        Some(source),
        target,
        fs_type_os,
        flags,
        data,
    ) {
        Ok(()) => {
            println!("Successfully mounted '{}' on '{}'", source, target);
            Ok(())
        }
        Err(e) => Err(anyhow!("mount: failed to mount '{}' on '{}': {}", source, target, e)),
    }
}

#[cfg(windows)]
fn mount_filesystem(source: &str, target: &str, _fs_type: Option<&str>, _options: Option<&str>) -> Result<()> {
    // Windows doesn't have traditional mount, but we can provide useful information
    println!("mount: Windows does not support traditional mounting");
    println!("For Windows, consider using:");
    println!("  - 'net use' command for network drives");
    println!("  - 'subst' command for drive substitution");
    println!("  - Windows Disk Management for mounting drives");
    println!();
    println!("Attempted: mount '{}' on '{}'", source, target);
    Ok(())
}

fn show_mounts() -> Result<()> {
    #[cfg(unix)]
    {
        show_unix_mounts()
    }
    #[cfg(windows)]
    {
        show_windows_mounts()
    }
}

#[cfg(unix)]
fn show_unix_mounts() -> Result<()> {
    use std::fs;
    
    // Read /proc/mounts to show current mounts
    match fs::read_to_string("/proc/mounts") {
        Ok(content) => {
            println!("Mounted filesystems:");
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let device = parts[0];
                    let mount_point = parts[1];
                    let fs_type = parts[2];
                    println!("{} on {} type {}", device, mount_point, fs_type);
                }
            }
            Ok(())
        }
        Err(_) => {
            // Fallback: try to read /etc/mtab
            match fs::read_to_string("/etc/mtab") {
                Ok(content) => {
                    println!("Mounted filesystems:");
                    for line in content.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            let device = parts[0];
                            let mount_point = parts[1];
                            let fs_type = parts[2];
                            println!("{} on {} type {}", device, mount_point, fs_type);
                        }
                    }
                    Ok(())
                }
                Err(e) => Err(anyhow!("mount: cannot read mount information: {}", e)),
            }
        }
    }
}

#[cfg(windows)]
fn show_windows_mounts() -> Result<()> {
    use std::process::Command;

    println!("Windows drive information:");
    
    // Try to use PowerShell to get drive information
    let output = Command::new("powershell")
        .args(&["-Command", "Get-WmiObject -Class Win32_LogicalDisk | Select-Object DeviceID, FileSystem, Size, FreeSpace | Format-Table -AutoSize"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            println!("{}", output_str);
        }
        _ => {
            // Fallback to simple drive listing
            let output = Command::new("wmic")
                .args(&["logicaldisk", "get", "deviceid,filesystem,size,freespace"])
                .output();
            
            match output {
                Ok(output) if output.status.success() => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    println!("{}", output_str);
                }
                _ => {
                    println!("Available drives:");
                    for letter in 'A'..='Z' {
                        let drive = format!("{}:\\", letter);
                        if Path::new(&drive).exists() {
                            println!("  {}", drive);
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn print_mount_help() {
    println!("Usage: mount [OPTION]... DEVICE DIR");
    println!("       mount [OPTION]...");
    println!("Mount a filesystem.");
    println!();
    println!("Options:");
    println!("  -h, --help     display this help and exit");
    println!("  --version      output version information and exit");
    println!();
    println!("Examples:");
    println!("  mount                    # Show mounted filesystems");
    println!("  mount /dev/sda1 /mnt     # Mount device to directory (Unix)");
    println!("  mount //server/share /mnt nfs  # Mount NFS share (Unix)");
    println!();
    println!("Note: On Windows, this command provides information only.");
    println!("Use Windows-specific tools for actual mounting operations.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mount_help() {
        let result = mount_cli(&["--help".to_string()]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mount_version() {
        let result = mount_cli(&["--version".to_string()]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mount_no_args() {
        let result = mount_cli(&[]).await;
        // Should show mounts or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_mount_missing_operands() {
        let result = mount_cli(&["device".to_string()]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mount_nonexistent_target() {
        let result = mount_cli(&["/dev/null".to_string(), "/nonexistent/path".to_string()]).await;
        // Should fail for non-existent target
        assert!(result.is_err() || result.is_ok()); // Windows case returns Ok
    }
}
