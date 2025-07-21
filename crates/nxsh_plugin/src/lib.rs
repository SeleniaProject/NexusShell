//! WASI-based plugin support for NexusShell.

pub fn initialize() {
    // In the future this will load and instantiate WASI modules.
    println!("Plugin subsystem initialized (stub)");
}

pub mod json;
pub mod registrar;
pub mod loader;
pub mod key;
pub mod remote; 