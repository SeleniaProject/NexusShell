//! `id` builtin  Eprint user and group IDs.
//!
//! TEMPORARILY DISABLED: Unix-specific user/group functionality requires platform-specific implementation
//! Windows implementation will use alternative methods

use anyhow::{anyhow, Result};

// Platform-specific imports
#[cfg(unix)]
// Removed uzers dependency - using alternative user management methods
// use uzers::{Users, UsersCache, get_user_by_uid, get_group_by_gid, get_current_uid, get_current_gid, get_effective_uid, get_effective_gid};
#[cfg(unix)]
use nix::libc::{getgid, getgroups, getuid};

#[cfg(windows)]
use whoami;

pub fn id_cli(args: &[String]) -> Result<()> {
    #[cfg(unix)]
    {
        unix_id_impl(args)
    }
    #[cfg(windows)]
    {
        windows_id_impl(args)
    }
}

#[cfg(unix)]
fn unix_id_impl(args: &[String]) -> Result<()> {
    let uid = get_current_uid();
    let gid = get_current_gid();
    let _euid = get_effective_uid();
    let _egid = get_effective_gid();
    
    if args.is_empty() {
        // Default: show uid, gid, and groups
        let user_name = get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().to_string())
            .unwrap_or_else(|| uid.to_string());
            
        let group_name = get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().to_string())
            .unwrap_or_else(|| gid.to_string());
            
        print!("uid={}({}) gid={}({})", uid, user_name, gid, group_name);
        
        // Show supplementary groups
        unsafe {
            let mut groups = vec![0u32; 64];
            let n = getgroups(groups.len() as i32, groups.as_mut_ptr());
            if n > 0 {
                groups.truncate(n as usize);
                print!(" groups=");
                for (i, &gid_i) in groups.iter().enumerate() {
                    if i > 0 { print!(","); }
                    let group_name = get_group_by_gid(gid_i.into())
                        .map(|g| g.name().to_string_lossy().to_string())
                        .unwrap_or_else(|| gid_i.to_string());
                    print!("{}({})", gid_i, group_name);
                }
            }
        }
        println!();
    } else {
        // Handle command line options
        for arg in args {
            match arg.as_str() {
                "-u" | "--user" => {
                    println!("{}", uid);
                }
                "-g" | "--group" => {
                    println!("{}", gid);
                }
                "-un" | "--user" if args.contains(&"--name".to_string()) => {
                    let user_name = get_user_by_uid(uid)
                        .map(|u| u.name().to_string_lossy().to_string())
                        .unwrap_or_else(|| uid.to_string());
                    println!("{}", user_name);
                }
                "-gn" | "--group" if args.contains(&"--name".to_string()) => {
                    let group_name = get_group_by_gid(gid)
                        .map(|g| g.name().to_string_lossy().to_string())
                        .unwrap_or_else(|| gid.to_string());
                    println!("{}", group_name);
                }
                _ => return Err(anyhow!("Unknown option: {}", arg)),
            }
        }
    }
    Ok(())
}

#[cfg(windows)]
fn windows_id_impl(args: &[String]) -> Result<()> {
    let username = whoami::username();
    
    if args.is_empty() {
        // Default: show user info (Windows doesn't have traditional Unix uid/gid)
        println!("user={} domain={}", username, whoami::hostname());
    } else {
        // Handle command line options  
        for arg in args {
            match arg.as_str() {
                "-u" | "--user" => {
                    println!("{}", username);
                }
                "-g" | "--group" => {
                    println!("Users"); // Default group for Windows
                }
                _ => return Err(anyhow!("Unknown option: {}", arg)),
            }
        }
    }
    Ok(())
} 
