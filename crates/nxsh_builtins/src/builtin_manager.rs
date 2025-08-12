use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use nxsh_core::{ShellResult, ExecutionResult};

/// Builtin command management system
#[derive(Debug)]
pub struct BuiltinManager {
    commands: Arc<RwLock<HashMap<String, BuiltinCommand>>>,
    stats: Arc<RwLock<BuiltinStats>>,
    performance_monitor: Arc<RwLock<PerformanceMonitor>>,
}

impl BuiltinManager {
    pub fn new() -> Self {
        let mut manager = Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(BuiltinStats::default())),
            performance_monitor: Arc::new(RwLock::new(PerformanceMonitor::new())),
        };
        
        manager.register_all_builtins();
        manager
    }

    /// Register all built-in commands
    fn register_all_builtins(&mut self) {
        let mut commands = self.commands.write().unwrap();
        
        // Core Shell Commands
        commands.insert("cd".to_string(), BuiltinCommand::new("cd", "Change directory", BuiltinCategory::Core));
        commands.insert("pwd".to_string(), BuiltinCommand::new("pwd", "Print working directory", BuiltinCategory::Core));
        commands.insert("exit".to_string(), BuiltinCommand::new("exit", "Exit shell", BuiltinCategory::Core));
        commands.insert("help".to_string(), BuiltinCommand::new("help", "Show help", BuiltinCategory::Core));
        commands.insert("history".to_string(), BuiltinCommand::new("history", "Command history", BuiltinCategory::Core));

        // File Operations
        commands.insert("ls".to_string(), BuiltinCommand::new("ls", "List files", BuiltinCategory::FileSystem));
        commands.insert("cat".to_string(), BuiltinCommand::new("cat", "Display file contents", BuiltinCategory::FileSystem));
        commands.insert("cp".to_string(), BuiltinCommand::new("cp", "Copy files", BuiltinCategory::FileSystem));
        commands.insert("mv".to_string(), BuiltinCommand::new("mv", "Move/rename files", BuiltinCategory::FileSystem));
        commands.insert("rm".to_string(), BuiltinCommand::new("rm", "Remove files", BuiltinCategory::FileSystem));
        commands.insert("mkdir".to_string(), BuiltinCommand::new("mkdir", "Create directories", BuiltinCategory::FileSystem));
        commands.insert("rmdir".to_string(), BuiltinCommand::new("rmdir", "Remove directories", BuiltinCategory::FileSystem));
        commands.insert("chmod".to_string(), BuiltinCommand::new("chmod", "Change file permissions", BuiltinCategory::FileSystem));
        commands.insert("chown".to_string(), BuiltinCommand::new("chown", "Change file ownership", BuiltinCategory::FileSystem));
        commands.insert("find".to_string(), BuiltinCommand::new("find", "Find files", BuiltinCategory::FileSystem));
        commands.insert("grep".to_string(), BuiltinCommand::new("grep", "Search text patterns", BuiltinCategory::FileSystem));
        commands.insert("head".to_string(), BuiltinCommand::new("head", "Show file beginning", BuiltinCategory::FileSystem));
        commands.insert("tail".to_string(), BuiltinCommand::new("tail", "Show file end", BuiltinCategory::FileSystem));
        commands.insert("touch".to_string(), BuiltinCommand::new("touch", "Create/update files", BuiltinCategory::FileSystem));
        commands.insert("ln".to_string(), BuiltinCommand::new("ln", "Create links", BuiltinCategory::FileSystem));

        // Text Processing
        commands.insert("awk".to_string(), BuiltinCommand::new("awk", "Text processing", BuiltinCategory::TextProcessing));
        commands.insert("sed".to_string(), BuiltinCommand::new("sed", "Stream editor", BuiltinCategory::TextProcessing));
        commands.insert("sort".to_string(), BuiltinCommand::new("sort", "Sort text", BuiltinCategory::TextProcessing));
        commands.insert("uniq".to_string(), BuiltinCommand::new("uniq", "Remove duplicates", BuiltinCategory::TextProcessing));
        commands.insert("cut".to_string(), BuiltinCommand::new("cut", "Cut text columns", BuiltinCategory::TextProcessing));
        commands.insert("tr".to_string(), BuiltinCommand::new("tr", "Translate characters", BuiltinCategory::TextProcessing));
        commands.insert("wc".to_string(), BuiltinCommand::new("wc", "Word count", BuiltinCategory::TextProcessing));

        // System Information
        commands.insert("ps".to_string(), BuiltinCommand::new("ps", "Process status", BuiltinCategory::System));
        commands.insert("top".to_string(), BuiltinCommand::new("top", "System monitor", BuiltinCategory::System));
        commands.insert("df".to_string(), BuiltinCommand::new("df", "Disk usage", BuiltinCategory::System));
        commands.insert("du".to_string(), BuiltinCommand::new("du", "Directory usage", BuiltinCategory::System));
        commands.insert("free".to_string(), BuiltinCommand::new("free", "Memory usage", BuiltinCategory::System));
        commands.insert("uname".to_string(), BuiltinCommand::new("uname", "System information", BuiltinCategory::System));
        commands.insert("uptime".to_string(), BuiltinCommand::new("uptime", "System uptime", BuiltinCategory::System));
        commands.insert("whoami".to_string(), BuiltinCommand::new("whoami", "Current user", BuiltinCategory::System));
        commands.insert("id".to_string(), BuiltinCommand::new("id", "User/group IDs", BuiltinCategory::System));

        // Compression
        commands.insert("tar".to_string(), BuiltinCommand::new("tar", "Archive files", BuiltinCategory::Compression));
        commands.insert("gzip".to_string(), BuiltinCommand::new("gzip", "Gzip compression", BuiltinCategory::Compression));
        commands.insert("gunzip".to_string(), BuiltinCommand::new("gunzip", "Gzip decompression", BuiltinCategory::Compression));
        commands.insert("zip".to_string(), BuiltinCommand::new("zip", "ZIP compression", BuiltinCategory::Compression));
        commands.insert("unzip".to_string(), BuiltinCommand::new("unzip", "ZIP extraction", BuiltinCategory::Compression));
        commands.insert("bzip2".to_string(), BuiltinCommand::new("bzip2", "Bzip2 compression", BuiltinCategory::Compression));
        commands.insert("bunzip2".to_string(), BuiltinCommand::new("bunzip2", "Bzip2 decompression", BuiltinCategory::Compression));
        commands.insert("xz".to_string(), BuiltinCommand::new("xz", "XZ compression", BuiltinCategory::Compression));
        commands.insert("unxz".to_string(), BuiltinCommand::new("unxz", "XZ decompression", BuiltinCategory::Compression));
        commands.insert("zstd".to_string(), BuiltinCommand::new("zstd", "Zstandard compression", BuiltinCategory::Compression));

        // Network
        commands.insert("curl".to_string(), BuiltinCommand::new("curl", "Transfer data", BuiltinCategory::Network));
        commands.insert("wget".to_string(), BuiltinCommand::new("wget", "Download files", BuiltinCategory::Network));
        commands.insert("ping".to_string(), BuiltinCommand::new("ping", "Test connectivity", BuiltinCategory::Network));
        commands.insert("ssh".to_string(), BuiltinCommand::new("ssh", "Secure shell", BuiltinCategory::Network));
        commands.insert("scp".to_string(), BuiltinCommand::new("scp", "Secure copy", BuiltinCategory::Network));
        commands.insert("netstat".to_string(), BuiltinCommand::new("netstat", "Network statistics", BuiltinCategory::Network));

        // Environment
        commands.insert("env".to_string(), BuiltinCommand::new("env", "Environment variables", BuiltinCategory::Environment));
        commands.insert("export".to_string(), BuiltinCommand::new("export", "Export variables", BuiltinCategory::Environment));
        commands.insert("unset".to_string(), BuiltinCommand::new("unset", "Unset variables", BuiltinCategory::Environment));
        commands.insert("alias".to_string(), BuiltinCommand::new("alias", "Create aliases", BuiltinCategory::Environment));
        commands.insert("unalias".to_string(), BuiltinCommand::new("unalias", "Remove aliases", BuiltinCategory::Environment));
        
        // Add more commands as needed to reach 250+
        // This represents the core set of most commonly used commands
    }

    /// Execute a built-in command
    pub async fn execute(&self, command: &str, args: &[String], ctx: &mut nxsh_core::context::ShellContext) -> anyhow::Result<()> {
        let start = std::time::Instant::now();
        
        // Update stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_executions += 1;
        }

        let result = match command {
            "cd" => crate::cd::cd_cli(args, ctx).map_err(|e| e.into()),
            "pwd" => crate::pwd::pwd_cli(args, ctx).map_err(|e| e.into()),
            "ls" => crate::ls::ls_cli(args).map_err(|e| e.into()),
            "cat" => crate::cat::cat_cli(args).map_err(|e| e.into()),
            "echo" => crate::echo::echo_cli(args).map_err(|e| e.into()),
            "grep" => crate::grep::grep_cli(args).map_err(|e| e.into()),
            "find" => crate::find::find_cli(args).map_err(|e| e.into()),
            "ps" => crate::ps::ps_cli(args).map_err(|e| e.into()),
            "top" => crate::top::top_cli(args).map_err(|e| e.into()),
            "id" => crate::id::id_cli(args).map_err(|e| e.into()),
            "bzip2" => crate::bzip2::bzip2_cli(args).map_err(|e| e.into()),
            "bunzip2" => crate::bunzip2::bunzip2_cli(args).map_err(|e| e.into()),
            "xz" => crate::xz::xz_cli(args).map_err(|e| e.into()),
            "unxz" => crate::unxz::unxz_cli(args).map_err(|e| e.into()),
            "zstd" => crate::zstd::zstd_cli(args).map_err(|e| e.into()),
            "help" => self.show_help(args, ctx).await,
            _ => {
                return Err(anyhow::anyhow!("Unknown builtin command: {}", command).into());
            }
        };

        let duration = start.elapsed();
        
        // Record performance
        {
            let mut monitor = self.performance_monitor.write().unwrap();
            monitor.record_execution(command, duration, result.is_ok());
        }

        result
    }

    /// Check if command is a built-in
    pub fn is_builtin(&self, command: &str) -> bool {
        self.commands.read().unwrap().contains_key(command)
    }

    /// Get list of all built-in commands
    pub fn list_commands(&self) -> Vec<String> {
        self.commands.read().unwrap().keys().cloned().collect()
    }

    /// Get command information
    pub fn get_command_info(&self, command: &str) -> Option<BuiltinCommand> {
        self.commands.read().unwrap().get(command).cloned()
    }

    /// Get builtin statistics
    pub fn get_stats(&self) -> BuiltinStats {
        self.stats.read().unwrap().clone()
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> PerformanceStats {
        self.performance_monitor.read().unwrap().get_stats()
    }

    /// Show help for builtins
    async fn show_help(&self, args: &[String], _ctx: &mut nxsh_core::context::ShellContext) -> anyhow::Result<()> {
        if args.is_empty() {
            println!("Available built-in commands:");
            
            let commands = self.commands.read().unwrap();
            let mut by_category: HashMap<BuiltinCategory, Vec<_>> = HashMap::new();
            
            for command in commands.values() {
                by_category.entry(command.category).or_default().push(command);
            }

            for (category, mut commands) in by_category {
                println!("\n{:?}:", category);
                commands.sort_by(|a, b| a.name.cmp(&b.name));
                for cmd in commands {
                    println!("  {:12} - {}", cmd.name, cmd.description);
                }
            }
            
            println!("\nTotal: {} commands", commands.len());
        } else {
            let command = &args[0];
            if let Some(info) = self.get_command_info(command) {
                println!("{}: {}", info.name, info.description);
            } else {
                println!("No help available for '{}'", command);
            }
        }
        
        Ok(())
    }
}

/// Built-in command information
#[derive(Debug, Clone)]
pub struct BuiltinCommand {
    pub name: String,
    pub description: String,
    pub category: BuiltinCategory,
}

impl BuiltinCommand {
    fn new(name: &str, description: &str, category: BuiltinCategory) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            category,
        }
    }
}

/// Command categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinCategory {
    Core,
    FileSystem,
    TextProcessing,
    System,
    Network,
    Compression,
    Environment,
}

/// Builtin execution statistics
#[derive(Debug, Clone, Default)]
pub struct BuiltinStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
}

/// Performance monitoring for builtins
#[derive(Debug)]
pub struct PerformanceMonitor {
    command_stats: HashMap<String, CommandStats>,
}

impl PerformanceMonitor {
    fn new() -> Self {
        Self {
            command_stats: HashMap::new(),
        }
    }

    fn record_execution(&mut self, command: &str, duration: std::time::Duration, success: bool) {
        let stats = self.command_stats.entry(command.to_string()).or_default();
        stats.executions += 1;
        stats.total_time += duration;
        
        if duration < stats.fastest_time || stats.fastest_time.is_zero() {
            stats.fastest_time = duration;
        }
        
        if duration > stats.slowest_time {
            stats.slowest_time = duration;
        }
        
        if success {
            stats.successes += 1;
        } else {
            stats.failures += 1;
        }
    }

    fn get_stats(&self) -> PerformanceStats {
        PerformanceStats {
            command_count: self.command_stats.len(),
            total_executions: self.command_stats.values().map(|s| s.executions).sum(),
            total_time: self.command_stats.values().map(|s| s.total_time).sum(),
        }
    }
}

#[derive(Debug, Default)]
struct CommandStats {
    executions: u64,
    successes: u64,
    failures: u64,
    total_time: std::time::Duration,
    fastest_time: std::time::Duration,
    slowest_time: std::time::Duration,
}

#[derive(Debug)]
pub struct PerformanceStats {
    pub command_count: usize,
    pub total_executions: u64,
    pub total_time: std::time::Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_manager_creation() {
        let manager = BuiltinManager::new();
        let commands = manager.list_commands();
        
        // Should have core commands registered
        assert!(commands.contains(&"cd".to_string()));
        assert!(commands.contains(&"ls".to_string()));
        assert!(commands.contains(&"grep".to_string()));
        
        // Should have substantial command count (aiming for 250+)
        assert!(commands.len() >= 50, "Expected at least 50 commands, got {}", commands.len());
    }

    #[test]
    fn test_builtin_detection() {
        let manager = BuiltinManager::new();
        
        assert!(manager.is_builtin("cd"));
        assert!(manager.is_builtin("ls"));
        assert!(!manager.is_builtin("nonexistent_command"));
    }

    #[test]
    fn test_command_categories() {
        let manager = BuiltinManager::new();
        
        let cd_info = manager.get_command_info("cd").unwrap();
        assert_eq!(cd_info.category, BuiltinCategory::Core);
        
        let ls_info = manager.get_command_info("ls").unwrap();
        assert_eq!(ls_info.category, BuiltinCategory::FileSystem);
    }
}
