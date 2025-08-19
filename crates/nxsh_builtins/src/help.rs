// Comprehensive help system for NexusShell built-in commands
// This module provides detailed help information for all built-in commands

use anyhow::Result;
use nxsh_core::{Context, ExecutionResult};
use std::collections::HashMap;
use crate::ui_design::{
    TableFormatter, Colorize, CommandWizard, WizardStep, InputType,
    StatusDashboard, DashboardSection, StatusItem, ItemStatus, SectionStyle,
    Animation, ProgressBar, Notification, NotificationType, create_advanced_table,
    TableOptions, BorderStyle, TextAlignment
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
pub fn help_cli(_ctx: &mut Context, args: &[String]) -> Result<ExecutionResult> {
    // Initialize the help system with all available commands
    let help_map = initialize_help_map();
    
    match args.len() {
        0 => {
            // Show general help with list of available commands
            show_general_help(&help_map);
        }
        1 => {
            // Show help for specific command
            let command = &args[0];
            if let Some(help) = help_map.get(command) {
                show_command_help(help);
            } else {
                eprintln!("help: no help available for '{command}'");
                show_available_commands(&help_map);
                return Ok(ExecutionResult::success(1));
            }
        }
        _ => {
            eprintln!("help: too many arguments");
            eprintln!("Usage: help [COMMAND]");
            return Ok(ExecutionResult::success(1));
        }
    }
    
    Ok(ExecutionResult::success(0))
}

/// Initialize the comprehensive help map for all built-in commands
fn initialize_help_map() -> HashMap<String, CommandHelp> {
    let mut help_map = HashMap::new();
    
    // Add help for core commands
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
        see_also: vec![],
    });
    
    help_map.insert("ls".to_string(), CommandHelp {
        name: "ls".to_string(),
        summary: "List directory contents".to_string(),
        description: "List information about files and directories. By default, lists the current directory in a simple format.".to_string(),
        usage: "ls [OPTION]... [FILE]...".to_string(),
        options: vec![
            OptionHelp {
                short: Some("-a".to_string()),
                long: "--all".to_string(),
                description: "Show hidden files and directories".to_string(),
                value_type: None,
            },
            OptionHelp {
                short: Some("-l".to_string()),
                long: "--long".to_string(),
                description: "Use long listing format".to_string(),
                value_type: None,
            },
            OptionHelp {
                short: Some("-h".to_string()),
                long: "--human-readable".to_string(),
                description: "Print file sizes in human readable format".to_string(),
                value_type: None,
            },
        ],
        examples: vec![
            ExampleHelp {
                command: "ls".to_string(),
                description: "List files in current directory".to_string(),
            },
            ExampleHelp {
                command: "ls -la".to_string(),
                description: "List all files in long format".to_string(),
            },
        ],
        see_also: vec!["find".to_string(), "stat".to_string()],
    });
    
    // Add help for logstats
    help_map.insert("logstats".to_string(), CommandHelp {
        name: "logstats".to_string(),
        summary: "Display logging statistics and rates".to_string(),
        description: "Display metrics from the structured logging subsystem. Supports plain text and JSON output, and computes rate metrics using a persisted snapshot.".to_string(),
        usage: "logstats [--json|--pretty|--prom|--prometheus]".to_string(),
        options: vec![
            OptionHelp { short: None, long: "--json".to_string(), description: "Output metrics as a compact JSON object".to_string(), value_type: None },
            OptionHelp { short: None, long: "--pretty".to_string(), description: "Output metrics as pretty-printed JSON".to_string(), value_type: None },
            OptionHelp { short: None, long: "--prom".to_string(), description: "Output metrics in Prometheus text format".to_string(), value_type: None },
            OptionHelp { short: None, long: "--prometheus".to_string(), description: "Alias of --prom".to_string(), value_type: None },
        ],
        examples: vec![
            ExampleHelp { command: "logstats".to_string(), description: "Print metrics as key:value lines".to_string() },
            ExampleHelp { command: "logstats --json".to_string(), description: "Print metrics as JSON (machine-readable)".to_string() },
            ExampleHelp { command: "logstats --prom".to_string(), description: "Expose metrics in Prometheus text format".to_string() },
        ],
        see_also: vec!["update".to_string()],
    });
    
    // Add basic help for other common commands
    let common_commands = [
        ("cd", "Change directory"),
        ("pwd", "Print working directory"),
        ("echo", "Display text"),
        ("cat", "Display file contents"),
        ("grep", "Search text patterns"),
        ("find", "Search for files and directories"),
        ("ps", "Display running processes"),
        ("kill", "Terminate processes"),
        ("top", "Display system processes"),
        ("free", "Display memory usage"),
        ("df", "Display filesystem usage"),
        ("du", "Display directory usage"),
        ("mkdir", "Create directories"),
        ("rmdir", "Remove directories"),
        ("rm", "Remove files and directories"),
        ("cp", "Copy files and directories"),
        ("mv", "Move/rename files and directories"),
        ("ln", "Create links"),
        ("chmod", "Change file permissions"),
        ("chown", "Change file ownership"),
        ("alias", "Create command aliases"),
        ("export", "Set environment variables"),
        ("history", "Command history"),
        ("jobs", "Display active jobs"),
        ("exit", "Exit the shell"),
        ("logstats", "Display logging statistics and rates"),
    ];
    
    for (cmd, summary) in &common_commands {
        help_map.insert(cmd.to_string(), CommandHelp {
            name: cmd.to_string(),
            summary: summary.to_string(),
            description: format!("The {} command {}.", cmd, summary.to_lowercase()),
            usage: format!("{cmd} [OPTION]... [ARG]..."),
            options: vec![],
            examples: vec![],
            see_also: vec![],
        });
    }
    
    help_map
}

/// Show general help with list of available commands
fn show_general_help(help_map: &HashMap<String, CommandHelp>) {
    println!("NexusShell Built-in Commands Help");
    println!("=================================");
    println!();
    println!("Usage: help [COMMAND]");
    println!();
    println!("Available commands:");
    
    let mut commands: Vec<_> = help_map.keys().collect();
    commands.sort();
    
    for command in commands {
        if let Some(help) = help_map.get(command) {
            println!("  {:12} - {}", command, help.summary);
        }
    }
    
    println!();
    println!("Use 'help COMMAND' for detailed information about a specific command.");
}

/// Show detailed help for a specific command
fn show_command_help(help: &CommandHelp) {
    println!("{}", help.name.to_uppercase());
    println!("{}(1)", help.name);
    println!();
    
    println!("NAME");
    println!("    {} - {}", help.name, help.summary);
    println!();
    
    println!("SYNOPSIS");
    println!("    {}", help.usage);
    println!();
    
    println!("DESCRIPTION");
    println!("    {}", help.description);
    println!();
    
    if !help.options.is_empty() {
        println!("OPTIONS");
        for option in &help.options {
            let opt_line = if let Some(short) = &option.short {
                format!("{}, {}", short, option.long)
            } else {
                option.long.clone()
            };
            
            if let Some(value_type) = &option.value_type {
                println!("    {opt_line}={value_type}");
            } else {
                println!("    {opt_line}");
            }
            println!("        {}", option.description);
            println!();
        }
    }
    
    if !help.examples.is_empty() {
        println!("EXAMPLES");
        for example in &help.examples {
            println!("    {}", example.command);
            println!("        {}", example.description);
            println!();
        }
    }
    
    if !help.see_also.is_empty() {
        println!("SEE ALSO");
        println!("    {}", help.see_also.join(", "));
        println!();
    }
}

/// Show list of available commands when command not found
fn show_available_commands(help_map: &HashMap<String, CommandHelp>) {
    println!();
    println!("Available commands:");
    
    let mut commands: Vec<_> = help_map.keys().collect();
    commands.sort();
    
    let mut line = String::new();
    for (i, command) in commands.iter().enumerate() {
        if i > 0 && i % 8 == 0 {
            println!("  {line}");
            line.clear();
        }
        
        if !line.is_empty() {
            line.push_str(", ");
        }
        line.push_str(command);
    }
    
    if !line.is_empty() {
        println!("  {line}");
    }
    
    println!();
    println!("Use 'help COMMAND' for detailed information about a specific command.");
}
