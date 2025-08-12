use std::collections::HashMap;
use anyhow::Result;
// use nxsh_core::context::ShellContext; // Temporarily disabled to avoid circular dependency

/// Command information for plugin registration
#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub plugin_name: String,
    pub usage: String,
    pub examples: Vec<String>,
}

/// Simplified context for plugin registration
pub struct PluginContext {
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

/// Builtin command trait duplicated to avoid circular dep.
pub trait Builtin {
    fn name(&self) -> &'static str;
    fn synopsis(&self) -> &'static str;
    fn invoke(&self, ctx: &mut PluginContext) -> anyhow::Result<()>; // Use PluginContext instead
}

/// Registrar passed to plugins for self-registration.
pub struct PluginRegistrar {
    builtins: HashMap<String, Box<dyn Builtin + Send + Sync>>,
    registered_commands: HashMap<String, CommandInfo>,
}

impl std::fmt::Debug for PluginRegistrar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginRegistrar")
            .field("builtin_count", &self.builtins.len())
            .finish()
    }
}

impl Default for PluginRegistrar {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistrar {
    pub fn new() -> Self { 
        Self { 
            builtins: HashMap::new(),
            registered_commands: HashMap::new(),
        } 
    }
    
    pub fn register_builtin(&mut self, b: Box<dyn Builtin + Send + Sync>) {
        self.builtins.insert(b.name().to_string(), b);
    }
    
    pub fn register_command(&self, command_info: &CommandInfo) -> Result<()> {
        // In a real implementation, this would integrate with the shell's command registry
        // For now, we just log the registration
        log::info!("Registering plugin command: {} from {}", 
                  command_info.name, command_info.plugin_name);
        Ok(())
    }
    
    pub fn builtins(&self) -> impl Iterator<Item=&Box<dyn Builtin + Send + Sync>> {
        self.builtins.values()
    }
    
    pub fn get_registered_commands(&self) -> &HashMap<String, CommandInfo> {
        &self.registered_commands
    }
} 