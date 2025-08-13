use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;
use nxsh_core::advanced_scheduler::AdvancedScheduler;
use nxsh_core::compat::Result as CoreResult;
use std::sync::Arc;
use tokio::runtime::Runtime;

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
            println!("schedule: No scheduled tasks found");
        }
        "-d" | "--delete" => {
            if args.len() < 2 {
                eprintln!("schedule: missing task ID for delete");
                std::process::exit(1);
            }
            println!("schedule: Task deletion not implemented internally");
        }
        "-h" | "--help" => {
            println!("schedule: Simple task scheduler");
            println!("Usage: schedule [OPTIONS] TIME COMMAND");
            println!("Options:");
            println!("  -l, --list     List scheduled tasks");
            println!("  -d, --delete   Delete scheduled task");
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
            if is_cron_like(time_spec) {
                let rt = Runtime::new().map_err(|e| anyhow!("schedule: failed to init runtime: {e}"))?;
                let sched = AdvancedScheduler::new();
                let job_id = rt.block_on(async {
                    sched.schedule_cron(command.clone(), time_spec.clone()).await
                }).map_err(|e| anyhow!("schedule: failed to schedule: {e}"))?;
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
