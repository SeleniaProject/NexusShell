use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

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
        println!("  schedule '2024-12-25 09:00' 'echo Merry Christmas'");
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
            
            println!("schedule: Would schedule '{}' for time '{}'", command, time_spec);
            println!("schedule: Internal scheduling not implemented");
            eprintln!("schedule: Use external 'at' command for actual scheduling");
            std::process::exit(1);
        }
    }

    Ok(())
}
