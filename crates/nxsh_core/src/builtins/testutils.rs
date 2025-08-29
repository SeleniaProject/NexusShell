use crate::context::ShellContext;
use crate::error::ShellResult;
use crate::executor::{Builtin, ExecutionResult};

pub struct ArgDumpBuiltin;

impl Builtin for ArgDumpBuiltin {
    fn execute(
        &self,
        _context: &mut ShellContext,
        args: &[String],
    ) -> ShellResult<ExecutionResult> {
        // Format: first line count, then each arg on its own line (verbatim)
        let mut out = format!("count={}\n", args.len());
        for a in args {
            out.push_str(a);
            out.push('\n');
        }
        Ok(ExecutionResult {
            exit_code: 0,
            stdout: out,
            stderr: String::new(),
            execution_time: 0,
            strategy: crate::executor::ExecutionStrategy::DirectInterpreter,
            metrics: Default::default(),
        })
    }
    fn name(&self) -> &'static str {
        "__argdump"
    }
    fn help(&self) -> &'static str {
        "Test helper: dumps argument count and values"
    }
    fn synopsis(&self) -> &'static str {
        "__argdump [args...]"
    }
    fn description(&self) -> &'static str {
        "Internal test builtin for verifying argument splitting behavior."
    }
    fn usage(&self) -> &'static str {
        "__argdump"
    }
    fn affects_shell_state(&self) -> bool {
        false
    }
}

pub struct EchoBuiltin;

impl EchoBuiltin {
    fn process_escape_sequences(&self, input: &str) -> (String, bool) {
        let mut out = String::new();
        let mut it = input.chars().peekable();
        while let Some(ch) = it.next() {
            if ch != '\\' {
                out.push(ch);
                continue;
            }
            if let Some(&nx) = it.peek() {
                match nx {
                    'a' => {
                        out.push('\x07');
                        it.next();
                    }
                    'b' => {
                        out.push('\x08');
                        it.next();
                    }
                    'c' => {
                        return (out, true);
                    }
                    'e' | 'E' => {
                        out.push('\x1b');
                        it.next();
                    }
                    'f' => {
                        out.push('\x0c');
                        it.next();
                    }
                    'n' => {
                        out.push('\n');
                        it.next();
                    }
                    'r' => {
                        out.push('\r');
                        it.next();
                    }
                    't' => {
                        out.push('\t');
                        it.next();
                    }
                    'v' => {
                        out.push('\x0b');
                        it.next();
                    }
                    '\\' => {
                        out.push('\\');
                        it.next();
                    }
                    '0'..='7' => {
                        // up to 3 octal digits
                        let mut oct = String::new();
                        for _ in 0..3 {
                            if let Some(&d) = it.peek() {
                                if ('0'..='7').contains(&d) {
                                    oct.push(d);
                                    it.next();
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        if let Ok(v) = u8::from_str_radix(&oct, 8) {
                            if v == 0 {
                                return (out, true);
                            }
                            out.push(v as char);
                        } else {
                            out.push('\\');
                            out.push_str(&oct);
                        }
                    }
                    'x' => {
                        // hex: up to 2 digits
                        it.next();
                        let mut hex = String::new();
                        for _ in 0..2 {
                            if let Some(&h) = it.peek() {
                                if h.is_ascii_hexdigit() {
                                    hex.push(h);
                                    it.next();
                                } else {
                                    break;
                                }
                            }
                        }
                        if let Ok(v) = u8::from_str_radix(&hex, 16) {
                            out.push(v as char);
                        } else {
                            out.push('\\');
                            out.push('x');
                            out.push_str(&hex);
                        }
                    }
                    _ => {
                        out.push('\\');
                        out.push(nx);
                        it.next();
                    }
                }
            } else {
                out.push('\\');
            }
        }
        (out, false)
    }
}

impl Builtin for EchoBuiltin {
    fn execute(
        &self,
        _context: &mut ShellContext,
        args: &[String],
    ) -> ShellResult<ExecutionResult> {
        // Bash-compatible options: -n (no newline), -e (enable escapes), -E (disable escapes)
        let mut interpret_escapes = false;
        let mut suppress_newline = false;
        let mut disable_escapes = false;
        let mut parts: Vec<String> = Vec::new();

        for arg in args {
            if arg.starts_with('-') && arg.len() > 1 {
                for ch in arg.chars().skip(1) {
                    match ch {
                        'n' => suppress_newline = true,
                        'e' => interpret_escapes = true,
                        'E' => disable_escapes = true,
                        _ => {
                            // Keep POSIX-y behavior: unknown options treated as text
                            parts.push(arg.clone());
                            // Stop treating subsequent as options
                            // Push remaining original options if any and continue normal
                            continue;
                        }
                    }
                }
            } else {
                parts.push(arg.clone());
            }
        }

        if disable_escapes {
            interpret_escapes = false;
        }

        let mut out = String::new();
        for (i, p) in parts.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            if interpret_escapes && !disable_escapes {
                let (seg, stop) = self.process_escape_sequences(p);
                out.push_str(&seg);
                if stop {
                    suppress_newline = true;
                    break;
                }
            } else {
                out.push_str(p);
            }
        }
        if !suppress_newline {
            out.push('\n');
        }
        Ok(ExecutionResult {
            exit_code: 0,
            stdout: out,
            stderr: String::new(),
            execution_time: 0,
            strategy: crate::executor::ExecutionStrategy::DirectInterpreter,
            metrics: Default::default(),
        })
    }
    fn name(&self) -> &'static str {
        "echo"
    }
    fn help(&self) -> &'static str {
        "Echo arguments to standard output"
    }
    fn synopsis(&self) -> &'static str {
        "echo [args...]"
    }
    fn description(&self) -> &'static str {
        "Writes its arguments to standard output separated by spaces. Supports -n/-e/-E."
    }
    fn usage(&self) -> &'static str {
        "echo [-neE] [STRING...]"
    }
    fn affects_shell_state(&self) -> bool {
        false
    }
}
