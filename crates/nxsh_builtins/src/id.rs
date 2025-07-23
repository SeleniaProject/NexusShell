//! `id` builtin — print user and group IDs.
//!
//! Output format similar to GNU `id`:
//!   uid=1000(alice) gid=1000(alice) groups=1000(alice),27(sudo)
//! Name lookup via `sysinfo` user list where available; otherwise numeric only.

use anyhow::{anyhow, Result};
use users::{Users, UsersCache, get_user_by_uid, get_group_by_gid, get_current_uid, get_current_gid, get_effective_uid, get_effective_gid};
#[cfg(unix)]
use libc::{getgid, getgroups, getuid};

pub fn id_cli(args: &[String]) -> Result<()> {
    let uid = get_current_uid();
    let gid = get_current_gid();
    let euid = get_effective_uid();
    let egid = get_effective_gid();
    
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
        #[cfg(unix)]
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
                _ => {}
            }
        }
    }
    Ok(())
} 