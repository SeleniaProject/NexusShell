//! `unxz` builtin — LZMA decompression utility.
//!
//! Behaviour hierarchy:
//! 1. Use system `unxz` (or `xz -d`) when available.
//! 2. Fallback to Rust decoder via `xz2` supporting `unxz <FILE.xz>`.
//!
//! Flags unsupported in fallback mode.

use anyhow::{anyhow, Context, Result};
use std::{fs::File, io::{copy}, path::Path, process::Command};
use which::which;
use xz2::read::XzDecoder;

pub fn unxz_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("unxz") {
        let status = Command::new(path).args(args).status().map_err(|e| anyhow!("unxz: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    if let Ok(xz_bin) = which("xz") {
        let mut forwarded = vec!["-d".to_string()];
        forwarded.extend_from_slice(args);
        let status = Command::new(xz_bin).args(&forwarded).status().map_err(|e| anyhow!("unxz: fallback 'xz -d' failed: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    if args.len() != 1 {
        return Err(anyhow!("unxz: system binary missing; fallback supports only 'unxz <FILE.xz>'"));
    }
    let input = Path::new(&args[0]);
    if input.extension().and_then(|s| s.to_str()) != Some("xz") {
        return Err(anyhow!("unxz: expected .xz input file"));
    }
    if !input.is_file() { return Err(anyhow!("unxz: '{}' is not a regular file", input.display())); }
    let output = input.with_extension("");
    let infile = File::open(&input).with_context(|| format!("unxz: cannot open {:?}", input))?;
    let mut decoder = XzDecoder::new(infile);
    let mut outfile = File::create(&output).with_context(|| format!("unxz: cannot create {:?}", output))?;
    copy(&mut decoder, &mut outfile).context("unxz: decompression failed")?;
    Ok(())
} 