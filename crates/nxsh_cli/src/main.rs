#[cfg(feature = "cli-args")]
use clap::Parser;
use std::io::{self, IsTerminal};
use std::time::Instant;

// Add required imports for enhanced functionality
extern crate chrono;
extern crate whoami;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Show stylish startup banner
fn show_startup_banner() {
    // Cyberpunk color scheme
    let cyan = "\x1b[38;2;0;245;255m"; // #00f5ff
    let purple = "\x1b[38;2;153;69;255m"; // #9945ff
    let coral = "\x1b[38;2;255;71;87m"; // #ff4757
    let green = "\x1b[38;2;46;213;115m"; // #2ed573
    let yellow = "\x1b[38;2;255;190;11m"; // #ffbe0b
    let blue = "\x1b[38;2;116;185;255m"; // #74b9ff
    let bold = "\x1b[1m";
    let reset = "\x1b[0m";

    println!("{bold}{cyan}┌─────────────────────────────────────────────────────────────┐{reset}");
    println!("{cyan}│{reset}        {purple}███╗   ██╗███████╗██╗  ██╗██╗   ██╗███████╗{reset}        {cyan}│{reset}");
    println!("{cyan}│{reset}        {purple}████╗  ██║██╔════╝╚██╗██╔╝██║   ██║██╔════╝{reset}        {cyan}│{reset}");
    println!("{cyan}│{reset}        {purple}██╔██╗ ██║█████╗   ╚███╔╝ ██║   ██║███████╗{reset}        {cyan}│{reset}");
    println!("{cyan}│{reset}        {purple}██║╚██╗██║██╔══╝   ██╔██╗ ██║   ██║╚════██║{reset}        {cyan}│{reset}");
    println!("{cyan}│{reset}        {purple}██║ ╚████║███████╗██╔╝ ██╗╚██████╔╝███████║{reset}        {cyan}│{reset}");
    println!("{cyan}│{reset}        {purple}╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝{reset}        {cyan}│{reset}");
    println!("{cyan}├─────────────────────────────────────────────────────────────┤{reset}");
    println!("{cyan}│{reset}  {coral}🚀 Welcome to NexusShell v{yellow}{VERSION:<3}{coral} - Cyberpunk Edition 🚀{reset}   {cyan}│{reset}");
    println!("{cyan}│{reset}  {green}✨ Modern POSIX-compatible shell with style ✨{reset}             {cyan}│{reset}");
    println!("{cyan}├─────────────────────────────────────────────────────────────┤{reset}");
    println!("{cyan}│{reset}  {blue}💡 Quick Start:{reset}                                            {cyan}│{reset}");
    println!("{cyan}│{reset}    {yellow}• Type 'help' for command overview{reset}                      {cyan}│{reset}");
    println!("{cyan}│{reset}    {yellow}• Try 'echo \"Hello World!\"'{reset}                        {cyan}│{reset}");
    println!("{cyan}│{reset}    {yellow}• Use 'clear --banner' for welcome screen{reset}               {cyan}│{reset}");
    println!("{cyan}│{reset}    {yellow}• Type 'exit' or 'quit' to leave{reset}                        {cyan}│{reset}");
    println!("{cyan}└─────────────────────────────────────────────────────────────┘{reset}");
}

/// Show stylish bash-like prompt with cyberpunk colors
#[allow(dead_code)]
fn show_stylish_prompt() {
    use std::env;
    use std::path::PathBuf;

    // Cyberpunk color scheme
    let cyan = "\x1b[38;2;0;245;255m"; // #00f5ff
    let purple = "\x1b[38;2;153;69;255m"; // #9945ff
    let coral = "\x1b[38;2;255;71;87m"; // #ff4757
    let green = "\x1b[38;2;46;213;115m"; // #2ed573
    let yellow = "\x1b[38;2;255;190;11m"; // #ffbe0b
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
            format!(
                ".../{}/{}",
                components[components.len() - 2]
                    .as_os_str()
                    .to_string_lossy(),
                components[components.len() - 1]
                    .as_os_str()
                    .to_string_lossy()
            )
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
    print!("{bold}{cyan}╭─{reset}[{purple}{username}@{hostname}{purple}{reset}"); // Purple bracket close, reset

    print!(" {coral}📁 {display_path}{reset}");

    // Add git info if available
    if let Some(branch) = git_info {
        print!(" {green}🌿 {branch}{reset}");
    }

    print!("{purple}{reset}]{reset}");

    // Time display
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;

    print!(" {yellow}⏰ {hours:02}:{minutes:02}{reset}");

    println!();
    print!("{bold}{cyan}╰─{coral}❯{reset} ");
}

/// Get git branch information if in a git repository
#[allow(dead_code)]
fn get_git_info() -> Option<String> {
    use std::process::Command;

    // Try to get git branch
    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
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
    if std::env::var("NXSH_BUSYBOX")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return true;
    }
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--busybox") {
        return true;
    }
    if let Some(exe) = std::env::args().next() {
        // invoked name
        let prog = std::path::Path::new(&exe)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if prog == "nxsh-busybox" || prog.starts_with("nxsh-") {
            return true;
        }
    }
    false
}

fn busybox_mode() -> ! {
    let args: Vec<String> = std::env::args().collect();

    // Get first argument (program name or command name)
    let (command, cmd_args) = if args.len() > 1 && !args[1].starts_with("--") {
        (args[1].as_str(), &args[2..])
    } else {
        // Infer command from program name
        if let Some(exe) = args.first() {
            let prog = std::path::Path::new(exe)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if let Some(cmd) = prog.strip_prefix("nxsh-") {
                // Remove "nxsh-" prefix
                (cmd, &args[1..])
            } else if prog == "nxsh" && args.len() > 1 {
                // For nxsh, the actual command is the first argument
                (args[1].as_str(), &args[2..])
            } else {
                (prog, &args[1..])
            }
        } else {
            ("", &args[1..])
        }
    };

    // Prepare argument list is now done above

    // Execute builtin commands using the central dispatcher
    match nxsh_builtins::execute_builtin(command, cmd_args) {
        Ok(exit_code) => {
            std::process::exit(exit_code);
        }
        Err(error_str) => {
            eprintln!("nxsh-busybox: {command}: {error_str}");
            std::process::exit(127);
        }
    }
}

#[allow(dead_code)]
fn print_busybox_help() {
    println!("NexusShell BusyBox v{VERSION}");
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

    /// Execute command string
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

    /// Remaining arguments (treated as a command to execute)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[cfg(not(feature = "cli-args"))]
fn parse_simple_args() -> (bool, bool, Option<String>, bool, Option<String>) {
    let args: Vec<String> = std::env::args().collect();
    let mut busybox = false;
    let mut interactive = false;
    let mut command = None;
    let mut debug = false;
    let script_file = None; // Always None for simple args

    // If we have arguments, they represent a command to execute
    // Format: nxsh.exe command arg1 arg2 ...
    // This should be treated as: -c "command arg1 arg2 ..."
    if args.len() > 1 {
        // Join all arguments after the program name as a single command
        let cmd_parts: Vec<String> = args[1..].to_vec();
        let full_command = cmd_parts.join(" ");
        command = Some(full_command);
        return (busybox, interactive, command, debug, script_file);
    }

    (busybox, interactive, command, debug, script_file)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    // Start startup time measurement
    #[cfg(feature = "startup-profiling")]
    let _profiler = nxsh_core::StartupOptimizer::init(nxsh_core::StartupConfig::default());

    // Setup crash handler
    #[cfg(feature = "crash-handler")]
    let _crash_handler = nxsh_core::crash_handler::CrashHandler::new(
        nxsh_core::crash_handler::CrashHandlerConfig::default(),
    );

    // Early BusyBox mode detection
    if is_busybox_invocation() {
        busybox_mode();
    }

    // Parse CLI arguments
    #[cfg(not(feature = "cli-args"))]
    let (busybox, interactive, command, debug, script_file) = parse_simple_args();

    #[cfg(feature = "cli-args")]
    let (busybox, interactive, command, debug, script_file) = {
        let args = CliArgs::parse();
        let command = if args.command.is_some() {
            args.command
        } else if !args.args.is_empty() {
            // Treat remaining args as a command to execute
            Some(args.args.join(" "))
        } else {
            None
        };
        (
            args.busybox,
            args.interactive,
            command,
            args.debug,
            None::<String>, // No script_file in new structure
        )
    };

    // BusyBox mode
    if busybox {
        busybox_mode();
    }

    // Setup debug mode
    if debug {
        std::env::set_var("RUST_LOG", "debug");
        #[cfg(feature = "logging")]
        let _logger = nxsh_core::LoggingSystem::new(nxsh_core::logging::LoggingConfig::default())?;
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
    let _plugin_manager = nxsh_plugin::PluginManager::new();

    // Initialize parser
    let parser = nxsh_parser::ShellCommandParser::new();

    // Output startup time
    let startup_time = start_time.elapsed();
    if debug {
        println!("Startup time: {startup_time:?}");
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
    let is_interactive = interactive
        || (!cfg!(feature = "non-interactive-default")
            && io::stdin().is_terminal()
            && io::stdout().is_terminal());

    if is_interactive {
        // Start interactive mode
        run_interactive_mode(
            &mut shell_state,
            &parser,
            #[cfg(feature = "ui")]
            &mut ui,
        )
    } else {
        // Non-interactive mode (read commands from stdin)
        run_non_interactive_mode(&mut shell_state, &parser)
    }
}

fn run_command(
    command: &str,
    shell_state: &mut nxsh_core::ShellState,
    parser: &nxsh_parser::ShellCommandParser,
) -> Result<(), Box<dyn std::error::Error>> {
    // If the command contains shell operators/pipelines/redirections, use the full parser path.
    // This prevents mistakenly treating a complex command as a single builtin invocation.
    fn contains_shell_syntax(s: &str) -> bool {
        // A small set of common operators and constructs
        const TOKENS: &[&str] = &[
            "|", "||", "&&", ";", "&", "$(", "`", "<", ">", "<(", ">(", "(", ")", "{", "}", "[",
            "]",
        ];
        TOKENS.iter().any(|t| s.contains(t))
    }

    if contains_shell_syntax(command) {
        // Parse to AST, evaluate through nxsh_core::Shell to capture stdout/stderr
        let ast = parser.parse(command)?;
        let mut shell = nxsh_core::Shell::from_state(shell_state.clone());
        let result = shell.eval_ast(&ast)?;
        // Print captured outputs explicitly
        use std::io::Write;
        if !result.stdout.is_empty() {
            write!(std::io::stdout(), "{}", result.stdout)?;
            std::io::stdout().flush()?;
        }
        if !result.stderr.is_empty() {
            write!(std::io::stderr(), "{}", result.stderr)?;
            std::io::stderr().flush()?;
        }
        *shell_state = shell.into_state();
        if result.exit_code != 0 {
            std::process::exit(result.exit_code);
        }
        return Ok(());
    }

    // Parse the command line into parts
    let parts: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
    if parts.is_empty() {
        return Ok(());
    }

    let command_name = &parts[0];
    let args = &parts[1..];

    // Check if it's a built-in command in nxsh_builtins first
    if nxsh_builtins::is_builtin(command_name) {
        match nxsh_builtins::execute_builtin(command_name, args) {
            Ok(exit_code) => {
                if exit_code != 0 {
                    std::process::exit(exit_code);
                }
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }

    // Fall back to regular parser/AST execution via shell to capture output
    let ast = parser.parse(command)?;
    let mut shell = nxsh_core::Shell::from_state(shell_state.clone());
    let result = shell.eval_ast(&ast)?;
    use std::io::Write;
    if !result.stdout.is_empty() {
        write!(std::io::stdout(), "{}", result.stdout)?;
        std::io::stdout().flush()?;
    }
    if !result.stderr.is_empty() {
        write!(std::io::stderr(), "{}", result.stderr)?;
        std::io::stderr().flush()?;
    }
    *shell_state = shell.into_state();
    if result.exit_code != 0 {
        std::process::exit(result.exit_code);
    }
    Ok(())
}

fn run_script(
    script_path: &str,
    shell_state: &mut nxsh_core::ShellState,
    parser: &nxsh_parser::ShellCommandParser,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(script_path)?;
    let ast = parser.parse(&content)?;
    // Evaluate via shell to capture outputs
    let mut shell = nxsh_core::Shell::from_state(shell_state.clone());
    let result = shell.eval_ast(&ast)?;
    use std::io::Write;
    if !result.stdout.is_empty() {
        write!(std::io::stdout(), "{}", result.stdout)?;
        std::io::stdout().flush()?;
    }
    if !result.stderr.is_empty() {
        write!(std::io::stderr(), "{}", result.stderr)?;
        std::io::stderr().flush()?;
    }
    *shell_state = shell.into_state();
    if result.exit_code != 0 {
        std::process::exit(result.exit_code);
    }
    Ok(())
}

#[cfg(feature = "ui")]
fn run_interactive_mode(
    shell_state: &mut nxsh_core::ShellState,
    parser: &nxsh_parser::ShellCommandParser,
    _ui: &mut nxsh_ui::SimpleUiController,
) -> Result<(), Box<dyn std::error::Error>> {
    // Show stylish startup banner
    show_startup_banner();
    println!();
    // Use enhanced ReadLine with tab completion and syntax highlighting
    let mut rl = nxsh_ui::readline::ReadLine::new()?;

    loop {
        let prompt = get_enhanced_prompt();
        let input_line = rl.read_line(&prompt)?; // Handles Tab, arrows, highlight
        let input = input_line.trim();

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

            // Prefer built-ins
            if nxsh_builtins::is_builtin(command_name) {
                match nxsh_builtins::execute_builtin(command_name, args) {
                    Ok(exit_code) => {
                        if exit_code != 0 {
                            eprintln!("Command exited with code {exit_code}");
                        }
                        continue;
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        continue;
                    }
                }
            }
        }

        // Fall back to regular parser/AST execution via shell to capture outputs
        match parser.parse(input) {
            Ok(ast) => {
                let mut shell = nxsh_core::Shell::from_state(shell_state.clone());
                match shell.eval_ast(&ast) {
                    Ok(result) => {
                        use std::io::Write;
                        if !result.stdout.is_empty() {
                            write!(std::io::stdout(), "{}", result.stdout)?;
                            std::io::stdout().flush()?;
                        }
                        if !result.stderr.is_empty() {
                            write!(std::io::stderr(), "{}", result.stderr)?;
                            std::io::stderr().flush()?;
                        }
                        *shell_state = shell.into_state();
                        if result.exit_code != 0 {
                            eprintln!("Command exited with code {}", result.exit_code);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("Parse error: {e}");
            }
        }
    }

    println!("Exiting NexusShell.");
    Ok(())
}

#[cfg(not(feature = "ui"))]
fn run_interactive_mode(
    shell_state: &mut nxsh_core::ShellState,
    parser: &nxsh_parser::ShellCommandParser,
) -> Result<(), Box<dyn std::error::Error>> {
    // Minimal fallback interactive loop without advanced UI
    println!("NexusShell (UI disabled). Type 'exit' to quit.");
    let mut line = String::new();
    loop {
        use std::io::Write;
        print!("nxsh$ ");
        std::io::stdout().flush()?;
        line.clear();
        if std::io::stdin().read_line(&mut line)? == 0 {
            break;
        }
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if input == "exit" || input == "quit" {
            break;
        }

        let parts: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
        if !parts.is_empty() {
            let command_name = &parts[0];
            let args = &parts[1..];
            if nxsh_builtins::is_builtin(command_name) {
                match nxsh_builtins::execute_builtin(command_name, args) {
                    Ok(code) if code == 0 => {}
                    Ok(code) => eprintln!("Command exited with code {code}"),
                    Err(e) => eprintln!("Error: {e}"),
                }
                continue;
            }
        }
        match parser.parse(input) {
            Ok(ast) => {
                let mut shell = nxsh_core::Shell::from_state(shell_state.clone());
                match shell.eval_ast(&ast) {
                    Ok(result) => {
                        use std::io::Write;
                        if !result.stdout.is_empty() {
                            write!(std::io::stdout(), "{}", result.stdout)?;
                            std::io::stdout().flush()?;
                        }
                        if !result.stderr.is_empty() {
                            write!(std::io::stderr(), "{}", result.stderr)?;
                            std::io::stderr().flush()?;
                        }
                        *shell_state = shell.into_state();
                        if result.exit_code != 0 {
                            eprintln!("Command exited with code {}", result.exit_code);
                        }
                    }
                    Err(e) => eprintln!("Error: {e}"),
                }
            }
            Err(e) => eprintln!("Parse error: {e}"),
        }
    }
    println!("Exiting NexusShell.");
    Ok(())
}

/// Generate enhanced prompt for ReadLine
fn get_enhanced_prompt() -> String {
    use std::env;
    use std::path::PathBuf;
    // Emergency fallback: simple single-line prompt for terminals that have redraw issues
    if env::var("NXSH_SIMPLE_PROMPT")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return "$ ".to_string();
    }

    // Cyberpunk color scheme
    let cyan = "\x1b[38;2;0;245;255m"; // #00f5ff
    let purple = "\x1b[38;2;153;69;255m"; // #9945ff
    let coral = "\x1b[38;2;255;71;87m"; // #ff4757
    let green = "\x1b[38;2;46;213;115m"; // #2ed573
    let yellow = "\x1b[38;2;255;190;11m"; // #ffbe0b
    let bold = "\x1b[1m";
    let reset = "\x1b[0m";

    // Get username
    let username = whoami::username();

    // Get hostname (simplified)
    let hostname = env::var("COMPUTERNAME")
        .or_else(|_| env::var("HOSTNAME"))
        .unwrap_or_else(|_| "localhost".to_string());

    // Get current directory
    let current_dir = env::current_dir()
        .map(|path| {
            if let Ok(home) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
                let home_path = PathBuf::from(home);
                if let Ok(relative) = path.strip_prefix(&home_path) {
                    format!("~/{}", relative.display())
                } else {
                    path.display().to_string()
                }
            } else {
                path.display().to_string()
            }
        })
        .unwrap_or_else(|_| "?".to_string());

    // Get git branch if in git repository
    let git_branch = get_git_branch();
    let git_display = if let Some(branch) = git_branch {
        format!(" {yellow}🌿 {branch}{reset}")
    } else {
        String::new()
    };

    // Get current time
    let now = chrono::Local::now();
    let time_str = now.format("%H:%M").to_string();

    // Create multi-line prompt
    format!("{bold}{cyan}╭─[{green}{username}{reset}{cyan}@{purple}{hostname}{reset} {coral}📁 {}{green}{git_display}{cyan}] {yellow}⏰ {time_str}{reset}\n{cyan}╰─❯{reset} ", 
        abbreviate_path(&current_dir))
}

/// Abbreviate long paths for display
fn abbreviate_path(path: &str) -> String {
    let max_length = 50;
    if path.len() <= max_length {
        path.to_string()
    } else {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() > 3 {
            format!(".../{}", parts[parts.len() - 2..].join("/"))
        } else {
            format!("...{}", &path[path.len() - max_length + 3..])
        }
    }
}

/// Get current git branch
fn get_git_branch() -> Option<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout);
            let branch = branch.trim();
            if !branch.is_empty() && branch != "HEAD" {
                return Some(branch.to_string());
            }
        }
    }
    None
}

fn run_non_interactive_mode(
    shell_state: &mut nxsh_core::ShellState,
    parser: &nxsh_parser::ShellCommandParser,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    // Process each line as a separate command
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse the command line into parts
        let parts: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();
        if parts.is_empty() {
            continue;
        }

        let command_name = &parts[0];
        let args = &parts[1..];

        // Check if it's a built-in command in nxsh_builtins first
        if nxsh_builtins::is_builtin(command_name) {
            match nxsh_builtins::execute_builtin(command_name, args) {
                Ok(exit_code) => {
                    if exit_code != 0 {
                        eprintln!("Command exited with code {exit_code}");
                    }
                    continue;
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    continue;
                }
            }
        }

        // Fall back to regular parser/AST execution via shell to capture outputs
        match parser.parse(line) {
            Ok(ast) => {
                let mut shell = nxsh_core::Shell::from_state(shell_state.clone());
                match shell.eval_ast(&ast) {
                    Ok(result) => {
                        use std::io::Write;
                        if !result.stdout.is_empty() {
                            write!(std::io::stdout(), "{}", result.stdout)?;
                            std::io::stdout().flush()?;
                        }
                        if !result.stderr.is_empty() {
                            write!(std::io::stderr(), "{}", result.stderr)?;
                            std::io::stderr().flush()?;
                        }
                        *shell_state = shell.into_state();
                        if result.exit_code != 0 {
                            eprintln!("Command exited with code {}", result.exit_code);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("Parse error: {e}");
            }
        }
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
        assert!(!VERSION.trim().is_empty());
    }
}
