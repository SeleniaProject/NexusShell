//! `fgrep` command  Efixed-string search.
//! Usage: fgrep PATTERN [FILE...]
//! It escapes all regex meta characters and forwards to the `grep` builtin.

use anyhow::{anyhow, Result};
use crate::grep::grep_cli;

/// Execute `fgrep` builtin.
pub fn fgrep_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("fgrep: missing PATTERN"));
    }
    
    // Create modified args with -F flag for fixed strings
    let mut grep_args = vec!["-F".to_string(), args[0].clone()];
    grep_args.extend_from_slice(&args[1..]);
    
    grep_cli(&grep_args)
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

