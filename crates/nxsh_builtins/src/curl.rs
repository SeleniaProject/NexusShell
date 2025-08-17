//! `curl` builtin - HTTP client utility.
//!
//! Delegates to the system `curl` binary when available to provide complete
//! HTTP functionality. When the binary is unavailable, falls back to a simple
//! internal implementation using ureq.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

/// Entry point for the `curl` builtin.
pub fn curl_cli(args: &[String]) -> Result<()> {
    // Prefer system curl when available.
    if let Ok(path) = which("curl") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("curl: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    #[cfg(feature = "net-http")]
    {
        // Lightweight built-in fallback: only supports `curl <URL>` (simple GET).
        if args.len() == 1 {
            let url = &args[0];
            let body = ureq::get(url)
                .call()
                .map_err(|e| anyhow!("curl: request failed: {e}"))?
                .into_string()
                .map_err(|e| anyhow!("curl: failed to read body: {e}"))?;
            print!("{body}");
            return Ok(());
        }
        return Err(anyhow!(
            "curl: internal fallback enabled but only supports simple GET (curl <URL>)"
        ));
    }

    #[cfg(not(feature = "net-http"))]
    {
        Err(anyhow!(
            "curl: internal HTTP disabled (built without 'net-http' feature); install system curl or rebuild with --features net-http"
        ))
    }
}
