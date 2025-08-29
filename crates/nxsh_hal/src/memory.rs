//! Memory management abstraction layer
//!
//! This module provides platform-agnostic memory management operations
//! with platform-specific optimizations where available.

use crate::error::{HalError, HalResult};
use crate::platform::{Capabilities, Platform};
use std::alloc::{self, Layout};
use std::ptr::NonNull;

/// Memory manager for system-level memory operations
#[derive(Debug)]
pub struct MemoryManager {
    #[allow(dead_code)]
    platform: Platform,
    #[allow(dead_code)]
    capabilities: Capabilities,
}

impl MemoryManager {
    pub fn new() -> HalResult<Self> {
        Ok(Self {
            platform: Platform::current(),
            capabilities: Capabilities::current(),
        })
    }

    /// Get system memory information
    pub fn memory_info(&self) -> HalResult<MemoryInfo> {
        #[cfg(unix)]
        {
            self.memory_info_unix()
        }
        #[cfg(windows)]
        {
            self.memory_info_windows()
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(HalError::unsupported(
                "Memory info not supported on this platform",
            ))
        }
    }

    /// Allocate aligned memory
    pub fn allocate_aligned(&self, size: usize, alignment: usize) -> HalResult<NonNull<u8>> {
        if !alignment.is_power_of_two() {
            return Err(HalError::invalid("Alignment must be power of two"));
        }

        let layout = Layout::from_size_align(size, alignment)
            .map_err(|_| HalError::invalid("Invalid layout"))?;

        let ptr = unsafe { alloc::alloc(layout) };
        NonNull::new(ptr)
            .ok_or_else(|| HalError::memory_error("allocate_aligned", Some(size), "Out of memory"))
    }

    /// Deallocate aligned memory
    pub fn deallocate_aligned(
        &self,
        ptr: NonNull<u8>,
        size: usize,
        alignment: usize,
    ) -> HalResult<()> {
        let layout = Layout::from_size_align(size, alignment)
            .map_err(|_| HalError::invalid("Invalid layout"))?;

        unsafe {
            alloc::dealloc(ptr.as_ptr(), layout);
        }
        Ok(())
    }

    /// Lock memory pages (prevent swapping)
    pub fn lock_memory(&self, ptr: *const u8, size: usize) -> HalResult<()> {
        #[cfg(unix)]
        {
            // Use nix instead of direct libc calls for safety
            use nix::sys::mman::mlock;

            // nix mlock expects raw pointers
            match unsafe { mlock(ptr as *const std::ffi::c_void, size) } {
                Ok(()) => Ok(()),
                Err(err) => Err(HalError::memory_error(
                    "mlock",
                    Some(size),
                    &format!("Failed to lock memory: {}", err),
                )),
            }
        }
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Memory::VirtualLock;
            let result = unsafe { VirtualLock(ptr as *const std::ffi::c_void, size) };
            if result == 0 {
                return Err(HalError::memory_error(
                    "VirtualLock",
                    Some(size),
                    &format!("Failed to lock memory: {}", std::io::Error::last_os_error()),
                ));
            }
            Ok(())
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(HalError::unsupported(
                "Memory locking not supported on this platform",
            ))
        }
    }

    /// Unlock memory pages
    pub fn unlock_memory(&self, ptr: *const u8, size: usize) -> HalResult<()> {
        #[cfg(unix)]
        {
            // Use nix instead of direct libc calls for safety
            use nix::sys::mman::munlock;

            // nix munlock expects raw pointers
            match unsafe { munlock(ptr as *const std::ffi::c_void, size) } {
                Ok(()) => Ok(()),
                Err(err) => Err(HalError::memory_error(
                    "munlock",
                    Some(size),
                    &format!("Failed to unlock memory: {}", err),
                )),
            }
        }
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Memory::VirtualUnlock;
            let result = unsafe { VirtualUnlock(ptr as *const std::ffi::c_void, size) };
            if result == 0 {
                return Err(HalError::memory_error(
                    "VirtualUnlock",
                    Some(size),
                    &format!(
                        "Failed to unlock memory: {}",
                        std::io::Error::last_os_error()
                    ),
                ));
            }
            Ok(())
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(HalError::unsupported(
                "Memory unlocking not supported on this platform",
            ))
        }
    }

    /// Advise kernel about memory usage patterns
    pub fn memory_advise(
        &self,
        _ptr: *const u8,
        _size: usize,
        _advice: MemoryAdvice,
    ) -> HalResult<()> {
        if !self.capabilities.has_madvise {
            return Err(HalError::unsupported(
                "Memory advice not supported on this platform",
            ));
        }

        #[cfg(unix)]
        {
            // Use nix instead of direct libc calls for safety
            use nix::sys::mman::{madvise, MmapAdvise};

            let advice_flag = match _advice {
                MemoryAdvice::Normal => MmapAdvise::MADV_NORMAL,
                MemoryAdvice::Random => MmapAdvise::MADV_RANDOM,
                MemoryAdvice::Sequential => MmapAdvise::MADV_SEQUENTIAL,
                MemoryAdvice::WillNeed => MmapAdvise::MADV_WILLNEED,
                MemoryAdvice::DontNeed => MmapAdvise::MADV_DONTNEED,
            };

            match unsafe { madvise(_ptr as *mut std::ffi::c_void, _size, advice_flag) } {
                Ok(()) => Ok(()),
                Err(err) => Err(HalError::memory_error(
                    "madvise",
                    Some(_size),
                    &format!("Failed to advise memory: {}", err),
                )),
            }
        }
        #[cfg(not(unix))]
        {
            Err(HalError::unsupported(
                "Memory advice not supported on this platform",
            ))
        }
    }

    #[cfg(unix)]
    fn memory_info_unix(&self) -> HalResult<MemoryInfo> {
        let page_size = self.capabilities.page_size as u64;

        // Try to read memory info from /proc/meminfo on Linux for safer memory information
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                let mut total_kb = 0u64;
                let mut available_kb = 0u64;
                let mut free_kb = 0u64;
                let mut buffers_kb = 0u64;
                let mut cached_kb = 0u64;

                for line in meminfo.lines() {
                    if let Some(value) = line.strip_prefix("MemTotal:") {
                        if let Ok(kb) = value
                            .trim()
                            .split_whitespace()
                            .next()
                            .unwrap_or("0")
                            .parse::<u64>()
                        {
                            total_kb = kb;
                        }
                    } else if let Some(value) = line.strip_prefix("MemAvailable:") {
                        if let Ok(kb) = value
                            .trim()
                            .split_whitespace()
                            .next()
                            .unwrap_or("0")
                            .parse::<u64>()
                        {
                            available_kb = kb;
                        }
                    } else if let Some(value) = line.strip_prefix("MemFree:") {
                        if let Ok(kb) = value
                            .trim()
                            .split_whitespace()
                            .next()
                            .unwrap_or("0")
                            .parse::<u64>()
                        {
                            free_kb = kb;
                        }
                    } else if let Some(value) = line.strip_prefix("Buffers:") {
                        if let Ok(kb) = value
                            .trim()
                            .split_whitespace()
                            .next()
                            .unwrap_or("0")
                            .parse::<u64>()
                        {
                            buffers_kb = kb;
                        }
                    } else if let Some(value) = line.strip_prefix("Cached:") {
                        if let Ok(kb) = value
                            .trim()
                            .split_whitespace()
                            .next()
                            .unwrap_or("0")
                            .parse::<u64>()
                        {
                            cached_kb = kb;
                        }
                    }
                }

                let total_memory = total_kb * 1024;
                let available_memory = if available_kb > 0 {
                    available_kb * 1024
                } else {
                    (free_kb + buffers_kb + cached_kb) * 1024
                };
                let used_memory = total_memory.saturating_sub(available_memory);

                return Ok(MemoryInfo {
                    total_physical: total_memory,
                    available_physical: available_memory,
                    used_physical: used_memory,
                    total_virtual: total_memory, // Simplified
                    available_virtual: available_memory,
                    used_virtual: used_memory,
                    page_size,
                });
            }
        }

        // Fallback for other Unix systems using reasonable defaults
        let total_memory = (self.capabilities.cpu_count as u64) * 2 * 1024 * 1024 * 1024; // 2GB per CPU core estimate
        let available_memory = total_memory / 2; // Rough estimate: half available
        let used_memory = total_memory - available_memory;

        Ok(MemoryInfo {
            total_physical: total_memory,
            available_physical: available_memory,
            used_physical: used_memory,
            total_virtual: total_memory,
            available_virtual: available_memory,
            used_virtual: used_memory,
            page_size,
        })
    }

    #[cfg(windows)]
    fn memory_info_windows(&self) -> HalResult<MemoryInfo> {
        use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

        let mut memory_status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
        memory_status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;

        let result = unsafe { GlobalMemoryStatusEx(&mut memory_status) };
        if result == 0 {
            return Err(HalError::memory_error(
                "GlobalMemoryStatusEx",
                None,
                &format!(
                    "Failed to get memory info: {}",
                    std::io::Error::last_os_error()
                ),
            ));
        }

        Ok(MemoryInfo {
            total_physical: memory_status.ullTotalPhys,
            available_physical: memory_status.ullAvailPhys,
            used_physical: memory_status.ullTotalPhys - memory_status.ullAvailPhys,
            total_virtual: memory_status.ullTotalVirtual,
            available_virtual: memory_status.ullAvailVirtual,
            used_virtual: memory_status.ullTotalVirtual - memory_status.ullAvailVirtual,
            page_size: self.capabilities.page_size as u64,
        })
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// System memory information
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total_physical: u64,
    pub available_physical: u64,
    pub used_physical: u64,
    pub total_virtual: u64,
    pub available_virtual: u64,
    pub used_virtual: u64,
    pub page_size: u64,
}

impl MemoryInfo {
    /// Get physical memory usage percentage
    pub fn physical_usage_percentage(&self) -> f64 {
        if self.total_physical == 0 {
            0.0
        } else {
            (self.used_physical as f64 / self.total_physical as f64) * 100.0
        }
    }

    /// Get virtual memory usage percentage
    pub fn virtual_usage_percentage(&self) -> f64 {
        if self.total_virtual == 0 {
            0.0
        } else {
            (self.used_virtual as f64 / self.total_virtual as f64) * 100.0
        }
    }
}

/// Memory usage advice for the kernel
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryAdvice {
    /// Normal access pattern
    Normal,
    /// Random access pattern
    Random,
    /// Sequential access pattern
    Sequential,
    /// Will need this memory soon
    WillNeed,
    /// Don't need this memory
    DontNeed,
}
