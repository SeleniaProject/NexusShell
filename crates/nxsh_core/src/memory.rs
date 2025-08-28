use std::{
    collections::HashMap,
    sync::{Arc, RwLock, atomic::{AtomicU64, Ordering}},
    time::{Duration, SystemTime},
};

/// Memory pool for buffer reuse
pub struct MemoryPool {
    buffers: Arc<RwLock<Vec<Vec<u8>>>>,
    max_pool_size: usize,
    total_allocated: AtomicU64,
    total_freed: AtomicU64,
}

impl MemoryPool {
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            buffers: Arc::new(RwLock::new(Vec::new())),
            max_pool_size,
            total_allocated: AtomicU64::new(0),
            total_freed: AtomicU64::new(0),
        }
    }

    pub fn acquire(&self, min_size: usize) -> Vec<u8> {
        if let Ok(mut buffers) = self.buffers.write() {
            for i in 0..buffers.len() {
                if buffers[i].capacity() >= min_size {
                    let mut buffer = buffers.remove(i);
                    buffer.clear();
                    return buffer;
                }
            }
        }

        let buffer = Vec::with_capacity(min_size);
        self.total_allocated.fetch_add(min_size as u64, Ordering::Relaxed);
        buffer
    }

    pub fn release(&self, mut buffer: Vec<u8>) {
        let capacity = buffer.capacity();
        
        if let Ok(mut buffers) = self.buffers.write() {
            if buffers.len() < self.max_pool_size && capacity > 0 {
                buffer.clear();
                buffers.push(buffer);
                self.total_freed.fetch_add(capacity as u64, Ordering::Relaxed);
                return;
            }
        }
        
        // Buffer dropped here if pool is full
        self.total_freed.fetch_add(capacity as u64, Ordering::Relaxed);
    }

    pub fn stats(&self) -> MemoryPoolStats {
        let buffers_count = self.buffers.read().map(|b| b.len()).unwrap_or(0);
        MemoryPoolStats {
            buffers_in_pool: buffers_count,
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
            total_freed: self.total_freed.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryPoolStats {
    pub buffers_in_pool: usize,
    pub total_allocated: u64,
    pub total_freed: u64,
}

/// String interning for memory deduplication
pub struct StringInterner {
    strings: Arc<RwLock<HashMap<String, Arc<str>>>>,
    stats: Arc<RwLock<InternerStats>>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(InternerStats::default())),
        }
    }

    pub fn intern(&self, s: &str) -> Arc<str> {
        if let Ok(strings) = self.strings.read() {
            if let Some(interned) = strings.get(s) {
                if let Ok(mut stats) = self.stats.write() {
                    stats.cache_hits += 1;
                }
                return Arc::clone(interned);
            }
        }

        let arc_str: Arc<str> = Arc::from(s);
        
        if let Ok(mut strings) = self.strings.write() {
            strings.insert(s.to_string(), Arc::clone(&arc_str));
        }

        if let Ok(mut stats) = self.stats.write() {
            stats.cache_misses += 1;
            stats.total_strings += 1;
            stats.total_bytes += s.len() as u64;
        }

        arc_str
    }

    pub fn stats(&self) -> InternerStats {
        self.stats.read().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn cleanup(&self) {
        // Remove strings with only one reference (only held by interner)
        if let Ok(mut strings) = self.strings.write() {
            let before_count = strings.len();
            strings.retain(|_, v| Arc::strong_count(v) > 1);
            let after_count = strings.len();
            
            if let Ok(mut stats) = self.stats.write() {
                stats.cleaned_strings += before_count - after_count;
            }
        }
    }
}

impl Default for StringInterner {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, Default)]
pub struct InternerStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_strings: u64,
    pub total_bytes: u64,
    pub cleaned_strings: usize,
}

/// Object pooling for frequently allocated types
pub trait Poolable {
    fn reset(&mut self);
    fn new_for_pool() -> Self;
}

#[allow(dead_code)]
pub struct ObjectPool<T: Poolable> {
    objects: Arc<RwLock<Vec<T>>>,
    max_size: usize,
    created_count: AtomicU64,
    reused_count: AtomicU64,
}

impl<T: Poolable> ObjectPool<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            objects: Arc::new(RwLock::new(Vec::new())),
            max_size,
            created_count: AtomicU64::new(0),
            reused_count: AtomicU64::new(0),
        }
    }

    pub fn acquire(&self) -> PooledObject<T> {
        if let Ok(mut objects) = self.objects.write() {
            if let Some(mut object) = objects.pop() {
                object.reset();
                self.reused_count.fetch_add(1, Ordering::Relaxed);
                return PooledObject::new(object, Arc::clone(&self.objects));
            }
        }

        let object = T::new_for_pool();
        self.created_count.fetch_add(1, Ordering::Relaxed);
        PooledObject::new(object, Arc::clone(&self.objects))
    }

    pub fn stats(&self) -> ObjectPoolStats {
        ObjectPoolStats {
            objects_in_pool: self.objects.read().map(|o| o.len()).unwrap_or(0),
            created_count: self.created_count.load(Ordering::Relaxed),
            reused_count: self.reused_count.load(Ordering::Relaxed),
        }
    }
}

pub struct PooledObject<T: Poolable> {
    object: Option<T>,
    pool: Arc<RwLock<Vec<T>>>,
}

impl<T: Poolable> PooledObject<T> {
    fn new(object: T, pool: Arc<RwLock<Vec<T>>>) -> Self {
        Self {
            object: Some(object),
            pool,
        }
    }
}

impl<T: Poolable> std::ops::Deref for PooledObject<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<T: Poolable> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().unwrap()
    }
}

impl<T: Poolable> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            if let Ok(mut pool) = self.pool.write() {
                if pool.len() < pool.capacity() {
                    pool.push(object);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObjectPoolStats {
    pub objects_in_pool: usize,
    pub created_count: u64,
    pub reused_count: u64,
}

/// Memory monitoring and optimization
pub struct MemoryManager {
    pools: HashMap<String, Arc<MemoryPool>>,
    interner: Arc<StringInterner>,
    last_cleanup: SystemTime,
    cleanup_interval: Duration,
    memory_limit: Option<u64>,
}

impl MemoryManager {
    pub fn new() -> Self {
        let mut pools = HashMap::new();
        
        // Pre-create common buffer pools
        pools.insert("small".to_string(), Arc::new(MemoryPool::new(100))); // < 1KB
        pools.insert("medium".to_string(), Arc::new(MemoryPool::new(50)));  // 1KB-16KB  
        pools.insert("large".to_string(), Arc::new(MemoryPool::new(10)));   // > 16KB

        Self {
            pools,
            interner: Arc::new(StringInterner::new()),
            last_cleanup: SystemTime::now(),
            cleanup_interval: Duration::from_secs(30),
            memory_limit: None,
        }
    }

    pub fn get_buffer(&self, size: usize) -> Vec<u8> {
        let pool_name = match size {
            0..=1024 => "small",
            1025..=16384 => "medium", 
            _ => "large",
        };

        if let Some(pool) = self.pools.get(pool_name) {
            pool.acquire(size)
        } else {
            Vec::with_capacity(size)
        }
    }

    pub fn return_buffer(&self, buffer: Vec<u8>) {
        let size = buffer.capacity();
        let pool_name = match size {
            0..=1024 => "small",
            1025..=16384 => "medium",
            _ => "large", 
        };

        if let Some(pool) = self.pools.get(pool_name) {
            pool.release(buffer);
        }
    }

    pub fn intern_string(&self, s: &str) -> Arc<str> {
        self.interner.intern(s)
    }

    pub fn periodic_cleanup(&mut self) {
        let now = SystemTime::now();
        if now.duration_since(self.last_cleanup).unwrap_or_default() >= self.cleanup_interval {
            self.interner.cleanup();
            self.last_cleanup = now;
        }
    }

    pub fn memory_stats(&self) -> MemoryManagerStats {
        let mut pool_stats = HashMap::new();
        for (name, pool) in &self.pools {
            pool_stats.insert(name.clone(), pool.stats());
        }

        MemoryManagerStats {
            pool_stats,
            interner_stats: self.interner.stats(),
            current_memory_usage: self.estimate_memory_usage(),
        }
    }

    fn estimate_memory_usage(&self) -> u64 {
        // Rough estimation based on pool statistics
        let mut total = 0u64;
        
        for pool in self.pools.values() {
            let stats = pool.stats();
            total += stats.total_allocated.saturating_sub(stats.total_freed);
        }

        let interner_stats = self.interner.stats();
        total += interner_stats.total_bytes;

        total
    }

    pub fn set_memory_limit(&mut self, limit: u64) {
        self.memory_limit = Some(limit);
    }

    pub fn check_memory_pressure(&self) -> bool {
        if let Some(limit) = self.memory_limit {
            self.estimate_memory_usage() > limit * 8 / 10 // 80% threshold
        } else {
            false
        }
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MemoryManagerStats {
    pub pool_stats: HashMap<String, MemoryPoolStats>,
    pub interner_stats: InternerStats,
    pub current_memory_usage: u64,
}

// Global memory manager instance
use std::sync::OnceLock;
static GLOBAL_MEMORY_MANAGER: OnceLock<Arc<RwLock<MemoryManager>>> = OnceLock::new();

pub fn global_memory_manager() -> Arc<RwLock<MemoryManager>> {
    GLOBAL_MEMORY_MANAGER.get_or_init(|| {
        Arc::new(RwLock::new(MemoryManager::new()))
    }).clone()
}

/// Convenient functions for global memory management
pub fn get_buffer(size: usize) -> Vec<u8> {
    global_memory_manager().read().unwrap().get_buffer(size)
}

pub fn return_buffer(buffer: Vec<u8>) {
    global_memory_manager().read().unwrap().return_buffer(buffer)
}

pub fn intern_string(s: &str) -> Arc<str> {
    global_memory_manager().read().unwrap().intern_string(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn test_memory_pool_basic() {
        let pool = MemoryPool::new(5);
        
        let buffer1 = pool.acquire(1024);
        assert_eq!(buffer1.capacity(), 1024);
        
        pool.release(buffer1);
        let buffer2 = pool.acquire(512);  // Should reuse the buffer
        assert!(buffer2.capacity() >= 512); // May reuse larger buffer
        
        pool.release(buffer2); // Release buffer2 before checking stats
        
        let stats = pool.stats();
        assert_eq!(stats.buffers_in_pool, 1); // buffer2 now in pool
    }

    #[test]
    fn test_string_interner() {
        let interner = StringInterner::new();
        
        let s1 = interner.intern("hello");
        let s2 = interner.intern("hello");
        
        assert!(Arc::ptr_eq(&s1, &s2)); // Same object
        
        let stats = interner.stats();
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
    }
}
