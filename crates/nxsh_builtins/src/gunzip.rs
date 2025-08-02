//! `gunzip` builtin  EDEFLATE decompression utility.
//!
//! Execution order:
//! 1. Prefer the system `gunzip` binary for full option compatibility.
//! 2. If unavailable, fall back to a minimal Rust implementation using
//!    `flate2::read::GzDecoder`. The fallback currently supports the simplest
//!    form: `gunzip <FILE.gz>` which produces `<FILE>`.
//!
//! Limitations of the fallback: no wildcards, no stdin/stdout streaming, no
//! preservation of original timestamps, and no flags. These features are
//! available only when the external binary is installed.

use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use std::{fs::File, io::copy, path::Path};
use std::process::Command;
use which::which;

pub fn gunzip_cli(args: &[String]) -> Result<()> {
    // 1. Try system gunzip (often symlink to gzip -d)
    if let Ok(path) = which("gunzip") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("gunzip: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // 2. Fallback supports only single .gz file, no flags
    if args.len() != 1 {
        return Err(anyhow!(
            "gunzip: system binary missing; fallback supports only 'gunzip <FILE.gz>'"
        ));
    }

    let input_path = Path::new(&args[0]);
    if input_path.extension().and_then(|s| s.to_str()) != Some("gz") {
        return Err(anyhow!("gunzip: expected .gz input file"));
    }
    if !input_path.is_file() {
        return Err(anyhow!("gunzip: '{}' is not a regular file", input_path.display()));
    }

    let output_path = input_path.with_extension(""); // strip .gz
    let infile = File::open(&input_path)
        .with_context(|| format!("gunzip: cannot open input file {:?}", input_path))?;

    let mut decoder = GzDecoder::new(infile);
    let mut outfile = File::create(&output_path)
        .with_context(|| format!("gunzip: cannot create output file {:?}", output_path))?;

    copy(&mut decoder, &mut outfile).context("gunzip: decompression failed")?;

    Ok(())
} 
