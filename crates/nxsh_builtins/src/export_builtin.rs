use anyhow::Result;
use std::env;

/// CLI wrapper function for export command
pub fn export_cli(args: &[String]) -> Result<()> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("export - set environment variable");
        println!("Usage: export [NAME[=VALUE]]...");
        println!("  -h, --help     display this help and exit");
        
        if args.is_empty() {
            // Show all environment variables
            for (key, value) in env::vars() {
                println!("export {key}={value}");
            }
        }
        return Ok(());
    }
    
    for arg in args {
        if arg.starts_with('-') && arg != "--help" && arg != "-h" {
            eprintln!("export: unrecognized option '{arg}'");
            continue;
        }
        
        if let Some(equals_pos) = arg.find('=') {
            let name = &arg[..equals_pos];
            let value = &arg[equals_pos + 1..];
            env::set_var(name, value);
            println!("export {name}={value}");
        } else {
            // Export existing variable
            match env::var(arg) {
                Ok(value) => println!("export {arg}={value}"),
                Err(_) => {
                    env::set_var(arg, "");
                    println!("export {arg}=");
                }
            }
        }
    }
    
    Ok(())
}
