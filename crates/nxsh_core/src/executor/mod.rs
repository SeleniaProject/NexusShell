use anyhow::Result;

use crate::context::ShellContext;

/// Executor walks the AST and performs I/O redirections and process spawning.
pub struct Executor<'ctx> {
    context: &'ctx mut ShellContext,
}

impl<'ctx> Executor<'ctx> {
    pub fn new(context: &'ctx mut ShellContext) -> Self {
        Self { context }
    }

    /// Run a raw command string. (Parser integration will be added later.)
    pub fn run(&mut self, command: &str) -> Result<()> {
        // For now, simply echo the command to demonstrate control flow.
        println!("Executed command: {command}");
        let _ = &self.context; // keep context in scope
        Ok(())
    }
} 