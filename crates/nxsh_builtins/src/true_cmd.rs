//! NexusShell true command
//!
//! The true command that always succeeds (returns 0).

/// Execute the true command
pub fn execute(args: &[String]) -> Result<i32, String> {
    // true command ignores all arguments and always succeeds
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_always_succeeds() {
        assert_eq!(execute(&[]), Ok(0));
        assert_eq!(execute(&["any".to_string(), "args".to_string()]), Ok(0));
    }
}
