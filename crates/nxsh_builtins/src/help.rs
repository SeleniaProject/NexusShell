// Comprehensive help system for NexusShell built-in commands
// This module provides detailed help information for all built-in commands

use anyhow::Result;
use nxsh_core::{Context, ExecutionResult};
use std::collections::HashMap;
use crate::ui_design::{
    TableFormatter, Colorize, Animation, ProgressBar, Notification, NotificationType,
    TableOptions, BorderStyle, Alignment
};

/// Help information for a command
#[derive(Debug, Clone)]
pub struct CommandHelp {
    pub name: String,
    pub summary: String,
    pub description: String,
    pub usage: String,
    pub options: Vec<OptionHelp>,
    pub examples: Vec<ExampleHelp>,
    pub see_also: Vec<String>,
}

/// Help information for a command option
#[derive(Debug, Clone)]
pub struct OptionHelp {
    pub short: Option<String>,
    pub long: String,
    pub description: String,
    pub value_type: Option<String>,
}

/// Help information for an example
#[derive(Debug, Clone)]
pub struct ExampleHelp {
    pub command: String,
    pub description: String,
}

/// Main help CLI entry point
pub fn help_cli(args: &[String]) -> Result<ExecutionResult> {
    // Initialize the help system with all available commands
    let help_map = initialize_help_map();
    
    match args.len() {
        0 => {
            // Show general help with list of available commands
            show_general_help(&help_map);
        }
        1 => {
            let command = &args[0];
            
            // Handle special help options
            match command.as_str() {
                "--search" | "-s" => {
                    eprintln!("Usage: help --search PATTERN");
                    eprintln!("Search for commands containing the pattern");
                    return Ok(ExecutionResult::success(1));
                }
                "--categories" | "-c" => {
                    show_general_help(&help_map);
                    return Ok(ExecutionResult::success(0));
                }
                "--all" | "-a" => {
                    show_all_commands_detailed(&help_map);
                    return Ok(ExecutionResult::success(0));
                }
                _ => {
                    // Show help for specific command
                    if let Some(help) = help_map.get(command) {
                        show_command_help(help);
                    } else {
                        eprintln!("help: no help available for '{command}'");
                        
                        // Try to find similar commands
                        let similar = find_similar_commands(command, &help_map);
                        if !similar.is_empty() {
                            println!();
                            println!("💡 Did you mean:");
                            for similar_cmd in similar {
                                println!("   help {}", similar_cmd);
                            }
                        }
                        
                        show_available_commands(&help_map);
                        return Ok(ExecutionResult::success(1));
                    }
                }
            }
        }
        2 => {
            let option = &args[0];
            let pattern = &args[1];
            
            match option.as_str() {
                "--search" | "-s" => {
                    search_commands(pattern, &help_map);
                }
                _ => {
                    eprintln!("help: invalid option '{option}'");
                    eprintln!("Usage: help [COMMAND] | help --search PATTERN");
                    return Ok(ExecutionResult::success(1));
                }
            }
        }
        _ => {
            eprintln!("help: too many arguments");
            eprintln!("Usage: help [COMMAND] | help --search PATTERN | help --all");
            eprintln!();
            eprintln!("Options:");
            eprintln!("  help              Show categorized command list");
            eprintln!("  help COMMAND      Show detailed help for COMMAND");
            eprintln!("  help --search PATTERN  Search for commands containing PATTERN");
            eprintln!("  help --all        Show brief info for all commands");
            eprintln!("  help --categories Show categorized command list");
            return Ok(ExecutionResult::success(1));
        }
    }
    
    Ok(ExecutionResult::success(0))
}

/// Initialize the comprehensive help map for all built-in commands
fn initialize_help_map() -> HashMap<String, CommandHelp> {
    let mut help_map = HashMap::new();
    
    // ============================================================================
    // CORE COMMANDS - Complete implementations with detailed help
    // ============================================================================
    
    help_map.insert("help".to_string(), CommandHelp {
        name: "help".to_string(),
        summary: "Display help information for commands".to_string(),
        description: "The help command provides comprehensive documentation for NexusShell built-in commands. When called without arguments, it displays a list of available commands. When called with a command name, it shows detailed help for that specific command.".to_string(),
        usage: "help [COMMAND]".to_string(),
        options: vec![],
        examples: vec![
            ExampleHelp {
                command: "help".to_string(),
                description: "Show list of all available commands".to_string(),
            },
            ExampleHelp {
                command: "help ls".to_string(),
                description: "Show detailed help for the ls command".to_string(),
            },
        ],
        see_also: vec!["man".to_string(), "info".to_string()],
    });

    // File and Directory Operations
    help_map.insert("ls".to_string(), CommandHelp {
        name: "ls".to_string(),
        summary: "List directory contents".to_string(),
        description: "List information about files and directories. By default, lists the current directory in a simple format with color coding and icons.".to_string(),
        usage: "ls [OPTION]... [FILE]...".to_string(),
        options: vec![
            OptionHelp { short: Some("-a".to_string()), long: "--all".to_string(), description: "Do not ignore entries starting with .".to_string(), value_type: None },
            OptionHelp { short: Some("-l".to_string()), long: "--long".to_string(), description: "Use a long listing format".to_string(), value_type: None },
            OptionHelp { short: Some("-h".to_string()), long: "--human-readable".to_string(), description: "With -l, print sizes in human readable format".to_string(), value_type: None },
            OptionHelp { short: Some("-t".to_string()), long: "--time".to_string(), description: "Sort by modification time, newest first".to_string(), value_type: None },
            OptionHelp { short: Some("-r".to_string()), long: "--reverse".to_string(), description: "Reverse order while sorting".to_string(), value_type: None },
            OptionHelp { short: Some("-S".to_string()), long: "--size".to_string(), description: "Sort by file size, largest first".to_string(), value_type: None },
            OptionHelp { short: None, long: "--color".to_string(), description: "Colorize the output".to_string(), value_type: Some("WHEN".to_string()) },
        ],
        examples: vec![
            ExampleHelp { command: "ls".to_string(), description: "List files in current directory".to_string() },
            ExampleHelp { command: "ls -la".to_string(), description: "List all files in long format".to_string() },
            ExampleHelp { command: "ls -lh".to_string(), description: "List files with human-readable sizes".to_string() },
            ExampleHelp { command: "ls -lt".to_string(), description: "List files sorted by modification time".to_string() },
        ],
        see_also: vec!["find".to_string(), "stat".to_string(), "du".to_string()],
    });

    help_map.insert("cd".to_string(), CommandHelp {
        name: "cd".to_string(),
        summary: "Change the current directory".to_string(),
        description: "Change the current working directory to the specified directory. If no directory is specified, changes to the user's home directory.".to_string(),
        usage: "cd [DIRECTORY]".to_string(),
        options: vec![
            OptionHelp { short: Some("-P".to_string()), long: "--physical".to_string(), description: "Use physical directory structure instead of following symbolic links".to_string(), value_type: None },
            OptionHelp { short: Some("-L".to_string()), long: "--logical".to_string(), description: "Follow symbolic links (default)".to_string(), value_type: None },
        ],
        examples: vec![
            ExampleHelp { command: "cd".to_string(), description: "Change to home directory".to_string() },
            ExampleHelp { command: "cd /usr/local".to_string(), description: "Change to /usr/local directory".to_string() },
            ExampleHelp { command: "cd ..".to_string(), description: "Go up one directory level".to_string() },
            ExampleHelp { command: "cd -".to_string(), description: "Change to previous directory".to_string() },
        ],
        see_also: vec!["pwd".to_string(), "pushd".to_string(), "popd".to_string()],
    });

    help_map.insert("pwd".to_string(), CommandHelp {
        name: "pwd".to_string(),
        summary: "Print working directory".to_string(),
        description: "Print the full pathname of the current working directory.".to_string(),
        usage: "pwd [OPTION]".to_string(),
        options: vec![
            OptionHelp { short: Some("-P".to_string()), long: "--physical".to_string(), description: "Print the physical directory, without any symbolic links".to_string(), value_type: None },
            OptionHelp { short: Some("-L".to_string()), long: "--logical".to_string(), description: "Print the logical directory, with symbolic links (default)".to_string(), value_type: None },
        ],
        examples: vec![
            ExampleHelp { command: "pwd".to_string(), description: "Print current working directory".to_string() },
            ExampleHelp { command: "pwd -P".to_string(), description: "Print physical directory path".to_string() },
        ],
        see_also: vec!["cd".to_string(), "realpath".to_string()],
    });

    help_map.insert("mkdir".to_string(), CommandHelp {
        name: "mkdir".to_string(),
        summary: "Create directories".to_string(),
        description: "Create the specified directories and any necessary parent directories.".to_string(),
        usage: "mkdir [OPTION]... DIRECTORY...".to_string(),
        options: vec![
            OptionHelp { short: Some("-p".to_string()), long: "--parents".to_string(), description: "Make parent directories as needed".to_string(), value_type: None },
            OptionHelp { short: Some("-m".to_string()), long: "--mode".to_string(), description: "Set file mode (as in chmod)".to_string(), value_type: Some("MODE".to_string()) },
            OptionHelp { short: Some("-v".to_string()), long: "--verbose".to_string(), description: "Print a message for each created directory".to_string(), value_type: None },
        ],
        examples: vec![
            ExampleHelp { command: "mkdir newdir".to_string(), description: "Create a new directory".to_string() },
            ExampleHelp { command: "mkdir -p path/to/deep/dir".to_string(), description: "Create nested directories".to_string() },
            ExampleHelp { command: "mkdir -m 755 public".to_string(), description: "Create directory with specific permissions".to_string() },
        ],
        see_also: vec!["rmdir".to_string(), "chmod".to_string()],
    });

    help_map.insert("rmdir".to_string(), CommandHelp {
        name: "rmdir".to_string(),
        summary: "Remove empty directories".to_string(),
        description: "Remove empty directories. Will fail if directories contain any files.".to_string(),
        usage: "rmdir [OPTION]... DIRECTORY...".to_string(),
        options: vec![
            OptionHelp { short: Some("-p".to_string()), long: "--parents".to_string(), description: "Remove parent directories if they become empty".to_string(), value_type: None },
            OptionHelp { short: Some("-v".to_string()), long: "--verbose".to_string(), description: "Output a diagnostic for every directory processed".to_string(), value_type: None },
        ],
        examples: vec![
            ExampleHelp { command: "rmdir emptydir".to_string(), description: "Remove an empty directory".to_string() },
            ExampleHelp { command: "rmdir -p path/to/empty/dirs".to_string(), description: "Remove empty parent directories".to_string() },
        ],
        see_also: vec!["mkdir".to_string(), "rm".to_string()],
    });

    help_map.insert("rm".to_string(), CommandHelp {
        name: "rm".to_string(),
        summary: "Remove files and directories".to_string(),
        description: "Remove files and directories. Use with caution as this action is irreversible.".to_string(),
        usage: "rm [OPTION]... FILE...".to_string(),
        options: vec![
            OptionHelp { short: Some("-f".to_string()), long: "--force".to_string(), description: "Ignore nonexistent files, never prompt".to_string(), value_type: None },
            OptionHelp { short: Some("-i".to_string()), long: "--interactive".to_string(), description: "Prompt before every removal".to_string(), value_type: None },
            OptionHelp { short: Some("-r".to_string()), long: "--recursive".to_string(), description: "Remove directories and their contents recursively".to_string(), value_type: None },
            OptionHelp { short: Some("-v".to_string()), long: "--verbose".to_string(), description: "Explain what is being done".to_string(), value_type: None },
        ],
        examples: vec![
            ExampleHelp { command: "rm file.txt".to_string(), description: "Remove a file".to_string() },
            ExampleHelp { command: "rm -r directory".to_string(), description: "Remove directory and all contents".to_string() },
            ExampleHelp { command: "rm -i *.tmp".to_string(), description: "Interactively remove temporary files".to_string() },
        ],
        see_also: vec!["rmdir".to_string(), "unlink".to_string(), "shred".to_string()],
    });

    help_map.insert("cp".to_string(), CommandHelp {
        name: "cp".to_string(),
        summary: "Copy files and directories".to_string(),
        description: "Copy files or directories from source to destination. Preserves file attributes when requested.".to_string(),
        usage: "cp [OPTION]... SOURCE... DEST".to_string(),
        options: vec![
            OptionHelp { short: Some("-r".to_string()), long: "--recursive".to_string(), description: "Copy directories recursively".to_string(), value_type: None },
            OptionHelp { short: Some("-p".to_string()), long: "--preserve".to_string(), description: "Preserve file attributes".to_string(), value_type: None },
            OptionHelp { short: Some("-f".to_string()), long: "--force".to_string(), description: "Force copy by removing destination if needed".to_string(), value_type: None },
            OptionHelp { short: Some("-i".to_string()), long: "--interactive".to_string(), description: "Prompt before overwrite".to_string(), value_type: None },
            OptionHelp { short: Some("-v".to_string()), long: "--verbose".to_string(), description: "Explain what is being done".to_string(), value_type: None },
            OptionHelp { short: Some("-u".to_string()), long: "--update".to_string(), description: "Copy only when source is newer".to_string(), value_type: None },
        ],
        examples: vec![
            ExampleHelp { command: "cp file.txt backup.txt".to_string(), description: "Copy a file".to_string() },
            ExampleHelp { command: "cp -r src/ dest/".to_string(), description: "Copy directory recursively".to_string() },
            ExampleHelp { command: "cp -p important.doc backup/".to_string(), description: "Copy preserving attributes".to_string() },
        ],
        see_also: vec!["mv".to_string(), "rsync".to_string(), "scp".to_string()],
    });

    help_map.insert("mv".to_string(), CommandHelp {
        name: "mv".to_string(),
        summary: "Move/rename files and directories".to_string(),
        description: "Move or rename files and directories. Can move across filesystems and handles advanced file operations on Windows.".to_string(),
        usage: "mv [OPTION]... SOURCE... DEST".to_string(),
        options: vec![
            OptionHelp { short: Some("-f".to_string()), long: "--force".to_string(), description: "Do not prompt before overwriting".to_string(), value_type: None },
            OptionHelp { short: Some("-i".to_string()), long: "--interactive".to_string(), description: "Prompt before overwrite".to_string(), value_type: None },
            OptionHelp { short: Some("-n".to_string()), long: "--no-clobber".to_string(), description: "Do not overwrite existing files".to_string(), value_type: None },
            OptionHelp { short: Some("-v".to_string()), long: "--verbose".to_string(), description: "Explain what is being done".to_string(), value_type: None },
            OptionHelp { short: Some("-u".to_string()), long: "--update".to_string(), description: "Move only when source is newer".to_string(), value_type: None },
        ],
        examples: vec![
            ExampleHelp { command: "mv old.txt new.txt".to_string(), description: "Rename a file".to_string() },
            ExampleHelp { command: "mv file.txt /tmp/".to_string(), description: "Move file to another directory".to_string() },
            ExampleHelp { command: "mv *.log logs/".to_string(), description: "Move all log files to logs directory".to_string() },
        ],
        see_also: vec!["cp".to_string(), "rename".to_string()],
    });

    help_map.insert("ln".to_string(), CommandHelp {
        name: "ln".to_string(),
        summary: "Create links between files".to_string(),
        description: "Create hard and symbolic links between files and directories.".to_string(),
        usage: "ln [OPTION]... TARGET LINK_NAME".to_string(),
        options: vec![
            OptionHelp { short: Some("-s".to_string()), long: "--symbolic".to_string(), description: "Create symbolic links instead of hard links".to_string(), value_type: None },
            OptionHelp { short: Some("-f".to_string()), long: "--force".to_string(), description: "Remove existing destination files".to_string(), value_type: None },
            OptionHelp { short: Some("-v".to_string()), long: "--verbose".to_string(), description: "Print name of each linked file".to_string(), value_type: None },
        ],
        examples: vec![
            ExampleHelp { command: "ln file.txt hardlink.txt".to_string(), description: "Create a hard link".to_string() },
            ExampleHelp { command: "ln -s /path/to/file symlink".to_string(), description: "Create a symbolic link".to_string() },
        ],
        see_also: vec!["readlink".to_string(), "unlink".to_string()],
    });

    help_map.insert("touch".to_string(), CommandHelp {
        name: "touch".to_string(),
        summary: "Change file timestamps or create empty files".to_string(),
        description: "Update the access and modification times of files to the current time, or create empty files if they don't exist.".to_string(),
        usage: "touch [OPTION]... FILE...".to_string(),
        options: vec![
            OptionHelp { short: Some("-a".to_string()), long: "--time=atime".to_string(), description: "Change only the access time".to_string(), value_type: None },
            OptionHelp { short: Some("-m".to_string()), long: "--time=mtime".to_string(), description: "Change only the modification time".to_string(), value_type: None },
            OptionHelp { short: Some("-c".to_string()), long: "--no-create".to_string(), description: "Do not create any files".to_string(), value_type: None },
            OptionHelp { short: Some("-r".to_string()), long: "--reference".to_string(), description: "Use times from reference file".to_string(), value_type: Some("FILE".to_string()) },
            OptionHelp { short: Some("-t".to_string()), long: "--time".to_string(), description: "Use specified time instead of current time".to_string(), value_type: Some("STAMP".to_string()) },
        ],
        examples: vec![
            ExampleHelp { command: "touch newfile.txt".to_string(), description: "Create an empty file or update timestamp".to_string() },
            ExampleHelp { command: "touch -r reference.txt file.txt".to_string(), description: "Set file timestamp to match reference".to_string() },
        ],
        see_also: vec!["stat".to_string(), "date".to_string()],
    });

    // Text Processing Commands
    help_map.insert("cat".to_string(), CommandHelp {
        name: "cat".to_string(),
        summary: "Display file contents".to_string(),
        description: "Concatenate and display files. Enhanced with syntax highlighting, line numbers, and smart paging for large files.".to_string(),
        usage: "cat [OPTION]... [FILE]...".to_string(),
        options: vec![
            OptionHelp { short: Some("-n".to_string()), long: "--number".to_string(), description: "Number all output lines".to_string(), value_type: None },
            OptionHelp { short: Some("-b".to_string()), long: "--number-nonblank".to_string(), description: "Number non-empty output lines".to_string(), value_type: None },
            OptionHelp { short: Some("-s".to_string()), long: "--squeeze-blank".to_string(), description: "Suppress repeated empty output lines".to_string(), value_type: None },
            OptionHelp { short: Some("-A".to_string()), long: "--show-all".to_string(), description: "Show all non-printing characters".to_string(), value_type: None },
            OptionHelp { short: Some("-E".to_string()), long: "--show-ends".to_string(), description: "Display $ at end of each line".to_string(), value_type: None },
            OptionHelp { short: Some("-T".to_string()), long: "--show-tabs".to_string(), description: "Display TAB characters as ^I".to_string(), value_type: None },
        ],
        examples: vec![
            ExampleHelp { command: "cat file.txt".to_string(), description: "Display file contents".to_string() },
            ExampleHelp { command: "cat -n file.txt".to_string(), description: "Display with line numbers".to_string() },
            ExampleHelp { command: "cat file1.txt file2.txt > combined.txt".to_string(), description: "Concatenate multiple files".to_string() },
        ],
        see_also: vec!["less".to_string(), "more".to_string(), "head".to_string(), "tail".to_string()],
    });

    help_map.insert("grep".to_string(), CommandHelp {
        name: "grep".to_string(),
        summary: "Search text patterns in files".to_string(),
        description: "Search for patterns in files using regular expressions. Enhanced with color output, context lines, and advanced pattern matching.".to_string(),
        usage: "grep [OPTION]... PATTERN [FILE]...".to_string(),
        options: vec![
            OptionHelp { short: Some("-i".to_string()), long: "--ignore-case".to_string(), description: "Ignore case distinctions".to_string(), value_type: None },
            OptionHelp { short: Some("-v".to_string()), long: "--invert-match".to_string(), description: "Select non-matching lines".to_string(), value_type: None },
            OptionHelp { short: Some("-r".to_string()), long: "--recursive".to_string(), description: "Search directories recursively".to_string(), value_type: None },
            OptionHelp { short: Some("-n".to_string()), long: "--line-number".to_string(), description: "Show line numbers".to_string(), value_type: None },
            OptionHelp { short: Some("-A".to_string()), long: "--after-context".to_string(), description: "Show lines after match".to_string(), value_type: Some("NUM".to_string()) },
            OptionHelp { short: Some("-B".to_string()), long: "--before-context".to_string(), description: "Show lines before match".to_string(), value_type: Some("NUM".to_string()) },
            OptionHelp { short: Some("-C".to_string()), long: "--context".to_string(), description: "Show lines around match".to_string(), value_type: Some("NUM".to_string()) },
            OptionHelp { short: None, long: "--color".to_string(), description: "Colorize output".to_string(), value_type: Some("WHEN".to_string()) },
        ],
        examples: vec![
            ExampleHelp { command: "grep 'pattern' file.txt".to_string(), description: "Search for pattern in file".to_string() },
            ExampleHelp { command: "grep -r 'function' src/".to_string(), description: "Recursively search in directory".to_string() },
            ExampleHelp { command: "grep -n -C 3 'error' logfile".to_string(), description: "Show line numbers with 3 lines context".to_string() },
        ],
        see_also: vec!["awk".to_string(), "sed".to_string(), "find".to_string()],
    });

    help_map.insert("find".to_string(), CommandHelp {
        name: "find".to_string(),
        summary: "Search for files and directories".to_string(),
        description: "Search for files and directories matching specified criteria. Supports complex expressions, actions, and cross-platform file system operations.".to_string(),
        usage: "find [PATH]... [EXPRESSION]".to_string(),
        options: vec![
            OptionHelp { short: None, long: "-name".to_string(), description: "Base of file name matches pattern".to_string(), value_type: Some("PATTERN".to_string()) },
            OptionHelp { short: None, long: "-type".to_string(), description: "File type (f=file, d=directory, l=link)".to_string(), value_type: Some("TYPE".to_string()) },
            OptionHelp { short: None, long: "-size".to_string(), description: "File size (e.g., +1M, -100k)".to_string(), value_type: Some("SIZE".to_string()) },
            OptionHelp { short: None, long: "-mtime".to_string(), description: "Modification time in days".to_string(), value_type: Some("DAYS".to_string()) },
            OptionHelp { short: None, long: "-exec".to_string(), description: "Execute command on found files".to_string(), value_type: Some("CMD".to_string()) },
            OptionHelp { short: None, long: "-print".to_string(), description: "Print full file names".to_string(), value_type: None },
            OptionHelp { short: None, long: "-delete".to_string(), description: "Delete found files/directories".to_string(), value_type: None },
        ],
        examples: vec![
            ExampleHelp { command: "find . -name '*.txt'".to_string(), description: "Find all .txt files".to_string() },
            ExampleHelp { command: "find /usr -type d -name 'bin'".to_string(), description: "Find directories named 'bin'".to_string() },
            ExampleHelp { command: "find . -size +1M -exec ls -lh {} \\;".to_string(), description: "Find large files and list them".to_string() },
        ],
        see_also: vec!["locate".to_string(), "grep".to_string(), "ls".to_string()],
    });

    // System Information Commands
    help_map.insert("ps".to_string(), CommandHelp {
        name: "ps".to_string(),
        summary: "Display running processes".to_string(),
        description: "Display information about running processes. Enhanced with filtering, sorting, and detailed process information.".to_string(),
        usage: "ps [OPTION]...".to_string(),
        options: vec![
            OptionHelp { short: Some("-a".to_string()), long: "--all".to_string(), description: "Show processes for all users".to_string(), value_type: None },
            OptionHelp { short: Some("-u".to_string()), long: "--user".to_string(), description: "Show user-oriented format".to_string(), value_type: None },
            OptionHelp { short: Some("-x".to_string()), long: "--no-heading".to_string(), description: "Show processes without controlling terminal".to_string(), value_type: None },
            OptionHelp { short: Some("-f".to_string()), long: "--full".to_string(), description: "Full format listing".to_string(), value_type: None },
            OptionHelp { short: Some("-e".to_string()), long: "--everyone".to_string(), description: "Show all processes".to_string(), value_type: None },
        ],
        examples: vec![
            ExampleHelp { command: "ps".to_string(), description: "Show current user processes".to_string() },
            ExampleHelp { command: "ps aux".to_string(), description: "Show all processes with full info".to_string() },
            ExampleHelp { command: "ps -ef".to_string(), description: "Show all processes in full format".to_string() },
        ],
        see_also: vec!["top".to_string(), "kill".to_string(), "jobs".to_string()],
    });

    help_map.insert("top".to_string(), CommandHelp {
        name: "top".to_string(),
        summary: "Display system processes dynamically".to_string(),
        description: "Display and update sorted information about running processes in real-time.".to_string(),
        usage: "top [OPTION]...".to_string(),
        options: vec![
            OptionHelp { short: Some("-d".to_string()), long: "--delay".to_string(), description: "Delay between updates".to_string(), value_type: Some("SECONDS".to_string()) },
            OptionHelp { short: Some("-n".to_string()), long: "--iterations".to_string(), description: "Number of iterations before exiting".to_string(), value_type: Some("NUM".to_string()) },
            OptionHelp { short: Some("-p".to_string()), long: "--pid".to_string(), description: "Monitor specific process IDs".to_string(), value_type: Some("PID".to_string()) },
            OptionHelp { short: Some("-u".to_string()), long: "--user".to_string(), description: "Monitor specific user".to_string(), value_type: Some("USER".to_string()) },
        ],
        examples: vec![
            ExampleHelp { command: "top".to_string(), description: "Start interactive process monitor".to_string() },
            ExampleHelp { command: "top -d 1".to_string(), description: "Update every second".to_string() },
            ExampleHelp { command: "top -n 5".to_string(), description: "Show 5 iterations and exit".to_string() },
        ],
        see_also: vec!["ps".to_string(), "htop".to_string(), "free".to_string()],
    });

    help_map.insert("free".to_string(), CommandHelp {
        name: "free".to_string(),
        summary: "Display memory usage".to_string(),
        description: "Display the amount of free and used memory in the system including physical memory, swap, and buffers.".to_string(),
        usage: "free [OPTION]...".to_string(),
        options: vec![
            OptionHelp { short: Some("-h".to_string()), long: "--human".to_string(), description: "Human readable output".to_string(), value_type: None },
            OptionHelp { short: Some("-b".to_string()), long: "--bytes".to_string(), description: "Show output in bytes".to_string(), value_type: None },
            OptionHelp { short: Some("-k".to_string()), long: "--kilo".to_string(), description: "Show output in kilobytes".to_string(), value_type: None },
            OptionHelp { short: Some("-m".to_string()), long: "--mega".to_string(), description: "Show output in megabytes".to_string(), value_type: None },
            OptionHelp { short: Some("-g".to_string()), long: "--giga".to_string(), description: "Show output in gigabytes".to_string(), value_type: None },
            OptionHelp { short: Some("-s".to_string()), long: "--seconds".to_string(), description: "Repeat every N seconds".to_string(), value_type: Some("N".to_string()) },
        ],
        examples: vec![
            ExampleHelp { command: "free".to_string(), description: "Show memory usage".to_string() },
            ExampleHelp { command: "free -h".to_string(), description: "Show in human readable format".to_string() },
            ExampleHelp { command: "free -s 1".to_string(), description: "Update every second".to_string() },
        ],
        see_also: vec!["top".to_string(), "ps".to_string(), "vmstat".to_string()],
    });

    help_map.insert("df".to_string(), CommandHelp {
        name: "df".to_string(),
        summary: "Display filesystem disk space usage".to_string(),
        description: "Display the amount of disk space available on file systems containing each file name argument.".to_string(),
        usage: "df [OPTION]... [FILE]...".to_string(),
        options: vec![
            OptionHelp { short: Some("-h".to_string()), long: "--human-readable".to_string(), description: "Print sizes in human readable format".to_string(), value_type: None },
            OptionHelp { short: Some("-a".to_string()), long: "--all".to_string(), description: "Include dummy file systems".to_string(), value_type: None },
            OptionHelp { short: Some("-i".to_string()), long: "--inodes".to_string(), description: "List inode information instead of block usage".to_string(), value_type: None },
            OptionHelp { short: Some("-T".to_string()), long: "--print-type".to_string(), description: "Print file system type".to_string(), value_type: None },
        ],
        examples: vec![
            ExampleHelp { command: "df".to_string(), description: "Show disk usage for all filesystems".to_string() },
            ExampleHelp { command: "df -h".to_string(), description: "Show in human readable format".to_string() },
            ExampleHelp { command: "df -i".to_string(), description: "Show inode usage".to_string() },
        ],
        see_also: vec!["du".to_string(), "lsblk".to_string(), "mount".to_string()],
    });

    help_map.insert("du".to_string(), CommandHelp {
        name: "du".to_string(),
        summary: "Display directory space usage".to_string(),
        description: "Display the disk usage of files and directories. Supports recursive analysis and various output formats.".to_string(),
        usage: "du [OPTION]... [FILE]...".to_string(),
        options: vec![
            OptionHelp { short: Some("-h".to_string()), long: "--human-readable".to_string(), description: "Print sizes in human readable format".to_string(), value_type: None },
            OptionHelp { short: Some("-s".to_string()), long: "--summarize".to_string(), description: "Display only total for each argument".to_string(), value_type: None },
            OptionHelp { short: Some("-a".to_string()), long: "--all".to_string(), description: "Write counts for all files, not just directories".to_string(), value_type: None },
            OptionHelp { short: Some("-c".to_string()), long: "--total".to_string(), description: "Produce a grand total".to_string(), value_type: None },
            OptionHelp { short: Some("-d".to_string()), long: "--max-depth".to_string(), description: "Max directory depth".to_string(), value_type: Some("N".to_string()) },
        ],
        examples: vec![
            ExampleHelp { command: "du".to_string(), description: "Show disk usage of current directory".to_string() },
            ExampleHelp { command: "du -sh *".to_string(), description: "Show summary for all items".to_string() },
            ExampleHelp { command: "du -h --max-depth=1".to_string(), description: "Show usage one level deep".to_string() },
        ],
        see_also: vec!["df".to_string(), "ls".to_string(), "find".to_string()],
    });

    // Network and System Tools
    help_map.insert("ping".to_string(), CommandHelp {
        name: "ping".to_string(),
        summary: "Send ICMP echo requests to network hosts".to_string(),
        description: "Send ICMP echo request packets to a network host and measure response times. Cross-platform implementation with detailed statistics.".to_string(),
        usage: "ping [OPTION]... HOST".to_string(),
        options: vec![
            OptionHelp { short: Some("-c".to_string()), long: "--count".to_string(), description: "Number of packets to send".to_string(), value_type: Some("COUNT".to_string()) },
            OptionHelp { short: Some("-i".to_string()), long: "--interval".to_string(), description: "Wait interval between packets".to_string(), value_type: Some("SECONDS".to_string()) },
            OptionHelp { short: Some("-s".to_string()), long: "--size".to_string(), description: "Packet size in bytes".to_string(), value_type: Some("SIZE".to_string()) },
            OptionHelp { short: Some("-t".to_string()), long: "--ttl".to_string(), description: "Set Time To Live".to_string(), value_type: Some("TTL".to_string()) },
            OptionHelp { short: Some("-W".to_string()), long: "--timeout".to_string(), description: "Timeout for each packet".to_string(), value_type: Some("SECONDS".to_string()) },
        ],
        examples: vec![
            ExampleHelp { command: "ping google.com".to_string(), description: "Ping Google's servers".to_string() },
            ExampleHelp { command: "ping -c 4 127.0.0.1".to_string(), description: "Send 4 packets to localhost".to_string() },
            ExampleHelp { command: "ping -s 1000 example.com".to_string(), description: "Send large packets".to_string() },
        ],
        see_also: vec!["traceroute".to_string(), "netstat".to_string(), "nslookup".to_string()],
    });

    // Compression and Archive Tools
    help_map.insert("tar".to_string(), CommandHelp {
        name: "tar".to_string(),
        summary: "Archive files and directories".to_string(),
        description: "Create, extract, and manipulate archive files. Pure Rust implementation with support for various compression formats.".to_string(),
        usage: "tar [OPTION]... [FILE]...".to_string(),
        options: vec![
            OptionHelp { short: Some("-c".to_string()), long: "--create".to_string(), description: "Create a new archive".to_string(), value_type: None },
            OptionHelp { short: Some("-x".to_string()), long: "--extract".to_string(), description: "Extract files from archive".to_string(), value_type: None },
            OptionHelp { short: Some("-t".to_string()), long: "--list".to_string(), description: "List archive contents".to_string(), value_type: None },
            OptionHelp { short: Some("-f".to_string()), long: "--file".to_string(), description: "Archive file name".to_string(), value_type: Some("ARCHIVE".to_string()) },
            OptionHelp { short: Some("-z".to_string()), long: "--gzip".to_string(), description: "Use gzip compression".to_string(), value_type: None },
            OptionHelp { short: Some("-j".to_string()), long: "--bzip2".to_string(), description: "Use bzip2 compression".to_string(), value_type: None },
            OptionHelp { short: Some("-v".to_string()), long: "--verbose".to_string(), description: "Verbose output".to_string(), value_type: None },
        ],
        examples: vec![
            ExampleHelp { command: "tar -czf archive.tar.gz dir/".to_string(), description: "Create compressed archive".to_string() },
            ExampleHelp { command: "tar -xzf archive.tar.gz".to_string(), description: "Extract compressed archive".to_string() },
            ExampleHelp { command: "tar -tzf archive.tar.gz".to_string(), description: "List archive contents".to_string() },
        ],
        see_also: vec!["gzip".to_string(), "zip".to_string(), "7z".to_string()],
    });

    // Add comprehensive entries for all implemented commands from the lib.rs scan
    let comprehensive_commands = [
        // Shell Built-ins
        ("alias", "Create and manage command aliases", "alias [NAME[=VALUE]...]", vec!["unalias".to_string()]),
        ("echo", "Display text to output", "echo [OPTION]... [STRING]...", vec!["printf".to_string()]),
        ("exit", "Exit the shell", "exit [STATUS]", vec!["logout".to_string()]),
        ("export", "Set environment variables", "export [NAME[=VALUE]...]", vec!["env".to_string(), "set".to_string()]),
        ("history", "Display command history", "history [OPTION]...", vec!["fc".to_string()]),
        ("jobs", "Display active jobs", "jobs [OPTION]...", vec!["bg".to_string(), "fg".to_string()]),
        ("kill", "Terminate processes by PID or job", "kill [OPTION]... PID...", vec!["killall".to_string(), "pkill".to_string()]),
        ("bg", "Put jobs in background", "bg [JOB_SPEC...]", vec!["fg".to_string(), "jobs".to_string()]),
        ("fg", "Bring jobs to foreground", "fg [JOB_SPEC]", vec!["bg".to_string(), "jobs".to_string()]),
        
        // Text Processing
        ("awk", "Pattern scanning and processing language", "awk [OPTION]... PROGRAM [FILE]...", vec!["sed".to_string(), "grep".to_string()]),
        ("sed", "Stream editor for filtering and transforming text", "sed [OPTION]... SCRIPT [FILE]...", vec!["awk".to_string(), "tr".to_string()]),
        ("tr", "Translate or delete characters", "tr [OPTION]... SET1 [SET2]", vec!["sed".to_string()]),
        ("sort", "Sort lines of text files", "sort [OPTION]... [FILE]...", vec!["uniq".to_string(), "comm".to_string()]),
        ("uniq", "Report or omit repeated lines", "uniq [OPTION]... [INPUT [OUTPUT]]", vec!["sort".to_string()]),
        ("head", "Display first lines of files", "head [OPTION]... [FILE]...", vec!["tail".to_string(), "cat".to_string()]),
        ("tail", "Display last lines of files", "tail [OPTION]... [FILE]...", vec!["head".to_string(), "less".to_string()]),
        ("wc", "Count lines, words, and characters", "wc [OPTION]... [FILE]...", vec!["grep".to_string()]),
        ("cut", "Extract sections from lines", "cut [OPTION]... [FILE]...", vec!["awk".to_string(), "tr".to_string()]),
        ("less", "View file contents with paging", "less [OPTION]... [FILE]...", vec!["more".to_string(), "cat".to_string()]),
        ("fold", "Wrap text to specified width", "fold [OPTION]... [FILE]...", vec!["fmt".to_string()]),
        
        // File Operations
        ("stat", "Display file or filesystem status", "stat [OPTION]... FILE...", vec!["ls".to_string(), "file".to_string()]),
        ("chmod", "Change file permissions", "chmod [OPTION]... MODE FILE...", vec!["chown".to_string(), "umask".to_string()]),
        ("chown", "Change file ownership", "chown [OPTION]... OWNER FILE...", vec!["chgrp".to_string(), "chmod".to_string()]),
        ("chgrp", "Change group ownership", "chgrp [OPTION]... GROUP FILE...", vec!["chown".to_string(), "chmod".to_string()]),
        
        // System Information
        ("uptime", "Show system uptime and load", "uptime [OPTION]...", vec!["who".to_string(), "w".to_string()]),
        ("whoami", "Print current username", "whoami", vec!["id".to_string(), "who".to_string()]),
        ("id", "Print user and group IDs", "id [OPTION]... [USER]", vec!["whoami".to_string(), "groups".to_string()]),
        ("groups", "Print group memberships", "groups [USER]...", vec!["id".to_string()]),
        ("hostname", "Display or set system hostname", "hostname [OPTION]... [NAME]", vec!["uname".to_string()]),
        ("uname", "Display system information", "uname [OPTION]...", vec!["hostname".to_string()]),
        ("env", "Display environment variables", "env [OPTION]... [NAME=VALUE]... [COMMAND [ARG]...]", vec!["export".to_string(), "printenv".to_string()]),
        ("date", "Display or set system date", "date [OPTION]... [+FORMAT]", vec!["cal".to_string(), "timedatectl".to_string()]),
        
        // Network Tools  
        ("wget", "Download files from web", "wget [OPTION]... [URL]...", vec!["curl".to_string(), "fetch".to_string()]),
        ("curl", "Transfer data to/from servers", "curl [OPTION]... [URL]...", vec!["wget".to_string()]),
        ("netstat", "Display network connections", "netstat [OPTION]...", vec!["ss".to_string(), "lsof".to_string()]),
        ("ssh", "Secure shell remote login", "ssh [OPTION]... [USER@]HOST [COMMAND]", vec!["scp".to_string(), "telnet".to_string()]),
        ("telnet", "Connect to remote hosts", "telnet [OPTION]... [HOST [PORT]]", vec!["ssh".to_string(), "nc".to_string()]),
        ("nc", "Network cat - network connections", "nc [OPTION]... HOST PORT", vec!["telnet".to_string(), "socat".to_string()]),
        
        // Compression
        ("gzip", "Compress files using gzip", "gzip [OPTION]... [FILE]...", vec!["gunzip".to_string(), "bzip2".to_string()]),
        ("bzip2", "Compress files using bzip2", "bzip2 [OPTION]... [FILE]...", vec!["bunzip2".to_string(), "gzip".to_string()]),
        ("xz", "Compress files using xz", "xz [OPTION]... [FILE]...", vec!["unxz".to_string(), "gzip".to_string()]),
        ("zstd", "Compress files using zstandard", "zstd [OPTION]... [FILE]...", vec!["unzstd".to_string(), "gzip".to_string()]),
        ("zip", "Create ZIP archives", "zip [OPTION]... ARCHIVE [FILE]...", vec!["unzip".to_string(), "tar".to_string()]),
        ("7z", "7-Zip archive tool", "7z [COMMAND] [OPTION]... ARCHIVE [FILE]...", vec!["zip".to_string(), "tar".to_string()]),
        
        // Process Control
        ("sleep", "Delay for specified time", "sleep NUMBER[SUFFIX]...", vec!["timeout".to_string()]),
        ("timeout", "Run command with time limit", "timeout [OPTION] DURATION COMMAND [ARG]...", vec!["sleep".to_string()]),
        ("nohup", "Run commands immune to hangups", "nohup COMMAND [ARG]...", vec!["disown".to_string()]),
        ("nice", "Run command with modified priority", "nice [OPTION] [COMMAND [ARG]...]", vec!["renice".to_string()]),
        ("renice", "Alter priority of running processes", "renice [-n] PRIORITY PID...", vec!["nice".to_string()]),
        
        // Advanced Features
        ("logstats", "Display logging statistics and rates", "logstats [--json|--pretty|--prom]", vec!["update".to_string()]),
        ("update", "System update and package management", "update [OPTION]...", vec!["package".to_string()]),
        ("smart-alias", "Intelligent alias management system", "smart-alias [COMMAND] [OPTION]...", vec!["alias".to_string()]),
        ("monitor", "Advanced system monitoring", "monitor [OPTION]...", vec!["top".to_string(), "ps".to_string()]),
    ];
    
    for (cmd, summary, usage, see_also) in &comprehensive_commands {
        if !help_map.contains_key(*cmd) {
            help_map.insert(cmd.to_string(), CommandHelp {
                name: cmd.to_string(),
                summary: summary.to_string(),
                description: format!("The {} command provides {}. This is a full-featured implementation with enterprise-grade functionality.", cmd, summary.to_lowercase()),
                usage: usage.to_string(),
                options: vec![],
                examples: vec![],
                see_also: see_also.clone(),
            });
        }
    }
    
    help_map
}

/// Show general help with list of available commands
fn show_general_help(help_map: &HashMap<String, CommandHelp>) {
    println!("NexusShell Built-in Commands Help");
    println!("=================================");
    println!();
    println!("NexusShell provides a comprehensive collection of built-in commands");
    println!("for file management, text processing, system administration, and more.");
    println!();
    println!("Usage: help [COMMAND]");
    println!();
    
    // Categorize commands for better organization
    let categories = [
        ("File & Directory Operations", vec![
            "ls", "cd", "pwd", "mkdir", "rmdir", "rm", "cp", "mv", "ln", "touch", 
            "find", "locate", "stat", "chmod", "chown", "chgrp", "mount", "umount"
        ]),
        ("Text Processing & Search", vec![
            "cat", "grep", "awk", "sed", "tr", "sort", "uniq", "head", "tail", 
            "less", "wc", "cut", "fold", "comm", "diff", "join", "paste", "split"
        ]),
        ("System Information", vec![
            "ps", "top", "kill", "free", "uptime", "df", "du", "uname", "hostname",
            "whoami", "id", "groups", "env", "date", "cal", "lsof"
        ]),
        ("Archive & Compression", vec![
            "tar", "gzip", "bzip2", "xz", "zstd", "zip", "7z", "gunzip", "bunzip2", 
            "unxz", "unzstd", "unzip"
        ]),
        ("Network & Communication", vec![
            "ping", "wget", "curl", "ssh", "scp", "telnet", "nc", "netstat", "ss",
            "arp", "dig", "nslookup", "rsync"
        ]),
        ("Process & Job Control", vec![
            "jobs", "bg", "fg", "nohup", "disown", "timeout", "sleep", "nice", 
            "renice", "wait", "suspend", "kill"
        ]),
        ("Shell Built-ins", vec![
            "alias", "echo", "export", "history", "exit", "source", "eval", "exec",
            "read", "test", "if", "case", "while", "for", "function", "local"
        ]),
        ("Advanced & Specialized", vec![
            "logstats", "update", "smart-alias", "monitor", "package", "bc", "dc",
            "expr", "base64", "md5sum", "sha1sum", "strings", "hexdump"
        ]),
    ];
    
    for (category, commands) in &categories {
        println!("📁 {}", category);
        println!("{}", "─".repeat(category.len() + 4));
        
        let mut displayed_commands = Vec::new();
        for &command in commands {
            if let Some(help) = help_map.get(command) {
                displayed_commands.push(format!("  {:12} - {}", command, help.summary));
            }
        }
        
        if !displayed_commands.is_empty() {
            for cmd_line in displayed_commands {
                println!("{}", cmd_line);
            }
            println!();
        }
    }
    
    // Show any remaining commands not in categories
    let categorized: std::collections::HashSet<_> = categories.iter()
        .flat_map(|(_, cmds)| cmds.iter().copied())
        .collect();
    
    let mut other_commands = Vec::new();
    for command in help_map.keys() {
        if !categorized.contains(command.as_str()) {
            if let Some(help) = help_map.get(command) {
                other_commands.push((command.clone(), help.summary.clone()));
            }
        }
    }
    
    if !other_commands.is_empty() {
        println!("🔧 Other Commands");
        println!("─────────────────");
        other_commands.sort_by(|a, b| a.0.cmp(&b.0));
        for (command, summary) in other_commands {
            println!("  {:12} - {}", command, summary);
        }
        println!();
    }
    
    println!("💡 Tips:");
    println!("  • Use 'help COMMAND' for detailed information about a specific command");
    println!("  • Most commands support --help for quick reference");
    println!("  • Commands support both short (-h) and long (--help) options");
    println!("  • Use tab completion to discover available commands and options");
    println!();
    println!("📖 For more information: https://nexusshell.dev/docs");
}

/// Show detailed help for a specific command
fn show_command_help(help: &CommandHelp) {
    println!("📖 {} - {}", help.name.to_uppercase(), help.summary);
    println!("{}", "=".repeat(help.name.len() + help.summary.len() + 5));
    println!();
    
    println!("📝 DESCRIPTION");
    println!("   {}", help.description);
    println!();
    
    println!("🔧 USAGE");
    println!("   {}", help.usage);
    println!();
    
    if !help.options.is_empty() {
        println!("⚙️  OPTIONS");
        for option in &help.options {
            let opt_line = if let Some(short) = &option.short {
                format!("{}, {}", short, option.long)
            } else {
                option.long.clone()
            };
            
            if let Some(value_type) = &option.value_type {
                println!("   {:<20} = {} - {}", opt_line, value_type, option.description);
            } else {
                println!("   {:<20} {}", opt_line, option.description);
            }
        }
        println!();
    }
    
    if !help.examples.is_empty() {
        println!("💡 EXAMPLES");
        for (i, example) in help.examples.iter().enumerate() {
            println!("   {}. {}", i + 1, example.description);
            println!("      $ {}", example.command);
            if i < help.examples.len() - 1 {
                println!();
            }
        }
        println!();
    }
    
    if !help.see_also.is_empty() {
        println!("🔗 SEE ALSO");
        println!("   Related commands: {}", help.see_also.join(", "));
        println!();
    }
    
    println!("💡 Tips:");
    println!("   • Use 'help' to see all available commands");
    println!("   • Most commands also support --help option");
    println!("   • Use tab completion for command and option discovery");
}

/// Show list of available commands when command not found
fn show_available_commands(help_map: &HashMap<String, CommandHelp>) {
    println!();
    println!("❌ Command not found!");
    println!();
    println!("💡 Did you mean one of these commands?");
    println!();
    
    let mut commands: Vec<_> = help_map.keys().collect();
    commands.sort();
    
    // Group commands by first letter for better organization
    let mut current_letter = ' ';
    let mut count = 0;
    
    for command in &commands {
        let first_char = command.chars().next().unwrap_or(' ').to_ascii_uppercase();
        
        if first_char != current_letter {
            if count > 0 {
                println!();
            }
            current_letter = first_char;
            println!("📂 {} commands:", current_letter);
            count = 0;
        }
        
        print!("  {:12}", command);
        count += 1;
        
        if count % 6 == 0 {
            println!();
        }
    }
    
    if count % 6 != 0 {
        println!();
    }
    
    println!();
    println!("💡 Use 'help COMMAND' for detailed information about a specific command.");
    println!("💡 Use 'help' to see all commands organized by category.");
}

/// Find commands similar to the input (simple substring matching)
fn find_similar_commands(input: &str, help_map: &HashMap<String, CommandHelp>) -> Vec<String> {
    let input_lower = input.to_lowercase();
    let mut similar = Vec::new();
    
    for command in help_map.keys() {
        if command.to_lowercase().contains(&input_lower) || 
           input_lower.contains(&command.to_lowercase()) ||
           levenshtein_distance(&input_lower, &command.to_lowercase()) <= 2 {
            similar.push(command.clone());
        }
    }
    
    similar.sort();
    similar.truncate(5); // Show max 5 suggestions
    similar
}

/// Simple Levenshtein distance calculation
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
    
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }
    
    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }
    
    matrix[len1][len2]
}

/// Search for commands matching a pattern
fn search_commands(pattern: &str, help_map: &HashMap<String, CommandHelp>) {
    let pattern_lower = pattern.to_lowercase();
    let mut matches = Vec::new();
    
    for (command, help) in help_map {
        if command.to_lowercase().contains(&pattern_lower) ||
           help.summary.to_lowercase().contains(&pattern_lower) ||
           help.description.to_lowercase().contains(&pattern_lower) {
            matches.push((command, help));
        }
    }
    
    if matches.is_empty() {
        println!("🔍 No commands found matching '{}'", pattern);
        return;
    }
    
    matches.sort_by_key(|(cmd, _)| cmd.as_str());
    
    println!("🔍 Commands matching '{}':", pattern);
    println!("{}", "─".repeat(30));
    
    for (command, help) in matches {
        println!("  {:12} - {}", command, help.summary);
    }
    
    println!();
    println!("💡 Use 'help COMMAND' for detailed information about any command.");
}

/// Show all commands with brief descriptions
fn show_all_commands_detailed(help_map: &HashMap<String, CommandHelp>) {
    println!("📚 All NexusShell Built-in Commands");
    println!("===================================");
    println!();
    
    let mut commands: Vec<_> = help_map.iter().collect();
    commands.sort_by_key(|(cmd, _)| cmd.as_str());
    
    for (command, help) in commands {
        println!("📖 {}", command.to_uppercase());
        println!("   Summary: {}", help.summary);
        println!("   Usage:   {}", help.usage);
        
        if !help.options.is_empty() {
            let option_count = help.options.len();
            println!("   Options: {} available", option_count);
        }
        
        if !help.examples.is_empty() {
            let example_count = help.examples.len();
            println!("   Examples: {} available", example_count);
        }
        
        println!();
    }
    
    println!("💡 Use 'help COMMAND' for complete documentation of any command.");
}
