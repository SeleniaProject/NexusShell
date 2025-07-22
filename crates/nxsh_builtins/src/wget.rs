//! `wget` builtin — non-interactive network downloader.
//!
//! Strategy:
//! 1. If a full-featured `wget` binary exists in `PATH`, spawn it and forward
//!    all command-line arguments to retain compatibility.
//! 2. If not available, fall back to a minimal internal implementation that
//!    supports the most common invocation pattern: `wget <URL>` or
//!    `wget -O <OUTPUT> <URL>`.
//!
//! The fallback uses the blocking client of the `reqwest` crate and writes the
//! retrieved bytes directly to the target path (or the basename of the URL when
//! no `-O` is supplied). HTTP(S) redirects are followed automatically. For
//! large files a streaming write is performed to avoid holding the entire body
//! in memory.
//!
//! Note: Only a subset of `wget` options are recognised by the fallback. When
//! advanced features are required (recursive download, FTP, user auth, etc.),
//! ensure the external `wget` binary is present.

use anyhow::{anyhow, Context, Result};
use std::{fs::File, io::copy, path::PathBuf, process::Command};
use url::Url;
use which::which;

/// Entry point for the `wget` builtin.
pub fn wget_cli(args: &[String]) -> Result<()> {
    // Prefer external binary if available for full feature set.
    if let Ok(path) = which("wget") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("wget: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // --- Internal fallback parsing ---
    // Accept patterns: [URL] OR [-O output] URL
    let mut output: Option<PathBuf> = None;
    let mut url_pos: Option<usize> = None;
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "-O" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(anyhow!("wget: -O requires an argument"));
                }
                output = Some(PathBuf::from(&args[idx]));
            }
            s if !s.starts_with('-') && url_pos.is_none() => {
                url_pos = Some(idx);
            }
            flag => {
                return Err(anyhow!(
                    "wget: unsupported option '{flag}' in internal fallback (install system wget for full support)"
                ));
            }
        }
        idx += 1;
    }

    let url_idx = url_pos.ok_or_else(|| anyhow!("wget: missing URL"))?;
    let url = &args[url_idx];

    let parsed = Url::parse(url).context("wget: invalid URL")?;
    let default_name = parsed
        .path_segments()
        .and_then(|segments| segments.last())
        .filter(|s| !s.is_empty())
        .unwrap_or("index.html");
    let outfile = output.unwrap_or_else(|| PathBuf::from(default_name));

    let response = reqwest::blocking::get(url)
        .with_context(|| format!("wget: failed to fetch {url}"))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "wget: server responded with HTTP status {}",
            response.status()
        ));
    }

    let mut file = File::create(&outfile)
        .with_context(|| format!("wget: cannot create file {:?}", outfile))?;
    let mut source = std::io::BufReader::new(response);
    copy(&mut source, &mut file).context("wget: failed while writing to file")?;

    Ok(())
} 