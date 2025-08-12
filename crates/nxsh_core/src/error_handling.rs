use crate::compat::{Result, Context, Error};
use std::collections::HashMap;

/// Advanced error handling system for NexusShell scripts
#[derive(Debug, Clone)]
pub struct ErrorHandlingSystem {
    error_handlers: HashMap<String, ErrorHandler>,
    global_error_handler: Option<ErrorHandler>,
    error_stack: Vec<ErrorInfo>,
    retry_policies: HashMap<String, RetryPolicy>,
}

impl ErrorHandlingSystem {
    pub fn new() -> Self {
        let mut system = Self {
            error_handlers: HashMap::new(),
            global_error_handler: None,
            error_stack: Vec::new(),
            retry_policies: HashMap::new(),
        };
        
        system.register_default_policies();
        system
    }

    /// Register an error handler for a specific error type
    pub fn register_handler(&mut self, error_type: String, handler: ErrorHandler) {
        self.error_handlers.insert(error_type, handler);
    }

    /// Set a global error handler
    pub fn set_global_handler(&mut self, handler: ErrorHandler) {
        self.global_error_handler = Some(handler);
    }

    /// Handle an error with registered handlers
    pub fn handle_error(&mut self, error: ErrorInfo) -> Result<ErrorResult> {
        // Add to error stack
        self.error_stack.push(error.clone());
        
        // Try specific error type handler first
        if let Some(handler) = self.error_handlers.get(&error.error_type).cloned() {
            return self.execute_handler(&handler, &error);
        }
        
        // Try global handler
        if let Some(handler) = &self.global_error_handler.clone() {
            return self.execute_handler(handler, &error);
        }
        
        // No handler found, return unhandled
        Ok(ErrorResult::Unhandled)
    }

    /// Execute an error handler
    fn execute_handler(&mut self, handler: &ErrorHandler, error: &ErrorInfo) -> Result<ErrorResult> {
        match &handler.action {
            ErrorAction::Ignore => Ok(ErrorResult::Ignored),
            
            ErrorAction::Log { level } => {
                self.log_error(error, level);
                Ok(ErrorResult::Handled)
            },
            
            ErrorAction::Retry { max_attempts, delay_ms } => {
                let retry_key = format!("{}:{}", error.error_type, error.source_location);
                
                if let Some(policy) = self.retry_policies.get_mut(&retry_key) {
                    
                    // Check if we've already reached max attempts
                    if policy.attempt_count >= *max_attempts {
                        // Max attempts reached
                        return Ok(ErrorResult::MaxRetriesExceeded);
                    }
                    
                    policy.attempt_count += 1;
                    
                    // Check again after incrementing - if we've now reached max, don't retry
                    if policy.attempt_count >= *max_attempts {
                        return Ok(ErrorResult::MaxRetriesExceeded);
                    }
                    
                    // Apply delay if specified
                    if *delay_ms > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(*delay_ms));
                    }
                    
                    return Ok(ErrorResult::Retry);
                } else {
                    // First attempt
                    self.retry_policies.insert(retry_key, RetryPolicy {
                        attempt_count: 1,
                        max_attempts: *max_attempts,
                    });
                    
                    // If max_attempts is 1, we shouldn't retry at all
                    if *max_attempts <= 1 {
                        return Ok(ErrorResult::MaxRetriesExceeded);
                    }
                    
                    if *delay_ms > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(*delay_ms));
                    }
                    
                    return Ok(ErrorResult::Retry);
                }
            },
            
            ErrorAction::Fallback { fallback_value } => {
                Ok(ErrorResult::Fallback(fallback_value.clone()))
            },
            
            ErrorAction::Terminate => {
                Ok(ErrorResult::Terminate)
            },
            
            ErrorAction::Custom { callback } => {
                callback(error)
            },
        }
    }

    /// Try-catch block execution
    pub fn try_catch<T, F>(&mut self, try_block: F, catch_handlers: Vec<CatchHandler>) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        match try_block() {
            Ok(value) => Ok(value),
            Err(err) => {
                let error_info = ErrorInfo {
                    error_type: self.classify_error(&err),
                    message: err.to_string(),
                    source_location: "unknown".to_string(), // Would be filled by parser
                    timestamp: std::time::SystemTime::now(),
                    severity: ErrorSeverity::Medium,
                    context: HashMap::new(),
                };

                // Try each catch handler
                for handler in &catch_handlers {
                    if self.matches_pattern(&error_info, &handler.pattern) {
                        return match &handler.action {
                            CatchAction::Return(value) => {
                                // This is a placeholder - actual implementation would need proper type handling
    Err(crate::anyhow!("Catch handler executed"))
                            },
                            CatchAction::Execute(callback) => {
                                callback(&error_info)?;
                                Err(err)
                            },
                            CatchAction::Rethrow => Err(err),
                        };
                    }
                }

                Err(err)
            }
        }
    }

    /// Finally block execution
    pub fn try_finally<T, F, G>(&mut self, try_block: F, finally_block: G) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
        G: FnOnce(),
    {
        let result = try_block();
        finally_block();
        result
    }

    /// Exception propagation with context
    pub fn propagate_with_context(&mut self, error: Error, context: HashMap<String, String>) -> Error {
        let error_info = ErrorInfo {
            error_type: self.classify_error(&error),
            message: error.to_string(),
            source_location: "unknown".to_string(),
            timestamp: std::time::SystemTime::now(),
            severity: ErrorSeverity::Medium,
            context,
        };

        self.error_stack.push(error_info);
    // In minimal mode `Error` is `ShellError`; we can't call `.context()` on it directly.
    // Just wrap by creating a new internal error embedding original message.
    crate::compat::anyhow(format!("Error propagated with additional context: {}", error))
    }

    /// Get error statistics
    pub fn get_error_statistics(&self) -> ErrorStatistics {
        let mut stats = ErrorStatistics {
            total_errors: self.error_stack.len(),
            errors_by_type: HashMap::new(),
            errors_by_severity: HashMap::new(),
            most_recent_error: self.error_stack.last().cloned(),
        };

        for error in &self.error_stack {
            *stats.errors_by_type.entry(error.error_type.clone()).or_insert(0) += 1;
            *stats.errors_by_severity.entry(error.severity.clone()).or_insert(0) += 1;
        }

        stats
    }

    /// Clear error history
    pub fn clear_error_history(&mut self) {
        self.error_stack.clear();
        self.retry_policies.clear();
    }

    /// Custom assertion with error handling
    pub fn assert_with_handler<F>(&mut self, condition: bool, message: &str, handler: F) -> Result<()>
    where
        F: FnOnce(&ErrorInfo) -> Result<ErrorResult>,
    {
        if !condition {
            let error_info = ErrorInfo {
                error_type: "AssertionError".to_string(),
                message: message.to_string(),
                source_location: "assertion".to_string(),
                timestamp: std::time::SystemTime::now(),
                severity: ErrorSeverity::High,
                context: HashMap::new(),
            };

            let result = handler(&error_info)?;
            match result {
                ErrorResult::Handled | ErrorResult::Ignored => Ok(()),
                ErrorResult::Fallback(_) => Ok(()),
                _ => Err(crate::anyhow!("Assertion failed: {}", message)),
            }
        } else {
            Ok(())
        }
    }

    /// Register default retry policies
    fn register_default_policies(&mut self) {
        // Network errors
        self.register_handler("NetworkError".to_string(), ErrorHandler {
            action: ErrorAction::Retry { max_attempts: 3, delay_ms: 1000 },
        });

        // File system errors
        self.register_handler("FileSystemError".to_string(), ErrorHandler {
            action: ErrorAction::Log { level: LogLevel::Error },
        });

        // Permission errors
        self.register_handler("PermissionError".to_string(), ErrorHandler {
            action: ErrorAction::Fallback { 
                fallback_value: "Permission denied - using fallback".to_string(),
            },
        });
    }

    fn classify_error(&self, error: &crate::compat::Error) -> String {
        let error_str = error.to_string().to_lowercase();
        
        if error_str.contains("network") || error_str.contains("connection") {
            "NetworkError".to_string()
        } else if error_str.contains("file") || error_str.contains("directory") || error_str.contains("io") {
            "FileSystemError".to_string()
        } else if error_str.contains("permission") || error_str.contains("access") {
            "PermissionError".to_string()
        } else if error_str.contains("parse") || error_str.contains("syntax") {
            "ParseError".to_string()
        } else if error_str.contains("timeout") {
            "TimeoutError".to_string()
        } else {
            "GenericError".to_string()
        }
    }

    fn matches_pattern(&self, error: &ErrorInfo, pattern: &ErrorPattern) -> bool {
        match pattern {
            ErrorPattern::Type(error_type) => error.error_type == *error_type,
            ErrorPattern::Message(message_pattern) => error.message.contains(message_pattern),
            ErrorPattern::Severity(severity) => error.severity == *severity,
            ErrorPattern::Any => true,
        }
    }

    fn log_error(&self, error: &ErrorInfo, level: &LogLevel) {
        let timestamp = error.timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        match level {
            LogLevel::Debug => println!("[DEBUG:{}] {}: {}", timestamp, error.error_type, error.message),
            LogLevel::Info => println!("[INFO:{}] {}: {}", timestamp, error.error_type, error.message),
            LogLevel::Warn => println!("[WARN:{}] {}: {}", timestamp, error.error_type, error.message),
            LogLevel::Error => eprintln!("[ERROR:{}] {}: {}", timestamp, error.error_type, error.message),
            LogLevel::Fatal => eprintln!("[FATAL:{}] {}: {}", timestamp, error.error_type, error.message),
        }
    }
}

/// Error handler configuration
#[derive(Debug, Clone)]
pub struct ErrorHandler {
    pub action: ErrorAction,
}

/// Actions to take when handling errors
#[derive(Debug, Clone)]
pub enum ErrorAction {
    Ignore,
    Log { level: LogLevel },
    Retry { max_attempts: usize, delay_ms: u64 },
    Fallback { fallback_value: String },
    Terminate,
    Custom { callback: fn(&ErrorInfo) -> Result<ErrorResult> },
}

/// Error handling results
#[derive(Debug, Clone)]
pub enum ErrorResult {
    Handled,
    Ignored,
    Retry,
    MaxRetriesExceeded,
    Fallback(String),
    Terminate,
    Unhandled,
}

/// Error information
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub error_type: String,
    pub message: String,
    pub source_location: String,
    pub timestamp: std::time::SystemTime,
    pub severity: ErrorSeverity,
    pub context: HashMap<String, String>,
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Logging levels
#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

/// Retry policy for specific errors
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub attempt_count: usize,
    pub max_attempts: usize,
}

/// Catch handler for try-catch blocks
#[derive(Debug, Clone)]
pub struct CatchHandler {
    pub pattern: ErrorPattern,
    pub action: CatchAction,
}

/// Patterns for matching errors in catch blocks
#[derive(Debug, Clone)]
pub enum ErrorPattern {
    Type(String),
    Message(String),
    Severity(ErrorSeverity),
    Any,
}

/// Actions for catch handlers
#[derive(Debug, Clone)]
pub enum CatchAction {
    Return(String), // Simplified - would need proper type system
    Execute(fn(&ErrorInfo) -> Result<()>),
    Rethrow,
}

/// Error statistics
#[derive(Debug, Clone)]
pub struct ErrorStatistics {
    pub total_errors: usize,
    pub errors_by_type: HashMap<String, usize>,
    pub errors_by_severity: HashMap<ErrorSeverity, usize>,
    pub most_recent_error: Option<ErrorInfo>,
}

/// Macro system for error handling
pub mod macros {
    /// try! macro implementation
    #[macro_export]
    macro_rules! nxsh_try {
        ($expr:expr) => {
            match $expr {
                Ok(val) => val,
                Err(err) => {
                    // Handle error through error system
                    return Err(err);
                }
            }
        };
    }

    /// assert! macro with custom error handling
    #[macro_export]
    macro_rules! nxsh_assert {
        ($condition:expr, $message:expr) => {
            if !$condition {
                return Err(crate::anyhow!("Assertion failed: {}", $message));
            }
        };
    }

    /// expect! macro with context
    #[macro_export]
    macro_rules! nxsh_expect {
        ($expr:expr, $message:expr) => {
            match $expr {
                Ok(val) => val,
                Err(err) => {
                    return Err(err.context($message));
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_handler_registration() {
        let mut system = ErrorHandlingSystem::new();
        
        system.register_handler("TestError".to_string(), ErrorHandler {
            action: ErrorAction::Log { level: LogLevel::Error },
        });
        
        let error = ErrorInfo {
            error_type: "TestError".to_string(),
            message: "Test error message".to_string(),
            source_location: "test".to_string(),
            timestamp: std::time::SystemTime::now(),
            severity: ErrorSeverity::Medium,
            context: HashMap::new(),
        };
        
        let result = system.handle_error(error).unwrap();
        assert!(matches!(result, ErrorResult::Handled));
    }

    #[test]
    fn test_retry_policy() {
        let mut system = ErrorHandlingSystem::new();
        
        system.register_handler("RetryError".to_string(), ErrorHandler {
            action: ErrorAction::Retry { max_attempts: 2, delay_ms: 0 },
        });
        
        let error = ErrorInfo {
            error_type: "RetryError".to_string(),
            message: "Retry test".to_string(),
            source_location: "test:1".to_string(),
            timestamp: std::time::SystemTime::now(),
            severity: ErrorSeverity::Medium,
            context: HashMap::new(),
        };
        
        // First attempt should retry
        let result1 = system.handle_error(error.clone()).unwrap();
        println!("First attempt result: {:?}", result1);
        assert!(matches!(result1, ErrorResult::Retry));
        
        // Second attempt should exceed max retries
        let result2 = system.handle_error(error).unwrap();
        println!("Second attempt result: {:?}", result2);
        assert!(matches!(result2, ErrorResult::MaxRetriesExceeded));
    }

    #[test]
    fn test_fallback_handler() {
        let mut system = ErrorHandlingSystem::new();
        
        system.register_handler("FallbackError".to_string(), ErrorHandler {
            action: ErrorAction::Fallback { 
                fallback_value: "fallback_result".to_string(),
            },
        });
        
        let error = ErrorInfo {
            error_type: "FallbackError".to_string(),
            message: "Fallback test".to_string(),
            source_location: "test".to_string(),
            timestamp: std::time::SystemTime::now(),
            severity: ErrorSeverity::Medium,
            context: HashMap::new(),
        };
        
        let result = system.handle_error(error).unwrap();
        if let ErrorResult::Fallback(value) = result {
            assert_eq!(value, "fallback_result");
        } else {
            panic!("Expected fallback result");
        }
    }

    #[test]
    fn test_error_statistics() {
        let mut system = ErrorHandlingSystem::new();
        
        let error1 = ErrorInfo {
            error_type: "TypeA".to_string(),
            message: "Error 1".to_string(),
            source_location: "test".to_string(),
            timestamp: std::time::SystemTime::now(),
            severity: ErrorSeverity::High,
            context: HashMap::new(),
        };
        
        let error2 = ErrorInfo {
            error_type: "TypeB".to_string(),
            message: "Error 2".to_string(),
            source_location: "test".to_string(),
            timestamp: std::time::SystemTime::now(),
            severity: ErrorSeverity::High,
            context: HashMap::new(),
        };
        
        system.handle_error(error1).unwrap();
        system.handle_error(error2).unwrap();
        
        let stats = system.get_error_statistics();
        assert_eq!(stats.total_errors, 2);
        assert_eq!(stats.errors_by_severity.get(&ErrorSeverity::High), Some(&2));
    }
}
