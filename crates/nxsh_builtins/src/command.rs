//! `command` builtin  EPATH search & type query
//! Supports a subset of Bash `command` options:
//!   -v name … print the location/type of each name (concise)
//!   -V name … verbose output describing how each name would be interpreted
//!   Without -v/-V and with a command, this builtin executes the external
//!   command, bypassing shell aliases and functions.

use anyhow::{anyhow, Result};
use std::{env, path::{Path, PathBuf}};
use nxsh_core::context::ShellContext;
use std::process::Command as PCommand;

pub const BUILTIN_NAMES: &[&str] = &[
    // Shell built-ins and flow control
    "alias", "bg", "bind", "break", "builtin", "cd", "command", "continue", "disown", "echo", "eval", "exec", "exit", 
    "getopts", "hash", "let", "local", "pushd", "popd", "pwd", "read", "readonly", "return", "shift", "source", 
    "suspend", "times", "trap", "type", "ulimit", "umask", "unalias", "unset", "declare", "export", "set", "vars",
    "case", "for", "if", "while", "until", "function", "select", "test",

    // File operations
    "cp", "mv", "rm", "mkdir", "rmdir", "ln", "ls", "stat", "touch", "tree", "du", "df", "sync", "mount", "umount", 
    "chmod", "chown", "chgrp", "shred", "split", "cat", "more", "less", "head", "tail", "find", "locate",

    // Text processing
    "awk", "sed", "grep", "egrep", "fgrep", "cut", "sort", "uniq", "tr", "wc", "diff", "comm", "join", "paste", 
    "expand", "unexpand", "fmt", "fold", "nl", "pr", "rev", "strings", "expr", "bc", "dc",

    // Compression and archives
    "gzip", "gunzip", "bzip2", "bunzip2", "xz", "unxz", "zip", "unzip", "tar", "cpio", "ar", "zstd", "unzstd", "7z",

    // Network tools
    "ping", "wget", "curl", "ssh", "scp", "ftp", "telnet", "netcat", "nc", "dig", "nslookup", "host", "arp", 
    "netstat", "ss", "ifconfig", "ip", "route", "rsync",

    // System information
    "ps", "top", "htop", "free", "uptime", "uname", "whoami", "who", "id", "groups", "hostname", "history", 
    "jobs", "kill", "killall", "pgrep", "pkill", "nice", "renice", "ionice", "nohup", "timeout", "env", "printenv",

    // Hardware and system management  
    "lscpu", "lsblk", "lsusb", "lspci", "lsof", "dmidecode", "hwclock", "timedatectl", "fsck", 
    "mkfs", "fdisk", "blkid", "smartctl", "hdparm", "iostat", "iotop", "vmstat", "sar", "dstat", "strace", "ltrace",

    // Security and permissions
    "sudo", "su", "passwd", "getfacl", "setfacl", "visudo",

    // Date and time
    "date", "cal", "at", "cron", "crontab", "tzselect", "timer",

    // Hash and checksums
    "md5sum", "sha1sum", "sha256sum", "cksum", "base64",

    // Other utilities
    "sleep", "yes", "false", "true", "seq", "od", "hexdump", "xxd", "tee", "xargs", "watch", "help", "info", "man",
    "update", "package", "complete",

    // Development tools (basic)
    "make", "patch"
];

/// Entry function.
pub fn command_cli(args: &[String], ctx: &ShellContext) -> Result<()> {
    if args.is_empty() { return print_help(); }

    let mut verbose = false;
    let mut list_only = false;
    let mut use_default_path = false;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-v" => list_only = true,
            "-V" => {
                verbose = true;
                list_only = true;
            }
            "-p" => {
                // POSIX -p: use default PATH instead of current PATH
                use_default_path = true;
            }
            "-h" | "--help" => return print_help(),
            "--version" => return print_version(),
            arg if arg.starts_with('-') => {
                return Err(anyhow!("command: unknown option '{}'", arg));
            }
            _ => {
                // First non-option
                let mut names = vec![arg.clone()];
                names.extend(iter.cloned());
                if list_only {
                    return handle_query(names, ctx, list_only, verbose);
                } else {
                    return execute_direct(&names, use_default_path);
                }
            }
        }
    }
    
    // If only options were provided without command names
    if list_only {
        return Err(anyhow!("command: missing command name(s) for listing"));
    }
    
    Ok(())
}

fn print_help() -> Result<()> {
    println!("command - execute commands bypassing shell functions and aliases");
    println!("Usage: command [OPTIONS] COMMAND [ARGS...]");
    println!("       command [OPTIONS] -v|-V COMMAND...");
    println!();
    println!("Options:");
    println!("  -v            Print description of COMMAND (similar to 'type')");
    println!("  -V            More verbose description of COMMAND");
    println!("  -p            Use default PATH, ignoring current PATH");
    println!("  -h, --help    Show this help");
    println!("  --version     Show version information");
    println!();
    println!("Execute COMMAND with ARGS, bypassing shell functions and aliases.");
    println!("With -v or -V, show what COMMAND would resolve to.");
    Ok(())
}

fn print_version() -> Result<()> {
    println!("command (NexusShell) 1.0.0");
    Ok(())
}

fn handle_query(names: Vec<String>, ctx: &ShellContext, _list_only: bool, verbose: bool) -> Result<()> {
    for name in names {
        if let Some(alias) = ctx.get_alias(&name) {
            if verbose {
                println!("{name} is an alias for '{alias}'");
            } else {
                println!("{alias}");
            }
            continue;
        }
        if BUILTIN_NAMES.contains(&name.as_str()) {
            if verbose {
                println!("{name} is a shell builtin");
            } else {
                println!("{name}");
            }
            continue;
        }
        match lookup_path(&name) {
            Some(path) => {
                if verbose {
                    println!("{} is {}", name, path.display());
                } else {
                    println!("{}", path.display());
                }
            }
            None => {
                // Not found
                if verbose {
                    println!("{name} not found");
                }
            }
        }
    }
    Ok(())
}

fn execute_direct(words: &[String], use_default_path: bool) -> Result<()> {
    if words.is_empty() { return Ok(()); }
    let cmd = &words[0];
    let args = &words[1..];
    
    // Direct PATH lookup, with optional default PATH
    let path = if use_default_path {
        lookup_default_path(cmd)
    } else {
        lookup_path(cmd)
    }.unwrap_or_else(|| PathBuf::from(cmd));
    
    let status = PCommand::new(path).args(args).status()?;
    if !status.success() {
        return Err(anyhow!("command: '{}' exited with status {}", cmd, status.code().unwrap_or(-1)));
    }
    Ok(())
}

fn lookup_default_path(cmd: &str) -> Option<PathBuf> {
    // POSIX default PATH
    let default_path = if cfg!(windows) {
        "C:\\Windows\\System32;C:\\Windows;C:\\Windows\\System32\\Wbem"
    } else {
        "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
    };
    
    for dir in env::split_paths(default_path) {
        let p = dir.join(cmd);
        if p.is_file() && is_executable(&p) {
            return Some(p);
        }
        
        // On Windows, also try with common extensions
        #[cfg(windows)]
        {
            for ext in ["exe", "bat", "cmd"] {
                let p_ext = p.with_extension(ext);
                if p_ext.is_file() {
                    return Some(p_ext);
                }
            }
        }
    }
    None
}

fn lookup_path(cmd: &str) -> Option<PathBuf> {
    let path_var = env::var("PATH").unwrap_or_default();
    for dir in env::split_paths(&path_var) {
        let p = dir.join(cmd);
        if p.is_file() && is_executable(&p) {
            return Some(p);
        }
        
        // On Windows, also try with common extensions if not already specified
        #[cfg(windows)]
        {
            if !cmd.contains('.') {
                for ext in ["exe", "bat", "cmd"] {
                    let p_ext = p.with_extension(ext);
                    if p_ext.is_file() {
                        return Some(p_ext);
                    }
                }
            }
        }
    }
    None
}

#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
}

#[cfg(windows)]
fn is_executable(p: &Path) -> bool {
    p.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "exe" | "bat" | "cmd"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nxsh_core::context::ShellContext;

    #[test]
    fn alias_lookup() {
        let ctx = ShellContext::new();
        ctx.set_alias("ll", "ls -l").unwrap();
        command_cli(&["-v".into(), "ll".into()], &ctx).unwrap();
    }

    #[test]
    fn test_builtin_names_coverage() {
        // Test that common shell builtins are included
        assert!(BUILTIN_NAMES.contains(&"echo"));
        assert!(BUILTIN_NAMES.contains(&"cd"));
        assert!(BUILTIN_NAMES.contains(&"pwd"));
        assert!(BUILTIN_NAMES.contains(&"ls"));
        assert!(BUILTIN_NAMES.contains(&"cat"));
        assert!(BUILTIN_NAMES.contains(&"grep"));
        
        // Test newly added builtins
        assert!(BUILTIN_NAMES.contains(&"awk"));
        assert!(BUILTIN_NAMES.contains(&"sed"));
        assert!(BUILTIN_NAMES.contains(&"date"));
        assert!(BUILTIN_NAMES.contains(&"curl"));
        assert!(BUILTIN_NAMES.contains(&"wget"));
        assert!(BUILTIN_NAMES.contains(&"tar"));
        assert!(BUILTIN_NAMES.contains(&"gzip"));
        assert!(BUILTIN_NAMES.contains(&"ssh"));
        assert!(BUILTIN_NAMES.contains(&"top"));
        assert!(BUILTIN_NAMES.contains(&"ps"));
        
        // Test compression tools
        assert!(BUILTIN_NAMES.contains(&"bzip2"));
        assert!(BUILTIN_NAMES.contains(&"bunzip2"));
        assert!(BUILTIN_NAMES.contains(&"xz"));
        assert!(BUILTIN_NAMES.contains(&"unxz"));
        assert!(BUILTIN_NAMES.contains(&"zstd"));
        
        // Test network tools
        assert!(BUILTIN_NAMES.contains(&"ping"));
        assert!(BUILTIN_NAMES.contains(&"netstat"));
        assert!(BUILTIN_NAMES.contains(&"dig"));
        
        // Test system tools
        assert!(BUILTIN_NAMES.contains(&"lscpu"));
        assert!(BUILTIN_NAMES.contains(&"free"));
        assert!(BUILTIN_NAMES.contains(&"uptime"));
        
        // Verify we have a reasonable number of commands
        assert!(BUILTIN_NAMES.len() > 100, "Should have over 100 builtin commands");
    }

    #[test] 
    fn test_builtin_command_detection() {
        let ctx = ShellContext::new();
        
        // Test that built-in commands are properly detected
        // This should not error and should recognize these as builtins
        let result = command_cli(&["-V".into(), "echo".into()], &ctx);
        assert!(result.is_ok());
        
        let result = command_cli(&["-V".into(), "grep".into()], &ctx);
        assert!(result.is_ok());
        
        let result = command_cli(&["-V".into(), "awk".into()], &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_builtin_names_no_duplicates() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        let mut duplicates = Vec::new();
        
        for &name in BUILTIN_NAMES {
            if !seen.insert(name) {
                duplicates.push(name);
            }
        }
        
        assert!(duplicates.is_empty(), "Found duplicate builtin names: {:?}", duplicates);
    }

    #[test]
    fn test_builtin_names_sorted_categories() {
        // Verify that we have commands from different categories
        let shell_builtins: Vec<&str> = BUILTIN_NAMES.iter().filter(|&name| {
            matches!(*name, "alias" | "echo" | "cd" | "pwd" | "export" | "set" | "source")
        }).cloned().collect();
        assert!(!shell_builtins.is_empty(), "Should have shell builtin commands");

        let file_ops: Vec<&str> = BUILTIN_NAMES.iter().filter(|&name| {
            matches!(*name, "cp" | "mv" | "rm" | "mkdir" | "ls" | "find" | "chmod")
        }).cloned().collect();
        assert!(!file_ops.is_empty(), "Should have file operation commands");

        let text_processing: Vec<&str> = BUILTIN_NAMES.iter().filter(|&name| {
            matches!(*name, "grep" | "sed" | "awk" | "cut" | "sort" | "uniq" | "tr" | "wc")
        }).cloned().collect();
        assert!(!text_processing.is_empty(), "Should have text processing commands");
    }

    #[test]
    fn test_help_and_version() {
        assert!(print_help().is_ok());
        assert!(print_version().is_ok());
    }

    #[test]
    fn test_command_options() {
        let ctx = ShellContext::new();
        
        // Test help option
        let result = command_cli(&["--help".into()], &ctx);
        assert!(result.is_ok());
        
        // Test version option
        let result = command_cli(&["--version".into()], &ctx);
        assert!(result.is_ok());
        
        // Test invalid option
        let result = command_cli(&["--invalid".into()], &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_args() {
        let ctx = ShellContext::new();
        let result = command_cli(&[], &ctx);
        assert!(result.is_ok()); // Should show help
    }

    #[test]
    fn test_path_lookup() {
        // Test that we can look up basic commands
        // This might vary by system but should at least not panic
        let _result = lookup_path("echo");
        let _result = lookup_default_path("echo");
    }
} 
