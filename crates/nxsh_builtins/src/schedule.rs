use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;
use nxsh_core::advanced_scheduler::{AdvancedJobScheduler, SchedulerConfig};
use tokio::runtime::Runtime;
use once_cell::sync::OnceCell;
use crate::common::i18n::t;

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
        println!("{}", t("schedule-help-title"));
        println!("{}", t("schedule-help-usage"));
        println!("{}", t("schedule-help-options-title"));
        println!("{}", t("schedule-help-option-list"));
        println!("{}", t("schedule-help-option-delete"));
        println!("{}", t("schedule-help-option-help"));
        println!("");
        println!("{}", t("schedule-help-examples-title"));
        println!("{}", t("schedule-help-example-1"));
        println!("{}", t("schedule-help-example-2"));
        println!("{}", t("schedule-help-example-3"));
        return Ok(());
    }

    match args[0].as_str() {
        "-l" | "--list" => {
            let (rt, sched) = ensure_scheduler()?;
            let jobs = rt.block_on(async { sched.list_jobs().await });
            if jobs.is_empty() {
                println!("{}", t("schedule-no-tasks"));
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
                eprintln!("{}", t("schedule-delete-missing-id"));
                std::process::exit(1);
            }
            let job_id = &args[1];
            let (rt, sched) = ensure_scheduler()?;
            let ok = rt.block_on(async { sched.cancel_job(job_id).await })?;
            if ok { println!("{}", t("schedule-deleted")); } else { eprintln!("{}: {job_id}", t("schedule-job-not-found")); std::process::exit(1); }
        }
        "--stats" => {
            let (rt, sched) = ensure_scheduler()?;
            let s = rt.block_on(async { sched.get_statistics().await });
            println!("{} {}", t("schedule-stats-total"), s.total_jobs);
            println!("{} {}", t("schedule-stats-running"), s.running_jobs);
            println!("{} {}", t("schedule-stats-queued"), s.queued_jobs);
            println!("{} {:.1}%", t("schedule-stats-success-rate"), s.success_rate);
            println!("{} {:.1}", t("schedule-stats-avg-exec-ms"), s.avg_execution_time_ms);
        }
        "-h" | "--help" => {
            println!("{}", t("schedule-help-title"));
            println!("{}", t("schedule-help-usage"));
            println!("{}", t("schedule-help-options-title"));
            println!("{}", t("schedule-help-option-list-extended"));
            println!("{}", t("schedule-help-option-delete-extended"));
            println!("{}", t("schedule-help-option-stats"));
            println!("{}", t("schedule-help-option-enable"));
            println!("{}", t("schedule-help-option-disable"));
            println!("{}", t("schedule-help-option-interval"));
            println!("{}", t("schedule-help-option-at"));
            println!("{}", t("schedule-help-option-help"));
        }
        _ => {
            if args.len() < 2 {
                eprintln!("{}", t("schedule-missing-command"));
                eprintln!("{}", t("schedule-usage-time-cmd"));
                std::process::exit(1);
            }
            
            let time_spec = &args[0];
            let command = args[1..].join(" ");
            // Use core scheduler for cron-like expressions; for absolute times fall back to external 'at'
            if time_spec == "--enable" && args.len() >= 2 {
                let job_id = &args[1];
                let (rt, sched) = ensure_scheduler()?;
                let ok = rt.block_on(async { sched.enable_job(job_id).await })?;
                if ok { println!("{}", t("schedule-enabled")); } else { eprintln!("{}: {job_id}", t("schedule-job-not-found")); std::process::exit(1); }
                return Ok(());
            } else if time_spec == "--disable" && args.len() >= 2 {
                let job_id = &args[1];
                let (rt, sched) = ensure_scheduler()?;
                let ok = rt.block_on(async { sched.disable_job(job_id).await })?;
                if ok { println!("{}", t("schedule-disabled")); } else { eprintln!("{}: {job_id}", t("schedule-job-not-found")); std::process::exit(1); }
                return Ok(());
            } else if time_spec == "--interval" && args.len() >= 3 {
                let secs: u64 = args[1].parse().map_err(|_| anyhow!("schedule: invalid seconds"))?; // internal error string OK
                let cmd = args[2..].join(" ");
                let (rt, sched) = ensure_scheduler()?;
                let job_id = rt.block_on(async { sched.schedule_interval(cmd.clone(), secs).await })
                    .map_err(|e| anyhow!("schedule: failed to schedule: {e}"))?;
                println!("{} {job_id}", t("schedule-scheduled-as"));
                return Ok(());
            } else if time_spec == "--at" && args.len() >= 3 {
                let epoch: u64 = args[1].parse().map_err(|_| anyhow!("schedule: invalid epoch secs"))?; // internal error string OK
                let cmd = args[2..].join(" ");
                let (rt, sched) = ensure_scheduler()?;
                let job_id = rt.block_on(async { sched.schedule_once_epoch(cmd.clone(), epoch).await })
                    .map_err(|e| anyhow!("schedule: failed to schedule: {e}"))?;
                println!("{} {job_id}", t("schedule-scheduled-as"));
                return Ok(());
            } else if is_cron_like(time_spec) {
                let (rt, sched) = ensure_scheduler()?;
                let job_id = rt.block_on(async { sched.schedule_cron(command.clone(), time_spec.clone()).await })
                    .map_err(|e| anyhow!("schedule: failed to schedule: {e}"))?;
                println!("{} {job_id}", t("schedule-scheduled-as"));
                return Ok(());
            } else {
                println!("{}", t("schedule-delegating-at"));
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
