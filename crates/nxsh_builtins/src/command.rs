use std::env;
use crate::common::{BuiltinResult, BuiltinContext};

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub command_type: CommandType,
    pub path: Option<String>,
    pub description: String,
    pub name: String,
    pub usage: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum CommandType {
    Builtin,
    External,
    Alias,
    Function,
    Keyword,
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub commands: Vec<CommandInfo>,
    pub total_found: usize,
}

impl CommandResult {
    pub fn success(_output: &str) -> Self {
        Self {
            commands: vec![],
            total_found: 0,
        }
    }
    
    pub fn error(_error_msg: &str) -> Self {
        Self {
            commands: vec![],
            total_found: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShellState {
    pub aliases: std::collections::HashMap<String, String>,
    pub functions: std::collections::HashMap<String, String>,
    pub builtins: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub verbose: bool,
    pub print_type: bool,
    pub print_all: bool,
    pub commands: Vec<String>,
}

pub const BUILTIN_NAMES: &[&str] = &[
    "alias", "bg", "bind", "break", "builtin", "case", "cd", "command", "continue",
    "declare", "dirs", "disown", "echo", "enable", "eval", "exec", "exit", "export",
    "false", "fc", "fg", "getopts", "hash", "help", "history", "if", "jobs", "kill",
    "let", "local", "logout", "popd", "printf", "pushd", "pwd", "read", "readonly",
    "return", "set", "shift", "shopt", "source", "suspend", "test", "times", "trap",
    "true", "type", "typeset", "ulimit", "umask", "unalias", "unset", "until", "wait",
    "while", "clear", "ls", "cat", "mv", "cp", "rm", "mkdir", "rmdir", "touch", "find",
    "grep", "awk", "sed", "sort", "uniq", "cut", "tr", "head", "tail", "wc", "diff",
    "file", "stat", "df", "du", "mount", "umount", "ps", "top", "kill", "killall",
    "jobs", "nohup", "which", "whereis", "whatis", "man", "info", "apropos", "locate",
    "updatedb", "find", "xargs", "parallel", "tee", "split", "join", "paste", "fold",
    "fmt", "pr", "nl", "expand", "unexpand", "rev", "shuf", "od", "hexdump", "strings",
    "base64", "uuencode", "uudecode", "compress", "uncompress", "gzip", "gunzip",
    "zcat", "bzip2", "bunzip2", "bzcat", "xz", "unxz", "xzcat", "tar", "zip", "unzip",
    "ar", "objdump", "nm", "size", "strip", "readelf", "objcopy", "addr2line", "ld",
    "as", "gcc", "g++", "make", "cmake", "configure", "autoconf", "automake", "pkg-config",
    "curl", "wget", "rsync", "scp", "ssh", "telnet", "ftp", "sftp", "nc", "nmap", "ping",
    "traceroute", "dig", "nslookup", "host", "arp", "netstat", "ss", "lsof", "iftop",
    "tcpdump", "wireshark", "tshark", "git", "svn", "hg", "cvs", "bzr", "darcs", "fossil",
    "patch", "diff", "comm", "cmp", "colordiff", "vimdiff", "meld", "kdiff3", "xxdiff",
    "date", "cal", "uptime", "who", "w", "users", "last", "lastb", "finger", "id",
    "groups", "whoami", "su", "sudo", "chmod", "chown", "chgrp", "umask", "getfacl",
    "setfacl", "lsattr", "chattr", "visudo", "passwd", "chsh", "chfn", "newgrp",
    "crontab", "at", "batch", "sleep", "usleep", "timeout", "watch", "yes", "seq",
    "shred", "wipe", "srm", "dd", "sync", "fsync", "fdisk", "parted", "gparted",
    "mkfs", "fsck", "mount", "umount", "lsblk", "blkid", "findmnt", "lsusb", "lspci",
    "lscpu", "lsmem", "lshw", "dmidecode", "hdparm", "smartctl", "badblocks", "e2fsck",
    "tune2fs", "resize2fs", "xfs_repair", "xfs_growfs", "btrfs", "zpool", "zfs",
    "screen", "tmux", "byobu", "nohup", "disown", "setsid", "newgrp", "su", "runuser",
    "chroot", "unshare", "nsenter", "systemd-run", "nice", "ionice", "renice", "taskset",
    "cpulimit", "prlimit", "ulimit", "time", "timeout", "strace", "ltrace", "gdb",
    "valgrind", "perf", "top", "htop", "iotop", "iftop", "nethogs", "iperf", "ab",
    "siege", "wrk", "hey", "vegeta", "curl", "httpie", "postman", "insomnia", "newman",
    "yarn", "npm", "pip", "gem", "cargo", "composer", "maven", "gradle", "sbt", "lein",
    "stack", "cabal", "mix", "rebar3", "dub", "nimble", "shards", "pub", "flutter",
    "dotnet", "nuget", "paket", "mono", "mcs", "fsharpc", "vbc", "csc", "ilasm",
    "ildasm", "gacutil", "sn", "al", "tlbimp", "tlbexp", "regasm", "regsvcs", "installutil",
    "mage", "mt", "rc", "mc", "midl", "lib", "link", "dumpbin", "editbin", "cvtres",
    "ml", "ml64", "armasm", "armasm64", "clang", "clang++", "llvm-config", "lldb",
    "opt", "llc", "lli", "llvm-as", "llvm-dis", "llvm-link", "llvm-ar", "llvm-nm",
    "llvm-objdump", "llvm-readobj", "llvm-strip", "llvm-size", "llvm-strings", "llvm-symbolizer",
    "rustc", "rustdoc", "rustfmt", "clippy", "miri", "rls", "rust-analyzer", "bindgen",
    "cbindgen", "wasm-pack", "cargo-audit", "cargo-outdated", "cargo-tree", "cargo-expand",
    "cargo-bloat", "cargo-deps", "cargo-watch", "cargo-edit", "cargo-release", "cargo-make",
    "vim", "nvim", "emacs", "nano", "joe", "pico", "ed", "ex", "vi", "view", "rvim",
    "rview", "vimdiff", "nvim-qt", "gvim", "code", "subl", "atom", "gedit", "kate",
    "kwrite", "mousepad", "leafpad", "pluma", "xed", "geany", "bluefish", "brackets",
    "notepadqq", "retext", "ghostwriter", "typora", "mark", "remarkable", "zettlr",
    "joplin", "notable", "simplenote", "standardnotes", "boostnote", "trilium", "obsidian",
    "roam", "logseq", "athens", "dendron", "foam", "neuron", "emanote", "org-mode",
    "tiddlywiki", "dokuwiki", "mediawiki", "gitiles", "gitea", "gitlab", "github",
    "bitbucket", "sourceforge", "launchpad", "codeberg", "sr.ht", "pagure", "fossil",
    "sourcehut", "cgit", "gitweb", "gitolite", "gitosis", "gitblit", "rhodecode",
    "kallithea", "phabricator", "reviewboard", "gerrit", "crucible", "swarm", "upsource",
];

impl Default for ShellState {
    fn default() -> Self {
        Self {
            aliases: std::collections::HashMap::new(),
            functions: std::collections::HashMap::new(),
            builtins: BUILTIN_NAMES.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

pub fn execute(args: &[String], _context: &BuiltinContext) -> BuiltinResult<i32> {
    let command = parse_args(args)?;
    
    if command.commands.is_empty() {
        return Err("command: missing command name".into());
    }

    let shell_state = ShellState::default();
    let mut results = Vec::new();

    for cmd_name in &command.commands {
        let info = find_command(cmd_name, &shell_state, command.print_all);
        results.extend(info.commands);
    }

    if command.verbose {
        display_verbose_results(&results);
    } else if command.print_type {
        display_type_results(&results);
    } else {
        display_path_results(&results);
    }

    Ok(0)
}

fn parse_args(args: &[String]) -> Result<Command, Box<dyn std::error::Error>> {
    let mut command = Command {
        verbose: false,
        print_type: false,
        print_all: false,
        commands: Vec::new(),
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-v" | "--verbose" => command.verbose = true,
            "-t" | "--type" => command.print_type = true,
            "-a" | "--all" => command.print_all = true,
            arg if arg.starts_with('-') => {
                return Err(format!("command: unknown option: {}", arg).into());
            }
            _ => command.commands.push(args[i].clone()),
        }
        i += 1;
    }

    Ok(command)
}

fn find_command(name: &str, state: &ShellState, find_all: bool) -> CommandResult {
    let mut commands = Vec::new();

    // Check if it's a builtin
    if state.builtins.contains(&name.to_string()) {
        commands.push(CommandInfo {
            command_type: CommandType::Builtin,
            path: None,
            description: format!("{} is a shell builtin", name),
            name: name.to_string(),
            usage: format!("{} [arguments...]", name),
            examples: vec![name.to_string()],
        });
        if !find_all {
            return CommandResult { commands, total_found: 1 };
        }
    }

    // Check if it's an alias
    if let Some(alias_value) = state.aliases.get(name) {
        commands.push(CommandInfo {
            command_type: CommandType::Alias,
            path: None,
            description: format!("{} is aliased to `{}`", name, alias_value),
            name: name.to_string(),
            usage: format!("{} [arguments...]", name),
            examples: vec![alias_value.to_string()],
        });
        if !find_all {
            let total_found = commands.len();
            return CommandResult { commands, total_found };
        }
    }

    // Check if it's a function
    if state.functions.contains_key(name) {
        commands.push(CommandInfo {
            command_type: CommandType::Function,
            path: None,
            description: format!("{} is a function", name),
            name: name.to_string(),
            usage: format!("{} [arguments...]", name),
            examples: vec![name.to_string()],
        });
        if !find_all {
            let total_found = commands.len();
            return CommandResult { commands, total_found };
        }
    }

    // Search in PATH
    if let Some(path) = find_in_path(name) {
        commands.push(CommandInfo {
            command_type: CommandType::External,
            path: Some(path.clone()),
            description: format!("{} is {}", name, path),
            name: name.to_string(),
            usage: format!("{} [arguments...]", name),
            examples: vec![path.clone()],
        });
    }

    CommandResult { 
        total_found: commands.len(),
        commands,
    }
}

fn find_in_path(name: &str) -> Option<String> {
    if let Ok(path_var) = env::var("PATH") {
        let paths = env::split_paths(&path_var);
        
        for path_dir in paths {
            let full_path = path_dir.join(name);
            if full_path.is_file() {
                return Some(full_path.to_string_lossy().to_string());
            }
            
            // On Windows, also check with .exe extension
            #[cfg(windows)]
            {
                let exe_path = path_dir.join(format!("{}.exe", name));
                if exe_path.is_file() {
                    return Some(exe_path.to_string_lossy().to_string());
                }
                
                let cmd_path = path_dir.join(format!("{}.cmd", name));
                if cmd_path.is_file() {
                    return Some(cmd_path.to_string_lossy().to_string());
                }
                
                let bat_path = path_dir.join(format!("{}.bat", name));
                if bat_path.is_file() {
                    return Some(bat_path.to_string_lossy().to_string());
                }
            }
        }
    }
    None
}

fn display_verbose_results(results: &[CommandInfo]) {
    for info in results {
        match &info.command_type {
            CommandType::Builtin => println!("{}", info.description),
            CommandType::Alias => println!("{}", info.description),
            CommandType::Function => println!("{}", info.description),
            CommandType::External => {
                if let Some(path) = &info.path {
                    println!("{}", path);
                }
            }
            CommandType::Keyword => println!("{}", info.description),
        }
    }
}

fn display_type_results(results: &[CommandInfo]) {
    for info in results {
        let type_str = match info.command_type {
            CommandType::Builtin => "builtin",
            CommandType::Alias => "alias",
            CommandType::Function => "function",
            CommandType::External => "file",
            CommandType::Keyword => "keyword",
        };
        println!("{}", type_str);
    }
}

fn display_path_results(results: &[CommandInfo]) {
    for info in results {
        if let Some(path) = &info.path {
            println!("{}", path);
        }
    }
}
