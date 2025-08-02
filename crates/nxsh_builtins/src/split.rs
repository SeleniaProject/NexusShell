//! `split` command  Esplit a file into pieces.
//! Usage: split [-b N] FILE [PREFIX]
//!   -b N : byte size per piece, supports K/M suffix (default 1000000 bytes)
//! If PREFIX omitted, defaults to "x" producing xa, xb, ...

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tokio::task;

pub async fn split_cli(args: &[String]) -> Result<()> {
    if args.is_empty() { return Err(anyhow!("split: missing file operand")); }
    let mut size: u64 = 1_000_000; // 1 MB default
    let mut file_arg = "";
    let mut prefix = "x".to_string();
    let mut idx = 0;
    if args[0] == "-b" {
        if args.len() < 3 { return Err(anyhow!("split: invalid usage")); }
        size = parse_size(&args[1])?;
        idx = 2;
    }
    file_arg = &args[idx];
    if args.len() > idx + 1 { prefix = args[idx+1].clone(); }
    let p = Path::new(file_arg).to_path_buf();
    let pref = prefix.clone();
    task::spawn_blocking(move || split_file(p, pref, size)).await??;
    Ok(())
}

fn parse_size(s: &str) -> Result<u64> {
    if let Some(rest) = s.strip_suffix('K') { return Ok(rest.parse::<u64>()? * 1024); }
    if let Some(rest) = s.strip_suffix('M') { return Ok(rest.parse::<u64>()? * 1024*1024); }
    Ok(s.parse::<u64>()?)
}

fn split_file(path: std::path::PathBuf, prefix: String, chunk_size: u64) -> Result<()> {
    let mut infile = File::open(&path)?;
    let mut buf = vec![0u8; chunk_size as usize];
    let mut part = 0usize;
    loop {
        let n = infile.read(&mut buf)?;
        if n == 0 { break; }
        let suffix = encode_suffix(part);
        let out_path = format!("{}{}", prefix, suffix);
        let mut out = File::create(out_path)?;
        out.write_all(&buf[..n])?;
        part += 1;
    }
    Ok(())
}

fn encode_suffix(mut n: usize) -> String {
    // Coreutils default: aa, ab ... az, ba, bb ... etc.
    let mut chars = Vec::new();
    loop {
        chars.push(((n % 26) as u8 + b'a') as char);
        n /= 26;
        if n == 0 { break; }
        n -= 1; // adjust
    }
    chars.iter().rev().collect()
}

#[cfg(test)]
mod tests { use super::*; use tempfile::NamedTempFile; use std::io::Write;
#[tokio::test]
async fn split_basic(){ let mut f=NamedTempFile::new().unwrap(); f.write_all(&vec![0u8;2000]).unwrap(); split_cli(&[f.path().to_string_lossy().into()]).await.unwrap(); }} 
