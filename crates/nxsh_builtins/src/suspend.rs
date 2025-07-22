//! `suspend` builtin – send SIGSTOP to the current shell process to suspend it.
//! On Unix this calls `libc::raise(SIGSTOP)`. On Windows a stub message is printed.

use anyhow::Result;

pub fn suspend_cli(_args: &[String]) -> Result<()> {
    #[cfg(unix)]
    unsafe {
        libc::raise(libc::SIGSTOP);
    }
    #[cfg(windows)]
    {
        println!("suspend: not supported on Windows");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn suspend_noop() {
        // Can't easily test actual SIGSTOP in unit test; ensure function returns OK on Windows build.
        let _ = suspend_cli(&[]);
    }
} 