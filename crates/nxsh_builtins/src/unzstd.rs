//! `unzstd` builtin  EZstandard decompression utility.
//!
//! TEMPORARILY DISABLED: C-dependent zstd library removed
//! This functionality needs to be reimplemented using pure Rust alternatives

use anyhow::{anyhow, Context, Result};
use std::{path::Path, process::Command};
use which::which;
// Note: zstd crate may not be available - using fallback implementation
#[cfg(feature = "zstd")]
use zstd::Decoder;

pub fn unzstd_cli(args: &[String]) -> Result<()> {
    // Fallback to system unzstd command if available
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
    
    #[cfg(feature = "zstd")]
    {
        let infile = File::open(&input).with_context(|| format!("unzstd: cannot open {:?}", input))?;
        let mut decoder = Decoder::new(infile)?;
        let mut outfile = File::create(&output).with_context(|| format!("unzstd: cannot create {:?}", output))?;
        copy(&mut decoder, &mut outfile).context("unzstd: decompression failed")?;
    }
    
    #[cfg(not(feature = "zstd"))]
    {
        // Fallback to system unzstd command
        let mut cmd = Command::new("unzstd");
        cmd.arg(&input).arg("-o").arg(&output);
        let status = cmd.status().context("unzstd: failed to execute system command")?;
        if !status.success() {
            return Err(anyhow!("unzstd: system command failed"));
        }
    }
    Ok(())
} 
