use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;
use nxsh_core::advanced_scheduler::{AdvancedJobScheduler, SchedulerConfig};
use tokio::runtime::Runtime;
use once_cell::sync::OnceCell;

/// Entry point for the `schedule` builtin
pub fn schedule_cli(args: &[String]) -> Result<()> {
    // Try external binary first (schedule, sched, or at)
    for binary in &["schedule", "sched", "at"] {
        if let Ok(path) = which(binary) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("schedule: failed to launch backend: {e}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    // Basic internal fallback
    if args.is_empty() {
        println!("schedule: Simple task scheduler");
        println!("Usage: schedule [OPTIONS] TIME COMMAND");
        println!("Options:");
        println!("  -l, --list     List scheduled tasks");
        println!("  -d, --delete   Delete scheduled task");
        println!("  -h, --help     Show this help");
        println!("");
        println!("Examples:");
        println!("  schedule 15:30 'echo Hello'");
        println!("  schedule tomorrow 'backup.sh'");
        println!("  schedule '2024-01-01 09:00' 'echo Happy New Year'");
        return Ok(());
    }

    match args[0].as_str() {
        "-l" | "--list" => {
            let (rt, sched) = ensure_scheduler()?;
            let jobs = rt.block_on(async { sched.list_jobs().await });
            if jobs.is_empty() {
                println!("No scheduled tasks");
            } else {
                for job in jobs {
                    let (when, kind) = match job.schedule {
                        nxsh_core::advanced_scheduler::JobSchedule::Once { run_at } => (run_at, "once"),
                        nxsh_core::advanced_scheduler::JobSchedule::Recurring { next_run, .. } => (next_run, "cron"),
                        nxsh_core::advanced_scheduler::JobSchedule::Interval { next_run, .. } => (next_run, "interval"),
                        nxsh_core::advanced_scheduler::JobSchedule::EventBased { .. } => (std::time::SystemTime::UNIX_EPOCH, "event"),
                    };
                    let ts = match when.duration_since(std::time::SystemTime::UNIX_EPOCH) { Ok(d) => d.as_secs(), Err(_) => 0 };
                    println!("{:<14} {:<6} {:<8} {}", job.id, kind, ts, job.command);
                }
            }
        }
        "-d" | "--delete" => {
            if args.len() < 2 {
                eprintln!("schedule: missing task ID for delete");
                std::process::exit(1);
            }
            let job_id = &args[1];
            let (rt, sched) = ensure_scheduler()?;
            let ok = rt.block_on(async { sched.cancel_job(job_id).await })?;
            if ok { println!("Deleted {job_id}"); } else { eprintln!("schedule: job not found: {job_id}"); std::process::exit(1); }
        }
        "--stats" => {
            let (rt, sched) = ensure_scheduler()?;
            let s = rt.block_on(async { sched.get_statistics().await });
            println!("Total Jobs: {}", s.total_jobs);
            println!("Running: {}", s.running_jobs);
            println!("Queued: {}", s.queued_jobs);
            println!("Success Rate: {:.1}%", s.success_rate);
            println!("Avg Exec Time (ms): {:.1}", s.avg_execution_time_ms);
        }
        "-h" | "--help" => {
            println!("schedule: Simple task scheduler");
            println!("Usage: schedule [OPTIONS] TIME COMMAND");
            println!("Options:");
            println!("  -l, --list       List scheduled tasks");
            println!("  -d, --delete ID  Delete scheduled task");
            println!("      --stats      Show scheduler statistics");
            println!("      --enable ID  Enable a disabled job");
            println!("      --disable ID Disable a job");
            println!("      --interval SECS CMD  Schedule interval job");
            println!("      --at EPOCH_SECS CMD  Schedule one-shot job");
            println!("  -h, --help     Show this help");
        }
        _ => {
            if args.len() < 2 {
                eprintln!("schedule: missing command");
                eprintln!("Usage: schedule TIME COMMAND");
                std::process::exit(1);
            }
            
            let time_spec = &args[0];
            let command = args[1..].join(" ");
            // Use core scheduler for cron-like expressions; for absolute times fall back to external 'at'
            if time_spec == "--enable" && args.len() >= 2 {
                let job_id = &args[1];
                let (rt, sched) = ensure_scheduler()?;
                let ok = rt.block_on(async { sched.enable_job(job_id).await })?;
                if ok { println!("Enabled {job_id}"); } else { eprintln!("schedule: job not found: {job_id}"); std::process::exit(1); }
                return Ok(());
            } else if time_spec == "--disable" && args.len() >= 2 {
                let job_id = &args[1];
                let (rt, sched) = ensure_scheduler()?;
                let ok = rt.block_on(async { sched.disable_job(job_id).await })?;
                if ok { println!("Disabled {job_id}"); } else { eprintln!("schedule: job not found: {job_id}"); std::process::exit(1); }
                return Ok(());
            } else if time_spec == "--interval" && args.len() >= 3 {
                let secs: u64 = args[1].parse().map_err(|_| anyhow!("schedule: invalid seconds"))?;
                let cmd = args[2..].join(" ");
                let (rt, sched) = ensure_scheduler()?;
                let job_id = rt.block_on(async { sched.schedule_interval(cmd.clone(), secs).await })
                    .map_err(|e| anyhow!("schedule: failed to schedule: {e}"))?;
                println!("schedule: scheduled as {job_id}");
                return Ok(());
            } else if time_spec == "--at" && args.len() >= 3 {
                let epoch: u64 = args[1].parse().map_err(|_| anyhow!("schedule: invalid epoch secs"))?;
                let cmd = args[2..].join(" ");
                let (rt, sched) = ensure_scheduler()?;
                let job_id = rt.block_on(async { sched.schedule_once_epoch(cmd.clone(), epoch).await })
                    .map_err(|e| anyhow!("schedule: failed to schedule: {e}"))?;
                println!("schedule: scheduled as {job_id}");
                return Ok(());
            } else if is_cron_like(time_spec) {
                let (rt, sched) = ensure_scheduler()?;
                let job_id = rt.block_on(async { sched.schedule_cron(command.clone(), time_spec.clone()).await })
                    .map_err(|e| anyhow!("schedule: failed to schedule: {e}"))?;
                println!("schedule: scheduled as {job_id}");
                return Ok(());
            } else {
                println!("schedule: delegating absolute time to external 'at' if available");
                if let Ok(path) = which("at") {
                    let status = Command::new(path)
                        .arg(time_spec)
                        .arg(command)
                        .status()
                        .map_err(|e| anyhow!("schedule: failed to launch 'at': {e}"))?;
                    std::process::exit(status.code().unwrap_or(1));
                }
                return Err(anyhow!("schedule: absolute time scheduling requires 'at' command"));
            }
        }
    }

    Ok(())
}

fn is_cron_like(spec: &str) -> bool {
    // Simple heuristic: cron has 5 space-separated fields, allow 6th for seconds in future
    let parts = spec.split_whitespace().count();
    parts >= 5 && parts <= 6
}

// Global scheduler bootstrap (lazily started)
static RUNTIME: OnceCell<Runtime> = OnceCell::new();
static SCHEDULER: OnceCell<AdvancedJobScheduler> = OnceCell::new();

fn ensure_scheduler() -> Result<(&'static Runtime, &'static AdvancedJobScheduler)> {
    let rt = RUNTIME.get_or_try_init(|| Runtime::new().map_err(|e| anyhow!("schedule: runtime init failed: {e}")))?;
    if SCHEDULER.get().is_none() {
        let mut sched = AdvancedJobScheduler::new(SchedulerConfig::default());
        rt.block_on(async { sched.start().await }).map_err(|e| anyhow!("schedule: failed to start scheduler: {e}"))?;
        SCHEDULER.set(sched).map_err(|_| anyhow!("schedule: failed to set scheduler"))?;
    }
    Ok((RUNTIME.get().unwrap(), SCHEDULER.get().unwrap()))
}
