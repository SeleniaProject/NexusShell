//! `history` builtin command - manage command history
//!
//! This module implements the history builtin command for displaying
//! and managing the shell's command history.

use crate::common::{i18n::*, logging::*};
use std::io::Write;
use std::collections::HashMap;
use nxsh_core::{Builtin, Context, ExecutionResult, ShellResult, ShellError, ErrorKind, StreamData};
use nxsh_core::error::{RuntimeErrorKind, IoErrorKind, InternalErrorKind};

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

    fn affects_shell_state(&self) -> bool {
        true // history can modify the command history
    }

    fn invoke(&self, ctx: &mut Context) -> ShellResult<ExecutionResult> {
        let mut clear_history = false;
        let mut delete_offset: Option<usize> = None;
        let mut show_count: Option<usize> = None;

        // Parse arguments
        let mut i = 1; // Skip command name
        while i < ctx.args.len() {
            let arg = &ctx.args[i];
            
            match arg.as_str() {
                "-c" => clear_history = true,
                "-d" => {
                    // Next argument should be the offset to delete
                    if i + 1 < ctx.args.len() {
                        i += 1;
                        let offset_str = &ctx.args[i];
                        match offset_str.parse::<usize>() {
                            Ok(offset) => delete_offset = Some(offset),
                            Err(_) => {
                                return Err(ShellError::new(
                                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                                    format!("history: invalid offset: {}", offset_str)
                                ));
                            }
                        }
                    } else {
                        return Err(ShellError::new(
                            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                            "history: -d requires an offset argument"
                        ));
                    }
                }
                arg if arg.starts_with('-') => {
                    return Err(ShellError::new(
                        ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                        format!("history: invalid option: {}", arg)
                    ));
                }
                arg => {
                    // Numeric argument for show count
                    match arg.parse::<usize>() {
                        Ok(count) => show_count = Some(count),
                        Err(_) => {
                            return Err(ShellError::new(
                                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                                format!("history: invalid number: {}", arg)
                            ));
                        }
                    }
                }
            }
            
            i += 1;
        }

        // Execute the requested operation
        if clear_history {
            self.clear_history(ctx)
        } else if let Some(offset) = delete_offset {
            self.delete_history_entry(offset, ctx)
        } else {
            self.display_history(show_count, ctx)
        }
    }
}

impl HistoryCommand {
    /// Create a new history command instance
    pub fn new() -> Self {
        Self
    }

    /// Display the command history
    fn display_history(&self, show_count: Option<usize>, ctx: &mut Context) -> ShellResult<ExecutionResult> {
        let history = ctx.env.get_history();
        
        if history.is_empty() {
            // No history to display
            return Ok(ExecutionResult::success(0));
        }

        let mut output = String::new();
        
        // Determine which entries to show
        let start_index = if let Some(count) = show_count {
            if count >= history.len() {
                0
            } else {
                history.len() - count
            }
        } else {
            0
        };

        // Display history entries with line numbers
        for (i, entry) in history.iter().enumerate().skip(start_index) {
            output.push_str(&format!("{:5} {}\n", i + 1, entry));
        }

        // Write to stdout
        if !output.is_empty() {
            ctx.stdout.write(StreamData::Text(output))
                .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::FileWriteError), format!("Failed to write output: {}", e)))?;
        }

        Ok(ExecutionResult::success(0))
    }

    /// Clear the command history
    fn clear_history(&self, ctx: &mut Context) -> ShellResult<ExecutionResult> {
        // Clear the history in the shell context
        if let Ok(mut history) = ctx.env.history.lock() {
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
    fn delete_history_entry(&self, offset: usize, ctx: &mut Context) -> ShellResult<ExecutionResult> {
        if offset == 0 {
            return Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                "history: offset must be greater than 0"
            ));
        }

        // Convert 1-based offset to 0-based index
        let index = offset - 1;

        if let Ok(mut history) = ctx.env.history.lock() {
            if index >= history.len() {
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                    format!("history: offset {} out of range", offset)
                ));
            }
            
            history.remove(index);
        } else {
            return Err(ShellError::new(
                ErrorKind::InternalError(InternalErrorKind::LockError),
                "Failed to lock history for deletion"
            ));
        }

        Ok(ExecutionResult::success(0))
    }

    /// Add a command to history (utility function)
    pub fn add_to_history(command: &str, ctx: &mut Context) {
        ctx.env.add_history(command.to_string());
    }

    /// Get the last command from history
    pub fn get_last_command(ctx: &Context) -> Option<String> {
        let history = ctx.env.get_history();
        history.last().cloned()
    }

    /// Search history for commands containing a pattern
    pub fn search_history(pattern: &str, ctx: &Context) -> Vec<(usize, String)> {
        let history = ctx.env.get_history();
        let mut results = Vec::new();

        for (i, entry) in history.iter().enumerate() {
            if entry.contains(pattern) {
                results.push((i + 1, entry.clone())); // 1-based indexing
            }
        }

        results
    }

    /// Get history statistics
    pub fn get_history_stats(ctx: &Context) -> HistoryStats {
        let history = ctx.env.get_history();
        let total_commands = history.len();
        
        // Count unique commands
        let mut unique_commands = std::collections::HashSet::new();
        for entry in &history {
            unique_commands.insert(entry);
        }
        let unique_count = unique_commands.len();

        // Find most common commands
        let mut command_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for entry in &history {
            // Get the first word (command name)
            if let Some(command) = entry.split_whitespace().next() {
                *command_counts.entry(command).or_insert(0) += 1;
            }
        }

        let mut most_common: Vec<_> = command_counts.into_iter().collect();
        most_common.sort_by(|a, b| b.1.cmp(&a.1));
        most_common.truncate(10); // Top 10

        HistoryStats {
            total_commands,
            unique_commands: unique_count,
            most_common_commands: most_common.into_iter().map(|(cmd, count)| (cmd.to_string(), count)).collect(),
        }
    }
}

/// History statistics structure
#[derive(Debug, Clone)]
pub struct HistoryStats {
    pub total_commands: usize,
    pub unique_commands: usize,
    pub most_common_commands: Vec<(String, usize)>,
}

impl Default for HistoryCommand {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a history command
pub fn history_cli(args: &[String], ctx: &mut nxsh_core::context::ShellContext) -> ShellResult<()> {
    use nxsh_core::stream::{Stream, StreamType};
    
    let mut context = Context::new(
        args.to_vec(),
        ctx,
        Stream::new(StreamType::Byte),
        Stream::new(StreamType::Text),
        Stream::new(StreamType::Byte),
    )?;

    let history_cmd = HistoryCommand::new();
    let result = history_cmd.invoke(&mut context)?;
    
    // Output the result to stdout if any
    if let Ok(data) = context.stdout.collect() {
        for item in data {
            if let Ok(text) = item.to_string() {
                print!("{}", text);
            }
        }
    }

    if result.is_success() {
        Ok(())
    } else {
        Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound), format!("history failed with exit code {}", result.exit_code)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nxsh_core::context::ShellContext;
    use nxsh_core::stream::{Stream, StreamType};

    fn create_test_context(args: Vec<String>) -> (Context, ShellContext) {
        let mut shell_ctx = ShellContext::new();
        let context = Context::new(
            args,
            &mut shell_ctx,
            Stream::new(StreamType::Byte),
            Stream::new(StreamType::Text),
            Stream::new(StreamType::Byte),
        ).unwrap();
        (context, shell_ctx)
    }

    #[test]
    fn test_history_display() {
        let history_cmd = HistoryCommand::new();
        let (mut ctx, _shell_ctx) = create_test_context(vec!["history".to_string()]);
        
        // Add some history entries
        ctx.env.add_history("ls -l".to_string());
        ctx.env.add_history("cd /tmp".to_string());
        ctx.env.add_history("echo hello".to_string());
        
        let result = history_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        // Check that output was generated
        let output = ctx.stdout.collect().unwrap();
        if !output.is_empty() {
            let output_text = output[0].to_string().unwrap();
            assert!(output_text.contains("ls -l"));
            assert!(output_text.contains("cd /tmp"));
            assert!(output_text.contains("echo hello"));
        }
    }

    #[test]
    fn test_history_display_with_count() {
        let history_cmd = HistoryCommand::new();
        let (mut ctx, _shell_ctx) = create_test_context(vec!["history".to_string(), "2".to_string()]);
        
        // Add some history entries
        ctx.env.add_history("command1".to_string());
        ctx.env.add_history("command2".to_string());
        ctx.env.add_history("command3".to_string());
        
        let result = history_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        // Check that only the last 2 commands are shown
        let output = ctx.stdout.collect().unwrap();
        if !output.is_empty() {
            let output_text = output[0].to_string().unwrap();
            assert!(!output_text.contains("command1"));
            assert!(output_text.contains("command2"));
            assert!(output_text.contains("command3"));
        }
    }

    #[test]
    fn test_history_clear() {
        let history_cmd = HistoryCommand::new();
        let (mut ctx, _shell_ctx) = create_test_context(vec!["history".to_string(), "-c".to_string()]);
        
        // Add some history entries
        ctx.env.add_history("command1".to_string());
        ctx.env.add_history("command2".to_string());
        
        // Verify history has entries
        assert!(!ctx.env.get_history().is_empty());
        
        let result = history_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        // Verify history was cleared
        assert!(ctx.env.get_history().is_empty());
    }

    #[test]
    fn test_history_delete() {
        let history_cmd = HistoryCommand::new();
        let (mut ctx, _shell_ctx) = create_test_context(vec!["history".to_string(), "-d".to_string(), "2".to_string()]);
        
        // Add some history entries
        ctx.env.add_history("command1".to_string());
        ctx.env.add_history("command2".to_string());
        ctx.env.add_history("command3".to_string());
        
        let result = history_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        // Verify the second command was deleted
        let history = ctx.env.get_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0], "command1");
        assert_eq!(history[1], "command3");
    }

    #[test]
    fn test_history_delete_invalid_offset() {
        let history_cmd = HistoryCommand::new();
        let (mut ctx, _shell_ctx) = create_test_context(vec!["history".to_string(), "-d".to_string(), "0".to_string()]);
        
        let result = history_cmd.invoke(&mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_history_search() {
        let mut shell_ctx = ShellContext::new();
        shell_ctx.add_history("ls -l".to_string());
        shell_ctx.add_history("cd /tmp".to_string());
        shell_ctx.add_history("ls -a".to_string());
        
        let context = Context::new(
            vec!["history".to_string()],
            &mut shell_ctx,
            Stream::new(StreamType::Byte),
            Stream::new(StreamType::Text),
            Stream::new(StreamType::Byte),
        ).unwrap();

        let results = HistoryCommand::search_history("ls", &context);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].1, "ls -l");
        assert_eq!(results[1].1, "ls -a");
    }

    #[test]
    fn test_history_stats() {
        let mut shell_ctx = ShellContext::new();
        shell_ctx.add_history("ls -l".to_string());
        shell_ctx.add_history("cd /tmp".to_string());
        shell_ctx.add_history("ls -a".to_string());
        shell_ctx.add_history("ls".to_string());
        
        let context = Context::new(
            vec!["history".to_string()],
            &mut shell_ctx,
            Stream::new(StreamType::Byte),
            Stream::new(StreamType::Text),
            Stream::new(StreamType::Byte),
        ).unwrap();

        let stats = HistoryCommand::get_history_stats(&context);
        assert_eq!(stats.total_commands, 4);
        assert_eq!(stats.unique_commands, 4);
        
        // ls should be the most common command (appears 3 times)
        assert!(!stats.most_common_commands.is_empty());
        assert_eq!(stats.most_common_commands[0].0, "ls");
        assert_eq!(stats.most_common_commands[0].1, 3);
    }

    #[test]
    fn test_get_last_command() {
        let mut shell_ctx = ShellContext::new();
        shell_ctx.add_history("first command".to_string());
        shell_ctx.add_history("last command".to_string());
        
        let context = Context::new(
            vec!["history".to_string()],
            &mut shell_ctx,
            Stream::new(StreamType::Byte),
            Stream::new(StreamType::Text),
            Stream::new(StreamType::Byte),
        ).unwrap();

        let last_cmd = HistoryCommand::get_last_command(&context);
        assert_eq!(last_cmd, Some("last command".to_string()));
    }
} 
