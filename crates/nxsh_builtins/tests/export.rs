use nxsh_builtins::export;
use nxsh_core::context::ShellContext;

#[test]
fn export_set_and_get() {
    let ctx = ShellContext::new();
    export(&["TEST_EXPORT=42".into()], &ctx).unwrap();
    assert_eq!(ctx.get_var("TEST_EXPORT").unwrap(), "42");
} 