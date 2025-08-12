//! Built-in command registry for efficient command lookup and management

use std::collections::HashMap;

/// Registry of all built-in commands for efficient lookup and management
#[derive(Clone)]
pub struct BuiltinRegistry {
    /// Map of command names to builtin descriptions
    commands: HashMap<String, String>,
}

impl BuiltinRegistry {
    /// Create a new builtin registry with all standard built-in commands
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        
        // Register all standard built-in commands
        registry.register_standard_builtins();
        
        registry
    }
    
    /// Register a new builtin command
    pub fn register(&mut self, name: String, description: String) {
        self.commands.insert(name, description);
    }
    
    /// Get a builtin command description by name
    pub fn get(&self, name: &str) -> Option<&String> {
        self.commands.get(name)
    }
    
    /// Check if a command is a builtin
    pub fn is_builtin(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }
    
    /// Get list of all builtin command names
    pub fn list_commands(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }
    
    /// Get iterator over all builtin commands (name, description)
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.commands.iter()
    }
    
    /// Register all standard built-in commands
    fn register_standard_builtins(&mut self) {
        // Standard shell builtins
        self.register("cd".to_string(), "Change directory".to_string());
        self.register("pwd".to_string(), "Print working directory".to_string());
        self.register("ls".to_string(), "List directory contents".to_string());
        self.register("cp".to_string(), "Copy files or directories".to_string());
        self.register("mv".to_string(), "Move files or directories".to_string());
        self.register("rm".to_string(), "Remove files or directories".to_string());
        self.register("mkdir".to_string(), "Create directories".to_string());
        self.register("rmdir".to_string(), "Remove directories".to_string());
        self.register("cat".to_string(), "Display file contents".to_string());
        self.register("echo".to_string(), "Display text".to_string());
        self.register("grep".to_string(), "Search text patterns".to_string());
        self.register("find".to_string(), "Find files and directories".to_string());
        self.register("ps".to_string(), "List running processes".to_string());
        self.register("kill".to_string(), "Terminate processes".to_string());
        self.register("top".to_string(), "Display system processes".to_string());
        self.register("df".to_string(), "Display filesystem usage".to_string());
        self.register("free".to_string(), "Display memory usage".to_string());
        self.register("uptime".to_string(), "Display system uptime".to_string());
        self.register("date".to_string(), "Display or set date".to_string());
        self.register("cal".to_string(), "Display calendar".to_string());
        self.register("which".to_string(), "Locate commands".to_string());
        self.register("whereis".to_string(), "Locate binary, source, manual".to_string());
        self.register("history".to_string(), "Command history".to_string());
        self.register("alias".to_string(), "Create command aliases".to_string());
        self.register("unalias".to_string(), "Remove command aliases".to_string());
        self.register("export".to_string(), "Set environment variables".to_string());
        self.register("unset".to_string(), "Unset variables".to_string());
        self.register("env".to_string(), "Display environment".to_string());
        self.register("set".to_string(), "Set shell options".to_string());
        self.register("exit".to_string(), "Exit shell".to_string());
        self.register("help".to_string(), "Display help information".to_string());
    }
}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global builtin registry instance
static BUILTIN_REGISTRY: std::sync::LazyLock<BuiltinRegistry> = std::sync::LazyLock::new(|| {
    BuiltinRegistry::new()
});

/// Get reference to the global builtin registry
pub fn get_builtin_registry() -> &'static BuiltinRegistry {
    &BUILTIN_REGISTRY
}
