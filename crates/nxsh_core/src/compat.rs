//! Compatibility layer mimicking a subset of `anyhow` when `error-rich` feature is off.
//! Provides `Result`, `anyhow!` macro, and basic `.context()` so existing code compiles.

#[cfg(feature = "error-rich")]
pub use anyhow::{Result, Error, Context};

// When error-rich is enabled, provide both a function and a crate-root macro shim
// so call sites can use either `crate::anyhow!` (macro) or `crate::compat::anyhow(...)` (function).
#[cfg(feature = "error-rich")]
#[inline]
pub fn anyhow(msg: impl core::fmt::Display) -> anyhow::Error {
    anyhow::anyhow!(msg.to_string())
}

#[cfg(feature = "error-rich")]
#[macro_export]
macro_rules! anyhow {
    ($($tt:tt)*) => { ::anyhow::anyhow!($($tt)*) };
}

#[cfg(not(feature = "error-rich"))]
use crate::error::{ShellError, ErrorKind, InternalErrorKind};

#[cfg(not(feature = "error-rich"))]
pub type Error = ShellError;
#[cfg(not(feature = "error-rich"))]
pub type Result<T> = core::result::Result<T, ShellError>;

#[cfg(not(feature = "error-rich"))]
pub fn anyhow(msg: impl core::fmt::Display) -> ShellError {
    ShellError::new(
        ErrorKind::InternalError(InternalErrorKind::InvalidState),
        msg.to_string(),
    )
}

#[cfg(not(feature = "error-rich"))]
pub trait Context<T> {
    fn context<C: core::fmt::Display>(self, context: C) -> Result<T>;
    fn with_context<F, C>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> C,
        C: core::fmt::Display;
}

#[cfg(not(feature = "error-rich"))]
impl<T, E> Context<T> for core::result::Result<T, E>
where
    E: core::fmt::Display + Send + Sync + 'static,
{
    fn context<C: core::fmt::Display>(self, context: C) -> Result<T> {
    self.map_err(|e| anyhow(format!("{context}: {e}")))
    }
    fn with_context<F, C>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> C,
        C: core::fmt::Display,
    {
        self.map_err(|e| anyhow(format!("{}: {}", f(), e)))
    }
}

#[cfg(not(feature = "error-rich"))]
#[macro_export]
macro_rules! anyhow {
    ($($tt:tt)*) => { $crate::compat::anyhow(format!($($tt)*)) };
}
