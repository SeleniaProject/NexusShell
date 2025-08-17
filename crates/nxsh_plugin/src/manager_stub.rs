//! Minimal stub for PluginManager when plugin features are disabled.

use anyhow::Result;

#[derive(Default, Debug, Clone)]
pub struct PluginManager {}

impl PluginManager {
    pub fn new() -> Self { Self {} }
}

// Lightweight fallbacks mirroring a tiny surface if needed in callers later.
pub type PluginEventHandler = (); 
