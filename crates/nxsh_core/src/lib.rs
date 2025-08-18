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
#[cfg(feature = "metrics")]
pub use metrics::{MetricsSystem, MetricsConfig};
#[cfg(feature = "logging")]
pub use logging::LoggingSystem;
pub use executor::{Builtin, Executor, ExecutionResult};
pub use pattern_matching::{PatternMatchingEngine, PatternValue, MatchResult};
pub use namespace::{NamespaceSystem, Module, Symbol, ImportStatement};
// Removed safe crate imports - implementing custom safe wrappers instead
pub use performance::{PerformanceOptimizer, PerformanceConfig, PerformanceReport};
pub use startup::{StartupOptimizer, StartupConfig, StartupTimer, StartupReport};
pub use memory::{MemoryManager, MemoryPool, StringInterner, global_memory_manager};
pub use closures::{ClosureSystem, Closure, Function, ExecutionContext};
pub use error_handling::{ErrorHandlingSystem, ErrorHandler, ErrorInfo, ErrorResult};
#[cfg(feature = "powershell_compat")]
pub use powershell_compat::{PowerShellCompat, PowerShellObject, CommandResult};
pub use structured_logging::{StructuredLogger, LogConfig, LogFormat, RotationConfig, CommandExecutionLog, LogStats};
#[cfg(feature = "monitoring")] pub use monitoring::{MonitoringSystem, MonitoringConfig, SystemMetrics, AlertConfig, AlertLevel, Alert, DashboardData};
#[cfg(feature = "advanced_scheduler")] pub use advanced_scheduler::{AdvancedJobScheduler, ScheduledJob, JobSchedule, JobExecutionResult, JobStatistics};
#[cfg(feature = "security_auditor")] pub use security_auditor::{SecurityAuditor, SecurityAuditReport, AuditScope, AuditFinding};
// NexusShell-inspired structured data processing
pub use structured_data::{StructuredValue, PipelineData, StructuredCommand};
pub use structured_commands::*;
#[cfg(feature = "system_optimizer")] pub use system_optimizer::{SystemOptimizer, SystemOptimizationReport, OptimizationProfile};
#[cfg(feature = "performance_profiler")] pub use performance_profiler::{PerformanceProfiler, ProfilingSession, BenchmarkResult, BottleneckAnalysis};
#[cfg(feature = "documentation_system")] pub use documentation_system::{DocumentationSystem, ApiDocumentationReport, UserDocumentationReport, DeveloperDocumentationReport};
#[cfg(feature = "internationalization")] pub use internationalization::{InternationalizationSystem, LanguagePack, ValidationReport, PluralizationRule};
#[cfg(feature = "test_framework")] pub use test_framework::{TestFramework, ComprehensiveTestReport, TestSuite, PerformanceBenchmark};

// Public modules
pub mod builtins;
pub mod error;
pub mod context;
pub mod executor;
pub mod job;
pub mod stream;
pub mod result;
pub mod macros; // macro system module
#[cfg(feature = "logging")]
pub mod logging;
#[cfg(feature = "metrics")]
pub mod metrics;
pub mod encryption;
pub mod crash_handler;
pub mod updater;
pub mod network_security;
#[cfg(feature = "internationalization")]
pub mod i18n;
pub mod mir;  // MIR System - Phase 1: Basic types  // Temporarily disabled for compilation stability
pub mod structured_logging;
pub mod compat; // new compatibility layer (anyhow substitute)
#[cfg(feature = "monitoring")] pub mod monitoring;
#[cfg(feature = "advanced_scheduler")] pub mod advanced_scheduler;
#[cfg(feature = "heavy-time")]
pub mod integration_test_system;
pub mod pattern_matching; // Advanced pattern matching engine
pub mod namespace; // Namespace and module system
pub mod safe; // Safe error handling to eliminate panic! calls
pub mod performance; // Performance optimization system
pub mod startup; // Startup time optimization system
pub mod structured_data; // NexusShell-inspired structured data processing
pub mod structured_commands; // Commands for structured data
pub mod memory;
pub mod io_optimization;
pub mod shell;
pub mod locale_alias;
#[cfg(feature = "security_auditor")] pub mod security_auditor; // Security audit and compliance system - Phase 4
#[cfg(feature = "system_optimizer")] pub mod system_optimizer; // Advanced system optimization and tuning - Phase 4
#[cfg(feature = "performance_profiler")] pub mod performance_profiler; // Performance profiling and benchmarking - Phase 4
#[cfg(feature = "documentation_system")] pub mod documentation_system; // Comprehensive documentation generation - Phase 4
#[cfg(feature = "internationalization")] pub mod internationalization; // Full internationalization system - Phase 4 
#[cfg(feature = "test_framework")] pub mod test_framework; // Comprehensive testing framework - Phase 4
pub mod closures; // First-class function and closure support
pub mod error_handling; // Advanced error handling system
#[cfg(feature = "powershell_compat")]
pub mod powershell_compat; // PowerShell compatibility mode

// Re-export after module declarations to avoid unresolved import during compilation order
pub use macros::{MacroSystem, Macro, MacroInfo};

/// Initialize the NexusShell core runtime
pub fn initialize() -> ShellResult<()> {
    nxsh_hal::initialize()?;
    nxsh_log_info!("NexusShell core initialized");
    Ok(())
}

/// Shutdown the NexusShell core runtime
pub fn shutdown() -> ShellResult<()> {
    nxsh_hal::shutdown()?;
    nxsh_log_info!("NexusShell core shutdown");
    Ok(())
} 

// Lightweight logging facade macros – keep call sites but allow stripping heavy formatting in minimal builds
#[cfg(feature = "minimal-logging")]
#[macro_export]
macro_rules! nxsh_log_debug { ($($tt:tt)*) => { /* stripped in minimal build */ }; }
#[cfg(not(feature = "minimal-logging"))]
#[macro_export]
macro_rules! nxsh_log_debug { ($($tt:tt)*) => { tracing::debug!($($tt)*); }; }

#[cfg(feature = "minimal-logging")]
#[macro_export]
macro_rules! nxsh_log_info { ($($tt:tt)*) => { /* stripped */ }; }
#[cfg(not(feature = "minimal-logging"))]
#[macro_export]
macro_rules! nxsh_log_info { ($($tt:tt)*) => { tracing::info!($($tt)*); }; }

#[cfg(feature = "minimal-logging")]
#[macro_export]
macro_rules! nxsh_log_warn { ($($tt:tt)*) => { /* stripped */ }; }
#[cfg(not(feature = "minimal-logging"))]
#[macro_export]
macro_rules! nxsh_log_warn { ($($tt:tt)*) => { tracing::warn!($($tt)*); }; }

#[cfg(feature = "minimal-logging")]
#[macro_export]
macro_rules! nxsh_log_error { ($($tt:tt)*) => { /* stripped */ }; }
#[cfg(not(feature = "minimal-logging"))]
#[macro_export]
macro_rules! nxsh_log_error { ($($tt:tt)*) => { tracing::error!($($tt)*); }; }