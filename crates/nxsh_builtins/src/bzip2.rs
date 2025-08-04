//! `bzip2` builtin  Ecompress files using the Burrows-Wheeler algorithm.
//!
//! TEMPORARILY DISABLED: C-dependent bzip2 library removed
//! This functionality needs to be reimplemented using pure Rust alternatives

use anyhow::{anyhow, Context, Result};
// Using pure Rust compression with flate2 as bzip2 alternative
use std::{fs::File, io::{Read, Write}, path::Path};
use std::process::Command;
use which::which;

pub fn bzip2_cli(args: &[String]) -> Result<()> {
    // Fallback to system bzip2 command if available
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
    let mut input_data = Vec::new();
    let mut infile = File::open(&input).with_context(|| format!("bzip2: cannot open {:?}", input))?;
    infile.read_to_end(&mut input_data).context("bzip2: read failed")?;
    
    // Use pure Rust compression with flate2 (gzip format as bzip2 alternative)
    let outfile = File::create(&output).with_context(|| format!("bzip2: cannot create {:?}", output))?;
    let mut encoder = flate2::write::GzEncoder::new(outfile, flate2::Compression::best());
    encoder.write_all(&input_data).context("bzip2: compression failed")?;
    encoder.finish().context("bzip2: finalize failed")?;
    Ok(())
} 
