use nxsh_plugin::manager::PluginManager;
use nxsh_plugin::manager::PluginMetadata;

#[test]
fn rejects_when_capabilities_required_and_missing() {
    std::env::set_var("NXSH_CAP_MANIFEST_REQUIRED", "1");
    let manager = PluginManager::new();
    let invalid = PluginMetadata {
        name: "test_plugin".into(),
        version: "0.0.0".into(),
        description: None,
        author: None,
        capabilities: vec![],
    };
    assert!(manager.validate_plugin_metadata(&invalid).is_err());
    std::env::remove_var("NXSH_CAP_MANIFEST_REQUIRED");
}

#[test]
fn accepts_when_capabilities_present() {
    std::env::set_var("NXSH_CAP_MANIFEST_REQUIRED", "1");
    let manager = PluginManager::new();
    let valid = PluginMetadata {
        name: "test_plugin".into(),
        version: "0.0.0".into(),
        description: None,
        author: None,
        capabilities: vec!["file_read".into()],
    };
    assert!(manager.validate_plugin_metadata(&valid).is_ok());
    std::env::remove_var("NXSH_CAP_MANIFEST_REQUIRED");
}


