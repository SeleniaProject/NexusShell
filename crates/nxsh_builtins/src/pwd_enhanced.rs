//! `pwd` builtin - print current working directory.
//! Supports options:
//!   -L : logical path from $PWD (default)
//!   -P : physical path with symlink resolution

use anyhow::Result;
use nxsh_core::context::ShellContext;
use std::env;
use super::ui_design::{
    Colorize, TableFormatter, ColorPalette, Icons, Animation, 
    Notification, NotificationType
};

pub fn pwd_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    let mut physical = false;
    if !args.is_empty() {
        match args[0].as_str() {
            "-P" => physical = true,
            "-L" => physical = false,
            _ => {}
        }
    }
    
    let formatter = TableFormatter::new();
    
    let path = if physical {
        env::current_dir()?
    } else {
        ctx.get_var("PWD").map(|s| s.into()).unwrap_or(env::current_dir()?)
    };
    
    // Enhanced display with path components
    println!("{}", formatter.create_header("Current Directory"));
    println!("{}", Animation::typewriter("Locating current path...", 15));
    
    // Show beautiful path with breadcrumbs
    let path_str = path.display().to_string();
    let components: Vec<&str> = path_str.split(std::path::MAIN_SEPARATOR).collect();
    
    println!("\n{} {}", 
        formatter.icons.folder, 
        "Path:".primary()
    );
    
    // Show path with visual separators
    let breadcrumb = components.iter()
        .filter(|&&c| !c.is_empty())
        .map(|&c| c.info())
        .collect::<Vec<_>>()
        .join(&format!(" {} ", "▶".dim()));
    
    if !breadcrumb.is_empty() {
        println!("   {}", breadcrumb);
    } else {
        println!("   {}", path_str.success());
    }
    
    // Additional info for enhanced experience
    println!("\n{}", "📍 Location Details:".primary());
    println!("   • Full Path: {}", path_str.info());
    println!("   • Mode: {}", if physical { "Physical (-P)".warning() } else { "Logical (-L)".success() });
    
    // Show directory status if accessible
    if let Ok(metadata) = std::fs::metadata(&path) {
        if metadata.is_dir() {
            println!("   • Type: {}", "Directory".success());
            if let Ok(entries) = std::fs::read_dir(&path) {
                let count = entries.count();
                println!("   • Contents: {} {}", count.to_string().info(), if count == 1 { "item" } else { "items" });
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_pwd() {
        let ctx = ShellContext::new();
        pwd_cli(&[], &ctx).unwrap();
    }
}
