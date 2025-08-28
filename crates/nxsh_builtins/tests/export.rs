use nxsh_builtins::export_builtin::export_cli;
use nxsh_core::context::ShellContext;

#[test]
fn export_set_and_get() {
    let _ctx = ShellContext::new();
    export_cli(&["TEST_EXPORT=42".to_string()]).unwrap();
    
    // Check if environment variable was set
    assert!(std::env::var("TEST_EXPORT").is_ok());
    assert_eq!(std::env::var("TEST_EXPORT").unwrap(), "42");
    
    // Clean up
    std::env::remove_var("TEST_EXPORT");
} 