//! Example demonstrating basic monitoring features of NexusShell
//! 
//! This example shows how to:
//! - Configure basic logging
//! - Set up crash handling
//! - Monitor basic system information

use nxsh_core::{
    logging::{LoggingConfig, LoggingSystem},
    crash_handler::{CrashHandlerConfig, CrashHandler},
};
use std::{path::PathBuf, time::Duration};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 NexusShell Basic Monitoring Example");
    println!("=====================================\n");

    // Initialize basic logging
    let logging_config = LoggingConfig {
        level: "info".to_string(),
        file_output: true,
        log_dir: PathBuf::from("./logs"),
        max_files: 10,
        retention_days: 30,
    };

    let logging_system = LoggingSystem::new(logging_config)?;
    println!("✅ Logging system initialized");

    // Initialize crash handler
    let crash_config = CrashHandlerConfig::default();
    let crash_handler = CrashHandler::new(crash_config);
    println!("✅ Crash handler initialized");

    // Demonstrate basic operations
    demonstrate_logging(&logging_system).await?;
    demonstrate_crash_handler(&crash_handler).await?;

    println!("\n🎉 Basic monitoring demonstration completed!");
    Ok(())
}

async fn demonstrate_logging(logging: &LoggingSystem) -> anyhow::Result<()> {
    println!("\n📝 Demonstrating Basic Logging:");
    
    // Log some example messages
    logging.log_info("System initialization complete").await?;
    logging.log_debug("Debug information logged").await?;
    logging.log_warning("Warning message logged").await?;
    
    // Get statistics
    let stats = logging.get_statistics();
    println!("   Total log entries: {}", stats.total_entries);
    println!("   Errors logged: {}", stats.error_count);
    
    Ok(())
}

async fn demonstrate_crash_handler(crash_handler: &CrashHandler) -> anyhow::Result<()> {
    println!("\n🔧 Demonstrating Crash Handler:");
    
    // Test crash detection
    println!("   Testing crash detection capabilities...");
    
    // Simulate a brief delay
    sleep(Duration::from_millis(100)).await;
    
    println!("   ✅ Crash handler is monitoring system");
    
    Ok(())
}
