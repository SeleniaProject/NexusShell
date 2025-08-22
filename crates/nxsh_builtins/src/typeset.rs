use nxsh_core::ShellError;

// typeset is an alias for declare in many shells
pub use crate::declare::*;

pub fn typeset_cli(args: &[String]) -> Result<(), ShellError> {
    // typeset is functionally identical to declare in most shells
    crate::declare::declare_cli(args)
}

