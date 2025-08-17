//! Comprehensive metrics collection and monitoring for NexusShell
//!
//! This module provides detailed metrics collection for monitoring shell performance,
//! resource usage, job execution statistics, and operational metrics in Prometheus format.

use crate::compat::Result;
use std::{
    sync::{Arc, RwLock, atomic::{AtomicU64, AtomicI64, Ordering}}, // Changed AtomicF64 to AtomicI64 for compatibility
    time::{Duration, SystemTime, Instant},
    collections::HashMap,
    fmt::Write,
};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Comprehensive metrics system with Prometheus format support
#[allow(dead_code)] // ランタイム集計の将来拡張用フィールドが未参照
pub struct MetricsSystem {
    config: MetricsConfig,
    statistics: Arc<MetricsStatistics>,
    start_time: SystemTime,
    start_instant: Instant,
    prometheus_registry: Arc<RwLock<PrometheusRegistry>>,
}

/// Enhanced metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Metrics collection interval in seconds
    pub collection_interval_secs: u64,
    /// Enable Prometheus format export
    pub prometheus_enabled: bool,
    /// Prometheus metrics endpoint port
    pub prometheus_port: u16,
    /// Enable detailed job metrics
    pub job_metrics_enabled: bool,
    /// Enable memory metrics
    pub memory_metrics_enabled: bool,
    /// Enable performance metrics
    pub performance_metrics_enabled: bool,
    /// Metrics retention period in hours
    pub retention_hours: u64,
    /// Enable histogram metrics
    pub histogram_enabled: bool,
    /// Custom labels for all metrics
    pub custom_labels: HashMap<String, String>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            collection_interval_secs: 30,
            prometheus_enabled: true,
            prometheus_port: 9090,
            job_metrics_enabled: true,
            memory_metrics_enabled: true,
            performance_metrics_enabled: true,
            retention_hours: 24,
            histogram_enabled: true,
            custom_labels: HashMap::new(),
        }
    }
}

/// Comprehensive metrics statistics with atomic operations
#[derive(Debug)]
pub struct MetricsStatistics {
    // System metrics
    pub uptime_seconds: AtomicU64,
    pub last_collection: Arc<RwLock<Option<SystemTime>>>,
    pub memory_usage_bytes: AtomicU64,
    pub cpu_usage_percent: AtomicI64,
    
    // Job metrics
    pub total_jobs_started: AtomicU64,
    pub total_jobs_completed: AtomicU64,
    pub total_jobs_failed: AtomicU64,
    pub active_jobs: AtomicU64,
    pub background_jobs: AtomicU64,
    pub foreground_jobs: AtomicU64,
    
    // Command execution metrics
    pub commands_executed: AtomicU64,
    pub builtin_commands_executed: AtomicU64,
    pub external_commands_executed: AtomicU64,
    pub command_execution_time_total_ms: AtomicU64,
    pub command_failures: AtomicU64,
    
    // Parser metrics
    pub lines_parsed: AtomicU64,
    pub parse_errors: AtomicU64,
    pub syntax_errors: AtomicU64,
    pub parse_time_total_ms: AtomicU64,
    
    // I/O metrics
    pub bytes_read: AtomicU64,
    pub bytes_written: AtomicU64,
    pub files_opened: AtomicU64,
    pub files_closed: AtomicU64,
    pub pipes_created: AtomicU64,
    
    // Network metrics
    pub network_connections_opened: AtomicU64,
    pub network_connections_closed: AtomicU64,
    pub network_bytes_sent: AtomicU64,
    pub network_bytes_received: AtomicU64,
    
    // Error metrics
    pub total_errors: AtomicU64,
    pub warning_count: AtomicU64,
    pub critical_errors: AtomicU64,
    pub recoverable_errors: AtomicU64,
    
    // Performance metrics
    pub startup_time_ms: AtomicU64,
    pub tab_completion_time_ms: AtomicU64,
    pub prompt_render_time_ms: AtomicU64,
    pub gc_collections: AtomicU64,
    pub gc_time_total_ms: AtomicU64,
}

impl MetricsStatistics {
    fn new() -> Self {
        Self {
            uptime_seconds: AtomicU64::new(0),
            last_collection: Arc::new(RwLock::new(None)),
            memory_usage_bytes: AtomicU64::new(0),
            cpu_usage_percent: AtomicI64::new(0),
            
            total_jobs_started: AtomicU64::new(0),
            total_jobs_completed: AtomicU64::new(0),
            total_jobs_failed: AtomicU64::new(0),
            active_jobs: AtomicU64::new(0),
            background_jobs: AtomicU64::new(0),
            foreground_jobs: AtomicU64::new(0),
            
            commands_executed: AtomicU64::new(0),
            builtin_commands_executed: AtomicU64::new(0),
            external_commands_executed: AtomicU64::new(0),
            command_execution_time_total_ms: AtomicU64::new(0),
            command_failures: AtomicU64::new(0),
            
            lines_parsed: AtomicU64::new(0),
            parse_errors: AtomicU64::new(0),
            syntax_errors: AtomicU64::new(0),
            parse_time_total_ms: AtomicU64::new(0),
            
            bytes_read: AtomicU64::new(0),
            bytes_written: AtomicU64::new(0),
            files_opened: AtomicU64::new(0),
            files_closed: AtomicU64::new(0),
            pipes_created: AtomicU64::new(0),
            
            network_connections_opened: AtomicU64::new(0),
            network_connections_closed: AtomicU64::new(0),
            network_bytes_sent: AtomicU64::new(0),
            network_bytes_received: AtomicU64::new(0),
            
            total_errors: AtomicU64::new(0),
            warning_count: AtomicU64::new(0),
            critical_errors: AtomicU64::new(0),
            recoverable_errors: AtomicU64::new(0),
            
            startup_time_ms: AtomicU64::new(0),
            tab_completion_time_ms: AtomicU64::new(0),
            prompt_render_time_ms: AtomicU64::new(0),
            gc_collections: AtomicU64::new(0),
            gc_time_total_ms: AtomicU64::new(0),
        }
    }
}

/// Prometheus metrics registry
#[derive(Debug, Default)]
pub struct PrometheusRegistry {
    counters: HashMap<String, PrometheusCounter>,
    gauges: HashMap<String, PrometheusGauge>,
    histograms: HashMap<String, PrometheusHistogram>,
}

/// Prometheus counter metric
#[derive(Debug)]
pub struct PrometheusCounter {
    pub name: String,
    pub help: String,
    pub value: AtomicU64,
    pub labels: HashMap<String, String>,
}

impl Clone for PrometheusCounter {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            help: self.help.clone(),
            value: AtomicU64::new(self.value.load(Ordering::Relaxed)),
            labels: self.labels.clone(),
        }
    }
}

/// Prometheus gauge metric  
#[derive(Debug)]
pub struct PrometheusGauge {
    pub name: String,
    pub help: String,
    pub value: AtomicI64, // Changed from AtomicF64 to AtomicI64 for compatibility
    pub labels: HashMap<String, String>,
}

impl Clone for PrometheusGauge {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            help: self.help.clone(),
            value: AtomicI64::new(self.value.load(Ordering::Relaxed)),
            labels: self.labels.clone(),
        }
    }
}

/// Prometheus histogram metric
#[derive(Debug)]
pub struct PrometheusHistogram {
    pub name: String,
    pub help: String,
    pub buckets: Vec<f64>,
    pub counts: Vec<AtomicU64>,
    pub sum: AtomicI64, // Changed from AtomicF64 to AtomicI64 for compatibility
    pub count: AtomicU64,
    pub labels: HashMap<String, String>,
}

impl Clone for PrometheusHistogram {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            help: self.help.clone(),
            buckets: self.buckets.clone(),
            counts: self.counts.iter()
                .map(|c| AtomicU64::new(c.load(Ordering::Relaxed)))
                .collect(),
            sum: AtomicI64::new(self.sum.load(Ordering::Relaxed)),
            count: AtomicU64::new(self.count.load(Ordering::Relaxed)),
            labels: self.labels.clone(),
        }
    }
}

/// Serializable metrics summary for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub timestamp: SystemTime,
    pub uptime_seconds: u64,
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f64,
    
    pub total_jobs_started: u64,
    pub total_jobs_completed: u64,
    pub total_jobs_failed: u64,
    pub active_jobs: u64,
    
    pub commands_executed: u64,
    pub command_execution_time_total_ms: u64,
    pub command_failures: u64,
    
    pub lines_parsed: u64,
    pub parse_errors: u64,
    pub parse_time_total_ms: u64,
    
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub files_opened: u64,
    
    pub total_errors: u64,
    pub warning_count: u64,
    pub critical_errors: u64,
}

impl MetricsSystem {
    /// Create a new comprehensive metrics system
    pub fn new(config: MetricsConfig) -> Result<Self> {
        let registry = PrometheusRegistry::default();
        
        Ok(Self {
            config,
            statistics: Arc::new(MetricsStatistics::new()),
            start_time: SystemTime::now(),
            start_instant: Instant::now(),
            prometheus_registry: Arc::new(RwLock::new(registry)),
        })
    }

    /// Initialize the metrics system
    pub async fn initialize(&mut self) -> Result<()> {
        if !self.config.enable_metrics {
            info!("Metrics collection disabled");
            return Ok(());
        }

        // Initialize Prometheus metrics
        if self.config.prometheus_enabled {
            self.initialize_prometheus_metrics().await?;
        }

        // Record startup time
        let startup_time = self.start_instant.elapsed();
        self.statistics.startup_time_ms.store(startup_time.as_millis() as u64, Ordering::Relaxed);

        info!(
            prometheus_enabled = self.config.prometheus_enabled,
            collection_interval = self.config.collection_interval_secs,
            "Metrics system initialized"
        );

        Ok(())
    }

    /// Initialize Prometheus metrics registry
    async fn initialize_prometheus_metrics(&self) -> Result<()> {
        let mut registry = self.prometheus_registry.write().unwrap();
        
        // System metrics
        registry.counters.insert("nxsh_uptime_seconds_total".to_string(), PrometheusCounter {
            name: "nxsh_uptime_seconds_total".to_string(),
            help: "Total uptime in seconds".to_string(),
            value: AtomicU64::new(0),
            labels: self.config.custom_labels.clone(),
        });

        // Job metrics
        registry.counters.insert("nxsh_jobs_started_total".to_string(), PrometheusCounter {
            name: "nxsh_jobs_started_total".to_string(),
            help: "Total number of jobs started".to_string(),
            value: AtomicU64::new(0),
            labels: self.config.custom_labels.clone(),
        });

        registry.counters.insert("nxsh_jobs_completed_total".to_string(), PrometheusCounter {
            name: "nxsh_jobs_completed_total".to_string(),
            help: "Total number of jobs completed".to_string(),
            value: AtomicU64::new(0),
            labels: self.config.custom_labels.clone(),
        });

        registry.gauges.insert("nxsh_active_jobs".to_string(), PrometheusGauge {
            name: "nxsh_active_jobs".to_string(),
            help: "Number of currently active jobs".to_string(),
            value: AtomicI64::new(0),
            labels: self.config.custom_labels.clone(),
        });

        // Command execution metrics
        registry.counters.insert("nxsh_commands_executed_total".to_string(), PrometheusCounter {
            name: "nxsh_commands_executed_total".to_string(),
            help: "Total number of commands executed".to_string(),
            value: AtomicU64::new(0),
            labels: self.config.custom_labels.clone(),
        });

        // Memory metrics
        registry.gauges.insert("nxsh_memory_usage_bytes".to_string(), PrometheusGauge {
            name: "nxsh_memory_usage_bytes".to_string(),
            help: "Current memory usage in bytes".to_string(),
            value: AtomicI64::new(0),
            labels: self.config.custom_labels.clone(),
        });

        // Performance histograms
        if self.config.histogram_enabled {
            registry.histograms.insert("nxsh_command_execution_duration_ms".to_string(), PrometheusHistogram {
                name: "nxsh_command_execution_duration_ms".to_string(),
                help: "Command execution duration in milliseconds".to_string(),
                buckets: vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0],
                counts: (0..13).map(|_| AtomicU64::new(0)).collect(),
                sum: AtomicI64::new(0),
                count: AtomicU64::new(0),
                labels: self.config.custom_labels.clone(),
            });
        }

        info!("Prometheus metrics registry initialized");
        Ok(())
    }

    // Job metrics
    pub fn increment_jobs_started(&self) {
        self.statistics.total_jobs_started.fetch_add(1, Ordering::Relaxed);
        self.update_prometheus_counter("nxsh_jobs_started_total", 1);
    }

    pub fn increment_jobs_completed(&self) {
        self.statistics.total_jobs_completed.fetch_add(1, Ordering::Relaxed);
        self.update_prometheus_counter("nxsh_jobs_completed_total", 1);
    }

    pub fn increment_jobs_failed(&self) {
        self.statistics.total_jobs_failed.fetch_add(1, Ordering::Relaxed);
        self.update_prometheus_counter("nxsh_jobs_failed_total", 1);
    }

    pub fn set_active_jobs(&self, count: u64) {
        self.statistics.active_jobs.store(count, Ordering::Relaxed);
        self.update_prometheus_gauge("nxsh_active_jobs", count as f64);
    }

    // Command metrics
    pub fn increment_commands_executed(&self) {
        self.statistics.commands_executed.fetch_add(1, Ordering::Relaxed);
        self.update_prometheus_counter("nxsh_commands_executed_total", 1);
    }

    pub fn record_command_execution_time(&self, duration: Duration) {
        let ms = duration.as_millis() as u64;
        self.statistics.command_execution_time_total_ms.fetch_add(ms, Ordering::Relaxed);
        self.update_prometheus_histogram("nxsh_command_execution_duration_ms", ms as f64);
    }

    pub fn increment_command_failures(&self) {
        self.statistics.command_failures.fetch_add(1, Ordering::Relaxed);
        self.update_prometheus_counter("nxsh_command_failures_total", 1);
    }

    // Memory and system metrics
    pub fn update_memory_usage(&self, bytes: u64) {
        self.statistics.memory_usage_bytes.store(bytes, Ordering::Relaxed);
        self.update_prometheus_gauge("nxsh_memory_usage_bytes", bytes as f64);
    }

    pub fn update_cpu_usage(&self, percent: f64) {
        let bits = percent.to_bits();
        self.statistics.cpu_usage_percent.store(bits as i64, Ordering::Relaxed);
        self.update_prometheus_gauge("nxsh_cpu_usage_percent", percent);
    }

    // I/O metrics
    pub fn record_bytes_read(&self, bytes: u64) {
        self.statistics.bytes_read.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_bytes_written(&self, bytes: u64) {
        self.statistics.bytes_written.fetch_add(bytes, Ordering::Relaxed);
    }

    // Error metrics
    pub fn increment_errors(&self) {
        self.statistics.total_errors.fetch_add(1, Ordering::Relaxed);
        self.update_prometheus_counter("nxsh_errors_total", 1);
    }

    pub fn increment_critical_errors(&self) {
        self.statistics.critical_errors.fetch_add(1, Ordering::Relaxed);
        self.update_prometheus_counter("nxsh_critical_errors_total", 1);
    }

    // Internal Prometheus update methods
    fn update_prometheus_counter(&self, name: &str, value: u64) {
        if !self.config.prometheus_enabled {
            return;
        }
        
        if let Ok(registry) = self.prometheus_registry.read() {
            if let Some(counter) = registry.counters.get(name) {
                counter.value.fetch_add(value, Ordering::Relaxed);
            }
        }
    }

    fn update_prometheus_gauge(&self, name: &str, value: f64) {
        if !self.config.prometheus_enabled {
            return;
        }
        
        if let Ok(registry) = self.prometheus_registry.read() {
            if let Some(gauge) = registry.gauges.get(name) {
                gauge.value.store(value as i64, Ordering::Relaxed);
            }
        }
    }

    fn update_prometheus_histogram(&self, name: &str, value: f64) {
        if !self.config.prometheus_enabled || !self.config.histogram_enabled {
            return;
        }
        
        if let Ok(registry) = self.prometheus_registry.read() {
            if let Some(histogram) = registry.histograms.get(name) {
                histogram.count.fetch_add(1, Ordering::Relaxed);
                histogram.sum.store((histogram.sum.load(Ordering::Relaxed) as f64 + value) as i64, Ordering::Relaxed);
                
                for (i, &bucket) in histogram.buckets.iter().enumerate() {
                    if value <= bucket {
                        histogram.counts[i].fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }
    }

    /// Get current metrics summary
    pub fn get_summary(&self) -> MetricsSummary {
        MetricsSummary {
            timestamp: SystemTime::now(),
            uptime_seconds: self.start_instant.elapsed().as_secs(),
            memory_usage_bytes: self.statistics.memory_usage_bytes.load(Ordering::Relaxed),
            cpu_usage_percent: self.statistics.cpu_usage_percent.load(Ordering::Relaxed) as f64 / 100.0,
            
            total_jobs_started: self.statistics.total_jobs_started.load(Ordering::Relaxed),
            total_jobs_completed: self.statistics.total_jobs_completed.load(Ordering::Relaxed),
            total_jobs_failed: self.statistics.total_jobs_failed.load(Ordering::Relaxed),
            active_jobs: self.statistics.active_jobs.load(Ordering::Relaxed),
            
            commands_executed: self.statistics.commands_executed.load(Ordering::Relaxed),
            command_execution_time_total_ms: self.statistics.command_execution_time_total_ms.load(Ordering::Relaxed),
            command_failures: self.statistics.command_failures.load(Ordering::Relaxed),
            
            lines_parsed: self.statistics.lines_parsed.load(Ordering::Relaxed),
            parse_errors: self.statistics.parse_errors.load(Ordering::Relaxed),
            parse_time_total_ms: self.statistics.parse_time_total_ms.load(Ordering::Relaxed),
            
            bytes_read: self.statistics.bytes_read.load(Ordering::Relaxed),
            bytes_written: self.statistics.bytes_written.load(Ordering::Relaxed),
            files_opened: self.statistics.files_opened.load(Ordering::Relaxed),
            
            total_errors: self.statistics.total_errors.load(Ordering::Relaxed),
            warning_count: self.statistics.warning_count.load(Ordering::Relaxed),
            critical_errors: self.statistics.critical_errors.load(Ordering::Relaxed),
        }
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> Result<String> {
        if !self.config.prometheus_enabled {
            return Ok(String::new());
        }

        let mut output = String::new();
        let registry = self.prometheus_registry.read().unwrap();

        // Export counters
    for counter in registry.counters.values() {
            writeln!(&mut output, "# HELP {} {}", counter.name, counter.help)?;
            writeln!(&mut output, "# TYPE {} counter", counter.name)?;
            let value = counter.value.load(Ordering::Relaxed);
            let labels = format_labels(&counter.labels);
            writeln!(&mut output, "{}{} {}", counter.name, labels, value)?;
        }

        // Export gauges
    for gauge in registry.gauges.values() {
            writeln!(&mut output, "# HELP {} {}", gauge.name, gauge.help)?;
            writeln!(&mut output, "# TYPE {} gauge", gauge.name)?;
            let value = gauge.value.load(Ordering::Relaxed) as f64;
            let labels = format_labels(&gauge.labels);
            writeln!(&mut output, "{}{} {}", gauge.name, labels, value)?;
        }

        // Export histograms
    for histogram in registry.histograms.values() {
            writeln!(&mut output, "# HELP {} {}", histogram.name, histogram.help)?;
            writeln!(&mut output, "# TYPE {} histogram", histogram.name)?;
            let labels = format_labels(&histogram.labels);
            
            for (i, &bucket) in histogram.buckets.iter().enumerate() {
                let count = histogram.counts[i].load(Ordering::Relaxed);
                let label_suffix = if labels.is_empty() { 
                    String::new() 
                } else { 
                    format!(",{}", labels.trim_start_matches('{').trim_end_matches('}'))
                };
                writeln!(&mut output, "{}_bucket{{le=\"{}\"{}}} {}", 
                    histogram.name, bucket, label_suffix, count)?;
            }
            
            let sum = histogram.sum.load(Ordering::Relaxed) as f64;
            let count = histogram.count.load(Ordering::Relaxed);
            writeln!(&mut output, "{}_sum{} {}", histogram.name, labels, sum)?;
            writeln!(&mut output, "{}_count{} {}", histogram.name, labels, count)?;
        }

        Ok(output)
    }

    /// Get configuration
    pub fn get_config(&self) -> &MetricsConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: MetricsConfig) -> Result<()> {
        let old_enabled = self.config.enable_metrics;
        self.config = config;
        
        if old_enabled != self.config.enable_metrics {
            info!(
                enabled = self.config.enable_metrics,
                "Metrics collection toggled"
            );
        }
        
        Ok(())
    }

    /// Shutdown metrics system
    pub async fn shutdown(&self) -> Result<()> {
        let summary = self.get_summary();
        info!(
            uptime_seconds = summary.uptime_seconds,
            total_commands = summary.commands_executed,
            total_jobs = summary.total_jobs_started,
            "Metrics system shutdown"
        );
        Ok(())
    }
}

/// Format Prometheus labels
fn format_labels(labels: &HashMap<String, String>) -> String {
    if labels.is_empty() {
        return String::new();
    }
    
    let label_pairs: Vec<String> = labels.iter()
        .map(|(k, v)| format!("{k}=\"{v}\""))
        .collect();
    
    format!("{{{}}}", label_pairs.join(","))
} 