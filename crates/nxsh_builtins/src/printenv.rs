//! `printenv` builtin  Eprint environment variable value.
//!
//! Usage:
//!   printenv VAR          # print value of VAR
//!   printenv              # list all KEY=VAL pairs like `env`
//! If VAR is not set, exits with error.

use anyhow::{anyhow, Result};
use std::env;

pub fn printenv_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        // same as env list
        for (k, v) in env::vars() {
            println!("{}={}", k, v);
        }
        return Ok(());
    }

    for var in args {
        match env::var(var) {
            Ok(val) => println!("{}", val),
            Err(_) => return Err(anyhow!("printenv: {} not set", var)),
        }
    }
    Ok(())
} 

