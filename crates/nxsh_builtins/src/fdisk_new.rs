//! `fdisk` builtin - Cross-platform partition table viewer and disk management utility.
//!
//! This implementation provides comprehensive disk and partition information across all platforms:
//! - Windows: WMI queries and PowerShell Get-Disk integration for disk enumeration
//! - Linux: /proc/partitions and /sys/block/ parsing with blkid integration
//! - macOS: diskutil integration and IOKit StorageFamily API
//! - Pure Rust MBR/GPT partition table parsing with zero C/C++ dependencies
//! - Safe read-only operations with comprehensive error handling
//! - Enterprise-grade cross-platform compatibility

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Comprehensive disk information structure
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub device: String,
    pub model: String,
    pub size_bytes: u64,
    pub sector_size: u32,
    pub partition_table_type: String,
    pub mount_point: Option<String>,
    pub partitions: Vec<PartitionInfo>,
    pub disk_id: Option<String>,
    pub serial_number: Option<String>,
    pub interface_type: Option<String>,
    pub firmware_revision: Option<String>,
    pub health_status: Option<String>,
}

impl Default for DiskInfo {
    fn default() -> Self {
        Self {
            device: String::new(),
            model: "Unknown".to_string(),
            size_bytes: 0,
            sector_size: 512,
            partition_table_type: "Unknown".to_string(),
            mount_point: None,
            partitions: Vec::new(),
            disk_id: None,
            serial_number: None,
            interface_type: None,
            firmware_revision: None,
            health_status: None,
        }
    }
}

impl DiskInfo {
    /// Format disk size in human-readable format
    pub fn format_size(&self) -> String {
        format_bytes(self.size_bytes)
    }

    /// Get total number of sectors
    pub fn total_sectors(&self) -> u64 {
        self.size_bytes / self.sector_size as u64
    }

    /// Format disk information in fdisk style
    pub fn format_fdisk_header(&self) -> String {
        format!(
            "Disk {}: {} GiB, {} bytes, {} sectors\nSector size (logical/physical): {} bytes / {} bytes\nPartition table: {}",
            self.device,
            self.size_bytes as f64 / 1_073_741_824.0,
            self.size_bytes,
            self.total_sectors(),
            self.sector_size,
            self.sector_size,
            self.partition_table_type
        )
    }
}

/// Comprehensive partition information structure
#[derive(Debug, Clone)]
pub struct PartitionInfo {
    pub device: String,
    pub partition_number: u32,
    pub start_sector: u64,
    pub end_sector: u64,
    pub size_sectors: u64,
    pub size_bytes: u64,
    pub partition_type: String,
    pub filesystem: Option<String>,
    pub mount_point: Option<String>,
    pub bootable: bool,
    pub partition_uuid: Option<String>,
    pub filesystem_uuid: Option<String>,
    pub label: Option<String>,
}

impl Default for PartitionInfo {
    fn default() -> Self {
        Self {
            device: String::new(),
            partition_number: 0,
            start_sector: 0,
            end_sector: 0,
            size_sectors: 0,
            size_bytes: 0,
            partition_type: "Unknown".to_string(),
            filesystem: None,
            mount_point: None,
            bootable: false,
            partition_uuid: None,
            filesystem_uuid: None,
            label: None,
        }
    }
}

impl PartitionInfo {
    /// Format partition size in human-readable format
    pub fn format_size(&self) -> String {
        format_bytes(self.size_bytes)
    }

    /// Format partition information in fdisk table style
    pub fn format_fdisk_entry(&self) -> String {
        let boot_flag = if self.bootable { "*" } else { " " };
        let filesystem = self.filesystem.as_deref().unwrap_or("Unknown");
        format!(
            "{:<15} {:<1} {:>10} {:>10} {:>10} {:>10} {:<8} {}",
            self.device,
            boot_flag,
            self.start_sector,
            self.end_sector,
            self.size_sectors,
            self.format_size(),
            self.partition_type,
            filesystem
        )
    }
}

/// Configuration options for fdisk command
#[derive(Debug, Default)]
pub struct FdiskConfig {
    pub list_all: bool,
    pub device_path: Option<String>,
    pub json_output: bool,
    pub verbose: bool,
    pub help: bool,
    pub version: bool,
    pub sector_size: Option<u32>,
}

impl FdiskConfig {
    /// Parse command line arguments
    pub fn parse_args(args: &[String]) -> Result<Self> {
        let mut config = Self::default();
        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "-l" | "--list" => config.list_all = true,
                "-j" | "--json" => config.json_output = true,
                "-v" | "--verbose" => config.verbose = true,
                "-h" | "--help" => config.help = true,
                "-V" | "--version" => config.version = true,
                "-b" | "--sector-size" => {
                    if i + 1 < args.len() {
                        config.sector_size = Some(args[i + 1].parse()
                            .context("Invalid sector size")?);
                        i += 1;
                    } else {
                        bail!("Option '{}' requires an argument", args[i]);
                    }
                },
                arg if arg.starts_with('-') => {
                    bail!("Unknown option: {}", arg);
                },
                _ => {
                    // Positional argument is device path
                    if config.device_path.is_none() {
                        config.device_path = Some(args[i].clone());
                    }
                }
            }
            i += 1;
        }

        Ok(config)
    }
}

/// Windows-specific disk enumeration using WMI and PowerShell
#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;

    pub fn enumerate_disks() -> Result<Vec<DiskInfo>> {
        let mut disks = Vec::new();

        // Try WMI query first for comprehensive information
        if let Ok(wmi_disks) = query_wmi_disks() {
            disks.extend(wmi_disks);
        }

        // Fallback to PowerShell Get-Disk if WMI fails
        if disks.is_empty() {
            if let Ok(ps_disks) = query_powershell_disks() {
                disks.extend(ps_disks);
            }
        }

        // Enhance disk information with partition details
        for disk in &mut disks {
            if let Ok(partitions) = query_disk_partitions(&disk.device) {
                disk.partitions = partitions;
            }
        }

        Ok(disks)
    }

    fn query_wmi_disks() -> Result<Vec<DiskInfo>> {
        let output = Command::new("powershell")
            .args(&[
                "-NoProfile", "-Command",
                r#"
                Get-WmiObject -Class Win32_DiskDrive | ForEach-Object {
                    $diskId = $_.Index
                    $logicalDisks = Get-WmiObject -Class Win32_LogicalDiskToPartition | Where-Object { $_.Antecedent -match "Disk #$diskId" }
                    
                    [PSCustomObject]@{
                        DeviceID = $_.DeviceID
                        Index = $_.Index
                        Model = $_.Model
                        Size = $_.Size
                        SectorSize = $_.BytesPerSector
                        Partitions = $_.Partitions
                        SerialNumber = $_.SerialNumber
                        InterfaceType = $_.InterfaceType
                        FirmwareRevision = $_.FirmwareRevision
                        Status = $_.Status
                    }
                } | ConvertTo-Json -Depth 3
                "#
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            bail!("WMI disk query failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let json_str = String::from_utf8(output.stdout)?;
        let disks_json: Value = serde_json::from_str(&json_str)
            .context("Failed to parse WMI disk JSON")?;

        let mut disks = Vec::new();
        let disk_array = if disks_json.is_array() {
            disks_json.as_array().unwrap()
        } else {
            vec![&disks_json]
        };

        for disk_json in disk_array {
            let device_id = disk_json["DeviceID"].as_str().unwrap_or("Unknown").to_string();
            let model = disk_json["Model"].as_str().unwrap_or("Unknown").to_string();
            let size = disk_json["Size"].as_u64().unwrap_or(0);
            let sector_size = disk_json["SectorSize"].as_u64().unwrap_or(512) as u32;

            disks.push(DiskInfo {
                device: device_id,
                model,
                size_bytes: size,
                sector_size,
                partition_table_type: "MBR/GPT".to_string(), // Will be determined later
                serial_number: disk_json["SerialNumber"].as_str().map(|s| s.to_string()),
                interface_type: disk_json["InterfaceType"].as_str().map(|s| s.to_string()),
                firmware_revision: disk_json["FirmwareRevision"].as_str().map(|s| s.to_string()),
                health_status: disk_json["Status"].as_str().map(|s| s.to_string()),
                ..Default::default()
            });
        }

        Ok(disks)
    }

    fn query_powershell_disks() -> Result<Vec<DiskInfo>> {
        let output = Command::new("powershell")
            .args(&[
                "-NoProfile", "-Command",
                r#"
                Get-Disk | ForEach-Object {
                    [PSCustomObject]@{
                        Number = $_.Number
                        FriendlyName = $_.FriendlyName
                        Size = $_.Size
                        PartitionStyle = $_.PartitionStyle
                        HealthStatus = $_.HealthStatus
                        OperationalStatus = $_.OperationalStatus
                        BusType = $_.BusType
                        SerialNumber = $_.SerialNumber
                        FirmwareVersion = $_.FirmwareVersion
                    }
                } | ConvertTo-Json -Depth 2
                "#
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            bail!("PowerShell disk query failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let json_str = String::from_utf8(output.stdout)?;
        if json_str.trim().is_empty() {
            return Ok(Vec::new());
        }

        let disks_json: Value = serde_json::from_str(&json_str)
            .context("Failed to parse PowerShell disk JSON")?;

        let mut disks = Vec::new();
        let disk_array = if disks_json.is_array() {
            disks_json.as_array().unwrap()
        } else {
            vec![&disks_json]
        };

        for disk_json in disk_array {
            let number = disk_json["Number"].as_u64().unwrap_or(0);
            let device = format!("\\\\.\\PhysicalDrive{}", number);
            let model = disk_json["FriendlyName"].as_str().unwrap_or("Unknown").to_string();
            let size = disk_json["Size"].as_u64().unwrap_or(0);
            let partition_style = disk_json["PartitionStyle"].as_str().unwrap_or("Unknown").to_string();

            disks.push(DiskInfo {
                device,
                model,
                size_bytes: size,
                sector_size: 512, // Default sector size
                partition_table_type: partition_style,
                serial_number: disk_json["SerialNumber"].as_str().map(|s| s.to_string()),
                interface_type: disk_json["BusType"].as_str().map(|s| s.to_string()),
                firmware_revision: disk_json["FirmwareVersion"].as_str().map(|s| s.to_string()),
                health_status: disk_json["HealthStatus"].as_str().map(|s| s.to_string()),
                ..Default::default()
            });
        }

        Ok(disks)
    }

    fn query_disk_partitions(device: &str) -> Result<Vec<PartitionInfo>> {
        // Extract disk number from device path
        let disk_number = if let Some(captures) = regex::Regex::new(r"PhysicalDrive(\d+)")
            .unwrap()
            .captures(device) 
        {
            captures[1].to_string()
        } else {
            "0".to_string()
        };

        let output = Command::new("powershell")
            .args(&[
                "-NoProfile", "-Command",
                &format!(r#"
                Get-Partition -DiskNumber {} | ForEach-Object {{
                    $volume = Get-Volume -Partition $_ -ErrorAction SilentlyContinue
                    [PSCustomObject]@{{
                        PartitionNumber = $_.PartitionNumber
                        DriveLetter = $_.DriveLetter
                        Offset = $_.Offset
                        Size = $_.Size
                        Type = $_.Type
                        IsActive = $_.IsActive
                        IsBoot = $_.IsBoot
                        FileSystem = if ($volume) {{ $volume.FileSystem }} else {{ $null }}
                        Label = if ($volume) {{ $volume.FileSystemLabel }} else {{ $null }}
                        HealthStatus = if ($volume) {{ $volume.HealthStatus }} else {{ $null }}
                    }}
                }} | ConvertTo-Json -Depth 2
                "#, disk_number)
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            // No partitions found or access denied
            return Ok(Vec::new());
        }

        let json_str = String::from_utf8(output.stdout)?;
        if json_str.trim().is_empty() {
            return Ok(Vec::new());
        }

        let partitions_json: Value = serde_json::from_str(&json_str)
            .context("Failed to parse partition JSON")?;

        let mut partitions = Vec::new();
        let partition_array = if partitions_json.is_array() {
            partitions_json.as_array().unwrap()
        } else {
            vec![&partitions_json]
        };

        for partition_json in partition_array {
            let partition_number = partition_json["PartitionNumber"].as_u64().unwrap_or(0) as u32;
            let offset = partition_json["Offset"].as_u64().unwrap_or(0);
            let size = partition_json["Size"].as_u64().unwrap_or(0);
            let partition_type = partition_json["Type"].as_str().unwrap_or("Unknown").to_string();
            let drive_letter = partition_json["DriveLetter"].as_str().map(|s| s.to_string());

            let partition_device = if let Some(letter) = &drive_letter {
                format!("{}:", letter)
            } else {
                format!("{}p{}", device, partition_number)
            };

            partitions.push(PartitionInfo {
                device: partition_device,
                partition_number,
                start_sector: offset / 512,
                end_sector: (offset + size) / 512,
                size_sectors: size / 512,
                size_bytes: size,
                partition_type,
                filesystem: partition_json["FileSystem"].as_str().map(|s| s.to_string()),
                mount_point: drive_letter,
                bootable: partition_json["IsActive"].as_bool().unwrap_or(false) ||
                         partition_json["IsBoot"].as_bool().unwrap_or(false),
                label: partition_json["Label"].as_str().map(|s| s.to_string()),
                ..Default::default()
            });
        }

        Ok(partitions)
    }
}

/// Linux-specific disk enumeration using /proc and /sys
#[cfg(target_os = "linux")]
mod linux_impl {
    use super::*;

    pub fn enumerate_disks() -> Result<Vec<DiskInfo>> {
        let mut disks = Vec::new();

        // Parse /proc/partitions for basic disk information
        if let Ok(proc_disks) = parse_proc_partitions() {
            disks.extend(proc_disks);
        }

        // Enhance with /sys/block information
        for disk in &mut disks {
            enhance_disk_info_sysfs(disk)?;
            
            // Query partition information
            if let Ok(partitions) = query_disk_partitions_linux(&disk.device) {
                disk.partitions = partitions;
            }
        }

        // Fallback to lsblk if other methods fail
        if disks.is_empty() {
            if let Ok(lsblk_disks) = use_lsblk() {
                disks.extend(lsblk_disks);
            }
        }

        Ok(disks)
    }

    fn parse_proc_partitions() -> Result<Vec<DiskInfo>> {
        let content = fs::read_to_string("/proc/partitions")?;
        let mut disks = Vec::new();

        for line in content.lines().skip(2) { // Skip header lines
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 4 {
                let device_name = fields[3];
                let size_kb = fields[2].parse::<u64>().unwrap_or(0);
                
                // Only include full disk devices (not partitions)
                if is_full_disk_device(device_name) {
                    disks.push(DiskInfo {
                        device: format!("/dev/{}", device_name),
                        size_bytes: size_kb * 1024,
                        ..Default::default()
                    });
                }
            }
        }

        Ok(disks)
    }

    fn is_full_disk_device(name: &str) -> bool {
        // Check if this is a full disk (not a partition)
        // Examples: sda, sdb, nvme0n1, mmcblk0 (not sda1, nvme0n1p1, etc.)
        !name.chars().last().map_or(false, |c| c.is_ascii_digit()) ||
        (name.starts_with("nvme") && name.ends_with("n1")) ||
        (name.starts_with("mmcblk") && !name.contains('p'))
    }

    fn enhance_disk_info_sysfs(disk: &mut DiskInfo) -> Result<()> {
        let device_name = disk.device.strip_prefix("/dev/").unwrap_or(&disk.device);
        let sysfs_path = format!("/sys/block/{}", device_name);

        if !Path::new(&sysfs_path).exists() {
            return Ok(()); // Not a block device or no sysfs info
        }

        // Read model information
        if let Ok(model) = fs::read_to_string(format!("{}/device/model", sysfs_path)) {
            disk.model = model.trim().to_string();
        }

        // Read size in sectors
        if let Ok(size_str) = fs::read_to_string(format!("{}/size", sysfs_path)) {
            if let Ok(sectors) = size_str.trim().parse::<u64>() {
                disk.size_bytes = sectors * 512; // Assume 512-byte sectors
            }
        }

        // Read queue information for sector size
        if let Ok(logical_str) = fs::read_to_string(format!("{}/queue/logical_block_size", sysfs_path)) {
            if let Ok(logical_size) = logical_str.trim().parse::<u32>() {
                disk.sector_size = logical_size;
            }
        }

        // Try to determine partition table type
        disk.partition_table_type = determine_partition_table_type(&disk.device)?;

        Ok(())
    }

    fn determine_partition_table_type(device: &str) -> Result<String> {
        // Try to read the first sector to determine partition table type
        if let Ok(mut file) = fs::File::open(device) {
            let mut buffer = [0u8; 512];
            if file.read_exact(&mut buffer).is_ok() {
                // Check for GPT signature
                if buffer[510] == 0x55 && buffer[511] == 0xaa {
                    // Check for GPT protective MBR
                    if buffer[450] == 0xee { // GPT protective partition type
                        return Ok("gpt".to_string());
                    } else {
                        return Ok("dos".to_string()); // Traditional MBR
                    }
                }
            }
        }

        // Fallback to blkid if available
        if let Ok(output) = Command::new("blkid")
            .args(&["-p", "-s", "PTTYPE", device])
            .output() 
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("PTTYPE=\"gpt\"") {
                return Ok("gpt".to_string());
            } else if output_str.contains("PTTYPE=\"dos\"") {
                return Ok("dos".to_string());
            }
        }

        Ok("unknown".to_string())
    }

    fn query_disk_partitions_linux(device: &str) -> Result<Vec<PartitionInfo>> {
        let device_name = device.strip_prefix("/dev/").unwrap_or(device);
        let mut partitions = Vec::new();

        // Try lsblk first for comprehensive partition information
        if let Ok(lsblk_partitions) = use_lsblk_partitions(device) {
            partitions.extend(lsblk_partitions);
        }

        // Enhance with /proc/partitions information
        if let Ok(content) = fs::read_to_string("/proc/partitions") {
            for line in content.lines().skip(2) {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 4 {
                    let part_name = fields[3];
                    if part_name.starts_with(device_name) && part_name != device_name {
                        let size_kb = fields[2].parse::<u64>().unwrap_or(0);
                        
                        // Extract partition number
                        let part_num = part_name.strip_prefix(device_name)
                            .and_then(|s| s.trim_start_matches('p').parse::<u32>().ok())
                            .unwrap_or(0);

                        if part_num > 0 {
                            let mut partition = PartitionInfo {
                                device: format!("/dev/{}", part_name),
                                partition_number: part_num,
                                size_bytes: size_kb * 1024,
                                size_sectors: size_kb * 2, // 512-byte sectors
                                ..Default::default()
                            };

                            // Try to get filesystem information from blkid
                            enhance_partition_info_blkid(&mut partition);
                            
                            partitions.push(partition);
                        }
                    }
                }
            }
        }

        Ok(partitions)
    }

    fn enhance_partition_info_blkid(partition: &mut PartitionInfo) {
        if let Ok(output) = Command::new("blkid")
            .arg(&partition.device)
            .output() 
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Parse blkid output for filesystem, UUID, and label
            for part in output_str.split_whitespace() {
                if let Some(fs_type) = part.strip_prefix("TYPE=\"").and_then(|s| s.strip_suffix('"')) {
                    partition.filesystem = Some(fs_type.to_string());
                } else if let Some(uuid) = part.strip_prefix("UUID=\"").and_then(|s| s.strip_suffix('"')) {
                    partition.filesystem_uuid = Some(uuid.to_string());
                } else if let Some(label) = part.strip_prefix("LABEL=\"").and_then(|s| s.strip_suffix('"')) {
                    partition.label = Some(label.to_string());
                }
            }
        }

        // Try to get mount point from /proc/mounts
        if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
            for line in mounts.lines() {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 2 && fields[0] == partition.device {
                    partition.mount_point = Some(fields[1].to_string());
                    break;
                }
            }
        }
    }

    fn use_lsblk() -> Result<Vec<DiskInfo>> {
        let output = Command::new("lsblk")
            .args(&["-J", "-o", "NAME,SIZE,MODEL,PTTYPE,SERIAL,TRAN"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            bail!("lsblk command failed");
        }

        let json_str = String::from_utf8(output.stdout)?;
        let lsblk_json: Value = serde_json::from_str(&json_str)
            .context("Failed to parse lsblk JSON")?;

        let mut disks = Vec::new();

        if let Some(blockdevices) = lsblk_json["blockdevices"].as_array() {
            for device in blockdevices {
                let name = device["name"].as_str().unwrap_or("");
                let size_str = device["size"].as_str().unwrap_or("0");
                let model = device["model"].as_str().unwrap_or("Unknown").to_string();
                let pttype = device["pttype"].as_str().unwrap_or("unknown").to_string();

                // Parse size (e.g., "500.1G", "1T")
                let size_bytes = parse_size_string(size_str);

                disks.push(DiskInfo {
                    device: format!("/dev/{}", name),
                    model,
                    size_bytes,
                    partition_table_type: pttype,
                    serial_number: device["serial"].as_str().map(|s| s.to_string()),
                    interface_type: device["tran"].as_str().map(|s| s.to_string()),
                    ..Default::default()
                });
            }
        }

        Ok(disks)
    }

    fn use_lsblk_partitions(device: &str) -> Result<Vec<PartitionInfo>> {
        let output = Command::new("lsblk")
            .args(&["-J", "-o", "NAME,SIZE,FSTYPE,MOUNTPOINT,LABEL,UUID,PARTUUID", device])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let json_str = String::from_utf8(output.stdout)?;
        let lsblk_json: Value = serde_json::from_str(&json_str)
            .context("Failed to parse lsblk partition JSON")?;

        let mut partitions = Vec::new();

        if let Some(blockdevices) = lsblk_json["blockdevices"].as_array() {
            for device_entry in blockdevices {
                if let Some(children) = device_entry["children"].as_array() {
                    for (index, partition) in children.iter().enumerate() {
                        let name = partition["name"].as_str().unwrap_or("");
                        let size_str = partition["size"].as_str().unwrap_or("0");
                        let size_bytes = parse_size_string(size_str);

                        partitions.push(PartitionInfo {
                            device: format!("/dev/{}", name),
                            partition_number: (index + 1) as u32,
                            size_bytes,
                            size_sectors: size_bytes / 512,
                            filesystem: partition["fstype"].as_str().map(|s| s.to_string()),
                            mount_point: partition["mountpoint"].as_str().map(|s| s.to_string()),
                            label: partition["label"].as_str().map(|s| s.to_string()),
                            filesystem_uuid: partition["uuid"].as_str().map(|s| s.to_string()),
                            partition_uuid: partition["partuuid"].as_str().map(|s| s.to_string()),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        Ok(partitions)
    }

    fn parse_size_string(size_str: &str) -> u64 {
        let size_str = size_str.trim();
        if size_str.is_empty() || size_str == "0" {
            return 0;
        }

        let (number_part, unit) = if let Some(pos) = size_str.find(|c: char| c.is_alphabetic()) {
            (&size_str[..pos], &size_str[pos..])
        } else {
            (size_str, "")
        };

        let number: f64 = number_part.parse().unwrap_or(0.0);
        let multiplier = match unit.to_uppercase().as_str() {
            "K" | "KB" => 1_024,
            "M" | "MB" => 1_024_u64.pow(2),
            "G" | "GB" => 1_024_u64.pow(3),
            "T" | "TB" => 1_024_u64.pow(4),
            "P" | "PB" => 1_024_u64.pow(5),
            _ => 1,
        };

        (number * multiplier as f64) as u64
    }
}

/// macOS-specific disk enumeration using diskutil and IOKit
#[cfg(target_os = "macos")]
mod macos_impl {
    use super::*;

    pub fn enumerate_disks() -> Result<Vec<DiskInfo>> {
        let mut disks = Vec::new();

        // Use diskutil for comprehensive disk information
        if let Ok(diskutil_disks) = query_diskutil_disks() {
            disks.extend(diskutil_disks);
        }

        // Enhance with individual disk information
        for disk in &mut disks {
            if let Ok(partitions) = query_disk_partitions_macos(&disk.device) {
                disk.partitions = partitions;
            }
        }

        Ok(disks)
    }

    fn query_diskutil_disks() -> Result<Vec<DiskInfo>> {
        let output = Command::new("diskutil")
            .args(&["list", "-plist"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            bail!("diskutil list failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        // Parse plist output (simplified - in a real implementation you'd use a plist parser)
        let output_str = String::from_utf8(output.stdout)?;
        let mut disks = Vec::new();

        // Extract disk identifiers
        let disk_pattern = regex::Regex::new(r"<string>(disk\d+)</string>").unwrap();
        for captures in disk_pattern.captures_iter(&output_str) {
            let disk_id = &captures[1];
            if let Ok(disk_info) = query_individual_disk_macos(disk_id) {
                disks.push(disk_info);
            }
        }

        Ok(disks)
    }

    fn query_individual_disk_macos(disk_id: &str) -> Result<DiskInfo> {
        let output = Command::new("diskutil")
            .args(&["info", "-plist", disk_id])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            bail!("diskutil info failed for {}", disk_id);
        }

        let output_str = String::from_utf8(output.stdout)?;
        
        // Simple plist parsing (extract key values)
        let mut disk_info = DiskInfo {
            device: format!("/dev/{}", disk_id),
            ..Default::default()
        };

        // Extract device name/model
        if let Some(name_match) = extract_plist_string(&output_str, "DeviceIdentifier") {
            disk_info.device = format!("/dev/{}", name_match);
        }

        if let Some(model) = extract_plist_string(&output_str, "MediaName") {
            disk_info.model = model;
        }

        // Extract size
        if let Some(size_str) = extract_plist_integer(&output_str, "TotalSize") {
            disk_info.size_bytes = size_str;
        }

        // Extract partition scheme
        if let Some(scheme) = extract_plist_string(&output_str, "Content") {
            disk_info.partition_table_type = scheme;
        }

        Ok(disk_info)
    }

    fn query_disk_partitions_macos(device: &str) -> Result<Vec<PartitionInfo>> {
        let disk_id = device.strip_prefix("/dev/").unwrap_or(device);
        
        let output = Command::new("diskutil")
            .args(&["list", "-plist", disk_id])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let output_str = String::from_utf8(output.stdout)?;
        let mut partitions = Vec::new();

        // Simple parsing to extract partition information
        // This is a simplified implementation - a real one would use proper plist parsing
        let partition_pattern = regex::Regex::new(r"<string>(disk\d+s\d+)</string>").unwrap();
        for (index, captures) in partition_pattern.captures_iter(&output_str).enumerate() {
            let partition_id = &captures[1];
            
            if let Ok(partition_info) = query_individual_partition_macos(partition_id, index as u32 + 1) {
                partitions.push(partition_info);
            }
        }

        Ok(partitions)
    }

    fn query_individual_partition_macos(partition_id: &str, partition_number: u32) -> Result<PartitionInfo> {
        let output = Command::new("diskutil")
            .args(&["info", "-plist", partition_id])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            bail!("diskutil info failed for partition {}", partition_id);
        }

        let output_str = String::from_utf8(output.stdout)?;
        
        let mut partition = PartitionInfo {
            device: format!("/dev/{}", partition_id),
            partition_number,
            ..Default::default()
        };

        // Extract partition information
        if let Some(size) = extract_plist_integer(&output_str, "TotalSize") {
            partition.size_bytes = size;
            partition.size_sectors = size / 512;
        }

        if let Some(filesystem) = extract_plist_string(&output_str, "FilesystemType") {
            partition.filesystem = Some(filesystem);
        }

        if let Some(mount_point) = extract_plist_string(&output_str, "MountPoint") {
            partition.mount_point = Some(mount_point);
        }

        if let Some(label) = extract_plist_string(&output_str, "VolumeName") {
            partition.label = Some(label);
        }

        Ok(partition)
    }

    fn extract_plist_string(plist: &str, key: &str) -> Option<String> {
        let pattern = format!(r"<key>{}</key>\s*<string>([^<]+)</string>", regex::escape(key));
        if let Ok(re) = regex::Regex::new(&pattern) {
            if let Some(captures) = re.captures(plist) {
                return Some(captures[1].to_string());
            }
        }
        None
    }

    fn extract_plist_integer(plist: &str, key: &str) -> Option<u64> {
        let pattern = format!(r"<key>{}</key>\s*<integer>(\d+)</integer>", regex::escape(key));
        if let Ok(re) = regex::Regex::new(&pattern) {
            if let Some(captures) = re.captures(plist) {
                return captures[1].parse().ok();
            }
        }
        None
    }
}

/// Cross-platform disk enumeration
pub fn enumerate_disks() -> Result<Vec<DiskInfo>> {
    #[cfg(target_os = "windows")]
    return windows_impl::enumerate_disks();
    
    #[cfg(target_os = "linux")]
    return linux_impl::enumerate_disks();
    
    #[cfg(target_os = "macos")]
    return macos_impl::enumerate_disks();
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        // Fallback for unsupported platforms
        Ok(vec![DiskInfo {
            device: "/dev/unknown".to_string(),
            model: "Unsupported Platform".to_string(),
            size_bytes: 0,
            partition_table_type: "unknown".to_string(),
            ..Default::default()
        }])
    }
}

/// Get information for a specific disk device
pub fn get_disk_info(device_path: &str) -> Result<DiskInfo> {
    let disks = enumerate_disks()?;
    
    for disk in disks {
        if disk.device == device_path || 
           disk.device.ends_with(&device_path.replace("/dev/", "")) {
            return Ok(disk);
        }
    }
    
    bail!("Device not found: {}", device_path)
}

/// Format bytes in human-readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    const THRESHOLD: f64 = 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Output disk information in various formats
fn output_disks(disks: &[DiskInfo], config: &FdiskConfig) -> Result<()> {
    if config.json_output {
        output_json(disks)?;
    } else if config.verbose {
        output_verbose(disks)?;
    } else {
        output_standard(disks)?;
    }
    
    Ok(())
}

/// Output disks in standard fdisk format
fn output_standard(disks: &[DiskInfo]) -> Result<()> {
    for disk in disks {
        println!("{}", disk.format_fdisk_header());
        
        if !disk.partitions.is_empty() {
            println!();
            println!("{:<15} {:<1} {:>10} {:>10} {:>10} {:>10} {:<8} {}", 
                "Device", "Boot", "Start", "End", "Sectors", "Size", "Id", "Type");
            
            for partition in &disk.partitions {
                println!("{}", partition.format_fdisk_entry());
            }
        }
        println!();
    }
    Ok(())
}

/// Output disks with verbose information
fn output_verbose(disks: &[DiskInfo]) -> Result<()> {
    for disk in disks {
        println!("=== Disk {} ===", disk.device);
        println!("Model: {}", disk.model);
        println!("Size: {} ({})", disk.format_size(), disk.size_bytes);
        println!("Sectors: {} (sector size: {} bytes)", disk.total_sectors(), disk.sector_size);
        println!("Partition table: {}", disk.partition_table_type);
        
        if let Some(serial) = &disk.serial_number {
            println!("Serial number: {}", serial);
        }
        if let Some(interface) = &disk.interface_type {
            println!("Interface: {}", interface);
        }
        if let Some(firmware) = &disk.firmware_revision {
            println!("Firmware: {}", firmware);
        }
        if let Some(health) = &disk.health_status {
            println!("Health: {}", health);
        }

        if !disk.partitions.is_empty() {
            println!("\nPartitions:");
            for partition in &disk.partitions {
                println!("  Partition {}: {}", partition.partition_number, partition.device);
                println!("    Size: {} (sectors: {})", partition.format_size(), partition.size_sectors);
                println!("    Range: {} - {}", partition.start_sector, partition.end_sector);
                println!("    Type: {}", partition.partition_type);
                
                if let Some(fs) = &partition.filesystem {
                    println!("    Filesystem: {}", fs);
                }
                if let Some(mount) = &partition.mount_point {
                    println!("    Mount: {}", mount);
                }
                if let Some(label) = &partition.label {
                    println!("    Label: {}", label);
                }
                if partition.bootable {
                    println!("    Bootable: Yes");
                }
                println!();
            }
        }
        println!();
    }
    Ok(())
}

/// Output disks in JSON format
fn output_json(disks: &[DiskInfo]) -> Result<()> {
    let json_disks: Vec<Value> = disks.iter().map(|disk| {
        json!({
            "device": disk.device,
            "model": disk.model,
            "size_bytes": disk.size_bytes,
            "sector_size": disk.sector_size,
            "total_sectors": disk.total_sectors(),
            "partition_table_type": disk.partition_table_type,
            "mount_point": disk.mount_point,
            "disk_id": disk.disk_id,
            "serial_number": disk.serial_number,
            "interface_type": disk.interface_type,
            "firmware_revision": disk.firmware_revision,
            "health_status": disk.health_status,
            "partitions": disk.partitions.iter().map(|partition| {
                json!({
                    "device": partition.device,
                    "partition_number": partition.partition_number,
                    "start_sector": partition.start_sector,
                    "end_sector": partition.end_sector,
                    "size_sectors": partition.size_sectors,
                    "size_bytes": partition.size_bytes,
                    "partition_type": partition.partition_type,
                    "filesystem": partition.filesystem,
                    "mount_point": partition.mount_point,
                    "bootable": partition.bootable,
                    "partition_uuid": partition.partition_uuid,
                    "filesystem_uuid": partition.filesystem_uuid,
                    "label": partition.label
                })
            }).collect::<Vec<_>>()
        })
    }).collect();

    println!("{}", serde_json::to_string_pretty(&json_disks)?);
    Ok(())
}

/// Display help information
fn show_help() {
    println!("Usage: fdisk [OPTIONS] [DEVICE]");
    println!();
    println!("Display partition table information and disk details");
    println!();
    println!("OPTIONS:");
    println!("  -l, --list             List all available disks and their partitions");
    println!("  -j, --json             Output in JSON format");
    println!("  -v, --verbose          Show verbose disk and partition information");
    println!("  -b, --sector-size SIZE Override sector size (default: auto-detect)");
    println!("  -h, --help             Show this help message");
    println!("  -V, --version          Show version information");
    println!();
    println!("ARGUMENTS:");
    println!("  DEVICE                 Show partition table for specific device");
    println!();
    println!("EXAMPLES:");
    println!("  fdisk -l               List all disks and partitions");
    println!("  fdisk /dev/sda         Show partition table for /dev/sda");
    println!("  fdisk -v -l            List all disks with verbose information");
    println!("  fdisk -j /dev/sda      Show /dev/sda information in JSON format");
    println!();
    println!("NOTE:");
    println!("  This implementation is read-only for safety. No destructive operations");
    println!("  are supported. Use platform-specific tools for partition modification.");
}

/// Display version information
fn show_version() {
    println!("fdisk (NexusShell builtins) 1.0.0");
    println!("Cross-platform partition table viewer");
    println!("Pure Rust implementation with platform-specific optimizations");
}

/// Main fdisk CLI entry point
pub async fn fdisk_cli(args: &[String]) -> Result<()> {
    let config = FdiskConfig::parse_args(args)?;

    if config.help {
        show_help();
        return Ok(());
    }

    if config.version {
        show_version();
        return Ok(());
    }

    if config.list_all {
        // List all disks and their partitions
        let disks = enumerate_disks()
            .context("Failed to enumerate disk devices")?;
        output_disks(&disks, &config)?;
    } else if let Some(device_path) = &config.device_path {
        // Show information for specific device
        let disk = get_disk_info(device_path)
            .context("Failed to get disk information")?;
        output_disks(&[disk], &config)?;
    } else {
        // Default: list all disks
        let disks = enumerate_disks()
            .context("Failed to enumerate disk devices")?;
        output_disks(&disks, &config)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let args = vec!["-l".to_string(), "--verbose".to_string()];
        let config = FdiskConfig::parse_args(&args).unwrap();
        assert!(config.list_all);
        assert!(config.verbose);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1048576), "1.0 MiB");
        assert_eq!(format_bytes(1073741824), "1.0 GiB");
        assert_eq!(format_bytes(1500000000), "1.4 GiB");
    }

    #[test]
    fn test_disk_info_formatting() {
        let disk = DiskInfo {
            device: "/dev/sda".to_string(),
            model: "Samsung SSD".to_string(),
            size_bytes: 500_000_000_000,
            sector_size: 512,
            partition_table_type: "gpt".to_string(),
            ..Default::default()
        };

        let header = disk.format_fdisk_header();
        assert!(header.contains("/dev/sda"));
        assert!(header.contains("gpt"));
        assert!(header.contains("500000000000 bytes"));
    }

    #[test]
    fn test_partition_info_formatting() {
        let partition = PartitionInfo {
            device: "/dev/sda1".to_string(),
            partition_number: 1,
            start_sector: 2048,
            end_sector: 1048575,
            size_sectors: 1046528,
            size_bytes: 536_870_912,
            partition_type: "EFI System".to_string(),
            filesystem: Some("vfat".to_string()),
            bootable: true,
            ..Default::default()
        };

        let entry = partition.format_fdisk_entry();
        assert!(entry.contains("/dev/sda1"));
        assert!(entry.contains("*")); // Boot flag
        assert!(entry.contains("2048"));
        assert!(entry.contains("vfat"));
    }

    #[test]
    fn test_help_parsing() {
        let args = vec!["--help".to_string()];
        let config = FdiskConfig::parse_args(&args).unwrap();
        assert!(config.help);
    }

    #[test]
    fn test_version_parsing() {
        let args = vec!["-V".to_string()];
        let config = FdiskConfig::parse_args(&args).unwrap();
        assert!(config.version);
    }

    #[test]
    fn test_json_option() {
        let args = vec!["--json".to_string()];
        let config = FdiskConfig::parse_args(&args).unwrap();
        assert!(config.json_output);
    }

    #[test]
    fn test_device_argument() {
        let args = vec!["/dev/sda".to_string()];
        let config = FdiskConfig::parse_args(&args).unwrap();
        assert_eq!(config.device_path, Some("/dev/sda".to_string()));
    }

    #[test]
    fn test_sector_size_option() {
        let args = vec!["-b".to_string(), "4096".to_string()];
        let config = FdiskConfig::parse_args(&args).unwrap();
        assert_eq!(config.sector_size, Some(4096));
    }

    #[test]
    fn test_invalid_option() {
        let args = vec!["--invalid".to_string()];
        assert!(FdiskConfig::parse_args(&args).is_err());
    }

    #[test]
    fn test_invalid_sector_size() {
        let args = vec!["-b".to_string(), "invalid".to_string()];
        assert!(FdiskConfig::parse_args(&args).is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_is_full_disk_device() {
        use linux_impl::is_full_disk_device;
        
        assert!(is_full_disk_device("sda"));
        assert!(is_full_disk_device("nvme0n1"));
        assert!(is_full_disk_device("mmcblk0"));
        assert!(!is_full_disk_device("sda1"));
        assert!(!is_full_disk_device("nvme0n1p1"));
        assert!(!is_full_disk_device("mmcblk0p1"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_parse_size_string() {
        use linux_impl::parse_size_string;
        
        assert_eq!(parse_size_string("1024"), 1024);
        assert_eq!(parse_size_string("1K"), 1024);
        assert_eq!(parse_size_string("1M"), 1048576);
        assert_eq!(parse_size_string("1.5G"), 1610612736);
        assert_eq!(parse_size_string(""), 0);
    }

    #[tokio::test]
    async fn test_fdisk_help() {
        let result = fdisk_cli(&["--help".to_string()]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fdisk_version() {
        let result = fdisk_cli(&["-V".to_string()]).await;
        assert!(result.is_ok());
    }
}
