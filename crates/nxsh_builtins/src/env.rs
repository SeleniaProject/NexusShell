//! `env` builtin  Edisplay or modify environment variables.
//!
//! Usage:
//!   env                 # list all env vars in KEY=VAL format
//!   env KEY             # print value for KEY
//!   env KEY=VAL ... CMD # set variables and exec command (not yet supported)
//! For now we only implement listing and retrieving variables.

use anyhow::{anyhow, Result};
use std::env;
use super::ui_design::{Colorize, TableFormatter, ColorPalette, Icons};

pub fn env_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        let header = format!(
            "{} {} Environment Variables {}",
            Icons::ENVIRONMENT,
            "┌─".colorize(&ColorPalette::BORDER),
            "─┐".colorize(&ColorPalette::BORDER)
        );
        println!("{}", header);
        
        let mut vars: Vec<_> = env::vars().collect();
        vars.sort_by(|a, b| a.0.cmp(&b.0));
        
        for (k, v) in vars {
            let key_colored = k.colorize(&ColorPalette::ACCENT);
            let value_colored = v.colorize(&ColorPalette::INFO);
            println!("{} {}={}", "│".colorize(&ColorPalette::BORDER), key_colored, value_colored);
        }
        
        let footer = format!(
            "{} {}",
            "└─".colorize(&ColorPalette::BORDER),
            "─".repeat(50).colorize(&ColorPalette::BORDER)
        );
        println!("{}{}", footer, "┘".colorize(&ColorPalette::BORDER));
        return Ok(());
    }

    if args.len() == 1 {
        let key = &args[0];
        match env::var(key) {
            Ok(val) => {
                println!("{} {} = {}", 
                    Icons::ENVIRONMENT,
                    key.colorize(&ColorPalette::ACCENT),
                    val.colorize(&ColorPalette::SUCCESS)
                );
            },
            Err(_) => return Err(anyhow!("env: {} not set", key)),
        }
        return Ok(());
    }

    Err(anyhow!("env: complex usage not yet supported"))
} 
