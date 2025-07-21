#[cfg(target_os = "linux")]
use anyhow::Result;
#[cfg(target_os = "linux")]
use seccomp_sys::*;

#[cfg(target_os = "linux")]
/// Apply a conservative seccomp filter allowing only read, write, exit, and fstat.
pub fn apply_seccomp() -> Result<()> {
    unsafe {
        let ctx = seccomp_init(SCMP_ACT_KILL_PROCESS as u32);
        if ctx.is_null() { anyhow::bail!("seccomp_init failed"); }
        let allow = |call| {
            let res = seccomp_rule_add(ctx, SCMP_ACT_ALLOW as u32, call as i32, 0);
            if res != 0 { anyhow::bail!("seccomp_rule_add failed"); }
            Ok(())
        };
        allow(libc::SYS_read)?;
        allow(libc::SYS_write)?;
        allow(libc::SYS_fstat)?;
        allow(libc::SYS_exit)?;
        allow(libc::SYS_exit_group)?;
        if seccomp_load(ctx) != 0 {
            anyhow::bail!("seccomp_load failed");
        }
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn apply_seccomp() -> anyhow::Result<()> { Ok(()) } 