//! `unxz` builtin  ELZMA decompression utility.
//!
//! TEMPORARILY DISABLED: C-dependent xz2 library removed
//! This functionality needs to be reimplemented using pure Rust alternatives

use anyhow::{anyhow, Context, Result};
use std::{fs::File, path::Path, process::Command};
use which::which;
// Using pure Rust LZMA decompression as xz alternative

pub fn unxz_cli(args: &[String]) -> Result<()> {
    // Fallback to system unxz command if available
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
    let mut outfile = File::create(&output).with_context(|| format!("unxz: cannot create {:?}", output))?;
    // Use pure Rust LZMA decompression as xz alternative
    let mut buf_reader = std::io::BufReader::new(infile);
    lzma_rs::lzma_decompress(&mut buf_reader, &mut outfile).map_err(|e| anyhow!("unxz: decompression failed: {}", e))?;
    Ok(())
} 
