use crate::{ShellCommandParser, ast::AstNode};

#[test]
fn test_parse_simple_closure() {
    let parser = ShellCommandParser::new();
    let src = "(x){ return x; }";
    let ast = parser.parse(src).unwrap();
    match ast {
        AstNode::Closure { params, .. } => {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "x");
        }
        other => panic!("expected Closure AST, got {:?}", other)
    }
}
