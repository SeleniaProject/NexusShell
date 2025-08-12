//! wait builtin: wait for specified jobs (or all) to finish.
//! Syntax: wait [JOB_ID...]
use anyhow::Result;
use nxsh_core::job::{with_global_job_manager, JobManager};

pub fn wait_cli(args: &[String]) -> Result<()> {
    let job_ids: Vec<u32> = with_global_job_manager(|jm: &mut JobManager| {
        if args.is_empty() { jm.get_all_jobs().into_iter().map(|j| j.id).collect() } else {
            let mut v = Vec::new();
            for a in args { if let Ok(id) = a.parse::<u32>() { v.push(id); } }
            v
        }
    });
    for id in job_ids {
        with_global_job_manager(|jm: &mut JobManager| {
            let _ = jm.wait_for_job(id); // Ignore errors for now (already finished / not found)
        });
    }
    Ok(())
}
