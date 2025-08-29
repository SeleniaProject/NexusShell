//! SIMD-accelerated operations for world-class performance
//!
//! This module provides SIMD (Single Instruction, Multiple Data) optimizations
//! for performance-critical operations in NexusShell.

use std::arch::x86_64::*;

/// SIMD-accelerated string operations
pub struct SimdStringOps;

impl SimdStringOps {
    /// Find byte in string using SIMD acceleration
    #[cfg(target_arch = "x86_64")]
    pub fn find_byte_simd(haystack: &[u8], needle: u8) -> Option<usize> {
        if haystack.len() < 16 {
            return haystack.iter().position(|&b| b == needle);
        }

        unsafe {
            if !is_x86_feature_detected!("sse2") {
                return haystack.iter().position(|&b| b == needle);
            }

            let needle_vec = _mm_set1_epi8(needle as i8);
            let mut pos = 0;

            while pos + 16 <= haystack.len() {
                let data = _mm_loadu_si128(haystack.as_ptr().add(pos) as *const __m128i);
                let cmp = _mm_cmpeq_epi8(data, needle_vec);
                let mask = _mm_movemask_epi8(cmp) as u32;

                if mask != 0 {
                    return Some(pos + mask.trailing_zeros() as usize);
                }

                pos += 16;
            }

            // Handle remaining bytes
            haystack[pos..]
                .iter()
                .position(|&b| b == needle)
                .map(|i| pos + i)
        }
    }

    /// Count specific byte in string using SIMD
    #[cfg(target_arch = "x86_64")]
    pub fn count_byte_simd(haystack: &[u8], needle: u8) -> usize {
        if haystack.len() < 16 {
            return haystack.iter().filter(|&&b| b == needle).count();
        }

        unsafe {
            if !is_x86_feature_detected!("sse2") {
                return haystack.iter().filter(|&&b| b == needle).count();
            }

            let needle_vec = _mm_set1_epi8(needle as i8);
            let mut count = 0;
            let mut pos = 0;

            while pos + 16 <= haystack.len() {
                let data = _mm_loadu_si128(haystack.as_ptr().add(pos) as *const __m128i);
                let cmp = _mm_cmpeq_epi8(data, needle_vec);
                let mask = _mm_movemask_epi8(cmp) as u32;

                count += mask.count_ones() as usize;
                pos += 16;
            }

            // Handle remaining bytes
            count + haystack[pos..].iter().filter(|&&b| b == needle).count()
        }
    }

    /// Fast memory comparison using SIMD
    #[cfg(target_arch = "x86_64")]
    pub fn memory_equal_simd(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        if a.len() < 16 {
            return a == b;
        }

        unsafe {
            if !is_x86_feature_detected!("sse2") {
                return a == b;
            }

            let mut pos = 0;
            while pos + 16 <= a.len() {
                let a_data = _mm_loadu_si128(a.as_ptr().add(pos) as *const __m128i);
                let b_data = _mm_loadu_si128(b.as_ptr().add(pos) as *const __m128i);
                let cmp = _mm_cmpeq_epi8(a_data, b_data);
                let mask = _mm_movemask_epi8(cmp) as u32;

                if mask != 0xFFFF {
                    return false;
                }

                pos += 16;
            }

            // Handle remaining bytes
            a[pos..] == b[pos..]
        }
    }

    /// Fast string copying with SIMD acceleration
    #[cfg(target_arch = "x86_64")]
    pub fn copy_memory_simd(src: &[u8], dst: &mut [u8]) {
        assert!(src.len() <= dst.len());

        if src.len() < 16 {
            dst[..src.len()].copy_from_slice(src);
            return;
        }

        unsafe {
            if !is_x86_feature_detected!("sse2") {
                dst[..src.len()].copy_from_slice(src);
                return;
            }

            let mut pos = 0;
            while pos + 16 <= src.len() {
                let data = _mm_loadu_si128(src.as_ptr().add(pos) as *const __m128i);
                _mm_storeu_si128(dst.as_mut_ptr().add(pos) as *mut __m128i, data);
                pos += 16;
            }

            // Handle remaining bytes
            if pos < src.len() {
                dst[pos..src.len()].copy_from_slice(&src[pos..]);
            }
        }
    }
}

/// Fallback implementations for non-x86_64 architectures
#[cfg(not(target_arch = "x86_64"))]
impl SimdStringOps {
    pub fn find_byte_simd(haystack: &[u8], needle: u8) -> Option<usize> {
        haystack.iter().position(|&b| b == needle)
    }

    pub fn count_byte_simd(haystack: &[u8], needle: u8) -> usize {
        haystack.iter().filter(|&&b| b == needle).count()
    }

    pub fn memory_equal_simd(a: &[u8], b: &[u8]) -> bool {
        a == b
    }

    pub fn copy_memory_simd(src: &[u8], dst: &mut [u8]) {
        dst[..src.len()].copy_from_slice(src);
    }
}

/// CPU-specific optimizations
pub struct CpuOptimizer {
    cpu_features: CpuFeatures,
}

#[derive(Debug, Clone)]
pub struct CpuFeatures {
    pub sse2: bool,
    pub sse3: bool,
    pub sse41: bool,
    pub sse42: bool,
    pub avx: bool,
    pub avx2: bool,
    pub bmi1: bool,
    pub bmi2: bool,
    pub popcnt: bool,
}

impl Default for CpuFeatures {
    fn default() -> Self {
        Self::detect()
    }
}

impl CpuFeatures {
    /// Detect available CPU features
    #[cfg(target_arch = "x86_64")]
    pub fn detect() -> Self {
        Self {
            sse2: is_x86_feature_detected!("sse2"),
            sse3: is_x86_feature_detected!("sse3"),
            sse41: is_x86_feature_detected!("sse4.1"),
            sse42: is_x86_feature_detected!("sse4.2"),
            avx: is_x86_feature_detected!("avx"),
            avx2: is_x86_feature_detected!("avx2"),
            bmi1: is_x86_feature_detected!("bmi1"),
            bmi2: is_x86_feature_detected!("bmi2"),
            popcnt: is_x86_feature_detected!("popcnt"),
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    pub fn detect() -> Self {
        Self {
            sse2: false,
            sse3: false,
            sse41: false,
            sse42: false,
            avx: false,
            avx2: false,
            bmi1: false,
            bmi2: false,
            popcnt: false,
        }
    }
}

impl CpuOptimizer {
    pub fn new() -> Self {
        Self {
            cpu_features: CpuFeatures::detect(),
        }
    }

    /// Get CPU optimization level based on available features
    pub fn optimization_level(&self) -> u8 {
        let mut level = 0;

        if self.cpu_features.sse2 {
            level += 1;
        }
        if self.cpu_features.sse3 {
            level += 1;
        }
        if self.cpu_features.sse41 {
            level += 1;
        }
        if self.cpu_features.sse42 {
            level += 1;
        }
        if self.cpu_features.avx {
            level += 2;
        }
        if self.cpu_features.avx2 {
            level += 2;
        }
        if self.cpu_features.popcnt {
            level += 1;
        }

        level
    }

    /// Fast population count using hardware acceleration
    #[cfg(target_arch = "x86_64")]
    pub fn popcount_optimized(&self, value: u64) -> u32 {
        if self.cpu_features.popcnt {
            unsafe { _popcnt64(value as i64) as u32 }
        } else {
            value.count_ones()
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    pub fn popcount_optimized(&self, value: u64) -> u32 {
        value.count_ones()
    }

    /// Get optimal buffer size based on CPU cache
    pub fn optimal_buffer_size(&self) -> usize {
        // L1 cache optimized sizes based on CPU features
        if self.cpu_features.avx2 {
            64 * 1024 // 64KB for AVX2 systems
        } else if self.cpu_features.avx {
            32 * 1024 // 32KB for AVX systems
        } else if self.cpu_features.sse42 {
            16 * 1024 // 16KB for SSE4.2 systems
        } else {
            8 * 1024 // 8KB fallback
        }
    }
}

impl Default for CpuOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_find_byte() {
        let data = b"Hello, World!";
        assert_eq!(SimdStringOps::find_byte_simd(data, b'W'), Some(7));
        assert_eq!(SimdStringOps::find_byte_simd(data, b'z'), None);
    }

    #[test]
    fn test_simd_count_byte() {
        let data = b"Hello, World!";
        assert_eq!(SimdStringOps::count_byte_simd(data, b'l'), 3);
        assert_eq!(SimdStringOps::count_byte_simd(data, b'z'), 0);
    }

    #[test]
    fn test_simd_memory_equal() {
        let a = b"Hello, World!";
        let b = b"Hello, World!";
        let c = b"Hello, World?";

        assert!(SimdStringOps::memory_equal_simd(a, b));
        assert!(!SimdStringOps::memory_equal_simd(a, c));
    }

    #[test]
    fn test_cpu_features_detection() {
        let _features = CpuFeatures::detect();
        let optimizer = CpuOptimizer::new();
        // Just ensure it doesn't panic and returns a valid level
        let level = optimizer.optimization_level();
        assert!(level <= 10); // Max reasonable level check
    }

    #[test]
    fn test_cpu_optimizer() {
        let optimizer = CpuOptimizer::new();
        assert!(optimizer.optimal_buffer_size() > 0);
        assert!(optimizer.popcount_optimized(0xFF) == 8);
    }
}
