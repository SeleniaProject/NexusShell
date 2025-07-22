//! `break` builtin – exit N levels of loop.
//! Syntax: `break [N]` (default N=1)
//!
//! Currently NexusShell loop execution is under development. This builtin
//! therefore signals higher-level control flow via a custom `BreakSignal`
//! error which will be intercepted by future loop executor logic.
//! The implementation follows Bash semantics: if `N` is greater than the
//! number of nested loops, all loops are exited.

use anyhow::Result;
use std::num::ParseIntError;

/// Break signal type – wrapped in anyhow::Error when returned upstream.
#[derive(Debug, thiserror::Error)]
#[error("Break {levels} level(s)")]
pub struct BreakSignal {
    pub levels: usize,
}

/// Entry function for the builtin.
pub fn break_cli(args: &[String]) -> Result<()> {
    let levels = if args.is_empty() {
        1
    } else {
        parse_levels(&args[0])?
    };
    // Use anyhow to propagate control flow.
    Err(anyhow::Error::new(BreakSignal { levels }))
}

fn parse_levels(s: &str) -> Result<usize, ParseIntError> {
    if s.is_empty() {
        return Ok(1);
    }
    s.parse::<usize>()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_level() {
        let err = break_cli(&[]).unwrap_err();
        assert_eq!(err.downcast_ref::<BreakSignal>().unwrap().levels, 1);
    }
    #[test]
    fn level_two() {
        let err = break_cli(&["2".into()]).unwrap_err();
        assert_eq!(err.downcast_ref::<BreakSignal>().unwrap().levels, 2);
    }
} 