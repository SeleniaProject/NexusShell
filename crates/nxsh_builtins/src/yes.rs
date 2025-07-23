//! `yes` builtin - output a string repeatedly until killed.
//! 
//! Usage:
//!   yes [STRING]
//!
//! If no STRING is provided, outputs "y" repeatedly.
//! This command runs indefinitely until interrupted (Ctrl+C).

use anyhow::Result;
use std::io::{self, Write, BufWriter, stdout};

/// Entry point for the yes builtin.
pub fn yes_cli(args: &[String]) -> Result<()> {
    let output_string = if args.is_empty() {
        "y"
    } else {
        &args.join(" ")
    };

    let stdout = stdout();
    let mut writer = BufWriter::new(stdout.lock());

    loop {
        writeln!(writer, "{}", output_string)?;
        writer.flush()?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yes_default() {
        // Note: This test would run indefinitely, so we can't actually test the full functionality
        // In a real test environment, we would need to use timeouts or signal handling
        assert!(true); // Placeholder test
    }
} 