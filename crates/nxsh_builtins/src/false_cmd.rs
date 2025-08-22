use nxsh_core::{ShellError, ErrorKind};
use nxsh_core::error::RuntimeErrorKind;
use crate::common::{BuiltinResult, BuiltinContext};

pub fn false_cmd_cli(_args: &[String]) -> Result<(), ShellError> {
    // The false command always fails
    Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "false"))
}

pub fn false_builtin() -> Result<i32, ShellError> {
    // Return exit status 1 (failure)
    Ok(1)
}

/// Execute the false builtin command
pub fn execute(_args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    // The false command always returns failure status
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_false_command() {
        assert!(false_cmd_cli(&[]).is_err());
        assert!(false_cmd_cli(&["any".to_string(), "args".to_string()]).is_err());
        assert_eq!(false_builtin().unwrap(), 1);
    }
}

