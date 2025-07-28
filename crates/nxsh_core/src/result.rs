//! Result types and extensions for NexusShell
//!
//! This module provides enhanced result types and utility traits
//! for working with shell operations and error handling.

use crate::error::{ShellError, ErrorKind, SourceLocation};
use std::result;

/// Standard result type for all NexusShell operations
pub type ShellResult<T> = result::Result<T, ShellError>;

/// Extension trait for Result types to add shell-specific functionality
pub trait ShellResultExt<T> {
    /// Add context to an error result
    fn with_context(self, key: impl Into<String>, value: impl Into<String>) -> ShellResult<T>;
    
    /// Add multiple context entries to an error result
    fn with_contexts(self, contexts: std::collections::HashMap<String, String>) -> ShellResult<T>;
    
    /// Add source location to an error result
    fn with_location(self, location: SourceLocation) -> ShellResult<T>;
    
    /// Chain this result with additional error information
    fn with_inner_error(self, inner: ShellError) -> ShellResult<T>;
    
    /// Convert to a different error kind while preserving context
    fn map_error_kind(self, kind: ErrorKind) -> ShellResult<T>;
    
    /// Add recovery suggestions to the error
    fn with_suggestion(self, suggestion: impl Into<String>) -> ShellResult<T>;
    
    /// Execute a closure if this is an error, useful for logging
    fn inspect_error<F>(self, f: F) -> ShellResult<T>
    where
        F: FnOnce(&ShellError);
    
    /// Convert certain error types to Ok with a default value
    fn ok_or_default_on_error<F>(self, default_fn: F, recoverable_kinds: &[ErrorKind]) -> ShellResult<T>
    where
        F: FnOnce() -> T;
    
    /// Retry operation on recoverable errors
    fn retry_on_recoverable<F>(self, retry_fn: F, max_retries: usize) -> ShellResult<T>
    where
        F: Fn() -> ShellResult<T>;
}

impl<T> ShellResultExt<T> for ShellResult<T> {
    fn with_context(self, key: impl Into<String>, value: impl Into<String>) -> ShellResult<T> {
        self.map_err(|e| e.with_context(key, value))
    }
    
    fn with_contexts(self, contexts: std::collections::HashMap<String, String>) -> ShellResult<T> {
        self.map_err(|e| e.with_contexts(contexts))
    }
    
    fn with_location(self, location: SourceLocation) -> ShellResult<T> {
        self.map_err(|e| e.with_location(location))
    }
    
    fn with_inner_error(self, inner: ShellError) -> ShellResult<T> {
        self.map_err(|e| e.with_inner(inner))
    }
    
    fn map_error_kind(self, kind: ErrorKind) -> ShellResult<T> {
        self.map_err(|e| ShellError::new(kind, e.message).with_contexts(e.context))
    }
    
    fn with_suggestion(self, suggestion: impl Into<String>) -> ShellResult<T> {
        self.with_context("suggestion", suggestion)
    }
    
    fn inspect_error<F>(self, f: F) -> ShellResult<T>
    where
        F: FnOnce(&ShellError),
    {
        if let Err(ref e) = self {
            f(e);
        }
        self
    }
    
    fn ok_or_default_on_error<F>(self, default_fn: F, recoverable_kinds: &[ErrorKind]) -> ShellResult<T>
    where
        F: FnOnce() -> T,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => {
                if recoverable_kinds.iter().any(|kind| error.contains_kind(kind)) {
                    Ok(default_fn())
                } else {
                    Err(error)
                }
            }
        }
    }
    
    fn retry_on_recoverable<F>(self, retry_fn: F, max_retries: usize) -> ShellResult<T>
    where
        F: Fn() -> ShellResult<T>,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => {
                if error.is_recoverable() && max_retries > 0 {
                    // Wait a bit before retrying
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    retry_fn().retry_on_recoverable(retry_fn, max_retries - 1)
                } else {
                    Err(error)
                }
            }
        }
    }
}

/// Extension trait for Option types to convert to ShellResult
pub trait OptionExt<T> {
    /// Convert Option to ShellResult with a custom error
    fn ok_or_shell_error(self, error: ShellError) -> ShellResult<T>;
    
    /// Convert Option to ShellResult with a runtime error
    fn ok_or_not_found(self, item: &str) -> ShellResult<T>;
    
    /// Convert Option to ShellResult with a variable not found error
    fn ok_or_variable_not_found(self, variable: &str) -> ShellResult<T>;
    
    /// Convert Option to ShellResult with a command not found error
    fn ok_or_command_not_found(self, command: &str) -> ShellResult<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_shell_error(self, error: ShellError) -> ShellResult<T> {
        self.ok_or(error)
    }
    
    fn ok_or_not_found(self, item: &str) -> ShellResult<T> {
        self.ok_or_else(|| {
            ShellError::new(
                ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::FileNotFound),
                format!("Item '{}' not found", item),
            )
            .with_context("item", item.to_string())
        })
    }
    
    fn ok_or_variable_not_found(self, variable: &str) -> ShellResult<T> {
        self.ok_or_else(|| {
            ShellError::new(
                ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::VariableNotFound),
                format!("Variable '{}' not found", variable),
            )
            .with_context("variable", variable.to_string())
        })
    }
    
    fn ok_or_command_not_found(self, command: &str) -> ShellResult<T> {
        self.ok_or_else(|| ShellError::command_not_found(command))
    }
}

/// Macro for creating shell errors with location information
#[macro_export]
macro_rules! shell_error {
    ($kind:expr, $msg:expr) => {
        $crate::error::ShellError::new($kind, $msg)
    };
    ($kind:expr, $msg:expr, $($key:expr => $value:expr),+) => {
        {
            let mut error = $crate::error::ShellError::new($kind, $msg);
            $(
                error = error.with_context($key, $value);
            )+
            error
        }
    };
}

/// Macro for creating shell errors with automatic location
#[macro_export]
macro_rules! shell_error_here {
    ($kind:expr, $msg:expr) => {
        $crate::error::ShellError::new($kind, $msg)
            .with_location($crate::error::SourceLocation::new(line!(), column!()))
    };
    ($kind:expr, $msg:expr, $($key:expr => $value:expr),+) => {
        {
            let mut error = $crate::error::ShellError::new($kind, $msg)
                .with_location($crate::error::SourceLocation::new(line!(), column!()));
            $(
                error = error.with_context($key, $value);
            )+
            error
        }
    };
}

/// Macro for early return with shell error
#[macro_export]
macro_rules! shell_bail {
    ($kind:expr, $msg:expr) => {
        return Err($crate::shell_error!($kind, $msg));
    };
    ($kind:expr, $msg:expr, $($key:expr => $value:expr),+) => {
        return Err($crate::shell_error!($kind, $msg, $($key => $value),+));
    };
}

/// Macro for ensuring a condition or returning a shell error
#[macro_export]
macro_rules! shell_ensure {
    ($cond:expr, $kind:expr, $msg:expr) => {
        if !$cond {
            $crate::shell_bail!($kind, $msg);
        }
    };
    ($cond:expr, $kind:expr, $msg:expr, $($key:expr => $value:expr),+) => {
        if !$cond {
            $crate::shell_bail!($kind, $msg, $($key => $value),+);
        }
    };
}

/// Utility functions for common result operations
pub mod utils {
    use super::*;
    
    /// Collect results, stopping at the first error
    pub fn collect_results<T, I>(iter: I) -> ShellResult<Vec<T>>
    where
        I: IntoIterator<Item = ShellResult<T>>,
    {
        iter.into_iter().collect()
    }
    
    /// Collect results, collecting all errors
    pub fn collect_results_all_errors<T, I>(iter: I) -> result::Result<Vec<T>, Vec<ShellError>>
    where
        I: IntoIterator<Item = ShellResult<T>>,
    {
        let mut successes = Vec::new();
        let mut errors = Vec::new();
        
        for result in iter {
            match result {
                Ok(value) => successes.push(value),
                Err(error) => errors.push(error),
            }
        }
        
        if errors.is_empty() {
            Ok(successes)
        } else {
            Err(errors)
        }
    }
    
    /// Try multiple operations, returning the first success
    pub fn try_multiple<T, F>(operations: Vec<F>) -> ShellResult<T>
    where
        F: FnOnce() -> ShellResult<T>,
    {
        let mut last_error = None;
        
        for op in operations {
            match op() {
                Ok(value) => return Ok(value),
                Err(error) => last_error = Some(error),
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            ShellError::internal_error("No operations provided to try_multiple")
        }))
    }
    
    /// Execute operations in parallel and collect results
    #[cfg(feature = "async")]
    pub async fn parallel_results<T, F, Fut>(operations: Vec<F>) -> ShellResult<Vec<T>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = ShellResult<T>>,
        T: Send + 'static,
    {
        use futures::future::join_all;
        
        let futures: Vec<_> = operations.into_iter().map(|op| op()).collect();
        let results = join_all(futures).await;
        
        results.into_iter().collect()
    }
    
    /// Timeout wrapper for shell operations
    pub fn with_timeout<T, F>(operation: F, timeout: std::time::Duration) -> ShellResult<T>
    where
        F: FnOnce() -> ShellResult<T> + Send + 'static,
        T: Send + 'static,
    {
        use std::sync::mpsc;
        use std::thread;
        
        let (tx, rx) = mpsc::channel();
        
        thread::spawn(move || {
            let result = operation();
            let _ = tx.send(result);
        });
        
        match rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(_) => Err(ShellError::new(
                ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::Timeout),
                format!("Operation timed out after {:?}", timeout),
            )),
        }
    }
} 