//! `lsof` builtin — list open files.
//!
//! Full re-implementation of `lsof` is complex and platform-dependent。
//! このビルトインでは、システムに `lsof` または `gnumsf` がインストールされている場合は
//! そのバイナリに委譲し、存在しない場合はエラーを返します。
//!
//! Usage: `lsof [OPTIONS] [FILE|PID...]` — 引数は委譲先にそのまま渡されます。

use anyhow::{anyhow, Result};
use std::process::Command;
use which::which;

pub fn lsof_cli(args: &[String]) -> Result<()> {
    // 候補バイナリ
    for bin in ["lsof", "lsof.exe"].iter() {
        if let Ok(path) = which(bin) {
            let status = Command::new(path)
                .args(args)
                .status()
                .map_err(|e| anyhow!("lsof: failed to launch external binary: {e}"))?;
            if status.success() {
                return Ok(());
            } else {
                return Err(anyhow!("lsof: external binary exited with status {:?}", status.code()));
            }
        }
    }
    Err(anyhow!("lsof: no compatible backend found in PATH"))
} 