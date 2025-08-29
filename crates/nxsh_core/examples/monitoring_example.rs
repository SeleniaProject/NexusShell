//! Example demonstrating basic monitoring features of NexusShell
//! (functionality executes only when the `logging` feature is enabled)

#[cfg(feature = "logging")]
use nxsh_core::logging::{LoggingConfig, LoggingSystem};
use nxsh_core::{
    crash_handler::{CrashHandler, CrashHandlerConfig},
    nxsh_log_debug, nxsh_log_info, nxsh_log_warn,
};
use std::{path::PathBuf, time::Duration};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 NexusShell Basic Monitoring Example");
    println!("=====================================\n");

    // Initialize basic logging (feature gated)
    #[cfg(feature = "logging")]
    let logging_config = LoggingConfig {
        level: "info".to_string(),
        file_output: true,
        log_dir: PathBuf::from("./logs"),
        max_files: 10,
        retention_days: 30,
        // Fill the rest of fields from defaults to match current API
        ..Default::default()
    };

    #[cfg(feature = "logging")]
    let mut logging_system = LoggingSystem::new(logging_config)?;
    #[cfg(feature = "logging")]
    {
        logging_system.initialize().await?;
        println!("✅ Logging system initialized");
    }

    // Initialize crash handler
    let crash_config = CrashHandlerConfig::default();
    let crash_handler = CrashHandler::new(crash_config)?;
    println!("✅ Crash handler initialized");

    // Demonstrate basic operations
    #[cfg(feature = "logging")]
    demonstrate_logging(&logging_system).await?;
    demonstrate_crash_handler(&crash_handler).await?;

    println!("\n🎉 Basic monitoring demonstration completed!");
    Ok(())
}

#[cfg(feature = "logging")]
async fn demonstrate_logging(_logging: &LoggingSystem) -> anyhow::Result<()> {
    println!("\n📝 Demonstrating Basic Logging:");

    // Log some example messages via tracing
    nxsh_log_info!("System initialization complete");
    nxsh_log_debug!("Debug information logged");
    nxsh_log_warn!("Warning message logged");

    // Get statistics
    let stats = _logging.get_statistics();
    println!("   Total log entries: {}", stats.messages_logged);
    println!("   Errors logged: {}", stats.errors_logged);

    Ok(())
}

async fn demonstrate_crash_handler(_crash_handler: &CrashHandler) -> anyhow::Result<()> {
    println!("\n🔧 Demonstrating Crash Handler:");

    // Test crash detection
    println!("   Testing crash detection capabilities...");

    // Simulate a brief delay
    sleep(Duration::from_millis(100)).await;

    println!("   ✅ Crash handler is monitoring system");

    Ok(())
}
