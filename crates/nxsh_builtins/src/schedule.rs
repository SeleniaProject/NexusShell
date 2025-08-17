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

    // Enhanced interactive fallback when no args provided
    if args.is_empty() {
        return interactive_schedule_guide();
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

/// Interactive scheduling guide when no arguments are provided
fn interactive_schedule_guide() -> Result<()> {
    use std::io::{self, Write};

    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│                    📅 Task Scheduler Guide                   │");
    println!("╰─────────────────────────────────────────────────────────────╯");
    println!();
    
    // Show current status
    if let Ok((rt, sched)) = ensure_scheduler() {
        let jobs = rt.block_on(async { sched.list_jobs().await });
        let stats = rt.block_on(async { sched.get_statistics().await });
        
        println!("📊 Current Status:");
        println!("   • Total jobs: {}", stats.total_jobs);
        println!("   • Running: {}", stats.running_jobs);
        println!("   • Queued: {}", stats.queued_jobs);
        println!("   • Success rate: {:.1}%", stats.success_rate);
        println!();
        
        if !jobs.is_empty() {
            println!("🕒 Recent Jobs:");
            for (i, job) in jobs.iter().take(5).enumerate() {
                let status = if matches!(job.schedule, nxsh_core::advanced_scheduler::JobSchedule::EventBased { .. }) {
                    "Event-based"
                } else {
                    "Scheduled"
                };
                println!("   {}. [{}] {} - {}", i + 1, job.id, status, job.command);
            }
            println!();
        }
    }
    
    println!("🎯 What would you like to do?");
    println!();
    println!("   1️⃣  Schedule a one-time task");
    println!("   2️⃣  Schedule a recurring task (cron-style)");
    println!("   3️⃣  Schedule an interval-based task");
    println!("   4️⃣  List all scheduled tasks");
    println!("   5️⃣  View task statistics");
    println!("   6️⃣  Delete a task");
    println!("   7️⃣  Enable/disable a task");
    println!("   8️⃣  Show help and examples");
    println!("   0️⃣  Exit");
    println!();
    
    loop {
        print!("👉 Enter your choice (1-8, 0 to exit): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();
        
        match choice {
            "1" => {
                println!();
                println!("⏰ Schedule One-Time Task");
                println!("─────────────────────────");
                
                print!("📅 When to run (format: 'YYYY-MM-DD HH:MM' or epoch seconds): ");
                io::stdout().flush()?;
                let mut time_input = String::new();
                io::stdin().read_line(&mut time_input)?;
                let time_spec = time_input.trim();
                
                print!("💻 Command to run: ");
                io::stdout().flush()?;
                let mut cmd_input = String::new();
                io::stdin().read_line(&mut cmd_input)?;
                let command = cmd_input.trim();
                
                if !command.is_empty() {
                    if let Ok(epoch) = time_spec.parse::<u64>() {
                        schedule_task_once_epoch(command, epoch)?;
                    } else {
                        println!("📝 For absolute time parsing, using external 'at' command...");
                        schedule_task_external(time_spec, command)?;
                    }
                } else {
                    println!("❌ Command cannot be empty!");
                }
            },
            "2" => {
                println!();
                println!("🔄 Schedule Recurring Task (Cron-style)");
                println!("───────────────────────────────────────");
                
                println!("📋 Cron format: minute hour day month day-of-week");
                println!("   Examples:");
                println!("   • '0 9 * * *'     - Every day at 9 AM");
                println!("   • '*/15 * * * *'  - Every 15 minutes");
                println!("   • '0 0 1 * *'     - First day of each month");
                println!("   • '0 18 * * 1-5'  - Weekdays at 6 PM");
                println!();
                
                print!("⏰ Cron schedule: ");
                io::stdout().flush()?;
                let mut cron_input = String::new();
                io::stdin().read_line(&mut cron_input)?;
                let cron_spec = cron_input.trim();
                
                print!("💻 Command to run: ");
                io::stdout().flush()?;
                let mut cmd_input = String::new();
                io::stdin().read_line(&mut cmd_input)?;
                let command = cmd_input.trim();
                
                if !command.is_empty() && !cron_spec.is_empty() {
                    schedule_task_cron(command, cron_spec)?;
                } else {
                    println!("❌ Both schedule and command are required!");
                }
            },
            "3" => {
                println!();
                println!("⚡ Schedule Interval-Based Task");
                println!("──────────────────────────────");
                
                print!("⏱️  Interval in seconds: ");
                io::stdout().flush()?;
                let mut interval_input = String::new();
                io::stdin().read_line(&mut interval_input)?;
                
                if let Ok(seconds) = interval_input.trim().parse::<u64>() {
                    print!("💻 Command to run: ");
                    io::stdout().flush()?;
                    let mut cmd_input = String::new();
                    io::stdin().read_line(&mut cmd_input)?;
                    let command = cmd_input.trim();
                    
                    if !command.is_empty() {
                        schedule_task_interval(command, seconds)?;
                    } else {
                        println!("❌ Command cannot be empty!");
                    }
                } else {
                    println!("❌ Invalid interval! Please enter a number of seconds.");
                }
            },
            "4" => {
                println!();
                println!("📋 Scheduled Tasks");
                println!("─────────────────");
                list_scheduled_tasks()?;
            },
            "5" => {
                println!();
                println!("📊 Task Statistics");
                println!("─────────────────");
                show_task_statistics()?;
            },
            "6" => {
                println!();
                println!("🗑️  Delete Task");
                println!("──────────────");
                
                print!("🆔 Task ID to delete: ");
                io::stdout().flush()?;
                let mut id_input = String::new();
                io::stdin().read_line(&mut id_input)?;
                let task_id = id_input.trim();
                
                if !task_id.is_empty() {
                    delete_task(task_id)?;
                } else {
                    println!("❌ Task ID cannot be empty!");
                }
            },
            "7" => {
                println!();
                println!("⚙️  Enable/Disable Task");
                println!("─────────────────────");
                
                print!("🆔 Task ID: ");
                io::stdout().flush()?;
                let mut id_input = String::new();
                io::stdin().read_line(&mut id_input)?;
                let task_id = id_input.trim();
                
                if !task_id.is_empty() {
                    print!("🔘 Action (enable/disable): ");
                    io::stdout().flush()?;
                    let mut action_input = String::new();
                    io::stdin().read_line(&mut action_input)?;
                    let action = action_input.trim().to_lowercase();
                    
                    match action.as_str() {
                        "enable" => enable_task(task_id)?,
                        "disable" => disable_task(task_id)?,
                        _ => println!("❌ Invalid action! Use 'enable' or 'disable'."),
                    }
                } else {
                    println!("❌ Task ID cannot be empty!");
                }
            },
            "8" => {
                println!();
                show_help_and_examples();
            },
            "0" => {
                println!("👋 Goodbye!");
                break;
            },
            _ => {
                println!("❌ Invalid choice! Please enter a number from 0-8.");
            },
        }
        
        println!();
        println!("Press Enter to continue...");
        let mut pause = String::new();
        io::stdin().read_line(&mut pause)?;
        println!();
    }
    
    Ok(())
}

fn schedule_task_once_epoch(command: &str, epoch: u64) -> Result<()> {
    let (rt, sched) = ensure_scheduler()?;
    let job_id = rt.block_on(async { sched.schedule_once_epoch(command.to_string(), epoch).await })
        .map_err(|e| anyhow!("Failed to schedule task: {e}"))?;
    println!("✅ Task scheduled successfully with ID: {}", job_id);
    Ok(())
}

fn schedule_task_external(time_spec: &str, command: &str) -> Result<()> {
    if let Ok(path) = which("at") {
        let status = Command::new(path)
            .arg(time_spec)
            .arg("-c")
            .arg(command)
            .status()
            .map_err(|e| anyhow!("Failed to launch 'at': {e}"))?;
        
        if status.success() {
            println!("✅ Task scheduled successfully using 'at' command");
        } else {
            return Err(anyhow!("'at' command failed"));
        }
    } else {
        return Err(anyhow!("External 'at' command not found. Use epoch seconds instead."));
    }
    Ok(())
}

fn schedule_task_cron(command: &str, cron_spec: &str) -> Result<()> {
    let (rt, sched) = ensure_scheduler()?;
    let job_id = rt.block_on(async { sched.schedule_cron(command.to_string(), cron_spec.to_string()).await })
        .map_err(|e| anyhow!("Failed to schedule cron task: {e}"))?;
    println!("✅ Cron task scheduled successfully with ID: {}", job_id);
    Ok(())
}

fn schedule_task_interval(command: &str, seconds: u64) -> Result<()> {
    let (rt, sched) = ensure_scheduler()?;
    let job_id = rt.block_on(async { sched.schedule_interval(command.to_string(), seconds).await })
        .map_err(|e| anyhow!("Failed to schedule interval task: {e}"))?;
    println!("✅ Interval task scheduled successfully with ID: {}", job_id);
    Ok(())
}

fn list_scheduled_tasks() -> Result<()> {
    let (rt, sched) = ensure_scheduler()?;
    let jobs = rt.block_on(async { sched.list_jobs().await });
    
    if jobs.is_empty() {
        println!("📭 No scheduled tasks found.");
        return Ok(());
    }
    
    println!("┌──────────────┬────────────┬─────────────────────┬─────────────────────────────────────────┐");
    println!("│ Task ID      │ Type       │ Next Run            │ Command                                 │");
    println!("├──────────────┼────────────┼─────────────────────┼─────────────────────────────────────────┤");
    
    for job in jobs {
        let (when, kind) = match job.schedule {
            nxsh_core::advanced_scheduler::JobSchedule::Once { run_at } => (run_at, "Once"),
            nxsh_core::advanced_scheduler::JobSchedule::Recurring { next_run, .. } => (next_run, "Cron"),
            nxsh_core::advanced_scheduler::JobSchedule::Interval { next_run, .. } => (next_run, "Interval"),
            nxsh_core::advanced_scheduler::JobSchedule::EventBased { .. } => (std::time::SystemTime::UNIX_EPOCH, "Event"),
        };
        
        let time_str = if when == std::time::SystemTime::UNIX_EPOCH {
            "Event-based".to_string()
        } else {
            match when.duration_since(std::time::SystemTime::UNIX_EPOCH) {
                Ok(d) => {
                    let timestamp = d.as_secs();
                    // Simple timestamp to readable format
                    format!("{}", timestamp)
                },
                Err(_) => "Unknown".to_string(),
            }
        };
        
        let cmd_truncated = if job.command.len() > 39 {
            format!("{}...", &job.command[..36])
        } else {
            job.command.clone()
        };
        
        println!("│ {:<12} │ {:<10} │ {:<19} │ {:<39} │", 
                 job.id, kind, time_str, cmd_truncated);
    }
    
    println!("└──────────────┴────────────┴─────────────────────┴─────────────────────────────────────────┘");
    Ok(())
}

fn show_task_statistics() -> Result<()> {
    let (rt, sched) = ensure_scheduler()?;
    let stats = rt.block_on(async { sched.get_statistics().await });
    
    println!("📈 Scheduler Statistics:");
    println!("├─ Total jobs: {}", stats.total_jobs);
    println!("├─ Running jobs: {}", stats.running_jobs);
    println!("├─ Queued jobs: {}", stats.queued_jobs);
    println!("├─ Success rate: {:.1}%", stats.success_rate);
    println!("└─ Average execution time: {:.1} ms", stats.avg_execution_time_ms);
    Ok(())
}

fn delete_task(task_id: &str) -> Result<()> {
    let (rt, sched) = ensure_scheduler()?;
    let ok = rt.block_on(async { sched.cancel_job(task_id).await })?;
    
    if ok {
        println!("✅ Task {} deleted successfully", task_id);
    } else {
        println!("❌ Task {} not found", task_id);
    }
    Ok(())
}

fn enable_task(task_id: &str) -> Result<()> {
    let (rt, sched) = ensure_scheduler()?;
    let ok = rt.block_on(async { sched.enable_job(task_id).await })?;
    
    if ok {
        println!("✅ Task {} enabled successfully", task_id);
    } else {
        println!("❌ Task {} not found", task_id);
    }
    Ok(())
}

fn disable_task(task_id: &str) -> Result<()> {
    let (rt, sched) = ensure_scheduler()?;
    let ok = rt.block_on(async { sched.disable_job(task_id).await })?;
    
    if ok {
        println!("✅ Task {} disabled successfully", task_id);
    } else {
        println!("❌ Task {} not found", task_id);
    }
    Ok(())
}

fn show_help_and_examples() {
    println!("🎓 Scheduling Help & Examples");
    println!("────────────────────────────");
    println!();
    
    println!("📋 Command Line Usage:");
    println!("   schedule [options] [time] [command]");
    println!();
    
    println!("🎯 Options:");
    println!("   -l, --list        List all scheduled tasks");
    println!("   -d, --delete ID   Delete a specific task");
    println!("   --stats           Show scheduler statistics");
    println!("   --enable ID       Enable a specific task");
    println!("   --disable ID      Disable a specific task");
    println!("   --interval N CMD  Schedule task every N seconds");
    println!("   --at EPOCH CMD    Schedule task at specific epoch time");
    println!("   -h, --help        Show this help");
    println!();
    
    println!("📅 Time Formats:");
    println!("   • Cron: '0 9 * * *' (daily at 9 AM)");
    println!("   • Cron: '*/15 * * * *' (every 15 minutes)");
    println!("   • Epoch: Unix timestamp in seconds");
    println!("   • Date: 'YYYY-MM-DD HH:MM' (requires external 'at')");
    println!();
    
    println!("💡 Examples:");
    println!("   schedule '0 2 * * *' 'backup.sh'");
    println!("   schedule --interval 300 'check_health.sh'");
    println!("   schedule --at 1640995200 'new_year_message.sh'");
    println!("   schedule -l");
    println!("   schedule --delete task_001");
    println!("   schedule --stats");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_cron_like() {
        // Valid cron expressions
        assert!(is_cron_like("0 9 * * *"));  // 5 fields
        assert!(is_cron_like("*/15 * * * *"));  // 5 fields with */
        assert!(is_cron_like("0 0 1 * * 6"));  // 6 fields (with seconds)
        assert!(is_cron_like("30 2 * * 1-5"));  // 5 fields with ranges
        
        // Invalid expressions
        assert!(!is_cron_like("tomorrow"));  // 1 field
        assert!(!is_cron_like("9am"));  // 1 field
        assert!(!is_cron_like("* * *"));  // 3 fields
        assert!(!is_cron_like("* * * *"));  // 4 fields
        assert!(!is_cron_like("* * * * * * *"));  // 7 fields
    }

    #[test]
    fn test_schedule_cli_help() {
        // Test help option
        let result = schedule_cli(&["-h".to_string()]);
        assert!(result.is_ok());
        
        let result = schedule_cli(&["--help".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_schedule_cli_stats() {
        // Test stats option (should work even without scheduler)
        let result = schedule_cli(&["--stats".to_string()]);
        // May fail if scheduler can't initialize, but shouldn't panic
        let _ = result;
    }

    #[test]
    fn test_schedule_cli_list() {
        // Test list option
        let result = schedule_cli(&["-l".to_string()]);
        // May fail if scheduler can't initialize, but shouldn't panic
        let _ = result;
        
        let result = schedule_cli(&["--list".to_string()]);
        let _ = result;
    }

    #[test]
    fn test_schedule_cli_invalid_delete() {
        // Test delete without ID
        let result = schedule_cli(&["-d".to_string()]);
        // Should exit with error, but we can't test exit behavior easily
        let _ = result;
    }

    #[tokio::test]
    async fn test_scheduler_initialization() {
        // Test that scheduler can be initialized
        let result = ensure_scheduler();
        // This may fail in test environment, but shouldn't panic
        let _ = result;
    }

    #[test]
    fn test_interactive_functions() {
        // Test that helper functions don't panic with invalid inputs
        let result = schedule_task_once_epoch("echo test", 1640995200);
        let _ = result; // May fail without scheduler, but shouldn't panic
        
        let result = schedule_task_cron("echo test", "0 9 * * *");
        let _ = result; // May fail without scheduler, but shouldn't panic
        
        let result = schedule_task_interval("echo test", 300);
        let _ = result; // May fail without scheduler, but shouldn't panic
    }

    #[test]
    fn test_task_management_functions() {
        // Test task management functions
        let result = delete_task("nonexistent");
        let _ = result; // May fail without scheduler, but shouldn't panic
        
        let result = enable_task("nonexistent");
        let _ = result; // May fail without scheduler, but shouldn't panic
        
        let result = disable_task("nonexistent");
        let _ = result; // May fail without scheduler, but shouldn't panic
    }

    #[test]
    fn test_help_and_examples() {
        // Test that help function doesn't panic
        show_help_and_examples();
    }

    #[test]
    fn test_list_and_stats_functions() {
        // Test listing and stats functions
        let result = list_scheduled_tasks();
        let _ = result; // May fail without scheduler, but shouldn't panic
        
        let result = show_task_statistics();
        let _ = result; // May fail without scheduler, but shouldn't panic
    }
}
