//! `echo` builtin command - display text
//!
//! This module implements the echo builtin command with support for
//! escape sequences, formatting options, and proper output handling.

use std::io::Write;
use nxsh_core::{Builtin, ShellContext, ExecutionResult, ShellResult, ShellError, ErrorKind};
use nxsh_core::error::RuntimeErrorKind;

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

    fn affects_shell_state(&self) -> bool {
        false // echo doesn't modify shell state
    }

    fn help(&self) -> &'static str {
        "Display text. Use 'echo --help' for detailed usage information."
    }

    fn execute(&self, ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
        let mut interpret_escapes = false;
        let mut suppress_newline = false;
        let mut disable_escapes = false;
        let mut message_parts = Vec::new();

        // Parse arguments
        let mut i = 0; // Start from 0 since args doesn't include command name
        while i < args.len() {
            let arg = &args[i];
            
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
                                format!("echo: invalid option: -{ch}")
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

        // Process message parts
        for (part_idx, part) in message_parts.iter().enumerate() {
            if part_idx > 0 {
                write!(ctx.stdout, " ")?;
            }

            if interpret_escapes && !disable_escapes {
                let (processed, stop) = self.process_escape_sequences(part)?;
                write!(ctx.stdout, "{processed}")?;
                if stop {
                    // Suppress further output including trailing newline
                    suppress_newline = true;
                    break;
                }
            } else {
                write!(ctx.stdout, "{part}")?;
            }
        }

        if !suppress_newline {
            writeln!(ctx.stdout)?;
        }

        Ok(ExecutionResult::success(0))
    }
}

impl EchoCommand {
    /// Create a new echo command instance
    pub fn new() -> Self {
        EchoCommand
    }

    /// Process escape sequences in the input string
    fn process_escape_sequences(&self, input: &str) -> ShellResult<(String, bool)> {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
    // 'stop_output' は以前の設計からの名残で未使用。早期 return で表現されるため不要。

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
                            return Ok((result, true));
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
                        '0'..='7' => {
                            // Octal escape sequence (support both \0NN and \NNN forms)
                            // Do not consume next_ch yet; collect up to 3 octal digits including it
                            let mut octal = String::new();
                            // Collect up to 3 octal digits
                            for _ in 0..3 {
                                if let Some(&d) = chars.peek() {
                                    if ('0'..='7').contains(&d) {
                                        octal.push(d);
                                        chars.next();
                                    } else { break; }
                                } else { break; }
                            }
                            if let Ok(octal_value) = u8::from_str_radix(&octal, 8) {
                                if octal_value == 0 {
                                    // Null character - terminate string
                                    return Ok((result, true));
                                }
                                result.push(octal_value as char);
                            } else {
                                // Invalid octal sequence, treat as literal
                                result.push('\\');
                                result.push_str(&octal);
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

        Ok((result, false))
    }

    // collect_octal_digits was removed; octal handling is done inline in process_escape_sequences.

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

/// CLI wrapper for the echo command
pub fn echo_cli(args: &[String]) -> anyhow::Result<()> {
    let echo_cmd = EchoCommand::new();
    let mut ctx = ShellContext::new();
    let result = echo_cmd.execute(&mut ctx, args)?;
    if result.is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Echo command failed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nxsh_core::context::ShellContext;
    

    fn create_test_context() -> ShellContext { ShellContext::new() }

    #[test]
    fn test_echo_simple() {
        let echo_cmd = EchoCommand::new();
    let mut ctx = create_test_context();
    ctx.enable_stdout_capture();
        
        let result = echo_cmd.execute(&mut ctx, &["hello".into(), "world".into()]).unwrap();
        assert!(result.is_success());
    assert_eq!(ctx.stdout_captured(), "hello world\n");
    }

    #[test]
    fn test_echo_no_newline() {
        let echo_cmd = EchoCommand::new();
    let mut ctx = create_test_context();
    ctx.enable_stdout_capture();
        
        let result = echo_cmd.execute(&mut ctx, &["-n".into(), "hello".into()]).unwrap();
        assert!(result.is_success());
    assert_eq!(ctx.stdout_captured(), "hello");
    }

    #[test]
    fn test_echo_escape_sequences() {
        let echo_cmd = EchoCommand::new();
    let mut ctx = create_test_context();
    ctx.enable_stdout_capture();

    let result = echo_cmd.execute(&mut ctx, &["-e".into(), "hello\\nworld".into()]).unwrap();
    assert!(result.is_success());
    assert_eq!(ctx.stdout_captured(), "hello\nworld\n");
    }

    #[test]
    fn test_echo_escape_tab() {
        let echo_cmd = EchoCommand::new();
    let mut ctx = create_test_context();
    ctx.enable_stdout_capture();

    let result = echo_cmd.execute(&mut ctx, &["-e".into(), "hello\\tworld".into()]).unwrap();
    assert!(result.is_success());
    assert_eq!(ctx.stdout_captured(), "hello\tworld\n");
    }

    #[test]
    fn test_echo_octal_escape() {
        let echo_cmd = EchoCommand::new();
    let mut ctx = create_test_context();
    ctx.enable_stdout_capture();

    let result = echo_cmd.execute(&mut ctx, &["-e".into(), "\\101".into()]).unwrap();
    assert!(result.is_success());
    assert_eq!(ctx.stdout_captured(), "A\n");
    }

    #[test]
    fn test_echo_hex_escape() {
        let echo_cmd = EchoCommand::new();
    let mut ctx = create_test_context();
    ctx.enable_stdout_capture();

    let result = echo_cmd.execute(&mut ctx, &["-e".into(), "\\x41".into()]).unwrap();
    assert!(result.is_success());
    assert_eq!(ctx.stdout_captured(), "A\n");
    }

    #[test]
    fn test_echo_suppress_further_output() {
        let echo_cmd = EchoCommand::new();
    let mut ctx = create_test_context();
    ctx.enable_stdout_capture();

    let result = echo_cmd.execute(&mut ctx, &["-e".into(), "hello\\cworld".into()]).unwrap();
    assert!(result.is_success());
    // \c 以降は出力しない（改行も抑止）
    assert_eq!(ctx.stdout_captured(), "hello");
    }

    #[test]
    fn test_echo_empty() {
        let echo_cmd = EchoCommand::new();
    let mut ctx = create_test_context();
    ctx.enable_stdout_capture();

    let result = echo_cmd.execute(&mut ctx, &[]).unwrap();
    assert!(result.is_success());
    // 引数なしは改行のみ
    assert_eq!(ctx.stdout_captured(), "\n");
    }

    #[test]
    fn test_echo_disable_escapes() {
        let echo_cmd = EchoCommand::new();
    let mut ctx = create_test_context();
    ctx.enable_stdout_capture();

    let result = echo_cmd.execute(&mut ctx, &["-E".into(), "hello\\nworld".into()]).unwrap();
    assert!(result.is_success());
    // -E ではエスケープは解釈しない
    assert_eq!(ctx.stdout_captured(), "hello\\nworld\n");
    }
}

/// CLI execution wrapper for echo command
pub fn execute(args: &[String], _context: &crate::common::BuiltinContext) -> crate::common::BuiltinResult<i32> {
    let echo_cmd = EchoCommand::new();
    let mut shell_context = ShellContext::new();
    
    // Convert BuiltinContext to ShellContext if needed
    // For now, just execute with the args
    match echo_cmd.execute(&mut shell_context, args) {
        Ok(execution_result) => {
            if execution_result.is_success() {
                Ok(0)
            } else {
                Ok(execution_result.exit_code)
            }
        }
        Err(_) => Ok(1),
    }
} 
