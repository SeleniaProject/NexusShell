//! `zstd` builtin — high-speed compression utility (Zstandard).
//!
//! Execution order:
//! 1. Delegate to system `zstd` binary for full flag compatibility.
//! 2. Fallback to minimal Rust implementation via `zstd` crate supporting
//!    `zstd <FILE>` (produces `<FILE>.zst`) with default level.
//!
//! Unsupported flags trigger an error in fallback mode.

use anyhow::{anyhow, Context, Result};
use std::{fs::File, io::{copy}, path::Path, process::Command};
use which::which;
use zstd::stream::write::Encoder;

pub fn zstd_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("zstd") {
        let status = Command::new(path).args(args).status().map_err(|e| anyhow!("zstd: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    if args.len() != 1 {
        return Err(anyhow!("zstd: system binary missing; fallback supports only 'zstd <FILE>'"));
    }
    let input = Path::new(&args[0]);
    if !input.is_file() {
        return Err(anyhow!("zstd: '{}' is not a regular file", input.display()));
    }
    let output = input.with_extension(format!("{}zst", input.extension().map(|s| s.to_string_lossy()+".").unwrap_or_default()));
    let mut infile = File::open(&input).with_context(|| format!("zstd: cannot open {:?}", input))?;
    let outfile = File::create(&output).with_context(|| format!("zstd: cannot create {:?}", output))?;
    let mut encoder = Encoder::new(outfile, 3)?;
    copy(&mut infile, &mut encoder).context("zstd: compression failed")?;
    encoder.finish().context("zstd: finalize failed")?;
    Ok(())
} 