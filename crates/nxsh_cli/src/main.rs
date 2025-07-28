use clap::{Parser};

/// Simple NexusShell CLI wrapper.
#[derive(Parser, Debug)]
#[command(author, version, about = "NexusShell command-line interface", long_about = None)]
struct Cli {
    /// Command to execute instead of launching the interactive shell.
    #[arg()]
    command: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut context = nxsh_core::context::ShellContext::new();
    let mut exec = nxsh_core::executor::Executor::new(&mut context);

    if let Some(cmd) = cli.command {
        exec.run(&cmd)?;
    } else {
        // Start interactive TUI
        nxsh_ui::run_tui(&mut context, &mut exec)?;
    }

    Ok(())
} 