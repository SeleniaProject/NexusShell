//! `bunzip2` builtin — decompress .bz2 archives.
//!
//! Order of execution:
//! 1. Delegate to system `bunzip2`/`bzip2 -d` if available.
//! 2. Fallback Rust implementation via `bzip2::read::BzDecoder` supporting
//!    `bunzip2 <FILE.bz2>` → `<FILE>`.
//!
//! Flags are unsupported in fallback mode.

use anyhow::{anyhow, Context, Result};
use bzip2::read::BzDecoder;
use std::{fs::File, io::copy, path::Path};
use std::process::Command;
use which::which;

pub fn bunzip2_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("bunzip2") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("bunzip2: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // some distros use `bzip2 -d` when bunzip2 missing
    if let Ok(bzip2_bin) = which("bzip2") {
        let mut forwarded = vec!["-d".to_string()];
        forwarded.extend_from_slice(args);
        let status = Command::new(bzip2_bin)
            .args(&forwarded)
            .status()
            .map_err(|e| anyhow!("bunzip2: fallback 'bzip2 -d' failed: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    if args.len() != 1 {
        return Err(anyhow!("bunzip2: system binary missing; fallback supports only 'bunzip2 <FILE.bz2>'"));
    }
    let input = Path::new(&args[0]);
    if input.extension().and_then(|s| s.to_str()) != Some("bz2") {
        return Err(anyhow!("bunzip2: expected .bz2 input file"));
    }
    if !input.is_file() {
        return Err(anyhow!("bunzip2: '{}' is not a regular file", input.display()));
    }
    let output = input.with_extension("");
    let infile = File::open(&input).with_context(|| format!("bunzip2: cannot open {:?}", input))?;
    let mut decoder = BzDecoder::new(infile);
    let mut outfile = File::create(&output).with_context(|| format!("bunzip2: cannot create {:?}", output))?;
    copy(&mut decoder, &mut outfile).context("bunzip2: decompression failed")?;
    Ok(())
} 