use nxsh_plugin::{PluginManager, PluginMetadata};
use std::collections::HashMap;

fn base_meta() -> PluginMetadata {
    PluginMetadata {
        name: "test_plugin".into(),
        version: "0.0.0".into(),
        description: "desc".into(),
        author: "author".into(),
        license: "MIT".into(),
        homepage: None,
        repository: None,
        keywords: vec![],
        categories: vec![],
        dependencies: HashMap::new(),
        capabilities: vec![],
        exports: vec!["main".into()],
        min_nexus_version: "0.0.0".into(),
        max_nexus_version: None,
    }
}

#[test]
fn rejects_when_capabilities_required_and_missing() {
    std::env::set_var("NXSH_CAP_MANIFEST_REQUIRED", "1");
    let manager = PluginManager::new();
    let invalid = base_meta();
    assert!(manager.validate_plugin_metadata(&invalid).is_err());
    std::env::remove_var("NXSH_CAP_MANIFEST_REQUIRED");
}

#[test]
fn accepts_when_capabilities_present() {
    std::env::set_var("NXSH_CAP_MANIFEST_REQUIRED", "1");
    let manager = PluginManager::new();
    let mut valid = base_meta();
    valid.capabilities = vec!["file_read".into()];
    assert!(manager.validate_plugin_metadata(&valid).is_ok());
    std::env::remove_var("NXSH_CAP_MANIFEST_REQUIRED");
}


