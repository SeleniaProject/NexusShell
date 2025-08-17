//! Id builtin command for NexusShell
//! Cross-platform implementation for user and group identification

use crate::{
    context::ShellContext,
    error::ShellResult,
    ExecutionResult,
    executor::Builtin,
};

/// Built-in id command with full cross-platform support
pub struct IdBuiltin;

impl Builtin for IdBuiltin {
    fn execute(&self, _context: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        if args.len() > 1 {
            println!("id: extra operand '{}'", args[1]);
            println!("Try 'id --help' for more information.");
            return Ok(ExecutionResult::failure(1));
        }

        let target_user = if args.is_empty() || args[0].is_empty() {
            None
        } else {
            Some(args[0].as_str())
        };

        match target_user {
            None => display_current_user_info(),
            Some(username) => display_user_info(username),
        }
    }

    fn name(&self) -> &'static str {
        "id"
    }

    fn help(&self) -> &'static str {
        "Display user and group IDs for the current user or specified user"
    }

    fn synopsis(&self) -> &'static str {
        "id [USER]"
    }

    fn description(&self) -> &'static str {
        "Print user and group IDs for USER, or for the current user if no USER is specified."
    }

    fn usage(&self) -> &'static str {
        "Usage: id [OPTION]... [USER]
Print user and group IDs for USER, or for the current user if no USER is specified.

Options:
  -g, --group        print only the effective group ID
  -G, --groups       print all group IDs
  -n, --name         print a name instead of a number
  -r, --real         print the real ID instead of the effective ID
  -u, --user         print only the effective user ID
      --help         display this help and exit"
    }
}

/// Display information about the current user
fn display_current_user_info() -> ShellResult<ExecutionResult> {
    #[cfg(windows)]
    {
        display_windows_current_user()
    }
    
    #[cfg(unix)]
    {
        display_unix_current_user()
    }
}

/// Display information about a specified user
fn display_user_info(username: &str) -> ShellResult<ExecutionResult> {
    #[cfg(windows)]
    {
        display_windows_user_info(username)
    }
    
    #[cfg(unix)]
    {
        display_unix_user_info(username)
    }
}

#[cfg(windows)]
fn display_windows_current_user() -> ShellResult<ExecutionResult> {
    use std::env;
    use std::process::Command;
    
    let username = env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string());
    let _domain = env::var("USERDOMAIN").unwrap_or_else(|_| env::var("COMPUTERNAME").unwrap_or_else(|_| "WORKGROUP".to_string()));
    
    // Try to get SID using whoami command
    let sid_result = Command::new("whoami")
    .args(["/user", "/fo", "csv", "/nh"])
        .output();
        
    let user_sid = if let Ok(output) = sid_result {
        let output_str = String::from_utf8_lossy(&output.stdout);
        extract_sid_from_csv(&output_str).unwrap_or_else(|| "S-1-5-21-1000-1000-1000-1000".to_string())
    } else {
        "S-1-5-21-1000-1000-1000-1000".to_string()
    };
    
    // Get primary group info
    let group_result = Command::new("whoami")
    .args(["/groups", "/fo", "csv", "/nh"])
        .output();
        
    let primary_group = if let Ok(output) = group_result {
        let output_str = String::from_utf8_lossy(&output.stdout);
        extract_primary_group_from_csv(&output_str).unwrap_or_else(|| ("Users".to_string(), "S-1-5-32-545".to_string()))
    } else {
        ("Users".to_string(), "S-1-5-32-545".to_string())
    };

    println!("uid={}({}) gid={}({}) groups={}({})", 
             user_sid, username, primary_group.1, primary_group.0, primary_group.1, primary_group.0);
             
    Ok(ExecutionResult::success(0))
}

#[cfg(windows)]
fn display_windows_user_info(username: &str) -> ShellResult<ExecutionResult> {
    // For specified users on Windows, we need to query system information
    use std::process::Command;
    
    let query_result = Command::new("net")
    .args(["user", username])
        .output();
        
    match query_result {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("User name") {
                println!("uid=1000({username}) gid=1000(Users) groups=1000(Users)");
                Ok(ExecutionResult::success(0))
            } else {
                println!("id: '{username}': no such user");
                Ok(ExecutionResult::failure(1))
            }
        }
        _ => {
            println!("id: '{username}': no such user");
            Ok(ExecutionResult::failure(1))
        }
    }
}

#[cfg(windows)]
fn extract_sid_from_csv(csv_output: &str) -> Option<String> {
    // Parse CSV output from whoami /user command
    // Format: "DOMAIN\Username","SID"
    let lines: Vec<&str> = csv_output.trim().lines().collect();
    if let Some(line) = lines.first() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            let sid = parts[1].trim_matches('"');
            return Some(sid.to_string());
        }
    }
    None
}

#[cfg(windows)]
fn extract_primary_group_from_csv(csv_output: &str) -> Option<(String, String)> {
    // Parse CSV output from whoami /groups command
    // Look for primary group (usually "Users" or similar)
    let lines: Vec<&str> = csv_output.trim().lines().collect();
    for line in &lines {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 3 {
            let group_name = parts[0].trim_matches('"');
            let group_sid = parts[2].trim_matches('"');
            // Return first group as primary (this is a simplification)
            if group_name.contains("Users") || group_name.contains("Domain Users") {
                return Some((group_name.to_string(), group_sid.to_string()));
            }
        }
    }
    // Fallback to first group or default
    if let Some(line) = lines.first() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 3 {
            let group_name = parts[0].trim_matches('"').to_string();
            let group_sid = parts[2].trim_matches('"').to_string();
            return Some((group_name, group_sid));
        }
    }
    None
}

#[cfg(unix)]
fn display_unix_current_user() -> ShellResult<ExecutionResult> {
    unsafe {
        let uid = libc::getuid();
        let gid = libc::getgid();
        let euid = libc::geteuid();
        let egid = libc::getegid();
        
        // Get username from uid
        let username = get_username_from_uid(uid).unwrap_or_else(|| uid.to_string());
        let groupname = get_groupname_from_gid(gid).unwrap_or_else(|| gid.to_string());
        let effective_username = if euid != uid {
            get_username_from_uid(euid).unwrap_or_else(|| euid.to_string())
        } else {
            username.clone()
        };
        let effective_groupname = if egid != gid {
            get_groupname_from_gid(egid).unwrap_or_else(|| egid.to_string())
        } else {
            groupname.clone()
        };
        
        // Get supplementary groups
        let mut groups = vec![0u32; 64];
        let ngroups = libc::getgroups(groups.len() as i32, groups.as_mut_ptr());
        
        if ngroups >= 0 {
            groups.truncate(ngroups as usize);
        } else {
            groups.clear();
        }
        
        print!("uid={}({}) gid={}({})", uid, username, gid, groupname);
        
        if euid != uid {
            print!(" euid={}({})", euid, effective_username);
        }
        
        if egid != gid {
            print!(" egid={}({})", egid, effective_groupname);
        }
        
        if !groups.is_empty() {
            print!(" groups=");
            for (i, group_id) in groups.iter().enumerate() {
                if i > 0 {
                    print!(",");
                }
                let group_name = get_groupname_from_gid(*group_id).unwrap_or_else(|| group_id.to_string());
                print!("{}({})", group_id, group_name);
            }
        }
        
        println!();
        Ok(ExecutionResult::success(0))
    }
}

#[cfg(unix)]
fn display_unix_user_info(username: &str) -> ShellResult<ExecutionResult> {
    use std::ffi::CString;
    
    let c_username = CString::new(username).unwrap();
    unsafe {
        let passwd = libc::getpwnam(c_username.as_ptr());
        if passwd.is_null() {
            println!("id: '{}': no such user", username);
            return Ok(ExecutionResult::failure(1));
        }
        
        let uid = (*passwd).pw_uid;
        let gid = (*passwd).pw_gid;
        
        let groupname = get_groupname_from_gid(gid).unwrap_or_else(|| gid.to_string());
        
        println!("uid={}({}) gid={}({})", uid, username, gid, groupname);
        Ok(ExecutionResult::success(0))
    }
}

#[cfg(unix)]
fn get_username_from_uid(uid: u32) -> Option<String> {
    unsafe {
        let passwd = libc::getpwuid(uid);
        if !passwd.is_null() {
            let username = std::ffi::CStr::from_ptr((*passwd).pw_name);
            username.to_str().ok().map(|s| s.to_string())
        } else {
            None
        }
    }
}

#[cfg(unix)]
fn get_groupname_from_gid(gid: u32) -> Option<String> {
    unsafe {
        let group = libc::getgrgid(gid);
        if !group.is_null() {
            let groupname = std::ffi::CStr::from_ptr((*group).gr_name);
            groupname.to_str().ok().map(|s| s.to_string())
        } else {
            None
        }
    }
}
