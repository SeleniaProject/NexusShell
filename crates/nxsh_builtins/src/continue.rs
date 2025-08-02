//! `continue` builtin  Eskip N levels of loop and continue next iteration.
//! Usage: `continue [N]` (default 1)
//!
//! The actual control-flow handling is deferred to the loop executor via a
//! custom `ContinueSignal` error.

use anyhow::Result;
use std::num::ParseIntError;

#[derive(Debug, thiserror::Error)]
#[error("Continue {levels} level(s)")]
pub struct ContinueSignal {
    pub levels: usize,
}

pub fn continue_cli(args: &[String]) -> Result<()> {
    let levels = if args.is_empty() { 1 } else { parse_levels(&args[0])? };
    Err(anyhow::Error::new(ContinueSignal { levels }))
}

fn parse_levels(s: &str) -> Result<usize, ParseIntError> { s.parse::<usize>() }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_level() {
        let err = continue_cli(&[]).unwrap_err();
        assert_eq!(err.downcast_ref::<ContinueSignal>().unwrap().levels, 1);
    }
    #[test]
    fn level_three() {
        let err = continue_cli(&["3".into()]).unwrap_err();
        assert_eq!(err.downcast_ref::<ContinueSignal>().unwrap().levels, 3);
    }
} 
