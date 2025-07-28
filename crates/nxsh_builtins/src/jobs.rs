use crate::common::{i18n::*, logging::*};
use std::io::Write;
use std::collections::HashMap;
use nxsh_core::{Context, JobManager, JobStatus as JobState, ShellResult, ExecutionResult};
use anyhow::Result;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct JobRow {
    id: u32,
    pid: u32,
    state: String,
    cmd: String,
}

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

pub fn jobs_cli() {
    let rows: Vec<JobRow> = JOB_TABLE
        .list()
        .into_iter()
        .map(|j| JobRow {
            id: j.id,
            pid: j.pid,
            state: format!("{:?}", j.state),
            cmd: j.cmd,
        })
        .collect();
    println!("{}", Table::new(rows).to_string());
}

pub fn wait_cli(arg: Option<u32>) -> Result<()> {
    let id = arg.unwrap_or(0);
    if let Some(job) = JOB_TABLE.get(id) {
        #[cfg(unix)]
        {
            use nix::unistd::Pid;
            use nix::sys::wait::waitpid;
            waitpid(Pid::from_raw(job.pid as i32), None)?;
        }
        while let Some(j) = JOB_TABLE.get(id) {
            if matches!(j.state, JobState::Completed(_)) { break; }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
    Ok(())
}

pub fn disown_cli(all: bool, id: Option<u32>) {
    if all {
        JOB_TABLE.disown_all();
        println!("All jobs disowned");
    } else if let Some(i) = id {
        JOB_TABLE.disown(i);
        println!("Job {} disowned", i);
    }
} 