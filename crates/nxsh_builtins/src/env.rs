//! `env` builtin  Edisplay or modify environment variables.
//!
//! Usage:
//!   env                 # list all env vars in KEY=VAL format
//!   env KEY             # print value for KEY
//!   env KEY=VAL ... CMD # set variables and exec command (not yet supported)
//! For now we only implement listing and retrieving variables.

use anyhow::{anyhow, Result};
use std::env;

pub fn env_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        for (k, v) in env::vars() {
            println!("{}={}", k, v);
        }
        return Ok(());
    }

    if args.len() == 1 {
        let key = &args[0];
        match env::var(key) {
            Ok(val) => println!("{}", val),
            Err(_) => return Err(anyhow!("env: {} not set", key)),
        }
        return Ok(());
    }

    Err(anyhow!("env: complex usage not yet supported"))
} 
