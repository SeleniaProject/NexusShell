use nxsh_builtins::{builtin_let, declare, printf};
use nxsh_core::context::ShellContext;

#[test]
fn let_addition() {
    let ctx = ShellContext::new();
    builtin_let(&["a=1+1".into()], &ctx).unwrap();
    assert_eq!(ctx.get_var("a").unwrap(), "2");
}

#[test]
fn let_plus_equal() {
    let ctx = ShellContext::new();
    builtin_let(&["a=1".into()], &ctx).unwrap();
    builtin_let(&["a += 2".into()], &ctx).unwrap();
    assert_eq!(ctx.get_var("a").unwrap(), "3");
}

#[test]
fn declare_assoc() {
    let ctx = ShellContext::new();
    declare(&["-A".into(), "myarr".into()], &ctx).unwrap();
    assert_eq!(ctx.get_var("myarr").unwrap(), "__assoc_array__");
}

#[test]
fn printf_hex() {
    let mut buf = Vec::new();
    {
        // redirect stdout
        use std::io::Write;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        let orig = std::mem::replace(&mut *handle, buf.clone());
    }
    printf(&["%08x\n".into(), "255".into()]).unwrap();
} 