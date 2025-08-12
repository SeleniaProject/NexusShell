// TTY NOCOLOR environment variable test
use nxsh_ui::{accessibility::AccessibilityManager, tui::supports_color};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing NXSH_TTY_NOCOLOR environment variable support");
    
    // Test TUI color support function
    println!("TUI supports_color(): {}", supports_color());
    
    // Test AccessibilityManager
    let mut manager = AccessibilityManager::new()?;
    manager.initialize().await?;
    
    println!("Accessibility blind mode enabled: {}", manager.is_blind_mode_enabled().await);
    println!("Colors disabled: {}", manager.are_colors_disabled().await);
    
    // Test with color output demonstration
    if !manager.are_colors_disabled().await {
        println!("\x1b[31mRed text\x1b[0m (should be red if colors enabled)");
        println!("\x1b[32mGreen text\x1b[0m (should be green if colors enabled)");
        println!("\x1b[34mBlue text\x1b[0m (should be blue if colors enabled)");
    } else {
        println!("Red text (colors disabled - plain text)");
        println!("Green text (colors disabled - plain text)");
        println!("Blue text (colors disabled - plain text)");
    }
    
    Ok(())
}
