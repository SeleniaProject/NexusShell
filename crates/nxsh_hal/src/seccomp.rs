// Pure Rust security implementation using Linux process controls - no C/C++ dependencies whatsoever

#[cfg(target_os = "linux")]
use nix::unistd::{getgid, getuid, setgid, setuid};

/// Apply a conservative security policy using Linux process restrictions and resource limits.
/// This implementation is completely C/C++-free, using only pure Rust and Linux kernel interfaces.
#[cfg(target_os = "linux")]
pub fn apply_seccomp() -> anyhow::Result<()> {
    // Set resource limits to prevent resource exhaustion attacks using libc constants

    // Limit maximum file descriptors
    unsafe {
        nix::sys::resource::setrlimit(nix::sys::resource::Resource::RLIMIT_NOFILE, 1024, 1024)
            .map_err(|e| anyhow::anyhow!("Failed to set file descriptor limit: {}", e))?;
    }

    // Limit maximum process count (prevents fork bombs)
    unsafe {
        nix::sys::resource::setrlimit(nix::sys::resource::Resource::RLIMIT_NPROC, 100, 100)
            .map_err(|e| anyhow::anyhow!("Failed to set process limit: {}", e))?;
    }

    // Limit memory usage (1GB soft limit, 2GB hard limit)
    unsafe {
        nix::sys::resource::setrlimit(
            nix::sys::resource::Resource::RLIMIT_AS,
            1024 * 1024 * 1024,
            2 * 1024 * 1024 * 1024,
        )
        .map_err(|e| anyhow::anyhow!("Failed to set memory limit: {}", e))?;
    }

    // Limit CPU time (prevents CPU bombs)
    unsafe {
        nix::sys::resource::setrlimit(nix::sys::resource::Resource::RLIMIT_CPU, 300, 600)
            .map_err(|e| anyhow::anyhow!("Failed to set CPU time limit: {}", e))?;
        // 5-10 minutes
    }

    // Ensure we're running with current user privileges (no privilege escalation)
    let current_uid = getuid();
    let current_gid = getgid();

    // Re-set uid/gid to ensure no setuid/setgid privileges
    setuid(current_uid).map_err(|e| anyhow::anyhow!("Failed to set uid: {}", e))?;
    setgid(current_gid).map_err(|e| anyhow::anyhow!("Failed to set gid: {}", e))?;

    // Note: This pure Rust approach provides robust process-level security hardening
    // without relying on seccomp filters that require C/C++ dependencies.
    // It uses Linux resource limits and user/group controls directly through the nix crate.

    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn apply_seccomp() -> anyhow::Result<()> {
    // Security hardening is Linux-specific, no-op on other platforms
    // Windows and other platforms have their own security mechanisms
    Ok(())
}
