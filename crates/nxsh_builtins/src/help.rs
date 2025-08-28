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
    println!("{cyan}╔══════════════════════════════════════════════════════════════════════════════╗{reset}");
    println!("{cyan}║{purple}                    🚀 NEXUSSHELL COMPLETE COMMAND SUITE 🚀                   {cyan}║{reset}");
    println!("{cyan}╚══════════════════════════════════════════════════════════════════════════════╝{reset}");
    println!();

    // File Operations
    println!("{purple}📂 FILE OPERATIONS & MANAGEMENT{reset}");
    println!("  {yellow}ls{reset}        - 📋 List directory contents with style");
    println!("  {yellow}pwd{reset}       - 📍 Show current working directory");
    println!("  {yellow}cd{reset}        - 🔄 Change directory intelligently");
    println!("  {yellow}touch{reset}     - ✨ Create/update file timestamps");
    println!("  {yellow}mkdir{reset}     - 📁 Create directories recursively");
    println!("  {yellow}cp{reset}        - 📄 Copy files and directories");
    println!("  {yellow}mv{reset}        - 🔀 Move/rename files and folders");
    println!("  {yellow}rm{reset}        - 🗑️  Remove files and directories");
    println!("  {yellow}ln{reset}        - 🔗 Create symbolic/hard links");
    println!("  {yellow}chmod{reset}     - 🔐 Change file permissions");
    println!("  {yellow}chown{reset}     - 👤 Change file ownership");
    println!("  {yellow}find{reset}      - 🔍 Advanced file search with patterns");
    println!("  {yellow}locate{reset}    - ⚡ Fast file location");
    println!("  {yellow}du{reset}        - 📊 Disk usage analysis");
    println!("  {yellow}df{reset}        - 💿 Filesystem disk space info");
    println!("  {yellow}stat{reset}      - 📋 Detailed file statistics");
    println!();

    // Text Processing
    println!("{coral}💬 TEXT PROCESSING & DATA MANIPULATION{reset}");
    println!("  {yellow}cat{reset}       - 📖 Display file contents beautifully");
    println!("  {yellow}echo{reset}      - 🗨️  Output text with style options");
    println!("  {yellow}head{reset}      - 📄 Display first lines of files");
    println!("  {yellow}tail{reset}      - 📄 Display last lines (with follow)");
    println!("  {yellow}wc{reset}        - 📏 Count lines, words, characters");
    println!("  {yellow}uniq{reset}      - 🎯 Remove or count duplicate lines");
    println!("  {yellow}cut{reset}       - ✂️  Extract columns from text");
    println!("  {yellow}tr{reset}        - 🔄 Translate/transform characters");
    println!("  {yellow}tee{reset}       - 🔀 Split output to file and stdout");
    println!("  {yellow}sed{reset}       - ✏️  Stream editor for filtering");
    println!("  {yellow}awk{reset}       - 🧮 Pattern scanning and processing");
    println!("  {yellow}sort{reset}      - 📊 Sort lines with various options");
    println!("  {yellow}join{reset}      - 🔗 Join lines from two files");
    println!("  {yellow}paste{reset}     - 📋 Merge lines from files");
    println!("  {yellow}split{reset}     - ✂️  Split files into pieces");
    println!("  {yellow}comm{reset}      - 🔍 Compare two sorted files");
    println!("  {yellow}diff{reset}      - 📊 Show differences between files");
    println!("  {yellow}patch{reset}     - 🩹 Apply patches to files");
    println!("  {yellow}grep{reset}      - 🔍 Search text patterns with colors");
    println!("  {yellow}egrep{reset}     - 🔍 Extended regular expressions");
    println!("  {yellow}fgrep{reset}     - 🔍 Fixed string search");
    println!();

    // System Monitoring
    println!("{green}⚙️  SYSTEM MONITORING & PROCESS MANAGEMENT{reset}");
    println!("  {yellow}ps{reset}        - 📋 List running processes");
    println!("  {yellow}top{reset}       - 📊 Real-time process monitor");
    println!("  {yellow}htop{reset}      - 🌈 Enhanced interactive monitor");
    println!("  {yellow}kill{reset}      - ⚡ Terminate processes by PID");
    println!("  {yellow}killall{reset}   - ⚡ Kill processes by name");
    println!("  {yellow}pgrep{reset}     - 🔍 Find processes by pattern");
    println!("  {yellow}pkill{reset}     - ⚡ Kill processes by pattern");
    println!("  {yellow}jobs{reset}      - 💼 Display active jobs");
    println!("  {yellow}bg{reset}        - 🔙 Put jobs in background");
    println!("  {yellow}fg{reset}        - 🔜 Bring jobs to foreground");
    println!("  {yellow}nohup{reset}     - 🛡️  Run commands persistently");
    println!("  {yellow}disown{reset}    - 🚫 Remove jobs from table");
    println!("  {yellow}free{reset}      - 💾 Display memory usage");
    println!("  {yellow}uptime{reset}    - ⏰ Show system uptime and load");
    println!("  {yellow}uname{reset}     - 💻 System information display");
    println!("  {yellow}whoami{reset}    - 👤 Current username");
    println!("  {yellow}who{reset}       - 👥 Show logged-in users");
    println!("  {yellow}id{reset}        - 🆔 User and group IDs");
    println!("  {yellow}groups{reset}    - 👥 Show user groups");
    println!();

    // Network Tools
    println!("{blue}🌐 NETWORK TOOLS & CONNECTIVITY{reset}");
    println!("  {yellow}ping{reset}      - 🏓 Test network connectivity");
    println!("  {yellow}curl{reset}      - 🌐 HTTP/HTTPS client tool");
    println!("  {yellow}wget{reset}      - ⬇️  Download files from web");
    println!("  {yellow}nc{reset}        - 🔌 Network swiss army knife");
    println!("  {yellow}netcat{reset}    - 🔌 Advanced network utility");
    println!("  {yellow}ssh{reset}       - 🔐 Secure shell connection");
    println!("  {yellow}scp{reset}       - 📁 Secure file copy");
    println!("  {yellow}rsync{reset}     - 🔄 Efficient file synchronization");
    println!("  {yellow}ftp{reset}       - 📁 File transfer protocol");
    println!("  {yellow}telnet{reset}    - 📞 Remote terminal access");
    println!("  {yellow}host{reset}      - 🌐 DNS lookup utility");
    println!("  {yellow}nslookup{reset}  - 🌐 Interactive DNS lookup");
    println!("  {yellow}dig{reset}       - 🌐 Advanced DNS lookup");
    println!("  {yellow}traceroute{reset} - 🗺️  Trace network route");
    println!("  {yellow}netstat{reset}   - 🌐 Network statistics");
    println!("  {yellow}ss{reset}        - 🌐 Socket statistics");
    println!();

    // Archive & Compression
    println!("{orange}📦 ARCHIVE & COMPRESSION TOOLS{reset}");
    println!("  {yellow}tar{reset}       - 📦 Create/extract tape archives");
    println!("  {yellow}zip{reset}       - 📁 Create ZIP archives");
    println!("  {yellow}unzip{reset}     - 📂 Extract ZIP archives");
    println!("  {yellow}gzip{reset}      - 🗜️  GZIP compression");
    println!("  {yellow}gunzip{reset}    - 📂 GZIP decompression");
    println!("  {yellow}xz{reset}        - 🗜️  XZ compression (high ratio)");
    println!("  {yellow}unxz{reset}      - 📂 XZ decompression");
    println!("  {yellow}zstd{reset}      - ⚡ Zstandard compression (fast)");
    println!("  {yellow}unzstd{reset}    - 📂 Zstandard decompression");
    println!("  {yellow}bzip2{reset}     - 🗜️  BZIP2 compression");
    println!("  {yellow}bunzip2{reset}   - 📂 BZIP2 decompression");
    println!("  {yellow}7z{reset}        - 📁 7-Zip archive utility");
    println!();

    // Shell Features
    println!("{pink}🔧 SHELL FEATURES & ENVIRONMENT{reset}");
    println!("  {yellow}alias{reset}     - 🔗 Create command shortcuts");
    println!("  {yellow}unalias{reset}   - 🚫 Remove command aliases");
    println!("  {yellow}history{reset}   - 📚 Command history management");
    println!("  {yellow}export{reset}    - 🔄 Set environment variables");
    println!("  {yellow}unset{reset}     - 🗑️  Remove variables");
    println!("  {yellow}env{reset}       - 🌍 Show/modify environment");
    println!("  {yellow}set{reset}       - ⚙️  Set shell options");
    println!("  {yellow}declare{reset}   - 📋 Declare variables/functions");
    println!("  {yellow}which{reset}     - 🔍 Locate command files");
    println!("  {yellow}type{reset}      - 🔍 Show command type");
    println!("  {yellow}builtin{reset}   - 🏠 Execute builtin commands");
    println!();

    // Utilities
    println!("{lime}🛠️  SYSTEM UTILITIES & TOOLS{reset}");
    println!("  {yellow}sleep{reset}     - 😴 Pause for specified time");
    println!("  {yellow}timeout{reset}   - ⏲️  Run command with timeout");
    println!("  {yellow}yes{reset}       - ♻️  Repeat string infinitely");
    println!("  {yellow}seq{reset}       - 🔢 Generate number sequences");
    println!("  {yellow}date{reset}      - 📅 Display/set system date");
    println!("  {yellow}cal{reset}       - 📅 Display calendar");
    println!("  {yellow}bc{reset}        - 🧮 Command-line calculator");
    println!("  {yellow}expr{reset}      - 🧮 Evaluate expressions");
    println!("  {yellow}true{reset}      - ✅ Always return success");
    println!("  {yellow}false{reset}     - ❌ Always return failure");
    println!("  {yellow}test{reset}      - 🧪 Evaluate conditional expressions");
    println!("  {yellow}clear{reset}     - 🧹 Clear terminal screen");
    println!("  {yellow}reset{reset}     - 🔄 Reset terminal to initial state");
    println!();

    println!("{lavender}💡 TIPS:{reset}");
    println!("  • Type {yellow}help <command>{reset} for detailed information");
    println!("  • Use {yellow}Tab{reset} for command completion"); 
    println!("  • Press {yellow}Ctrl+C{reset} to interrupt commands");
    println!("  • Use {yellow}man <command>{reset} for full manual pages");
    println!();
    
    println!("{cyan}🎨 UI Features:{reset}");
    println!("  • {green}Syntax highlighting{reset} for commands");
    println!("  • {blue}Smart completion{reset} with context");
    println!("  • {purple}Beautiful file listings{reset} with icons");
    println!("  • {coral}Colorized output{reset} for readability");
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
            println!("{cyan}📋 ls - Beautiful Directory Listing{reset}");
            println!("{yellow}Usage:{reset} ls [OPTIONS] [PATH...]{reset}");
            println!();
            println!("{green}Options:{reset}");
            println!("  {blue}-l, --long{reset}     Show detailed information");
            println!("  {blue}-a, --all{reset}      Show hidden files");
            println!("  {blue}-h, --human{reset}    Human readable sizes");
            println!("  {blue}-R, --recursive{reset} List subdirectories recursively");
            println!("  {blue}-t, --time{reset}     Sort by modification time");
            println!("  {blue}-S, --size{reset}     Sort by file size");
            println!("  {blue}-r, --reverse{reset}  Reverse sort order");
            println!("  {blue}--color{reset}        Colorize output");
            println!("  {blue}--icons{reset}        Show file type icons");
        }
        
        "cat" => {
            println!("{cyan}📖 cat - Display File Contents{reset}");
            println!("{yellow}Usage:{reset} cat [OPTIONS] [FILE...]{reset}");
            println!();
            println!("{green}Options:{reset}");
            println!("  {blue}-n, --number{reset}   Number all output lines");
            println!("  {blue}-b, --number-nonblank{reset} Number non-empty lines");
            println!("  {blue}-s, --squeeze-blank{reset} Squeeze multiple blank lines");
            println!("  {blue}-v, --show-nonprinting{reset} Show non-printing characters");
            println!("  {blue}-E, --show-ends{reset} Display $ at end of lines");
            println!("  {blue}-T, --show-tabs{reset} Display tabs as ^I");
        }

        "wc" => {
            println!("{cyan}📏 wc - Word, Line, Character Counter{reset}");
            println!("{yellow}Usage:{reset} wc [OPTIONS] [FILE...]{reset}");
            println!();
            println!("{green}Options:{reset}");
            println!("  {blue}-l, --lines{reset}    Count lines");
            println!("  {blue}-w, --words{reset}    Count words");
            println!("  {blue}-c, --chars{reset}    Count characters");
            println!("  {blue}-m, --chars{reset}    Count characters (UTF-8 aware)");
            println!("  {blue}-L, --max-line-length{reset} Show longest line length");
            println!("  {blue}--total{reset}        Show grand total for multiple files");
        }

        "grep" => {
            println!("{cyan}🔍 grep - Pattern Search with Style{reset}");
            println!("{yellow}Usage:{reset} grep [OPTIONS] PATTERN [FILE...]{reset}");
            println!();
            println!("{green}Options:{reset}");
            println!("  {blue}-i, --ignore-case{reset} Case insensitive search");
            println!("  {blue}-v, --invert-match{reset} Invert match (show non-matching)");
            println!("  {blue}-n, --line-number{reset} Show line numbers");
            println!("  {blue}-H, --with-filename{reset} Show filename with matches");
            println!("  {blue}-r, --recursive{reset} Search directories recursively");
            println!("  {blue}-E, --extended-regexp{reset} Extended regular expressions");
            println!("  {blue}-F, --fixed-strings{reset} Fixed string search");
            println!("  {blue}-C, --context=NUM{reset} Show NUM lines of context");
            println!("  {blue}--color=auto{reset}   Colorize matches");
        }

        "tar" => {
            println!("{cyan}📦 tar - Archive Management{reset}");
            println!("{yellow}Usage:{reset} tar [OPTIONS] [FILE...]{reset}");
            println!();
            println!("{green}Main Operations:{reset}");
            println!("  {blue}-c, --create{reset}   Create new archive");
            println!("  {blue}-x, --extract{reset}  Extract from archive");
            println!("  {blue}-t, --list{reset}     List archive contents");
            println!("  {blue}-r, --append{reset}   Append files to archive");
            println!("  {blue}-u, --update{reset}   Update archive with newer files");
            println!();
            println!("{green}Compression:{reset}");
            println!("  {blue}-z, --gzip{reset}     GZIP compression");
            println!("  {blue}-j, --bzip2{reset}    BZIP2 compression");
            println!("  {blue}-J, --xz{reset}       XZ compression");
            println!("  {blue}--zstd{reset}         Zstandard compression");
            println!();
            println!("{green}Common Options:{reset}");
            println!("  {blue}-f, --file={reset}    Archive filename");
            println!("  {blue}-v, --verbose{reset}  Verbose output");
            println!("  {blue}-C, --directory{reset} Change to directory");
        }

        "ps" => {
            println!("{cyan}📋 ps - Process Status{reset}");
            println!("{yellow}Usage:{reset} ps [OPTIONS]{reset}");
            println!();
            println!("{green}Options:{reset}");
            println!("  {blue}-e, --everyone{reset} Show all processes");
            println!("  {blue}-f, --full{reset}     Full format listing");
            println!("  {blue}-l, --long{reset}     Long format");
            println!("  {blue}-u, --user{reset}     User-oriented format");
            println!("  {blue}-x, --no-heading{reset} Show processes without controlling terminal");
            println!("  {blue}--forest{reset}       ASCII art process tree");
            println!("  {blue}--sort={reset}        Sort by specified field");
        }

        "kill" => {
            println!("{cyan}⚡ kill - Terminate Processes{reset}");
            println!("{yellow}Usage:{reset} kill [SIGNAL] PID...{reset}");
            println!();
            println!("{green}Common Signals:{reset}");
            println!("  {blue}TERM (15){reset}      Polite termination request");
            println!("  {blue}KILL (9){reset}       Force immediate termination");
            println!("  {blue}HUP (1){reset}        Hang up (reload config)");
            println!("  {blue}INT (2){reset}        Interrupt (Ctrl+C)");
            println!("  {blue}STOP (19){reset}      Stop (pause) process");
            println!("  {blue}CONT (18){reset}      Continue stopped process");
            println!();
            println!("{green}Examples:{reset}");
            println!("  kill 1234          Send TERM signal to PID 1234");
            println!("  kill -9 1234       Force kill PID 1234");
            println!("  kill -HUP 1234     Send hang-up signal");
        }

        "curl" => {
            println!("{cyan}🌐 curl - HTTP/HTTPS Client{reset}");
            println!("{yellow}Usage:{reset} curl [OPTIONS] URL{reset}");
            println!();
            println!("{green}Common Options:{reset}");
            println!("  {blue}-o, --output{reset}   Write output to file");
            println!("  {blue}-O, --remote-name{reset} Save with remote filename");
            println!("  {blue}-L, --location{reset} Follow redirects");
            println!("  {blue}-i, --include{reset}  Include response headers");
            println!("  {blue}-v, --verbose{reset}  Verbose output");
            println!("  {blue}-s, --silent{reset}   Silent mode");
            println!("  {blue}-X, --request{reset}  HTTP method (GET, POST, etc.)");
            println!("  {blue}-H, --header{reset}   Custom header");
            println!("  {blue}-d, --data{reset}     Send data in POST request");
            println!("  {blue}--json{reset}         Send JSON data");
        }

        "ssh" => {
            println!("{cyan}🔐 ssh - Secure Shell{reset}");
            println!("{yellow}Usage:{reset} ssh [OPTIONS] [user@]hostname [command]{reset}");
            println!();
            println!("{green}Options:{reset}");
            println!("  {blue}-p, --port{reset}     Specify port number");
            println!("  {blue}-i, --identity{reset} Use specific private key");
            println!("  {blue}-L, --local{reset}    Local port forwarding");
            println!("  {blue}-R, --remote{reset}   Remote port forwarding");
            println!("  {blue}-N, --no-command{reset} No remote command");
            println!("  {blue}-f, --fork{reset}     Go to background");
            println!("  {blue}-v, --verbose{reset}  Verbose output");
            println!("  {blue}-A, --forward-agent{reset} Forward authentication agent");
            println!("  {blue}-X, --x11{reset}      Enable X11 forwarding");
        }

        "yes" => {
            println!("{cyan}♻️  yes - Repeat Output{reset}");
            println!("{yellow}Usage:{reset} yes [STRING]{reset}");
            println!();
            println!("{green}Description:{reset}");
            println!("  Outputs STRING (or 'y' by default) repeatedly until killed.");
            println!("  Useful for automating confirmations in scripts.");
            println!();
            println!("{green}Examples:{reset}");
            println!("  yes                Output 'y' infinitely");
            println!("  yes hello          Output 'hello' infinitely");
            println!("  yes | head -5      Output 'y' 5 times");
        }

        "true" => {
            println!("{cyan}✅ true - Success Command{reset}");
            println!("{yellow}Usage:{reset} true{reset}");
            println!();
            println!("{green}Description:{reset}");
            println!("  Always exits with status 0 (success).");
            println!("  Useful in shell scripts for infinite loops and conditional expressions.");
            println!();
            println!("{green}Examples:{reset}");
            println!("  while true; do echo hello; sleep 1; done");
            println!("  if true; then echo 'This always runs'; fi");
        }

        "false" => {
            println!("{cyan}❌ false - Failure Command{reset}");
            println!("{yellow}Usage:{reset} false{reset}");
            println!();
            println!("{green}Description:{reset}");
            println!("  Always exits with status 1 (failure).");
            println!("  Useful in shell scripts for testing and conditional expressions.");
        }

        "uname" => {
            println!("{cyan}💻 uname - System Information{reset}");
            println!("{yellow}Usage:{reset} uname [OPTIONS]{reset}");
            println!();
            println!("{green}Options:{reset}");
            println!("  {blue}-a, --all{reset}      Print all information");
            println!("  {blue}-s, --kernel-name{reset} Print kernel name");
            println!("  {blue}-n, --nodename{reset} Print network node hostname");
            println!("  {blue}-r, --release{reset}  Print kernel release");
            println!("  {blue}-v, --version{reset}  Print kernel version");
            println!("  {blue}-m, --machine{reset}  Print machine hardware name");
            println!("  {blue}-p, --processor{reset} Print processor type");
            println!("  {blue}-o, --operating-system{reset} Print operating system");
        }

        "alias" => {
            println!("{cyan}🔗 alias - Command Shortcuts{reset}");
            println!("{yellow}Usage:{reset} alias [NAME[=VALUE]...]{reset}");
            println!();
            println!("{green}Description:{reset}");
            println!("  Create shortcuts for frequently used commands.");
            println!("  Without arguments, shows all current aliases.");
            println!();
            println!("{green}Examples:{reset}");
            println!("  alias ll='ls -la'     Create 'll' alias");
            println!("  alias grep='grep --color=auto'");
            println!("  alias                 Show all aliases");
        }

        "history" => {
            println!("{cyan}📚 history - Command History{reset}");
            println!("{yellow}Usage:{reset} history [OPTIONS] [N]{reset}");
            println!();
            println!("{green}Options:{reset}");
            println!("  {blue}-c, --clear{reset}    Clear history");
            println!("  {blue}-d, --delete{reset}   Delete specific entry");
            println!("  {blue}-a, --append{reset}   Append to history file");
            println!("  {blue}-r, --read{reset}     Read history file");
            println!("  {blue}-w, --write{reset}    Write history to file");
        }

        _ => {
            // Attempt to delegate to builtin's own --help if available
            let known_simple = [
                "ls","cp","mv","rm","mkdir","rmdir","touch","grep","find","head","tail","wc","cut","tr","uniq",
                "ps","kill","free","uptime","uname","ping","wget","curl","zip","unzip","xz","bzip2","zstd","unzstd",
                "alias","unalias","export","unset","history","which","date","cal","echo","cat","stat","du","df"
            ];
            if known_simple.contains(&command) {
                // Reuse central dispatcher so behavior matches actual command
                if let Err(e) = crate::execute_builtin(command, &["--help".to_string()]) {
                    // Fallback to generic message if command doesn't support --help yet
                    println!("{coral}❓ Command '{yellow}{command}{coral}' - No detailed help available ({e}){reset}");
                }
                return;
            }

            // Generic fallback list
            println!("{coral}❓ Command '{yellow}{command}{coral}' - No detailed help available{reset}");
            println!();
            println!("{green}📚 Available commands with detailed help:{reset}");
            println!();
            println!("{blue}File Operations:{reset} ls, cat, cp, mv, rm, ln, chmod, find, du, df");
            println!("{blue}Text Processing:{reset} grep, wc, head, tail, cut, tr, sed, awk, sort");
            println!("{blue}System Tools:{reset} ps, kill, top, ssh, curl, tar, zip");
            println!("{blue}Shell Features:{reset} alias, history, export, which, true, false");
            println!("{blue}Network:{reset} ping, wget, curl, ssh, scp, netstat");
            println!("{blue}Archives:{reset} tar, zip, unzip, gzip, xz, zstd");
            println!();
            println!("{yellow}💡 Try:{reset} help <command> for specific information");
        }
    }
    println!();
}
