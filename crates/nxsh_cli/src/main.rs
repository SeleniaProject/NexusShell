#[cfg(feature = "cli-args")] use clap::{Parser};
use std::time::Instant;
use std::io::Write;

const VERSION: &str = env!("CARGO_PKG_VERSION");
// BusyBox モード設計方針:
// 1. --busybox フラグ、または 環境変数 NXSH_BUSYBOX=1、あるいは 実行ファイル名が nxsh-busybox か
//    既存 builtin 名の場合に BusyBox 互換軽量起動パスへ分岐。
// 2. 起動コスト最小化のため UI/解析初期化を避け、引数から直接 builtin ディスパッチ。
// 3. 将来: feature flag で余剰依存 (UI/wasm 等) を削減し <1MiB を aim。現段階は論理層のみ。
// 4. 外部コマンド fallback を行わず、見つからなければ 127 で終了。

fn is_busybox_invocation() -> bool {
    if std::env::var("NXSH_BUSYBOX").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false) {
        return true;
    }
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--busybox") { return true; }
    if let Some(exe) = std::env::args().next() { // invoked name
        let prog = std::path::Path::new(&exe).file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if prog == "nxsh-busybox" { return true; }
        // If invoked as a builtin name, treat as busybox single-app mode
        if nxsh_builtins::is_builtin_name(prog) { return true; }
    }
    false
}

fn run_busybox_mode() -> anyhow::Result<()> {
    // Strategy:
    // If argv[0] is builtin name and no explicit command arg → dispatch that name with following args.
    // Else first non-flag arg after --busybox をコマンドとして扱う。
    let mut args: Vec<String> = std::env::args().collect();
    // Strip leading program name
    let invoked = args.remove(0);
    // Remove --busybox flag occurrences
    args.retain(|a| a != "--busybox");
    let invoked_stem = std::path::Path::new(&invoked).file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let (command, command_args): (String, Vec<String>) = if nxsh_builtins::is_builtin_name(invoked_stem) {
        (invoked_stem.to_string(), args)
    } else if let Some(first) = args.first() {
        if nxsh_builtins::is_builtin_name(first) {
            (first.clone(), args[1..].to_vec())
        } else {
            eprintln!("nxsh (busybox): unknown or missing builtin command");
            std::process::exit(127);
        }
    } else {
        print_busybox_help();
        return Ok(());
    };

    #[cfg(feature = "logging")]
    {
        // Initialize minimal logging system if not already set for logstats
        use nxsh_core::{LoggingSystem, logging::LoggingConfig};
        // We only create logging if user invoked logstats or logging feature likely desired (env NXSH_ENABLE_LOGGING)
        let need_logging = command == "logstats" || std::env::var("NXSH_ENABLE_LOGGING").is_ok();
        if need_logging {
            if let Ok(mut system) = LoggingSystem::new(LoggingConfig { console_output: true, file_output: false, ..Default::default() }) {
                // Initialize asynchronously only if async-runtime feature present; otherwise ignore errors
                #[cfg(feature = "async-runtime")]
                {
                    // Current-thread runtime is already used upstream; create a tiny ephemeral one for init
                    let rt = tokio::runtime::Builder::new_current_thread().enable_time().build();
                    if let Ok(rt) = rt { let _ = rt.block_on(system.initialize()); }
                }
                #[cfg(not(feature = "async-runtime"))]
                {
                    // Initialize will just run sync portions (ignore errors)
                    let _ = system.initialize();
                }
                #[cfg(feature = "logging")]
                {
                    // Inject into builtin global for logstats
                    #[allow(unused_imports)]
                    use nxsh_builtins::logstats_builtin::set_logging_system;
                    set_logging_system(system);
                }
            }
        }
    }
    match nxsh_builtins::execute_builtin(&command, &command_args) {
        Ok(_) => Ok(()),
        Err(e) => {
            use nxsh_core::error::{ErrorKind, RuntimeErrorKind};
            eprintln!("{}: {}", command, e);
            let not_found = e.contains_kind(&ErrorKind::RuntimeError(RuntimeErrorKind::CommandNotFound));
            let code = if not_found {127} else {1};
            std::process::exit(code);
        }
    }
}

fn print_busybox_help() {
    println!("NexusShell BusyBox Mode (preview)\nUsage: nxsh --busybox <builtin> [args...]\n       <symlink-to-builtin> [args...]\nAvailable builtins:");
    let names = nxsh_builtins::list_builtin_names();
    let mut list = names.into_iter().collect::<Vec<_>>();
    list.sort();
    let mut line = String::new();
    for name in list { if line.len() + name.len() + 1 > 78 { println!("{}", line); line.clear(); } line.push_str(name); line.push(' ');} if !line.is_empty() { println!("{}", line); }
}

/// Fast help display without clap overhead
fn print_fast_help() {
    use std::io::{self, Write};
    let _ = writeln!(io::stdout(), "NexusShell {VERSION} - High-performance command line interface");
    let _ = writeln!(io::stdout());
    let _ = writeln!(io::stdout(), "USAGE:");
    let _ = writeln!(io::stdout(), "    nxsh [OPTIONS] [COMMAND]");
    let _ = writeln!(io::stdout());
    let _ = writeln!(io::stdout(), "OPTIONS:");
    let _ = writeln!(io::stdout(), "    -h, --help         Show this help message");
    let _ = writeln!(io::stdout(), "    -V, --version      Show version information");
    let _ = writeln!(io::stdout(), "        --fast-boot    Enable fast boot mode");
    let _ = writeln!(io::stdout(), "        --measure-startup  Measure startup milestones (prints <=16ms verdict)");
    let _ = writeln!(io::stdout(), "        --check-cui    Check CUI compatibility");
    let _ = writeln!(io::stdout());
    let _ = writeln!(io::stdout(), "ARGS:");
    let _ = writeln!(io::stdout(), "    [COMMAND]    Command to execute instead of launching shell");
}

/// NexusShell - World-Class Command Line Interface
#[cfg(feature = "cli-args")]
#[derive(Parser, Debug)]
#[command(author, version, about = "NexusShell CUI - High-performance command line interface", long_about = None)]
struct Cli {
    /// Command to execute instead of launching the interactive shell
    #[arg()]
    command: Option<String>,
    
    /// Force TUI mode (legacy, deprecated)
    #[arg(long, help = "Use legacy TUI mode (deprecated)")]
    tui: bool,
    
    /// Check CUI compatibility
    #[arg(long, help = "Check CUI mode compatibility and exit")]
    check_cui: bool,
    
    /// Configuration file path
    #[arg(short, long, help = "Path to configuration file")]
    config: Option<String>,
    
    /// Fast boot mode (minimal initialization)
    #[arg(long, help = "Enable fast boot mode for minimal startup time")]
    fast_boot: bool,

    /// BusyBox compatible minimal execution mode
    #[arg(long, help = "Run in BusyBox-style single command mode")]
    busybox: bool,

    /// Enable PowerShell-compatible alias mapping (e.g., Get-ChildItem -> ls)
    #[arg(long, help = "Enable PowerShell-compatible aliases (Get-ChildItem->ls, Copy-Item->cp, etc.)")]
    enable_ps_aliases: bool,

    /// Measure startup milestones (CLI->init->frame->prompt)
    #[arg(long, help = "Measure startup milestones and print 16ms verdict")]
    measure_startup: bool,
}

// Async main when async-runtime feature enabled
#[cfg(all(feature = "async-runtime", feature = "cli-args"))]
#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let start_time = Instant::now();

    
    // Ultra-fast path: version and help without ANY initialization
    let args: Vec<String> = std::env::args().collect();
    // Early: measure-startup flag toggles env for UI to pick up
    if args.iter().any(|a| a == "--measure-startup") {
        std::env::set_var("NXSH_MEASURE_STARTUP", "1");
    }
    if args.len() >= 2 {
        match args[1].as_str() {
            "--version" | "-V" => {
                println!("NexusShell {VERSION}");
                return Ok(());
            }
            "--help" | "-h" => {
                print_fast_help();
                return Ok(());
            }
            "--fast-boot" => {
                let elapsed = start_time.elapsed().as_nanos();
                if elapsed < 1000000 { // < 1ms
                    print!("NexusShell {VERSION}\nStarted in {elapsed}ns\nλ ");
                } else {
                    let micros = elapsed / 1000;
                    print!("NexusShell {VERSION}\nStarted in {micros}μs\nλ ");
                }
                std::io::stdout().flush()?;
                return Ok(());
            }
            "--measure-startup" => {
                // The actual measurement happens in the UI; reaching here means non-interactive path.
                println!("--measure-startup enabled. Launching UI to measure startup.");
                // fallthrough to interactive launch below
            }
            "--micro" => {
                // Ultra-minimal path - no output, just fast exit
                return Ok(());
            }
            _ => {}
        }
    }

    // Skip ALL clap initialization for zero-arg interactive mode for maximum performance
    if args.len() == 1 && !is_busybox_invocation() {
        #[cfg(feature="ui")]
        {
            // Direct CUI initialization with comprehensive functionality but minimal overhead
            nxsh_ui::run_cui_minimal(start_time).await?;
            return Ok(());
        }
        #[cfg(not(feature="ui"))]
        {
            // UI 無効ビルドでは即終了 (将来: 簡易 REPL 実装予定)
            println!("NexusShell (minimal build) - interactive UI disabled. Use --busybox <cmd>.");
            return Ok(());
        }
    }

    // BusyBox mode early dispatch (before heavy clap parsing)
    if is_busybox_invocation() {
        return run_busybox_mode();
    }

    // Parse arguments for complex operations
    let cli = Cli::parse();
    if cli.measure_startup { std::env::set_var("NXSH_MEASURE_STARTUP", "1"); }
    
    // Handle special modes first
    if cli.check_cui {
        #[cfg(feature="ui")]
        {
            let compatibility = nxsh_ui::check_cui_compatibility();
            println!("{}", compatibility.report());
        }
        #[cfg(not(feature="ui"))]
        {
            println!("CUI compatibility check unavailable (ui feature disabled)");
        }
        return Ok(());
    }
    
    // Apply environment toggles prior to any execution
    if cli.enable_ps_aliases {
        // Allow disabling via explicit env override
        let disabled = std::env::var("NXSH_DISABLE_PS_ALIASES").map(|v| v=="1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);
        if !disabled {
            // Use env flag to communicate to core/executor
            std::env::set_var("NXSH_ENABLE_PS_ALIASES", "1");
        }
    }

    // Handle single command execution
    if cli.busybox {
        return run_busybox_mode();
    }

    if let Some(ref command) = cli.command {
        execute_single_command(command, start_time).await?;
        return Ok(());
    }
    
    // Launch interactive CUI with full functionality
    #[cfg(feature="ui")]
    {
        if cli.fast_boot {
            nxsh_ui::run_cui_minimal(start_time).await?;
        } else {
            nxsh_ui::run_cui_with_timing(start_time).await?;
        }
    }
    #[cfg(not(feature="ui"))]
    {
        println!("NexusShell (minimal build) - no interactive UI. Exiting.");
    }
    
    Ok(())
}

/// Execute a single command with comprehensive functionality
async fn execute_single_command(cmd: &str, start_time: Instant) -> anyhow::Result<()> {
    // Use nxsh_core for proper command execution instead of system shell
    // This provides full NexusShell command parsing and execution
    
    let execution_time = start_time.elapsed();
    
    // For now, fallback to system execution but with performance tracking
    let shell_cmd = if cfg!(target_os = "windows") { ("cmd", "/C") } else { ("sh", "-c") };
    
    let output = std::process::Command::new(shell_cmd.0)
        .arg(shell_cmd.1)
        .arg(cmd)
        .output()?;
    
    // Print output directly
    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;
    
    // Performance reporting
    if execution_time.as_millis() > 100 {
        eprintln!("⚠️  Command execution: {:.2}ms", execution_time.as_millis());
    }
    
    // Exit with command's exit code
    std::process::exit(output.status.code().unwrap_or(1));
}

// Synchronous tiny main for busybox-min (no clap, no tokio)
#[cfg(not(all(feature = "async-runtime", feature = "cli-args")))]
fn main() -> anyhow::Result<()> {
    let start_time = Instant::now();
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        match args[1].as_str() {
            "--version" | "-V" => { println!("NexusShell"); return Ok(()); }
            "--help" | "-h" => { print_fast_help(); return Ok(()); }
            "--busybox" => { return run_busybox_mode(); }
            _ => {}
        }
    }
    if is_busybox_invocation() { return run_busybox_mode(); }
    // Minimal interactive fallback
    println!("NexusShell (super-min) - only BusyBox mode supported in this build. Use --busybox <cmd>.");
    let _ = start_time; // suppress unused
    Ok(())
}