use nxsh_core::ShellError;

pub fn true_cmd_cli(_args: &[String]) -> Result<(), ShellError> {
    // The true command always succeeds and does nothing
    Ok(())
}

pub fn true_builtin() -> Result<i32, ShellError> {
    // Return exit status 0 (success)
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_command() {
        assert!(true_cmd_cli(&[]).is_ok());
        assert!(true_cmd_cli(&["any".to_string(), "args".to_string()]).is_ok());
        assert_eq!(true_builtin().unwrap(), 0);
    }
}
