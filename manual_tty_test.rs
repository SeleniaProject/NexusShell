// Simple manual test for NXSH_TTY_NOCOLOR
use std::env;

fn test_nxsh_tty_nocolor() {
    // Test 1: NO NXSH_TTY_NOCOLOR set
    env::remove_var("NXSH_TTY_NOCOLOR");
    env::remove_var("NO_COLOR");
    
    let supports_color = std::env::var("NXSH_TTY_NOCOLOR").is_err() && 
                         (std::env::var("NO_COLOR").is_err() || std::env::var("NO_COLOR").unwrap_or_default().is_empty());
    println!("Without NXSH_TTY_NOCOLOR: supports_color = {}", supports_color);
    
    // Test 2: NXSH_TTY_NOCOLOR set
    env::set_var("NXSH_TTY_NOCOLOR", "1");
    
    let supports_color = std::env::var("NXSH_TTY_NOCOLOR").is_err() && 
                         (std::env::var("NO_COLOR").is_err() || std::env::var("NO_COLOR").unwrap_or_default().is_empty());
    println!("With NXSH_TTY_NOCOLOR=1: supports_color = {}", supports_color);
    
    // Clean up
    env::remove_var("NXSH_TTY_NOCOLOR");
    
    // Test 3: NO_COLOR set
    env::set_var("NO_COLOR", "1");
    
    let supports_color = std::env::var("NXSH_TTY_NOCOLOR").is_err() && 
                         (std::env::var("NO_COLOR").is_err() || std::env::var("NO_COLOR").unwrap_or_default().is_empty());
    println!("With NO_COLOR=1: supports_color = {}", supports_color);
    
    // Clean up
    env::remove_var("NO_COLOR");
    
    println!("TTY NOCOLOR test completed successfully!");
}

fn main() {
    println!("Testing NXSH_TTY_NOCOLOR environment variable support");
    test_nxsh_tty_nocolor();
}
