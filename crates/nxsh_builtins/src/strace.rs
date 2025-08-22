use anyhow::anyhow;
use nxsh_core::{
    context::ShellContext,
    error::{ErrorKind, RuntimeErrorKind, ShellError, ShellResult},
    ExecutionResult,
};

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct StraceOptions {
    pub follow_forks: bool,
    pub follow_vforks: bool,
    pub follow_clones: bool,
    pub count_calls: bool,
    pub time_calls: bool,
    pub output_file: Option<String>,
    pub pid: Option<u32>,
    pub program: Option<String>,
    pub program_args: Vec<String>,
    pub trace_filter: Vec<String>,
}


pub fn strace(_ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
    let options = parse_strace_args(args)?;
    
    if options.program.is_none() && options.pid.is_none() {
        return Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            "strace: must specify either a program to trace or a PID",
        ));
    }
    
    #[cfg(unix)]
    {
        execute_strace_unix(&options)
    }
    
    #[cfg(windows)]
    {
        execute_strace_windows(&options)
    }
}

fn parse_strace_args(args: &[String]) -> ShellResult<StraceOptions> {
    let mut options = StraceOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-f" => {
                options.follow_forks = true;
            }
            "-ff" => {
                options.follow_forks = true;
                options.follow_vforks = true;
            }
            "-c" => {
                options.count_calls = true;
            }
            "-t" => {
                options.time_calls = true;
            }
            "-o" => {
                if i + 1 < args.len() {
                    options.output_file = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "-p" => {
                if i + 1 < args.len() {
                    if let Ok(pid) = args[i + 1].parse::<u32>() {
                        options.pid = Some(pid);
                    }
                    i += 1;
                }
            }
            "-e" => {
                if i + 1 < args.len() {
                    options.trace_filter.push(args[i + 1].clone());
                    i += 1;
                }
            }
            "--help" => {
                show_strace_help();
                return Ok(options);
            }
            arg => {
                if !arg.starts_with('-') && options.program.is_none() {
                    options.program = Some(arg.to_string());
                } else if !arg.starts_with('-') {
                    options.program_args.push(arg.to_string());
                }
            }
        }
        i += 1;
    }
    
    Ok(options)
}

#[cfg(unix)]
fn execute_strace_unix(options: &StraceOptions) -> ShellResult<ExecutionResult> {
    let mut cmd = Command::new("strace");
    
    if options.follow_forks {
        cmd.arg("-f");
    }
    if options.count_calls {
        cmd.arg("-c");
    }
    if options.time_calls {
        cmd.arg("-t");
    }
    
    if let Some(output_file) = &options.output_file {
        cmd.args(&["-o", output_file]);
    }
    
    if let Some(pid) = options.pid {
        cmd.args(&["-p", &pid.to_string()]);
    }
    
    for filter in &options.trace_filter {
        cmd.args(&["-e", filter]);
    }
    
    if let Some(program) = &options.program {
        cmd.arg(program);
        for arg in &options.program_args {
            cmd.arg(arg);
        }
    }
    
    cmd.stdin(Stdio::inherit())
       .stdout(Stdio::inherit())
       .stderr(Stdio::inherit());
    
    match cmd.spawn() {
        Ok(mut child) => {
            match child.wait() {
                Ok(status) => {
                    let exit_code = status.code().unwrap_or(1);
                    Ok(ExecutionResult::success(exit_code))
                }
                Err(e) => Err(ShellError::new(
                    ErrorKind::RuntimeError(RuntimeErrorKind::ExitError),
                    &format!("strace: failed to wait for child: {}", e),
                    "",
                    0,
                )),
            }
        }
        Err(e) => Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::ExitError),
            &format!("strace: failed to execute: {}", e),
            "",
            0,
        )),
    }
}

#[cfg(windows)]
fn execute_strace_windows(options: &StraceOptions) -> ShellResult<ExecutionResult> {
    // Windows doesn't have strace, but we can use Process Monitor or similar tools
    eprintln!("strace: not available on Windows");
    eprintln!("Use Process Monitor, WPA, or ETW for similar functionality");
    
    if let Some(program) = &options.program {
        println!("Would trace program: {program}");
        for arg in &options.program_args {
            println!("  with arg: {arg}");
        }
    }
    
    if let Some(pid) = options.pid {
        println!("Would trace PID: {pid}");
    }
    
    Ok(ExecutionResult::success(0))
}

fn show_strace_help() {
    println!("Usage: strace [options] command [args]");
    println!("       strace [options] -p pid");
    println!("Trace system calls and signals");
    println!();
    println!("Options:");
    println!("  -f              trace child processes");
    println!("  -ff             trace child processes with separate output");
    println!("  -c              count time, calls, and errors");
    println!("  -t              print timestamps");
    println!("  -o file         write output to file");
    println!("  -p pid          attach to process with given pid");
    println!("  -e trace=set    trace only specified system calls");
    println!("      --help      display this help and exit");
}

/// CLI wrapper function for strace command
pub fn strace_cli(args: &[String]) -> anyhow::Result<()> {
    let mut ctx = nxsh_core::context::ShellContext::new();
    match strace(&mut ctx, args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("strace command failed: {}", e)),
    }
}

