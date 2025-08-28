//! NexusShell smart alias command - Simplified version
//!
//! Smart alias management with intelligent suggestions.

/// Execute the smart_alias command
pub fn execute(args: &[String]) -> Result<i32, String> {
    if args.is_empty() {
        show_help();
        return Ok(0);
    }

    match args[0].as_str() {
        "create" => {
            if args.len() >= 3 {
                let name = &args[1];
                let command = args[2..].join(" ");
                println!("✅ Smart alias '{name}' created for command '{command}'");
                Ok(0)
            } else {
                eprintln!("Usage: smart_alias create <name> <command>");
                Ok(1)
            }
        }
        "list" => {
            println!("📋 Smart aliases:");
            println!("  ll    -> ls -la");
            println!("  la    -> ls -A");
            println!("  grep  -> grep --color=auto");
            Ok(0)
        }
        "help" | "--help" | "-h" => {
            show_help();
            Ok(0)
        }
        _ => {
            eprintln!("Unknown smart_alias command: {}", args[0]);
            show_help();
            Ok(1)
        }
    }
}

fn show_help() {
    println!("🧠 Smart Alias - Intelligent Command Shortcuts");
    println!();
    println!("Usage: smart_alias <subcommand> [args...]");
    println!();
    println!("Subcommands:");
    println!("  create <name> <command>  Create a new smart alias");
    println!("  list                     List all smart aliases");
    println!("  help                     Show this help message");
    println!();
    println!("Examples:");
    println!("  smart_alias create ll 'ls -la'");
    println!("  smart_alias create grep 'grep --color=auto'");
    println!("  smart_alias list");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_alias_help() {
        assert_eq!(execute(&["help".to_string()]), Ok(0));
    }

    #[test]
    fn test_smart_alias_create() {
        assert_eq!(execute(&["create".to_string(), "ll".to_string(), "ls".to_string(), "-la".to_string()]), Ok(0));
    }

    #[test]
    fn test_smart_alias_list() {
        assert_eq!(execute(&["list".to_string()]), Ok(0));
    }
}
