//! Performance optimization system for NexusShell
//!
//! This module provides comprehensive performance optimization capabilities
//! targeting 10× performance improvements through multiple strategies:
//! - Startup time optimization
//! - Memory management optimization  
//! - I/O operation optimization
//! - CPU-intensive operation optimization
//! - Caching strategies
//! - Lazy loading
//! - Zero-copy operations
//! - SIMD optimizations where applicable

use crate::nxsh_log_info;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hint::black_box,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// Performance optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable startup time optimization
    pub optimize_startup: bool,
    /// Enable memory optimization
    pub optimize_memory: bool,
    /// Enable I/O optimization
    pub optimize_io: bool,
    /// Enable CPU optimization
    pub optimize_cpu: bool,
    /// Enable caching
    pub enable_caching: bool,
    /// Enable lazy loading
    pub enable_lazy_loading: bool,
    /// Enable zero-copy operations
    pub enable_zero_copy: bool,
    /// Enable SIMD optimizations
    pub enable_simd: bool,
    /// Maximum cache size in bytes
    pub max_cache_size: u64,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Memory pool initial size
    pub memory_pool_size: usize,
    /// I/O buffer size
    pub io_buffer_size: usize,
    /// Number of worker threads
    pub worker_threads: usize,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            optimize_startup: true,
            optimize_memory: true,
            optimize_io: true,
            optimize_cpu: true,
            enable_caching: true,
            enable_lazy_loading: true,
            enable_zero_copy: true,
            enable_simd: cfg!(target_feature = "sse2"),
            max_cache_size: 256 * 1024 * 1024, // 256MB
            cache_ttl_seconds: 3600,           // 1 hour
            memory_pool_size: 1024 * 1024,     // 1MB
            io_buffer_size: 64 * 1024,         // 64KB
            worker_threads: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
            enable_monitoring: true,
        }
    }
}

/// Performance optimization system
pub struct PerformanceOptimizer {
    config: PerformanceConfig,
    cache: Arc<RwLock<PerformanceCache>>,
    memory_pool: Arc<RwLock<MemoryPool>>,
    metrics: PerformanceMetrics,
    startup_optimizer: StartupOptimizer,
    io_optimizer: IoOptimizer,
    cpu_optimizer: CpuOptimizer,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    pub async fn new(config: PerformanceConfig) -> crate::compat::Result<Self> {
        let cache = Arc::new(RwLock::new(PerformanceCache::new(config.max_cache_size)?));
        let memory_pool = Arc::new(RwLock::new(MemoryPool::new(config.memory_pool_size)?));
        let metrics = PerformanceMetrics::new();
        let startup_optimizer = StartupOptimizer::new(&config)?;
        let io_optimizer = IoOptimizer::new(&config)?;
        let cpu_optimizer = CpuOptimizer::new(&config)?;

        Ok(Self {
            config,
            cache,
            memory_pool,
            metrics,
            startup_optimizer,
            io_optimizer,
            cpu_optimizer,
        })
    }

    /// Initialize performance optimizations
    pub async fn initialize(&self) -> crate::compat::Result<()> {
        nxsh_log_info!("Initializing performance optimizations");

        if self.config.optimize_startup {
            self.startup_optimizer.initialize().await?;
        }

        if self.config.optimize_io {
            self.io_optimizer.initialize().await?;
        }

        if self.config.optimize_cpu {
            self.cpu_optimizer.initialize()?;
        }

        nxsh_log_info!("Performance optimizations initialized successfully");
        Ok(())
    }

    /// Optimize a function call with caching
    pub async fn cached_operation<F, T, K>(&self, key: K, operation: F) -> crate::compat::Result<T>
    where
        F: std::future::Future<Output = crate::compat::Result<T>>,
        T: Clone + Send + Sync + 'static,
        K: Into<String>,
    {
        let cache_key = key.into();

        if self.config.enable_caching {
            // Try to get from cache first
            {
                let cache = self.cache.read().await;
                if let Some(cached_value) = cache.get::<T>(&cache_key) {
                    self.metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
                    return Ok(cached_value);
                }
            }

            self.metrics.cache_misses.fetch_add(1, Ordering::Relaxed);
        }

        // Execute operation
        let start_time = Instant::now();
        let result = operation.await?;
        let duration = start_time.elapsed();

        if self.config.enable_caching {
            // Store in cache
            let mut cache = self.cache.write().await;
            cache.set(cache_key, result.clone(), duration)?;
        }

        self.metrics.record_operation_time(duration);
        Ok(result)
    }

    /// Optimize memory allocation using pool
    pub async fn allocate_optimized(&self, size: usize) -> crate::compat::Result<Vec<u8>> {
        if self.config.optimize_memory {
            let mut pool = self.memory_pool.write().await;
            pool.allocate(size)
        } else {
            Ok(vec![0; size])
        }
    }

    /// Optimize I/O operations
    pub async fn optimized_read(&self, path: &std::path::Path) -> crate::compat::Result<Vec<u8>> {
        if self.config.optimize_io {
            self.io_optimizer.read_optimized(path).await
        } else {
            Ok(tokio::fs::read(path).await?)
        }
    }

    /// Optimize I/O write operations
    pub async fn optimized_write(
        &self,
        path: &std::path::Path,
        data: &[u8],
    ) -> crate::compat::Result<()> {
        if self.config.optimize_io {
            self.io_optimizer.write_optimized(path, data).await
        } else {
            tokio::fs::write(path, data).await?;
            Ok(())
        }
    }

    /// Optimize CPU-intensive operations
    pub fn optimized_compute<F, T>(&self, operation: F) -> crate::compat::Result<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        if self.config.optimize_cpu {
            self.cpu_optimizer.compute_optimized(operation)
        } else {
            Ok(operation())
        }
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> PerformanceReport {
        self.metrics.generate_report()
    }

    /// Force garbage collection and cleanup
    pub async fn cleanup(&self) -> crate::compat::Result<()> {
        {
            let mut cache = self.cache.write().await;
            cache.cleanup()?;
        }

        {
            let mut pool = self.memory_pool.write().await;
            pool.cleanup()?;
        }

        nxsh_log_info!("Performance optimization cleanup completed");
        Ok(())
    }
}

/// High-performance caching system
struct PerformanceCache {
    data: HashMap<String, CacheEntry>,
    max_size: u64,
    current_size: u64,
    ttl: Duration,
}

impl PerformanceCache {
    fn new(max_size: u64) -> crate::compat::Result<Self> {
        Ok(Self {
            data: HashMap::new(),
            max_size,
            current_size: 0,
            ttl: Duration::from_secs(3600),
        })
    }

    fn get<T: Clone + 'static>(&self, key: &str) -> Option<T> {
        if let Some(entry) = self.data.get(key) {
            if entry.is_valid() {
                if let Some(value) = entry.value.downcast_ref::<T>() {
                    return Some(value.clone());
                }
            }
        }
        None
    }

    fn set<T: Send + Sync + 'static>(
        &mut self,
        key: String,
        value: T,
        _cost: Duration,
    ) -> crate::compat::Result<()> {
        let size_estimate = std::mem::size_of::<T>() as u64;

        // Evict if necessary
        while self.current_size + size_estimate > self.max_size {
            if !self.evict_oldest()? {
                break; // No more entries to evict
            }
        }

        let entry = CacheEntry::new(Box::new(value), self.ttl, size_estimate);

        if let Some(old_entry) = self.data.insert(key, entry) {
            self.current_size -= old_entry.size;
        }

        self.current_size += size_estimate;
        Ok(())
    }

    fn evict_oldest(&mut self) -> crate::compat::Result<bool> {
        let oldest_key = self
            .data
            .iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(key, _)| key.clone());

        if let Some(key) = oldest_key {
            if let Some(entry) = self.data.remove(&key) {
                self.current_size -= entry.size;
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn cleanup(&mut self) -> crate::compat::Result<()> {
        let expired_keys: Vec<String> = self
            .data
            .iter()
            .filter(|(_, entry)| !entry.is_valid())
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            if let Some(entry) = self.data.remove(&key) {
                self.current_size -= entry.size;
            }
        }

        Ok(())
    }
}

/// Cache entry with TTL and type erasure
struct CacheEntry {
    value: Box<dyn std::any::Any + Send + Sync>,
    expires_at: Instant,
    created_at: Instant,
    size: u64,
}

impl CacheEntry {
    fn new(value: Box<dyn std::any::Any + Send + Sync>, ttl: Duration, size: u64) -> Self {
        let now = Instant::now();
        Self {
            value,
            expires_at: now + ttl,
            created_at: now,
            size,
        }
    }

    fn is_valid(&self) -> bool {
        Instant::now() < self.expires_at
    }
}

/// Memory pool for optimized allocations
#[allow(dead_code)]
struct MemoryPool {
    pools: HashMap<usize, Vec<Vec<u8>>>,
    total_size: usize,
    max_size: usize,
}

impl MemoryPool {
    fn new(max_size: usize) -> crate::compat::Result<Self> {
        Ok(Self {
            pools: HashMap::new(),
            total_size: 0,
            max_size,
        })
    }

    fn allocate(&mut self, size: usize) -> crate::compat::Result<Vec<u8>> {
        // Round up to next power of 2 for better pooling
        let pool_size = size.next_power_of_two();

        if let Some(pool) = self.pools.get_mut(&pool_size) {
            if let Some(mut buffer) = pool.pop() {
                buffer.clear();
                buffer.resize(size, 0);
                self.total_size -= pool_size;
                return Ok(buffer);
            }
        }

        // Allocate new buffer
        Ok(vec![0; size])
    }

    #[allow(dead_code)]
    fn deallocate(&mut self, buffer: Vec<u8>) -> crate::compat::Result<()> {
        let capacity = buffer.capacity();
        let pool_size = capacity.next_power_of_two();

        // Only pool if we have space
        if self.total_size + pool_size <= self.max_size {
            let pool = self.pools.entry(pool_size).or_default();
            pool.push(buffer);
            self.total_size += pool_size;
        }

        Ok(())
    }

    fn cleanup(&mut self) -> crate::compat::Result<()> {
        self.pools.clear();
        self.total_size = 0;
        Ok(())
    }
}

/// Startup time optimization
#[allow(dead_code)]
struct StartupOptimizer {
    preloaded_modules: Arc<RwLock<HashMap<String, Box<dyn std::any::Any + Send + Sync>>>>,
    initialization_order: Vec<String>,
}

impl StartupOptimizer {
    fn new(_config: &PerformanceConfig) -> crate::compat::Result<Self> {
        Ok(Self {
            preloaded_modules: Arc::new(RwLock::new(HashMap::new())),
            initialization_order: vec![
                "core".to_string(),
                "parser".to_string(),
                "executor".to_string(),
                "builtins".to_string(),
                "ui".to_string(),
            ],
        })
    }

    async fn initialize(&self) -> crate::compat::Result<()> {
        nxsh_log_info!("Optimizing startup time");

        // Preload critical modules in optimal order
        for module in &self.initialization_order {
            self.preload_module(module).await?;
        }

        Ok(())
    }

    /// Preload a specified module to improve startup performance
    ///
    /// This asynchronous function simulates module preloading by performing
    /// initialization tasks that can be done ahead of time. The actual
    /// implementation would contain module-specific initialization logic.
    ///
    /// # Arguments
    /// * `_module_name` - Name of the module to preload (currently unused in simulation)
    ///
    /// # Returns
    /// Result indicating success or failure of the preload operation
    async fn preload_module(&self, _module_name: &str) -> crate::compat::Result<()> {
        // Simulate module preloading with async yield
        let start_time = Instant::now();

        // This would contain actual module initialization logic
        // For now, we yield to allow other tasks to run
        tokio::task::yield_now().await;

        let _duration = start_time.elapsed();
        // Note: Logging removed to avoid unused variable warning
        // In production, this would log the actual module loading time

        Ok(())
    }
}

/// I/O operation optimization
struct IoOptimizer {
    buffer_pool: Arc<RwLock<Vec<Vec<u8>>>>,
    buffer_size: usize,
}

impl IoOptimizer {
    fn new(config: &PerformanceConfig) -> crate::compat::Result<Self> {
        Ok(Self {
            buffer_pool: Arc::new(RwLock::new(Vec::new())),
            buffer_size: config.io_buffer_size,
        })
    }

    async fn initialize(&self) -> crate::compat::Result<()> {
        // Pre-allocate I/O buffers
        let mut pool = self.buffer_pool.write().await;
        for _ in 0..10 {
            pool.push(vec![0; self.buffer_size]);
        }
        Ok(())
    }

    async fn read_optimized(&self, path: &std::path::Path) -> crate::compat::Result<Vec<u8>> {
        // Adaptive I/O strategy based on file size analysis
        let mut file = tokio::fs::File::open(path).await?;
        let metadata = file.metadata().await?;
        let file_size = metadata.len() as usize;

        if file_size <= 4096 {
            // Very small files - direct read without buffering overhead (faster than buffered)
            let mut buffer = vec![0; file_size];
            tokio::io::AsyncReadExt::read_exact(&mut file, &mut buffer).await?;
            Ok(buffer)
        } else if file_size <= 64 * 1024 {
            // Medium files - use optimal buffer size based on benchmarks
            let optimal_buffer_size = std::cmp::min(file_size, 16384);
            let mut result = Vec::with_capacity(file_size);
            let mut buffer = vec![0; optimal_buffer_size];

            loop {
                let bytes_read = tokio::io::AsyncReadExt::read(&mut file, &mut buffer).await?;
                if bytes_read == 0 {
                    break;
                }
                result.extend_from_slice(&buffer[..bytes_read]);
            }
            Ok(result)
        } else {
            // Large files - use large buffers for maximum throughput
            let mut result = Vec::with_capacity(file_size);
            let mut buffer = vec![0; 32768]; // Larger buffer for big files

            loop {
                let bytes_read = tokio::io::AsyncReadExt::read(&mut file, &mut buffer).await?;
                if bytes_read == 0 {
                    break;
                }
                result.extend_from_slice(&buffer[..bytes_read]);
            }
            Ok(result)
        }
    }

    async fn write_optimized(
        &self,
        path: &std::path::Path,
        data: &[u8],
    ) -> crate::compat::Result<()> {
        if data.len() <= self.buffer_size {
            // Small write - direct
            tokio::fs::write(path, data).await?;
        } else {
            // Large write - chunked
            let mut file = tokio::fs::File::create(path).await?;

            for chunk in data.chunks(self.buffer_size) {
                tokio::io::AsyncWriteExt::write_all(&mut file, chunk).await?;
            }

            tokio::io::AsyncWriteExt::flush(&mut file).await?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    async fn get_buffer(&self) -> Vec<u8> {
        let mut pool = self.buffer_pool.write().await;
        pool.pop().unwrap_or_else(|| vec![0; self.buffer_size])
    }

    #[allow(dead_code)]
    async fn return_buffer(&self, buffer: Vec<u8>) {
        let mut pool = self.buffer_pool.write().await;
        if pool.len() < 20 {
            // Limit pool size
            pool.push(buffer);
        }
    }
}

/// CPU optimization system with parallel processing and SIMD acceleration
#[allow(dead_code)]
struct CpuOptimizer {
    thread_pool: Option<tokio::runtime::Handle>,
    worker_threads: usize,
    simd_optimizer: crate::simd_optimization::CpuOptimizer,
    #[cfg(feature = "parallel")]
    rayon_pool: Option<rayon::ThreadPool>,
}

#[allow(dead_code)]
impl CpuOptimizer {
    fn new(config: &PerformanceConfig) -> crate::compat::Result<Self> {
        #[cfg(feature = "parallel")]
        let rayon_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(config.worker_threads)
            .thread_name(|i| format!("nxsh-worker-{i}"))
            .build()
            .ok();

        Ok(Self {
            thread_pool: None,
            worker_threads: config.worker_threads,
            simd_optimizer: crate::simd_optimization::CpuOptimizer::new(),
            #[cfg(feature = "parallel")]
            rayon_pool,
        })
    }

    fn initialize(&self) -> crate::compat::Result<()> {
        // Initialize thread pool would go here
        let _opt_level = self.simd_optimizer.optimization_level();
        let _optimal_buffer = self.simd_optimizer.optimal_buffer_size();
        nxsh_log_info!("CPU optimizer initialized with {} worker threads, SIMD level: {}, optimal buffer: {}KB", 
                      self.worker_threads, _opt_level, _optimal_buffer / 1024);
        #[cfg(feature = "parallel")]
        if self.rayon_pool.is_some() {
            nxsh_log_info!("Rayon thread pool initialized for parallel processing");
        }
        Ok(())
    }

    fn compute_optimized<F, T>(&self, operation: F) -> crate::compat::Result<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        // For CPU-intensive operations, we would use the thread pool
        // For now, just execute directly with some optimization hints
        let result = black_box(operation());
        Ok(result)
    }

    /// Parallel map operation for data processing
    #[cfg(feature = "parallel")]
    fn parallel_map<T, U, F>(&self, data: Vec<T>, mapper: F) -> Vec<U>
    where
        T: Send,
        U: Send,
        F: Fn(T) -> U + Send + Sync,
    {
        if let Some(pool) = &self.rayon_pool {
            pool.install(|| {
                use rayon::prelude::*;
                data.into_par_iter().map(mapper).collect()
            })
        } else {
            data.into_iter().map(mapper).collect()
        }
    }

    #[cfg(not(feature = "parallel"))]
    fn parallel_map<T, U, F>(&self, data: Vec<T>, mapper: F) -> Vec<U>
    where
        T: Send,
        U: Send,
        F: Fn(T) -> U + Send + Sync,
    {
        data.into_iter().map(mapper).collect()
    }

    /// Parallel filter operation
    #[cfg(feature = "parallel")]
    fn parallel_filter<T, F>(&self, data: Vec<T>, predicate: F) -> Vec<T>
    where
        T: Send,
        F: Fn(&T) -> bool + Send + Sync,
    {
        if let Some(pool) = &self.rayon_pool {
            pool.install(|| {
                use rayon::prelude::*;
                data.into_par_iter().filter(|x| predicate(x)).collect()
            })
        } else {
            data.into_iter().filter(|x| predicate(x)).collect()
        }
    }

    #[cfg(not(feature = "parallel"))]
    fn parallel_filter<T, F>(&self, data: Vec<T>, predicate: F) -> Vec<T>
    where
        T: Send,
        F: Fn(&T) -> bool + Send + Sync,
    {
        data.into_iter().filter(|x| predicate(x)).collect()
    }

    /// Parallel reduce operation with thread-safe constraints
    ///
    /// This function performs a parallel reduction operation on a vector of elements
    /// using the Rayon library for parallel processing. The operation requires that
    /// the element type T is Send + Sync + Clone for safe sharing between threads.
    ///
    /// # Type Parameters
    /// * `T` - Element type that must be Send, Sync, and Clone for thread safety
    /// * `F` - Reducer function type that combines two elements into one
    ///
    /// # Arguments
    /// * `data` - Vector of elements to reduce
    /// * `identity` - Identity element for the reduction operation
    /// * `reducer` - Function that combines two elements of type T
    ///
    /// # Returns
    /// Single element of type T representing the reduced result
    #[cfg(feature = "parallel")]
    fn parallel_reduce<T, F>(&self, data: Vec<T>, identity: T, reducer: F) -> T
    where
        T: Send + Clone + Sync,
        F: Fn(T, T) -> T + Send + Sync,
    {
        if let Some(pool) = &self.rayon_pool {
            pool.install(|| {
                use rayon::prelude::*;
                data.into_par_iter().reduce(|| identity.clone(), reducer)
            })
        } else {
            data.into_iter().fold(identity, reducer)
        }
    }

    #[cfg(not(feature = "parallel"))]
    fn parallel_reduce<T, F>(&self, data: Vec<T>, identity: T, reducer: F) -> T
    where
        T: Send + Clone,
        F: Fn(T, T) -> T + Send + Sync,
    {
        data.into_iter().fold(identity, reducer)
    }

    /// Parallel sorting for large datasets
    #[cfg(feature = "parallel")]
    fn parallel_sort<T: Send + Ord>(&self, mut data: Vec<T>) -> Vec<T> {
        if let Some(pool) = &self.rayon_pool {
            pool.install(|| {
                use rayon::prelude::*;
                data.par_sort();
            });
        } else {
            data.sort();
        }
        data
    }

    #[cfg(not(feature = "parallel"))]
    fn parallel_sort<T: Send + Ord>(&self, mut data: Vec<T>) -> Vec<T> {
        data.sort();
        data
    }

    /// Parallel search in large datasets
    #[cfg(feature = "parallel")]
    fn parallel_find<'a, T, F>(&self, data: &'a [T], predicate: F) -> Option<&'a T>
    where
        T: Send + Sync,
        F: Fn(&T) -> bool + Send + Sync,
    {
        if let Some(pool) = &self.rayon_pool {
            pool.install(|| {
                use rayon::prelude::*;
                data.par_iter().find_any(|x| predicate(x))
            })
        } else {
            data.iter().find(|x| predicate(x))
        }
    }

    #[cfg(not(feature = "parallel"))]
    fn parallel_find<'a, T, F>(&self, data: &'a [T], predicate: F) -> Option<&'a T>
    where
        T: Send + Sync,
        F: Fn(&T) -> bool + Send + Sync,
    {
        data.iter().find(|x| predicate(x))
    }
}

/// Performance metrics collection
#[derive(Debug, Default)]
struct PerformanceMetrics {
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    operations_total: AtomicU64,
    total_operation_time: AtomicU64, // microseconds
    memory_allocations: AtomicU64,
    io_operations: AtomicU64,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self::default()
    }

    fn record_operation_time(&self, duration: Duration) {
        self.operations_total.fetch_add(1, Ordering::Relaxed);
        self.total_operation_time
            .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
    }

    fn generate_report(&self) -> PerformanceReport {
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);
        let operations_total = self.operations_total.load(Ordering::Relaxed);
        let total_operation_time = self.total_operation_time.load(Ordering::Relaxed);

        let cache_hit_rate = if cache_hits + cache_misses > 0 {
            cache_hits as f64 / (cache_hits + cache_misses) as f64
        } else {
            0.0
        };

        let average_operation_time = if operations_total > 0 {
            Duration::from_micros(total_operation_time / operations_total)
        } else {
            Duration::ZERO
        };

        PerformanceReport {
            cache_hit_rate,
            cache_hits,
            cache_misses,
            operations_total,
            average_operation_time,
            memory_allocations: self.memory_allocations.load(Ordering::Relaxed),
            io_operations: self.io_operations.load(Ordering::Relaxed),
        }
    }
}

/// Performance optimization report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub cache_hit_rate: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub operations_total: u64,
    pub average_operation_time: Duration,
    pub memory_allocations: u64,
    pub io_operations: u64,
}

impl PerformanceReport {
    /// Calculate performance improvement factor
    pub fn performance_factor(&self, baseline: &PerformanceReport) -> f64 {
        if baseline.average_operation_time.is_zero() || self.average_operation_time.is_zero() {
            return 1.0;
        }

        baseline.average_operation_time.as_nanos() as f64
            / self.average_operation_time.as_nanos() as f64
    }

    /// Check if 10x performance target is met
    pub fn meets_10x_target(&self, baseline: &PerformanceReport) -> bool {
        self.performance_factor(baseline) >= 10.0
    }
}

/// SIMD-optimized operations (when available)
#[cfg(target_arch = "x86_64")]
pub mod simd {
    use std::arch::x86_64::*;

    /// SIMD-optimized string search
    #[target_feature(enable = "sse2")]
    /// # Safety
    /// Caller must ensure CPU supports SSE2 (guarded by target_feature) and that
    /// the provided slices are valid for reads of the required widths. The function
    /// performs unaligned loads but does not dereference beyond slice bounds.
    pub unsafe fn find_byte_simd(haystack: &[u8], needle: u8) -> Option<usize> {
        if haystack.len() < 16 {
            // Fall back to scalar search for small inputs
            return haystack.iter().position(|&b| b == needle);
        }

        let needle_vec = _mm_set1_epi8(needle as i8);
        let mut i = 0;

        while i + 16 <= haystack.len() {
            let chunk = _mm_loadu_si128(haystack.as_ptr().add(i) as *const __m128i);
            let cmp = _mm_cmpeq_epi8(chunk, needle_vec);
            let mask = _mm_movemask_epi8(cmp);

            if mask != 0 {
                return Some(i + mask.trailing_zeros() as usize);
            }

            i += 16;
        }

        // Check remaining bytes
        haystack[i..]
            .iter()
            .position(|&b| b == needle)
            .map(|pos| i + pos)
    }

    /// SIMD-optimized string search for patterns
    #[target_feature(enable = "sse2")]
    /// Find pattern in haystack using SIMD operations
    ///
    /// # Safety
    ///
    /// This function uses SIMD intrinsics and requires that the input slices
    /// are valid and properly aligned for optimal performance.
    pub unsafe fn find_pattern_simd(haystack: &[u8], pattern: &[u8]) -> Option<usize> {
        if pattern.is_empty() || haystack.len() < pattern.len() {
            return None;
        }

        if pattern.len() == 1 {
            return find_byte_simd(haystack, pattern[0]);
        }

        // Use first byte as initial filter
        let first_byte = pattern[0];
        let mut pos = 0;

        while let Some(found) = find_byte_simd(&haystack[pos..], first_byte) {
            let actual_pos = pos + found;
            if actual_pos + pattern.len() <= haystack.len()
                && memory_equal_simd(&haystack[actual_pos..actual_pos + pattern.len()], pattern)
            {
                return Some(actual_pos);
            }
            pos = actual_pos + 1;
        }

        None
    }

    /// SIMD-optimized memory comparison
    #[target_feature(enable = "sse2")]
    /// # Safety
    /// Caller must ensure CPU supports SSE2 and both slices are valid for the
    /// performed reads. Lengths are checked; unaligned loads are used safely within
    /// slice bounds.
    pub unsafe fn memory_equal_simd(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        if a.len() < 16 {
            return a == b;
        }

        let mut i = 0;
        while i + 16 <= a.len() {
            let chunk_a = _mm_loadu_si128(a.as_ptr().add(i) as *const __m128i);
            let chunk_b = _mm_loadu_si128(b.as_ptr().add(i) as *const __m128i);
            let cmp = _mm_cmpeq_epi8(chunk_a, chunk_b);
            let mask = _mm_movemask_epi8(cmp);

            if mask != 0xFFFF {
                return false;
            }

            i += 16;
        }

        // Check remaining bytes
        a[i..] == b[i..]
    }

    /// SIMD-optimized memory copy for large buffers
    #[target_feature(enable = "sse2")]
    /// Copy memory using SIMD operations for better performance
    ///
    /// # Safety
    ///
    /// This function uses SIMD intrinsics and requires that both source and destination
    /// slices are valid and non-overlapping.
    pub unsafe fn memory_copy_simd(src: &[u8], dst: &mut [u8]) {
        assert_eq!(src.len(), dst.len());

        if src.len() < 16 {
            dst.copy_from_slice(src);
            return;
        }

        let mut i = 0;
        while i + 16 <= src.len() {
            let chunk = _mm_loadu_si128(src.as_ptr().add(i) as *const __m128i);
            _mm_storeu_si128(dst.as_mut_ptr().add(i) as *mut __m128i, chunk);
            i += 16;
        }

        // Copy remaining bytes
        if i < src.len() {
            dst[i..].copy_from_slice(&src[i..]);
        }
    }

    /// SIMD-optimized byte counting
    #[target_feature(enable = "sse2")]
    /// Count occurrences of a byte using SIMD operations
    ///
    /// # Safety
    ///
    /// This function uses SIMD intrinsics and requires that the haystack slice
    /// is valid and properly aligned for optimal performance.
    pub unsafe fn count_byte_simd(haystack: &[u8], needle: u8) -> usize {
        if haystack.len() < 16 {
            return haystack.iter().filter(|&&b| b == needle).count();
        }

        let needle_vec = _mm_set1_epi8(needle as i8);
        let mut count = 0;
        let mut i = 0;

        while i + 16 <= haystack.len() {
            let chunk = _mm_loadu_si128(haystack.as_ptr().add(i) as *const __m128i);
            let cmp = _mm_cmpeq_epi8(chunk, needle_vec);
            let mask = _mm_movemask_epi8(cmp);
            count += mask.count_ones() as usize;
            i += 16;
        }

        // Count remaining bytes
        count + haystack[i..].iter().filter(|&&b| b == needle).count()
    }

    /// SIMD-optimized case-insensitive comparison for ASCII
    #[target_feature(enable = "sse2")]
    /// ASCII case-insensitive comparison using SIMD operations
    ///
    /// # Safety
    ///
    /// This function uses SIMD intrinsics and requires that both input slices
    /// are valid and contain only ASCII bytes.
    pub unsafe fn ascii_case_insensitive_equal_simd(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        if a.len() < 16 {
            return a.iter().zip(b).all(|(&x, &y)| x.eq_ignore_ascii_case(&y));
        }

        let mask_upper = _mm_set1_epi8(0x20); // Space for converting case
        let mask_alpha_min = _mm_set1_epi8(b'A' as i8);
        let mask_alpha_max = _mm_set1_epi8(b'Z' as i8);

        let mut i = 0;
        while i + 16 <= a.len() {
            let chunk_a = _mm_loadu_si128(a.as_ptr().add(i) as *const __m128i);
            let chunk_b = _mm_loadu_si128(b.as_ptr().add(i) as *const __m128i);

            // Convert to lowercase if needed
            let is_upper_a = _mm_and_si128(
                _mm_cmpgt_epi8(chunk_a, _mm_sub_epi8(mask_alpha_min, _mm_set1_epi8(1))),
                _mm_cmplt_epi8(chunk_a, _mm_add_epi8(mask_alpha_max, _mm_set1_epi8(1))),
            );
            let is_upper_b = _mm_and_si128(
                _mm_cmpgt_epi8(chunk_b, _mm_sub_epi8(mask_alpha_min, _mm_set1_epi8(1))),
                _mm_cmplt_epi8(chunk_b, _mm_add_epi8(mask_alpha_max, _mm_set1_epi8(1))),
            );

            let lower_a = _mm_or_si128(chunk_a, _mm_and_si128(is_upper_a, mask_upper));
            let lower_b = _mm_or_si128(chunk_b, _mm_and_si128(is_upper_b, mask_upper));

            let cmp = _mm_cmpeq_epi8(lower_a, lower_b);
            let mask = _mm_movemask_epi8(cmp);

            if mask != 0xFFFF {
                return false;
            }

            i += 16;
        }

        // Check remaining bytes
        a[i..]
            .iter()
            .zip(&b[i..])
            .all(|(&x, &y)| x.eq_ignore_ascii_case(&y))
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub mod simd {
    /// Fallback implementations for non-x86_64 architectures
    pub fn find_byte_simd(haystack: &[u8], needle: u8) -> Option<usize> {
        haystack.iter().position(|&b| b == needle)
    }

    pub fn find_pattern_simd(haystack: &[u8], pattern: &[u8]) -> Option<usize> {
        haystack
            .windows(pattern.len())
            .position(|window| window == pattern)
    }

    pub fn memory_equal_simd(a: &[u8], b: &[u8]) -> bool {
        a == b
    }

    pub fn memory_copy_simd(src: &[u8], dst: &mut [u8]) {
        dst.copy_from_slice(src);
    }

    pub fn count_byte_simd(haystack: &[u8], needle: u8) -> usize {
        haystack.iter().filter(|&&b| b == needle).count()
    }

    pub fn ascii_case_insensitive_equal_simd(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        a.iter()
            .zip(b)
            .all(|(&x, &y)| x.to_ascii_lowercase() == y.to_ascii_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_optimizer_creation() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config).await;
        assert!(optimizer.is_ok());
    }

    #[tokio::test]
    async fn test_cached_operation() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config).await.unwrap();

        let expensive_operation = || async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok::<i32, crate::compat::Error>(42)
        };

        // First call should miss cache
        let start = Instant::now();
        let result1 = optimizer
            .cached_operation("test_key", expensive_operation())
            .await;
        let first_duration = start.elapsed();
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), 42);

        // Second call should hit cache and be faster
        let start = Instant::now();
        let result2 = optimizer
            .cached_operation("test_key", expensive_operation())
            .await;
        let second_duration = start.elapsed();
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), 42);

        // Cache hit should be significantly faster
        assert!(second_duration < first_duration / 2);
    }

    #[tokio::test]
    async fn test_memory_optimization() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config).await.unwrap();

        let buffer = optimizer.allocate_optimized(1024).await;
        assert!(buffer.is_ok());
        assert_eq!(buffer.unwrap().len(), 1024);
    }

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics::new();

        metrics.record_operation_time(Duration::from_millis(10));
        metrics.record_operation_time(Duration::from_millis(20));

        let report = metrics.generate_report();
        assert_eq!(report.operations_total, 2);
        assert_eq!(report.average_operation_time, Duration::from_millis(15));
    }

    #[test]
    fn test_performance_factor_calculation() {
        let baseline = PerformanceReport {
            cache_hit_rate: 0.5,
            cache_hits: 50,
            cache_misses: 50,
            operations_total: 100,
            average_operation_time: Duration::from_millis(100),
            memory_allocations: 1000,
            io_operations: 50,
        };

        let optimized = PerformanceReport {
            cache_hit_rate: 0.9,
            cache_hits: 90,
            cache_misses: 10,
            operations_total: 100,
            average_operation_time: Duration::from_millis(10),
            memory_allocations: 100,
            io_operations: 50,
        };

        let factor = optimized.performance_factor(&baseline);
        assert_eq!(factor, 10.0);
        assert!(optimized.meets_10x_target(&baseline));
    }

    #[test]
    fn test_simd_operations() {
        let haystack = b"hello world, this is a test string for searching";
        let needle = b'w';

        let result = unsafe { simd::find_byte_simd(haystack, needle) };
        assert_eq!(result, Some(6)); // Position of 'w' in "world"

        let a = b"test string";
        let b = b"test string";
        let c = b"different";

        assert!(unsafe { simd::memory_equal_simd(a, b) });
        assert!(!unsafe { simd::memory_equal_simd(a, c) });
    }
}
