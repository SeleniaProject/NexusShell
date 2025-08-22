//! Stylish echo command for NexusShell

use crate::common::{BuiltinResult, BuiltinError, BuiltinContext};

/// Execute the echo command with stylish output
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    if args.is_empty() {
        println!();
        return Ok(0);
    }

    // Check for special styling options
    let mut interpret_escapes = false;
    let mut no_newline = false;
    let mut colorful = false;
    let mut text_parts = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-n" => no_newline = true,
            "-e" => interpret_escapes = true,
            "-E" => interpret_escapes = false,
            "--color" => colorful = true,
            "--stylish" => {
                // Special NexusShell stylish mode
                let message = args.iter()
                    .filter(|a| !a.starts_with("-"))
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ");
                print_stylish_message(&message);
                return Ok(0);
            }
            _ if !arg.starts_with("-") => {
                text_parts.push(arg.clone());
            }
            _ => {} // Ignore unknown options
        }
    }

    let message = text_parts.join(" ");
    
    if colorful {
        print_colorful_message(&message);
    } else if interpret_escapes {
        print_with_escapes(&message);
    } else {
        print!("{}", message);
    }

    if !no_newline {
        println!();
    }

    Ok(0)
}

/// Print message with stylish cyberpunk formatting
fn print_stylish_message(message: &str) {
    println!("┌─ 🚀 NexusShell Output ─────────────────┐");
    println!("│                                        │");
    println!("│  ✨ {}  ✨", message);
    println!("│                                        │");
    println!("└────────────────────────────────────────┘");
}

/// Print message with colorful ANSI codes
fn print_colorful_message(message: &str) {
    // Use our cyberpunk theme colors
    let cyan = "\x1b[38;2;0;245;255m";     // #00f5ff
    let purple = "\x1b[38;2;153;69;255m";  // #9945ff
    let coral = "\x1b[38;2;255;71;87m";    // #ff4757
    let reset = "\x1b[0m";

    // Alternate colors for each word
    let words: Vec<&str> = message.split_whitespace().collect();
    let colors = [cyan, purple, coral];
    
    for (i, word) in words.iter().enumerate() {
        let color = colors[i % colors.len()];
        print!("{}{}{}", color, word, reset);
        if i < words.len() - 1 {
            print!(" ");
        }
    }
}

/// Print message with escape sequence interpretation
fn print_with_escapes(message: &str) {
    let mut chars = message.chars().peekable();
    let mut result = String::new();
    
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next_char) = chars.peek() {
                match next_char {
                    'n' => {
                        result.push('\n');
                        chars.next();
                    }
                    't' => {
                        result.push('\t');
                        chars.next();
                    }
                    'r' => {
                        result.push('\r');
                        chars.next();
                    }
                    '\\' => {
                        result.push('\\');
                        chars.next();
                    }
                    _ => {
                        result.push(c);
                    }
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    
    print!("{}", result);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::BuiltinContext;

    #[test]
    fn test_basic_echo() {
        let args = vec!["Hello".to_string(), "World".to_string()];
        let context = BuiltinContext::default();
        let result = execute(&args, &context);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_empty_echo() {
        let args = vec![];
        let context = BuiltinContext::default();
        let result = execute(&args, &context);
        assert_eq!(result.unwrap(), 0);
    }
}
