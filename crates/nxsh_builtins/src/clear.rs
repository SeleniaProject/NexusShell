//! `clear` builtin - Clear the terminal screen
//!
//! Cross-platform terminal screen clearing implementation

use std::io::{self, Write};
use crate::common::{BuiltinResult, BuiltinContext};

pub fn clear_cli(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        println!("clear - clear the terminal screen");
        println!("Usage: clear [OPTION]");
        println!("  -h, --help    display this help and exit");
        return Ok(());
    }

    // Send ANSI escape sequence to clear screen and move cursor to top-left
    print!("\x1B[2J\x1B[H");
    io::stdout().flush()?;
    
    Ok(())
}

pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    // Check for special styling options
    let show_banner = args.iter().any(|arg| arg == "--banner" || arg == "--stylish");
    
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("clear - clear the terminal screen with style");
        println!("Usage: clear [OPTION]");
        println!("  -h, --help     display this help and exit");
        println!("  --banner       show stylish NexusShell banner after clearing");
        println!("  --stylish      same as --banner");
        return Ok(0);
    }

    // Send ANSI escape sequence to clear screen and move cursor to top-left
    print!("\x1B[2J\x1B[H");
    if let Err(e) = io::stdout().flush() {
        eprintln!("clear: {}", e);
        return Ok(1);
    }
    
    if show_banner {
        show_nexus_banner();
    }
    
    Ok(0)
}

/// Display a stylish NexusShell banner
fn show_nexus_banner() {
    let cyan = "\x1b[38;2;0;245;255m";     // #00f5ff
    let purple = "\x1b[38;2;153;69;255m";  // #9945ff
    let coral = "\x1b[38;2;255;71;87m";    // #ff4757
    let green = "\x1b[38;2;46;213;115m";   // #2ed573
    let reset = "\x1b[0m";

    println!();
    println!("{}        ███╗   ██╗███████╗██╗  ██╗██╗   ██╗███████╗{}", cyan, reset);
    println!("{}        ████╗  ██║██╔════╝╚██╗██╔╝██║   ██║██╔════╝{}", cyan, reset);
    println!("{}        ██╔██╗ ██║█████╗   ╚███╔╝ ██║   ██║███████╗{}", purple, reset);
    println!("{}        ██║╚██╗██║██╔══╝   ██╔██╗ ██║   ██║╚════██║{}", purple, reset);
    println!("{}        ██║ ╚████║███████╗██╔╝ ██╗╚██████╔╝███████║{}", coral, reset);
    println!("{}        ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝{}", coral, reset);
    println!();
    println!("{}           ╔═══════════════════════════════════╗{}", green, reset);
    println!("{}           ║  {}🚀 Welcome to NexusShell 🚀{}     ║{}", green, cyan, green, reset);
    println!("{}           ║  {}✨ Cyberpunk Edition ✨{}         ║{}", green, purple, green, reset);
    println!("{}           ╚═══════════════════════════════════╝{}", green, reset);
    println!();
    println!("{}💡 Try: help, echo --stylish \"Hello!\", smart_alias list{}", cyan, reset);
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_basic() {
        let args = vec!["clear".to_string()];
        assert!(clear_cli(&args).is_ok());
    }

    #[test]
    fn test_clear_help() {
        let args = vec!["clear".to_string(), "--help".to_string()];
        assert!(clear_cli(&args).is_ok());
    }
}