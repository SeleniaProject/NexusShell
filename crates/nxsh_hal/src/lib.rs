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

pub mod fs;
pub mod fs_enhanced;
pub mod process;
pub mod process_enhanced;
pub mod pipe;
pub mod seccomp;
pub mod platform;
pub mod error;
pub mod memory;
pub mod time;
pub mod time_enhanced;
pub mod network;
pub mod completion;
pub mod fast_completion;

pub use error::{HalError, HalResult};

/// Platform detection and capabilities
pub use platform::{Platform, Capabilities, detect_platform};

/// Re-export commonly used types
pub use fs::{FileSystem, FileHandle, DirectoryHandle, FileMetadata};
pub use process::{ProcessManager, ProcessHandle, ProcessInfo};
pub use pipe::{PipeManager, PipeHandle};
pub use memory::{MemoryManager, MemoryInfo};
pub use time::TimeManager;
pub use network::{NetworkManager};

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