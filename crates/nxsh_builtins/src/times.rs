//! `times` builtin – display cumulative user/system CPU times for the shell and child processes.
//! Output format similar to Bash:
//!    <user>  <system>
//!    <child_user>  <child_system>
//! Times are printed in seconds with 2 decimal precision.

use anyhow::Result;

#[cfg(unix)]
use libc::{getrusage, rusage, timeval, RUSAGE_CHILDREN, RUSAGE_SELF};

#[cfg(unix)]
pub fn times_cli(_args: &[String]) -> Result<()> {
    unsafe {
        let mut self_usage: rusage = std::mem::zeroed();
        let mut child_usage: rusage = std::mem::zeroed();
        if getrusage(RUSAGE_SELF, &mut self_usage) != 0 {
            return Ok(());
        }
        if getrusage(RUSAGE_CHILDREN, &mut child_usage) != 0 {
            return Ok(());
        }
        let (u, s) = tv_to_secs(self_usage.ru_utime, self_usage.ru_stime);
        let (cu, cs) = tv_to_secs(child_usage.ru_utime, child_usage.ru_stime);
        println!("{:.2}  {:.2}\n{:.2}  {:.2}", u, s, cu, cs);
    }
    Ok(())
}

#[cfg(unix)]
fn tv_to_secs(ut: timeval, st: timeval) -> (f64, f64) {
    let user = ut.tv_sec as f64 + ut.tv_usec as f64 / 1_000_000.0;
    let sys = st.tv_sec as f64 + st.tv_usec as f64 / 1_000_000.0;
    (user, sys)
}

#[cfg(windows)]
pub fn times_cli(_args: &[String]) -> Result<()> {
    println!("times: not supported on Windows yet");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn times_runs() {
        let _ = times_cli(&[]);
    }
} 