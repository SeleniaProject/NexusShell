//! `dmidecode` builtin  Edump BIOS/DMI information.
//! Thin wrapper around the external `dmidecode` command. Requires root on most
//! systems.

use anyhow::Result;

pub async fn dmidecode_cli(args: &[String]) -> Result<()> {
    #[cfg(not(unix))]
    {
        println!("dmidecode: unsupported on this platform");
        return Ok(());
    }

    #[cfg(unix)]
    {
        if which::which("dmidecode").is_err() {
            println!("dmidecode: external 'dmidecode' not found. Install it first.");
            return Ok(());
        }
        let status = Command::new("dmidecode").args(args).status()?;
        if !status.success() {
            return Err(anyhow!("dmidecode: command exited with status {}", status));
        }
        Ok(())
    }
} 
