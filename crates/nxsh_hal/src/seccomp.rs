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
        
        // Use system call numbers directly instead of libc constants for C/C++ independence
        // These are standard Linux syscall numbers defined in the kernel ABI
        allow(0)?;    // SYS_read
        allow(1)?;    // SYS_write  
        allow(5)?;    // SYS_fstat (x86_64)
        allow(60)?;   // SYS_exit (x86_64)
        allow(231)?;  // SYS_exit_group (x86_64)
        
        if seccomp_load(ctx) != 0 {
            anyhow::bail!("seccomp_load failed");
        }
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn apply_seccomp() -> anyhow::Result<()> { Ok(()) } 