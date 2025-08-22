use std::fmt;

pub struct HelpCommand;

impl fmt::Display for HelpCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "help")
    }
}

// Note: use the common types defined in crate::common
pub fn execute(args: &[String], _context: &crate::common::BuiltinContext) -> Result<i32, crate::common::BuiltinError> {
    if args.is_empty() {
        show_stylish_general_help();
    } else {
        show_stylish_command_help(&args[0]);
    }
    Ok(0)
}

fn show_stylish_general_help() {
    // Beautiful color scheme
    let cyan = "\x1b[38;2;0;245;255m";     // #00f5ff - Bright cyan
    let purple = "\x1b[38;2;153;69;255m";  // #9945ff - Electric purple  
    let coral = "\x1b[38;2;255;71;87m";    // #ff4757 - Coral red
    let green = "\x1b[38;2;46;213;115m";   // #2ed573 - Mint green
    let yellow = "\x1b[38;2;255;190;11m";  // #ffbe0b - Golden yellow
    let blue = "\x1b[38;2;116;185;255m";   // #74b9ff - Sky blue
    let orange = "\x1b[38;2;255;159;67m";  // #ff9f43 - Orange
    let pink = "\x1b[38;2;255;107;129m";   // #ff6b81 - Pink
    let lime = "\x1b[38;2;129;236;236m";   // #81ecec - Lime
    let lavender = "\x1b[38;2;116;125;140m"; // #747d8c - Lavender
    let reset = "\x1b[0m";
    
    println!();
    println!("{}╔══════════════════════════════════════════════════════════════════════════════╗{}", cyan, reset);
    println!("{}║{}                    🚀 NEXUSSHELL COMPLETE COMMAND SUITE 🚀                   {}║{}", cyan, purple, cyan, reset);
    println!("{}╚══════════════════════════════════════════════════════════════════════════════╝{}", cyan, reset);
    println!();

    // File Operations
    println!("{}📂 FILE OPERATIONS & MANAGEMENT{}", purple, reset);
    println!("  {}ls{}        - 📋 List directory contents with style", yellow, reset);
    println!("  {}pwd{}       - 📍 Show current working directory", yellow, reset);
    println!("  {}cd{}        - 🔄 Change directory intelligently", yellow, reset);
    println!("  {}touch{}     - ✨ Create/update file timestamps", yellow, reset);
    println!("  {}mkdir{}     - 📁 Create directories recursively", yellow, reset);
    println!("  {}cp{}        - 📄 Copy files and directories", yellow, reset);
    println!("  {}mv{}        - 🔀 Move/rename files and folders", yellow, reset);
    println!("  {}rm{}        - 🗑️  Remove files and directories", yellow, reset);
    println!("  {}ln{}        - 🔗 Create symbolic/hard links", yellow, reset);
    println!("  {}chmod{}     - 🔐 Change file permissions", yellow, reset);
    println!("  {}chown{}     - 👤 Change file ownership", yellow, reset);
    println!("  {}find{}      - 🔍 Advanced file search with patterns", yellow, reset);
    println!("  {}locate{}    - ⚡ Fast file location", yellow, reset);
    println!("  {}du{}        - 📊 Disk usage analysis", yellow, reset);
    println!("  {}df{}        - 💿 Filesystem disk space info", yellow, reset);
    println!("  {}stat{}      - 📋 Detailed file statistics", yellow, reset);
    println!();

    // Text Processing
    println!("{}💬 TEXT PROCESSING & DATA MANIPULATION{}", coral, reset);
    println!("  {}cat{}       - 📖 Display file contents beautifully", yellow, reset);
    println!("  {}echo{}      - 🗨️  Output text with style options", yellow, reset);
    println!("  {}head{}      - 📄 Display first lines of files", yellow, reset);
    println!("  {}tail{}      - 📄 Display last lines (with follow)", yellow, reset);
    println!("  {}wc{}        - 📏 Count lines, words, characters", yellow, reset);
    println!("  {}uniq{}      - 🎯 Remove or count duplicate lines", yellow, reset);
    println!("  {}cut{}       - ✂️  Extract columns from text", yellow, reset);
    println!("  {}tr{}        - 🔄 Translate/transform characters", yellow, reset);
    println!("  {}tee{}       - 🔀 Split output to file and stdout", yellow, reset);
    println!("  {}sed{}       - ✏️  Stream editor for filtering", yellow, reset);
    println!("  {}awk{}       - 🧮 Pattern scanning and processing", yellow, reset);
    println!("  {}sort{}      - 📊 Sort lines with various options", yellow, reset);
    println!("  {}join{}      - 🔗 Join lines from two files", yellow, reset);
    println!("  {}paste{}     - 📋 Merge lines from files", yellow, reset);
    println!("  {}split{}     - ✂️  Split files into pieces", yellow, reset);
    println!("  {}comm{}      - 🔍 Compare two sorted files", yellow, reset);
    println!("  {}diff{}      - 📊 Show differences between files", yellow, reset);
    println!("  {}patch{}     - 🩹 Apply patches to files", yellow, reset);
    println!("  {}grep{}      - 🔍 Search text patterns with colors", yellow, reset);
    println!("  {}egrep{}     - 🔍 Extended regular expressions", yellow, reset);
    println!("  {}fgrep{}     - 🔍 Fixed string search", yellow, reset);
    println!();

    // System Monitoring
    println!("{}⚙️  SYSTEM MONITORING & PROCESS MANAGEMENT{}", green, reset);
    println!("  {}ps{}        - 📋 List running processes", yellow, reset);
    println!("  {}top{}       - 📊 Real-time process monitor", yellow, reset);
    println!("  {}htop{}      - 🌈 Enhanced interactive monitor", yellow, reset);
    println!("  {}kill{}      - ⚡ Terminate processes by PID", yellow, reset);
    println!("  {}killall{}   - ⚡ Kill processes by name", yellow, reset);
    println!("  {}pgrep{}     - 🔍 Find processes by pattern", yellow, reset);
    println!("  {}pkill{}     - ⚡ Kill processes by pattern", yellow, reset);
    println!("  {}jobs{}      - 💼 Display active jobs", yellow, reset);
    println!("  {}bg{}        - 🔙 Put jobs in background", yellow, reset);
    println!("  {}fg{}        - 🔜 Bring jobs to foreground", yellow, reset);
    println!("  {}nohup{}     - 🛡️  Run commands persistently", yellow, reset);
    println!("  {}disown{}    - 🚫 Remove jobs from table", yellow, reset);
    println!("  {}free{}      - 💾 Display memory usage", yellow, reset);
    println!("  {}uptime{}    - ⏰ Show system uptime and load", yellow, reset);
    println!("  {}uname{}     - 💻 System information display", yellow, reset);
    println!("  {}whoami{}    - 👤 Current username", yellow, reset);
    println!("  {}who{}       - 👥 Show logged-in users", yellow, reset);
    println!("  {}id{}        - 🆔 User and group IDs", yellow, reset);
    println!("  {}groups{}    - 👥 Show user groups", yellow, reset);
    println!();

    // Network Tools
    println!("{}🌐 NETWORK TOOLS & CONNECTIVITY{}", blue, reset);
    println!("  {}ping{}      - 🏓 Test network connectivity", yellow, reset);
    println!("  {}curl{}      - 🌐 HTTP/HTTPS client tool", yellow, reset);
    println!("  {}wget{}      - ⬇️  Download files from web", yellow, reset);
    println!("  {}nc{}        - 🔌 Network swiss army knife", yellow, reset);
    println!("  {}netcat{}    - 🔌 Advanced network utility", yellow, reset);
    println!("  {}ssh{}       - 🔐 Secure shell connection", yellow, reset);
    println!("  {}scp{}       - 📁 Secure file copy", yellow, reset);
    println!("  {}rsync{}     - 🔄 Efficient file synchronization", yellow, reset);
    println!("  {}ftp{}       - 📁 File transfer protocol", yellow, reset);
    println!("  {}telnet{}    - 📞 Remote terminal access", yellow, reset);
    println!("  {}host{}      - 🌐 DNS lookup utility", yellow, reset);
    println!("  {}nslookup{}  - 🌐 Interactive DNS lookup", yellow, reset);
    println!("  {}dig{}       - 🌐 Advanced DNS lookup", yellow, reset);
    println!("  {}traceroute{} - 🗺️  Trace network route", yellow, reset);
    println!("  {}netstat{}   - 🌐 Network statistics", yellow, reset);
    println!("  {}ss{}        - 🌐 Socket statistics", yellow, reset);
    println!();

    // Archive & Compression
    println!("{}📦 ARCHIVE & COMPRESSION TOOLS{}", orange, reset);
    println!("  {}tar{}       - 📦 Create/extract tape archives", yellow, reset);
    println!("  {}zip{}       - 📁 Create ZIP archives", yellow, reset);
    println!("  {}unzip{}     - 📂 Extract ZIP archives", yellow, reset);
    println!("  {}gzip{}      - 🗜️  GZIP compression", yellow, reset);
    println!("  {}gunzip{}    - 📂 GZIP decompression", yellow, reset);
    println!("  {}xz{}        - 🗜️  XZ compression (high ratio)", yellow, reset);
    println!("  {}unxz{}      - 📂 XZ decompression", yellow, reset);
    println!("  {}zstd{}      - ⚡ Zstandard compression (fast)", yellow, reset);
    println!("  {}unzstd{}    - 📂 Zstandard decompression", yellow, reset);
    println!("  {}bzip2{}     - 🗜️  BZIP2 compression", yellow, reset);
    println!("  {}bunzip2{}   - 📂 BZIP2 decompression", yellow, reset);
    println!("  {}7z{}        - 📁 7-Zip archive utility", yellow, reset);
    println!();

    // Shell Features
    println!("{}🔧 SHELL FEATURES & ENVIRONMENT{}", pink, reset);
    println!("  {}alias{}     - 🔗 Create command shortcuts", yellow, reset);
    println!("  {}unalias{}   - 🚫 Remove command aliases", yellow, reset);
    println!("  {}history{}   - 📚 Command history management", yellow, reset);
    println!("  {}export{}    - 🔄 Set environment variables", yellow, reset);
    println!("  {}unset{}     - 🗑️  Remove variables", yellow, reset);
    println!("  {}env{}       - 🌍 Show/modify environment", yellow, reset);
    println!("  {}set{}       - ⚙️  Set shell options", yellow, reset);
    println!("  {}declare{}   - 📋 Declare variables/functions", yellow, reset);
    println!("  {}which{}     - 🔍 Locate command files", yellow, reset);
    println!("  {}type{}      - 🔍 Show command type", yellow, reset);
    println!("  {}builtin{}   - 🏠 Execute builtin commands", yellow, reset);
    println!();

    // Utilities
    println!("{}🛠️  SYSTEM UTILITIES & TOOLS{}", lime, reset);
    println!("  {}sleep{}     - 😴 Pause for specified time", yellow, reset);
    println!("  {}timeout{}   - ⏲️  Run command with timeout", yellow, reset);
    println!("  {}yes{}       - ♻️  Repeat string infinitely", yellow, reset);
    println!("  {}seq{}       - 🔢 Generate number sequences", yellow, reset);
    println!("  {}date{}      - 📅 Display/set system date", yellow, reset);
    println!("  {}cal{}       - 📅 Display calendar", yellow, reset);
    println!("  {}bc{}        - 🧮 Command-line calculator", yellow, reset);
    println!("  {}expr{}      - 🧮 Evaluate expressions", yellow, reset);
    println!("  {}true{}      - ✅ Always return success", yellow, reset);
    println!("  {}false{}     - ❌ Always return failure", yellow, reset);
    println!("  {}test{}      - 🧪 Evaluate conditional expressions", yellow, reset);
    println!("  {}clear{}     - 🧹 Clear terminal screen", yellow, reset);
    println!("  {}reset{}     - 🔄 Reset terminal to initial state", yellow, reset);
    println!();

    println!("{}💡 TIPS:{}", lavender, reset);
    println!("  • Type {}help <command>{} for detailed information", yellow, reset);
    println!("  • Use {}Tab{} for command completion", yellow, reset); 
    println!("  • Press {}Ctrl+C{} to interrupt commands", yellow, reset);
    println!("  • Use {}man <command>{} for full manual pages", yellow, reset);
    println!();
    
    println!("{}🎨 UI Features:{}", cyan, reset);
    println!("  • {}Syntax highlighting{} for commands", green, reset);
    println!("  • {}Smart completion{} with context", blue, reset);
    println!("  • {}Beautiful file listings{} with icons", purple, reset);
    println!("  • {}Colorized output{} for readability", coral, reset);
    println!();
}

fn show_stylish_command_help(command: &str) {
    let cyan = "\x1b[38;2;0;245;255m";
    let purple = "\x1b[38;2;153;69;255m";
    let coral = "\x1b[38;2;255;71;87m";
    let green = "\x1b[38;2;46;213;115m";
    let yellow = "\x1b[38;2;255;190;11m";
    let blue = "\x1b[38;2;116;185;255m";
    let reset = "\x1b[0m";

    match command {
        // File Operations
        "ls" => {
            println!("{}📋 ls - Beautiful Directory Listing{}", cyan, reset);
            println!("{}Usage:{} ls [OPTIONS] [PATH...]{}", yellow, reset, reset);
            println!();
            println!("{}Options:{}", green, reset);
            println!("  {}-l, --long{}     Show detailed information", blue, reset);
            println!("  {}-a, --all{}      Show hidden files", blue, reset);
            println!("  {}-h, --human{}    Human readable sizes", blue, reset);
            println!("  {}-R, --recursive{} List subdirectories recursively", blue, reset);
            println!("  {}-t, --time{}     Sort by modification time", blue, reset);
            println!("  {}-S, --size{}     Sort by file size", blue, reset);
            println!("  {}-r, --reverse{}  Reverse sort order", blue, reset);
            println!("  {}--color{}        Colorize output", blue, reset);
            println!("  {}--icons{}        Show file type icons", blue, reset);
        }
        
        "cat" => {
            println!("{}📖 cat - Display File Contents{}", cyan, reset);
            println!("{}Usage:{} cat [OPTIONS] [FILE...]{}", yellow, reset, reset);
            println!();
            println!("{}Options:{}", green, reset);
            println!("  {}-n, --number{}   Number all output lines", blue, reset);
            println!("  {}-b, --number-nonblank{} Number non-empty lines", blue, reset);
            println!("  {}-s, --squeeze-blank{} Squeeze multiple blank lines", blue, reset);
            println!("  {}-v, --show-nonprinting{} Show non-printing characters", blue, reset);
            println!("  {}-E, --show-ends{} Display $ at end of lines", blue, reset);
            println!("  {}-T, --show-tabs{} Display tabs as ^I", blue, reset);
        }

        "wc" => {
            println!("{}📏 wc - Word, Line, Character Counter{}", cyan, reset);
            println!("{}Usage:{} wc [OPTIONS] [FILE...]{}", yellow, reset, reset);
            println!();
            println!("{}Options:{}", green, reset);
            println!("  {}-l, --lines{}    Count lines", blue, reset);
            println!("  {}-w, --words{}    Count words", blue, reset);
            println!("  {}-c, --chars{}    Count characters", blue, reset);
            println!("  {}-m, --chars{}    Count characters (UTF-8 aware)", blue, reset);
            println!("  {}-L, --max-line-length{} Show longest line length", blue, reset);
            println!("  {}--total{}        Show grand total for multiple files", blue, reset);
        }

        "grep" => {
            println!("{}🔍 grep - Pattern Search with Style{}", cyan, reset);
            println!("{}Usage:{} grep [OPTIONS] PATTERN [FILE...]{}", yellow, reset, reset);
            println!();
            println!("{}Options:{}", green, reset);
            println!("  {}-i, --ignore-case{} Case insensitive search", blue, reset);
            println!("  {}-v, --invert-match{} Invert match (show non-matching)", blue, reset);
            println!("  {}-n, --line-number{} Show line numbers", blue, reset);
            println!("  {}-H, --with-filename{} Show filename with matches", blue, reset);
            println!("  {}-r, --recursive{} Search directories recursively", blue, reset);
            println!("  {}-E, --extended-regexp{} Extended regular expressions", blue, reset);
            println!("  {}-F, --fixed-strings{} Fixed string search", blue, reset);
            println!("  {}-C, --context=NUM{} Show NUM lines of context", blue, reset);
            println!("  {}--color=auto{}   Colorize matches", blue, reset);
        }

        "tar" => {
            println!("{}📦 tar - Archive Management{}", cyan, reset);
            println!("{}Usage:{} tar [OPTIONS] [FILE...]{}", yellow, reset, reset);
            println!();
            println!("{}Main Operations:{}", green, reset);
            println!("  {}-c, --create{}   Create new archive", blue, reset);
            println!("  {}-x, --extract{}  Extract from archive", blue, reset);
            println!("  {}-t, --list{}     List archive contents", blue, reset);
            println!("  {}-r, --append{}   Append files to archive", blue, reset);
            println!("  {}-u, --update{}   Update archive with newer files", blue, reset);
            println!();
            println!("{}Compression:{}", green, reset);
            println!("  {}-z, --gzip{}     GZIP compression", blue, reset);
            println!("  {}-j, --bzip2{}    BZIP2 compression", blue, reset);
            println!("  {}-J, --xz{}       XZ compression", blue, reset);
            println!("  {}--zstd{}         Zstandard compression", blue, reset);
            println!();
            println!("{}Common Options:{}", green, reset);
            println!("  {}-f, --file={}    Archive filename", blue, reset);
            println!("  {}-v, --verbose{}  Verbose output", blue, reset);
            println!("  {}-C, --directory{} Change to directory", blue, reset);
        }

        "ps" => {
            println!("{}📋 ps - Process Status{}", cyan, reset);
            println!("{}Usage:{} ps [OPTIONS]{}", yellow, reset, reset);
            println!();
            println!("{}Options:{}", green, reset);
            println!("  {}-e, --everyone{} Show all processes", blue, reset);
            println!("  {}-f, --full{}     Full format listing", blue, reset);
            println!("  {}-l, --long{}     Long format", blue, reset);
            println!("  {}-u, --user{}     User-oriented format", blue, reset);
            println!("  {}-x, --no-heading{} Show processes without controlling terminal", blue, reset);
            println!("  {}--forest{}       ASCII art process tree", blue, reset);
            println!("  {}--sort={}        Sort by specified field", blue, reset);
        }

        "kill" => {
            println!("{}⚡ kill - Terminate Processes{}", cyan, reset);
            println!("{}Usage:{} kill [SIGNAL] PID...{}", yellow, reset, reset);
            println!();
            println!("{}Common Signals:{}", green, reset);
            println!("  {}TERM (15){}      Polite termination request", blue, reset);
            println!("  {}KILL (9){}       Force immediate termination", blue, reset);
            println!("  {}HUP (1){}        Hang up (reload config)", blue, reset);
            println!("  {}INT (2){}        Interrupt (Ctrl+C)", blue, reset);
            println!("  {}STOP (19){}      Stop (pause) process", blue, reset);
            println!("  {}CONT (18){}      Continue stopped process", blue, reset);
            println!();
            println!("{}Examples:{}", green, reset);
            println!("  kill 1234          Send TERM signal to PID 1234");
            println!("  kill -9 1234       Force kill PID 1234");
            println!("  kill -HUP 1234     Send hang-up signal");
        }

        "curl" => {
            println!("{}🌐 curl - HTTP/HTTPS Client{}", cyan, reset);
            println!("{}Usage:{} curl [OPTIONS] URL{}", yellow, reset, reset);
            println!();
            println!("{}Common Options:{}", green, reset);
            println!("  {}-o, --output{}   Write output to file", blue, reset);
            println!("  {}-O, --remote-name{} Save with remote filename", blue, reset);
            println!("  {}-L, --location{} Follow redirects", blue, reset);
            println!("  {}-i, --include{}  Include response headers", blue, reset);
            println!("  {}-v, --verbose{}  Verbose output", blue, reset);
            println!("  {}-s, --silent{}   Silent mode", blue, reset);
            println!("  {}-X, --request{}  HTTP method (GET, POST, etc.)", blue, reset);
            println!("  {}-H, --header{}   Custom header", blue, reset);
            println!("  {}-d, --data{}     Send data in POST request", blue, reset);
            println!("  {}--json{}         Send JSON data", blue, reset);
        }

        "ssh" => {
            println!("{}🔐 ssh - Secure Shell{}", cyan, reset);
            println!("{}Usage:{} ssh [OPTIONS] [user@]hostname [command]{}", yellow, reset, reset);
            println!();
            println!("{}Options:{}", green, reset);
            println!("  {}-p, --port{}     Specify port number", blue, reset);
            println!("  {}-i, --identity{} Use specific private key", blue, reset);
            println!("  {}-L, --local{}    Local port forwarding", blue, reset);
            println!("  {}-R, --remote{}   Remote port forwarding", blue, reset);
            println!("  {}-N, --no-command{} No remote command", blue, reset);
            println!("  {}-f, --fork{}     Go to background", blue, reset);
            println!("  {}-v, --verbose{}  Verbose output", blue, reset);
            println!("  {}-A, --forward-agent{} Forward authentication agent", blue, reset);
            println!("  {}-X, --x11{}      Enable X11 forwarding", blue, reset);
        }

        "yes" => {
            println!("{}♻️  yes - Repeat Output{}", cyan, reset);
            println!("{}Usage:{} yes [STRING]{}", yellow, reset, reset);
            println!();
            println!("{}Description:{}", green, reset);
            println!("  Outputs STRING (or 'y' by default) repeatedly until killed.");
            println!("  Useful for automating confirmations in scripts.");
            println!();
            println!("{}Examples:{}", green, reset);
            println!("  yes                Output 'y' infinitely");
            println!("  yes hello          Output 'hello' infinitely");
            println!("  yes | head -5      Output 'y' 5 times");
        }

        "true" => {
            println!("{}✅ true - Success Command{}", cyan, reset);
            println!("{}Usage:{} true{}", yellow, reset, reset);
            println!();
            println!("{}Description:{}", green, reset);
            println!("  Always exits with status 0 (success).");
            println!("  Useful in shell scripts for infinite loops and conditional expressions.");
            println!();
            println!("{}Examples:{}", green, reset);
            println!("  while true; do echo hello; sleep 1; done");
            println!("  if true; then echo 'This always runs'; fi");
        }

        "false" => {
            println!("{}❌ false - Failure Command{}", cyan, reset);
            println!("{}Usage:{} false{}", yellow, reset, reset);
            println!();
            println!("{}Description:{}", green, reset);
            println!("  Always exits with status 1 (failure).");
            println!("  Useful in shell scripts for testing and conditional expressions.");
        }

        "uname" => {
            println!("{}💻 uname - System Information{}", cyan, reset);
            println!("{}Usage:{} uname [OPTIONS]{}", yellow, reset, reset);
            println!();
            println!("{}Options:{}", green, reset);
            println!("  {}-a, --all{}      Print all information", blue, reset);
            println!("  {}-s, --kernel-name{} Print kernel name", blue, reset);
            println!("  {}-n, --nodename{} Print network node hostname", blue, reset);
            println!("  {}-r, --release{}  Print kernel release", blue, reset);
            println!("  {}-v, --version{}  Print kernel version", blue, reset);
            println!("  {}-m, --machine{}  Print machine hardware name", blue, reset);
            println!("  {}-p, --processor{} Print processor type", blue, reset);
            println!("  {}-o, --operating-system{} Print operating system", blue, reset);
        }

        "alias" => {
            println!("{}🔗 alias - Command Shortcuts{}", cyan, reset);
            println!("{}Usage:{} alias [NAME[=VALUE]...]{}", yellow, reset, reset);
            println!();
            println!("{}Description:{}", green, reset);
            println!("  Create shortcuts for frequently used commands.");
            println!("  Without arguments, shows all current aliases.");
            println!();
            println!("{}Examples:{}", green, reset);
            println!("  alias ll='ls -la'     Create 'll' alias");
            println!("  alias grep='grep --color=auto'");
            println!("  alias                 Show all aliases");
        }

        "history" => {
            println!("{}📚 history - Command History{}", cyan, reset);
            println!("{}Usage:{} history [OPTIONS] [N]{}", yellow, reset, reset);
            println!();
            println!("{}Options:{}", green, reset);
            println!("  {}-c, --clear{}    Clear history", blue, reset);
            println!("  {}-d, --delete{}   Delete specific entry", blue, reset);
            println!("  {}-a, --append{}   Append to history file", blue, reset);
            println!("  {}-r, --read{}     Read history file", blue, reset);
            println!("  {}-w, --write{}    Write history to file", blue, reset);
        }

        _ => {
            println!("{}❓ Command '{}{}{}' - No detailed help available{}", coral, yellow, command, coral, reset);
            println!();
            println!("{}📚 Available commands with detailed help:{}", green, reset);
            println!();
            println!("{}File Operations:{} ls, cat, cp, mv, rm, ln, chmod, find, du, df", blue, reset);
            println!("{}Text Processing:{} grep, wc, head, tail, cut, tr, sed, awk, sort", blue, reset);
            println!("{}System Tools:{} ps, kill, top, ssh, curl, tar, zip", blue, reset);
            println!("{}Shell Features:{} alias, history, export, which, true, false", blue, reset);
            println!("{}Network:{} ping, wget, curl, ssh, scp, netstat", blue, reset);
            println!("{}Archives:{} tar, zip, unzip, gzip, xz, zstd", blue, reset);
            println!();
            println!("{}💡 Try:{} help <command> for specific information", yellow, reset);
        }
    }
    println!();
}
