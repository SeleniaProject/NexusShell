//! `free` command - display amount of free and used memory in the system
//!
//! Full free implementation with various formatting options and memory statistics

use crate::common::{i18n::*, logging::*};
use std::io::Write;
use std::collections::HashMap;
use nxsh_core::{Builtin, Context, ExecutionResult, ShellResult};
use std::fs;
use std::thread;
use std::time::Duration;

pub struct FreeBuiltin;

#[derive(Debug, Clone)]
pub struct FreeOptions {
    pub human_readable: bool,
    pub si_units: bool,
    pub bytes: bool,
    pub kilo: bool,
    pub mega: bool,
    pub giga: bool,
    pub tera: bool,
    pub show_total: bool,
    pub wide_output: bool,
    pub continuous: bool,
    pub interval: Option<u64>,
    pub count: Option<u32>,
    pub show_available: bool,
    pub show_buffers_cache: bool,
    pub show_committed: bool,
}

#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total: u64,
    pub free: u64,
    pub available: u64,
    pub buffers: u64,
    pub cached: u64,
    pub slab: u64,
    pub swap_cached: u64,
    pub active: u64,
    pub inactive: u64,
    pub dirty: u64,
    pub writeback: u64,
    pub anon_pages: u64,
    pub mapped: u64,
    pub shmem: u64,
    pub kreclaimable: u64,
    pub sunreclaim: u64,
    pub kernel_stack: u64,
    pub page_tables: u64,
    pub nfs_unstable: u64,
    pub bounce: u64,
    pub writeback_tmp: u64,
    pub commit_limit: u64,
    pub committed_as: u64,
    pub vmalloc_total: u64,
    pub vmalloc_used: u64,
    pub vmalloc_chunk: u64,
    pub hardware_corrupted: u64,
    pub anon_huge_pages: u64,
    pub shmem_huge_pages: u64,
    pub shmem_pmd_mapped: u64,
    pub cma_total: u64,
    pub cma_free: u64,
    pub huge_pages_total: u64,
    pub huge_pages_free: u64,
    pub huge_pages_rsvd: u64,
    pub huge_pages_surp: u64,
    pub hugepagesize: u64,
    pub direct_map_4k: u64,
    pub direct_map_2m: u64,
    pub direct_map_1g: u64,
}

#[derive(Debug, Clone)]
pub struct SwapInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub cached: u64,
}

impl Builtin for FreeBuiltin {
    fn name(&self) -> &str {
        "free"
    }

    fn execute(&self, context: &mut Context, args: Vec<String>) -> ShellResult<i32> {
        let options = parse_free_args(&args)?;
        
        if options.continuous {
            run_continuous_mode(&options)?;
        } else {
            display_memory_info(&options)?;
        }
        
        Ok(0)
    }

    fn help(&self) -> &str {
        "free - display amount of free and used memory in the system

USAGE:
    free [OPTIONS]

OPTIONS:
    -b, --bytes           Show output in bytes
    -k, --kibi            Show output in kibibytes (default)
    -m, --mebi            Show output in mebibytes
    -g, --gibi            Show output in gibibytes
    -t, --tera            Show output in tebibytes
    --kilo                Show output in kilobytes
    --mega                Show output in megabytes
    --giga                Show output in gigabytes
    --tera                Show output in terabytes
    -h, --human           Show human-readable output
    --si                  Use powers of 1000 (not 1024)
    -w, --wide            Wide mode (show cache and available columns)
    -c, --count=COUNT     Display the result COUNT times
    -s, --seconds=DELAY   Repeat printing every DELAY seconds
    -t, --total           Display a line showing column totals
    --help                Display this help and exit

EXAMPLES:
    free                  Show memory usage in KiB
    free -h               Show memory usage in human-readable format
    free -m               Show memory usage in MiB
    free -s 5             Update every 5 seconds
    free -c 3 -s 2        Show 3 times with 2 second intervals
    free -w               Show wide output with cache details"
    }
}

fn parse_free_args(args: &[String]) -> ShellResult<FreeOptions> {
    let mut options = FreeOptions {
        human_readable: false,
        si_units: false,
        bytes: false,
        kilo: true, // Default
        mega: false,
        giga: false,
        tera: false,
        show_total: false,
        wide_output: false,
        continuous: false,
        interval: None,
        count: None,
        show_available: true,
        show_buffers_cache: true,
        show_committed: false,
    };

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "-b" | "--bytes" => {
                options.bytes = true;
                options.kilo = false;
            }
            "-k" | "--kibi" => {
                options.kilo = true;
                options.bytes = false;
                options.mega = false;
                options.giga = false;
                options.tera = false;
            }
            "-m" | "--mebi" => {
                options.mega = true;
                options.bytes = false;
                options.kilo = false;
                options.giga = false;
                options.tera = false;
            }
            "-g" | "--gibi" => {
                options.giga = true;
                options.bytes = false;
                options.kilo = false;
                options.mega = false;
                options.tera = false;
            }
            "--tera" => {
                options.tera = true;
                options.bytes = false;
                options.kilo = false;
                options.mega = false;
                options.giga = false;
            }
            "--kilo" => {
                options.kilo = true;
                options.si_units = true;
                options.bytes = false;
            }
            "--mega" => {
                options.mega = true;
                options.si_units = true;
                options.kilo = false;
                options.bytes = false;
            }
            "--giga" => {
                options.giga = true;
                options.si_units = true;
                options.kilo = false;
                options.bytes = false;
            }
            "-h" | "--human" => options.human_readable = true,
            "--si" => options.si_units = true,
            "-w" | "--wide" => options.wide_output = true,
            "-t" | "--total" => options.show_total = true,
            "-s" | "--seconds" => {
                i += 1;
                if i >= args.len() {
                    return Err(ShellError::runtime("Option -s requires an argument"));
                }
                let interval = args[i].parse::<u64>()
                    .map_err(|_| ShellError::runtime("Invalid interval value"))?;
                options.interval = Some(interval);
                options.continuous = true;
            }
            "-c" | "--count" => {
                i += 1;
                if i >= args.len() {
                    return Err(ShellError::runtime("Option -c requires an argument"));
                }
                options.count = Some(args[i].parse::<u32>()
                    .map_err(|_| ShellError::runtime("Invalid count value"))?);
            }
            "--help" => return Err(ShellError::runtime("Help requested")),
            _ if arg.starts_with("-") => {
                return Err(ShellError::runtime(format!("Unknown option: {}", arg)));
            }
            _ => return Err(ShellError::runtime(format!("Unknown argument: {}", arg))),
        }
        i += 1;
    }

    Ok(options)
}

fn run_continuous_mode(options: &FreeOptions) -> ShellResult<()> {
    let interval = Duration::from_secs(options.interval.unwrap_or(1));
    let mut iteration = 0;
    
    loop {
        if let Some(max_count) = options.count {
            if iteration >= max_count {
                break;
            }
        }
        
        display_memory_info(options)?;
        iteration += 1;
        
        if options.count.is_some() && iteration >= options.count.unwrap() {
            break;
        }
        
        thread::sleep(interval);
        println!(); // Add blank line between iterations
    }
    
    Ok(())
}

fn display_memory_info(options: &FreeOptions) -> ShellResult<()> {
    let memory_info = collect_memory_info()?;
    let swap_info = collect_swap_info()?;
    
    // Print header
    print_header(options);
    
    // Print memory line
    print_memory_line(&memory_info, options);
    
    // Print swap line
    print_swap_line(&swap_info, options);
    
    // Print total line if requested
    if options.show_total {
        print_total_line(&memory_info, &swap_info, options);
    }
    
    Ok(())
}

fn collect_memory_info() -> ShellResult<MemoryInfo> {
    #[cfg(target_os = "linux")]
    {
        collect_linux_memory_info()
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        // Simplified memory info for other platforms
        Ok(MemoryInfo {
            total: 0,
            free: 0,
            available: 0,
            buffers: 0,
            cached: 0,
            slab: 0,
            swap_cached: 0,
            active: 0,
            inactive: 0,
            dirty: 0,
            writeback: 0,
            anon_pages: 0,
            mapped: 0,
            shmem: 0,
            kreclaimable: 0,
            sunreclaim: 0,
            kernel_stack: 0,
            page_tables: 0,
            nfs_unstable: 0,
            bounce: 0,
            writeback_tmp: 0,
            commit_limit: 0,
            committed_as: 0,
            vmalloc_total: 0,
            vmalloc_used: 0,
            vmalloc_chunk: 0,
            hardware_corrupted: 0,
            anon_huge_pages: 0,
            shmem_huge_pages: 0,
            shmem_pmd_mapped: 0,
            cma_total: 0,
            cma_free: 0,
            huge_pages_total: 0,
            huge_pages_free: 0,
            huge_pages_rsvd: 0,
            huge_pages_surp: 0,
            hugepagesize: 0,
            direct_map_4k: 0,
            direct_map_2m: 0,
            direct_map_1g: 0,
        })
    }
}

#[cfg(target_os = "linux")]
fn collect_linux_memory_info() -> ShellResult<MemoryInfo> {
    let content = fs::read_to_string("/proc/meminfo")
        .map_err(|e| ShellError::io(format!("Cannot read /proc/meminfo: {}", e)))?;
    
    let mut memory_info = MemoryInfo {
        total: 0, free: 0, available: 0, buffers: 0, cached: 0, slab: 0,
        swap_cached: 0, active: 0, inactive: 0, dirty: 0, writeback: 0,
        anon_pages: 0, mapped: 0, shmem: 0, kreclaimable: 0, sunreclaim: 0,
        kernel_stack: 0, page_tables: 0, nfs_unstable: 0, bounce: 0,
        writeback_tmp: 0, commit_limit: 0, committed_as: 0, vmalloc_total: 0,
        vmalloc_used: 0, vmalloc_chunk: 0, hardware_corrupted: 0,
        anon_huge_pages: 0, shmem_huge_pages: 0, shmem_pmd_mapped: 0,
        cma_total: 0, cma_free: 0, huge_pages_total: 0, huge_pages_free: 0,
        huge_pages_rsvd: 0, huge_pages_surp: 0, hugepagesize: 0,
        direct_map_4k: 0, direct_map_2m: 0, direct_map_1g: 0,
    };
    
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let key = parts[0].trim_end_matches(':');
            let value = parts[1].parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes
            
            match key {
                "MemTotal" => memory_info.total = value,
                "MemFree" => memory_info.free = value,
                "MemAvailable" => memory_info.available = value,
                "Buffers" => memory_info.buffers = value,
                "Cached" => memory_info.cached = value,
                "Slab" => memory_info.slab = value,
                "SwapCached" => memory_info.swap_cached = value,
                "Active" => memory_info.active = value,
                "Inactive" => memory_info.inactive = value,
                "Dirty" => memory_info.dirty = value,
                "Writeback" => memory_info.writeback = value,
                "AnonPages" => memory_info.anon_pages = value,
                "Mapped" => memory_info.mapped = value,
                "Shmem" => memory_info.shmem = value,
                "KReclaimable" => memory_info.kreclaimable = value,
                "SUnreclaim" => memory_info.sunreclaim = value,
                "KernelStack" => memory_info.kernel_stack = value,
                "PageTables" => memory_info.page_tables = value,
                "NFS_Unstable" => memory_info.nfs_unstable = value,
                "Bounce" => memory_info.bounce = value,
                "WritebackTmp" => memory_info.writeback_tmp = value,
                "CommitLimit" => memory_info.commit_limit = value,
                "Committed_AS" => memory_info.committed_as = value,
                "VmallocTotal" => memory_info.vmalloc_total = value,
                "VmallocUsed" => memory_info.vmalloc_used = value,
                "VmallocChunk" => memory_info.vmalloc_chunk = value,
                "HardwareCorrupted" => memory_info.hardware_corrupted = value,
                "AnonHugePages" => memory_info.anon_huge_pages = value,
                "ShmemHugePages" => memory_info.shmem_huge_pages = value,
                "ShmemPmdMapped" => memory_info.shmem_pmd_mapped = value,
                "CmaTotal" => memory_info.cma_total = value,
                "CmaFree" => memory_info.cma_free = value,
                "HugePages_Total" => memory_info.huge_pages_total = value / 1024, // Already in pages
                "HugePages_Free" => memory_info.huge_pages_free = value / 1024,
                "HugePages_Rsvd" => memory_info.huge_pages_rsvd = value / 1024,
                "HugePages_Surp" => memory_info.huge_pages_surp = value / 1024,
                "Hugepagesize" => memory_info.hugepagesize = value,
                "DirectMap4k" => memory_info.direct_map_4k = value,
                "DirectMap2M" => memory_info.direct_map_2m = value,
                "DirectMap1G" => memory_info.direct_map_1g = value,
                _ => {}
            }
        }
    }
    
    // Calculate available if not provided
    if memory_info.available == 0 {
        memory_info.available = memory_info.free + memory_info.buffers + memory_info.cached;
    }
    
    Ok(memory_info)
}

fn collect_swap_info() -> ShellResult<SwapInfo> {
    #[cfg(target_os = "linux")]
    {
        let content = fs::read_to_string("/proc/meminfo")
            .map_err(|e| ShellError::io(format!("Cannot read /proc/meminfo: {}", e)))?;
        
        let mut swap_total = 0;
        let mut swap_free = 0;
        let mut swap_cached = 0;
        
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let key = parts[0].trim_end_matches(':');
                let value = parts[1].parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes
                
                match key {
                    "SwapTotal" => swap_total = value,
                    "SwapFree" => swap_free = value,
                    "SwapCached" => swap_cached = value,
                    _ => {}
                }
            }
        }
        
        let swap_used = swap_total.saturating_sub(swap_free);
        
        Ok(SwapInfo {
            total: swap_total,
            used: swap_used,
            free: swap_free,
            cached: swap_cached,
        })
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        Ok(SwapInfo {
            total: 0,
            used: 0,
            free: 0,
            cached: 0,
        })
    }
}

fn print_header(options: &FreeOptions) {
    if options.wide_output {
        println!("{:>14} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
            "", "total", "used", "free", "shared", "buff/cache", "available");
    } else {
        println!("{:>14} {:>10} {:>10} {:>10} {:>10} {:>10}",
            "", "total", "used", "free", "shared", "buff/cache");
    }
}

fn print_memory_line(memory_info: &MemoryInfo, options: &FreeOptions) {
    let used = memory_info.total.saturating_sub(memory_info.free + memory_info.buffers + memory_info.cached);
    let shared = memory_info.shmem;
    let buff_cache = memory_info.buffers + memory_info.cached;
    
    if options.wide_output {
        println!("{:>14} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
            "Mem:",
            format_memory(memory_info.total, options),
            format_memory(used, options),
            format_memory(memory_info.free, options),
            format_memory(shared, options),
            format_memory(buff_cache, options),
            format_memory(memory_info.available, options)
        );
    } else {
        println!("{:>14} {:>10} {:>10} {:>10} {:>10} {:>10}",
            "Mem:",
            format_memory(memory_info.total, options),
            format_memory(used, options),
            format_memory(memory_info.free, options),
            format_memory(shared, options),
            format_memory(buff_cache, options)
        );
    }
}

fn print_swap_line(swap_info: &SwapInfo, options: &FreeOptions) {
    if options.wide_output {
        println!("{:>14} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
            "Swap:",
            format_memory(swap_info.total, options),
            format_memory(swap_info.used, options),
            format_memory(swap_info.free, options),
            "",
            "",
            ""
        );
    } else {
        println!("{:>14} {:>10} {:>10} {:>10} {:>10} {:>10}",
            "Swap:",
            format_memory(swap_info.total, options),
            format_memory(swap_info.used, options),
            format_memory(swap_info.free, options),
            "",
            ""
        );
    }
}

fn print_total_line(memory_info: &MemoryInfo, swap_info: &SwapInfo, options: &FreeOptions) {
    let total_total = memory_info.total + swap_info.total;
    let total_used = (memory_info.total - memory_info.available) + swap_info.used;
    let total_free = memory_info.available + swap_info.free;
    
    if options.wide_output {
        println!("{:>14} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
            "Total:",
            format_memory(total_total, options),
            format_memory(total_used, options),
            format_memory(total_free, options),
            "",
            "",
            ""
        );
    } else {
        println!("{:>14} {:>10} {:>10} {:>10} {:>10} {:>10}",
            "Total:",
            format_memory(total_total, options),
            format_memory(total_used, options),
            format_memory(total_free, options),
            "",
            ""
        );
    }
}

fn format_memory(bytes: u64, options: &FreeOptions) -> String {
    if options.human_readable {
        format_human_readable(bytes, options.si_units)
    } else if options.bytes {
        bytes.to_string()
    } else if options.mega {
        let divisor = if options.si_units { 1_000_000 } else { 1_048_576 };
        (bytes / divisor).to_string()
    } else if options.giga {
        let divisor = if options.si_units { 1_000_000_000 } else { 1_073_741_824 };
        (bytes / divisor).to_string()
    } else if options.tera {
        let divisor = if options.si_units { 1_000_000_000_000 } else { 1_099_511_627_776 };
        (bytes / divisor).to_string()
    } else {
        // Default: kilo
        let divisor = if options.si_units { 1_000 } else { 1_024 };
        (bytes / divisor).to_string()
    }
}

fn format_human_readable(bytes: u64, si_units: bool) -> String {
    let units = if si_units {
        ["B", "K", "M", "G", "T", "P"]
    } else {
        ["B", "Ki", "Mi", "Gi", "Ti", "Pi"]
    };
    
    let base = if si_units { 1000.0 } else { 1024.0 };
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= base && unit_index < units.len() - 1 {
        size /= base;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{}B", bytes)
    } else if size >= 10.0 {
        format!("{:.0}{}", size, units[unit_index])
    } else {
        format!("{:.1}{}", size, units[unit_index])
    }
} 