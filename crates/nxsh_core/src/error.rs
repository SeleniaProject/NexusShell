//! Comprehensive error handling system for NexusShell
//!
//! This module provides structured error types for all shell operations,
//! enabling precise error reporting, recovery, and debugging capabilities.

use std::fmt;
use std::path::PathBuf;
use std::collections::HashMap;
use std::time::SystemTimeError;

// Additional external error types frequently bubbled up via `?` in subsystems
use std::fmt::Error as FmtError;

// serde derives appear in multiple modules; we avoid unconditional new deps by only
// referencing serde_json when the crate already uses it (always true in core modules)
// so pulling it here does not introduce a new feature edge.
use serde_json::Error as SerdeJsonError;

// tokio semaphore AcquireError (always available because tokio is an unconditional dependency in core)
use tokio::sync::AcquireError;

/// Result type for all NexusShell operations
pub type ShellResult<T> = Result<T, ShellError>;

/// Main error type for all NexusShell operations
#[derive(Debug, Clone)]
pub struct ShellError {
    pub kind: ErrorKind,
    pub message: String,
    // Box the SourceLocation to keep ShellError size small (clippy::result_large_err)
    pub source_location: Option<Box<SourceLocation>>,
    // Box the context map (can grow); most errors have few/no entries
    pub context: Box<HashMap<String, String>>,
    pub inner: Option<Box<ShellError>>,
}

/// Categories of errors that can occur in NexusShell
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    // Parsing errors
    ParseError(ParseErrorKind),
    
    // Runtime errors
    RuntimeError(RuntimeErrorKind),
    
    // I/O errors
    IoError(IoErrorKind),
    
    // Security errors
    SecurityError(SecurityErrorKind),
    
    // System errors
    SystemError(SystemErrorKind),
    
    // Plugin errors
    PluginError(PluginErrorKind),
    
    // Configuration errors
    ConfigError(ConfigErrorKind),
    
    // Network errors
    NetworkError(NetworkErrorKind),
    
    // Cryptography errors
    CryptoError(CryptoErrorKind),
    
    // Serialization errors
    SerializationError(SerializationErrorKind),
    
    // Internal errors
    InternalError(InternalErrorKind),
}

/// Parse error subcategories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    SyntaxError,
    UnexpectedToken,
    UnterminatedString,
    UnterminatedComment,
    InvalidEscape,
    InvalidNumber,
    InvalidRegex,
    UnbalancedParentheses,
    UnbalancedBraces,
    UnbalancedBrackets,
    InvalidVariable,
    InvalidFunction,
    InvalidPipeline,
    InvalidRedirection,
    InvalidGlob,
    UnsupportedFeature,
}

/// Runtime error subcategories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    CommandNotFound,
    PermissionDenied,
    FileNotFound,
    DirectoryNotFound,
    PathNotFound,
    InvalidArgument,
    TooManyArguments,
    TooFewArguments,
    VariableNotFound,
    FunctionNotFound,
    AliasNotFound,
    TypeMismatch,
    ConversionError,
    OverflowError,
    DivisionByZero,
    IndexOutOfBounds,
    KeyNotFound,
    NullPointer,
    ResourceExhausted,
    Timeout,
    Interrupted,
    Cancelled,
    DeadLock,
    PoisonedLock,
}

/// I/O error subcategories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IoErrorKind {
    FileReadError,
    FileWriteError,
    FileCreateError,
    FileDeleteError,
    DirectoryCreateError,
    DirectoryDeleteError,
    PermissionError,
    NotFound,
    AlreadyExists,
    InvalidPath,
    PathTooLong,
    DiskFull,
    QuotaExceeded,
    DeviceError,
    BrokenPipe,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    Other,
    TimedOut,
    WouldBlock,
    UnexpectedEof,
    InvalidData,
    WriteZero,
}

/// Security error subcategories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityErrorKind {
    AccessDenied,
    AuthenticationFailed,
    AuthorizationFailed,
    CertificateError,
    EncryptionError,
    DecryptionError,
    SignatureError,
    HashError,
    TokenExpired,
    InvalidToken,
    PrivilegeEscalation,
    SandboxViolation,
    PolicyViolation,
    UnsafeOperation,
}

/// System error subcategories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemErrorKind {
    OutOfMemory,
    OutOfHandles,
    OutOfSpace,
    ProcessError,
    ThreadError,
    SignalError,
    LibraryError,
    DriverError,
    HardwareError,
    KernelError,
    SystemCallError,
    UnsupportedOperation,
    PlatformError,
    LockError,
}

/// Plugin error subcategories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginErrorKind {
    LoadError,
    InitError,
    ExecutionError,
    VersionMismatch,
    DependencyError,
    ConfigurationError,
    CompatibilityError,
    SignatureError,
    SandboxError,
    CommunicationError,
}

/// Configuration error subcategories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigErrorKind {
    InvalidFormat,
    MissingField,
    InvalidValue,
    ValidationError,
    SchemaError,
    VersionError,
    EnvironmentError,
    ProfileError,
}

/// Network error subcategories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkErrorKind {
    ConnectionError,
    ResolveError,
    TimeoutError,
    ProtocolError,
    CertificateError,
    AuthenticationError,
    ProxyError,
    RedirectError,
    RateLimitError,
    ServiceUnavailable,
}

/// Cryptography error subcategories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoErrorKind {
    KeyGenerationFailed,
    EncryptionFailed,
    DecryptionFailed,
    HashingFailed,
    SigningFailed,
    VerificationFailed,
    InvalidKey,
    InvalidNonce,
    InvalidTag,
    InvalidCiphertext,
    UnsupportedAlgorithm,
    WeakKey,
    ExpiredKey,
    KeyDerivationFailed,
}

/// Serialization error subcategories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializationErrorKind {
    JsonError,
    YamlError,
    TomlError,
    BinaryError,
    XmlError,
    CsvError,
    InvalidFormat,
    InvalidData,
    SchemaValidationFailed,
    EncodingError,
    DecodingError,
    CompressionError,
    DecompressionError,
}

/// Internal error subcategories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalErrorKind {
    AssertionFailed,
    UnreachableCode,
    NotImplemented,
    InvalidState,
    CorruptedData,
    VersionMismatch,
    MemoryCorruption,
    StackOverflow,
    InfiniteLoop,
    DeadCode,
    LockError,
}

/// Source location information for errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: Option<PathBuf>,
    pub line: u32,
    pub column: u32,
    pub length: Option<u32>,
    pub source_text: Option<String>,
}

impl ShellError {
    /// Create a new shell error
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            source_location: None,
            context: Box::new(HashMap::new()),
            inner: None,
        }
    }

    /// Create an I/O error
    pub fn io(err: std::io::Error) -> Self {
        Self::new(
            ErrorKind::SystemError(SystemErrorKind::SystemCallError),
            format!("I/O error: {err}")
        )
    }

    /// Create a command not found error
    pub fn command_not_found(command: &str) -> Self {
        Self::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound),
            format!("Command not found: {command}")
        )
    }

    /// Create an error with source location
    pub fn with_location(mut self, location: SourceLocation) -> Self {
    self.source_location = Some(Box::new(location));
        self
    }

    /// Add context information to the error
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
    self.context.insert(key.into(), value.into());
        self
    }

    /// Add multiple context entries
    pub fn with_contexts(mut self, contexts: HashMap<String, String>) -> Self {
    self.context.extend(contexts);
        self
    }

    /// Chain this error with an inner error
    pub fn with_inner(mut self, inner: ShellError) -> Self {
        self.inner = Some(Box::new(inner));
        self
    }

    /// Get the error chain as a vector
    pub fn error_chain(&self) -> Vec<&ShellError> {
        let mut chain = vec![self];
        let mut current = self;
        while let Some(ref inner) = current.inner {
            chain.push(inner);
            current = inner;
        }
        chain
    }

    /// Check if this error or any in the chain matches the given kind
    pub fn contains_kind(&self, kind: &ErrorKind) -> bool {
        self.error_chain().iter().any(|e| &e.kind == kind)
    }

    /// Get the root cause of this error
    pub fn root_cause(&self) -> &ShellError {
        let mut current = self;
        while let Some(ref inner) = current.inner {
            current = inner;
        }
        current
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match &self.kind {
            ErrorKind::ParseError(_) => true,
            ErrorKind::RuntimeError(kind) => matches!(
                kind,
                RuntimeErrorKind::CommandNotFound
                    | RuntimeErrorKind::FileNotFound
                    | RuntimeErrorKind::DirectoryNotFound
                    | RuntimeErrorKind::InvalidArgument
                    | RuntimeErrorKind::VariableNotFound
                    | RuntimeErrorKind::FunctionNotFound
                    | RuntimeErrorKind::AliasNotFound
            ),
            ErrorKind::IoError(kind) => matches!(
                kind,
                IoErrorKind::NotFound | IoErrorKind::PermissionError | IoErrorKind::InvalidPath | IoErrorKind::AlreadyExists
            ),
            ErrorKind::SecurityError(_) => false,
            ErrorKind::SystemError(_) => false,
            ErrorKind::PluginError(_) => true,
            ErrorKind::ConfigError(_) => true,
            ErrorKind::NetworkError(_) => true,
            ErrorKind::CryptoError(_) => false,
            ErrorKind::SerializationError(_) => true,
            ErrorKind::InternalError(_) => false,
        }
    }

    /// Get suggested recovery actions
    pub fn recovery_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        match &self.kind {
            ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound) => {
                suggestions.push("Check if the command is installed and in PATH".to_string());
                suggestions.push("Use 'which <command>' to find the command location".to_string());
                suggestions.push("Install the missing command or package".to_string());
            }
            ErrorKind::RuntimeError(RuntimeErrorKind::PermissionDenied) => {
                suggestions.push("Check file permissions with 'ls -la'".to_string());
                suggestions.push("Use 'sudo' if elevated privileges are needed".to_string());
                suggestions.push("Change file permissions with 'chmod'".to_string());
            }
            ErrorKind::IoError(IoErrorKind::NotFound) => {
                suggestions.push("Verify the file or directory path is correct".to_string());
                suggestions.push("Check if the file exists with 'ls' or 'stat'".to_string());
                suggestions.push("Create the missing file or directory".to_string());
            }
            ErrorKind::ParseError(ParseErrorKind::SyntaxError) => {
                suggestions.push("Check command syntax and fix any typos".to_string());
                suggestions.push("Refer to command documentation or help".to_string());
                suggestions.push("Use shell completion to verify syntax".to_string());
            }
            _ => {
                suggestions.push("Check the error message for specific guidance".to_string());
                suggestions.push("Consult documentation or help resources".to_string());
            }
        }

        suggestions
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match &self.kind {
            ErrorKind::ParseError(_) => ErrorSeverity::Warning,
            ErrorKind::RuntimeError(kind) => match kind {
                RuntimeErrorKind::CommandNotFound
                | RuntimeErrorKind::FileNotFound
                | RuntimeErrorKind::DirectoryNotFound
                | RuntimeErrorKind::InvalidArgument => ErrorSeverity::Error,
                RuntimeErrorKind::PermissionDenied => ErrorSeverity::Error,
                RuntimeErrorKind::ResourceExhausted
                | RuntimeErrorKind::DeadLock
                | RuntimeErrorKind::PoisonedLock => ErrorSeverity::Critical,
                _ => ErrorSeverity::Error,
            },
            ErrorKind::IoError(_) => ErrorSeverity::Error,
            ErrorKind::SecurityError(_) => ErrorSeverity::Critical,
            ErrorKind::SystemError(_) => ErrorSeverity::Critical,
            ErrorKind::PluginError(_) => ErrorSeverity::Warning,
            ErrorKind::ConfigError(_) => ErrorSeverity::Warning,
            ErrorKind::NetworkError(_) => ErrorSeverity::Error,
            ErrorKind::CryptoError(_) => ErrorSeverity::Critical,
            ErrorKind::SerializationError(_) => ErrorSeverity::Error,
            ErrorKind::InternalError(_) => ErrorSeverity::Critical,
        }
    }
}

// (AcquireError mapping moved below with cfg(feature = "async"))

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
    Fatal,
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)?;

    if let Some(ref location) = self.source_location {
            if let Some(ref file) = location.file {
                write!(f, " at {}:{}:{}", file.display(), location.line, location.column)?;
            } else {
                write!(f, " at line {}, column {}", location.line, location.column)?;
            }
        }

    if !self.context.is_empty() {
            write!(f, " (")?;
            let mut first = true;
            for (key, value) in self.context.iter() {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{key}: {value}")?;
                first = false;
            }
            write!(f, ")")?;
        }

        if let Some(ref inner) = self.inner {
            write!(f, "\nCaused by: {inner}")?;
        }

        Ok(())
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::ParseError(kind) => write!(f, "Parse error: {kind:?}"),
            ErrorKind::RuntimeError(kind) => write!(f, "Runtime error: {kind:?}"),
            ErrorKind::IoError(kind) => write!(f, "I/O error: {kind:?}"),
            ErrorKind::SecurityError(kind) => write!(f, "Security error: {kind:?}"),
            ErrorKind::SystemError(kind) => write!(f, "System error: {kind:?}"),
            ErrorKind::PluginError(kind) => write!(f, "Plugin error: {kind:?}"),
            ErrorKind::ConfigError(kind) => write!(f, "Configuration error: {kind:?}"),
            ErrorKind::NetworkError(kind) => write!(f, "Network error: {kind:?}"),
            ErrorKind::CryptoError(kind) => write!(f, "Cryptography error: {kind:?}"),
            ErrorKind::SerializationError(kind) => write!(f, "Serialization error: {kind:?}"),
            ErrorKind::InternalError(kind) => write!(f, "Internal error: {kind:?}"),
        }
    }
}

impl std::error::Error for ShellError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.as_ref().map(|e| e as &dyn std::error::Error)
    }
}

impl From<std::io::Error> for ShellError {
    fn from(err: std::io::Error) -> Self {
        let kind = match err.kind() {
            std::io::ErrorKind::NotFound => ErrorKind::IoError(IoErrorKind::NotFound),
            std::io::ErrorKind::PermissionDenied => ErrorKind::IoError(IoErrorKind::PermissionError),
            std::io::ErrorKind::ConnectionRefused => ErrorKind::IoError(IoErrorKind::ConnectionRefused),
            std::io::ErrorKind::ConnectionReset => ErrorKind::IoError(IoErrorKind::ConnectionReset),
            std::io::ErrorKind::ConnectionAborted => ErrorKind::IoError(IoErrorKind::ConnectionAborted),
            std::io::ErrorKind::NotConnected => ErrorKind::IoError(IoErrorKind::ConnectionRefused),
            std::io::ErrorKind::AddrInUse => ErrorKind::IoError(IoErrorKind::AlreadyExists),
            std::io::ErrorKind::AddrNotAvailable => ErrorKind::IoError(IoErrorKind::NotFound),
            std::io::ErrorKind::BrokenPipe => ErrorKind::IoError(IoErrorKind::BrokenPipe),
            std::io::ErrorKind::AlreadyExists => ErrorKind::IoError(IoErrorKind::AlreadyExists),
            std::io::ErrorKind::WouldBlock => ErrorKind::IoError(IoErrorKind::WouldBlock),
            std::io::ErrorKind::InvalidInput => ErrorKind::IoError(IoErrorKind::InvalidData),
            std::io::ErrorKind::InvalidData => ErrorKind::IoError(IoErrorKind::InvalidData),
            std::io::ErrorKind::TimedOut => ErrorKind::IoError(IoErrorKind::TimedOut),
            std::io::ErrorKind::WriteZero => ErrorKind::IoError(IoErrorKind::WriteZero),
            std::io::ErrorKind::Interrupted => ErrorKind::RuntimeError(RuntimeErrorKind::Interrupted),
            std::io::ErrorKind::UnexpectedEof => ErrorKind::IoError(IoErrorKind::UnexpectedEof),
            _ => ErrorKind::IoError(IoErrorKind::DeviceError),
        };

        ShellError::new(kind, err.to_string())
    }
}

impl From<nxsh_hal::HalError> for ShellError {
    fn from(err: nxsh_hal::HalError) -> Self {
        match err {
            nxsh_hal::HalError::Io(io_err) => {
                let kind = ErrorKind::IoError(IoErrorKind::DeviceError);
                ShellError::new(kind, io_err.message)
                    .with_context("operation", io_err.operation)
                    .with_context("path", io_err.path.unwrap_or_else(|| "unknown".to_string()))
            }
            nxsh_hal::HalError::Process(proc_err) => {
                let kind = ErrorKind::SystemError(SystemErrorKind::ProcessError);
                ShellError::new(kind, proc_err.message)
                    .with_context("operation", proc_err.operation)
                    .with_context("pid", proc_err.pid.map(|p| p.to_string()).unwrap_or_else(|| "unknown".to_string()))
            }
            nxsh_hal::HalError::Memory(mem_err) => {
                let kind = ErrorKind::SystemError(SystemErrorKind::OutOfMemory);
                ShellError::new(kind, mem_err.message)
                    .with_context("operation", mem_err.operation)
                    .with_context("requested_size", mem_err.requested_size.map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string()))
            }
            nxsh_hal::HalError::Network(net_err) => {
                let kind = ErrorKind::NetworkError(NetworkErrorKind::ConnectionError);
                ShellError::new(kind, net_err.message)
                    .with_context("operation", net_err.operation)
                    .with_context("address", net_err.address.unwrap_or_else(|| "unknown".to_string()))
            }
            nxsh_hal::HalError::Platform(plat_err) => {
                let kind = ErrorKind::SystemError(SystemErrorKind::PlatformError);
                ShellError::new(kind, plat_err.message)
                    .with_context("platform", plat_err.platform)
                    .with_context("operation", plat_err.operation)
            }
            nxsh_hal::HalError::Security(sec_err) => {
                let kind = ErrorKind::SecurityError(SecurityErrorKind::AccessDenied);
                ShellError::new(kind, sec_err.message)
                    .with_context("operation", sec_err.operation)
                    .with_context("required_permission", sec_err.required_permission)
            }
            nxsh_hal::HalError::Resource(res_err) => {
                let kind = ErrorKind::SystemError(SystemErrorKind::OutOfSpace);
                ShellError::new(kind, res_err.message)
                    .with_context("resource_type", res_err.resource_type)
                    .with_context("limit", res_err.limit.map(|l| l.to_string()).unwrap_or_else(|| "unknown".to_string()))
            }
            nxsh_hal::HalError::Invalid(msg) => {
                let kind = ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument);
                ShellError::new(kind, msg)
            }
            nxsh_hal::HalError::Unsupported(msg) => {
                let kind = ErrorKind::SystemError(SystemErrorKind::UnsupportedOperation);
                ShellError::new(kind, msg)
            }
        }
    }
}

// Allow use of `?` with standard address parsing errors in networking code
impl From<std::net::AddrParseError> for ShellError {
    fn from(err: std::net::AddrParseError) -> Self {
        ShellError::new(
            ErrorKind::NetworkError(NetworkErrorKind::ResolveError),
            err.to_string(),
        )
    }
}

// Enable `?` on std::fmt formatting fallible operations
impl From<FmtError> for ShellError {
    fn from(err: FmtError) -> Self {
        // Map formatting failures to our existing InvalidFormat variant
        ShellError::new(
            ErrorKind::SerializationError(SerializationErrorKind::InvalidFormat),
            err.to_string(),
        )
    }
}

// Enable `?` on serde_json (parsing / serialization) without manual mapping
impl From<SerdeJsonError> for ShellError {
    fn from(err: SerdeJsonError) -> Self {
        // Heuristic: treat messages suggesting syntax structure issues as InvalidFormat, others as InvalidData
        let msg = err.to_string();
        let kind = if msg.contains("expected") || msg.contains("EOF while") || msg.contains("invalid type") {
            SerializationErrorKind::InvalidFormat
        } else {
            SerializationErrorKind::InvalidData
        };
        ShellError::new(ErrorKind::SerializationError(kind), msg)
    }
}

// SystemTimeError (duration_since) conversions
impl From<SystemTimeError> for ShellError {
    fn from(err: SystemTimeError) -> Self {
        ShellError::new(
            ErrorKind::SystemError(SystemErrorKind::SystemCallError),
            err.to_string(),
        )
    }
}

// StripPrefixError (path manipulation)
impl From<std::path::StripPrefixError> for ShellError {
    fn from(err: std::path::StripPrefixError) -> Self {
        // We don't have a dedicated path error kind; map to IoError::InvalidPath to reuse existing taxonomy
        ShellError::new(
            ErrorKind::IoError(IoErrorKind::InvalidPath),
            err.to_string(),
        )
    }
}

// AcquireError (tokio semaphore) – map into ResourceExhausted so `?` works on semaphore.acquire()
impl From<AcquireError> for ShellError {
    fn from(err: AcquireError) -> Self {
        ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::ResourceExhausted),
            err.to_string(),
        )
    }
}

#[cfg(feature = "error-rich")]
impl From<anyhow::Error> for ShellError {
    fn from(err: anyhow::Error) -> Self {
        // Extract the root cause if possible
        let root_cause = err.root_cause();
        let kind = if root_cause.downcast_ref::<std::io::Error>().is_some() {
            ErrorKind::IoError(IoErrorKind::DeviceError)
        } else {
            ErrorKind::RuntimeError(RuntimeErrorKind::ConversionError)
        };
        
        ShellError::new(kind, err.to_string())
    }
}

// Convenience constructors for common error types
impl ShellError {

    pub fn file_not_found(path: &str) -> Self {
        ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::FileNotFound),
            format!("File '{path}' not found"),
        )
        .with_context("path", path.to_string())
    }

    pub fn permission_denied(resource: &str) -> Self {
        ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::PermissionDenied),
            format!("Permission denied accessing '{resource}'"),
        )
        .with_context("resource", resource.to_string())
    }

    pub fn syntax_error(message: &str, location: SourceLocation) -> Self {
        ShellError::new(
            ErrorKind::ParseError(ParseErrorKind::SyntaxError),
            message.to_string(),
        )
        .with_location(location)
    }

    pub fn type_mismatch(expected: &str, actual: &str) -> Self {
        ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::TypeMismatch),
            format!("Type mismatch: expected {expected}, got {actual}"),
        )
        .with_context("expected", expected.to_string())
        .with_context("actual", actual.to_string())
    }

    pub fn internal_error(message: &str) -> Self {
        ShellError::new(
            ErrorKind::InternalError(InternalErrorKind::InvalidState),
            message.to_string(),
        )
    }
}

impl SourceLocation {
    pub fn new(line: u32, column: u32) -> Self {
        Self {
            file: None,
            line,
            column,
            length: None,
            source_text: None,
        }
    }

    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    pub fn with_length(mut self, length: u32) -> Self {
        self.length = Some(length);
        self
    }

    pub fn with_source_text(mut self, text: String) -> Self {
        self.source_text = Some(text);
        self
    }
} 