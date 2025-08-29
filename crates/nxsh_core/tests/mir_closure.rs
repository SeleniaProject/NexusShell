use nxsh_core::mir::lower::Lowerer;
use nxsh_core::mir::{MirExecutor, MirInstruction, MirValue};
use nxsh_parser::ast::{AssignmentOperator, NumberType};
use nxsh_parser::ast::{AstNode, Parameter};

#[test]
fn lower_closure_and_call() {
    // まだパーサにクロージャ構文を導入していないため、ASTを直接構築
    let closure = AstNode::Closure {
        params: vec![Parameter {
            name: "y",
            default: None,
            is_variadic: false,
        }],
        body: Box::new(AstNode::Return(Some(Box::new(AstNode::Word("y"))))),
        captures: vec![],
        is_async: false,
    };
    let program = AstNode::Program(vec![closure]);
    let prog = Lowerer::new().lower_program(&program);
    let main_fn = prog.get_function("main").expect("main function exists");
    let mut found = false;
    for block in main_fn.blocks.values() {
        for inst in &block.instructions {
            if matches!(inst, MirInstruction::ClosureCreate { .. }) {
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }
    assert!(found, "ClosureCreate instruction not found in lowered MIR");
}

#[test]
fn lower_closure_with_capture_registers() {
    // x=10; closure capturing x returning x
    let assign = AstNode::VariableAssignment {
        name: "x",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::NumberLiteral {
            value: "10",
            number_type: NumberType::Decimal,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let closure = AstNode::Closure {
        params: vec![Parameter {
            name: "y",
            default: None,
            is_variadic: false,
        }],
        body: Box::new(AstNode::Return(Some(Box::new(AstNode::Word("x"))))),
        captures: vec!["x"],
        is_async: false,
    };
    let program = AstNode::Program(vec![assign, closure]);
    let prog = Lowerer::new().lower_program(&program);
    let main_fn = prog.get_function("main").expect("main function exists");
    let mut found = false;
    for block in main_fn.blocks.values() {
        for inst in &block.instructions {
            if let MirInstruction::ClosureCreate {
                captures,
                capture_regs,
                ..
            } = inst
            {
                assert_eq!(captures.len(), 1, "expected one capture value");
                assert_eq!(capture_regs.len(), 1, "expected one capture reg");
                match &captures[0] {
                    MirValue::Register(_) => {}
                    other => panic!("expected capture to lower to register, got {other:?}"),
                }
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }
    assert!(found, "ClosureCreate with capture not found");
}

#[test]
fn lower_nested_closure_captures() {
    // x=7; outer closure capturing x; inner closure capturing x; ensure inner capture resolves to register
    let assign = AstNode::VariableAssignment {
        name: "x",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::NumberLiteral {
            value: "7",
            number_type: NumberType::Decimal,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let inner = AstNode::Closure {
        params: vec![],
        body: Box::new(AstNode::Return(Some(Box::new(AstNode::Word("x"))))),
        captures: vec!["x"],
        is_async: false,
    };
    let outer_body = AstNode::Program(vec![inner]);
    let outer = AstNode::Closure {
        params: vec![Parameter {
            name: "y",
            default: None,
            is_variadic: false,
        }],
        body: Box::new(outer_body),
        captures: vec!["x"],
        is_async: false,
    };
    let program = AstNode::Program(vec![assign, outer]);
    let prog = Lowerer::new().lower_program(&program);
    let main_fn = prog.get_function("main").expect("main function exists");
    // Find outer closure to get its body block id if needed; but we just scan all blocks for inner closure
    let mut inner_found = false;
    for block in main_fn.blocks.values() {
        for inst in &block.instructions {
            if let MirInstruction::ClosureCreate { captures, .. } = inst {
                // inner closure must have captures len 1 and capture is register if nested environment worked
                if captures.len() == 1 {
                    if let MirValue::Register(_) = &captures[0] {
                        inner_found = true;
                    }
                }
            }
        }
    }
    assert!(
        inner_found,
        "Nested inner closure with register capture not found"
    );
}

#[test]
fn execute_lowered_closure_via_variable_call() {
    // x=5; c = closure(y) { return x }; invoke c(9) => 5
    // NOTE: current lowering doesn't yet support assigning closure to name 'c'; we emulate by capturing register and calling via that register directly.
    let assign_x = AstNode::VariableAssignment {
        name: "x",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::NumberLiteral {
            value: "5",
            number_type: NumberType::Decimal,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let closure = AstNode::Closure {
        params: vec![Parameter {
            name: "y",
            default: None,
            is_variadic: false,
        }],
        body: Box::new(AstNode::Return(Some(Box::new(AstNode::Word("x"))))),
        captures: vec!["x"],
        is_async: false,
    };
    // Build program: x assignment then closure then (manual) call expression of that closure (direct node reference)
    let call = AstNode::FunctionCall {
        name: Box::new(closure.clone()), // directly call closure expression
        args: vec![AstNode::NumberLiteral {
            value: "9",
            number_type: NumberType::Decimal,
        }],
        is_async: false,
        generics: vec![],
    };
    let program = AstNode::Program(vec![assign_x, call]);
    let prog = Lowerer::new().lower_program(&program);
    let main_fn = prog.get_function("main").unwrap();
    // Scan for both create & call
    let mut saw_create = false;
    let mut saw_call = false;
    for block in main_fn.blocks.values() {
        for inst in &block.instructions {
            match inst {
                MirInstruction::ClosureCreate { .. } => saw_create = true,
                MirInstruction::ClosureCall { .. } => saw_call = true,
                _ => {}
            }
        }
    }
    assert!(
        saw_create && saw_call,
        "Expected ClosureCreate and ClosureCall when calling inline closure expression"
    );
}

#[test]
fn execute_main_with_inline_closure_returns_captured_value() {
    // x=42; inline closure(y){ return x }; call closure(7) => 42 should be final return
    let assign_x = AstNode::VariableAssignment {
        name: "x",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::NumberLiteral {
            value: "42",
            number_type: NumberType::Decimal,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let call = AstNode::FunctionCall {
        name: Box::new(AstNode::Closure {
            params: vec![Parameter {
                name: "y",
                default: None,
                is_variadic: false,
            }],
            body: Box::new(AstNode::Return(Some(Box::new(AstNode::Word("x"))))),
            captures: vec!["x"],
            is_async: false,
        }),
        args: vec![AstNode::NumberLiteral {
            value: "7",
            number_type: NumberType::Decimal,
        }],
        is_async: false,
        generics: vec![],
    };
    let program = AstNode::Program(vec![assign_x, call]);
    let prog = Lowerer::new().lower_program(&program);
    let mut exec = MirExecutor::new();
    let result = exec.execute_main(&prog).expect("execute main");
    assert_eq!(result, MirValue::Integer(42));
}

#[test]
fn execute_main_with_variable_assigned_closure_returns_captured_value() {
    // x=42; f = closure(y){ return x }; f(0) => 42
    let assign_x = AstNode::VariableAssignment {
        name: "x",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::NumberLiteral {
            value: "42",
            number_type: NumberType::Decimal,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let assign_f = AstNode::VariableAssignment {
        name: "f",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::Closure {
            params: vec![Parameter {
                name: "y",
                default: None,
                is_variadic: false,
            }],
            body: Box::new(AstNode::Return(Some(Box::new(AstNode::Word("x"))))),
            captures: vec!["x"],
            is_async: false,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let call_f = AstNode::FunctionCall {
        name: Box::new(AstNode::Word("f")),
        args: vec![AstNode::NumberLiteral {
            value: "0",
            number_type: NumberType::Decimal,
        }],
        is_async: false,
        generics: vec![],
    };
    let program = AstNode::Program(vec![assign_x, assign_f, call_f]);
    let prog = Lowerer::new().lower_program(&program);
    let mut exec = MirExecutor::new();
    let result = exec.execute_main(&prog).expect("execute main");
    assert_eq!(result, MirValue::Integer(42));
}

#[test]
fn execute_main_closure_with_arithmetic_body() {
    // x=10; y=32; f = closure(){ return x + y }; f() => 42
    let assign_x = AstNode::VariableAssignment {
        name: "x",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::NumberLiteral {
            value: "10",
            number_type: NumberType::Decimal,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let assign_y = AstNode::VariableAssignment {
        name: "y",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::NumberLiteral {
            value: "32",
            number_type: NumberType::Decimal,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let closure_body = AstNode::Return(Some(Box::new(AstNode::BinaryExpression {
        left: Box::new(AstNode::Word("x")),
        operator: nxsh_parser::ast::BinaryOperator::Add,
        right: Box::new(AstNode::Word("y")),
    })));
    let assign_f = AstNode::VariableAssignment {
        name: "f",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::Closure {
            params: vec![],
            body: Box::new(closure_body),
            captures: vec!["x", "y"],
            is_async: false,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let call_f = AstNode::FunctionCall {
        name: Box::new(AstNode::Word("f")),
        args: vec![],
        is_async: false,
        generics: vec![],
    };
    let program = AstNode::Program(vec![assign_x, assign_y, assign_f, call_f]);
    let prog = Lowerer::new().lower_program(&program);
    // sanity: ensure Add instruction present
    let main_fn = prog.get_function("main").unwrap();
    let mut saw_add = false;
    for block in main_fn.blocks.values() {
        for inst in &block.instructions {
            if let MirInstruction::Add { .. } = inst {
                saw_add = true;
            }
        }
    }
    assert!(saw_add, "Add instruction not lowered");
    let mut exec = MirExecutor::new();
    let result = exec.execute_main(&prog).expect("execute main");
    assert_eq!(result, MirValue::Integer(42));
}

#[test]
fn execute_main_closure_with_comparisons_and_logic() {
    // x=5; y=10; f=closure(){ return (x < y) && (y == 10) }; f() => true
    use nxsh_parser::ast::BinaryOperator as BO;
    let assign_x = AstNode::VariableAssignment {
        name: "x",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::NumberLiteral {
            value: "5",
            number_type: NumberType::Decimal,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let assign_y = AstNode::VariableAssignment {
        name: "y",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::NumberLiteral {
            value: "10",
            number_type: NumberType::Decimal,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    // (x < y)
    let cmp1 = AstNode::BinaryExpression {
        left: Box::new(AstNode::Word("x")),
        operator: BO::Less,
        right: Box::new(AstNode::Word("y")),
    };
    // (y == 10)
    let cmp2 = AstNode::BinaryExpression {
        left: Box::new(AstNode::Word("y")),
        operator: BO::Equal,
        right: Box::new(AstNode::NumberLiteral {
            value: "10",
            number_type: NumberType::Decimal,
        }),
    };
    // (x < y) && (y == 10)
    let and_expr = AstNode::BinaryExpression {
        left: Box::new(cmp1),
        operator: BO::LogicalAnd,
        right: Box::new(cmp2),
    };
    let closure_body = AstNode::Return(Some(Box::new(and_expr)));
    let assign_f = AstNode::VariableAssignment {
        name: "f",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::Closure {
            params: vec![],
            body: Box::new(closure_body),
            captures: vec!["x", "y"],
            is_async: false,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let call_f = AstNode::FunctionCall {
        name: Box::new(AstNode::Word("f")),
        args: vec![],
        is_async: false,
        generics: vec![],
    };
    let program = AstNode::Program(vec![assign_x, assign_y, assign_f, call_f]);
    let prog = Lowerer::new().lower_program(&program);
    // ensure LessThan / Equal / logical conjunction lowered (accept And or AndSC)
    let main_fn = prog.get_function("main").unwrap();
    let mut saw_less = false;
    let mut saw_equal = false;
    let mut saw_and = false;
    for block in main_fn.blocks.values() {
        for inst in &block.instructions {
            match inst {
                MirInstruction::LessThan { .. } => saw_less = true,
                MirInstruction::Equal { .. } => saw_equal = true,
                MirInstruction::And { .. } => saw_and = true,
                MirInstruction::AndSC { .. } => saw_and = true,
                _ => {}
            }
        }
    }
    assert!(
        saw_less && saw_equal && saw_and,
        "comparison/logic instructions not lowered"
    );
    let mut exec = MirExecutor::new();
    let result = exec.execute_main(&prog).expect("execute main");
    assert_eq!(result, MirValue::Boolean(true));
}

#[test]
fn execute_main_closure_with_power_and_bitwise() {
    // x=2; y=5; z=3; f(){ return (x ** y) & ((1 << z) - 1) } => (32 & (8-1)) = 32 & 7 = 0
    use nxsh_parser::ast::BinaryOperator as BO;
    let assign_x = AstNode::VariableAssignment {
        name: "x",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::NumberLiteral {
            value: "2",
            number_type: NumberType::Decimal,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let assign_y = AstNode::VariableAssignment {
        name: "y",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::NumberLiteral {
            value: "5",
            number_type: NumberType::Decimal,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let assign_z = AstNode::VariableAssignment {
        name: "z",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::NumberLiteral {
            value: "3",
            number_type: NumberType::Decimal,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    // x ** y
    let pow_expr = AstNode::BinaryExpression {
        left: Box::new(AstNode::Word("x")),
        operator: BO::Power,
        right: Box::new(AstNode::Word("y")),
    };
    // 1 << z
    let one = AstNode::NumberLiteral {
        value: "1",
        number_type: NumberType::Decimal,
    };
    let shift = AstNode::BinaryExpression {
        left: Box::new(one),
        operator: BO::LeftShift,
        right: Box::new(AstNode::Word("z")),
    };
    // (1 << z) - 1
    let minus_one = AstNode::BinaryExpression {
        left: Box::new(shift),
        operator: BO::Subtract,
        right: Box::new(AstNode::NumberLiteral {
            value: "1",
            number_type: NumberType::Decimal,
        }),
    };
    // (x ** y) & ((1 << z) - 1)
    let bit_and = AstNode::BinaryExpression {
        left: Box::new(pow_expr),
        operator: BO::BitwiseAnd,
        right: Box::new(minus_one),
    };
    let closure_body = AstNode::Return(Some(Box::new(bit_and)));
    let assign_f = AstNode::VariableAssignment {
        name: "f",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::Closure {
            params: vec![],
            body: Box::new(closure_body),
            captures: vec!["x", "y", "z"],
            is_async: false,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let call_f = AstNode::FunctionCall {
        name: Box::new(AstNode::Word("f")),
        args: vec![],
        is_async: false,
        generics: vec![],
    };
    let program = AstNode::Program(vec![assign_x, assign_y, assign_z, assign_f, call_f]);
    let prog = Lowerer::new().lower_program(&program);
    let mut saw_pow = false;
    let mut saw_and = false;
    let mut saw_shl = false;
    if let Some(main_fn) = prog.get_function("main") {
        for block in main_fn.blocks.values() {
            for inst in &block.instructions {
                match inst {
                    MirInstruction::Pow { .. } => saw_pow = true,
                    MirInstruction::BitAnd { .. } => saw_and = true,
                    MirInstruction::Shl { .. } => saw_shl = true,
                    _ => {}
                }
            }
        }
    }
    assert!(
        saw_pow && saw_and && saw_shl,
        "expected pow/bitand/shl lowering"
    );
    let mut exec = MirExecutor::new();
    let result = exec.execute_main(&prog).expect("execute main");
    assert_eq!(result, MirValue::Integer(0));
}

#[test]
fn execute_main_closure_with_regex_match() {
    // s="hello123"; p="^hello\\d+$"; f(){ return s =~ p }; => true
    use nxsh_parser::ast::{BinaryOperator as BO, QuoteType};
    let assign_s = AstNode::VariableAssignment {
        name: "s",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::StringLiteral {
            value: "hello123",
            quote_type: QuoteType::Double,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let assign_p = AstNode::VariableAssignment {
        name: "p",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::StringLiteral {
            value: "^hello\\d+$",
            quote_type: QuoteType::Double,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let match_expr = AstNode::BinaryExpression {
        left: Box::new(AstNode::Word("s")),
        operator: BO::Match,
        right: Box::new(AstNode::Word("p")),
    };
    let closure_body = AstNode::Return(Some(Box::new(match_expr)));
    let assign_f = AstNode::VariableAssignment {
        name: "f",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::Closure {
            params: vec![],
            body: Box::new(closure_body),
            captures: vec!["s", "p"],
            is_async: false,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let call_f = AstNode::FunctionCall {
        name: Box::new(AstNode::Word("f")),
        args: vec![],
        is_async: false,
        generics: vec![],
    };
    let program = AstNode::Program(vec![assign_s, assign_p, assign_f, call_f]);
    let prog = Lowerer::new().lower_program(&program);
    let mut saw_regex = false;
    if let Some(main_fn) = prog.get_function("main") {
        for block in main_fn.blocks.values() {
            for inst in &block.instructions {
                if let MirInstruction::RegexMatch { .. } = inst {
                    saw_regex = true;
                }
            }
        }
    }
    assert!(saw_regex, "regex match not lowered");
    let mut exec = MirExecutor::new();
    let result = exec.execute_main(&prog).expect("execute main");
    assert_eq!(result, MirValue::Boolean(true));
}

#[test]
fn execute_main_closure_with_regex_not_match() {
    // s="hello"; p="^world"; f(){ return s !~ p }; => true (not match)
    use nxsh_parser::ast::{BinaryOperator as BO, QuoteType};
    let assign_s = AstNode::VariableAssignment {
        name: "s",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::StringLiteral {
            value: "hello",
            quote_type: QuoteType::Double,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let assign_p = AstNode::VariableAssignment {
        name: "p",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::StringLiteral {
            value: "^world",
            quote_type: QuoteType::Double,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let match_expr = AstNode::BinaryExpression {
        left: Box::new(AstNode::Word("s")),
        operator: BO::NotMatch,
        right: Box::new(AstNode::Word("p")),
    };
    let closure_body = AstNode::Return(Some(Box::new(match_expr)));
    let assign_f = AstNode::VariableAssignment {
        name: "f",
        operator: AssignmentOperator::Assign,
        value: Box::new(AstNode::Closure {
            params: vec![],
            body: Box::new(closure_body),
            captures: vec!["s", "p"],
            is_async: false,
        }),
        is_local: false,
        is_export: false,
        is_readonly: false,
    };
    let call_f = AstNode::FunctionCall {
        name: Box::new(AstNode::Word("f")),
        args: vec![],
        is_async: false,
        generics: vec![],
    };
    let program = AstNode::Program(vec![assign_s, assign_p, assign_f, call_f]);
    let prog = Lowerer::new().lower_program(&program);
    let mut saw_regex = false;
    if let Some(main_fn) = prog.get_function("main") {
        for block in main_fn.blocks.values() {
            for inst in &block.instructions {
                if let MirInstruction::RegexMatch { not, .. } = inst {
                    if *not {
                        saw_regex = true;
                    }
                }
            }
        }
    }
    assert!(saw_regex, "regex not-match not lowered");
    let mut exec = MirExecutor::new();
    let result = exec.execute_main(&prog).expect("execute main");
    assert_eq!(result, MirValue::Boolean(true));
}
