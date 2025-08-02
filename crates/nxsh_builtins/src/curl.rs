//! `curl` builtin  EHTTP client utility.
//!
//! Delegates to the system `curl` binary when available in `PATH` to preserve the
//! complete feature set and CLI surface area. When the binary is unavailable
//! (e.g. minimal containers or Windows without Git for Windows), it falls back
//! to a minimal internal implementation that currently supports simple HTTP
//! GET requests using the `ureq` crate with native-certs. Additional HTTP verbs and options
//! can be extended in subsequent iterations.

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

/// Entry-point exposed to the built-in registry.
///
/// `args` contains the raw, unparsed command-line arguments passed from the
/// NexusShell runtime (excluding the command name itself). The function either
/// delegates execution to the system `curl` binary or performs a basic blocking
/// GET request internally and writes the response body to stdout.
pub fn curl_cli(args: &[String]) -> Result<()> {
    // Prefer the full-featured system implementation when present.
    if let Ok(path) = which("curl") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("curl: failed to launch backend: {e}"))?;

        // Propagate the exact exit status so that shell scripts can rely on it.
        std::process::exit(status.code().unwrap_or(1));
    }

    // Lightweight built-in fallback: only supports `curl <URL>` (single GET).
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

    Err(anyhow!(
        "curl: backend not found and internal fallback supports only simple GET requests"
    ))
} 
