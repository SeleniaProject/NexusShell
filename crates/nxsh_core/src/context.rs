use dashmap::DashMap;
use std::sync::Arc;

/// ShellContext owns environment variables and process-wide metadata.
#[derive(Debug, Default)]
pub struct ShellContext {
    /// Environment key-value pairs.
    pub env: Arc<DashMap<String, String>>,
    pub aliases: Arc<DashMap<String, String>>,
}

impl ShellContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self {
            env: Arc::new(DashMap::new()),
            aliases: Arc::new(DashMap::new()),
        }
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

    /// Get alias value if exists.
    pub fn get_alias(&self, key: &str) -> Option<String> {
        self.aliases.get(key).map(|v| v.value().clone())
    }

    /// Set alias with cycle detection.
    pub fn set_alias<K, V>(&self, key: K, val: V) -> anyhow::Result<()>
    where
        K: Into<String>,
        V: Into<String>,
    {
        let k = key.into();
        let v = val.into();
        // Detect cycles by following chain up to len aliases
        let mut seen = std::collections::HashSet::new();
        let mut current = v.clone();
        for _ in 0..self.aliases.len() + 1 {
            if !self.aliases.contains_key(&current) {
                break;
            }
            if !seen.insert(current.clone()) {
                return Err(anyhow::anyhow!("cyclic alias detected"));
            }
            current = self.aliases.get(&current).unwrap().value().clone();
        }
        self.aliases.insert(k, v);
        Ok(())
    }

    pub fn list_aliases(&self) -> Vec<(String, String)> {
        self.aliases
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }
} 