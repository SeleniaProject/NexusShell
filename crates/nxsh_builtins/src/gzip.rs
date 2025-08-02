//! `gzip` builtin  Ecompress files with DEFLATE algorithm.
//!
//! Preferred strategy:
//! 1. Delegate to the system `gzip` binary (ensures full option compatibility).
//! 2. If unavailable, fall back to an internal Rust implementation using the
//!    `flate2` crate. The fallback currently supports the most common usage
//!    pattern: `gzip <FILE>` which produces `<FILE>.gz` with default
//!    compression level.
//!
//! Unsupported flags will result in an error when using the fallback path.

use anyhow::{anyhow, Context, Result};
use flate2::{write::GzEncoder, Compression};
use std::{fs::File, io::copy, path::Path};
use std::process::Command;
use which::which;

pub fn gzip_cli(args: &[String]) -> Result<()> {
    // Try system gzip first
    if let Ok(path) = which("gzip") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("gzip: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // Fallback: only support single file compression without flags
    if args.len() != 1 {
        return Err(anyhow!("gzip: system binary missing; fallback supports only 'gzip <FILE>'"));
    }
    let input_path = Path::new(&args[0]);
    if !input_path.is_file() {
        return Err(anyhow!("gzip: '{}' is not a regular file", input_path.display()));
    }
    let output_path = input_path.with_extension(format!("{}gz", input_path.extension().map(|s| s.to_string_lossy() + ".").unwrap_or_default()));

    let mut infile = File::open(&input_path)
        .with_context(|| format!("gzip: cannot open input file {:?}", input_path))?;
    let outfile = File::create(&output_path)
        .with_context(|| format!("gzip: cannot create output file {:?}", output_path))?;
    let mut encoder = GzEncoder::new(outfile, Compression::default());
    copy(&mut infile, &mut encoder).context("gzip: compression failed")?;
    encoder.finish().context("gzip: finalize failed")?;

    Ok(())
} 
