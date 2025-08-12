use nxsh_core::{Executor, ShellContext};
use nxsh_parser::Parser;

#[test]
fn generic_function_definition_and_call_monomorphizes() {
    let mut ex = Executor::new();
    let mut ctx = ShellContext::new();

    // Register a generic template for id<T>(x) = echo $x
    ctx.register_generic_function_template("id", &["T"], "#params:x", "echo $x");
    // Monomorphize for <int>
    let spec = ctx.ensure_monomorphized("id", &["int"]).expect("template missing");
    assert_eq!(spec, "id__gen_int");
    // Execute specialized function call
    let parser = Parser::new();
    let ast = parser.parse("id__gen_int 42").expect("parse failed");
    let res = ex.execute(&ast, &mut ctx).expect("execute failed");
    assert_eq!(res.exit_code, 0);
}

#[test]
fn generic_function_multiple_specializations() {
    let mut ex = Executor::new();
    let mut ctx = ShellContext::new();
    // Register template and create two specializations
    ctx.register_generic_function_template("wrap", &["T"], "#params:x", "echo [$x]");
    assert_eq!(ctx.ensure_monomorphized("wrap", &["str"]).as_deref(), Some("wrap__gen_str"));
    assert_eq!(ctx.ensure_monomorphized("wrap", &["num"]).as_deref(), Some("wrap__gen_num"));
    let parser = Parser::new();
    let ast = parser.parse("wrap__gen_str hello; wrap__gen_num 7").expect("parse failed");
    let res = ex.execute(&ast, &mut ctx).expect("execute failed");
    assert_eq!(res.exit_code, 0);
}


