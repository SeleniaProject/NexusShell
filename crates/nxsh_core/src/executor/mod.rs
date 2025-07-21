use anyhow::Result;

use crate::context::ShellContext;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::io;
use crate::mir::{Program, BasicBlock, Instruction};

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
    unsafe { opts.custom_flags(CLOEXEC).open(file) }
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
        Self { context }
    }

    /// Run a raw command string. (Parser integration will be added later.)
    pub fn run(&mut self, command: &str) -> Result<()> {
        // For now, simply echo the command to demonstrate control flow.
        println!("Executed command: {command}");
        let _ = &self.context; // keep context in scope
        Ok(())
    }

    /// Walk the AST and execute commands / pipelines.
    pub fn execute(&mut self, node: &nxsh_parser::ast::AstNode) -> anyhow::Result<()> {
        use nxsh_parser::ast::AstNode as N;
        match node {
            N::Program(nodes) | N::Sequence(nodes) => {
                for n in nodes {
                    self.execute(n)?;
                }
            }
            N::Command(cmd) => {
                self.spawn_command(cmd)?;
            }
            N::Pipeline(stages) => {
                self.execute_pipeline(stages)?;
            }
            N::RedirectOut { file, append } => {
                let f = open_redirect(file, *append)?;
                let stream = crate::stream::Stream::from_file(f);
                // Save to context for future executor revision
                println!("Opened redirect to {} (append={})", file, append);
                drop(stream);
            }
            _ => {
                // For unimplemented nodes, no-op.
            }
        }
        Ok(())
    }

    fn spawn_command(&self, cmd: &nxsh_parser::ast::Command) -> anyhow::Result<()> {
        use std::process::Command as SysCmd;
        let output = SysCmd::new(&cmd.name).args(cmd.args.iter().map(|a| match a {
            nxsh_parser::ast::Argument::Word(w) => w,
            nxsh_parser::ast::Argument::String(s) => s,
            nxsh_parser::ast::Argument::Number(n) => &n.to_string(),
            nxsh_parser::ast::Argument::Variable(v) => self
                .context
                .get_var(v)
                .as_deref()
                .unwrap_or("")
                .into(),
            nxsh_parser::ast::Argument::Array(_) => "", // unsupported
        })).output()?;
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;
        Ok(())
    }

    fn execute_pipeline(&mut self, stages: &[nxsh_parser::ast::AstNode]) -> anyhow::Result<()> {
        println!("Pipeline execution stub for {} stages", stages.len());
        Ok(())
    }

    /// Example function to generate MIR from simple math expression.
    pub fn generate_mir_example(&self) -> Program {
        let mut prog = Program { blocks: vec![] };
        let mut block = BasicBlock { id: 0, instrs: vec![] };
        block.instrs.push(Instruction::ConstInt { id: 1, value: 2 });
        block.instrs.push(Instruction::ConstInt { id: 2, value: 3 });
        block.instrs.push(Instruction::Add { dst: 3, lhs: 1, rhs: 2 });
        prog.blocks.push(block);
        prog
    }
} 