use dashmap::DashMap;
use std::sync::Arc;

/// ShellContext owns environment variables and process-wide metadata.
#[derive(Debug, Default)]
pub struct ShellContext {
    /// Environment key-value pairs.
    pub env: Arc<DashMap<String, String>>,
}

impl ShellContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self { env: Arc::new(DashMap::new()) }
    }

    /// Get environment variable or fallback to default.
    pub fn get_var(&self, key: &str) -> Option<String> {
        self.env.get(key).map(|v| v.value().clone())
    }

    /// Set or override environment variable.
    pub fn set_var<K, V>(&self, key: K, val: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.env.insert(key.into(), val.into());
    }
} 