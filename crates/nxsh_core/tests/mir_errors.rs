use nxsh_core::mir::{MirExecutor, MirValue, lower::Lowerer};
use nxsh_parser::ast::{AstNode, NumberType, AssignmentOperator};

// Helper to build simple program from sequence of nodes
fn run(nodes: Vec<AstNode>) -> Result<MirValue, nxsh_core::mir::MirError> {
    let ast = AstNode::Program(nodes);
    let prog = Lowerer::new().lower_program(&ast);
    let mut exec = MirExecutor::new();
    exec.execute_main(&prog)
}

#[test]
fn error_div_by_zero() {
    use nxsh_parser::ast::BinaryOperator as BO;
    let expr = AstNode::BinaryExpression { left: Box::new(AstNode::NumberLiteral { value: "1", number_type: NumberType::Decimal }), operator: BO::Divide, right: Box::new(AstNode::NumberLiteral { value: "0", number_type: NumberType::Decimal }) };
    let closure = AstNode::Closure { params: vec![], body: Box::new(AstNode::Return(Some(Box::new(expr)))), captures: vec![], is_async: false };
    let call = AstNode::FunctionCall { name: Box::new(closure), args: vec![], is_async: false, generics: vec![] };
    let result = run(vec![call]);
    assert!(matches!(result, Err(nxsh_core::mir::MirError::DivByZero)), "expected DivByZero got {:?}", result);
}

#[test]
fn error_type_mismatch_arith() {
    use nxsh_parser::ast::BinaryOperator as BO;
    // "a" + 1 -> type mismatch
    let expr = AstNode::BinaryExpression { left: Box::new(AstNode::StringLiteral { value: "a", quote_type: nxsh_parser::ast::QuoteType::Double }), operator: BO::Add, right: Box::new(AstNode::NumberLiteral { value: "1", number_type: NumberType::Decimal }) };
    let closure = AstNode::Closure { params: vec![], body: Box::new(AstNode::Return(Some(Box::new(expr)))), captures: vec![], is_async: false };
    let call = AstNode::FunctionCall { name: Box::new(closure), args: vec![], is_async: false, generics: vec![] };
    let result = run(vec![call]);
    assert!(matches!(result, Err(nxsh_core::mir::MirError::TypeMismatch(_))), "expected TypeMismatch got {:?}", result);
}

#[test]
fn error_regex_compile() {
    use nxsh_parser::ast::{BinaryOperator as BO, QuoteType};
    // invalid regex pattern
    let assign_s = AstNode::VariableAssignment { name: "s", operator: AssignmentOperator::Assign, value: Box::new(AstNode::StringLiteral { value: "text", quote_type: QuoteType::Double }), is_local: false, is_export: false, is_readonly: false };
    // pattern with unclosed (
    let assign_p = AstNode::VariableAssignment { name: "p", operator: AssignmentOperator::Assign, value: Box::new(AstNode::StringLiteral { value: "(abc", quote_type: QuoteType::Double }), is_local: false, is_export: false, is_readonly: false };
    let match_expr = AstNode::BinaryExpression { left: Box::new(AstNode::Word("s")), operator: BO::Match, right: Box::new(AstNode::Word("p")) };
    let closure_body = AstNode::Return(Some(Box::new(match_expr)));
    let assign_f = AstNode::VariableAssignment { name: "f", operator: AssignmentOperator::Assign, value: Box::new(AstNode::Closure { params: vec![], body: Box::new(closure_body), captures: vec!["s","p"], is_async: false }), is_local: false, is_export: false, is_readonly: false };
    let call_f = AstNode::FunctionCall { name: Box::new(AstNode::Word("f")), args: vec![], is_async: false, generics: vec![] };
    let result = run(vec![assign_s, assign_p, assign_f, call_f]);
    assert!(matches!(result, Err(nxsh_core::mir::MirError::RegexCompile(_, _))), "expected RegexCompile error got {:?}", result);
}

#[test]
fn error_regex_type_mismatch() {
    use nxsh_parser::ast::{BinaryOperator as BO, QuoteType};
    // left string, right integer pattern -> mismatch
    let assign_s = AstNode::VariableAssignment { name: "s", operator: AssignmentOperator::Assign, value: Box::new(AstNode::StringLiteral { value: "abc", quote_type: QuoteType::Double }), is_local: false, is_export: false, is_readonly: false };
    let assign_p = AstNode::VariableAssignment { name: "p", operator: AssignmentOperator::Assign, value: Box::new(AstNode::NumberLiteral { value: "1", number_type: NumberType::Decimal }), is_local: false, is_export: false, is_readonly: false };
    let match_expr = AstNode::BinaryExpression { left: Box::new(AstNode::Word("s")), operator: BO::Match, right: Box::new(AstNode::Word("p")) };
    let closure_body = AstNode::Return(Some(Box::new(match_expr)));
    let assign_f = AstNode::VariableAssignment { name: "f", operator: AssignmentOperator::Assign, value: Box::new(AstNode::Closure { params: vec![], body: Box::new(closure_body), captures: vec!["s","p"], is_async: false }), is_local: false, is_export: false, is_readonly: false };
    let call_f = AstNode::FunctionCall { name: Box::new(AstNode::Word("f")), args: vec![], is_async: false, generics: vec![] };
    let result = run(vec![assign_s, assign_p, assign_f, call_f]);
    assert!(matches!(result, Err(nxsh_core::mir::MirError::TypeMismatch(_))), "expected regex TypeMismatch got {:?}", result);
}
