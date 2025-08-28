//! `yes` builtin - output a string repeatedly until killed.
//! 
//! Usage:
//!   yes [STRING]
//!
//! If no STRING is provided, outputs "y" repeatedly.
//! This command runs indefinitely until interrupted (Ctrl+C).

use anyhow::Result;
use std::io::{Write, BufWriter, stdout};
use crate::common::{BuiltinResult, BuiltinContext};

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
        writeln!(writer, "{output_string}")?;
        writer.flush()?;
    }
}

/// Execute the yes builtin command
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    let output_string = if args.is_empty() {
        "y".to_string()
    } else {
        args.join(" ")
    };

    loop {
        println!("{output_string}");
        
        // Flush stdout to ensure immediate output
        if stdout().flush().is_err() {
            break;
        }
    }

    // This should never be reached in normal operation
    // as the command runs until interrupted
    Ok(0)
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_yes_default() {
        // Note: This test would run indefinitely, so we can't actually test the full functionality
        // In a real test environment, we would need to use timeouts or signal handling
    // Removed redundant assert!(true)
    }
} 

