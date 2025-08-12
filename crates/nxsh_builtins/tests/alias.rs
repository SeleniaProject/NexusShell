use nxsh_builtins::alias::AliasCommand;
use nxsh_core::{Builtin, context::ShellContext};

#[test]
fn alias_cycle_detection() {
    let mut ctx = ShellContext::new();
    let alias_cmd = AliasCommand;
    
    // First alias should succeed
    let result1 = alias_cmd.execute(&mut ctx, &["foo=bar".into()]);
    assert!(result1.is_ok());
    
    // Second alias that creates a cycle should fail
    let result2 = alias_cmd.execute(&mut ctx, &["bar=foo".into()]);
    assert!(result2.is_err());
} 