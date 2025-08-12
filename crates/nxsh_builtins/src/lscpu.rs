use std::collections::HashMap;
use std::process::Command;
use anyhow::Result;

pub fn lscpu_cli(args: Vec<String>) -> Result<()> {
    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let mut json_output = false;
    let mut extended = false;
    let mut parse_only = None;

    for i in 0..args.len() {
        match args[i].as_str() {
            "-J" | "--json" => json_output = true,
            "-e" | "--extended" => extended = true,
            "-p" | "--parse" => {
                if i + 1 < args.len() {
                    parse_only = Some(args[i + 1].clone());
                }
            }
            _ => {}
        }
    }

    let cpu_info = get_cpu_information()?;

    if json_output {
        print_json_output(&cpu_info);
    } else {
        print_standard_output(&cpu_info, extended, parse_only.as_deref());
    }

    Ok(())
}

fn print_help() {
    println!("Usage: lscpu [options]");
    println!();
    println!("Display information about the CPU architecture.");
    println!();
    println!("Options:");
    println!("  -e, --extended       Show extended information");
    println!("  -J, --json           Show output in JSON format");
    println!("  -p, --parse COLUMN   Parse specific column only");
    println!("  -h, --help          Show this help message");
    println!();
    println!("Available columns for --parse:");
    println!("  CPU, Core, Socket, Node, Book, Drawer, Cache, Address, Online");
}

#[derive(Debug, Clone)]
struct CpuInfo {
    architecture: String,
    cpu_op_modes: Vec<String>,
    byte_order: String,
    cpus: u32,
    on_line_cpus: String,
    threads_per_core: u32,
    cores_per_socket: u32,
    sockets: u32,
    numa_nodes: u32,
    vendor_id: String,
    model_name: String,
    cpu_family: u32,
    model: u32,
    stepping: u32,
    cpu_mhz: f64,
    cpu_max_mhz: f64,
    cpu_min_mhz: f64,
    bogomips: f64,
    l1d_cache: String,
    l1i_cache: String,
    l2_cache: String,
    l3_cache: String,
    numa_info: Vec<String>,
    flags: Vec<String>,
    vulnerabilities: HashMap<String, String>,
}

// Windows / 非Windows で個別に定義して重複コード (identical blocks) を排除
#[cfg(windows)]
fn get_cpu_information() -> Result<CpuInfo> { get_windows_cpu_info() }
#[cfg(not(windows))]
fn get_cpu_information() -> Result<CpuInfo> { get_linux_cpu_info() }

#[cfg(windows)]
fn get_windows_cpu_info() -> Result<CpuInfo> {
    let mut cpu_info = CpuInfo::default();

    // Use Windows Management Instrumentation (WMI) via PowerShell
    let wmi_query = "Get-WmiObject -Class Win32_Processor | Select-Object Name, Manufacturer, Architecture, NumberOfCores, NumberOfLogicalProcessors, MaxClockSpeed, CurrentClockSpeed";
    
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(wmi_query)
        .output();

    match output {
        Ok(result) => {
            let text = String::from_utf8_lossy(&result.stdout);
            parse_windows_wmi_output(&mut cpu_info, &text);
        }
        Err(_) => {
            // Fallback to environment variables and system info
            cpu_info.architecture = std::env::consts::ARCH.to_string();
            cpu_info.cpus = std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(1);
        }
    }

    // Additional Windows-specific information
    cpu_info.byte_order = "Little Endian".to_string();
    cpu_info.on_line_cpus = format!("0-{}", cpu_info.cpus - 1);
    
    Ok(cpu_info)
}

#[cfg(not(windows))]
fn get_linux_cpu_info() -> Result<CpuInfo> {
    let mut cpu_info = CpuInfo::default();
    
    // Parse /proc/cpuinfo
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        parse_proc_cpuinfo(&mut cpu_info, &cpuinfo);
    }

    // Parse lscpu if available
    if let Ok(output) = Command::new("lscpu").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        parse_lscpu_output(&mut cpu_info, &text);
    }

    // Get cache information from /sys/devices/system/cpu
    get_cache_info(&mut cpu_info);

    // Get vulnerability information
    get_vulnerability_info(&mut cpu_info);

    // Get NUMA information
    get_numa_info(&mut cpu_info);

    Ok(cpu_info)
}

impl Default for CpuInfo {
    fn default() -> Self {
        CpuInfo {
            architecture: "unknown".to_string(),
            cpu_op_modes: vec!["32-bit".to_string(), "64-bit".to_string()],
            byte_order: "Little Endian".to_string(),
            cpus: std::thread::available_parallelism().map(|n| n.get() as u32).unwrap_or(1),
            on_line_cpus: "0".to_string(),
            threads_per_core: 1,
            cores_per_socket: 1,
            sockets: 1,
            numa_nodes: 1,
            vendor_id: "Unknown".to_string(),
            model_name: "Unknown CPU".to_string(),
            cpu_family: 0,
            model: 0,
            stepping: 0,
            cpu_mhz: 0.0,
            cpu_max_mhz: 0.0,
            cpu_min_mhz: 0.0,
            bogomips: 0.0,
            l1d_cache: "Unknown".to_string(),
            l1i_cache: "Unknown".to_string(),
            l2_cache: "Unknown".to_string(),
            l3_cache: "Unknown".to_string(),
            numa_info: Vec::new(),
            flags: Vec::new(),
            vulnerabilities: HashMap::new(),
        }
    }
}

#[cfg(windows)]
fn parse_windows_wmi_output(cpu_info: &mut CpuInfo, output: &str) {
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("Name") {
            if let Some(name) = line.split(':').nth(1) {
                cpu_info.model_name = name.trim().to_string();
            }
        } else if line.starts_with("Manufacturer") {
            if let Some(vendor) = line.split(':').nth(1) {
                cpu_info.vendor_id = vendor.trim().to_string();
            }
        } else if line.starts_with("NumberOfCores") {
            if let Some(cores) = line.split(':').nth(1) {
                cpu_info.cores_per_socket = cores.trim().parse().unwrap_or(1);
            }
        } else if line.starts_with("NumberOfLogicalProcessors") {
            if let Some(threads) = line.split(':').nth(1) {
                cpu_info.cpus = threads.trim().parse().unwrap_or(1);
            }
        } else if line.starts_with("MaxClockSpeed") {
            if let Some(mhz) = line.split(':').nth(1) {
                cpu_info.cpu_max_mhz = mhz.trim().parse::<f64>().unwrap_or(0.0);
                cpu_info.cpu_mhz = cpu_info.cpu_max_mhz;
            }
        }
    }
    
    cpu_info.threads_per_core = cpu_info.cpus / cpu_info.cores_per_socket;
    cpu_info.on_line_cpus = format!("0-{}", cpu_info.cpus - 1);
}

#[cfg(not(windows))]
fn parse_proc_cpuinfo(cpu_info: &mut CpuInfo, content: &str) {
    let mut processor_count = 0;
    
    for line in content.lines() {
        let line = line.trim();
        
        if line.starts_with("processor") {
            processor_count += 1;
        } else if line.starts_with("vendor_id") {
            if let Some(vendor) = line.split(':').nth(1) {
                cpu_info.vendor_id = vendor.trim().to_string();
            }
        } else if line.starts_with("model name") {
            if let Some(model) = line.split(':').nth(1) {
                cpu_info.model_name = model.trim().to_string();
            }
        } else if line.starts_with("cpu family") {
            if let Some(family) = line.split(':').nth(1) {
                cpu_info.cpu_family = family.trim().parse().unwrap_or(0);
            }
        } else if line.starts_with("model") && !line.starts_with("model name") {
            if let Some(model) = line.split(':').nth(1) {
                cpu_info.model = model.trim().parse().unwrap_or(0);
            }
        } else if line.starts_with("stepping") {
            if let Some(stepping) = line.split(':').nth(1) {
                cpu_info.stepping = stepping.trim().parse().unwrap_or(0);
            }
        } else if line.starts_with("cpu MHz") {
            if let Some(mhz) = line.split(':').nth(1) {
                cpu_info.cpu_mhz = mhz.trim().parse().unwrap_or(0.0);
            }
        } else if line.starts_with("bogomips") {
            if let Some(bogomips) = line.split(':').nth(1) {
                cpu_info.bogomips = bogomips.trim().parse().unwrap_or(0.0);
            }
        } else if line.starts_with("flags") {
            if let Some(flags) = line.split(':').nth(1) {
                cpu_info.flags = flags.trim().split_whitespace().map(|s| s.to_string()).collect();
            }
        }
    }
    
    cpu_info.cpus = processor_count;
    cpu_info.on_line_cpus = if processor_count > 1 {
        format!("0-{}", processor_count - 1)
    } else {
        "0".to_string()
    };
}

#[cfg(not(windows))]
fn parse_lscpu_output(cpu_info: &mut CpuInfo, output: &str) {
    for line in output.lines() {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }
        
        let key = parts[0].trim();
        let value = parts[1].trim();
        
        match key {
            "Architecture" => cpu_info.architecture = value.to_string(),
            "CPU op-mode(s)" => cpu_info.cpu_op_modes = value.split(", ").map(|s| s.to_string()).collect(),
            "Byte Order" => cpu_info.byte_order = value.to_string(),
            "CPU(s)" => cpu_info.cpus = value.parse().unwrap_or(cpu_info.cpus),
            "Thread(s) per core" => cpu_info.threads_per_core = value.parse().unwrap_or(1),
            "Core(s) per socket" => cpu_info.cores_per_socket = value.parse().unwrap_or(1),
            "Socket(s)" => cpu_info.sockets = value.parse().unwrap_or(1),
            "NUMA node(s)" => cpu_info.numa_nodes = value.parse().unwrap_or(1),
            "CPU max MHz" => cpu_info.cpu_max_mhz = value.parse().unwrap_or(0.0),
            "CPU min MHz" => cpu_info.cpu_min_mhz = value.parse().unwrap_or(0.0),
            "L1d cache" => cpu_info.l1d_cache = value.to_string(),
            "L1i cache" => cpu_info.l1i_cache = value.to_string(),
            "L2 cache" => cpu_info.l2_cache = value.to_string(),
            "L3 cache" => cpu_info.l3_cache = value.to_string(),
            _ => {}
        }
    }
}

#[cfg(not(windows))]
fn get_cache_info(cpu_info: &mut CpuInfo) {
    let cache_path = "/sys/devices/system/cpu/cpu0/cache";
    if std::path::Path::new(cache_path).exists() {
        if let Ok(entries) = fs::read_dir(cache_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(index_name) = path.file_name().and_then(|n| n.to_str()) {
                    if index_name.starts_with("index") {
                        read_cache_level(&path, cpu_info);
                    }
                }
            }
        }
    }
}

#[cfg(not(windows))]
fn read_cache_level(cache_path: &std::path::Path, cpu_info: &mut CpuInfo) {
    let level_path = cache_path.join("level");
    let size_path = cache_path.join("size");
    let type_path = cache_path.join("type");
    
    if let (Ok(level), Ok(size), Ok(cache_type)) = (
        fs::read_to_string(&level_path),
        fs::read_to_string(&size_path),
        fs::read_to_string(&type_path),
    ) {
        let level = level.trim();
        let size = size.trim();
        let cache_type = cache_type.trim();
        
        match (level, cache_type) {
            ("1", "Data") => cpu_info.l1d_cache = size.to_string(),
            ("1", "Instruction") => cpu_info.l1i_cache = size.to_string(),
            ("2", _) => cpu_info.l2_cache = size.to_string(),
            ("3", _) => cpu_info.l3_cache = size.to_string(),
            _ => {}
        }
    }
}

#[cfg(not(windows))]
fn get_vulnerability_info(cpu_info: &mut CpuInfo) {
    let vuln_path = "/sys/devices/system/cpu/vulnerabilities";
    if std::path::Path::new(vuln_path).exists() {
        if let Ok(entries) = fs::read_dir(vuln_path) {
            for entry in entries.flatten() {
                if let Some(vuln_name) = entry.file_name().to_str() {
                    if let Ok(status) = fs::read_to_string(entry.path()) {
                        cpu_info.vulnerabilities.insert(
                            vuln_name.to_string(),
                            status.trim().to_string(),
                        );
                    }
                }
            }
        }
    }
}

#[cfg(not(windows))]
fn get_numa_info(cpu_info: &mut CpuInfo) {
    let numa_path = "/sys/devices/system/node";
    if std::path::Path::new(numa_path).exists() {
        if let Ok(entries) = fs::read_dir(numa_path) {
            for entry in entries.flatten() {
                if let Some(node_name) = entry.file_name().to_str() {
                    if node_name.starts_with("node") {
                        let cpulist_path = entry.path().join("cpulist");
                        if let Ok(cpulist) = fs::read_to_string(&cpulist_path) {
                            cpu_info.numa_info.push(format!("{}: {}", node_name, cpulist.trim()));
                        }
                    }
                }
            }
        }
    }
}

fn print_standard_output(cpu_info: &CpuInfo, extended: bool, parse_column: Option<&str>) {
    if let Some(column) = parse_column {
        print_parsed_column(cpu_info, column);
        return;
    }

    println!("Architecture:        {}", cpu_info.architecture);
    println!("CPU op-mode(s):      {}", cpu_info.cpu_op_modes.join(", "));
    println!("Byte Order:          {}", cpu_info.byte_order);
    println!("CPU(s):              {}", cpu_info.cpus);
    println!("On-line CPU(s) list: {}", cpu_info.on_line_cpus);
    println!("Thread(s) per core:  {}", cpu_info.threads_per_core);
    println!("Core(s) per socket:  {}", cpu_info.cores_per_socket);
    println!("Socket(s):           {}", cpu_info.sockets);
    println!("NUMA node(s):        {}", cpu_info.numa_nodes);
    println!("Vendor ID:           {}", cpu_info.vendor_id);
    println!("Model name:          {}", cpu_info.model_name);
    println!("CPU family:          {}", cpu_info.cpu_family);
    println!("Model:               {}", cpu_info.model);
    println!("Stepping:            {}", cpu_info.stepping);
    
    if cpu_info.cpu_mhz > 0.0 {
        println!("CPU MHz:             {:.3}", cpu_info.cpu_mhz);
    }
    if cpu_info.cpu_max_mhz > 0.0 {
        println!("CPU max MHz:         {:.4}", cpu_info.cpu_max_mhz);
    }
    if cpu_info.cpu_min_mhz > 0.0 {
        println!("CPU min MHz:         {:.4}", cpu_info.cpu_min_mhz);
    }
    if cpu_info.bogomips > 0.0 {
        println!("BogoMIPS:            {:.2}", cpu_info.bogomips);
    }
    
    if cpu_info.l1d_cache != "Unknown" {
        println!("L1d cache:           {}", cpu_info.l1d_cache);
    }
    if cpu_info.l1i_cache != "Unknown" {
        println!("L1i cache:           {}", cpu_info.l1i_cache);
    }
    if cpu_info.l2_cache != "Unknown" {
        println!("L2 cache:            {}", cpu_info.l2_cache);
    }
    if cpu_info.l3_cache != "Unknown" {
        println!("L3 cache:            {}", cpu_info.l3_cache);
    }
    
    if !cpu_info.numa_info.is_empty() {
        for numa_line in &cpu_info.numa_info {
            println!("NUMA {numa_line}");
        }
    }

    if extended && !cpu_info.vulnerabilities.is_empty() {
        for (vuln, status) in &cpu_info.vulnerabilities {
            println!("Vulnerability {}:{}{}",
                vuln,
                " ".repeat(8 - vuln.len().min(7)),
                status
            );
        }
    }

    if extended && !cpu_info.flags.is_empty() {
        println!("Flags:               {}", cpu_info.flags.join(" "));
    }
}

fn print_parsed_column(cpu_info: &CpuInfo, column: &str) {
    match column.to_lowercase().as_str() {
        "cpu" => {
            for i in 0..cpu_info.cpus {
                println!("{i}");
            }
        }
        "core" => {
            for i in 0..cpu_info.cores_per_socket {
                println!("{i}");
            }
        }
        "socket" => {
            for i in 0..cpu_info.sockets {
                println!("{i}");
            }
        }
        "node" => {
            for i in 0..cpu_info.numa_nodes {
                println!("{i}");
            }
        }
        "online" => println!("{}", cpu_info.on_line_cpus),
        _ => eprintln!("Unknown column: {column}"),
    }
}

fn print_json_output(cpu_info: &CpuInfo) {
    println!("{{");
    println!("  \"architecture\": \"{}\",", cpu_info.architecture);
    println!("  \"cpuOpModes\": [{}],",
        cpu_info.cpu_op_modes.iter()
            .map(|s| format!("\"{s}\""))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("  \"byteOrder\": \"{}\",", cpu_info.byte_order);
    println!("  \"cpus\": {},", cpu_info.cpus);
    println!("  \"onLineCpus\": \"{}\",", cpu_info.on_line_cpus);
    println!("  \"threadsPerCore\": {},", cpu_info.threads_per_core);
    println!("  \"coresPerSocket\": {},", cpu_info.cores_per_socket);
    println!("  \"sockets\": {},", cpu_info.sockets);
    println!("  \"numaNodes\": {},", cpu_info.numa_nodes);
    println!("  \"vendorId\": \"{}\",", cpu_info.vendor_id);
    println!("  \"modelName\": \"{}\",", cpu_info.model_name);
    println!("  \"cpuFamily\": {},", cpu_info.cpu_family);
    println!("  \"model\": {},", cpu_info.model);
    println!("  \"stepping\": {},", cpu_info.stepping);
    
    if cpu_info.cpu_mhz > 0.0 {
        println!("  \"cpuMhz\": {},", cpu_info.cpu_mhz);
    }
    if cpu_info.cpu_max_mhz > 0.0 {
        println!("  \"cpuMaxMhz\": {},", cpu_info.cpu_max_mhz);
    }
    if cpu_info.cpu_min_mhz > 0.0 {
        println!("  \"cpuMinMhz\": {},", cpu_info.cpu_min_mhz);
    }
    
    println!("  \"caches\": {{");
    if cpu_info.l1d_cache != "Unknown" {
        println!("    \"l1d\": \"{}\",", cpu_info.l1d_cache);
    }
    if cpu_info.l1i_cache != "Unknown" {
        println!("    \"l1i\": \"{}\",", cpu_info.l1i_cache);
    }
    if cpu_info.l2_cache != "Unknown" {
        println!("    \"l2\": \"{}\",", cpu_info.l2_cache);
    }
    if cpu_info.l3_cache != "Unknown" {
        println!("    \"l3\": \"{}\"", cpu_info.l3_cache);
    }
    println!("  }},");
    
    if !cpu_info.flags.is_empty() {
        println!("  \"flags\": [{}],",
            cpu_info.flags.iter()
                .map(|s| format!("\"{s}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if !cpu_info.vulnerabilities.is_empty() {
        println!("  \"vulnerabilities\": {{");
        let vuln_items: Vec<String> = cpu_info.vulnerabilities.iter()
            .map(|(k, v)| format!("    \"{k}\": \"{v}\""))
            .collect();
        println!("{}", vuln_items.join(",\n"));
        println!("  }}");
    }
    
    println!("}}");
}
