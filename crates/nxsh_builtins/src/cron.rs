//! `cron` builtin – interface to system crontab.
//!
//! This command is a thin wrapper around the external `crontab` utility to keep
//! compatibility with existing cron workflows while integrating into
//! NexusShell.
//!
//! Supported options (mirroring common `crontab`):
//!   cron -l           # list current user crontab
//!   cron -e           # edit crontab via $EDITOR
//!   cron -r           # remove crontab
//!   cron FILE         # install crontab from FILE
//!
//! If no arguments are given, behaves like `cron -l`.
//! Requires `crontab` binary to be present in PATH.

use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;

pub async fn cron_cli(args: &[String]) -> Result<()> {
    #[cfg(not(unix))]
    {
        println!("cron: unsupported on this platform");
        return Ok(());
    }

    #[cfg(unix)]
    {
        if which::which("crontab").is_err() {
            println!("cron: external 'crontab' command not found. Install cron.");
            return Ok(());
        }

        let mut cmd = Command::new("crontab");
        match args.first() {
            None => {
                cmd.arg("-l");
            }
            Some(flag) if flag == "-l" || flag == "-e" || flag == "-r" => {
                cmd.arg(flag);
            }
            Some(file) => {
                if !Path::new(file).exists() {
                    return Err(anyhow!("cron: file '{}' not found", file));
                }
                cmd.arg(file);
            }
        }

        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("cron: command exited with status {}", status));
        }
        Ok(())
    }
} 