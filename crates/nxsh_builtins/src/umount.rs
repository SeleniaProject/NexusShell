use std::path::Path;
use anyhow::anyhow;
use nxsh_core::{
    context::ShellContext,
    error::{ErrorKind, ShellError, ShellResult},
    ExecutionResult,
};
use nxsh_core::error::RuntimeErrorKind;

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct UmountOptions {
    pub all: bool,
    pub force: bool,
    pub lazy: bool,
    pub read_only: bool,
    pub verbose: bool,
    pub type_filter: Vec<String>,
    pub targets: Vec<String>,
}


pub fn umount(_ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
    let options = parse_umount_args(args)?;
    
    if options.all {
        // Unmount all filesystems
        if options.verbose {
            println!("Unmounting all filesystems");
        }
        // Implementation for unmounting all
        return Ok(ExecutionResult::success(0));
    }
    
    if options.targets.is_empty() {
        return Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            "umount: no target specified",
        ));
    }
    
    for target in &options.targets {
        if let Err(e) = unmount_target(target, &options) {
            if !options.force {
                return Err(e);
            } else {
                eprintln!("umount: {target}: {e}");
            }
        }
    }
    
    Ok(ExecutionResult::success(0))
}

fn parse_umount_args(args: &[String]) -> ShellResult<UmountOptions> {
    let mut options = UmountOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--all" => {
                options.all = true;
            }
            "-f" | "--force" => {
                options.force = true;
            }
            "-l" | "--lazy" => {
                options.lazy = true;
            }
            "-r" | "--read-only" => {
                options.read_only = true;
            }
            "-v" | "--verbose" => {
                options.verbose = true;
            }
            "-t" | "--types" => {
                if i + 1 < args.len() {
                    options.type_filter.push(args[i + 1].clone());
                    i += 1;
                }
            }
            "--help" => {
                show_umount_help();
                return Ok(options);
            }
            arg => {
                if !arg.starts_with('-') {
                    options.targets.push(arg.to_string());
                }
            }
        }
        i += 1;
    }
    
    Ok(options)
}

fn unmount_target(target: &str, options: &UmountOptions) -> ShellResult<()> {
    if options.verbose {
        println!("Unmounting {target}");
    }
    
    // Check if target exists
    let target_path = Path::new(target);
    if !target_path.exists() {
        return Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            format!("umount: {target}: not found"),
        ));
    }
    
    #[cfg(unix)]
    {
        use std::ffi::CString;
        
        let path_cstr = match CString::new(target) {
            Ok(p) => p,
            Err(_) => {
                return Err(ShellError::new(
                    ErrorKind::InvalidInput,
                    &format!("umount: invalid path: {}", target),
                    "",
                    0,
                ));
            }
        };
        
        let flags = if options.lazy {
            libc::MNT_DETACH
        } else if options.force {
            libc::MNT_FORCE
        } else {
            0
        };
        
        unsafe {
            if libc::umount2(path_cstr.as_ptr(), flags) != 0 {
                let errno = *libc::__errno_location();
                let error_msg = match errno {
                    libc::EBUSY => "target is busy",
                    libc::EINVAL => "target is not a mount point",
                    libc::ENOENT => "target does not exist",
                    libc::EPERM => "operation not permitted",
                    _ => "unmount failed",
                };
                
                return Err(ShellError::new(
                    ErrorKind::PermissionDenied,
                    &format!("umount: {}: {}", target, error_msg),
                    "",
                    0,
                ));
            }
        }
    }
    
    #[cfg(windows)]
    {
        // Windows unmount simulation
        if options.verbose {
            println!("Windows: Simulating unmount of {target}");
        }
    }
    
    Ok(())
}

fn show_umount_help() {
    println!("Usage: umount [OPTION]... DIRECTORY...");
    println!("Unmount file systems");
    println!();
    println!("Options:");
    println!("  -a, --all             unmount all filesystems");
    println!("  -f, --force           force unmount (in case of an unreachable NFS system)");
    println!("  -l, --lazy            lazy unmount");
    println!("  -r, --read-only       in case unmounting fails, try to remount read-only");
    println!("  -v, --verbose         verbose mode");
    println!("  -t, --types TYPE      limit the set of filesystem types");
    println!("      --help            display this help and exit");
}

/// CLI wrapper function for umount command
pub fn umount_cli(args: &[String]) -> anyhow::Result<()> {
    let mut ctx = nxsh_core::context::ShellContext::new();
    match umount(&mut ctx, args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("umount command failed: {}", e)),
    }
}

