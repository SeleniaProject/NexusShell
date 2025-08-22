//! `history` builtin command - manage command history

use std::io::Write;
use nxsh_core::{ShellResult, ExecutionResult, context::ShellContext, Builtin};
use nxsh_core::error::{ShellError, ErrorKind, RuntimeErrorKind, IoErrorKind, InternalErrorKind};

/// The `history` builtin command implementation
pub struct HistoryCommand;

impl Builtin for HistoryCommand {
    fn name(&self) -> &'static str {
        "history"
    }

    fn synopsis(&self) -> &'static str {
        "Display or manipulate command history"
    }

    fn description(&self) -> &'static str {
        "Display the command history list with line numbers. With options, can clear or delete specific entries."
    }

    fn usage(&self) -> &'static str {
        "history [-c] [-d offset] [n]"
    }

    fn execute(&self, ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        let mut clear_history = false;
        let mut delete_offset: Option<usize> = None;
        let mut show_count: Option<usize> = None;

        // Parse arguments
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-c" | "--clear" => {
                    clear_history = true;
                }
                "-d" | "--delete" => {
                    if i + 1 >= args.len() {
                        return Err(ShellError::new(
                            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                            "history: -d requires an offset argument"
                        ));
                    }
                    i += 1;
                    match args[i].parse::<usize>() {
                        Ok(offset) => delete_offset = Some(offset),
                        Err(_) => {
                            return Err(ShellError::new(
                                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                                format!("history: invalid offset '{}'", args[i])
                            ));
                        }
                    }
                }
                "-h" | "--help" => {
                    return self.show_help(ctx);
                }
                arg if arg.starts_with('-') => {
                    return Err(ShellError::new(
                        ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                        format!("history: unknown option '{arg}'")
                    ));
                }
                _ => {
                    // Try to parse as a count
                    match args[i].parse::<usize>() {
                        Ok(count) => show_count = Some(count),
                        Err(_) => {
                            return Err(ShellError::new(
                                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                                format!("history: invalid count '{}'", args[i])
                            ));
                        }
                    }
                }
            }
            i += 1;
        }

        if clear_history {
            self.clear_history(ctx)
        } else if let Some(offset) = delete_offset {
            self.delete_history_entry(offset, ctx)
        } else {
            self.show_history(show_count, ctx)
        }
    }

    fn help(&self) -> &'static str {
        "Display or manipulate command history"
    }
}

impl HistoryCommand {
    /// Show command history
    fn show_history(&self, count: Option<usize>, ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        if let Ok(history) = ctx.history.lock() {
            let entries: Vec<String> = if let Some(n) = count {
                // Show last n entries
                history.iter()
                    .enumerate()
                    .skip(history.len().saturating_sub(n))
                    .map(|(i, entry)| format!("{:4}  {}", i + 1, entry))
                    .collect()
            } else {
                // Show all entries with line numbers
                history.iter()
                    .enumerate()
                    .map(|(i, entry)| format!("{:4}  {}", i + 1, entry))
                    .collect()
            };

            let mut output = entries.join("\n");
            if !output.is_empty() {
                output.push('\n');
            }

            ctx.stdout.write(output.as_bytes())
                .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::FileWriteError), format!("Failed to write output: {e}")))?;

            Ok(ExecutionResult::success(0))
        } else {
            Err(ShellError::new(
                ErrorKind::InternalError(InternalErrorKind::LockError),
                "Failed to lock history"
            ))
        }
    }

    /// Show help text
    fn show_help(&self, ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let help_text = r#"history - Display or manipulate command history

USAGE:
    history [options] [n]

OPTIONS:
    -c, --clear     Clear the history list
    -d, --delete N  Delete the history entry at offset N
    -h, --help      Show this help message

ARGUMENTS:
    n               Show only the last n history entries

EXAMPLES:
    history         Show all history
    history 10      Show last 10 entries
    history -c      Clear all history
    history -d 5    Delete entry number 5
"#;

        ctx.stdout.write(help_text.as_bytes())
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::FileWriteError), format!("Failed to write help: {e}")))?;

        Ok(ExecutionResult::success(0))
    }

    /// Clear command history
    fn clear_history(&self, ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        if let Ok(mut history) = ctx.history.lock() {
            history.clear();
        } else {
            return Err(ShellError::new(
                ErrorKind::InternalError(InternalErrorKind::LockError),
                "Failed to lock history for clearing"
            ));
        }

        Ok(ExecutionResult::success(0))
    }

    /// Delete a specific history entry
    fn delete_history_entry(&self, offset: usize, ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        if offset == 0 {
            return Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                "history: offset must be greater than 0"
            ));
        }

        // Convert 1-based offset to 0-based index
        let index = offset - 1;

        if let Ok(mut history) = ctx.history.lock() {
            if index >= history.len() {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("history: offset {offset} out of range")
                ));
            }

            history.remove(index);
            Ok(ExecutionResult::success(0))
        } else {
            Err(ShellError::new(
                ErrorKind::InternalError(InternalErrorKind::LockError),
                "Failed to lock history for deletion"
            ))
        }
    }
}

/// Entry point for the history CLI
pub async fn history_cli(args: &[String]) -> anyhow::Result<()> {
    use nxsh_core::context::ShellContext;

    let mut context = ShellContext::new();
    let cmd = HistoryCommand;
    
    match cmd.execute(&mut context, args) {
        Ok(result) => {
            if result.exit_code == 0 {
                Ok(())
            } else {
                Err(anyhow::anyhow!("history failed with exit code {}", result.exit_code))
            }
        }
        Err(e) => Err(anyhow::anyhow!("history error: {}", e))
    }
}


/// Execute function stub
pub fn execute(_args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    eprintln!("Command not yet implemented");
    Ok(1)
}
