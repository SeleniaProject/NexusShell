//! suspend builtin: suspend the shell (Unix) or report unsupported (Windows).
//! Controlled via NXSH_ENABLE_SUSPEND=1 environment variable to avoid accidental stops.
use anyhow::{Result, anyhow};

pub fn suspend_cli(_args: &[String]) -> Result<()> {
    if std::env::var("NXSH_ENABLE_SUSPEND").ok().as_deref() != Some("1") {
        return Err(anyhow!("suspend disabled (set NXSH_ENABLE_SUSPEND=1 to enable)"));
    }
    #[cfg(unix)]
    unsafe {
        libc::raise(libc::SIGTSTP);
        return Ok(());
    }
    #[cfg(windows)]
    {
    Err(anyhow!("suspend: unsupported on this platform"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn suspend_guard() {
        // Should error without env var
        let res = suspend_cli(&[]);
        assert!(res.is_err());
    }
} 
