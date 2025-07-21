use std::collections::HashMap;

/// ShellContext owns environment variables and process-wide metadata.
#[derive(Debug, Default)]
pub struct ShellContext {
    /// Environment key-value pairs.
    pub env: HashMap<String, String>,
}

impl ShellContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self { env: HashMap::new() }
    }

    /// Get environment variable or fallback to default.
    pub fn get_var(&self, key: &str) -> Option<&str> {
        self.env.get(key).map(String::as_str)
    }

    /// Set or override environment variable.
    pub fn set_var<K, V>(&mut self, key: K, val: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.env.insert(key.into(), val.into());
    }
} 