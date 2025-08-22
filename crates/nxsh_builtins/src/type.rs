//! `type` builtin

use crate::common::{BuiltinResult, BuiltinContext};
use std::path::PathBuf;
use std::env;
use crate::command::BUILTIN_NAMES;
use crate::ui_design::{Colorize, ColorPalette, Icons};

pub fn execute(args: &[String], _ctx: &BuiltinContext) -> BuiltinResult<i32> {
    if args.is_empty() {
        eprintln!("type: usage: type [-afptP] name [name ...]");
        return Ok(1);
    }

    let _colors = ColorPalette::default();

    for name in args {
        // Note: Alias lookup not available through BuiltinContext yet
        if BUILTIN_NAMES.contains(&name.as_str()) {
            println!("{} {} {}", 
                Icons::EXECUTABLE, // Using EXECUTABLE instead of missing COMMAND
                name.colorize("blue"), 
                "is a shell builtin".colorize("green")
            );
            continue;
        }

        if let Some(path) = find_in_path(name) {
            println!("{} {} {} {}", 
                Icons::EXECUTABLE,
                name.colorize("magenta"),
                "is".colorize("green"),
                path.display().to_string().colorize("yellow")
            );
            continue;
        }

        println!("{} {}: {}", 
            Icons::ERROR,
            name.colorize("red"),
            "not found".colorize("red")
        );
    }

    Ok(0)
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    if let Ok(path_var) = env::var("PATH") {
        for path_dir in env::split_paths(&path_var) {
            let mut full_path = path_dir.join(name);
            
            // Try with common executable extensions on Windows
            #[cfg(windows)]
            {
                let extensions = ["", ".exe", ".cmd", ".bat", ".com"];
                for ext in &extensions {
                    if ext.is_empty() {
                        full_path = path_dir.join(name);
                    } else {
                        full_path = path_dir.join(format!("{}{}", name, ext));
                    }
                    
                    if full_path.exists() && full_path.is_file() {
                        return Some(full_path);
                    }
                }
            }
            
            #[cfg(not(windows))]
            {
                if full_path.exists() && full_path.is_file() {
                    return Some(full_path);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_in_path() {
        // This test will vary by system, but we can at least test the function doesn't panic
        let _ = find_in_path("ls");
        let _ = find_in_path("nonexistent_command_12345");
    }

    #[test]
    fn test_type_builtin() {
        let context = BuiltinContext::default();
        let result = execute(&["ls".to_string()], &context);
        assert!(result.is_ok());
    }
}
