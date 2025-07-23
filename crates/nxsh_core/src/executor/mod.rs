use anyhow::Result;
use std::io::{self, Write};
use crate::context::ShellContext;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use crate::mir::{Program, BasicBlock, Instruction};
use crate::job;

#[cfg(unix)]
const CLOEXEC: i32 = libc::O_CLOEXEC;

#[cfg(unix)]
fn open_redirect(file: &str, append: bool) -> io::Result<std::fs::File> {
    let mut opts = OpenOptions::new();
    opts.write(true);
    if append {
        opts.append(true);
    } else {
        opts.create(true).truncate(true);
    }
    // SAFETY: we just set custom flags
    opts.custom_flags(CLOEXEC).open(file)
}

#[cfg(windows)]
fn open_redirect(file: &str, append: bool) -> io::Result<std::fs::File> {
    let mut opts = OpenOptions::new();
    opts.write(true);
    if append {
        opts.append(true);
    } else {
        opts.create(true).truncate(true);
    }
    opts.open(file)
}

/// Executor walks the AST and performs I/O redirections and process spawning.
pub struct Executor<'ctx> {
    context: &'ctx mut ShellContext,
}

impl<'ctx> Executor<'ctx> {
    pub fn new(context: &'ctx mut ShellContext) -> Self {
        job::init();
        Self { context }
    }

    /// Run a raw command string. (Parser integration will be added later.)
    pub fn run(&mut self, command: &str) -> Result<()> {
        // For now, simply echo the command to demonstrate control flow.
        println!("Executing: {}", command);
        Ok(())
    }

    /// Execute an AST node (placeholder for future parser integration)
    pub fn execute(&mut self, node: &nxsh_parser::ast::AstNode) -> anyhow::Result<()> {
        use nxsh_parser::ast::AstNode as N;
        match node {
            N::Command(cmd) => self.spawn_command(cmd),
            N::Pipeline(stages) => self.execute_pipeline(stages),
            _ => {
                println!("Unsupported AST node type");
                Ok(())
            }
        }
    }

    fn spawn_command(&self, cmd: &nxsh_parser::ast::Command) -> anyhow::Result<()> {
        let cmd_name = match &cmd.args.first() {
            Some(nxsh_parser::ast::Argument::Word(w)) => w,
            Some(nxsh_parser::ast::Argument::String(s)) => s,
            Some(nxsh_parser::ast::Argument::Number(n)) => &n.to_string(),
            Some(nxsh_parser::ast::Argument::Variable(v)) => &self
                .context
                .get_var(v)
                .unwrap_or("".to_string()),
            Some(nxsh_parser::ast::Argument::Array(_)) => "", // unsupported
            None => return Ok(()),
        };

        let output = std::process::Command::new(cmd_name).output()?;
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;
        Ok(())
    }

    fn execute_pipeline(&mut self, stages: &[nxsh_parser::ast::AstNode]) -> anyhow::Result<()> {
        // Placeholder implementation
        for stage in stages {
            self.execute(stage)?;
        }
        Ok(())
    }

    /// Compile and execute MIR program
    pub fn execute_mir(&mut self, program: &Program) -> Result<()> {
        for block in &program.blocks {
            self.execute_block(block)?;
        }
        Ok(())
    }

    fn execute_block(&mut self, block: &BasicBlock) -> Result<()> {
        for instruction in &block.instrs {
            self.execute_instruction(instruction)?;
        }
        Ok(())
    }

    fn execute_instruction(&mut self, instruction: &Instruction) -> Result<()> {
        match instruction {
            Instruction::Add { dst: _, lhs: _, rhs: _ } => {
                // Placeholder for arithmetic operations
                Ok(())
            }
            Instruction::Sub { dst: _, lhs: _, rhs: _ } => {
                // Placeholder for arithmetic operations
                Ok(())
            }
            Instruction::Mul { dst: _, lhs: _, rhs: _ } => {
                // Placeholder for arithmetic operations
                Ok(())
            }
            Instruction::Div { dst: _, lhs: _, rhs: _ } => {
                // Placeholder for arithmetic operations
                Ok(())
            }
            Instruction::ConstInt { id: _, value: _ } => {
                // Placeholder for constant loading
                Ok(())
            }
        }
    }
} 