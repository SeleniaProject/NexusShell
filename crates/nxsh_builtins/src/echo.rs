//! `echo` builtin command - display text
//!
//! This module implements the echo builtin command with support for
//! escape sequences, formatting options, and proper output handling.

use crate::common::{i18n::*, logging::*};
use std::io::Write;
use std::collections::HashMap;
use nxsh_core::{Builtin, Context, ExecutionResult, ShellResult, ShellError, ErrorKind, StreamData};
use nxsh_core::error::{RuntimeErrorKind, IoErrorKind};

/// The `echo` builtin command implementation
pub struct EchoCommand;

impl Builtin for EchoCommand {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn synopsis(&self) -> &'static str {
        "Display text"
    }

    fn description(&self) -> &'static str {
        "Display the given text to standard output. Supports escape sequences when -e is used."
    }

    fn usage(&self) -> &'static str {
        "echo [-neE] [STRING...]"
    }

    fn invoke(&self, ctx: &mut Context) -> ShellResult<ExecutionResult> {
        let mut interpret_escapes = false;
        let mut suppress_newline = false;
        let mut disable_escapes = false;
        let mut message_parts = Vec::new();

        // Parse arguments
        let mut i = 1; // Skip command name
        while i < ctx.args.len() {
            let arg = &ctx.args[i];
            
            if arg.starts_with('-') && arg.len() > 1 {
                // Parse options
                for ch in arg.chars().skip(1) {
                    match ch {
                        'n' => suppress_newline = true,
                        'e' => interpret_escapes = true,
                        'E' => disable_escapes = true,
                        _ => {
                            return Err(ShellError::new(
                                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                                format!("echo: invalid option: -{}", ch)
                            ));
                        }
                    }
                }
            } else {
                // Regular argument
                message_parts.push(arg.clone());
            }
            
            i += 1;
        }

        // If -E is specified, it overrides -e
        if disable_escapes {
            interpret_escapes = false;
        }

        // Build the output message
        let message = if message_parts.is_empty() {
            String::new()
        } else {
            message_parts.join(" ")
        };

        // Process escape sequences if enabled
        let processed_message = if interpret_escapes {
            self.process_escape_sequences(&message)?
        } else {
            message
        };

        // Write to output stream
        let mut output = processed_message;
        if !suppress_newline {
            output.push('\n');
        }

        // Write to stdout stream
        ctx.stdout.write(StreamData::Text(output))
            .map_err(|e| ShellError::new(ErrorKind::IoError(IoErrorKind::FileWriteError), format!("Failed to write output: {}", e)))?;

        Ok(ExecutionResult::success(0))
    }
}

impl EchoCommand {
    /// Create a new echo command instance
    pub fn new() -> Self {
        Self
    }

    /// Process escape sequences in the input string
    fn process_escape_sequences(&self, input: &str) -> ShellResult<String> {
        let mut result = String::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        'a' => {
                            result.push('\x07'); // Alert (bell)
                            chars.next();
                        }
                        'b' => {
                            result.push('\x08'); // Backspace
                            chars.next();
                        }
                        'c' => {
                            // Suppress further output (like -n but stronger)
                            return Ok(result);
                        }
                        'e' | 'E' => {
                            result.push('\x1b'); // Escape
                            chars.next();
                        }
                        'f' => {
                            result.push('\x0c'); // Form feed
                            chars.next();
                        }
                        'n' => {
                            result.push('\n'); // Newline
                            chars.next();
                        }
                        'r' => {
                            result.push('\r'); // Carriage return
                            chars.next();
                        }
                        't' => {
                            result.push('\t'); // Tab
                            chars.next();
                        }
                        'v' => {
                            result.push('\x0b'); // Vertical tab
                            chars.next();
                        }
                        '\\' => {
                            result.push('\\'); // Literal backslash
                            chars.next();
                        }
                        '0' => {
                            // Octal escape sequence
                            chars.next(); // consume the '0'
                            let octal_str = self.collect_octal_digits(&mut chars);
                            if let Ok(octal_value) = u8::from_str_radix(&octal_str, 8) {
                                if octal_value == 0 {
                                    // Null character - terminate string
                                    return Ok(result);
                                }
                                result.push(octal_value as char);
                            } else {
                                // Invalid octal sequence, treat as literal
                                result.push('\\');
                                result.push('0');
                                result.push_str(&octal_str);
                            }
                        }
                        'x' => {
                            // Hexadecimal escape sequence
                            chars.next(); // consume the 'x'
                            let hex_str = self.collect_hex_digits(&mut chars);
                            if let Ok(hex_value) = u8::from_str_radix(&hex_str, 16) {
                                result.push(hex_value as char);
                            } else {
                                // Invalid hex sequence, treat as literal
                                result.push('\\');
                                result.push('x');
                                result.push_str(&hex_str);
                            }
                        }
                        _ => {
                            // Unknown escape sequence, treat literally
                            result.push('\\');
                            result.push(next_ch);
                            chars.next();
                        }
                    }
                } else {
                    // Backslash at end of string
                    result.push('\\');
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    /// Collect up to 3 octal digits
    fn collect_octal_digits(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut octal = String::new();
        
        for _ in 0..3 {
            if let Some(&ch) = chars.peek() {
                if ch.is_ascii_digit() && ch <= '7' {
                    octal.push(ch);
                    chars.next();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        octal
    }

    /// Collect up to 2 hexadecimal digits
    fn collect_hex_digits(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut hex = String::new();
        
        for _ in 0..2 {
            if let Some(&ch) = chars.peek() {
                if ch.is_ascii_hexdigit() {
                    hex.push(ch);
                    chars.next();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        hex
    }
}

impl Default for EchoCommand {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create an echo command
pub fn echo_cli(args: &[String], ctx: &mut nxsh_core::context::ShellContext) -> ShellResult<()> {
    use nxsh_core::stream::{Stream, StreamType};
    
    let mut context = Context::new(
        args.to_vec(),
        ctx,
        Stream::new(StreamType::Byte),
        Stream::new(StreamType::Text),
        Stream::new(StreamType::Byte),
    )?;

    let echo_cmd = EchoCommand::new();
    let result = echo_cmd.invoke(&mut context)?;
    
    // Output the result to stdout
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
        Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound), format!("echo failed with exit code {}", result.exit_code)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nxsh_core::context::ShellContext;
    use nxsh_core::stream::{Stream, StreamType};

    fn create_test_context(args: Vec<String>) -> Context {
        let mut shell_ctx = ShellContext::new();
        Context::new(
            args,
            &mut shell_ctx,
            Stream::new(StreamType::Byte),
            Stream::new(StreamType::Text),
            Stream::new(StreamType::Byte),
        ).unwrap()
    }

    #[test]
    fn test_echo_simple() {
        let echo_cmd = EchoCommand::new();
        let mut ctx = create_test_context(vec!["echo".to_string(), "hello".to_string(), "world".to_string()]);
        
        let result = echo_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        let output = ctx.stdout.collect().unwrap();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].to_string().unwrap(), "hello world\n");
    }

    #[test]
    fn test_echo_no_newline() {
        let echo_cmd = EchoCommand::new();
        let mut ctx = create_test_context(vec!["echo".to_string(), "-n".to_string(), "hello".to_string()]);
        
        let result = echo_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        let output = ctx.stdout.collect().unwrap();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].to_string().unwrap(), "hello");
    }

    #[test]
    fn test_echo_escape_sequences() {
        let echo_cmd = EchoCommand::new();
        let mut ctx = create_test_context(vec!["echo".to_string(), "-e".to_string(), "hello\\nworld".to_string()]);
        
        let result = echo_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        let output = ctx.stdout.collect().unwrap();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].to_string().unwrap(), "hello\nworld\n");
    }

    #[test]
    fn test_echo_escape_tab() {
        let echo_cmd = EchoCommand::new();
        let mut ctx = create_test_context(vec!["echo".to_string(), "-e".to_string(), "hello\\tworld".to_string()]);
        
        let result = echo_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        let output = ctx.stdout.collect().unwrap();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].to_string().unwrap(), "hello\tworld\n");
    }

    #[test]
    fn test_echo_octal_escape() {
        let echo_cmd = EchoCommand::new();
        let mut ctx = create_test_context(vec!["echo".to_string(), "-e".to_string(), "\\101".to_string()]);
        
        let result = echo_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        let output = ctx.stdout.collect().unwrap();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].to_string().unwrap(), "A\n"); // \101 is 'A' in octal
    }

    #[test]
    fn test_echo_hex_escape() {
        let echo_cmd = EchoCommand::new();
        let mut ctx = create_test_context(vec!["echo".to_string(), "-e".to_string(), "\\x41".to_string()]);
        
        let result = echo_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        let output = ctx.stdout.collect().unwrap();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].to_string().unwrap(), "A\n"); // \x41 is 'A' in hex
    }

    #[test]
    fn test_echo_suppress_further_output() {
        let echo_cmd = EchoCommand::new();
        let mut ctx = create_test_context(vec!["echo".to_string(), "-e".to_string(), "hello\\cworld".to_string()]);
        
        let result = echo_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        let output = ctx.stdout.collect().unwrap();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].to_string().unwrap(), "hello"); // \c suppresses further output
    }

    #[test]
    fn test_echo_empty() {
        let echo_cmd = EchoCommand::new();
        let mut ctx = create_test_context(vec!["echo".to_string()]);
        
        let result = echo_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        let output = ctx.stdout.collect().unwrap();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].to_string().unwrap(), "\n");
    }

    #[test]
    fn test_echo_disable_escapes() {
        let echo_cmd = EchoCommand::new();
        let mut ctx = create_test_context(vec!["echo".to_string(), "-E".to_string(), "hello\\nworld".to_string()]);
        
        let result = echo_cmd.invoke(&mut ctx).unwrap();
        assert!(result.is_success());
        
        let output = ctx.stdout.collect().unwrap();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].to_string().unwrap(), "hello\\nworld\n"); // Literal backslash-n
    }
} 
