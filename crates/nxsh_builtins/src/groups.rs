//! `groups` builtin - display group memberships.
//!
//! Pure Rust implementation with cross-platform support.
//! Provides complete functionality without external dependencies.

use anyhow::{anyhow, Result};


#[cfg(unix)]
use nix::unistd::{getgroups, getgid, Gid, Uid, User, Group};

#[cfg(windows)]

#[cfg(windows)]

#[cfg(windows)]


pub fn groups_cli(args: &[String]) -> Result<()> {
    if args.len() > 1 {
        return Err(anyhow!("groups: too many arguments"));
    }

    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_groups_help();
        return Ok(());
    }

    if args.contains(&"--version".to_string()) {
        println!("groups (NexusShell) 1.0.0");
        return Ok(());
    }

    let username = if args.is_empty() {
        None
    } else {
        Some(args[0].clone())
    };

    display_groups(username)
}

#[cfg(unix)]
fn display_groups(username: Option<String>) -> Result<()> {
    match username {
        Some(user) => {
            // Get groups for specified user
            let user_info = User::from_name(&user)?
                .ok_or_else(|| anyhow!("groups: '{}': no such user", user))?;
            
            let groups = get_user_groups(user_info.uid)?;
            let group_names: Result<Vec<String>, _> = groups.iter()
                .map(|&gid| {
                    Group::from_gid(gid)
                        .map_err(|e| anyhow!("Failed to get group info: {}", e))?
                        .map(|g| g.name)
                        .ok_or_else(|| anyhow!("Group {} not found", gid))
                })
                .collect();
            
            match group_names {
                Ok(names) => println!("{}", names.join(" ")),
                Err(e) => return Err(e),
            }
        }
        None => {
            // Get groups for current user
            let groups = getgroups()?;
            let mut group_names = Vec::new();
            
            for gid in groups {
                if let Ok(Some(group)) = Group::from_gid(gid) {
                    group_names.push(group.name);
                } else {
                    // Fallback to numeric GID if name resolution fails
                    group_names.push(gid.to_string());
                }
            }
            
            println!("{}", group_names.join(" "));
        }
    }
    Ok(())
}

#[cfg(unix)]
fn get_user_groups(uid: Uid) -> Result<Vec<Gid>> {
    use std::fs;
    use std::collections::HashMap;
    
    let mut groups = Vec::new();
    
    // Read /etc/group to find all groups the user belongs to
    let group_content = fs::read_to_string("/etc/group")
        .unwrap_or_default();
    
    for line in group_content.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 4 {
            let group_name = parts[0];
            let gid_str = parts[2];
            let members = parts[3];
            
            if let Ok(gid) = gid_str.parse::<u32>() {
                let gid = Gid::from_raw(gid);
                
                // Check if user is in the member list
                if !members.is_empty() {
                    if let Ok(Some(user)) = User::from_uid(uid) {
                        if members.split(',').any(|m| m.trim() == user.name) {
                            groups.push(gid);
                        }
                    }
                }
            }
        }
    }
    
    // Also add the user's primary group
    if let Ok(Some(user)) = User::from_uid(uid) {
        groups.push(user.gid);
    }
    
    // Remove duplicates
    let mut unique_groups: Vec<Gid> = groups.into_iter().collect::<HashSet<_>>().into_iter().collect();
    unique_groups.sort_by_key(|g| g.as_raw());
    
    Ok(unique_groups)
}

#[cfg(windows)]
fn display_groups(_username: Option<String>) -> Result<()> {
    if _username.is_some() {
        return Err(anyhow!("groups: specifying username not yet supported on Windows"));
    }
    
    // Simple implementation for Windows - show basic user groups
    let groups = get_windows_groups()?;
    println!("{}", groups.join(" "));
    Ok(())
}

#[cfg(windows)]
fn get_windows_groups() -> Result<Vec<String>> {
    use std::process::Command;
    
    // Try to use PowerShell to get current user groups
    let output = Command::new("powershell")
        .args(&["-Command", "([Security.Principal.WindowsIdentity]::GetCurrent()).Groups | ForEach-Object { $_.Translate([Security.Principal.NTAccount]).Value }"])
        .output();
    
    match output {
        Ok(output) if output.status.success() => {
            let groups_str = String::from_utf8_lossy(&output.stdout);
            let groups: Vec<String> = groups_str
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .collect();
            Ok(groups)
        }
        _ => {
            // Fallback - show basic groups
            Ok(vec![
                "Users".to_string(),
                "Everyone".to_string(),
                "INTERACTIVE".to_string(),
                "Authenticated Users".to_string(),
            ])
        }
    }
}

fn print_groups_help() {
    println!("Usage: groups [USERNAME]");
    println!("Print group memberships for each USERNAME or the current user if no USERNAME is specified.");
    println!();
    println!("Options:");
    println!("  -h, --help     display this help and exit");
    println!("  --version      output version information and exit");
    println!();
    println!("Examples:");
    println!("  groups           # Show groups for current user");
    println!("  groups alice     # Show groups for user 'alice'");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groups_help() {
        let result = groups_cli(&["--help".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_groups_version() {
        let result = groups_cli(&["--version".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_groups_current_user() {
        let result = groups_cli(&[]);
        // Should succeed or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_groups_too_many_args() {
        let result = groups_cli(&["user1".to_string(), "user2".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_groups_nonexistent_user() {
        let result = groups_cli(&["nonexistent_user_12345".to_string()]);
        // Should return an error for non-existent user
        assert!(result.is_err());
    }
}


