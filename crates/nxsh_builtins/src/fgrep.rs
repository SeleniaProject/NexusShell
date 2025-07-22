//! `fgrep` command – fixed-string search.
//! Usage: fgrep PATTERN [FILE...]
//! It escapes all regex meta characters and forwards to the `grep` builtin.

use anyhow::{anyhow, Result};
use crate::grep::{grep_cli, GrepOptions};
use regex::escape as regex_escape;

/// Execute `fgrep` builtin.
pub fn fgrep_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("fgrep: missing PATTERN"));
    }
    let pattern = regex_escape(&args[0]);
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
    fn fgrep_matches_literal() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "a+b").unwrap();
        // Should match literally 'a+b' not regex plus.
        fgrep_cli(&["a+b".into(), file.path().to_string_lossy().into()]).unwrap();
    }
} 