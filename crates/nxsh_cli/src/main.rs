#[cfg(feature = "cli-args")] use clap::{Parser};
use std::time::Instant;
use std::io::{self, Write, IsTerminal};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Show stylish startup banner
fn show_startup_banner() {
    // Cyberpunk color scheme
    let cyan = "\x1b[38;2;0;245;255m";      // #00f5ff
    let purple = "\x1b[38;2;153;69;255m";   // #9945ff
    let coral = "\x1b[38;2;255;71;87m";     // #ff4757
    let green = "\x1b[38;2;46;213;115m";    // #2ed573
    let yellow = "\x1b[38;2;255;190;11m";   // #ffbe0b
    let blue = "\x1b[38;2;116;185;255m";    // #74b9ff
    let bold = "\x1b[1m";
    let reset = "\x1b[0m";
    
    println!("{}{}┌─────────────────────────────────────────────────────────────┐{}", bold, cyan, reset);
    println!("{}│{}        {}███╗   ██╗███████╗██╗  ██╗██╗   ██╗███████╗{}        {}│{}", cyan, reset, purple, reset, cyan, reset);
    println!("{}│{}        {}████╗  ██║██╔════╝╚██╗██╔╝██║   ██║██╔════╝{}        {}│{}", cyan, reset, purple, reset, cyan, reset);
    println!("{}│{}        {}██╔██╗ ██║█████╗   ╚███╔╝ ██║   ██║███████╗{}        {}│{}", cyan, reset, purple, reset, cyan, reset);
    println!("{}│{}        {}██║╚██╗██║██╔══╝   ██╔██╗ ██║   ██║╚════██║{}        {}│{}", cyan, reset, purple, reset, cyan, reset);
    println!("{}│{}        {}██║ ╚████║███████╗██╔╝ ██╗╚██████╔╝███████║{}        {}│{}", cyan, reset, purple, reset, cyan, reset);
    println!("{}│{}        {}╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝{}        {}│{}", cyan, reset, purple, reset, cyan, reset);
    println!("{}├─────────────────────────────────────────────────────────────┤{}", cyan, reset);
    println!("{}│{}  {}🚀 Welcome to NexusShell v{}{:<3}{} - Cyberpunk Edition 🚀{}   {}│{}", cyan, reset, coral, yellow, VERSION, coral, reset, cyan, reset);
    println!("{}│{}  {}✨ Modern POSIX-compatible shell with style ✨{}             {}│{}", cyan, reset, green, reset, cyan, reset);
    println!("{}├─────────────────────────────────────────────────────────────┤{}", cyan, reset);
    println!("{}│{}  {}💡 Quick Start:{}                                            {}│{}", cyan, reset, blue, reset, cyan, reset);
    println!("{}│{}    {}• Type 'help' for command overview{}                      {}│{}", cyan, reset, yellow, reset, cyan, reset);
    println!("{}│{}    {}• Try 'echo --stylish \"Hello World!\"'{}                   {}│{}", cyan, reset, yellow, reset, cyan, reset);
    println!("{}│{}    {}• Use 'clear --banner' for welcome screen{}               {}│{}", cyan, reset, yellow, reset, cyan, reset);
    println!("{}│{}    {}• Type 'exit' or 'quit' to leave{}                        {}│{}", cyan, reset, yellow, reset, cyan, reset);
    println!("{}└─────────────────────────────────────────────────────────────┘{}", cyan, reset);
}

/// Show stylish bash-like prompt with cyberpunk colors
fn show_stylish_prompt() {
    use std::env;
    use std::path::PathBuf;
    
    // Cyberpunk color scheme
    let cyan = "\x1b[38;2;0;245;255m";      // #00f5ff
    let purple = "\x1b[38;2;153;69;255m";   // #9945ff
    let coral = "\x1b[38;2;255;71;87m";     // #ff4757
    let green = "\x1b[38;2;46;213;115m";    // #2ed573
    let yellow = "\x1b[38;2;255;190;11m";   // #ffbe0b
    let _white = "\x1b[37m";
    let bold = "\x1b[1m";
    let reset = "\x1b[0m";

    // Get current directory
    let current_dir = env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("~"))
        .display()
        .to_string();
    
    // Shorten path for display (show last 2 components if too long)
    let display_path = if current_dir.len() > 40 {
        let path = PathBuf::from(&current_dir);
        let components: Vec<_> = path.components().collect();
        if components.len() > 2 {
            format!(".../{}/{}", 
                components[components.len()-2].as_os_str().to_string_lossy(),
                components[components.len()-1].as_os_str().to_string_lossy())
        } else {
            current_dir
        }
    } else {
        current_dir
    };

    // Get username (or fallback)
    let username = env::var("USERNAME")
        .or_else(|_| env::var("USER"))
        .unwrap_or_else(|_| "nexus".to_string());

    // Get hostname (or fallback)  
    let hostname = env::var("COMPUTERNAME")
        .or_else(|_| env::var("HOSTNAME"))
        .unwrap_or_else(|_| "nexus-shell".to_string());

    // Check if we're in a git repository
    let git_info = get_git_info();

    // Construct the stylish prompt
    print!("{}{}╭─{}[{}{}@{}{}{}", 
        bold, cyan,           // Bold cyan for box drawing
        reset, purple,        // Reset, then purple for bracket
        username, hostname,   // Username and hostname
        purple, reset);       // Purple bracket close, reset

    print!(" {}📁 {}{}", coral, display_path, reset);

    // Add git info if available
    if let Some(branch) = git_info {
        print!(" {}🌿 {}{}", green, branch, reset);
    }

    print!("{}{}]{}", purple, reset, reset);
    
    // Time display
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    
    print!(" {}⏰ {:02}:{:02}{}", yellow, hours, minutes, reset);
    
    println!();
    print!("{}{}╰─{}❯{} ", bold, cyan, coral, reset);
}

/// Get git branch information if in a git repository
fn get_git_info() -> Option<String> {
    use std::process::Command;
    
    // Try to get git branch
    if let Ok(output) = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output() 
    {
        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !branch.is_empty() && branch != "HEAD" {
                return Some(branch);
            }
        }
    }
    None
}

// BusyBox mode design principles:
// 1. --busybox flag, or environment variable NXSH_BUSYBOX=1, or executable name nxsh-busybox
//    switches to BusyBox compatible lightweight startup path for existing builtin names
// 2. Minimize startup cost by avoiding UI/parser initialization, dispatch directly to builtins from arguments
// 3. Goal: reduce excess dependencies (UI/wasm etc) with feature flags to aim for <1MiB. Currently logic layer only
// 4. No external command fallback, exit with 127 if not found

fn is_busybox_invocation() -> bool {
    if std::env::var("NXSH_BUSYBOX").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false) {
        return true;
    }
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--busybox") { return true; }
    if let Some(exe) = std::env::args().next() { // invoked name
        let prog = std::path::Path::new(&exe).file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if prog == "nxsh-busybox" || prog.starts_with("nxsh-") {
            return true;
        }
    }
    false
}

fn busybox_mode() -> ! {
    let args: Vec<String> = std::env::args().collect();
    
    // Get first argument (program name or command name)
    let command = if args.len() > 1 && !args[1].starts_with("--") {
        &args[1]
    } else {
        // Infer command from program name
        if let Some(exe) = args.get(0) {
            let prog = std::path::Path::new(exe).file_stem()
                .and_then(|s| s.to_str()).unwrap_or("");
            if prog.starts_with("nxsh-") {
                &prog[5..] // Remove "nxsh-" prefix
            } else {
                prog
            }
        } else {
            ""
        }
    };

    // Prepare argument list
    let cmd_args = if args.len() > 1 && !args[1].starts_with("--") {
        &args[2..] // Exclude command name
    } else {
        &args[1..] // When inferred from program name
    };

    // Execute builtin commands using the central dispatcher
    let context = nxsh_builtins::BuiltinContext::new();
    match nxsh_builtins::execute_builtin(command, cmd_args, &context) {
        Ok(exit_code) => {
            std::process::exit(exit_code);
        }
        Err(nxsh_builtins::BuiltinError::UnknownCommand(_)) => {
            eprintln!("nxsh-busybox: {}: command not found", command);
            std::process::exit(127);
        }
        Err(e) => {
            eprintln!("nxsh-busybox: {}: {}", command, e);
            std::process::exit(1);
        }
    }
}

#[allow(dead_code)]
fn print_busybox_help() {
    println!("NexusShell BusyBox v{}", VERSION);
    println!("Usage: nxsh-busybox [COMMAND] [ARGS...]");
    println!();
    println!("Supported commands:");
    println!("  ls, pwd, cd, echo, cat, touch, mkdir, rm, cp, mv");
    println!("  grep, find, head, tail, wc, sort, uniq, date, env");
    println!("  which, history, alias, help");
    println!();
    println!("For individual command help: nxsh-busybox COMMAND --help");
}

#[cfg(feature = "cli-args")]
#[derive(Parser)]
#[command(name = "nxsh")]
#[command(version = VERSION)]
#[command(about = "NexusShell - Modern POSIX-compatible shell")]
#[command(author = "NexusShell Project")]
struct CliArgs {
    /// Start in BusyBox compatible mode
    #[arg(long)]
    busybox: bool,
    
    /// Force interactive mode
    #[arg(short, long)]
    interactive: bool,
    
    /// Force non-interactive mode
    #[arg(long)]
    non_interactive: bool,
    
    /// Execute script file
    #[arg(short = 'c', long)]
    command: Option<String>,
    
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,
    
    /// Configuration file path
    #[arg(long)]
    config: Option<String>,
    
    /// Theme name
    #[arg(long)]
    theme: Option<String>,
    
    /// Script file to execute
    script_file: Option<String>,
    
    /// Arguments to pass to script
    script_args: Vec<String>,
}

#[cfg(not(feature = "cli-args"))]
fn parse_simple_args() -> (bool, bool, Option<String>, bool, Option<String>) {
    let args: Vec<String> = std::env::args().collect();
    let mut busybox = false;
    let mut interactive = false;
    let mut command = None;
    let mut debug = false;
    let mut script_file = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--busybox" => busybox = true,
            "-i" | "--interactive" => interactive = true,
            "-c" => {
                if i + 1 < args.len() {
                    command = Some(args[i + 1].clone());
                    i += 1;
                }
            },
            "-d" | "--debug" => debug = true,
            arg if !arg.starts_with("-") => {
                script_file = Some(arg.to_string());
                break;
            },
            _ => {} // Ignore unknown options
        }
        i += 1;
    }
    
    (busybox, interactive, command, debug, script_file)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    // Start startup time measurement
    #[cfg(feature = "startup-profiling")]
    let _profiler = nxsh_core::startup_profiler::StartupProfiler::new();
    
    // Setup crash handler
    #[cfg(feature = "crash-handler")]
    nxsh_core::crash_handler::setup_crash_handler();
    
    // Early BusyBox mode detection
    if is_busybox_invocation() {
        busybox_mode();
    }
    
    // Parse CLI arguments
    #[cfg(feature = "cli-args")]
    let args = CliArgs::parse();
    
    #[cfg(not(feature = "cli-args"))]
    let (busybox, interactive, command, debug, script_file) = parse_simple_args();
    
    #[cfg(feature = "cli-args")]
    let (busybox, interactive, command, debug, script_file) = (
        args.busybox,
        args.interactive,
        args.command,
        args.debug,
        args.script_file
    );
    
    // BusyBox mode
    if busybox {
        busybox_mode();
    }
    
    // Setup debug mode
    if debug {
        std::env::set_var("RUST_LOG", "debug");
        #[cfg(feature = "logging")]
        nxsh_core::logging::init_logger()?;
    }
    
    // Load configuration - use simplified approach for now
    let config = nxsh_core::Config::default();
    
    // Initialize UI system
    #[cfg(feature = "ui")]
    let mut ui = nxsh_ui::SimpleUiController::new()?;
    
    // Initialize core system - use simplified shell state for now
    let mut shell_state = nxsh_core::ShellState::new(config.clone())?;
    
    // Initialize plugin system
    #[cfg(feature = "plugins")]
    let plugin_manager = nxsh_plugin::PluginManager::new()?;
    
    // Initialize parser
    let parser = nxsh_parser::ShellCommandParser::new();
    
    // Output startup time
    let startup_time = start_time.elapsed();
    if debug {
        println!("Startup time: {:?}", startup_time);
    }
    
    // Command execution mode
    if let Some(cmd) = command {
        return run_command(&cmd, &mut shell_state, &parser);
    }
    
    // Script execution mode
    if let Some(script) = script_file {
        return run_script(&script, &mut shell_state, &parser);
    }
    
    // Interactive mode detection - simplified
    let is_interactive = interactive || 
        (!cfg!(feature = "non-interactive-default") && 
         io::stdin().is_terminal() && 
         io::stdout().is_terminal());
    
    if is_interactive {
        // Start interactive mode
        run_interactive_mode(&mut shell_state, &parser, #[cfg(feature = "ui")] &mut ui)
    } else {
        // Non-interactive mode (read commands from stdin)
        run_non_interactive_mode(&mut shell_state, &parser)
    }
}

fn run_command(command: &str, shell_state: &mut nxsh_core::ShellState, parser: &nxsh_parser::ShellCommandParser) -> Result<(), Box<dyn std::error::Error>> {
    // Parse the command line into parts
    let parts: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
    if parts.is_empty() {
        return Ok(());
    }
    
    let command_name = &parts[0];
    let args = &parts[1..];
    
    // Check if it's a built-in command in nxsh_builtins first
    if nxsh_builtins::is_builtin(command_name) {
        let context = nxsh_builtins::BuiltinContext::new();
        match nxsh_builtins::execute_builtin(command_name, args, &context) {
            Ok(exit_code) => {
                if exit_code != 0 {
                    std::process::exit(exit_code);
                }
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    // Fall back to regular parser/AST execution
    let ast = parser.parse(command)?;
    let exit_code = nxsh_core::execute_ast(&ast, shell_state)?;
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

fn run_script(script_path: &str, shell_state: &mut nxsh_core::ShellState, parser: &nxsh_parser::ShellCommandParser) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(script_path)?;
    let ast = parser.parse(&content)?;
    let exit_code = nxsh_core::execute_ast(&ast, shell_state)?;
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

fn run_interactive_mode(
    shell_state: &mut nxsh_core::ShellState, 
    parser: &nxsh_parser::ShellCommandParser,
    #[cfg(feature = "ui")] _ui: &mut nxsh_ui::SimpleUiController
) -> Result<(), Box<dyn std::error::Error>> {
    // Show stylish startup banner
    show_startup_banner();
    println!();
    
    loop {
        // Display stylish prompt
        show_stylish_prompt();
        std::io::stdout().flush()?;
        
        // Read input
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                
                // Handle exit commands
                if input == "exit" || input == "quit" {
                    break;
                }
                
                // Parse and execute commands
                let parts: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
                if !parts.is_empty() {
                    let command_name = &parts[0];
                    let args = &parts[1..];
                    
                    // Check if it's a built-in command in nxsh_builtins first
                    if nxsh_builtins::is_builtin(command_name) {
                        let context = nxsh_builtins::BuiltinContext::new();
                        match nxsh_builtins::execute_builtin(command_name, args, &context) {
                            Ok(exit_code) => {
                                if exit_code != 0 {
                                    eprintln!("Command exited with code {}", exit_code);
                                }
                                continue;
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                continue;
                            }
                        }
                    }
                }
                
                // Fall back to regular parser/AST execution
                match parser.parse(input) {
                    Ok(ast) => {
                        match nxsh_core::execute_ast(&ast, shell_state) {
                            Ok(exit_code) => {
                                if exit_code != 0 {
                                    eprintln!("Command exited with code {}", exit_code);
                                }
                            },
                            Err(e) => {
                                eprintln!("Error: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Parse error: {}", e);
                    }
                }
            },
            Err(e) => {
                eprintln!("Input error: {}", e);
                break;
            }
        }
    }
    
    println!("Exiting NexusShell.");
    Ok(())
}

fn run_non_interactive_mode(shell_state: &mut nxsh_core::ShellState, parser: &nxsh_parser::ShellCommandParser) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;
    
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    
    let ast = parser.parse(&input)?;
    let exit_code = nxsh_core::execute_ast(&ast, shell_state)?;
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_busybox_detection() {
        // Environment variable test
        std::env::set_var("NXSH_BUSYBOX", "1");
        assert!(is_busybox_invocation());
        std::env::remove_var("NXSH_BUSYBOX");
    }

    #[test]
    fn test_version_constant() {
        assert!(!VERSION.is_empty());
    }
}
