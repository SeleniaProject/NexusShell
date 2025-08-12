//! `hdparm` builtin – simple disk performance benchmarking.
//!
//! Currently implemented options (subset):
//!   -t   : Buffered (sequential) read timing
//!   -T   : Cached timing (OS cache)  Emeasures memory copy speed
//!
//! Usage examples:
//!     hdparm -t /dev/sda
//!     hdparm -t -T disk.img
//!
//! Only read-only benchmarking is supported and limited to Unix-like systems.
//! On unsupported platforms the command prints a graceful message.

use anyhow::{anyhow, Result};
#[cfg(unix)] use std::{fs::File, io::{Read, Seek, SeekFrom}, path::Path, time::Instant};

pub async fn hdparm_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("hdparm: missing operand"));
    }

    let mut test_buffered = false;
    let mut test_cached = false;
    let mut device: Option<String> = None;

    for arg in args {
        match arg.as_str() {
            "-t" => test_buffered = true,
            "-T" => test_cached = true,
            _ => device = Some(arg.clone()),
        }
    }

    let dev = device.ok_or_else(|| anyhow!("hdparm: missing DEVICE"))?;

    if !test_buffered && !test_cached {
        return Err(anyhow!("hdparm: specify at least -t or -T"));
    }

    #[cfg(not(unix))]
    { let _ = &dev; println!("hdparm: benchmarking not supported on this platform"); return Ok(()); }
    #[cfg(unix)] {
        if test_cached { cached_test(&dev)?; }
        if test_buffered { buffered_test(&dev)?; }
        Ok(())
    }
}

#[cfg(unix)]
fn cached_test(dev: &str) -> Result<()> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    use std::path::Path;
    let mut file = File::open(Path::new(dev))?;
    // Read a small amount twice; second read should be cached.
    const SIZE: usize = 4 * 1024 * 1024; // 4 MiB
    let mut buf = vec![0u8; SIZE];

    // First read to warm cache (discard timing)
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut buf)?;

    // Cached read timing
    let start = Instant::now();
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut buf)?;
    let elapsed = start.elapsed().as_secs_f64();
    let mbps = (SIZE as f64 / 1_048_576_f64) / elapsed;
    println!("Cached read: {:.2} MB/s ({} bytes in {:.3} s)", mbps, SIZE, elapsed);
    Ok(())
}

#[cfg(unix)]
fn buffered_test(dev: &str) -> Result<()> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    use std::path::Path;
    let mut file = File::open(Path::new(dev))?;
    const TOTAL: usize = 128 * 1024 * 1024; // 128 MiB to sample
    const CHUNK: usize = 4 * 1024 * 1024; // 4 MiB buffer

    let mut buf = vec![0u8; CHUNK];
    file.seek(SeekFrom::Start(0))?;

    let start = Instant::now();
    let mut read_bytes = 0usize;
    while read_bytes < TOTAL {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break; // EOF encountered before TOTAL  Efine
        }
        read_bytes += n;
    }
    let elapsed = start.elapsed().as_secs_f64();
    if elapsed == 0.0 {
        return Err(anyhow!("hdparm: measurement too fast"));
    }
    let mbps = (read_bytes as f64 / 1_048_576_f64) / elapsed;
    println!("Buffered read: {:.2} MB/s ({} bytes in {:.3} s)", mbps, read_bytes, elapsed);
    Ok(())
} 
