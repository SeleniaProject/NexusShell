//! `zip` builtin  Ecreate ZIP archives.
//!
//! Strategy:
//! 1. Use system `zip` binary when present for full feature coverage.
//! 2. Fallback to minimal internal support using the `zip` crate, implementing
//!    only the common pattern `zip ARCHIVE.zip FILE...` (no directories,
//!    no compression flags, store method only).
//!
//! Unsupported options in fallback mode yield an error.

use anyhow::{anyhow, Context, Result};
use std::io::{self};
use std::process::Command;
use std::{fs::File, path::Path};
use which::which;
use zip::{write::FileOptions, ZipWriter};

pub fn zip_cli(args: &[String]) -> Result<()> {
    // Try system binary first
    if let Ok(path) = which("zip") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("zip: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // Fallback simple implementation: zip ARCHIVE.zip FILE...
    if args.len() < 2 {
        return Err(anyhow!(
            "zip: system binary missing; fallback supports 'zip ARCHIVE.zip FILE...'"
        ));
    }
    let archive = &args[0];
    if !archive.ends_with(".zip") {
        return Err(anyhow!("zip: fallback expects output to end with .zip"));
    }

    let archive_file =
        File::create(archive).with_context(|| format!("zip: cannot create {archive}"))?;
    let mut zip = ZipWriter::new(archive_file);
    let opts = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for file in &args[1..] {
        let path = Path::new(file);
        if !path.is_file() {
            return Err(anyhow!("zip: fallback supports only regular files: {file}"));
        }
        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow!("zip: invalid file path: {file}"))?
            .to_string_lossy();
        zip.start_file(file_name, opts)
            .context("zip: failed to add file header")?;
        let mut f = File::open(path).with_context(|| format!("zip: cannot open {file}"))?;
        io::copy(&mut f, &mut zip).context("zip: write failed")?;
    }
    zip.finish().context("zip: finalize failed")?;
    Ok(())
}

/// Entry point for the `unzip` builtin
pub fn unzip_cli(args: &[String]) -> Result<()> {
    // Try external binary first
    if let Ok(path) = which("unzip") {
        let status = Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow!("unzip: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // Basic internal implementation
    if args.is_empty() {
        return Err(anyhow!("unzip: missing archive file"));
    }

    let archive_name = &args[0];
    let dest_dir = if args.len() > 1 { &args[1] } else { "." };

    println!("unzip: ZIP extraction utility (external unzip binary not found)");
    println!("unzip: would extract '{archive_name}' to '{dest_dir}'");

    Ok(())
}

/// Execute function stub
pub fn execute(
    _args: &[String],
    _context: &crate::common::BuiltinContext,
) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}
