// Pure Rust seccomp implementation - no C/C++ dependencies

#[cfg(target_os = "linux")]
use seccomp::{Context, Action, ScmpSyscall};

/// Apply a conservative seccomp filter allowing only essential system calls for shell operations.
/// This implementation uses pure Rust seccomp library instead of C bindings.
#[cfg(target_os = "linux")]
pub fn apply_seccomp() -> anyhow::Result<()> {
    // Create seccomp context with default deny action
    let mut ctx = Context::new(Action::KillProcess)?;
    
    // Allow essential system calls for shell operations
    let allowed_syscalls = [
        ScmpSyscall::from_name("read")?,
        ScmpSyscall::from_name("write")?,
        ScmpSyscall::from_name("exit")?,
        ScmpSyscall::from_name("exit_group")?,
        ScmpSyscall::from_name("fstat")?,
        ScmpSyscall::from_name("newfstatat")?,
        ScmpSyscall::from_name("close")?,
        ScmpSyscall::from_name("mmap")?,
        ScmpSyscall::from_name("munmap")?,
        ScmpSyscall::from_name("brk")?,
        ScmpSyscall::from_name("rt_sigaction")?,
        ScmpSyscall::from_name("rt_sigprocmask")?,
        ScmpSyscall::from_name("ioctl")?,
        ScmpSyscall::from_name("poll")?,
        ScmpSyscall::from_name("lseek")?,
        ScmpSyscall::from_name("getcwd")?,
        ScmpSyscall::from_name("chdir")?,
        ScmpSyscall::from_name("openat")?,
        ScmpSyscall::from_name("execve")?,
        ScmpSyscall::from_name("wait4")?,
        ScmpSyscall::from_name("pipe")?,
        ScmpSyscall::from_name("dup2")?,
        ScmpSyscall::from_name("fork")?,
        ScmpSyscall::from_name("clone")?,
    ];
    
    // Add rules to allow these system calls
    for syscall in &allowed_syscalls {
        ctx.add_rule(Action::Allow, *syscall)?;
    }
    
    // Load the seccomp filter
    ctx.load()?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn apply_seccomp() -> anyhow::Result<()> { 
    // Seccomp is Linux-specific, no-op on other platforms
    Ok(()) 
} 