//! `unzip` builtin  Eextract ZIP archives.
//!
//! 1. Delegate to system `unzip` for full feature set.
//! 2. Fallback to `zip` crate supporting `unzip ARCHIVE.zip` extracting to cwd.
//!
//! Flags unsupported in fallback mode.

use anyhow::{anyhow, Context, Result};
use std::{fs::File, path::Path, process::Command};
use which::which;
use zip::read::ZipArchive;

pub fn unzip_cli(args: &[String]) -> Result<()> {
    if let Ok(path) = which("unzip") {
        let status = Command::new(path).args(args).status().map_err(|e| anyhow!("unzip: failed to launch backend: {e}"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
    if args.len() != 1 {
        return Err(anyhow!("unzip: system binary missing; fallback supports only 'unzip ARCHIVE.zip'"));
    }
    let archive = &args[0];
    let path = Path::new(archive);
    if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("zip") {
        return Err(anyhow!("unzip: '{archive}' is not a .zip file"));
    }
    let file = File::open(path).with_context(|| format!("unzip: cannot open {archive}"))?;
    let mut archive = ZipArchive::new(file).context("unzip: invalid zip archive")?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).context("unzip: read entry failed")?;
        if entry.name().ends_with('/') { continue; }
        let mut outfile = File::create(entry.name()).with_context(|| format!("unzip: cannot create {}", entry.name()))?;
        std::io::copy(&mut entry, &mut outfile).context("unzip: extract failed")?;
    }
    Ok(())
} 


/// Execute function stub
pub fn execute(_args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}
