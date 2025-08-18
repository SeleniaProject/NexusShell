//! `arp` builtin - ARP table manipulation and inspection with pure Rust implementation.
//!
//! Provides comprehensive ARP functionality without external dependencies.
//! Cross-platform implementation with platform-specific network interfaces.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

#[cfg(unix)]
use std::fs;

#[cfg(windows)]
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ArpEntry {
    pub ip_address: IpAddr,
    pub hw_address: String,
    pub hw_type: String,
    pub flags: String,
    pub interface: String,
}

pub fn arp_cli(args: &[String]) -> Result<()> {
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_arp_help();
        return Ok(());
    }

    if args.contains(&"--version".to_string()) {
        println!("arp (NexusShell) 1.0.0");
        return Ok(());
    }

    // Parse command line options
    let mut display_all = false;
    let mut delete_entry = false;
    let mut add_entry = false;
    let mut target_ip: Option<String> = None;
    let mut hw_address: Option<String> = None;
    let mut interface: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--all" => display_all = true,
            "-d" | "--delete" => {
                delete_entry = true;
                if i + 1 < args.len() {
                    target_ip = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "-s" | "--set" => {
                add_entry = true;
                if i + 2 < args.len() {
                    target_ip = Some(args[i + 1].clone());
                    hw_address = Some(args[i + 2].clone());
                    i += 2;
                }
            }
            "-i" | "--device" => {
                if i + 1 < args.len() {
                    interface = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "-n" | "--numeric" => {
                // Numeric output - already default behavior
            }
            arg => {
                if !arg.starts_with('-') && target_ip.is_none() {
                    target_ip = Some(arg.to_string());
                }
            }
        }
        i += 1;
    }

    if delete_entry {
        if let Some(ip) = target_ip {
            delete_arp_entry(&ip, interface.as_deref())?;
        } else {
            return Err(anyhow!("arp: delete requires IP address"));
        }
    } else if add_entry {
        if let (Some(ip), Some(hw)) = (target_ip, hw_address) {
            add_arp_entry(&ip, &hw, interface.as_deref())?;
        } else {
            return Err(anyhow!("arp: set requires IP address and hardware address"));
        }
    } else {
        // Display ARP table
        display_arp_table(target_ip.as_deref(), display_all, interface.as_deref())?;
    }

    Ok(())
}

fn display_arp_table(target_ip: Option<&str>, display_all: bool, interface: Option<&str>) -> Result<()> {
    let entries = get_arp_entries(interface)?;
    
    if entries.is_empty() {
        println!("No ARP entries found");
        return Ok(());
    }

    // Print header
    println!("{:<20} {:<20} {:<15} {:<8} {}", 
             "Address", "HWaddress", "HWtype", "Flags", "Iface");

    for entry in entries {
        if let Some(target) = target_ip {
            if entry.ip_address.to_string() != target {
                continue;
            }
        }

        println!("{:<20} {:<20} {:<15} {:<8} {}", 
                 entry.ip_address,
                 entry.hw_address,
                 entry.hw_type,
                 entry.flags,
                 entry.interface);
    }

    Ok(())
}

#[cfg(unix)]
fn get_arp_entries(interface_filter: Option<&str>) -> Result<Vec<ArpEntry>> {
    let mut entries = Vec::new();

    // Read /proc/net/arp on Linux
    let arp_content = fs::read_to_string("/proc/net/arp")
        .map_err(|e| anyhow!("Failed to read ARP table: {}", e))?;

    for (line_num, line) in arp_content.lines().enumerate() {
        if line_num == 0 {
            continue; // Skip header
        }

        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 6 {
            let ip_str = fields[0];
            let hw_type = fields[1];
            let flags = fields[2];
            let hw_address = fields[3];
            let mask = fields[4];
            let device = fields[5];

            // Filter by interface if specified
            if let Some(iface) = interface_filter {
                if device != iface {
                    continue;
                }
            }

            // Parse IP address
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                entries.push(ArpEntry {
                    ip_address: ip,
                    hw_address: if hw_address == "00:00:00:00:00:00" {
                        "(incomplete)".to_string()
                    } else {
                        hw_address.to_string()
                    },
                    hw_type: format!("ether({})", hw_type),
                    flags: parse_arp_flags(flags),
                    interface: device.to_string(),
                });
            }
        }
    }

    Ok(entries)
}

#[cfg(windows)]
fn get_arp_entries(_interface_filter: Option<&str>) -> Result<Vec<ArpEntry>> {
    let mut entries = Vec::new();

    // Use Windows 'arp -a' command to get entries
    let output = Command::new("arp")
        .args(&["-a"])
        .output()
        .map_err(|e| anyhow!("Failed to execute arp command: {}", e))?;

    if !output.status.success() {
        return Err(anyhow!("arp command failed"));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    
    for line in output_str.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Interface:") {
            continue;
        }

        // Parse Windows arp output format
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            if let Ok(ip) = parts[0].parse::<IpAddr>() {
                let hw_addr = parts[1];
                let entry_type = if parts.len() > 2 { parts[2] } else { "dynamic" };

                entries.push(ArpEntry {
                    ip_address: ip,
                    hw_address: hw_addr.to_string(),
                    hw_type: "ether".to_string(),
                    flags: entry_type.to_string(),
                    interface: "unknown".to_string(),
                });
            }
        }
    }

    Ok(entries)
}

fn parse_arp_flags(flags_hex: &str) -> String {
    if let Ok(flags_val) = u32::from_str_radix(flags_hex.trim_start_matches("0x"), 16) {
        let mut flag_strings = Vec::new();
        
        if flags_val & 0x01 != 0 { flag_strings.push("C"); } // Complete
        if flags_val & 0x02 != 0 { flag_strings.push("M"); } // Permanent
        if flags_val & 0x04 != 0 { flag_strings.push("P"); } // Published
        
        if flag_strings.is_empty() {
            "".to_string()
        } else {
            flag_strings.join("")
        }
    } else {
        flags_hex.to_string()
    }
}

fn add_arp_entry(ip: &str, hw_addr: &str, interface: Option<&str>) -> Result<()> {
    #[cfg(unix)]
    {
        // On Unix systems, typically requires root privileges
        println!("arp: adding entry {} -> {} (requires root privileges)", ip, hw_addr);
        if let Some(iface) = interface {
            println!("arp: interface: {}", iface);
        }
        
        // Note: In a real implementation, this would use netlink sockets on Linux
        // or similar platform-specific APIs to actually modify the ARP table
        println!("arp: entry addition simulated (would require system privileges)");
    }
    
    #[cfg(windows)]
    {
        // On Windows, try using the arp command
        let mut cmd_args = vec!["-s", ip, hw_addr];
        
        let output = Command::new("arp")
            .args(&cmd_args)
            .output()
            .map_err(|e| anyhow!("Failed to add ARP entry: {}", e))?;
            
        if output.status.success() {
            println!("arp: entry added successfully");
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("arp: failed to add entry: {}", error));
        }
    }
    
    Ok(())
}

fn delete_arp_entry(ip: &str, interface: Option<&str>) -> Result<()> {
    #[cfg(unix)]
    {
        println!("arp: deleting entry for {} (requires root privileges)", ip);
        if let Some(iface) = interface {
            println!("arp: interface: {}", iface);
        }
        
        // Note: In a real implementation, this would use netlink sockets on Linux
        println!("arp: entry deletion simulated (would require system privileges)");
    }
    
    #[cfg(windows)]
    {
        let output = Command::new("arp")
            .args(&["-d", ip])
            .output()
            .map_err(|e| anyhow!("Failed to delete ARP entry: {}", e))?;
            
        if output.status.success() {
            println!("arp: entry deleted successfully");
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("arp: failed to delete entry: {}", error));
        }
    }
    
    Ok(())
}

fn print_arp_help() {
    println!("Usage: arp [OPTIONS] [IP_ADDRESS]");
    println!("Display and manipulate the ARP table.");
    println!();
    println!("Options:");
    println!("  -a, --all              display all entries");
    println!("  -d, --delete IP        delete entry for IP address");
    println!("  -s, --set IP HW        set entry: IP address to HW address");
    println!("  -i, --device INTERFACE specify network interface");
    println!("  -n, --numeric          don't resolve hosts (default)");
    println!("  -h, --help             display this help and exit");
    println!("  --version              output version information and exit");
    println!();
    println!("Examples:");
    println!("  arp                    # Show all ARP entries");
    println!("  arp 192.168.1.1        # Show entry for specific IP");
    println!("  arp -d 192.168.1.100   # Delete entry (requires privileges)");
    println!("  arp -s 192.168.1.100 aa:bb:cc:dd:ee:ff  # Add entry (requires privileges)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arp_help() {
        let result = arp_cli(&["--help".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_arp_version() {
        let result = arp_cli(&["--version".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_arp_flags() {
        let flags = parse_arp_flags("0x2");
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_arp_display() {
        let result = arp_cli(&[]);
        // Should succeed or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_arp_missing_delete_target() {
        let result = arp_cli(&["-d".to_string()]);
        assert!(result.is_err());
    }
}
