//! `type` builtin  Edetermine how a command name would be interpreted.
//! Output categories: alias, builtin, function (reserved), file, not found.

use anyhow::Result;
use nxsh_core::context::ShellContext;
use std::env;
use std::path::PathBuf;

use crate::command::BUILTIN_NAMES;
use super::ui_design::{Colorize, TableFormatter, ColorPalette, Icons};

pub fn type_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    for name in args {
        if let Some(alias) = ctx.get_alias(name) {
            println!("{} {} {} {}", 
                Icons::ALIAS,
                name.colorize(&ColorPalette::ACCENT), 
                "is an alias for".colorize(&ColorPalette::INFO),
                alias.colorize(&ColorPalette::SUCCESS)
            );
            continue;
        }
        if BUILTIN_NAMES.contains(&name.as_str()) {
            println!("{} {} {}", 
                Icons::BUILTIN,
                name.colorize(&ColorPalette::ACCENT), 
                "is a shell builtin".colorize(&ColorPalette::SUCCESS)
            );
            continue;
        }
        if let Some(path) = lookup_path(name) {
            println!("{} {} {} {}", 
                Icons::EXECUTABLE,
                name.colorize(&ColorPalette::ACCENT), 
                "is".colorize(&ColorPalette::INFO),
                path.display().to_string().colorize(&ColorPalette::SUCCESS)
            );
            continue;
        }
        println!("{} {} {}", 
            Icons::ERROR,
            name.colorize(&ColorPalette::ACCENT), 
            "not found".colorize(&ColorPalette::ERROR)
        );
    }
    Ok(())
}

fn lookup_path(cmd: &str) -> Option<PathBuf> {
    if cmd.contains('/') { let p = PathBuf::from(cmd); if p.is_file() { return Some(p); } }
    let path_env = env::var("PATH").unwrap_or_default();
    for dir in env::split_paths(&path_env) {
        let p = dir.join(cmd);
        if p.is_file() { return Some(p); }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn type_builtin() {
        let ctx = ShellContext::new();
        type_cli(&["echo".into()], &ctx).unwrap();
    }
} 
