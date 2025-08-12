use anyhow::anyhow;
use nxsh_core::{
    context::ShellContext,
    error::{ErrorKind, RuntimeErrorKind, ShellError, ShellResult},
    ExecutionResult,
};

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct LtraceOptions {
    pub follow_forks: bool,
    pub count_calls: bool,
    pub time_calls: bool,
    pub output_file: Option<String>,
    pub pid: Option<u32>,
    pub program: Option<String>,
    pub program_args: Vec<String>,
    pub library_filter: Vec<String>,
    pub function_filter: Vec<String>,
}


pub fn ltrace(_ctx: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult> {
    let options = parse_ltrace_args(args)?;
    
    if options.program.is_none() && options.pid.is_none() {
        return Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
            "ltrace: must specify either a program to trace or a PID",
        ));
    }
    
    #[cfg(unix)]
    {
        execute_ltrace_unix(&options)
    }
    
    #[cfg(windows)]
    {
        execute_ltrace_windows(&options)
    }
}

fn parse_ltrace_args(args: &[String]) -> ShellResult<LtraceOptions> {
    let mut options = LtraceOptions::default();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "-f" => {
                options.follow_forks = true;
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
            "-l" => {
                if i + 1 < args.len() {
                    options.library_filter.push(args[i + 1].clone());
                    i += 1;
                }
            }
            "-e" => {
                if i + 1 < args.len() {
                    options.function_filter.push(args[i + 1].clone());
                    i += 1;
                }
            }
            "--help" => {
                show_ltrace_help();
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
fn execute_ltrace_unix(options: &LtraceOptions) -> ShellResult<ExecutionResult> {
    let mut cmd = Command::new("ltrace");
    
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
    
    for library in &options.library_filter {
        cmd.args(&["-l", library]);
    }
    
    for function in &options.function_filter {
        cmd.args(&["-e", function]);
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
                    &format!("ltrace: failed to wait for child: {}", e),
                    "",
                    0,
                )),
            }
        }
        Err(e) => Err(ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::ExitError),
            &format!("ltrace: failed to execute: {}", e),
            "",
            0,
        )),
    }
}

#[cfg(windows)]
fn execute_ltrace_windows(options: &LtraceOptions) -> ShellResult<ExecutionResult> {
    // Windows doesn't have ltrace, but we can use API Monitor or similar tools
    eprintln!("ltrace: not available on Windows");
    eprintln!("Use API Monitor, Detours, or similar tools for library call tracing");
    
    if let Some(program) = &options.program {
        println!("Would trace library calls for program: {program}");
        for arg in &options.program_args {
            println!("  with arg: {arg}");
        }
    }
    
    if let Some(pid) = options.pid {
        println!("Would trace library calls for PID: {pid}");
    }
    
    for library in &options.library_filter {
        println!("Filter library: {library}");
    }
    
    for function in &options.function_filter {
        println!("Filter function: {function}");
    }
    
    Ok(ExecutionResult::success(0))
}

fn show_ltrace_help() {
    println!("Usage: ltrace [options] command [args]");
    println!("       ltrace [options] -p pid");
    println!("Trace library calls");
    println!();
    println!("Options:");
    println!("  -f              trace child processes");
    println!("  -c              count time, calls, and errors");
    println!("  -t              print timestamps");
    println!("  -o file         write output to file");
    println!("  -p pid          attach to process with given pid");
    println!("  -l library      trace calls from specified library");
    println!("  -e filter       trace only specified function calls");
    println!("      --help      display this help and exit");
}

/// CLI wrapper function for ltrace command
pub fn ltrace_cli(args: &[String]) -> anyhow::Result<()> {
    let mut ctx = nxsh_core::context::ShellContext::new();
    match ltrace(&mut ctx, args) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("ltrace command failed: {}", e)),
    }
}
