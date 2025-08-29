//! NexusShell Hardware Abstraction Layer (HAL)
//!
//! This crate provides a unified abstraction layer over platform-specific
//! system calls and operations. It minimizes unsafe code usage and isolates
//! platform-specific functionality.
//!
//! The HAL is designed to:
//! - Minimize unsafe code by containing it in well-defined boundaries
//! - Provide platform-agnostic APIs for shell operations
//! - Enable easy testing through abstraction
//! - Support multiple platforms (Unix, Linux, macOS, Windows, FreeBSD)

pub mod command;
pub mod completion;
pub mod error;
pub mod fast_completion;
pub mod fs;
pub mod fs_enhanced;
pub mod memory;
pub mod network;
pub mod pipe;
pub mod platform;
pub mod process;
pub mod process_enhanced;
pub mod seccomp;
pub mod time;
pub mod time_enhanced;

pub use error::{HalError, HalResult};

/// Platform detection and capabilities
pub use platform::{detect_platform, Capabilities, Platform};

pub use command::{Command, CommandResult};
/// Re-export commonly used types
pub use fs::{DirectoryHandle, FileHandle, FileMetadata, FileSystem};
pub use memory::{MemoryInfo, MemoryManager};
pub use network::NetworkManager;
pub use pipe::{PipeHandle, PipeManager};
pub use process::{ProcessHandle, ProcessInfo, ProcessManager};
pub use time::TimeManager;

/// Initialize the HAL with platform-specific optimizations
pub fn initialize() -> HalResult<()> {
    platform::initialize_platform()?;
    Ok(())
}

/// Shutdown the HAL and cleanup resources
pub fn shutdown() -> HalResult<()> {
    platform::cleanup_platform()?;
    Ok(())
}
