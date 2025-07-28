//! Network security and firewall management for NexusShell
//!
//! This module provides network security features including firewall rules,
//! intrusion detection, and secure network communication.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use serde::{Deserialize, Serialize};

/// Basic network security manager
pub struct NetworkSecurityManager {
    security_policies: Arc<RwLock<HashMap<String, SecurityPolicy>>>,
}

/// Security policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub name: String,
    pub allow_http: bool,
    pub require_auth: bool,
    pub timeout: std::time::Duration,
}

impl NetworkSecurityManager {
    /// Create a new network security manager
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            security_policies: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Add a security policy
    pub fn add_policy(&self, policy: SecurityPolicy) -> anyhow::Result<()> {
        let mut policies = self.security_policies.write().unwrap();
        policies.insert(policy.name.clone(), policy);
        Ok(())
    }
} 