//! Example demonstrating the advanced monitoring and observability features of NexusShell
//! 
//! This example shows how to:
//! - Initialize the comprehensive monitoring system
//! - Configure structured logging with JSON output
//! - Set up Prometheus metrics collection
//! - Enable crash handling and reporting
//! - Configure automatic updates with signature verification
//! - Monitor system performance and health
//! - Handle monitoring events and alerts

use nxsh_core::{
    initialize_monitoring, MonitoringSystem,
    LoggingConfig, LogFormat, MetricsConfig, CrashHandlerConfig, UpdateConfig, UpdateChannel,
    logging::LogEntry, metrics::MetricEvent, crash_handler::CrashEvent, updater::UpdateEvent,
};
use std::{collections::HashMap, path::PathBuf, time::Duration};
use tokio::time::sleep;
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 NexusShell Advanced Monitoring System Demo");
    println!("==============================================");

    // Initialize monitoring system with custom configuration
    let mut monitoring = initialize_comprehensive_monitoring().await?;

    // Demonstrate structured logging
    demonstrate_structured_logging(&monitoring).await?;

    // Demonstrate metrics collection
    demonstrate_metrics_collection(&monitoring).await?;

    // Demonstrate crash handling (simulated)
    demonstrate_crash_handling(&monitoring).await?;

    // Demonstrate update system
    demonstrate_update_system(&monitoring).await?;

    // Monitor system for a while
    monitor_system_health(&monitoring).await?;

    // Shutdown gracefully
    monitoring.shutdown().await?;

    println!("✅ Monitoring system demo completed successfully!");
    Ok(())
}

/// Initialize comprehensive monitoring with custom configuration
async fn initialize_comprehensive_monitoring() -> anyhow::Result<MonitoringSystem> {
    println!("📊 Initializing comprehensive monitoring system...");

    // Configure structured logging
    let logging_config = LoggingConfig {
        level: "debug".to_string(),
        format: LogFormat::Json,
        log_dir: PathBuf::from("./demo_logs"),
        console_output: true,
        file_output: true,
        retention_days: 7,
        encryption: false, // Disabled for demo
        sanitization: true,
        performance_monitoring: true,
        audit_logging: true,
        distributed_tracing: false, // Disabled for demo
        ..Default::default()
    };

    // Configure metrics collection
    let metrics_config = MetricsConfig {
        enabled: true,
        export_port: 9090,
        collection_interval_secs: 10,
        system_metrics: true,
        job_metrics: true,
        plugin_metrics: true,
        network_metrics: true,
        alerting: true,
        default_labels: {
            let mut labels = HashMap::new();
            labels.insert("environment".to_string(), "demo".to_string());
            labels.insert("instance".to_string(), "monitoring-demo".to_string());
            labels
        },
        ..Default::default()
    };

    // Configure crash handling
    let crash_config = CrashHandlerConfig {
        enabled: true,
        crash_dir: PathBuf::from("./demo_crashes"),
        minidump_enabled: true,
        stack_trace_enabled: true,
        privacy_mode: true,
        auto_submit: false, // Disabled for demo
        recovery_enabled: true,
        prevention_enabled: true,
        realtime_monitoring: true,
        monitoring_interval_secs: 30,
        ..Default::default()
    };

    // Configure update system
    let update_config = UpdateConfig {
        auto_update: false, // Manual for demo
        channel: UpdateChannel::Beta,
        check_interval_hours: 1,
        signature_verification: true,
        differential_updates: true,
        rollback_on_failure: true,
        user_consent_required: false, // Disabled for demo
        validation_enabled: true,
        ..Default::default()
    };

    let mut monitoring = MonitoringSystem::new(
        logging_config,
        metrics_config,
        crash_config,
        update_config,
    )?;

    monitoring.initialize().await?;
    println!("✅ Monitoring system initialized successfully!");

    Ok(monitoring)
}

/// Demonstrate structured logging capabilities
async fn demonstrate_structured_logging(monitoring: &MonitoringSystem) -> anyhow::Result<()> {
    println!("\n📝 Demonstrating structured logging...");

    // Create sample log entries with rich context
    let log_entries = vec![
        LogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            message: "System startup completed".to_string(),
            module: Some("main".to_string()),
            file: Some("main.rs".to_string()),
            line: Some(42),
            thread_id: "main".to_string(),
            span_id: Some("span-123".to_string()),
            trace_id: Some("trace-456".to_string()),
            fields: {
                let mut fields = HashMap::new();
                fields.insert("startup_time_ms".to_string(), serde_json::json!(1250));
                fields.insert("version".to_string(), serde_json::json!("1.0.0"));
                fields.insert("build".to_string(), serde_json::json!("debug"));
                fields
            },
            pid: std::process::id(),
            hostname: "demo-host".to_string(),
            service: "nxsh".to_string(),
            version: "1.0.0".to_string(),
            environment: "demo".to_string(),
            user_id: Some("demo-user".to_string()),
            session_id: Some("session-789".to_string()),
            request_id: None,
            performance: None,
        },
        LogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level: "WARN".to_string(),
            message: "High memory usage detected".to_string(),
            module: Some("monitor".to_string()),
            file: Some("monitor.rs".to_string()),
            line: Some(156),
            thread_id: "monitor".to_string(),
            span_id: None,
            trace_id: None,
            fields: {
                let mut fields = HashMap::new();
                fields.insert("memory_usage_mb".to_string(), serde_json::json!(856));
                fields.insert("threshold_mb".to_string(), serde_json::json!(800));
                fields.insert("process_count".to_string(), serde_json::json!(23));
                fields
            },
            pid: std::process::id(),
            hostname: "demo-host".to_string(),
            service: "nxsh".to_string(),
            version: "1.0.0".to_string(),
            environment: "demo".to_string(),
            user_id: Some("demo-user".to_string()),
            session_id: Some("session-789".to_string()),
            request_id: None,
            performance: None,
        },
    ];

    // Log entries and show statistics
    for entry in log_entries {
        monitoring.logging.log_entry(entry).await?;
    }

    // Wait for processing
    sleep(Duration::from_millis(100)).await;

    let stats = monitoring.logging.get_statistics();
    println!("  ✅ Logged {} entries", stats.total_entries);
    println!("  📊 Errors: {}, Warnings: {}", stats.errors, stats.warnings);
    println!("  ⏱️  Average processing time: {}μs", stats.avg_processing_time_micros);

    Ok(())
}

/// Demonstrate metrics collection and monitoring
async fn demonstrate_metrics_collection(monitoring: &MonitoringSystem) -> anyhow::Result<()> {
    println!("\n📈 Demonstrating metrics collection...");

    // Simulate some job executions
    for i in 0..5 {
        let duration = Duration::from_millis(100 + i * 50);
        let success = i % 4 != 0; // Simulate some failures
        let memory_usage = 1024 * 1024 * (10 + i); // Simulate varying memory usage

        monitoring.metrics.record_job_execution(duration, success, memory_usage);
        
        // Simulate plugin calls
        monitoring.metrics.record_plugin_call(
            &format!("demo_plugin_{}", i % 3),
            Duration::from_millis(25 + i * 10),
            success,
        );

        // Simulate command executions
        monitoring.metrics.record_command_execution(
            if i % 2 == 0 { "builtin" } else { "external" },
            duration,
            success,
        );

        sleep(Duration::from_millis(50)).await;
    }

    // Simulate network activity
    monitoring.metrics.record_network_activity(1024 * 50, 1024 * 25);

    // Get and display statistics
    let stats = monitoring.metrics.get_statistics();
    println!("  ✅ Total jobs: {}", stats.total_jobs);
    println!("  🟢 Successful jobs: {}", stats.successful_jobs);
    println!("  🔴 Failed jobs: {}", stats.failed_jobs);
    println!("  🔌 Plugin calls: {}", stats.plugin_calls);
    println!("  🌐 Network bytes sent: {}", stats.bytes_sent);
    println!("  📊 Metrics available at: http://localhost:9090/metrics");

    Ok(())
}

/// Demonstrate crash handling capabilities
async fn demonstrate_crash_handling(monitoring: &MonitoringSystem) -> anyhow::Result<()> {
    println!("\n🚨 Demonstrating crash handling...");

    // Subscribe to crash events
    let mut crash_events = monitoring.crash_handler.subscribe_events();

    // Simulate performance warnings (not actual crashes for demo safety)
    tokio::spawn(async move {
        while let Ok(event) = crash_events.recv().await {
            match event {
                CrashEvent::PerformanceWarning { metric, value, threshold } => {
                    println!("  ⚠️  Performance warning: {} = {:.2} (threshold: {:.2})", 
                        metric, value, threshold);
                }
                CrashEvent::CrashDetected { crash_id, severity } => {
                    println!("  🚨 Crash detected: {} (severity: {:?})", crash_id, severity);
                }
                _ => {}
            }
        }
    });

    // Get crash statistics
    let stats = monitoring.crash_handler.get_statistics();
    println!("  📊 Total crashes: {}", stats.total_crashes);
    println!("  📈 Crash frequency: {:.2} per hour", stats.crash_frequency);
    println!("  🔄 Recovery success rate: {:.1}%", stats.recovery_success_rate * 100.0);

    Ok(())
}

/// Demonstrate update system capabilities
async fn demonstrate_update_system(monitoring: &MonitoringSystem) -> anyhow::Result<()> {
    println!("\n🔄 Demonstrating update system...");

    // Subscribe to update events
    let mut update_events = monitoring.updater.subscribe_events();

    tokio::spawn(async move {
        while let Ok(event) = update_events.recv().await {
            match event {
                UpdateEvent::UpdateAvailable { version, channel, security_fixes } => {
                    println!("  📦 Update available: {} on {:?} ({} security fixes)", 
                        version, channel, security_fixes);
                }
                UpdateEvent::ProgressUpdate { operation_id, phase, progress, message } => {
                    println!("  🔄 Update progress [{}]: {:?} - {:.1}% - {}", 
                        &operation_id[..8], phase, progress, message);
                }
                UpdateEvent::UpdateCompleted { version, operation_id } => {
                    println!("  ✅ Update completed: {} [{}]", version, &operation_id[..8]);
                }
                UpdateEvent::UpdateFailed { version, operation_id, error } => {
                    println!("  ❌ Update failed: {} [{}] - {}", version, &operation_id[..8], error);
                }
                _ => {}
            }
        }
    });

    // Check for updates (will likely find none in demo environment)
    println!("  🔍 Checking for updates...");
    match monitoring.updater.check_for_updates().await {
        Ok(Some(version_info)) => {
            println!("  📦 Update available: {}", version_info.version);
            println!("    📅 Release date: {}", version_info.release_date);
            println!("    🔒 Security fixes: {}", version_info.security_fixes.len());
            println!("    📝 Release notes: {}", version_info.release_notes);
        }
        Ok(None) => {
            println!("  ✅ No updates available");
        }
        Err(e) => {
            println!("  ⚠️  Update check failed: {}", e);
        }
    }

    // Show update history
    let history = monitoring.updater.get_update_history();
    println!("  📚 Update history: {} entries", history.len());

    Ok(())
}

/// Monitor system health and show real-time information
async fn monitor_system_health(monitoring: &MonitoringSystem) -> anyhow::Result<()> {
    println!("\n🏥 Monitoring system health for 10 seconds...");

    let start_time = std::time::Instant::now();
    let mut interval = tokio::time::interval(Duration::from_secs(2));

    while start_time.elapsed() < Duration::from_secs(10) {
        interval.tick().await;

        // Get current statistics
        let log_stats = monitoring.logging.get_statistics();
        let metric_stats = monitoring.metrics.get_statistics();
        let crash_stats = monitoring.crash_handler.get_statistics();

        println!("  📊 Health Check:");
        println!("    📝 Log entries: {} (avg: {}μs)", 
            log_stats.total_entries, 
            log_stats.avg_processing_time_micros);
        println!("    📈 Active jobs: {}, Total: {}", 
            metric_stats.active_jobs, 
            metric_stats.total_jobs);
        println!("    🚨 Total crashes: {}", crash_stats.total_crashes);
        println!("    ⏱️  Uptime: {:?}", metric_stats.uptime);
        println!("    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    println!("  ✅ Health monitoring completed");
    Ok(())
}

/// Demonstrate advanced monitoring features
#[allow(dead_code)]
async fn demonstrate_advanced_features(monitoring: &MonitoringSystem) -> anyhow::Result<()> {
    println!("\n🔬 Demonstrating advanced monitoring features...");

    // Demonstrate custom metrics
    // monitoring.metrics.record_custom_metric("demo_counter", 42.0);
    // monitoring.metrics.record_histogram("demo_histogram", 123.45);

    // Demonstrate log correlation
    let correlation_id = Uuid::new_v4().to_string();
    println!("  🔗 Correlation ID: {}", correlation_id);

    // Demonstrate distributed tracing (if enabled)
    println!("  🌐 Distributed tracing: {}", 
        if monitoring.logging.config.distributed_tracing { "enabled" } else { "disabled" });

    // Demonstrate alerting
    println!("  🚨 Alerting: {}", 
        if monitoring.metrics.config.alerting { "enabled" } else { "disabled" });

    // Demonstrate performance profiling
    println!("  ⚡ Performance monitoring: {}", 
        if monitoring.logging.config.performance_monitoring { "enabled" } else { "disabled" });

    Ok(())
}

/// Helper function to format duration
#[allow(dead_code)]
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Helper function to format bytes
#[allow(dead_code)]
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit])
} 