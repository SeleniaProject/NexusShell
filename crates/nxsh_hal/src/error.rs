//! Error handling for the NexusShell HAL
//!
//! This module provides structured error types for all HAL operations,
//! enabling proper error propagation and handling throughout the shell.

use std::fmt;
use std::io;
use std::result;

/// Result type for HAL operations
pub type HalResult<T> = result::Result<T, HalError>;

/// Comprehensive error types for HAL operations
#[derive(Debug, Clone)]
pub enum HalError {
    /// I/O operation failed
    Io(IoError),
    /// Process operation failed
    Process(ProcessError),
    /// Memory operation failed
    Memory(MemoryError),
    /// Network operation failed
    Network(NetworkError),
    /// Platform-specific error
    Platform(PlatformError),
    /// Security/permission error
    Security(SecurityError),
    /// Resource exhaustion
    Resource(ResourceError),
    /// Invalid operation or state
    Invalid(String),
    /// Operation not supported on this platform
    Unsupported(String),
}

#[derive(Debug, Clone)]
pub struct IoError {
    pub operation: String,
    pub path: Option<String>,
    pub kind: io::ErrorKind,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ProcessError {
    pub operation: String,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct MemoryError {
    pub operation: String,
    pub requested_size: Option<usize>,
    pub available_size: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct NetworkError {
    pub operation: String,
    pub address: Option<String>,
    pub port: Option<u16>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct PlatformError {
    pub platform: String,
    pub operation: String,
    pub error_code: Option<i32>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct SecurityError {
    pub operation: String,
    pub required_permission: String,
    pub current_permission: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ResourceError {
    pub resource_type: String,
    pub limit: Option<u64>,
    pub current_usage: Option<u64>,
    pub message: String,
}

impl fmt::Display for HalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HalError::Io(err) => write!(f, "I/O error in {}: {}", err.operation, err.message),
            HalError::Process(err) => {
                write!(f, "Process error in {}: {}", err.operation, err.message)
            }
            HalError::Memory(err) => {
                write!(f, "Memory error in {}: {}", err.operation, err.message)
            }
            HalError::Network(err) => {
                write!(f, "Network error in {}: {}", err.operation, err.message)
            }
            HalError::Platform(err) => write!(
                f,
                "Platform error on {} in {}: {}",
                err.platform, err.operation, err.message
            ),
            HalError::Security(err) => write!(
                f,
                "Security error in {}: {} (required: {})",
                err.operation, err.message, err.required_permission
            ),
            HalError::Resource(err) => write!(
                f,
                "Resource error for {}: {}",
                err.resource_type, err.message
            ),
            HalError::Invalid(msg) => write!(f, "Invalid operation: {msg}"),
            HalError::Unsupported(msg) => write!(f, "Unsupported operation: {msg}"),
        }
    }
}

impl std::error::Error for HalError {}

impl From<io::Error> for HalError {
    fn from(err: io::Error) -> Self {
        HalError::Io(IoError {
            operation: "unknown".to_string(),
            path: None,
            kind: err.kind(),
            message: err.to_string(),
        })
    }
}

impl From<std::ffi::NulError> for HalError {
    fn from(err: std::ffi::NulError) -> Self {
        HalError::Invalid(format!("Invalid null byte in string: {err}"))
    }
}

// Helper functions for creating specific error types
impl HalError {
    pub fn io_error(operation: &str, path: Option<&str>, err: io::Error) -> Self {
        HalError::Io(IoError {
            operation: operation.to_string(),
            path: path.map(|s| s.to_string()),
            kind: err.kind(),
            message: err.to_string(),
        })
    }

    pub fn process_error(operation: &str, pid: Option<u32>, message: &str) -> Self {
        HalError::Process(ProcessError {
            operation: operation.to_string(),
            pid,
            exit_code: None,
            message: message.to_string(),
        })
    }

    pub fn memory_error(operation: &str, requested: Option<usize>, message: &str) -> Self {
        HalError::Memory(MemoryError {
            operation: operation.to_string(),
            requested_size: requested,
            available_size: None,
            message: message.to_string(),
        })
    }

    pub fn security_error(operation: &str, required_perm: &str, message: &str) -> Self {
        HalError::Security(SecurityError {
            operation: operation.to_string(),
            required_permission: required_perm.to_string(),
            current_permission: None,
            message: message.to_string(),
        })
    }

    pub fn unsupported(message: &str) -> Self {
        HalError::Unsupported(message.to_string())
    }

    pub fn invalid(message: &str) -> Self {
        HalError::Invalid(message.to_string())
    }

    pub fn network_error(
        operation: &str,
        host: Option<&str>,
        port: Option<u16>,
        message: &str,
    ) -> Self {
        HalError::Invalid(format!(
            "Network error in {operation}: {message} (host: {host:?}, port: {port:?})"
        ))
    }

    pub fn resource_error(message: &str) -> Self {
        HalError::Invalid(message.to_string())
    }

    pub fn invalid_input(message: &str) -> Self {
        HalError::Invalid(message.to_string())
    }
}
