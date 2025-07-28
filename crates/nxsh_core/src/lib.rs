//! NexusShell Core Library
//!
//! This is the core library for NexusShell, providing the fundamental
//! components for shell operations, including parsing, execution,
//! job control, and system integration.

// Re-export commonly used types and functions
pub use error::{ShellError, ErrorKind, ShellResult};
pub use context::{ShellContext, Context};
pub use job::{Job, JobManager, JobStatus};
pub use stream::{Stream, StreamType, StreamData};
pub use metrics::{MetricsSystem, MetricsConfig};
pub use logging::LoggingSystem;
pub use executor::{Builtin, Executor, ExecutionResult};

// Public modules
pub mod error;
pub mod context;
pub mod executor;
pub mod job;
pub mod stream;
pub mod result;
pub mod logging;
pub mod metrics;
pub mod encryption;
pub mod crash_handler;
pub mod updater;
pub mod network_security;
pub mod i18n;

/// Initialize the NexusShell core runtime
pub fn initialize() -> ShellResult<()> {
    nxsh_hal::initialize()?;
    tracing::info!("NexusShell core initialized");
    Ok(())
}

/// Shutdown the NexusShell core runtime
pub fn shutdown() -> ShellResult<()> {
    nxsh_hal::shutdown()?;
    tracing::info!("NexusShell core shutdown");
    Ok(())
} 