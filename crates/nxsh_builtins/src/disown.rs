//! disown builtin: detach jobs from shell job control.
//! Syntax: disown [-a] [JOB_ID...]
use anyhow::{Result, anyhow};
use nxsh_core::job::{with_global_job_manager, JobManager};

pub fn disown_cli(args: &[String]) -> Result<()> {
    let mut all = false;
    let mut targets: Vec<u32> = Vec::new();
    for a in args {
        if a == "-a" { all = true; continue; }
        if let Ok(id) = a.parse::<u32>() { targets.push(id); } else { return Err(anyhow!("disown: invalid job id '{a}'")); }
    }
    with_global_job_manager(|jm: &mut JobManager| {
        if all || targets.is_empty() {
            let ids: Vec<_> = jm.get_all_jobs().into_iter().map(|j| j.id).collect();
            for id in ids { let _ = jm.remove_job(id); }
        } else {
            for id in targets { let _ = jm.remove_job(id); }
        }
    });
    Ok(())
}

