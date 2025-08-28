//! Memory optimization utilities for NexusShell
//! 
//! This module provides memory-efficient utilities for common operations
//! like string building, buffer management, and frequent allocations.

use std::sync::Arc;
use crate::memory::{get_buffer, return_buffer, intern_string};

/// Memory-efficient string builder with buffer pooling
#[allow(dead_code)]
pub struct MemoryEfficientStringBuilder {
    buffer: Vec<u8>,
    #[allow(dead_code)]
    capacity_hint: usize,
}

impl MemoryEfficientStringBuilder {
    /// Create new string builder with capacity hint
    pub fn new(capacity_hint: usize) -> Self {
        let buffer = get_buffer(capacity_hint);
        Self {
            buffer,
            capacity_hint,
        }
    }

    /// Create new string builder with capacity hint (alias for new)
    pub fn with_capacity(capacity: usize) -> Self {
        Self::new(capacity)
    }

    /// Add string to builder (avoiding format! when possible)
    pub fn push_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let required_capacity = self.buffer.len() + bytes.len();
        
        // Ensure we have enough capacity with growth strategy
        if required_capacity > self.buffer.capacity() {
            let new_capacity = (required_capacity * 3 / 2).max(self.buffer.capacity() * 2);
            self.buffer.reserve(new_capacity - self.buffer.capacity());
        }
        
        // Use SIMD-optimized copy for large strings when available
        if bytes.len() > 32 {
            let start_pos = self.buffer.len();
            self.buffer.resize(required_capacity, 0);
            crate::simd_optimization::SimdStringOps::copy_memory_simd(
                bytes, 
                &mut self.buffer[start_pos..]
            );
        } else {
            self.buffer.extend_from_slice(bytes);
        }
    }

    /// Add character to builder
    pub fn push(&mut self, c: char) {
        let mut buffer = [0; 4];
        let s = c.encode_utf8(&mut buffer);
        self.push_str(s);
    }

    /// Add number to builder (avoiding format! for common cases)
    pub fn push_number(&mut self, n: i64) {
        if n == 0 {
            self.buffer.push(b'0');
            return;
        }

        let mut temp = [0u8; 32]; // Enough for any i64
        let mut idx = temp.len();
        let mut num = n.unsigned_abs();
        
        while num > 0 {
            idx -= 1;
            temp[idx] = (num % 10) as u8 + b'0';
            num /= 10;
        }
        
        if n < 0 {
            idx -= 1;
            temp[idx] = b'-';
        }
        
        self.buffer.extend_from_slice(&temp[idx..]);
    }

    /// Convert to final string (returning buffer to pool)
    pub fn into_string(mut self) -> String {
        // Convert buffer to String, ensuring only valid UTF-8 data
        let result = String::from_utf8(self.buffer.clone())
            .unwrap_or_else(|_| {
                // If conversion fails, try to recover by using valid parts
                String::from_utf8_lossy(&self.buffer).into_owned()
            });
        
        // Return buffer to pool if it's worth reusing
        if self.buffer.capacity() >= 64 {
            self.buffer.clear();
            return_buffer(std::mem::take(&mut self.buffer));
        }
        
        result
    }
    
    /// Get string view without consuming the builder
    pub fn as_string(&self) -> String {
        String::from_utf8_lossy(&self.buffer).into_owned()
    }

    /// Add character to builder
    pub fn push_char(&mut self, c: char) {
        let mut buf = [0u8; 4];
        let encoded = c.encode_utf8(&mut buf);
        self.buffer.extend_from_slice(encoded.as_bytes());
    }

    /// Build final string and return buffer to pool
    pub fn build(mut self) -> String {
        // Shrink to exact size
        self.buffer.shrink_to_fit();
        
        // Take ownership of buffer to avoid Drop trait issues
        let buffer = std::mem::take(&mut self.buffer);
        
        // Convert to string
        String::from_utf8(buffer).unwrap_or_default()
    }

    /// Build and intern the string for deduplication
    pub fn build_interned(self) -> Arc<str> {
        let s = self.build();
        intern_string(&s)
    }
}

impl Drop for MemoryEfficientStringBuilder {
    fn drop(&mut self) {
        // Return buffer to pool if it's a reasonable size
        if self.buffer.capacity() <= 16384 { // 16KB max for pooling
            return_buffer(std::mem::take(&mut self.buffer));
        }
    }
}

impl Default for MemoryEfficientStringBuilder {
    /// Create string builder with optimal default capacity (256 bytes)
    /// This size handles most string operations efficiently
    fn default() -> Self {
        Self::new(256)
    }
}

/// Optimized string formatting for common patterns
pub mod fast_format {
    use super::*;

    /// Fast formatting for "name: value" pairs
    pub fn name_value(name: &str, value: &str) -> String {
        let mut builder = MemoryEfficientStringBuilder::new(name.len() + value.len() + 2);
        builder.push_str(name);
        builder.push_str(": ");
        builder.push_str(value);
        builder.build()
    }

    /// Fast formatting for "Showing X of Y items"
    pub fn showing_items(current: usize, total: usize) -> String {
        let mut builder = MemoryEfficientStringBuilder::new(50);
        builder.push_str("Showing ");
        builder.push_number(current as i64);
        builder.push_str(" of ");
        builder.push_number(total as i64);
        builder.push_str(" items");
        builder.build()
    }

    /// Fast formatting for exit codes
    pub fn exit_code(code: i32) -> String {
        let mut builder = MemoryEfficientStringBuilder::new(30);
        builder.push_str(" (exit code: ");
        builder.push_number(code as i64);
        builder.push_char(')');
        builder.build()
    }

    /// Fast formatting for totals
    pub fn total_count(count: usize, item_type: &str) -> String {
        let mut builder = MemoryEfficientStringBuilder::new(30 + item_type.len());
        builder.push_str("Total: ");
        builder.push_number(count as i64);
        builder.push_char(' ');
        builder.push_str(item_type);
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_builder() {
        let mut builder = MemoryEfficientStringBuilder::new(20);
        builder.push_str("Hello");
        builder.push_char(' ');
        builder.push_str("World");
        builder.push_char('!');
        
        assert_eq!(builder.build(), "Hello World!");
    }

    #[test] 
    fn test_number_formatting() {
        let mut builder = MemoryEfficientStringBuilder::new(10);
        builder.push_number(42);
        assert_eq!(builder.build(), "42");
        
        let mut builder = MemoryEfficientStringBuilder::new(10);
        builder.push_number(-123);
        assert_eq!(builder.build(), "-123");
        
        let mut builder = MemoryEfficientStringBuilder::new(10);
        builder.push_number(0);
        assert_eq!(builder.build(), "0");
    }

    #[test]
    fn test_fast_format() {
        assert_eq!(fast_format::name_value("name", "value"), "name: value");
        assert_eq!(fast_format::showing_items(10, 100), "Showing 10 of 100 items");
        assert_eq!(fast_format::exit_code(1), " (exit code: 1)");
        assert_eq!(fast_format::total_count(5, "rows"), "Total: 5 rows");
    }
}
