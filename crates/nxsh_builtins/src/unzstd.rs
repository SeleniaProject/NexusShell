//! `unzstd` builtin — Zstandard decompression utility.
//!
//! 1. Try system `unzstd` (or `zstd -d`).
//! 2. Fallback to Rust `zstd` crate supporting `unzstd <FILE.zst>`.

use anyhow::{anyhow, Context, Result};
use std::{fs::File, io::copy, path::Path, process::Command};
use which::which;
use zstd::stream::read::Decoder;

pub fn unzstd_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("unzstd") {
        let status = Command::new(path).args(args).status().map_err(|e| anyhow!("unzstd: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    if let Ok(zstd_bin) = which("zstd") {
        let mut forwarded = vec!["-d".to_string()];
        forwarded.extend_from_slice(args);
        let status = Command::new(zstd_bin).args(&forwarded).status().map_err(|e| anyhow!("unzstd: fallback 'zstd -d' failed: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    if args.len() != 1 {
        return Err(anyhow!("unzstd: system binary missing; fallback supports only 'unzstd <FILE.zst>'"));
    }
    let input = Path::new(&args[0]);
    if input.extension().and_then(|s| s.to_str()) != Some("zst") {
        return Err(anyhow!("unzstd: expected .zst file"));
    }
    if !input.is_file() {
        return Err(anyhow!("unzstd: '{}' is not a regular file", input.display()));
    }
    let output = input.with_extension("");
    let infile = File::open(&input).with_context(|| format!("unzstd: cannot open {:?}", input))?;
    let mut decoder = Decoder::new(infile)?;
    let mut outfile = File::create(&output).with_context(|| format!("unzstd: cannot create {:?}", output))?;
    copy(&mut decoder, &mut outfile).context("unzstd: decompression failed")?;
    Ok(())
} 