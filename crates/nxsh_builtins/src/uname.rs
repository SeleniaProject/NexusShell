use std::env;
use crate::common::{BuiltinResult, BuiltinContext};

/// Display system information
pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    let mut show_all = false;
    let mut show_kernel_name = false;
    let mut show_nodename = false;
    let mut show_kernel_release = false;
    let mut show_kernel_version = false;
    let mut show_machine = false;
    let mut show_processor = false;
    let mut show_hardware_platform = false;
    let mut show_operating_system = false;

    // If no options are provided, default to showing kernel name
    let mut has_options = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--all" => {
                show_all = true;
                has_options = true;
            }
            "-s" | "--kernel-name" => {
                show_kernel_name = true;
                has_options = true;
            }
            "-n" | "--nodename" => {
                show_nodename = true;
                has_options = true;
            }
            "-r" | "--kernel-release" => {
                show_kernel_release = true;
                has_options = true;
            }
            "-v" | "--kernel-version" => {
                show_kernel_version = true;
                has_options = true;
            }
            "-m" | "--machine" => {
                show_machine = true;
                has_options = true;
            }
            "-p" | "--processor" => {
                show_processor = true;
                has_options = true;
            }
            "-i" | "--hardware-platform" => {
                show_hardware_platform = true;
                has_options = true;
            }
            "-o" | "--operating-system" => {
                show_operating_system = true;
                has_options = true;
            }
            "-h" | "--help" => {
                print_help();
                return Ok(0);
            }
            "--version" => {
                println!("uname (NexusShell builtins) 1.0.0");
                return Ok(0);
            }
            arg if arg.starts_with('-') => {
                eprintln!("uname: invalid option '{arg}'");
                return Ok(1);
            }
            _ => {
                eprintln!("uname: extra operand '{}'", args[i]);
                return Ok(1);
            }
        }
        i += 1;
    }

    // If no options specified, default to kernel name
    if !has_options {
        show_kernel_name = true;
    }

    // If -a is specified, show all information
    if show_all {
        show_kernel_name = true;
        show_nodename = true;
        show_kernel_release = true;
        show_kernel_version = true;
        show_machine = true;
        show_processor = true;
        show_hardware_platform = true;
        show_operating_system = true;
    }

    let mut values = Vec::new();

    if show_kernel_name {
        values.push(get_kernel_name());
    }

    if show_nodename {
        values.push(get_nodename());
    }

    if show_kernel_release {
        values.push(get_kernel_release());
    }

    if show_kernel_version {
        values.push(get_kernel_version());
    }

    if show_machine {
        values.push(get_machine());
    }

    if show_processor {
        values.push(get_processor());
    }

    if show_hardware_platform {
        values.push(get_hardware_platform());
    }

    if show_operating_system {
        values.push(get_operating_system());
    }

    if !values.is_empty() {
        println!("{}", values.join(" "));
    }

    Ok(0)
}

fn get_kernel_name() -> String {
    #[cfg(target_os = "linux")]
    return "Linux".to_string();
    
    #[cfg(target_os = "windows")]
    return "Windows".to_string();
    
    #[cfg(target_os = "macos")]
    return "Darwin".to_string();
    
    #[cfg(target_os = "freebsd")]
    return "FreeBSD".to_string();
    
    #[cfg(target_os = "openbsd")]
    return "OpenBSD".to_string();
    
    #[cfg(target_os = "netbsd")]
    return "NetBSD".to_string();
    
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos", 
                  target_os = "freebsd", target_os = "openbsd", target_os = "netbsd")))]
    return "Unknown".to_string();
}

fn get_nodename() -> String {
    // Try to get hostname from environment variables or system
    if let Ok(hostname) = env::var("HOSTNAME") {
        return hostname;
    }
    
    if let Ok(computername) = env::var("COMPUTERNAME") {
        return computername;
    }
    
    // Fallback to a generic name
    "localhost".to_string()
}

fn get_kernel_release() -> String {
    #[cfg(target_os = "windows")]
    {
        // On Windows, try to get version info
        "Unknown".to_string()
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // On Unix-like systems, would typically read from /proc/version or uname syscall
        "Unknown".to_string()
    }
}

fn get_kernel_version() -> String {
    #[cfg(target_os = "windows")]
    {
        // Would need Windows API calls to get detailed version
        "Unknown".to_string()
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Would typically parse /proc/version or use uname syscall
        "Unknown".to_string()
    }
}

fn get_machine() -> String {
    #[cfg(target_arch = "x86_64")]
    return "x86_64".to_string();
    
    #[cfg(target_arch = "x86")]
    return "i686".to_string();
    
    #[cfg(target_arch = "aarch64")]
    return "aarch64".to_string();
    
    #[cfg(target_arch = "arm")]
    return "arm".to_string();
    
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86", 
                  target_arch = "aarch64", target_arch = "arm")))]
    return "unknown".to_string();
}

fn get_processor() -> String {
    // Often same as machine architecture
    get_machine()
}

fn get_hardware_platform() -> String {
    // Often same as machine architecture
    get_machine()
}

fn get_operating_system() -> String {
    #[cfg(target_os = "linux")]
    return "GNU/Linux".to_string();
    
    #[cfg(target_os = "windows")]
    return "Windows".to_string();
    
    #[cfg(target_os = "macos")]
    return "Darwin".to_string();
    
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    return "Unknown".to_string();
}

fn print_help() {
    println!("Usage: uname [OPTION]...");
    println!("Print certain system information. With no OPTION, same as -s.");
    println!();
    println!("Options:");
    println!("  -a, --all                print all information, in the following order,");
    println!("                           except omit -p and -i if unknown:");
    println!("  -s, --kernel-name        print the kernel name");
    println!("  -n, --nodename           print the network node hostname");
    println!("  -r, --kernel-release     print the kernel release");
    println!("  -v, --kernel-version     print the kernel version");
    println!("  -m, --machine            print the machine hardware name");
    println!("  -p, --processor          print the processor type (non-portable)");
    println!("  -i, --hardware-platform  print the hardware platform (non-portable)");
    println!("  -o, --operating-system   print the operating system");
    println!("  -h, --help               display this help and exit");
    println!("      --version            output version information and exit");
    println!();
    println!("Examples:");
    println!("  uname           Print kernel name");
    println!("  uname -a        Print all system information");
    println!("  uname -sr       Print kernel name and release");
}
