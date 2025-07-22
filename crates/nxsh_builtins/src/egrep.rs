//! `egrep` command – extended regex search wrapper around `grep`.
//! Usage: egrep PATTERN [FILE...]
//! It forwards to `grep` builtin with identical behavior since PCRE2 already
//! supports extended regular expressions by default.

use anyhow::{anyhow, Result};
use crate::grep::{grep_cli, GrepOptions};

/// Execute `egrep` builtin.
pub fn egrep_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("egrep: missing PATTERN"));
    }
    let pattern = args[0].clone();
    let paths = &args[1..];
    let opts = GrepOptions { pattern, json: false };
    grep_cli(opts, paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn egrep_matches() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "foo123").unwrap();
        writeln!(file, "bar").unwrap();
        // Capture output
        egrep_cli(&["[0-9]+".into(), file.path().to_string_lossy().into()]).unwrap();
    }
} 