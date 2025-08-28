// Test program to check the improved tab completion and syntax highlighting
use nxsh_ui::readline::{ReadLine, ReadLineConfig};
use std::io::{self, Write};

fn main() -> io::Result<()> {
    println!("=== Enhanced NexusShell ReadLine Test ===");
    println!("Features:");
    println!("- ✅ Smart tab completion with space insertion");
    println!("- ✅ Real-time syntax highlighting");
    println!("- ✅ Common prefix completion");
    println!("- ✅ Enhanced display for multiple candidates");
    println!("");
    println!("Try typing:");
    println!("  - Commands: 'ls', 'git', 'car' + TAB");
    println!("  - Paths: '/home', './src' + TAB");
    println!("  - Variables: '$PA' + TAB");
    println!("  - Options: 'ls -' + TAB");
    println!("");
    println!("Type 'exit' to quit.");
    println!("");

    let config = ReadLineConfig {
        enable_history: true,
        enable_completion: true,
        enable_syntax_highlighting: true,
        history_size: 100,
        completion_max_items: 20,
        auto_completion: true,
        vi_mode: false,
    };

    let mut readline = ReadLine::new(config)?;

    loop {
        match readline.read_line("nxsh> ") {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed == "exit" || trimmed == "quit" {
                    println!("Goodbye! 👋");
                    break;
                } else if trimmed.is_empty() {
                    continue;
                } else {
                    println!("You entered: {}", trimmed);
                    
                    // Test command recognition
                    if trimmed.starts_with("ls") {
                        println!("  📂 Directory listing command detected!");
                    } else if trimmed.starts_with("git") {
                        println!("  🔧 Git command detected!");
                    } else if trimmed.starts_with("cargo") {
                        println!("  🦀 Cargo command detected!");
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
