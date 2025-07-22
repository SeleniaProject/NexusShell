//! `xz` builtin — LZMA compression utility.
//!
//! Behaviour:
//! 1. If a system `xz` binary exists, forward all CLI args for complete functionality.
//! 2. Otherwise, fallback to Rust implementation via `xz2` crate supporting only
//!    `xz <FILE>` -> `<FILE>.xz` with default compression.
//! Unsupported flags cause an error in fallback.

use anyhow::{anyhow, Context, Result};
use std::{fs::File, io::{copy, Read}, path::Path, process::Command};
use which::which;
use xz2::write::XzEncoder;

pub fn xz_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("xz") {
        let status = Command::new(path).args(args).status().map_err(|e| anyhow!("xz: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    if args.len() != 1 {
        return Err(anyhow!("xz: system binary missing; fallback supports only 'xz <FILE>'"));
    }
    let input = Path::new(&args[0]);
    if !input.is_file() { return Err(anyhow!("xz: '{}' is not a regular file", input.display())); }
    let output = input.with_extension(format!("{}xz", input.extension().map(|s| s.to_string_lossy()+".").unwrap_or_default()));
    let infile = File::open(&input).with_context(|| format!("xz: cannot open {:?}", input))?;
    let outfile = File::create(&output).with_context(|| format!("xz: cannot create {:?}", output))?;
    let mut enc = XzEncoder::new(outfile, 6);
    copy(&mut infile.take(u64::MAX), &mut enc).context("xz: compression failed")?;
    enc.finish().context("xz: finalize failed")?;
    Ok(())
} 