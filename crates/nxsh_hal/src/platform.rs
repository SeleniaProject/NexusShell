//! Platform detection and capability management
//!
//! This module provides comprehensive platform detection and capability
//! management for NexusShell, enabling platform-specific optimizations
//! and feature availability detection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Once;

use crate::error::HalResult;

static INIT: Once = Once::new();
static mut CURRENT_PLATFORM: Option<Platform> = None;
static mut CAPABILITIES: Option<Capabilities> = None;

// Missing type definitions
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub platform: Platform,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub disk_info: Vec<DiskInfo>,
    pub network_interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub cpu_count: usize,
    pub cpu_model: String,
    pub cpu_frequency: String,
    pub cpu_vendor: String,
}

#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total_memory: u64,
    pub available_memory: u64,
    pub used_memory: u64,
    pub free_memory: u64,
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub filesystem: String,
}

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_addresses: Vec<String>,
    pub mac_address: Option<String>,
    pub is_up: bool,
    pub is_loopback: bool,
    pub mtu: Option<u32>,
    pub statistics: Option<NetworkStatistics>,
}

/// Supported platforms
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
    FreeBSD,
    OpenBSD,
    NetBSD,
    Solaris,
    Android,
    Unknown(String),
}

/// Platform capabilities and feature flags
#[derive(Debug, Clone)]
pub struct Capabilities {
    // Process capabilities
    pub has_fork: bool,
    pub has_exec: bool,
    pub has_pipes: bool,
    pub has_signals: bool,
    pub has_job_control: bool,
    pub has_process_groups: bool,

    // File system capabilities
    pub has_file_locking: bool,
    pub has_memory_mapping: bool,
    pub has_shared_memory: bool,
    pub has_semaphores: bool,
    pub has_message_queues: bool,

    // Threading capabilities
    pub has_threads: bool,
    pub has_async_io: bool,

    // I/O multiplexing
    pub has_epoll: bool,
    pub has_kqueue: bool,
    pub has_iocp: bool,

    // Advanced file operations
    pub has_sendfile: bool,
    pub has_splice: bool,
    pub has_copy_file_range: bool,
    pub has_fallocate: bool,
    pub has_posix_fadvise: bool,
    pub has_madvise: bool,

    // Security features
    pub has_seccomp: bool,
    pub has_capabilities: bool,
    pub has_namespaces: bool,
    pub has_cgroups: bool,

    // System information
    pub cpu_count: usize,
    pub page_size: usize,
    pub max_path_length: usize,
    pub endianness: String,
    pub filesystem_features: Vec<String>,
    pub network_features: Vec<String>,
    pub security_features: Vec<String>,
    pub virtualization_features: Vec<String>,
    pub hardware_features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Endianness {
    Little,
    Big,
}

impl Platform {
    /// Get the current platform
    #[allow(static_mut_refs)]
    pub fn current() -> Self {
        unsafe {
            if let Some(ref platform) = CURRENT_PLATFORM {
                platform.clone()
            } else {
                // Initialize if not already done
                INIT.call_once(|| {
                    CURRENT_PLATFORM = Some(Platform::detect());
                    CAPABILITIES = Some(Capabilities::detect());
                });

                CURRENT_PLATFORM.clone().unwrap_or({
                    // This should never happen after init(), but provide a fallback
                    Platform::Linux // Use a concrete variant as fallback
                })
            }
        }
    }

    fn detect() -> Self {
        #[cfg(target_os = "linux")]
        return Platform::Linux;
        #[cfg(target_os = "macos")]
        return Platform::MacOS;
        #[cfg(target_os = "windows")]
        return Platform::Windows;
        #[cfg(target_os = "freebsd")]
        return Platform::FreeBSD;
        #[cfg(target_os = "openbsd")]
        return Platform::OpenBSD;
        #[cfg(target_os = "netbsd")]
        return Platform::NetBSD;
        #[cfg(target_os = "solaris")]
        return Platform::Solaris;
        #[cfg(target_os = "android")]
        return Platform::Android;
        #[cfg(not(any(
            target_os = "linux",
            target_os = "macos",
            target_os = "windows",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "solaris",
            target_os = "android"
        )))]
        return Platform::Unknown(std::env::consts::OS.to_string());
    }

    /// Get the name of the current platform
    pub fn name(&self) -> &'static str {
        match self {
            Platform::Linux => "Linux",
            Platform::MacOS => "macOS",
            Platform::Windows => "Windows",
            Platform::FreeBSD => "FreeBSD",
            Platform::OpenBSD => "OpenBSD",
            Platform::NetBSD => "NetBSD",
            Platform::Solaris => "Solaris",
            Platform::Android => "Android",
            Platform::Unknown(_) => "Unknown",
        }
    }

    /// Get the version of the current platform
    pub fn version(&self) -> String {
        std::env::consts::OS.to_string()
    }

    /// Get the architecture of the current platform
    pub fn architecture(&self) -> &'static str {
        std::env::consts::ARCH
    }

    /// Check if the platform is Unix-like
    pub fn is_unix(&self) -> bool {
        matches!(
            self,
            Platform::Linux
                | Platform::MacOS
                | Platform::FreeBSD
                | Platform::OpenBSD
                | Platform::NetBSD
                | Platform::Solaris
                | Platform::Android
        )
    }

    /// Check if the platform is Windows
    pub fn is_windows(&self) -> bool {
        matches!(self, Platform::Windows)
    }

    /// Check if the platform supports POSIX
    pub fn supports_posix(&self) -> bool {
        self.is_unix()
    }

    /// Check if the platform supports a specific feature
    pub fn supports_feature(&self, feature: &str) -> bool {
        // Simplified feature detection based on platform
        match feature {
            "fork" | "exec" | "pipes" | "signals" => self.is_unix(),
            "job_control" | "process_groups" => self.is_unix(),
            "file_locking" | "memory_mapping" => true,
            "shared_memory" | "semaphores" | "message_queues" => self.is_unix(),
            "threads" | "async_io" => true,
            "epoll" => matches!(self, Platform::Linux),
            "kqueue" => matches!(
                self,
                Platform::MacOS | Platform::FreeBSD | Platform::OpenBSD | Platform::NetBSD
            ),
            "iocp" => self.is_windows(),
            "sendfile" | "splice" | "copy_file_range" => matches!(self, Platform::Linux),
            "fallocate" | "posix_fadvise" | "madvise" => self.is_unix(),
            "seccomp" | "capabilities" | "namespaces" | "cgroups" => {
                matches!(self, Platform::Linux)
            }
            _ => false,
        }
    }

    /// Get an environment variable
    pub fn get_environment_variable(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    /// Set an environment variable
    pub fn set_environment_variable(&self, name: &str, value: &str) -> HalResult<()> {
        std::env::set_var(name, value);
        Ok(())
    }

    /// Get all environment variables
    pub fn get_all_environment_variables(&self) -> HashMap<String, String> {
        std::env::vars().collect()
    }

    /// Get the current user
    pub fn get_current_user(&self) -> HalResult<String> {
        Ok(std::env::var("USER").unwrap_or_else(|_| {
            std::env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string())
        }))
    }

    /// Get the hostname
    pub fn get_hostname(&self) -> HalResult<String> {
        Ok(std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string()))
    }

    /// Get system information
    pub fn get_system_info(&self) -> SystemInfo {
        SystemInfo {
            platform: self.clone(),
            cpu_info: self.get_cpu_info(),
            memory_info: self.get_memory_info(),
            disk_info: self.get_disk_info(),
            network_interfaces: self.get_network_interfaces(),
        }
    }

    /// Get CPU information
    pub fn get_cpu_info(&self) -> CpuInfo {
        CpuInfo {
            cpu_count: num_cpus::get(),
            cpu_model: std::env::consts::ARCH.to_string(),
            cpu_frequency: "unknown".to_string(),
            cpu_vendor: "unknown".to_string(),
        }
    }

    /// Get memory information
    pub fn get_memory_info(&self) -> MemoryInfo {
        MemoryInfo {
            total_memory: 0,
            available_memory: 0,
            used_memory: 0,
            free_memory: 0,
        }
    }

    /// Get disk information
    pub fn get_disk_info(&self) -> Vec<DiskInfo> {
        Vec::new()
    }

    /// Get network interfaces with MAC addresses and statistics
    pub fn get_network_interfaces(&self) -> Vec<NetworkInterface> {
        let mut interfaces = Vec::new();

        #[cfg(target_os = "windows")]
        {
            if let Ok(adapters) = self.get_windows_network_adapters() {
                interfaces.extend(adapters);
            }
        }

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        {
            if let Ok(adapters) = self.get_unix_network_interfaces() {
                interfaces.extend(adapters);
            }
        }

        // Fallback: try to get basic interface info
        if interfaces.is_empty() {
            if let Ok(basic_interfaces) = self.get_basic_network_interfaces() {
                interfaces.extend(basic_interfaces);
            }
        }

        interfaces
    }

    /// Check if the current user is root
    pub fn is_root(&self) -> bool {
        self.get_current_user().unwrap_or_default() == "root"
    }

    /// Check if a file can be executed
    pub fn can_execute(&self, path: &std::path::Path) -> bool {
        path.exists() && path.is_file()
    }

    /// Get file permissions
    pub fn get_file_permissions(&self, path: &std::path::Path) -> Option<u32> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            path.metadata().map(|m| m.permissions().mode()).ok()
        }
        #[cfg(not(unix))]
        {
            let _ = path;
            None
        }
    }

    /// Set file permissions
    pub fn set_file_permissions(&self, path: &std::path::Path, mode: u32) -> HalResult<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(mode);
            std::fs::set_permissions(path, permissions)?;
        }
        #[cfg(not(unix))]
        {
            let _ = (path, mode);
        }
        Ok(())
    }
}

impl Capabilities {
    /// Get the current platform capabilities
    #[allow(static_mut_refs)]
    pub fn current() -> Self {
        unsafe {
            CAPABILITIES
                .as_ref()
                .expect("Platform capabilities not initialized")
                .clone()
        }
    }

    fn detect() -> Self {
        let platform = Platform::detect();
        Self {
            has_fork: platform.is_unix(),
            has_exec: platform.is_unix(),
            has_pipes: true,
            has_signals: platform.is_unix(),
            has_job_control: platform.is_unix(),
            has_process_groups: platform.is_unix(),
            has_file_locking: true,
            has_memory_mapping: true,
            has_shared_memory: platform.is_unix(),
            has_semaphores: platform.is_unix(),
            has_message_queues: platform.is_unix(),
            has_threads: true,
            has_async_io: true,
            has_epoll: matches!(platform, Platform::Linux),
            has_kqueue: matches!(
                platform,
                Platform::MacOS | Platform::FreeBSD | Platform::OpenBSD | Platform::NetBSD
            ),
            has_iocp: platform.is_windows(),
            has_sendfile: matches!(platform, Platform::Linux),
            has_splice: matches!(platform, Platform::Linux),
            has_copy_file_range: matches!(platform, Platform::Linux),
            has_fallocate: platform.is_unix(),
            has_posix_fadvise: platform.is_unix(),
            has_madvise: platform.is_unix(),
            has_seccomp: matches!(platform, Platform::Linux),
            has_capabilities: matches!(platform, Platform::Linux),
            has_namespaces: matches!(platform, Platform::Linux),
            has_cgroups: matches!(platform, Platform::Linux),
            cpu_count: num_cpus::get(),
            page_size: detect_page_size(),
            max_path_length: detect_max_path_length(&platform),
            endianness: if cfg!(target_endian = "big") {
                "big".to_string()
            } else {
                "little".to_string()
            },
            filesystem_features: detect_filesystem_features(&platform),
            network_features: vec!["tcp".to_string(), "udp".to_string()],
            security_features: if platform.is_unix() {
                vec!["unix_permissions".to_string()]
            } else {
                vec![]
            },
            virtualization_features: vec![],
            hardware_features: vec![],
        }
    }

    /// Check if a specific capability is available
    pub fn has_capability(&self, capability: &str) -> bool {
        match capability {
            "fork" => self.has_fork,
            "exec" => self.has_exec,
            "pipes" => self.has_pipes,
            "signals" => self.has_signals,
            "job_control" => self.has_job_control,
            "process_groups" => self.has_process_groups,
            "file_locking" => self.has_file_locking,
            "memory_mapping" => self.has_memory_mapping,
            "shared_memory" => self.has_shared_memory,
            "semaphores" => self.has_semaphores,
            "message_queues" => self.has_message_queues,
            "threads" => self.has_threads,
            "async_io" => self.has_async_io,
            "epoll" => self.has_epoll,
            "kqueue" => self.has_kqueue,
            "iocp" => self.has_iocp,
            "sendfile" => self.has_sendfile,
            "splice" => self.has_splice,
            "copy_file_range" => self.has_copy_file_range,
            "fallocate" => self.has_fallocate,
            "posix_fadvise" => self.has_posix_fadvise,
            "madvise" => self.has_madvise,
            "seccomp" => self.has_seccomp,
            "capabilities" => self.has_capabilities,
            "namespaces" => self.has_namespaces,
            "cgroups" => self.has_cgroups,
            _ => false,
        }
    }

    /// Get filesystem-specific feature availability
    pub fn filesystem_feature(&self, feature: &str) -> bool {
        self.filesystem_features.contains(&feature.to_string())
    }
}

/// Detect the current platform
pub fn detect_platform() -> Platform {
    #[cfg(target_os = "linux")]
    return Platform::Linux;

    #[cfg(target_os = "macos")]
    return Platform::MacOS;

    #[cfg(target_os = "windows")]
    return Platform::Windows;

    #[cfg(target_os = "freebsd")]
    return Platform::FreeBSD;

    #[cfg(target_os = "openbsd")]
    return Platform::OpenBSD;

    #[cfg(target_os = "netbsd")]
    return Platform::NetBSD;

    #[cfg(target_os = "solaris")]
    return Platform::Solaris;

    #[cfg(target_os = "android")]
    return Platform::Android;

    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "solaris",
        target_os = "android"
    )))]
    return Platform::Unknown(std::env::consts::OS.to_string());
}

/// Detect platform capabilities
pub fn detect_capabilities(platform: &Platform) -> Capabilities {
    let mut caps = Capabilities {
        has_fork: false,
        has_exec: false,
        has_pipes: false,
        has_signals: false,
        has_job_control: false,
        has_process_groups: false,
        has_file_locking: false,
        has_memory_mapping: false,
        has_shared_memory: false,
        has_semaphores: false,
        has_message_queues: false,
        has_threads: true, // Rust always has threads
        has_async_io: false,
        has_epoll: false,
        has_kqueue: false,
        has_iocp: false,
        has_sendfile: false,
        has_splice: false,
        has_copy_file_range: false,
        has_fallocate: false,
        has_posix_fadvise: false,
        has_madvise: false,
        has_seccomp: false,
        has_capabilities: false,
        has_namespaces: false,
        has_cgroups: false,
        cpu_count: num_cpus::get(),
        page_size: detect_page_size(),
        max_path_length: detect_max_path_length(platform),
        endianness: if cfg!(target_endian = "little") {
            "Little".to_string()
        } else {
            "Big".to_string()
        },
        filesystem_features: Vec::new(),
        network_features: Vec::new(),
        security_features: Vec::new(),
        virtualization_features: Vec::new(),
        hardware_features: Vec::new(),
    };

    match platform {
        Platform::Linux => {
            caps.has_fork = true;
            caps.has_exec = true;
            caps.has_pipes = true;
            caps.has_signals = true;
            caps.has_job_control = true;
            caps.has_process_groups = true;
            caps.has_file_locking = true;
            caps.has_memory_mapping = true;
            caps.has_shared_memory = true;
            caps.has_semaphores = true;
            caps.has_message_queues = true;
            caps.has_async_io = true;
            caps.has_epoll = true;
            caps.has_sendfile = true;
            caps.has_splice = true;
            caps.has_copy_file_range = true;
            caps.has_fallocate = true;
            caps.has_posix_fadvise = true;
            caps.has_madvise = true;
            caps.has_seccomp = true;
            caps.has_capabilities = true;
            caps.has_namespaces = true;
            caps.has_cgroups = true;
            caps.filesystem_features = vec![
                "extended_attributes".to_string(),
                "case_sensitive".to_string(),
                "hard_links".to_string(),
                "symbolic_links".to_string(),
                "file_holes".to_string(),
                "reflinks".to_string(),
                "compression".to_string(),
                "encryption".to_string(),
            ];
            caps.network_features = vec!["ipv4".to_string(), "ipv6".to_string()];
            caps.security_features = vec!["apparmor".to_string(), "selinux".to_string()];
            caps.virtualization_features = vec!["kvm".to_string(), "vmware".to_string()];
            caps.hardware_features = vec![
                "sse".to_string(),
                "sse2".to_string(),
                "sse3".to_string(),
                "ssse3".to_string(),
                "sse4.1".to_string(),
                "sse4.2".to_string(),
                "avx".to_string(),
                "avx2".to_string(),
            ];
        }
        Platform::MacOS => {
            caps.has_fork = true;
            caps.has_exec = true;
            caps.has_pipes = true;
            caps.has_signals = true;
            caps.has_job_control = true;
            caps.has_process_groups = true;
            caps.has_file_locking = true;
            caps.has_memory_mapping = true;
            caps.has_shared_memory = true;
            caps.has_semaphores = true;
            caps.has_message_queues = true;
            caps.has_async_io = true;
            caps.has_kqueue = true;
            caps.has_sendfile = true;
            caps.has_posix_fadvise = true;
            caps.has_madvise = true;
            caps.filesystem_features = vec![
                "extended_attributes".to_string(),
                "case_sensitive".to_string(),
                "hard_links".to_string(),
                "symbolic_links".to_string(),
                "file_holes".to_string(),
                "resource_forks".to_string(),
                "compression".to_string(),
                "encryption".to_string(),
            ];
            caps.network_features = vec!["ipv4".to_string(), "ipv6".to_string()];
            caps.security_features = vec!["file_integrity".to_string(), "gatekeeper".to_string()];
            caps.virtualization_features = vec!["hypervisor".to_string(), "vmware".to_string()];
            caps.hardware_features = vec![
                "sse".to_string(),
                "sse2".to_string(),
                "sse3".to_string(),
                "ssse3".to_string(),
                "sse4.1".to_string(),
                "sse4.2".to_string(),
                "avx".to_string(),
                "avx2".to_string(),
            ];
        }
        Platform::Windows => {
            caps.has_pipes = true; // Named pipes
            caps.has_file_locking = true;
            caps.has_memory_mapping = true;
            caps.has_shared_memory = true;
            caps.has_semaphores = true;
            caps.has_async_io = true;
            caps.has_iocp = true;
            caps.filesystem_features = vec![
                "case_sensitive".to_string(),
                "hard_links".to_string(),
                "symbolic_links".to_string(),
                "file_holes".to_string(),
                "alternate_streams".to_string(),
                "compression".to_string(),
                "encryption".to_string(),
                "reparse_points".to_string(),
            ];
            caps.network_features = vec!["ipv4".to_string(), "ipv6".to_string()];
            caps.security_features = vec!["firewall".to_string(), "antivirus".to_string()];
            caps.virtualization_features = vec!["hypervisor".to_string(), "vmware".to_string()];
            caps.hardware_features = vec![
                "sse".to_string(),
                "sse2".to_string(),
                "sse3".to_string(),
                "ssse3".to_string(),
                "sse4.1".to_string(),
                "sse4.2".to_string(),
                "avx".to_string(),
                "avx2".to_string(),
            ];
        }
        Platform::FreeBSD | Platform::OpenBSD | Platform::NetBSD => {
            caps.has_fork = true;
            caps.has_exec = true;
            caps.has_pipes = true;
            caps.has_signals = true;
            caps.has_job_control = true;
            caps.has_process_groups = true;
            caps.has_file_locking = true;
            caps.has_memory_mapping = true;
            caps.has_shared_memory = true;
            caps.has_semaphores = true;
            caps.has_message_queues = true;
            caps.has_async_io = true;
            caps.has_kqueue = true;
            caps.has_sendfile = true;
            caps.has_posix_fadvise = true;
            caps.has_madvise = true;
            caps.filesystem_features = vec![
                "extended_attributes".to_string(),
                "case_sensitive".to_string(),
                "hard_links".to_string(),
                "symbolic_links".to_string(),
                "file_holes".to_string(),
                "compression".to_string(),
                "encryption".to_string(),
            ];
            caps.network_features = vec!["ipv4".to_string(), "ipv6".to_string()];
            caps.security_features = vec!["apparmor".to_string(), "selinux".to_string()];
            caps.virtualization_features = vec!["kvm".to_string(), "vmware".to_string()];
            caps.hardware_features = vec![
                "sse".to_string(),
                "sse2".to_string(),
                "sse3".to_string(),
                "ssse3".to_string(),
                "sse4.1".to_string(),
                "sse4.2".to_string(),
                "avx".to_string(),
                "avx2".to_string(),
            ];
        }
        _ => {
            // Conservative defaults for unknown platforms
            caps.filesystem_features = vec![
                "case_sensitive".to_string(),
                "hard_links".to_string(),
                "symbolic_links".to_string(),
            ];
            caps.network_features = vec!["ipv4".to_string(), "ipv6".to_string()];
            caps.security_features = vec!["firewall".to_string(), "antivirus".to_string()];
            caps.virtualization_features = vec!["hypervisor".to_string(), "vmware".to_string()];
            caps.hardware_features = vec![
                "sse".to_string(),
                "sse2".to_string(),
                "sse3".to_string(),
                "ssse3".to_string(),
                "sse4.1".to_string(),
                "sse4.2".to_string(),
                "avx".to_string(),
                "avx2".to_string(),
            ];
        }
    }

    caps
}

/// Detect system page size
fn detect_page_size() -> usize {
    #[cfg(unix)]
    {
        // Use safe alternative to get page size instead of direct libc/nix calls
        // Try to read from getconf if available, otherwise use reasonable defaults
        match std::process::Command::new("getconf")
            .arg("PAGE_SIZE")
            .output()
        {
            Ok(output) if output.status.success() => {
                if let Ok(size_str) = String::from_utf8(output.stdout) {
                    if let Ok(size) = size_str.trim().parse::<usize>() {
                        return size;
                    }
                }
            }
            _ => {}
        }

        // Fallback: Use standard defaults based on common Unix systems
        #[cfg(target_arch = "x86_64")]
        return 4096;
        #[cfg(target_arch = "aarch64")]
        return 4096;
        #[cfg(target_arch = "arm")]
        return 4096;
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "arm")))]
        return 4096; // Safe default for other architectures
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
        unsafe {
            let mut info: SYSTEM_INFO = std::mem::zeroed();
            GetSystemInfo(&mut info);
            info.dwPageSize as usize
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        4096 // Default assumption
    }
}

/// Detect maximum path length for the platform
fn detect_max_path_length(platform: &Platform) -> usize {
    match platform {
        Platform::Windows => 260, // MAX_PATH on Windows
        Platform::Linux => 4096,  // PATH_MAX on Linux
        Platform::MacOS => 1024,  // PATH_MAX on macOS
        _ => 1024,                // Conservative default
    }
}

/// Detect filesystem-specific features
fn detect_filesystem_features(platform: &Platform) -> Vec<String> {
    let mut features = Vec::new();

    match platform {
        Platform::Linux => {
            features.push("extended_attributes".to_string());
            features.push("case_sensitive".to_string());
            features.push("hard_links".to_string());
            features.push("symbolic_links".to_string());
            features.push("file_holes".to_string());
            features.push("reflinks".to_string());
            features.push("compression".to_string());
            features.push("encryption".to_string());
        }
        Platform::MacOS => {
            features.push("extended_attributes".to_string());
            features.push("case_sensitive".to_string()); // HFS+ default
            features.push("hard_links".to_string());
            features.push("symbolic_links".to_string());
            features.push("file_holes".to_string());
            features.push("resource_forks".to_string());
            features.push("compression".to_string());
            features.push("encryption".to_string());
        }
        Platform::Windows => {
            features.push("case_sensitive".to_string());
            features.push("hard_links".to_string());
            features.push("symbolic_links".to_string());
            features.push("file_holes".to_string());
            features.push("alternate_streams".to_string());
            features.push("compression".to_string());
            features.push("encryption".to_string());
            features.push("reparse_points".to_string());
        }
        _ => {
            // Conservative defaults
            features.push("case_sensitive".to_string());
            features.push("hard_links".to_string());
            features.push("symbolic_links".to_string());
        }
    }

    features
}

impl Platform {
    /// Get network interfaces on Windows
    #[cfg(target_os = "windows")]
    fn get_windows_network_adapters(
        &self,
    ) -> Result<Vec<NetworkInterface>, Box<dyn std::error::Error>> {
        use std::process::Command;

        let output = Command::new("netsh")
            .args(["interface", "show", "interface"])
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut interfaces = Vec::new();

        for line in output_str.lines().skip(3) {
            // Skip header lines
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let name = parts[3..].join(" ");
                let is_up = parts[0] == "Enabled";

                let interface = NetworkInterface {
                    name,
                    is_up,
                    is_loopback: false, // Will be determined later
                    ip_addresses: Vec::new(),
                    mac_address: self.get_windows_mac_address(&parts[3..].join(" ")).ok(),
                    mtu: None,
                    statistics: None,
                };

                interfaces.push(interface);
            }
        }

        Ok(interfaces)
    }

    /// Get MAC address on Windows
    #[cfg(target_os = "windows")]
    fn get_windows_mac_address(
        &self,
        interface_name: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use std::process::Command;

        let output = Command::new("getmac")
            .args(["/fo", "csv", "/nh"])
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains(interface_name) {
                let parts: Vec<&str> = line.split(',').collect();
                if !parts.is_empty() {
                    return Ok(parts[0].trim_matches('"').to_string());
                }
            }
        }

        Err("MAC address not found".into())
    }

    /// Get network interfaces on Unix systems
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
    fn get_unix_network_interfaces(
        &self,
    ) -> Result<Vec<NetworkInterface>, Box<dyn std::error::Error>> {
        use std::process::Command;

        let output = Command::new("ip")
            .args(&["link", "show"])
            .output()
            .or_else(|_| Command::new("ifconfig").arg("-a").output())?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut interfaces = Vec::new();

        // Parse ip link output
        for line in output_str.lines() {
            if line.starts_with(char::is_numeric) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[1].trim_end_matches(':').to_string();
                    let is_up = line.contains("UP");
                    let is_loopback = line.contains("LOOPBACK");

                    let mac_address = self.extract_mac_from_line(&line);
                    let mtu = self.extract_mtu_from_line(&line);

                    let interface = NetworkInterface {
                        name,
                        is_up,
                        is_loopback,
                        ip_addresses: Vec::new(), // Would need separate call to get IPs
                        mac_address,
                        mtu,
                        statistics: None, // Would need /proc/net/dev parsing on Linux
                    };

                    interfaces.push(interface);
                }
            }
        }

        Ok(interfaces)
    }

    /// Extract MAC address from interface line
    #[allow(dead_code)]
    fn extract_mac_from_line(&self, line: &str) -> Option<String> {
        // Look for MAC address pattern (XX:XX:XX:XX:XX:XX)
        let mac_regex = regex::Regex::new(r"([0-9a-fA-F]{2}:[0-9a-fA-F]{2}:[0-9a-fA-F]{2}:[0-9a-fA-F]{2}:[0-9a-fA-F]{2}:[0-9a-fA-F]{2})").ok()?;

        if let Some(captures) = mac_regex.captures(line) {
            return Some(captures[1].to_string());
        }

        None
    }

    /// Extract MTU from interface line
    #[allow(dead_code)]
    fn extract_mtu_from_line(&self, line: &str) -> Option<u32> {
        if let Some(mtu_start) = line.find("mtu ") {
            let mtu_part = &line[mtu_start + 4..];
            if let Some(space_pos) = mtu_part.find(' ') {
                if let Ok(mtu) = mtu_part[..space_pos].parse::<u32>() {
                    return Some(mtu);
                }
            }
        }

        None
    }

    /// Get basic network interfaces using platform-independent methods
    fn get_basic_network_interfaces(
        &self,
    ) -> Result<Vec<NetworkInterface>, Box<dyn std::error::Error>> {
        let mut interfaces = Vec::new();

        // Try to get interface names from /sys/class/net (Linux)
        #[cfg(target_os = "linux")]
        {
            if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        let is_up = self.is_interface_up_linux(name);
                        let is_loopback = name == "lo";

                        let interface = NetworkInterface {
                            name: name.to_string(),
                            is_up,
                            is_loopback,
                            ip_addresses: Vec::new(),
                            mac_address: self.get_linux_mac_address(name).ok(),
                            mtu: self.get_linux_mtu(name).ok(),
                            statistics: self.get_linux_interface_stats(name).ok(),
                        };

                        interfaces.push(interface);
                    }
                }
            }
        }

        // Fallback: create a minimal loopback interface
        if interfaces.is_empty() {
            interfaces.push(NetworkInterface {
                name: "lo".to_string(),
                is_up: true,
                is_loopback: true,
                ip_addresses: vec!["127.0.0.1".to_string()],
                mac_address: None,
                mtu: Some(65536),
                statistics: None,
            });
        }

        Ok(interfaces)
    }

    /// Check if interface is up on Linux
    #[cfg(target_os = "linux")]
    fn is_interface_up_linux(&self, interface: &str) -> bool {
        std::fs::read_to_string(format!("/sys/class/net/{}/operstate", interface))
            .map(|state| state.trim() == "up")
            .unwrap_or(false)
    }

    /// Get MAC address on Linux
    #[cfg(target_os = "linux")]
    fn get_linux_mac_address(&self, interface: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mac = std::fs::read_to_string(format!("/sys/class/net/{}/address", interface))?;
        Ok(mac.trim().to_string())
    }

    /// Get MTU on Linux
    #[cfg(target_os = "linux")]
    fn get_linux_mtu(&self, interface: &str) -> Result<u32, Box<dyn std::error::Error>> {
        let mtu_str = std::fs::read_to_string(format!("/sys/class/net/{}/mtu", interface))?;
        Ok(mtu_str.trim().parse()?)
    }

    /// Get interface statistics on Linux
    #[cfg(target_os = "linux")]
    fn get_linux_interface_stats(
        &self,
        interface: &str,
    ) -> Result<NetworkStatistics, Box<dyn std::error::Error>> {
        let stats_dir = format!("/sys/class/net/{}/statistics", interface);

        let rx_bytes = std::fs::read_to_string(format!("{}/rx_bytes", stats_dir))?
            .trim()
            .parse()?;
        let tx_bytes = std::fs::read_to_string(format!("{}/tx_bytes", stats_dir))?
            .trim()
            .parse()?;
        let rx_packets = std::fs::read_to_string(format!("{}/rx_packets", stats_dir))?
            .trim()
            .parse()?;
        let tx_packets = std::fs::read_to_string(format!("{}/tx_packets", stats_dir))?
            .trim()
            .parse()?;
        let rx_errors = std::fs::read_to_string(format!("{}/rx_errors", stats_dir))?
            .trim()
            .parse()?;
        let tx_errors = std::fs::read_to_string(format!("{}/tx_errors", stats_dir))?
            .trim()
            .parse()?;

        Ok(NetworkStatistics {
            rx_bytes,
            tx_bytes,
            rx_packets,
            tx_packets,
            rx_errors,
            tx_errors,
        })
    }
}

/// Network interface statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatistics {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

/// Initialize platform detection and capabilities
pub fn initialize_platform() -> HalResult<()> {
    INIT.call_once(|| {
        let platform = detect_platform();
        let capabilities = detect_capabilities(&platform);

        unsafe {
            CURRENT_PLATFORM = Some(platform);
            CAPABILITIES = Some(capabilities);
        }
    });

    Ok(())
}

/// Cleanup platform resources
pub fn cleanup_platform() -> HalResult<()> {
    // Currently no cleanup needed, but reserved for future use
    Ok(())
}
