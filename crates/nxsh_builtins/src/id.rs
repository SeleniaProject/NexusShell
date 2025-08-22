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
        let arg = &args[i];
        match arg.as_str() {
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
            arg if arg.starts_with('-') && arg.len() > 1 => {
                // Handle combined flags like -un, -ug, etc.
                let flags = &arg[1..];
                for flag_char in flags.chars() {
                    match flag_char {
                        'u' => user_only = true,
                        'g' => group_only = true,
                        'G' => all_groups = true,
                        'n' => use_name = true,
                        'r' => use_real = true,
                        'z' => zero_delimited = true,
                        _ => {
                            return Err(anyhow!("id: invalid option '-{}'", flag_char));
                        }
                    }
                }
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
    let user_info = get_windows_user_info()?;
    
    if user_only {
        if use_name { 
            print!("{}", user_info.username); 
        } else { 
            print!("{}", user_info.uid); 
        }
        if zero_delimited { print!("\0"); } else { println!(); }
        return Ok(());
    }
    
    if group_only {
        if use_name { 
            print!("{}", user_info.primary_group_name); 
        } else { 
            print!("{}", user_info.primary_gid); 
        }
        if zero_delimited { print!("\0"); } else { println!(); }
        return Ok(());
    }
    
    if all_groups {
        let groups = get_windows_groups()?;
        for (i, group) in groups.iter().enumerate() {
            if i > 0 { 
                if zero_delimited { print!("\0"); } else { print!(" "); } 
            }
            if use_name { 
                print!("{}", group.name); 
            } else { 
                print!("{}", group.id); 
            }
        }
        if zero_delimited { print!("\0"); } else { println!(); }
        return Ok(());
    }
    
    // Full info - format similar to Unix
    let groups = get_windows_groups()?;
    if use_name {
        print!("uid={}({}) gid={}({})", 
               user_info.uid, user_info.username,
               user_info.primary_gid, user_info.primary_group_name);
    } else {
        print!("uid={} gid={}", user_info.uid, user_info.primary_gid);
    }
    
    if !groups.is_empty() {
        print!(" groups=");
        for (i, group) in groups.iter().enumerate() {
            if i > 0 { print!(","); }
            if use_name {
                print!("{}({})", group.id, group.name);
            } else {
                print!("{}", group.id);
            }
        }
    }
    
    if zero_delimited { print!("\0"); } else { println!(); }
    Ok(())
}

#[cfg(windows)]
struct WindowsUserInfo {
    username: String,
    uid: u32,
    primary_group_name: String,
    primary_gid: u32,
}

#[cfg(windows)]
struct WindowsGroupInfo {
    name: String,
    id: u32,
}

#[cfg(windows)]
fn get_windows_user_info() -> Result<WindowsUserInfo> {
    let username = whoami::username();
    
    // Generate a hash-based ID for consistency
    let uid = generate_hash_id(&username);
    
    // Try to get real groups, fall back to defaults
    let _domain = "localhost"; // placeholder, removed deprecated hostname call
    let primary_group_name = if is_admin_user()? {
        "Administrators".to_string()
    } else {
        "Users".to_string()
    };
    let primary_gid = generate_hash_id(&primary_group_name);
    
    Ok(WindowsUserInfo {
        username,
        uid,
        primary_group_name,
        primary_gid,
    })
}

#[cfg(windows)]
fn get_windows_groups() -> Result<Vec<WindowsGroupInfo>> {
    let mut groups = Vec::new();
    
    // Add primary group
    let primary_group = if is_admin_user()? {
        WindowsGroupInfo {
            name: "Administrators".to_string(),
            id: generate_hash_id("Administrators"),
        }
    } else {
        WindowsGroupInfo {
            name: "Users".to_string(),
            id: generate_hash_id("Users"),
        }
    };
    groups.push(primary_group);
    
    // Add some common groups based on user context
    groups.push(WindowsGroupInfo {
        name: "Everyone".to_string(),
        id: generate_hash_id("Everyone"),
    });
    
    if is_admin_user()? {
        groups.push(WindowsGroupInfo {
            name: "Power Users".to_string(),
            id: generate_hash_id("Power Users"),
        });
    }
    
    Ok(groups)
}

#[cfg(windows)]
fn is_admin_user() -> Result<bool> {
    // Use environment variable to detect admin context
    // This is a simple heuristic - in a full implementation you'd use Windows APIs
    let is_elevated = std::env::var("PROCESSOR_ARCHITEW6432").is_ok() ||
                     std::env::var("SESSIONNAME").map(|s| s == "Console").unwrap_or(false);
    
    // Check if running in administrator context by attempting admin operations
    // For now, return based on environment hints
    Ok(is_elevated)
}

#[cfg(windows)]
fn generate_hash_id(name: &str) -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    (hasher.finish() as u32) % 60000 + 1000 // Keep in reasonable range
}

#[cfg(unix)]
fn print_user_info(user: &str, user_only: bool, group_only: bool, all_groups: bool, use_name: bool, _use_real: bool, zero_delimited: bool) -> Result<()> {
    use std::ffi::CString;
    use libc::{getpwnam, getgrgid, gid_t, c_char};

    let cuser = CString::new(user).map_err(|_| anyhow!("id: invalid user name"))?;
    let pwd = unsafe { getpwnam(cuser.as_ptr()) };
    if pwd.is_null() { return Err(anyhow!(format!("id: '{}' not found", user))); }
    unsafe {
        let uid: u32 = (*pwd).pw_uid;
        let gid: u32 = (*pwd).pw_gid;

        if user_only {
            if use_name {
                let name = get_user_name(uid).unwrap_or_else(|| uid.to_string());
                print!("{name}");
            } else {
                print!("{}", uid);
            }
            if zero_delimited { print!("\0"); } else { println!(); }
            return Ok(());
        }

        if group_only {
            if use_name {
                let name = get_group_name(gid).unwrap_or_else(|| gid.to_string());
                print!("{name}");
            } else {
                print!("{}", gid);
            }
            if zero_delimited { print!("\0"); } else { println!(); }
            return Ok(());
        }

        if all_groups {
            let groups = get_groups_for_user(&cuser, gid);
            for (i, g) in groups.iter().enumerate() {
                if i > 0 { if zero_delimited { print!("\0"); } else { print!(" "); } }
                if use_name {
                    let name = get_group_name(*g).unwrap_or_else(|| g.to_string());
                    print!("{name}");
                } else {
                    print!("{}", g);
                }
            }
            if zero_delimited { print!("\0"); } else { println!(); }
            return Ok(());
        }

        let uid_name = get_user_name(uid).unwrap_or_default();
        let gid_name = get_group_name(gid).unwrap_or_default();
        let uid_str = if use_name && !uid_name.is_empty() { uid_name } else { uid.to_string() };
        let gid_str = if use_name && !gid_name.is_empty() { gid_name } else { gid.to_string() };

        print!("uid={} gid={}", uid_str, gid_str);

        let groups = get_groups_for_user(&cuser, gid);
        if !groups.is_empty() {
            print!(" groups=");
            for (i, g) in groups.iter().enumerate() {
                if i > 0 { print!(","); }
                if use_name {
                    let name = get_group_name(*g).unwrap_or_else(|| g.to_string());
                    print!("{name}");
                } else {
                    print!("{}", g);
                }
            }
        }
        if zero_delimited { print!("\0"); } else { println!(); }
        Ok(())
    }
}

#[cfg(windows)]
fn print_user_info(user: &str, user_only: bool, group_only: bool, all_groups: bool, use_name: bool, _use_real: bool, zero_delimited: bool) -> Result<()> {
    // Compare to current user
    let current = whoami::username();
    if user.to_lowercase() != current.to_lowercase() {
        // Try to handle some common system users
        match user.to_lowercase().as_str() {
            "system" => {
                let uid = generate_hash_id("SYSTEM");
                let gid = generate_hash_id("SYSTEM");
                
                if user_only {
                    if use_name { print!("SYSTEM"); } else { print!("{}", uid); }
                } else if group_only {
                    if use_name { print!("SYSTEM"); } else { print!("{}", gid); }
                } else if all_groups {
                    if use_name { print!("SYSTEM"); } else { print!("{}", gid); }
                } else {
                    print!("uid={}(SYSTEM) gid={}(SYSTEM) groups={}(SYSTEM)", uid, gid, gid);
                }
                
                if zero_delimited { print!("\0"); } else { println!(); }
                return Ok(());
            },
            "administrator" => {
                let uid = generate_hash_id("Administrator");
                let gid = generate_hash_id("Administrators");
                
                if user_only {
                    if use_name { print!("Administrator"); } else { print!("{}", uid); }
                } else if group_only {
                    if use_name { print!("Administrators"); } else { print!("{}", gid); }
                } else if all_groups {
                    if use_name { print!("Administrators"); } else { print!("{}", gid); }
                } else {
                    print!("uid={}(Administrator) gid={}(Administrators) groups={}(Administrators)", uid, gid, gid);
                }
                
                if zero_delimited { print!("\0"); } else { println!(); }
                return Ok(());
            },
            _ => {
                return Err(anyhow!(format!("id: '{}': no such user", user)));
            }
        }
    }
    
    // For current user, use existing implementation
    print_current_user_info(user_only, group_only, all_groups, use_name, false, zero_delimited)
}

#[cfg(unix)]
fn get_groups_for_user(user: &std::ffi::CString, primary_gid: u32) -> Vec<u32> {
    use libc::{getgrouplist, gid_t};
    let mut ngroups: i32 = 0;
    unsafe {
        // First call to get required size
        let mut dummy: gid_t = 0;
        let mut size = 0;
        getgrouplist(user.as_ptr(), primary_gid as gid_t, &mut dummy as *mut gid_t, &mut size as *mut i32);
        if size <= 0 { return vec![primary_gid]; }
        let mut buf: Vec<gid_t> = vec![0; size as usize];
        let mut n = size;
        if getgrouplist(user.as_ptr(), primary_gid as gid_t, buf.as_mut_ptr(), &mut n as *mut i32) < 0 {
            return vec![primary_gid];
        }
        buf.truncate(n as usize);
        buf.into_iter().map(|g| g as u32).collect()
    }
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
    let groups = get_windows_groups()?;
    
    for (i, group) in groups.iter().enumerate() {
        if i > 0 { 
            if zero_delimited { print!("\0"); } else { print!(" "); } 
        }
        
        if use_name {
            print!("{}", group.name);
        } else {
            print!("{}", group.id);
        }
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
fn get_user_name(uid: u32) -> Option<String> {
    // Try to reverse-lookup from known UIDs, fallback to current user
    let current_user = whoami::username();
    let current_uid = generate_hash_id(&current_user);
    
    if uid == current_uid {
        Some(current_user)
    } else {
        // Check some common system UIDs
        let known_users = [
            ("SYSTEM", generate_hash_id("SYSTEM")),
            ("Administrator", generate_hash_id("Administrator")),
            ("Guest", generate_hash_id("Guest")),
        ];
        
        for (name, id) in &known_users {
            if *id == uid {
                return Some(name.to_string());
            }
        }
        
        None
    }
}

#[cfg(windows)]
fn get_group_name(gid: u32) -> Option<String> {
    // Try to reverse-lookup from known GIDs
    let known_groups = [
        ("Administrators", generate_hash_id("Administrators")),
        ("Users", generate_hash_id("Users")),
        ("Everyone", generate_hash_id("Everyone")),
        ("Power Users", generate_hash_id("Power Users")),
        ("SYSTEM", generate_hash_id("SYSTEM")),
    ];
    
    for (name, id) in &known_groups {
        if *id == gid {
            return Some(name.to_string());
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_help() {
        let result = id_cli(&["--help".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_id_current_user() {
        let result = id_cli(&[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_id_user_only() {
        let result = id_cli(&["-u".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_id_group_only() {
        let result = id_cli(&["-g".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_id_all_groups() {
        let result = id_cli(&["-G".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_id_with_names() {
        let result = id_cli(&["-n".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_id_real_ids() {
        let result = id_cli(&["-r".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_id_zero_delimited() {
        let result = id_cli(&["-z".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_id_combined_flags() {
        let result = id_cli(&["-un".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_id_invalid_option() {
        let result = id_cli(&["-x".to_string()]);
        assert!(result.is_err());
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_user_info() {
        let user_info = get_windows_user_info();
        assert!(user_info.is_ok());
        let info = user_info.unwrap();
        assert!(!info.username.is_empty());
        assert!(info.uid > 0);
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_groups() {
        let groups = get_windows_groups();
        assert!(groups.is_ok());
        let group_list = groups.unwrap();
        assert!(!group_list.is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn test_generate_hash_id() {
        let id1 = generate_hash_id("test");
        let id2 = generate_hash_id("test");
        let id3 = generate_hash_id("different");
        
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert!(id1 >= 1000);
        assert!(id1 < 61000);
    }

    #[cfg(windows)]
    #[test]
    fn test_system_users() {
        let result = id_cli(&["system".to_string()]);
        assert!(result.is_ok());
        
        let result = id_cli(&["administrator".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nonexistent_user() {
        let result = id_cli(&["nonexistentuser123".to_string()]);
        assert!(result.is_err());
    }
}


/// Execute function stub
pub fn execute(_args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}
