use nxsh_builtins::alias;
use nxsh_core::context::ShellContext;

#[test]
fn alias_cycle_detection() {
    let ctx = ShellContext::new();
    alias(&["foo=bar".into()], &ctx).unwrap();
    alias(&["bar=foo".into()], &ctx).unwrap_err();
} 