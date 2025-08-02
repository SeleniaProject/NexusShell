//! `return` builtin  Eterminate function execution with status.
//! Usage: return [N]
//! Emits `ReturnSignal` carrying status code which future function frame handler
//! will intercept.

use anyhow::Result;

#[derive(Debug, thiserror::Error)]
#[error("Function return with status {status}")]
pub struct ReturnSignal {
    pub status: i32,
}

pub fn return_cli(args: &[String]) -> Result<()> {
    let status = if args.is_empty() { 0 } else { args[0].parse().unwrap_or(1) };
    Err(anyhow::Error::new(ReturnSignal { status }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_status() {
        let err = return_cli(&[]).unwrap_err();
        assert_eq!(err.downcast_ref::<ReturnSignal>().unwrap().status, 0);
    }
} 
