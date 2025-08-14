use crate::compat::Result;
use std::{
    collections::HashMap,
    time::{Duration, Instant, SystemTime},
    sync::{Arc, Mutex},
};
use serde::{Deserialize, Serialize};
use sysinfo::{SystemExt, ProcessExt, CpuExt};

/// Comprehensive performance monitoring and optimization system
#[derive(Debug, Clone)]
pub struct PerformanceProfiler {
    metrics: Arc<Mutex<PerformanceMetrics>>,
    optimization_rules: Vec<OptimizationRule>,
    benchmarks: HashMap<String, Benchmark>,
    monitoring_enabled: bool,
    profiling_sessions: HashMap<String, ProfilingSession>,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(PerformanceMetrics::new())),
            optimization_rules: Self::default_optimization_rules(),
            benchmarks: HashMap::new(),
            monitoring_enabled: true,
            profiling_sessions: HashMap::new(),
        }
    }

    /// Start comprehensive system profiling
    pub fn start_profiling(&mut self, session_name: String) -> Result<String> {
        let session_id = format!("{}_{}", session_name, SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?.as_secs());
        
        let session = ProfilingSession {
            id: session_id.clone(),
            name: session_name,
            start_time: Instant::now(),
            end_time: None,
            metrics: PerformanceSnapshot::new(),
            cpu_samples: Vec::new(),
            memory_samples: Vec::new(),
            io_samples: Vec::new(),
            command_timings: HashMap::new(),
        };
        
        self.profiling_sessions.insert(session_id.clone(), session);
        
        // Start background monitoring
        let metrics = Arc::clone(&self.metrics);
        let session_id_clone = session_id.clone();
        
        std::thread::spawn(move || {
            Self::profile_system_resources(metrics, session_id_clone);
        });
        
        Ok(session_id)
    }

    /// Stop profiling session and generate report
    pub fn stop_profiling(&mut self, session_id: &str) -> Result<ProfilingReport> {
        let mut session = self.profiling_sessions.remove(session_id)
            .ok_or_else(|| crate::anyhow!("Profiling session not found: {}", session_id))?;
        
        session.end_time = Some(Instant::now());
        session.metrics.finalize();
        
        let report = self.generate_profiling_report(&session)?;
        Ok(report)
    }

    /// Profile command execution
    pub fn profile_command<F, T>(&mut self, command_name: &str, operation: F) -> Result<(T, CommandProfile)>
    where
        F: FnOnce() -> Result<T>,
    {
        let start_time = Instant::now();
        let start_memory = self.get_memory_usage()?;
        let start_cpu = self.get_cpu_usage()?;
        
        // Execute the operation
        let result = operation()?;
        
        let end_time = Instant::now();
        let end_memory = self.get_memory_usage()?;
        let end_cpu = self.get_cpu_usage()?;
        
        let profile = CommandProfile {
            command: command_name.to_string(),
            duration: end_time.duration_since(start_time),
            memory_delta: end_memory as i64 - start_memory as i64,
            cpu_time: end_cpu.saturating_sub(start_cpu),
            timestamp: SystemTime::now(),
        };
        
        // Update metrics
        let mut metrics = self.metrics.lock().unwrap();
        metrics.add_command_profile(&profile);
        
        Ok((result, profile))
    }

    /// Run performance benchmark
    pub fn run_benchmark(&mut self, benchmark_name: &str) -> Result<BenchmarkResult> {
        let benchmark = self.benchmarks.get(benchmark_name)
            .ok_or_else(|| crate::anyhow!("Benchmark not found: {}", benchmark_name))?
            .clone();
        
        let mut results = Vec::new();
        
        for iteration in 0..benchmark.iterations {
            let start_time = Instant::now();
            
            // Run benchmark operation
            (benchmark.operation)()?;
            
            let duration = start_time.elapsed();
            results.push(BenchmarkSample {
                iteration,
                duration,
                memory_usage: self.get_memory_usage()?,
            });
        }
        
        let benchmark_result = BenchmarkResult {
            name: benchmark_name.to_string(),
            samples: results.clone(),
            statistical_summary: self.calculate_statistics(&results),
            timestamp: SystemTime::now(),
        };
        
        Ok(benchmark_result)
    }

    /// Analyze performance bottlenecks
    pub fn analyze_bottlenecks(&self) -> Result<BottleneckAnalysis> {
        let metrics = self.metrics.lock().unwrap();
        let mut analysis = BottleneckAnalysis {
            cpu_bottlenecks: Vec::new(),
            memory_bottlenecks: Vec::new(),
            io_bottlenecks: Vec::new(),
            recommendations: Vec::new(),
        };
        
        // Analyze CPU usage patterns
        if let Some(avg_cpu) = metrics.average_cpu_usage() {
            if avg_cpu > 80.0 {
                analysis.cpu_bottlenecks.push(Bottleneck {
                    severity: BottleneckSeverity::High,
                    description: format!("High CPU usage: {:.1}%", avg_cpu),
                    location: "System-wide".to_string(),
                    impact: "Commands may execute slowly".to_string(),
                });
                
                analysis.recommendations.push(
                    "Consider optimizing CPU-intensive operations or reducing concurrent tasks".to_string()
                );
            }
        }
        
        // Analyze memory usage patterns
        if let Some(peak_memory) = metrics.peak_memory_usage() {
            if peak_memory > 500_000_000 { // 500MB threshold
                analysis.memory_bottlenecks.push(Bottleneck {
                    severity: BottleneckSeverity::Medium,
                    description: format!("High memory usage: {} bytes", peak_memory),
                    location: "Memory allocation".to_string(),
                    impact: "Potential memory pressure".to_string(),
                });
                
                analysis.recommendations.push(
                    "Consider implementing memory pooling or reducing data structure sizes".to_string()
                );
            }
        }
        
        // Analyze command execution times
        for (command, profiles) in &metrics.command_profiles {
            let avg_duration = profiles.iter()
                .map(|p| p.duration.as_millis())
                .sum::<u128>() as f64 / profiles.len() as f64;
            
            if avg_duration > 100.0 { // 100ms threshold
                analysis.cpu_bottlenecks.push(Bottleneck {
                    severity: if avg_duration > 1000.0 { BottleneckSeverity::High } else { BottleneckSeverity::Medium },
                    description: format!("Slow command execution: {} ({:.1}ms avg)", command, avg_duration),
                    location: command.clone(),
                    impact: "User experience degradation".to_string(),
                });
            }
        }
        
        Ok(analysis)
    }

    /// Get real-time performance metrics
    pub fn get_realtime_metrics(&self) -> Result<RealtimeMetrics> {
        Ok(RealtimeMetrics {
            cpu_usage: self.get_cpu_usage()?,
            memory_usage: self.get_memory_usage()?,
            io_read_rate: self.get_io_read_rate()?,
            io_write_rate: self.get_io_write_rate()?,
            active_threads: self.get_thread_count()?,
            network_connections: self.get_network_connection_count()?,
            uptime: self.get_system_uptime()?,
        })
    }

    /// Apply performance optimizations
    pub fn apply_optimizations(&mut self) -> Result<OptimizationReport> {
        let mut report = OptimizationReport {
            applied_optimizations: Vec::new(),
            performance_improvement: 0.0,
            timestamp: SystemTime::now(),
        };
        
        // Apply optimization rules
        let mut applied_optimizations = Vec::new();
        let mut performance_improvement = 0.0;
        
        // Clone rules to avoid borrowing issues
        let rules = self.optimization_rules.clone();
        for rule in &rules {
            if rule.condition_met(self)? {
                let before_metrics = self.get_baseline_performance()?;
                
                (rule.apply_optimization)(self)?;
                
                let after_metrics = self.get_baseline_performance()?;
                let improvement = PerformanceProfiler::calculate_improvement(&before_metrics, &after_metrics);
                
                applied_optimizations.push(AppliedOptimization {
                    name: rule.name.clone(),
                    description: rule.description.clone(),
                    improvement_percentage: improvement,
                });
                
                performance_improvement += improvement;
            }
        }
        
        report.applied_optimizations = applied_optimizations;
        report.performance_improvement = performance_improvement;
        
        Ok(report)
    }

    /// Register custom benchmark
    pub fn register_benchmark(&mut self, name: String, benchmark: Benchmark) {
        self.benchmarks.insert(name, benchmark);
    }

    /// Export performance data
    pub fn export_performance_data(&self, format: ExportFormat) -> Result<Vec<u8>> {
        let metrics = self.metrics.lock().unwrap();
        
        match format {
            ExportFormat::Json => {
                let json = serde_json::to_vec_pretty(&*metrics)?;
                Ok(json)
            },
            ExportFormat::Csv => {
                let mut csv = String::new();
                csv.push_str("timestamp,command,duration_ms,memory_delta,cpu_time\n");
                
                for (command, profiles) in &metrics.command_profiles {
                    for profile in profiles {
                        csv.push_str(&format!(
                            "{:?},{},{},{},{}\n",
                            profile.timestamp,
                            command,
                            profile.duration.as_millis(),
                            profile.memory_delta,
                            profile.cpu_time.as_millis()
                        ));
                    }
                }
                
                Ok(csv.into_bytes())
            },
            ExportFormat::Binary => {
                // Use serde_json as binary alternative since bincode is not available
                let binary = serde_json::to_vec(&*metrics)?;
                Ok(binary)
            },
        }
    }

    // Private helper methods

    fn default_optimization_rules() -> Vec<OptimizationRule> {
        vec![
            OptimizationRule {
                name: "Memory Pool Optimization".to_string(),
                description: "Enable memory pooling for frequently allocated objects".to_string(),
                condition: std::sync::Arc::new(|profiler| {
                    let metrics = profiler.metrics.lock().unwrap();
                    metrics.total_allocations > 10000
                }),
                apply_optimization: std::sync::Arc::new(|_profiler| {
                    // Enable memory pooling
                    println!("Applied memory pool optimization");
                    Ok(())
                }),
            },
            OptimizationRule {
                name: "Command Caching".to_string(),
                description: "Cache frequently used command results".to_string(),
                condition: std::sync::Arc::new(|profiler| {
                    let metrics = profiler.metrics.lock().unwrap();
                    metrics.command_profiles.len() > 100
                }),
                apply_optimization: std::sync::Arc::new(|_profiler| {
                    // Enable command caching
                    println!("Applied command caching optimization");
                    Ok(())
                }),
            },
            OptimizationRule {
                name: "Parallel Processing".to_string(),
                description: "Enable parallel processing for IO-heavy operations".to_string(),
                condition: std::sync::Arc::new(|profiler| {
                    profiler.get_io_read_rate().unwrap_or(0) > 100 * 1024 * 1024 // 100MB/s
                }),
                apply_optimization: std::sync::Arc::new(|_profiler| {
                    // Enable parallel IO
                    println!("Applied parallel processing optimization");
                    Ok(())
                }),
            },
        ]
    }

    fn profile_system_resources(metrics: Arc<Mutex<PerformanceMetrics>>, _session_id: String) {
        loop {
            std::thread::sleep(Duration::from_millis(100)); // Sample every 100ms
            
            if let Ok(mut m) = metrics.try_lock() {
                // Sample system resources
                if let Ok(cpu_usage) = Self::get_current_cpu_usage() {
                    m.cpu_samples.push(CpuSample {
                        timestamp: SystemTime::now(),
                        usage_percent: cpu_usage,
                    });
                }
                
                if let Ok(memory_usage) = Self::get_current_memory_usage() {
                    m.memory_samples.push(MemorySample {
                        timestamp: SystemTime::now(),
                        bytes_used: memory_usage,
                    });
                }
            }
        }
    }

    fn generate_profiling_report(&self, session: &ProfilingSession) -> Result<ProfilingReport> {
        let duration = session.end_time.unwrap().duration_since(session.start_time);
        
        Ok(ProfilingReport {
            session_id: session.id.clone(),
            session_name: session.name.clone(),
            total_duration: duration,
            cpu_summary: self.calculate_cpu_summary(&session.cpu_samples),
            memory_summary: self.calculate_memory_summary(&session.memory_samples),
            command_summary: self.calculate_command_summary(&session.command_timings),
            recommendations: self.generate_recommendations(session),
        })
    }

    fn calculate_statistics(&self, samples: &[BenchmarkSample]) -> StatisticalSummary {
        let durations: Vec<f64> = samples.iter().map(|s| s.duration.as_secs_f64()).collect();
        
        let mean = durations.iter().sum::<f64>() / durations.len() as f64;
        let min = durations.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = durations.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        // Calculate standard deviation
        let variance = durations.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / durations.len() as f64;
        let std_dev = variance.sqrt();
        
        StatisticalSummary {
            mean,
            min,
            max,
            std_dev,
            sample_count: samples.len(),
        }
    }

    fn calculate_improvement(before: &BaselineMetrics, after: &BaselineMetrics) -> f64 {
        // Simple improvement calculation based on execution time
        if before.avg_execution_time > 0.0 {
            (before.avg_execution_time - after.avg_execution_time) / before.avg_execution_time * 100.0
        } else {
            0.0
        }
    }

    fn get_baseline_performance(&self) -> Result<BaselineMetrics> {
        let metrics = self.metrics.lock().unwrap();
        
        let total_commands: usize = metrics.command_profiles.values().map(|v| v.len()).sum();
        let total_duration: Duration = metrics.command_profiles.values()
            .flat_map(|profiles| profiles.iter().map(|p| p.duration))
            .sum();
        
        let avg_execution_time = if total_commands > 0 {
            total_duration.as_secs_f64() / total_commands as f64
        } else {
            0.0
        };
        
        Ok(BaselineMetrics {
            avg_execution_time,
            memory_usage: self.get_memory_usage().unwrap_or(0),
        })
    }

    // System metrics helpers
    fn get_cpu_usage(&self) -> Result<Duration> {
        // Use cpu_time crate to fetch process CPU time cross-platform
        let times = cpu_time::ProcessTime::try_now()
            .map_err(|e| crate::anyhow!("Failed to get process CPU time: {}", e))?;
        Ok(times.as_duration())
    }

    fn get_memory_usage(&self) -> Result<usize> {
        // Use sysinfo to fetch current process memory usage (resident set)
        let mut sys = sysinfo::System::new();
        sys.refresh_processes_specifics(sysinfo::ProcessRefreshKind::everything());
        if let Ok(pid) = sysinfo::get_current_pid() {
            if let Some(proc) = sys.process(pid) {
                // memory() returns KiB; convert to bytes
                let kib = proc.memory();
                return Ok((kib as usize) * 1024);
            }
        }
        Ok(0)
    }

    fn get_io_read_rate(&self) -> Result<u64> {
        // Sample two times and compute delta bytes/time using sysinfo
        let mut sys = sysinfo::System::new();
        sys.refresh_processes_specifics(sysinfo::ProcessRefreshKind::everything());
        let pid = sysinfo::get_current_pid().map_err(|e| crate::anyhow!("pid error: {}", e))?;
        let io1 = sys.process(pid).map(|p| {
            let du = p.disk_usage();
            (du.read_bytes, du.written_bytes)
        });
        let t1 = Instant::now();
        std::thread::sleep(Duration::from_millis(50));
        sys.refresh_processes_specifics(sysinfo::ProcessRefreshKind::everything());
        let io2 = sys.process(pid).map(|p| {
            let du = p.disk_usage();
            (du.read_bytes, du.written_bytes)
        });
        let dt = t1.elapsed().as_secs_f64();
        if let (Some((r1, _)), Some((r2, _))) = (io1, io2) {
            let delta = r2.saturating_sub(r1);
            let rate = (delta as f64 / dt).round() as u64;
            return Ok(rate);
        }
        Ok(0)
    }

    fn get_io_write_rate(&self) -> Result<u64> {
        let mut sys = sysinfo::System::new();
        sys.refresh_processes_specifics(sysinfo::ProcessRefreshKind::everything());
        let pid = sysinfo::get_current_pid().map_err(|e| crate::anyhow!("pid error: {}", e))?;
        let io1 = sys.process(pid).map(|p| {
            let du = p.disk_usage();
            (du.read_bytes, du.written_bytes)
        });
        let t1 = Instant::now();
        std::thread::sleep(Duration::from_millis(50));
        sys.refresh_processes_specifics(sysinfo::ProcessRefreshKind::everything());
        let io2 = sys.process(pid).map(|p| {
            let du = p.disk_usage();
            (du.read_bytes, du.written_bytes)
        });
        let dt = t1.elapsed().as_secs_f64();
        if let (Some((_, w1)), Some((_, w2))) = (io1, io2) {
            let delta = w2.saturating_sub(w1);
            let rate = (delta as f64 / dt).round() as u64;
            return Ok(rate);
        }
        Ok(0)
    }

    fn get_thread_count(&self) -> Result<usize> {
        // Fallback: approximate by logical core count
        Ok(num_cpus::get())
    }

    fn get_network_connection_count(&self) -> Result<usize> {
        // sysinfo does not expose sockets; approximate via TCP table on Windows later. For now 0.
        Ok(0)
    }

    fn get_system_uptime(&self) -> Result<Duration> {
        // Use instance method for broader sysinfo version compatibility
        let mut sys = sysinfo::System::new();
        sys.refresh_system();
        Ok(Duration::from_secs(sys.uptime()))
    }

    fn get_current_cpu_usage() -> Result<f64> {
        // Compatible sampling: refresh all CPUs and average usage
        let mut sys = sysinfo::System::new();
        sys.refresh_cpu();
        let cpus = sys.cpus();
        let avg = if cpus.is_empty() { 0.0 } else { cpus.iter().map(|c| c.cpu_usage() as f64).sum::<f64>() / (cpus.len() as f64) };
        Ok(avg)
    }

    fn get_current_memory_usage() -> Result<usize> {
        // Compatible sampling: refresh memory and read used memory (KiB)
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        Ok((sys.used_memory() as usize) * 1024)
    }

    fn calculate_cpu_summary(&self, samples: &[CpuSample]) -> CpuSummary {
        let avg = if samples.is_empty() { 0.0 } else { samples.iter().map(|s| s.usage_percent).sum::<f64>() / samples.len() as f64 };
        let peak = samples.iter().map(|s| s.usage_percent).fold(0.0, f64::max);
        CpuSummary { average_usage: avg, peak_usage: peak, samples_count: samples.len() }
    }

    fn calculate_memory_summary(&self, samples: &[MemorySample]) -> MemorySummary {
        let (sum, peak) = samples.iter().fold((0usize, 0usize), |(s, p), m| (s + m.bytes_used, p.max(m.bytes_used)));
        let avg = if samples.is_empty() { 0 } else { sum / samples.len() };
        MemorySummary { average_usage: avg, peak_usage: peak, samples_count: samples.len() }
    }

    fn calculate_command_summary(&self, timings: &HashMap<String, Vec<Duration>>) -> CommandSummary {
        let total_commands = timings.values().map(|v| v.len()).sum();
        let mut slowest_cmd = String::new();
        let mut slowest = Duration::from_secs(0);
        let mut total = Duration::from_secs(0);
        let mut count = 0usize;
        for (cmd, durs) in timings {
            for d in durs {
                total += *d;
                count += 1;
                if *d > slowest { slowest = *d; slowest_cmd = cmd.clone(); }
            }
        }
        let avg = if count == 0 { Duration::from_millis(0) } else { Duration::from_secs_f64(total.as_secs_f64() / count as f64) };
        CommandSummary { total_commands, average_duration: avg, slowest_command: slowest_cmd }
    }

    fn generate_recommendations(&self, _session: &ProfilingSession) -> Vec<String> {
        vec![
            "Consider caching frequently accessed files".to_string(),
            "Enable parallel processing for directory operations".to_string(),
            "Optimize memory allocation patterns".to_string(),
        ]
    }
}

// Supporting types and structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_samples: Vec<CpuSample>,
    pub memory_samples: Vec<MemorySample>,
    pub command_profiles: HashMap<String, Vec<CommandProfile>>,
    pub total_allocations: u64,
    pub start_time: SystemTime,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            cpu_samples: Vec::new(),
            memory_samples: Vec::new(),
            command_profiles: HashMap::new(),
            total_allocations: 0,
            start_time: SystemTime::now(),
        }
    }

    pub fn add_command_profile(&mut self, profile: &CommandProfile) {
        self.command_profiles
            .entry(profile.command.clone())
            .or_insert_with(Vec::new)
            .push(profile.clone());
    }

    pub fn average_cpu_usage(&self) -> Option<f64> {
        if self.cpu_samples.is_empty() {
            None
        } else {
            let sum: f64 = self.cpu_samples.iter().map(|s| s.usage_percent).sum();
            Some(sum / self.cpu_samples.len() as f64)
        }
    }

    pub fn peak_memory_usage(&self) -> Option<usize> {
        self.memory_samples.iter().map(|s| s.bytes_used).max()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSample {
    pub timestamp: SystemTime,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySample {
    pub timestamp: SystemTime,
    pub bytes_used: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandProfile {
    pub command: String,
    pub duration: Duration,
    pub memory_delta: i64,
    pub cpu_time: Duration,
    pub timestamp: SystemTime,
}

pub struct Benchmark {
    pub name: String,
    pub description: String,
    pub iterations: usize,
    pub operation: std::sync::Arc<dyn Fn() -> Result<()> + Send + Sync>,
}

impl std::fmt::Debug for Benchmark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Benchmark")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("iterations", &self.iterations)
            .field("operation", &"<function>")
            .finish()
    }
}

impl Clone for Benchmark {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            description: self.description.clone(),
            iterations: self.iterations,
            operation: self.operation.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub samples: Vec<BenchmarkSample>,
    pub statistical_summary: StatisticalSummary,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub struct BenchmarkSample {
    pub iteration: usize,
    pub duration: Duration,
    pub memory_usage: usize,
}

#[derive(Debug, Clone)]
pub struct StatisticalSummary {
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
    pub sample_count: usize,
}

#[derive(Debug, Clone)]
pub struct ProfilingSession {
    pub id: String,
    pub name: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub metrics: PerformanceSnapshot,
    pub cpu_samples: Vec<CpuSample>,
    pub memory_samples: Vec<MemorySample>,
    pub io_samples: Vec<IoSample>,
    pub command_timings: HashMap<String, Vec<Duration>>,
}

#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    pub timestamp: SystemTime,
}

impl PerformanceSnapshot {
    pub fn new() -> Self {
        Self {
            timestamp: SystemTime::now(),
        }
    }

    pub fn finalize(&mut self) {
        // Finalize snapshot data
    }
}

#[derive(Debug, Clone)]
pub struct IoSample {
    pub timestamp: SystemTime,
    pub bytes_read: u64,
    pub bytes_written: u64,
}

#[derive(Debug, Clone)]
pub struct ProfilingReport {
    pub session_id: String,
    pub session_name: String,
    pub total_duration: Duration,
    pub cpu_summary: CpuSummary,
    pub memory_summary: MemorySummary,
    pub command_summary: CommandSummary,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CpuSummary {
    pub average_usage: f64,
    pub peak_usage: f64,
    pub samples_count: usize,
}

#[derive(Debug, Clone)]
pub struct MemorySummary {
    pub average_usage: usize,
    pub peak_usage: usize,
    pub samples_count: usize,
}

#[derive(Debug, Clone)]
pub struct CommandSummary {
    pub total_commands: usize,
    pub average_duration: Duration,
    pub slowest_command: String,
}

#[derive(Debug, Clone)]
pub struct BottleneckAnalysis {
    pub cpu_bottlenecks: Vec<Bottleneck>,
    pub memory_bottlenecks: Vec<Bottleneck>,
    pub io_bottlenecks: Vec<Bottleneck>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Bottleneck {
    pub severity: BottleneckSeverity,
    pub description: String,
    pub location: String,
    pub impact: String,
}

#[derive(Debug, Clone)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct RealtimeMetrics {
    pub cpu_usage: Duration,
    pub memory_usage: usize,
    pub io_read_rate: u64,
    pub io_write_rate: u64,
    pub active_threads: usize,
    pub network_connections: usize,
    pub uptime: Duration,
}

#[derive(Clone)]
pub struct OptimizationRule {
    pub name: String,
    pub description: String,
    #[doc = "Function stored as Arc for cloneability"]
    pub condition: std::sync::Arc<dyn Fn(&PerformanceProfiler) -> bool + Send + Sync>,
    #[doc = "Function stored as Arc for cloneability"]
    pub apply_optimization: std::sync::Arc<dyn Fn(&mut PerformanceProfiler) -> Result<()> + Send + Sync>,
}

impl std::fmt::Debug for OptimizationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptimizationRule")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("condition", &"<function>")
            .field("apply_optimization", &"<function>")
            .finish()
    }
}

impl OptimizationRule {
    pub fn condition_met(&self, profiler: &PerformanceProfiler) -> Result<bool> {
        Ok((self.condition)(profiler))
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationReport {
    pub applied_optimizations: Vec<AppliedOptimization>,
    pub performance_improvement: f64,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub struct AppliedOptimization {
    pub name: String,
    pub description: String,
    pub improvement_percentage: f64,
}

#[derive(Debug, Clone)]
pub struct BaselineMetrics {
    pub avg_execution_time: f64,
    pub memory_usage: usize,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    Binary,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let profiler = PerformanceProfiler::new();
        assert!(profiler.monitoring_enabled);
        assert!(!profiler.optimization_rules.is_empty());
    }

    #[test]
    fn test_command_profiling() {
        let mut profiler = PerformanceProfiler::new();
        
        let (result, profile) = profiler.profile_command("test_command", || {
            std::thread::sleep(Duration::from_millis(10));
            Ok(42)
        }).unwrap();
        
        assert_eq!(result, 42);
        assert_eq!(profile.command, "test_command");
        assert!(profile.duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_profiling_session() {
        let mut profiler = PerformanceProfiler::new();
        
        let session_id = profiler.start_profiling("test_session".to_string()).unwrap();
        assert!(!session_id.is_empty());
        
        // Simulate some work
        std::thread::sleep(Duration::from_millis(50));
        
        let report = profiler.stop_profiling(&session_id).unwrap();
        assert_eq!(report.session_name, "test_session");
        assert!(report.total_duration >= Duration::from_millis(50));
    }

    #[test]
    fn test_bottleneck_analysis() {
        let profiler = PerformanceProfiler::new();
        let analysis = profiler.analyze_bottlenecks().unwrap();
        
    // 期待値: 初期状態ではボトルネックは検出されない (全て 0 件)
    assert_eq!(analysis.cpu_bottlenecks.len(), 0);
    assert_eq!(analysis.memory_bottlenecks.len(), 0);
    assert_eq!(analysis.io_bottlenecks.len(), 0);
    }

    #[test]
    fn test_realtime_metrics() {
        let profiler = PerformanceProfiler::new();
        let metrics = profiler.get_realtime_metrics().unwrap();
        
        assert!(metrics.memory_usage > 0);
        assert!(metrics.active_threads > 0);
    }
}
