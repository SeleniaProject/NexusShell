//! `ulimit` builtin  Eview or set resource limits.
//! Supported options (subset):
//!   -a  : list all limits
//!   -n N: set/open files soft limit
//!   -c N: set core file size soft limit
//! When setting, `N` may be `unlimited`.
//! On Windows this builtin only prints an unsupported message.

use anyhow::{Result, anyhow};

#[cfg(unix)]
use nix::sys::resource::{getrlimit, setrlimit, Resource};

pub fn ulimit_cli(args: &[String]) -> Result<()> {
    #[cfg(windows)]
    {
        println!("ulimit: not supported on Windows");
        return Ok(());
    }
    #[cfg(unix)]
    {
        if args.is_empty() || (args.len() == 1 && args[0] == "-a") {
            print_limit("core file size", Resource::RLIMIT_CORE)?;
            print_limit("open files", Resource::RLIMIT_NOFILE)?;
            return Ok(());
        }
        if args.len() == 2 {
            let resource = match args[0].as_str() {
                "-n" => Resource::RLIMIT_NOFILE,
                "-c" => Resource::RLIMIT_CORE,
                _ => return Err(anyhow!("ulimit: unsupported option")),
            };
            let val = if args[1] == "unlimited" { 
                u64::MAX 
            } else { 
                args[1].parse::<u64>()? 
            };
            match setrlimit(resource, val, val) {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("Failed to set limit: {}", e)),
            }
        } else {
            Err(anyhow!("ulimit: invalid usage"))
        }
    }
}

#[cfg(unix)]
fn print_limit(name: &str, resource: Resource) -> Result<()> {
    match getrlimit(resource) {
        Ok((soft, _hard)) => {
            let v = if soft == u64::MAX { 
                "unlimited".to_string() 
            } else { 
                soft.to_string() 
            };
            println!("{}: {}", name, v);
            Ok(())
        }
        Err(e) => Err(anyhow!("Failed to get {}: {}", name, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn list_limits() {
        let _ = ulimit_cli(&["-a".into()]);
    }
} 
