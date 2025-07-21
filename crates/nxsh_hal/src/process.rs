use std::process::Command;

/// Spawn an external process and wait for completion.
pub fn spawn_and_wait(cmd: &str, args: &[&str]) -> std::io::Result<()> {
    Command::new(cmd).args(args).status().map(|_| ())
} 