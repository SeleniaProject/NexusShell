//! NexusShell Core Library
//!
//! This is the core library for NexusShell, providing the fundamental
//! components for shell operations, including parsing, execution,
//! job control, and system integration.

// Re-export commonly used types and functions
pub use context::{Context, ShellContext};
pub use error::{ErrorKind, ShellError, ShellResult};
pub use executor::{Builtin, ExecutionResult, Executor};
pub use job::{Job, JobManager, JobStatus};
#[cfg(feature = "logging")]
pub use logging::LoggingSystem;
#[cfg(feature = "metrics")]
pub use metrics::{MetricsConfig, MetricsSystem};
pub use namespace::{ImportStatement, Module, NamespaceSystem, Symbol};
pub use pattern_matching::{MatchResult, PatternMatchingEngine, PatternValue};
pub use shell::{Config, Shell, ShellState};
pub use stream::{Stream, StreamData, StreamType};
// Removed safe crate imports - implementing custom safe wrappers instead
#[cfg(feature = "advanced_scheduler")]
pub use advanced_scheduler::{
    AdvancedJobScheduler, JobExecutionResult, JobSchedule, JobStatistics, ScheduledJob,
};
pub use closures::{Closure, ClosureSystem, ExecutionContext, Function};
pub use error_handling::{ErrorHandler, ErrorHandlingSystem, ErrorInfo, ErrorResult};
pub use memory::{global_memory_manager, MemoryManager, MemoryPool, StringInterner};
pub use memory_efficient::{fast_format, MemoryEfficientStringBuilder};
#[cfg(feature = "monitoring")]
pub use monitoring::{
    Alert, AlertConfig, AlertLevel, DashboardData, MonitoringConfig, MonitoringSystem,
    SystemMetrics,
};
pub use performance::{PerformanceConfig, PerformanceOptimizer, PerformanceReport};
#[cfg(feature = "powershell_compat")]
pub use powershell_compat::{CommandResult, PowerShellCompat, PowerShellObject};
#[cfg(feature = "security_auditor")]
pub use security_auditor::{AuditFinding, AuditScope, SecurityAuditReport, SecurityAuditor};
pub use simd_optimization::{CpuFeatures, CpuOptimizer, SimdStringOps};
pub use startup::{StartupConfig, StartupOptimizer, StartupReport, StartupTimer};
pub use structured_logging::{
    CommandExecutionLog, LogConfig, LogFormat, LogStats, RotationConfig, StructuredLogger,
};
// NexusShell-inspired structured data processing
#[cfg(feature = "documentation_system")]
pub use documentation_system::{
    ApiDocumentationReport, DeveloperDocumentationReport, DocumentationSystem,
    UserDocumentationReport,
};
#[cfg(feature = "internationalization")]
pub use internationalization::{
    InternationalizationSystem, LanguagePack, PluralizationRule, ValidationReport,
};
#[cfg(feature = "performance_profiler")]
pub use performance_profiler::{
    BenchmarkResult, BottleneckAnalysis, PerformanceProfiler, ProfilingSession,
};
pub use structured_commands::*;
pub use structured_data::{PipelineData, StructuredCommand, StructuredValue};
#[cfg(feature = "system_optimizer")]
pub use system_optimizer::{OptimizationProfile, SystemOptimizationReport, SystemOptimizer};
#[cfg(feature = "test_framework")]
pub use test_framework::{ComprehensiveTestReport, PerformanceBenchmark, TestFramework, TestSuite};

// Public modules
#[cfg(feature = "advanced_scheduler")]
pub mod advanced_scheduler;
pub mod builtins;
pub mod closures; // First-class function and closure support
pub mod compat; // new compatibility layer (anyhow substitute)
pub mod context;
pub mod crash_handler;
#[cfg(feature = "documentation_system")]
pub mod documentation_system; // Comprehensive documentation generation - Phase 4
pub mod encryption;
pub mod error;
pub mod error_handling; // Advanced error handling system
pub mod executor;
#[cfg(feature = "internationalization")]
pub mod i18n;
#[cfg(feature = "heavy-time")]
pub mod integration_test_system;
#[cfg(feature = "internationalization")]
pub mod internationalization; // Full internationalization system - Phase 4
pub mod io_optimization;
pub mod job;
pub mod locale_alias;
#[cfg(feature = "logging")]
pub mod logging;
pub mod macros; // macro system module
pub mod memory;
pub mod memory_efficient;
#[cfg(feature = "metrics")]
pub mod metrics;
pub mod mir; // MIR System - Phase 1: Basic types  // Temporarily disabled for compilation stability
#[cfg(feature = "monitoring")]
pub mod monitoring;
pub mod namespace; // Namespace and module system
pub mod network_security;
pub mod pattern_matching; // Advanced pattern matching engine
pub mod performance; // Performance optimization system
#[cfg(feature = "performance_profiler")]
pub mod performance_profiler; // Performance profiling and benchmarking - Phase 4
#[cfg(feature = "powershell_compat")]
pub mod powershell_compat;
pub mod result;
pub mod safe; // Safe error handling to eliminate panic! calls
#[cfg(feature = "security_auditor")]
pub mod security_auditor; // Security audit and compliance system - Phase 4
pub mod shell;
pub mod simd_optimization;
pub mod startup; // Startup time optimization system
pub mod stream;
pub mod structured_commands; // Commands for structured data
pub mod structured_data; // NexusShell-inspired structured data processing
pub mod structured_logging;
#[cfg(feature = "system_optimizer")]
pub mod system_optimizer; // Advanced system optimization and tuning - Phase 4
#[cfg(feature = "test_framework")]
pub mod test_framework; // Comprehensive testing framework - Phase 4
pub mod updater; // PowerShell compatibility mode

// Re-export after module declarations to avoid unresolved import during compilation order
pub use macros::{Macro, MacroInfo, MacroSystem};

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

/// Execute an AST node using the core execution engine
pub fn execute_ast(
    ast: &nxsh_parser::ast::AstNode,
    shell_state: &mut ShellState,
) -> ShellResult<i32> {
    let mut shell = Shell::from_state(shell_state.clone());
    let result = shell.eval_ast(ast)?;
    *shell_state = shell.into_state();
    Ok(result.exit_code)
}

// Lightweight logging facade macros – keep call sites but allow stripping heavy formatting in minimal builds
#[cfg(feature = "minimal-logging")]
#[macro_export]
macro_rules! nxsh_log_debug {
    ($($tt:tt)*) => {
        /* stripped in minimal build */
    };
}
#[cfg(not(feature = "minimal-logging"))]
#[macro_export]
macro_rules! nxsh_log_debug { ($($tt:tt)*) => { tracing::debug!($($tt)*); }; }

#[cfg(feature = "minimal-logging")]
#[macro_export]
macro_rules! nxsh_log_info {
    ($($tt:tt)*) => {
        /* stripped */
    };
}
#[cfg(not(feature = "minimal-logging"))]
#[macro_export]
macro_rules! nxsh_log_info { ($($tt:tt)*) => { tracing::info!($($tt)*); }; }

#[cfg(feature = "minimal-logging")]
#[macro_export]
macro_rules! nxsh_log_warn {
    ($($tt:tt)*) => {
        /* stripped */
    };
}
#[cfg(not(feature = "minimal-logging"))]
#[macro_export]
macro_rules! nxsh_log_warn { ($($tt:tt)*) => { tracing::warn!($($tt)*); }; }

#[cfg(feature = "minimal-logging")]
#[macro_export]
macro_rules! nxsh_log_error {
    ($($tt:tt)*) => {
        /* stripped */
    };
}
#[cfg(not(feature = "minimal-logging"))]
#[macro_export]
macro_rules! nxsh_log_error { ($($tt:tt)*) => { tracing::error!($($tt)*); }; }
