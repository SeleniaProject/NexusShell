use std::collections::HashMap;
use nxsh_core::Job;
use anyhow::Result;
use tabled::{Table, Tabled};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use super::ui_design::{Colorize, TableFormatter, ColorPalette, Icons};

static JOB_TABLE: Lazy<Arc<Mutex<HashMap<u32, Job>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

#[derive(Tabled)]
struct JobRow {
    id: u32,
    pid: u32,
    state: String,
    cmd: String,
}

pub fn fg(id: Option<u32>) -> Result<()> {
    let id = id.unwrap_or(0);
    if let Ok(jobs) = JOB_TABLE.lock() {
    if let Some(_job) = jobs.get(&id) {
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            kill(Pid::from_raw(job.pid as i32), Signal::SIGCONT)?;
            // Wait for completion (simplified)
            nix::sys::wait::waitpid(Pid::from_raw(job.pid as i32), None)?;
        }
        println!("Job {id} brought to foreground");
        } else {
            println!("No such job {id}");
        }
    } else {
        println!("Failed to access job table");
    }
    Ok(())
}

pub fn bg(id: Option<u32>) -> Result<()> {
    let id = id.unwrap_or(0);
    if let Ok(jobs) = JOB_TABLE.lock() {
    if let Some(_job) = jobs.get(&id) {
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                kill(Pid::from_raw(job.pid as i32), Signal::SIGCONT)?;
            }
            println!("Job {id} resumed in background");
        } else {
            println!("No such job {id}");
        }
    } else {
        println!("Failed to access job table");
    }
    Ok(())
}

pub fn jobs_cli() {
    if let Ok(jobs) = JOB_TABLE.lock() {
        let rows: Vec<JobRow> = jobs
            .values()
            .map(|j| JobRow {
                id: j.id,
                pid: j.pgid,
                state: format!("{:?}", j.status),
                cmd: j.description.clone(),
            })
            .collect();
        println!("{}", Table::new(rows));
    } else {
        println!("Failed to access job table");
    }
}

pub fn wait_cli(arg: Option<u32>) -> Result<()> {
    let id = arg.unwrap_or(0);
    if let Ok(jobs) = JOB_TABLE.lock() {
    if let Some(_job) = jobs.get(&id) {
            #[cfg(unix)]
            {
                use nix::unistd::Pid;
                use nix::sys::wait::waitpid;
                if let Some(pid) = job.pid {
                    waitpid(Pid::from_raw(pid as i32), None)?;
                }
            }
            // Simplified wait logic
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    Ok(())
}

pub fn disown_cli(all: bool, id: Option<u32>) {
    if all {
        if let Ok(mut jobs) = JOB_TABLE.lock() {
            jobs.clear();
            println!("All jobs disowned");
        }
    } else if let Some(i) = id {
        if let Ok(mut jobs) = JOB_TABLE.lock() {
            jobs.remove(&i);
            println!("Job {i} disowned");
        }
    }
} 
