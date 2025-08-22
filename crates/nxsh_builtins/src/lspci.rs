//! `lspci` builtin  Elist PCI devices.
//! This command relies on the external `lspci` utility (pciutils). If the
//! binary is not found, an informative message is shown.

use anyhow::Result;

pub async fn lspci_cli(_args: &[String]) -> Result<()> {
    #[cfg(not(unix))]
    {
        println!("lspci: unsupported on this platform");
        Ok(())
    }

    #[cfg(unix)]
    {
        if which::which("lspci").is_err() {
            println!("lspci: external 'lspci' not found. Install pciutils.");
            return Ok(());
        }
    let status = Command::new("lspci").args(_args).status()?;
        if !status.success() {
            return Err(anyhow!("lspci: command exited with status {}", status));
        }
        Ok(())
    }
} 

