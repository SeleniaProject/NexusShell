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
    let mut exec = nxsh_core::executor::Executor::new();

    if let Some(cmd) = cli.command {
        // TODO: Implement command execution
        println!("Command execution not yet implemented: {}", cmd);
    } else {
        // Start interactive TUI  
        tokio::runtime::Runtime::new()?.block_on(async {
            nxsh_ui::run_tui(&mut context, &mut exec).await
        })?;
    }

    Ok(())
} 