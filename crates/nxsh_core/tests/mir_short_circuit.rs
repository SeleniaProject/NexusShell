use nxsh_core::mir::{lower::Lowerer, MirExecutor, MirValue};
use nxsh_parser::ast::{AssignmentOperator, AstNode, BinaryOperator, NumberType};

fn run_prog(nodes: Vec<AstNode>) -> MirValue {
    let ast = AstNode::Program(nodes);
    let prog = Lowerer::new().lower_program(&ast);
    let mut exec = MirExecutor::new();
    exec.execute_main(&prog).expect("exec main")
}

#[test]
fn and_short_circuits() {
    // left false => right side must not evaluate (we simulate side effect by dividing by zero if evaluated)
    let assign_a = AstNode::VariableAssignment {
        name: "a",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::NumberLiteral {
            value: "0",
            number_type: NumberType::Decimal,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    // (0 == 1) && (1 / 0 == 0)  -- right would panic if evaluated; expect false
    let left_cmp = AstNode::BinaryExpression {
        left: Box::new(AstNode::NumberLiteral {
            value: "0",
            number_type: NumberType::Decimal,
        }),
        operator: BinaryOperator::Equal,
        right: Box::new(AstNode::NumberLiteral {
            value: "1",
            number_type: NumberType::Decimal,
        }),
    };
    let div = AstNode::BinaryExpression {
        left: Box::new(AstNode::NumberLiteral {
            value: "1",
            number_type: NumberType::Decimal,
        }),
        operator: BinaryOperator::Divide,
        right: Box::new(AstNode::NumberLiteral {
            value: "0",
            number_type: NumberType::Decimal,
        }),
    };
    let right_cmp = AstNode::BinaryExpression {
        left: Box::new(div),
        operator: BinaryOperator::Equal,
        right: Box::new(AstNode::NumberLiteral {
            value: "0",
            number_type: NumberType::Decimal,
        }),
    };
    let and_expr = AstNode::BinaryExpression {
        left: Box::new(left_cmp),
        operator: BinaryOperator::LogicalAnd,
        right: Box::new(right_cmp),
    };
    let ret = AstNode::Return(Some(Box::new(and_expr)));
    let program = [
        assign_a,
        AstNode::Closure {
            params: vec![],
            body: Box::new(ret),
            captures: vec![],
            is_async: false,
        },
    ];
    // call closure inline
    let call = AstNode::FunctionCall {
        name: Box::new(program.last().unwrap().clone()),
        args: vec![],
        is_async: false,
        generics: vec![],
    };
    let result = run_prog(vec![program[0].clone(), call]);
    assert_eq!(result, MirValue::Boolean(false));
}

#[test]
fn or_short_circuits() {
    // left true => right side (division by zero) must not evaluate
    let left_true = AstNode::BinaryExpression {
        left: Box::new(AstNode::NumberLiteral {
            value: "1",
            number_type: NumberType::Decimal,
        }),
        operator: BinaryOperator::Equal,
        right: Box::new(AstNode::NumberLiteral {
            value: "1",
            number_type: NumberType::Decimal,
        }),
    };
    let div = AstNode::BinaryExpression {
        left: Box::new(AstNode::NumberLiteral {
            value: "1",
            number_type: NumberType::Decimal,
        }),
        operator: BinaryOperator::Divide,
        right: Box::new(AstNode::NumberLiteral {
            value: "0",
            number_type: NumberType::Decimal,
        }),
    };
    let right_cmp = AstNode::BinaryExpression {
        left: Box::new(div),
        operator: BinaryOperator::Equal,
        right: Box::new(AstNode::NumberLiteral {
            value: "2",
            number_type: NumberType::Decimal,
        }),
    };
    let or_expr = AstNode::BinaryExpression {
        left: Box::new(left_true),
        operator: BinaryOperator::LogicalOr,
        right: Box::new(right_cmp),
    };
    let closure = AstNode::Closure {
        params: vec![],
        body: Box::new(AstNode::Return(Some(Box::new(or_expr)))),
        captures: vec![],
        is_async: false,
    };
    let call = AstNode::FunctionCall {
        name: Box::new(closure),
        args: vec![],
        is_async: false,
        generics: vec![],
    };
    let result = run_prog(vec![call]);
    assert_eq!(result, MirValue::Boolean(true));
}
