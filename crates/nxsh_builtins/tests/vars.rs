use nxsh_builtins::vars::{declare_cli, let_cli, printf_cli};
use nxsh_core::context::ShellContext;

#[test]
fn let_addition() {
    let ctx = ShellContext::new();
    let_cli(&["a=1+1".into()], &ctx).unwrap();
    assert_eq!(ctx.get_var("a").unwrap(), "2");
}

#[test]
fn let_plus_equal() {
    let ctx = ShellContext::new();
    let_cli(&["a=1".into()], &ctx).unwrap();
    let_cli(&["a += 2".into()], &ctx).unwrap();
    assert_eq!(ctx.get_var("a").unwrap(), "3");
}

#[test]
fn declare_assoc() {
    let ctx = ShellContext::new();
    declare_cli(&["-A".into(), "myarr".into()], &ctx).unwrap();
    assert_eq!(ctx.get_var("myarr").unwrap(), "__assoc_array__");
}

#[test]
fn printf_hex() {
    // For now, just test that printf_cli doesn't crash
    let result = printf_cli(&["%08x\n".into(), "255".into()]);
    assert!(result.is_ok());
}
