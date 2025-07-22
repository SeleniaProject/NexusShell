//! `bzip2` builtin — compress files using the Burrows-Wheeler algorithm.
//!
//! Execution order:
//! 1. If `bzip2` binary exists, delegate all arguments for full compatibility.
//! 2. Otherwise use Rust fallback (`bzip2` crate) supporting simple `bzip2 <FILE>`
//!    producing `<FILE>.bz2` at default compression level.
//!
//! Unsupported flags in fallback will yield an error.

use anyhow::{anyhow, Context, Result};
use bzip2::{write::BzEncoder, Compression};
use std::{fs::File, io::copy, path::Path};
use std::process::Command;
use which::which;

pub fn bzip2_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("bzip2") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("bzip2: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    if args.len() != 1 {
        return Err(anyhow!("bzip2: system binary missing; fallback supports only 'bzip2 <FILE>'"));
    }
    let input = Path::new(&args[0]);
    if !input.is_file() {
        return Err(anyhow!("bzip2: '{}' is not a regular file", input.display()));
    }
    let output = input.with_extension(format!("{}bz2", input.extension().map(|s| s.to_string_lossy()+".").unwrap_or_default()));
    let mut infile = File::open(&input).with_context(|| format!("bzip2: cannot open {:?}", input))?;
    let outfile = File::create(&output).with_context(|| format!("bzip2: cannot create {:?}", output))?;
    let mut encoder = BzEncoder::new(outfile, Compression::best());
    copy(&mut infile, &mut encoder).context("bzip2: compression failed")?;
    encoder.finish().context("bzip2: finalize failed")?;
    Ok(())
} 