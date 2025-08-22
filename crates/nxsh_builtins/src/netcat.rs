//! `netcat` builtin - Alias for nc (netcat) functionality.

use anyhow::Result;

/// Entry point for the `netcat` builtin (alias for nc)
pub fn netcat_cli(args: &[String]) -> Result<()> {
    // Simply delegate to nc_cli
    crate::nc::nc_cli(args)
}

