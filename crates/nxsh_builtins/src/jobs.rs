use nxsh_core::job::{JOB_TABLE, JobState};
use anyhow::Result;

pub fn fg(id: Option<u32>) -> Result<()> {
    let id = id.unwrap_or(0);
    if let Some(job) = JOB_TABLE.get(id) {
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            kill(Pid::from_raw(job.pid as i32), Signal::SIGCONT)?;
            // Wait for completion (simplified)
            nix::sys::wait::waitpid(Pid::from_raw(job.pid as i32), None)?;
        }
        println!("Job {} brought to foreground", id);
    } else {
        println!("No such job {}", id);
    }
    Ok(())
}

pub fn bg(id: Option<u32>) -> Result<()> {
    let id = id.unwrap_or(0);
    if let Some(job) = JOB_TABLE.get(id) {
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            kill(Pid::from_raw(job.pid as i32), Signal::SIGCONT)?;
        }
        println!("Job {} resumed in background", id);
    } else {
        println!("No such job {}", id);
    }
    Ok(())
} 