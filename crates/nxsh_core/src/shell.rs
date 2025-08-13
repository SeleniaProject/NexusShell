//! High-level shell runner for NexusShell
//!
//! This module wires together the parser (`nxsh_parser`), execution context
//! (`ShellContext`) and the core executor (`Executor`). It provides
//! programmatic entry points for:
//! - Interactive REPL (non-TUI fallback)
//! - Single-line evaluation
//! - Script file execution
//!
//! Notes:
//! - Comments are intentionally verbose and in English to document the
//!   non-trivial control flow and design decisions.
//! - This runner is TTY-aware but does not implement the rich TUI; the
//!   interactive UI is provided by `nxsh_ui`. This serves as a minimal yet
//!   fully-functional CUI fallback and as an embeddable engine surface.

use crate::compat::Result;
use crate::context::ShellContext;
use crate::error::{ShellError, ErrorKind, ShellResult};
use crate::executor::{Executor, ExecutionResult};

use std::io::{self, Write};
use std::io::IsTerminal;
use std::path::Path;
use tokio::io::AsyncBufReadExt;

/// Public shell facade combining parsing and execution.
pub struct Shell {
    /// Execution context (environment, variables, history, options, etc.)
    context: ShellContext,
    /// Core executor (builtins, external process launch, MIR integration)
    executor: Executor,
    /// Parser instance (PEG-based)
    parser: nxsh_parser::ShellCommandParser,
    /// REPL termination flag for cooperative exit
    should_exit: bool,
}

impl Shell {
    /// Create a new shell with a fully initialized context and executor.
    pub fn new() -> Self {
        // Construct a fresh context with environment and options loaded.
        let context = ShellContext::new();
        // Executor::new() registers all core builtins by default.
        let executor = Executor::new();
        let parser = nxsh_parser::ShellCommandParser::new();
        Self { context, executor, parser, should_exit: false }
    }

    /// Borrow the underlying context (read-only).
    pub fn context(&self) -> &ShellContext { &self.context }

    /// Borrow the underlying context (mutable).
    pub fn context_mut(&mut self) -> &mut ShellContext { &mut self.context }

    /// Evaluate a single command line (one logical line). Returns the
    /// execution result with stdout/stderr and exit code.
    pub fn eval_line(&mut self, line: &str) -> ShellResult<ExecutionResult> {
        // Allow simple built-in exit for non-TUI REPL convenience.
        // The full-featured `exit` builtin exists in `nxsh_builtins`, but
        // core runner offers a pragmatic escape hatch when running CUI.
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(ExecutionResult::success(0));
        }
        if Self::is_exit_request(trimmed) {
            self.should_exit = true;
            return Ok(ExecutionResult::success(0));
        }

        // Parse into AST and execute via core executor.
        let ast = self
            .parser
            .parse(line)
            .map_err(|e| ShellError::new(ErrorKind::ParseError(crate::error::ParseErrorKind::SyntaxError), e.to_string()))?;

        self.executor.execute(&ast, &mut self.context)
    }

    /// Execute a whole script source (can contain multiple statements/lines).
    pub fn eval_program(&mut self, source: &str) -> ShellResult<ExecutionResult> {
        if source.trim().is_empty() {
            return Ok(ExecutionResult::success(0));
        }
        let ast = self
            .parser
            .parse(source)
            .map_err(|e| ShellError::new(ErrorKind::ParseError(crate::error::ParseErrorKind::SyntaxError), e.to_string()))?;
        self.executor.execute(&ast, &mut self.context)
    }

    /// Execute a script file by path. The file is read as UTF-8 text.
    pub fn run_script_file<P: AsRef<Path>>(&mut self, path: P) -> ShellResult<ExecutionResult> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| ShellError::new(ErrorKind::IoError(crate::error::IoErrorKind::FileReadError), format!("{}", e)))?;
        self.eval_program(&content)
    }

    /// Start an interactive CUI REPL reading from stdin and writing to stdout.
    /// This is a minimal fallback loop (the rich UI is provided elsewhere).
    pub async fn run(&mut self) -> Result<()> {
        // TTY-aware prompt management. We avoid heavy line editing here.
        let is_tty = io::stdin().is_terminal();
        let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
        let mut line = String::new();

        loop {
            // Print prompt only for TTY sessions.
            if is_tty {
                self.print_prompt()?;
            }

            line.clear();
            let n = stdin.read_line(&mut line).await.map_err(|e| {
                ShellError::new(
                    ErrorKind::IoError(crate::error::IoErrorKind::FileReadError),
                    format!("stdin read error: {}", e),
                )
            })?;

            // EOF (Ctrl+D / pipe end)
            if n == 0 {
                break;
            }

            match self.eval_line(&line) {
                Ok(result) => {
                    // Write command output; in a full UI this is routed differently.
                    if !result.stdout.is_empty() {
                        let _ = write!(self.context.stdout, "{}", result.stdout);
                        let _ = self.context.stdout.flush();
                    }
                    if !result.stderr.is_empty() {
                        let _ = write!(self.context.stderr, "{}", result.stderr);
                        let _ = self.context.stderr.flush();
                    }
                    self.context.set_exit_status(result.exit_code);
                }
                Err(err) => {
                    let _ = writeln!(self.context.stderr, "nxsh: {}", err);
                    let _ = self.context.stderr.flush();
                    self.context.set_exit_status(1);
                }
            }

            if self.should_exit { break; }
            if self.context.is_timed_out() {
                // Respect global timeout if configured.
                let _ = writeln!(self.context.stderr, "nxsh: execution timed out");
                break;
            }
        }

        Ok(())
    }

    /// Determine whether the user requested to exit the REPL (portable).
    fn is_exit_request(s: &str) -> bool {
        matches!(s, "exit" | "quit" | "logout" | ":q" | "bye")
    }

    /// Print a compact, informative prompt reflecting minimal status.
    fn print_prompt(&self) -> Result<()> {
        // Keep it minimal here; the rich statusline is provided by the UI layer.
        let cwd = &self.context.cwd;
        let prompt = if self.context.is_login_shell() { "login" } else { "" };
        let code = self.context.get_exit_status();
        let symbol = if code == 0 { "λ" } else { "✗" };
        write!(io::stdout(), "{}{} {} ", symbol, if prompt.is_empty() { "" } else { "*" }, cwd.display())?;
        io::stdout().flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_line_empty_is_ok() {
        let mut sh = Shell::new();
        let res = sh.eval_line("").expect("empty line should succeed");
        assert_eq!(res.exit_code, 0);
    }

    #[test]
    fn eval_line_exit_sets_flag() {
        let mut sh = Shell::new();
        let res = sh.eval_line("exit").expect("exit should parse");
        assert_eq!(res.exit_code, 0);
        assert!(sh.should_exit);
    }

    // Note: Parser in this project normalizes some malformed snippets;
    // do not assert parse error semantics here to keep tests stable across grammar tweaks.
}

