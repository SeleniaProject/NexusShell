//! `id` builtin - Display user and group IDs
//! Cross-platform user identification utility
//!
//! Usage:
//! id [OPTION]... [USERNAME]
//!
//! Options:
//! -u, --user      print only the effective user ID
//! -g, --group     print only the effective group ID  
//! -G, --groups    print all group IDs
//! -n, --name      print a name instead of a number, for -ugG
//! -r, --real      print the real ID instead of the effective ID, with -ugG
//! -z, --zero      delimit entries with NUL characters, not whitespace
//!     --help      display this help and exit

use anyhow::{Result, anyhow};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[cfg(windows)]
use whoami;

pub fn id_cli(args: &[String]) -> Result<()> {
    let mut user_only = false;
    let mut group_only = false;
    let mut all_groups = false;
    let mut use_name = false;
    let mut use_real = false;
    let mut zero_delimited = false;
    let mut target_user = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-u" | "--user" => user_only = true,
            "-g" | "--group" => group_only = true,
            "-G" | "--groups" => all_groups = true,
            "-n" | "--name" => use_name = true,
            "-r" | "--real" => use_real = true,
            "-z" | "--zero" => zero_delimited = true,
            "--help" => {
                print_help();
                return Ok(());
            }
            arg if !arg.starts_with('-') => {
                target_user = Some(arg.to_string());
            }
            _ => {
                return Err(anyhow!("id: invalid option '{}'", args[i]));
            }
        }
        i += 1;
    }

    if let Some(user) = target_user {
        print_user_info(&user, user_only, group_only, all_groups, use_name, use_real, zero_delimited)?;
    } else {
        print_current_user_info(user_only, group_only, all_groups, use_name, use_real, zero_delimited)?;
    }

    Ok(())
}

fn print_help() {
    println!("Usage: id [OPTION]... [USERNAME]");
    println!("Print user and group IDs for USERNAME or the current user.");
    println!();
    println!("Options:");
    println!("  -g, --group     print only the effective group ID");
    println!("  -G, --groups    print all group IDs");
    println!("  -n, --name      print a name instead of a number");
    println!("  -r, --real      print the real ID instead of effective ID");
    println!("  -u, --user      print only the effective user ID");
    println!("  -z, --zero      delimit entries with NUL characters");
    println!("      --help      display this help and exit");
}

#[cfg(unix)]
fn print_current_user_info(user_only: bool, group_only: bool, all_groups: bool, use_name: bool, use_real: bool, zero_delimited: bool) -> Result<()> {
    use std::ffi::CStr;
    
    let uid = if use_real { unsafe { libc::getuid() } } else { unsafe { libc::geteuid() } };
    let gid = if use_real { unsafe { libc::getgid() } } else { unsafe { libc::getegid() } };

    if user_only {
        if use_name {
            let name = get_user_name(uid).unwrap_or_else(|| uid.to_string());
            print!("{}", name);
        } else {
            print!("{}", uid);
        }
        if zero_delimited { print!("\0"); } else { println!(); }
    } else if group_only {
        if use_name {
            let name = get_group_name(gid).unwrap_or_else(|| gid.to_string());
            print!("{}", name);
        } else {
            print!("{}", gid);
        }
        if zero_delimited { print!("\0"); } else { println!(); }
    } else if all_groups {
        print_all_groups(use_name, zero_delimited)?;
    } else {
        // Print full info
        let uid_name = get_user_name(uid).unwrap_or_else(|| "".to_string());
        let gid_name = get_group_name(gid).unwrap_or_else(|| "".to_string());
        
        let uid_str = if uid_name.is_empty() { uid.to_string() } else { format!("{}({})", uid, uid_name) };
        let gid_str = if gid_name.is_empty() { gid.to_string() } else { format!("{}({})", gid, gid_name) };
        
        print!("uid={} gid={}", uid_str, gid_str);
        
        // Add supplementary groups
        let mut groups = vec![0u32; 64];
        let mut ngroups = groups.len() as i32;
        
        let result = unsafe {
            libc::getgroups(ngroups, groups.as_mut_ptr())
        };
        
        if result >= 0 {
            groups.truncate(result as usize);
            if !groups.is_empty() {
                print!(" groups=");
                for (i, &group) in groups.iter().enumerate() {
                    if i > 0 { print!(","); }
                    let group_name = get_group_name(group).unwrap_or_else(|| "".to_string());
                    if group_name.is_empty() {
                        print!("{}", group);
                    } else {
                        print!("{}({})", group, group_name);
                    }
                }
            }
        }
        
        if zero_delimited { print!("\0"); } else { println!(); }
    }
    
    Ok(())
}

#[cfg(windows)]
fn print_current_user_info(user_only: bool, group_only: bool, all_groups: bool, use_name: bool, _use_real: bool, zero_delimited: bool) -> Result<()> {
    let username = whoami::username();
    let _domain = std::env::var("USERDOMAIN").unwrap_or_else(|_| "WORKGROUP".to_string());
    
    if user_only {
        if use_name {
            print!("{username}");
        } else {
            print!("1000"); // Dummy UID for Windows
        }
        if zero_delimited { print!("\0"); } else { println!(); }
    } else if group_only {
        if use_name {
            print!("None"); // Windows doesn't have primary groups like Unix
        } else {
            print!("1000"); // Dummy GID for Windows
        }
        if zero_delimited { print!("\0"); } else { println!(); }
    } else if all_groups {
        print!("1000"); // Simplified for Windows
        if zero_delimited { print!("\0"); } else { println!(); }
    } else {
        print!("uid=1000({username}) gid=1000(None) groups=1000(None)");
        if zero_delimited { print!("\0"); } else { println!(); }
    }
    
    Ok(())
}

fn print_user_info(_user: &str, _user_only: bool, _group_only: bool, _all_groups: bool, _use_name: bool, _use_real: bool, _zero_delimited: bool) -> Result<()> {
    // For specified user, we'll need to look up user database
    // This is a simplified implementation
    Err(anyhow!("id: looking up user information not implemented yet"))
}

#[cfg(unix)]
fn print_all_groups(use_name: bool, zero_delimited: bool) -> Result<()> {
    let mut groups = vec![0u32; 64];
    let mut ngroups = groups.len() as i32;
    
    let result = unsafe {
        libc::getgroups(ngroups, groups.as_mut_ptr())
    };
    
    if result < 0 {
        return Err(anyhow!("Failed to get group list"));
    }
    
    groups.truncate(result as usize);
    
    for (i, &group) in groups.iter().enumerate() {
        if i > 0 { 
            if zero_delimited { 
                print!("\0"); 
            } else { 
                print!(" "); 
            } 
        }
        
        if use_name {
            let name = get_group_name(group).unwrap_or_else(|| group.to_string());
            print!("{}", name);
        } else {
            print!("{}", group);
        }
    }
    
    if zero_delimited { print!("\0"); } else { println!(); }
    Ok(())
}

#[cfg(windows)]
fn print_all_groups(use_name: bool, zero_delimited: bool) -> Result<()> {
    if use_name {
        print!("None");
    } else {
        print!("1000");
    }
    if zero_delimited { print!("\0"); } else { println!(); }
    Ok(())
}

#[cfg(unix)]
fn get_user_name(uid: u32) -> Option<String> {
    use std::ffi::CStr;
    use std::ptr;
    
    let pwd = unsafe { libc::getpwuid(uid) };
    if pwd.is_null() {
        return None;
    }
    
    unsafe {
        let name_ptr = (*pwd).pw_name;
        if name_ptr.is_null() {
            return None;
        }
        CStr::from_ptr(name_ptr).to_str().ok().map(|s| s.to_string())
    }
}

#[cfg(unix)]
fn get_group_name(gid: u32) -> Option<String> {
    use std::ffi::CStr;
    
    let grp = unsafe { libc::getgrgid(gid) };
    if grp.is_null() {
        return None;
    }
    
    unsafe {
        let name_ptr = (*grp).gr_name;
        if name_ptr.is_null() {
            return None;
        }
        CStr::from_ptr(name_ptr).to_str().ok().map(|s| s.to_string())
    }
}

#[cfg(windows)]
fn get_user_name(_uid: u32) -> Option<String> {
    Some(whoami::username())
}

#[cfg(windows)]
fn get_group_name(_gid: u32) -> Option<String> {
    Some("None".to_string())
}
