//! `id` builtin — print user and group IDs.
//!
//! Output format similar to GNU `id`:
//!   uid=1000(alice) gid=1000(alice) groups=1000(alice),27(sudo)
//! Name lookup via `sysinfo` user list where available; otherwise numeric only.

use anyhow::Result;
use sysinfo::{System, SystemExt, UserExt};
#[cfg(unix)]
use libc::{getgid, getgroups, getuid};

pub fn id_cli(_args: &[String]) -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_users_list();

    #[cfg(unix)]
    {
        unsafe {
            let uid = getuid();
            let gid = getgid();

            let uname = sys
                .get_user_by_id(uid)
                .map(|u| u.name().to_string())
                .unwrap_or_else(|| uid.to_string());
            let gname = sys
                .get_user_by_id(gid)
                .map(|u| u.name().to_string())
                .unwrap_or_else(|| gid.to_string());

            // groups
            let mut groups_buf: [libc::gid_t; 128] = [0; 128];
            let ngroups = getgroups(groups_buf.len() as i32, groups_buf.as_mut_ptr());
            let mut groups_out = Vec::new();
            if ngroups > 0 {
                for i in 0..(ngroups as usize) {
                    let gid_i = groups_buf[i];
                    let name = sys
                        .get_user_by_id(gid_i)
                        .map(|u| u.name().to_string())
                        .unwrap_or_else(|| gid_i.to_string());
                    groups_out.push(format!("{}({})", gid_i, name));
                }
            }

            println!(
                "uid={}({}) gid={}({}) groups={}",
                uid,
                uname,
                gid,
                gname,
                groups_out.join(",")
            );
        }
    }

    #[cfg(windows)]
    {
        // Fallback: delegate to external `whoami /all` if available
        println!("id: not implemented on Windows; use 'whoami' instead");
    }

    Ok(())
} 