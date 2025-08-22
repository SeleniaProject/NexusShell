//! Comprehensive unit tests for AWK implementation
//! 
//! This module provides exhaustive test coverage for all AWK functionality
//! including user-defined functions, mathematical operations, field processing,
//! regular expressions, control flow, and built-in functions.

#[cfg(test)]
mod comprehensive_awk_tests {
    use super::super::*;
    use std::collections::HashMap;

    /// Helper function to create a basic AWK context for testing
    fn create_test_context() -> AwkContext {
        let options = AwkOptions {
            field_separator: " ".to_string(),
            output_field_separator: " ".to_string(),
            record_separator: "\n".to_string(),
            output_record_separator: "\n".to_string(),
            program: String::new(),
            program_file: None,
            variables: HashMap::new(),
            files: vec![],
        };
        let mut ctx = AwkContext::new(&options);
        ctx.split_fields("alpha beta gamma delta");
        ctx
    }

    /// Helper function to create context with custom field data
    fn create_context_with_fields(line: &str) -> AwkContext {
        let mut ctx = create_test_context();
        ctx.split_fields(line);
        ctx
    }

    #[test]
    fn test_user_defined_function_creation() {
        let func = AwkFunction {
            name: "double".to_string(),
            parameters: vec!["x".to_string()],
            body: AwkAction::Return(Some(AwkExpression::Binary(
                Box::new(AwkExpression::Variable("x".to_string())),
                BinaryOp::Mul,
                Box::new(AwkExpression::Number(2.0)),
            ))),
            local_vars: vec!["x".to_string()],
        };
        
        assert_eq!(func.name, "double");
        assert_eq!(func.parameters.len(), 1);
        assert_eq!(func.parameters[0], "x");
    }

    #[test]
    fn test_dynamic_field_references() {
    let mut ctx = create_context_with_fields("one two three four five");
        
        // Test $(NF-1) - should be "four" (field 4)
        let nf_minus_one = AwkExpression::Field(Box::new(AwkExpression::Binary(
            Box::new(AwkExpression::Variable("NF".to_string())),
            BinaryOp::Sub,
            Box::new(AwkExpression::Number(1.0)),
        )));
        
    let result = evaluate_awk_expression(&nf_minus_one, &mut ctx).unwrap();
        assert_eq!(to_string_val(&result), "four");
    }

    #[test]
    fn test_ternary_operator() {
    let mut ctx = create_test_context();
        
        // Test: NF > 3 ? "many" : "few"
        let ternary = AwkExpression::Ternary(
            Box::new(AwkExpression::Binary(
                Box::new(AwkExpression::Variable("NF".to_string())),
                BinaryOp::Gt,
                Box::new(AwkExpression::Number(3.0)),
            )),
            Box::new(AwkExpression::String("many".to_string())),
            Box::new(AwkExpression::String("few".to_string())),
        );
        
    let result = evaluate_awk_expression(&ternary, &mut ctx).unwrap();
        assert_eq!(to_string_val(&result), "many"); // NF is 4
    }

    #[test]
    fn test_increment_decrement_operators() {
        let mut ctx = create_test_context();
        ctx.variables.insert("counter".to_string(), AwkValue::Number(5.0));
        
        // Test pre-increment: ++counter
        let pre_inc = AwkExpression::PreIncrement("counter".to_string());
    let result = evaluate_awk_expression(&pre_inc, &mut ctx).unwrap();
        assert_eq!(to_number(&result), 6.0);
        
        // Test post-increment: counter++
        let post_inc = AwkExpression::PostIncrement("counter".to_string());
    let result = evaluate_awk_expression(&post_inc, &mut ctx).unwrap();
        assert_eq!(to_number(&result), 6.0); // Returns old value
        
        // Verify counter was incremented
        let counter_val = ctx.variables.get("counter").unwrap();
        assert_eq!(to_number(counter_val), 7.0);
    }

    #[test]
    fn test_power_operator() {
    let mut ctx = create_test_context();
        
        // Test: 2 ** 3 = 8
        let power_expr = AwkExpression::Binary(
            Box::new(AwkExpression::Number(2.0)),
            BinaryOp::Power,
            Box::new(AwkExpression::Number(3.0)),
        );
        
    let result = evaluate_awk_expression(&power_expr, &mut ctx).unwrap();
        assert_eq!(to_number(&result), 8.0);
    }

    #[test]
    fn test_regex_match_operators() {
    let mut ctx = create_context_with_fields("hello world 123");
        
        // Test: $1 ~ /^h.*o$/
        let regex = Regex::new(r"^h.*o$").unwrap();
        let match_expr = AwkExpression::Match(
            Box::new(AwkExpression::Field(Box::new(AwkExpression::Number(1.0)))),
            regex,
        );
        
    let result = evaluate_awk_expression(&match_expr, &mut ctx).unwrap();
        assert_eq!(to_number(&result), 1.0); // Should match "hello"
    }

    #[test]
    fn test_in_operator_for_arrays() {
        let mut ctx = create_test_context();
        
        // Create associative array: arr["key"] = "value"
        let mut map = HashMap::new();
        map.insert("key".to_string(), AwkValue::String("value".to_string()));
        ctx.variables.insert("arr".to_string(), AwkValue::Map(map));
        
        // Test: "key" in arr
        let in_expr = AwkExpression::Binary(
            Box::new(AwkExpression::String("key".to_string())),
            BinaryOp::In,
            Box::new(AwkExpression::Variable("arr".to_string())),
        );
        
    let result = evaluate_awk_expression(&in_expr, &mut ctx).unwrap();
        assert_eq!(to_number(&result), 1.0); // Should be true
    }

    #[test]
    fn test_field_assignment() {
        let mut ctx = create_context_with_fields("one two three");
        
        // Test: $2 = "modified"
        let field_assign = AwkAction::FieldAssignment(
            AwkExpression::Number(2.0),
            AwkExpression::String("modified".to_string()),
        );
        
        execute_awk_action(&field_assign, &mut ctx).unwrap();
        assert_eq!(ctx.get_field(2), "modified");
    }

    #[test]
    fn test_for_in_loop() {
        let mut ctx = create_test_context();
        
        // Create array with multiple elements
        let mut map = HashMap::new();
        map.insert("a".to_string(), AwkValue::Number(1.0));
        map.insert("b".to_string(), AwkValue::Number(2.0));
        map.insert("c".to_string(), AwkValue::Number(3.0));
        ctx.variables.insert("arr".to_string(), AwkValue::Map(map));
        
        // Test: for (key in arr) sum += arr[key]
        ctx.variables.insert("sum".to_string(), AwkValue::Number(0.0));
        
        let for_in_action = AwkAction::ForIn(
            "key".to_string(),
            "arr".to_string(),
            Box::new(AwkAction::Assignment(
                "sum".to_string(),
                AwkExpression::Binary(
                    Box::new(AwkExpression::Variable("sum".to_string())),
                    BinaryOp::Add,
                    Box::new(AwkExpression::Index(
                        Box::new(AwkExpression::Variable("arr".to_string())),
                        Box::new(AwkExpression::Variable("key".to_string())),
                    )),
                ),
            )),
        );
        
        execute_awk_action(&for_in_action, &mut ctx).unwrap();
        
        let sum = ctx.variables.get("sum").unwrap();
        assert_eq!(to_number(sum), 6.0); // 1 + 2 + 3 = 6
    }

    #[test]
    fn test_mathematical_functions() {
    let mut ctx = create_test_context();
        
        // Test sin(π/2) ≁E1.0
        let sin_expr = AwkExpression::Function(
            "sin".to_string(),
            vec![AwkExpression::Binary(
                Box::new(AwkExpression::Number(std::f64::consts::PI)),
                BinaryOp::Div,
                Box::new(AwkExpression::Number(2.0)),
            )],
        );
        
    let result = evaluate_awk_expression(&sin_expr, &mut ctx).unwrap();
        assert!((to_number(&result) - 1.0).abs() < 1e-10);
        
        // Test sqrt(16) = 4
        let sqrt_expr = AwkExpression::Function(
            "sqrt".to_string(),
            vec![AwkExpression::Number(16.0)],
        );
        
    let result = evaluate_awk_expression(&sqrt_expr, &mut ctx).unwrap();
        assert_eq!(to_number(&result), 4.0);
    }

    #[test]
    fn test_string_functions() {
    let mut ctx = create_test_context();
        
        // Test length("hello") = 5
        let length_expr = AwkExpression::Function(
            "length".to_string(),
            vec![AwkExpression::String("hello".to_string())],
        );
        
    let result = evaluate_awk_expression(&length_expr, &mut ctx).unwrap();
        assert_eq!(to_number(&result), 5.0);
        
        // Test substr("hello", 2, 3) = "ell"
        let substr_expr = AwkExpression::Function(
            "substr".to_string(),
            vec![
                AwkExpression::String("hello".to_string()),
                AwkExpression::Number(2.0),
                AwkExpression::Number(3.0),
            ],
        );
        
    let result = evaluate_awk_expression(&substr_expr, &mut ctx).unwrap();
        assert_eq!(to_string_val(&result), "ell");
        
        // Test index("hello", "ll") = 3
        let index_expr = AwkExpression::Function(
            "index".to_string(),
            vec![
                AwkExpression::String("hello".to_string()),
                AwkExpression::String("ll".to_string()),
            ],
        );
        
    let result = evaluate_awk_expression(&index_expr, &mut ctx).unwrap();
        assert_eq!(to_number(&result), 3.0);
    }

    #[test]
    fn test_split_function() {
        let mut ctx = create_test_context();
        
        // Test split("a,b,c", arr, ",") = 3
        let split_expr = AwkExpression::Function(
            "split".to_string(),
            vec![
                AwkExpression::String("a,b,c".to_string()),
                AwkExpression::Variable("arr".to_string()),
                AwkExpression::String(",".to_string()),
            ],
        );
        
    let result = evaluate_awk_expression(&split_expr, &mut ctx).unwrap();
        assert_eq!(to_number(&result), 3.0);
        
        // Verify array was created correctly
        if let Some(AwkValue::Map(map)) = ctx.variables.get("arr") {
            assert_eq!(map.len(), 3);
            assert_eq!(to_string_val(map.get("1").unwrap()), "a");
            assert_eq!(to_string_val(map.get("2").unwrap()), "b");
            assert_eq!(to_string_val(map.get("3").unwrap()), "c");
        } else {
            panic!("split() did not create array");
        }
    }

    #[test]
    fn test_gsub_function() {
        let mut ctx = create_test_context();
        ctx.variables.insert("text".to_string(), AwkValue::String("hello hello hello".to_string()));
        
        // Test gsub(/hello/, "hi", text)
        let gsub_expr = AwkExpression::Function(
            "gsub".to_string(),
            vec![
                AwkExpression::String("hello".to_string()),
                AwkExpression::String("hi".to_string()),
                AwkExpression::Variable("text".to_string()),
            ],
        );
        
    let result = evaluate_awk_expression(&gsub_expr, &mut ctx).unwrap();
        assert_eq!(to_number(&result), 3.0); // Should replace 3 occurrences
        
        // Verify text was modified
        let text_val = ctx.variables.get("text").unwrap();
        assert_eq!(to_string_val(text_val), "hi hi hi");
    }

    #[test]
    fn test_random_functions() {
        let mut ctx = create_test_context();
        
        // Test srand(42) - set seed
        let srand_expr = AwkExpression::Function(
            "srand".to_string(),
            vec![AwkExpression::Number(42.0)],
        );
        
    evaluate_awk_expression(&srand_expr, &mut ctx).unwrap();
        assert_eq!(ctx.random_seed, 42);
        
        // Test rand() - should return value between 0 and 1
        let rand_expr = AwkExpression::Function("rand".to_string(), vec![]);
    let result = evaluate_awk_expression(&rand_expr, &mut ctx).unwrap();
        let rand_val = to_number(&result);
        assert!(rand_val >= 0.0 && rand_val < 1.0);
    }

    #[test]
    fn test_system_function() {
    let mut ctx = create_test_context();
        
        // Test system("echo hello") - should execute command
        let system_expr = AwkExpression::Function(
            "system".to_string(),
            vec![AwkExpression::String("echo hello".to_string())],
        );
        
    let result = evaluate_awk_expression(&system_expr, &mut ctx).unwrap();
        // Should return exit code (0 for success on most systems)
        assert_eq!(to_number(&result), 0.0);
    }

    #[test]
    fn test_break_continue_statements() {
        let mut ctx = create_test_context();
        ctx.variables.insert("i".to_string(), AwkValue::Number(0.0));
        ctx.variables.insert("sum".to_string(), AwkValue::Number(0.0));
        
        // Test while loop with break
        let while_action = AwkAction::While(
            AwkExpression::Binary(
                Box::new(AwkExpression::Variable("i".to_string())),
                BinaryOp::Lt,
                Box::new(AwkExpression::Number(10.0)),
            ),
            Box::new(AwkAction::Block(vec![
                AwkAction::Assignment(
                    "i".to_string(),
                    AwkExpression::Binary(
                        Box::new(AwkExpression::Variable("i".to_string())),
                        BinaryOp::Add,
                        Box::new(AwkExpression::Number(1.0)),
                    ),
                ),
                AwkAction::If(
                    AwkExpression::Binary(
                        Box::new(AwkExpression::Variable("i".to_string())),
                        BinaryOp::Eq,
                        Box::new(AwkExpression::Number(5.0)),
                    ),
                    Box::new(AwkAction::Break),
                    None,
                ),
                AwkAction::Assignment(
                    "sum".to_string(),
                    AwkExpression::Binary(
                        Box::new(AwkExpression::Variable("sum".to_string())),
                        BinaryOp::Add,
                        Box::new(AwkExpression::Variable("i".to_string())),
                    ),
                ),
            ])),
        );
        
        execute_awk_action(&while_action, &mut ctx).unwrap();
        
        let i_val = ctx.variables.get("i").unwrap();
        assert_eq!(to_number(i_val), 5.0); // Loop should break at i=5
    }

    #[test]
    fn test_printf_formatting() {
    let mut ctx = create_test_context();
        
        // Test various printf format specifiers
        let test_cases = vec![
            ("%d", vec![AwkExpression::Number(42.0)], "42"),
            ("%5d", vec![AwkExpression::Number(42.0)], "   42"),
            ("%-5d", vec![AwkExpression::Number(42.0)], "42   "),
            ("%05d", vec![AwkExpression::Number(42.0)], "00042"),
            ("%.2f", vec![AwkExpression::Number(3.14159)], "3.14"),
            ("%s", vec![AwkExpression::String("hello".to_string())], "hello"),
            ("%10s", vec![AwkExpression::String("hello".to_string())], "     hello"),
            ("%-10s", vec![AwkExpression::String("hello".to_string())], "hello     "),
            ("%c", vec![AwkExpression::Number(65.0)], "A"),
            ("%x", vec![AwkExpression::Number(255.0)], "ff"),
            ("%X", vec![AwkExpression::Number(255.0)], "FF"),
            ("%o", vec![AwkExpression::Number(8.0)], "10"),
        ];
        
        for (format, args, expected) in test_cases {
            let result = format_awk_printf(format, &args, &mut ctx).unwrap();
            assert_eq!(result, expected, "Failed for format: {}", format);
        }
    }

    #[test]
    fn test_complex_expression_evaluation() {
    let mut ctx = create_context_with_fields("10 20 30");
        
        // Test complex expression: ($1 + $2) * $3 / 2
        let complex_expr = AwkExpression::Binary(
            Box::new(AwkExpression::Binary(
                Box::new(AwkExpression::Binary(
                    Box::new(AwkExpression::Field(Box::new(AwkExpression::Number(1.0)))),
                    BinaryOp::Add,
                    Box::new(AwkExpression::Field(Box::new(AwkExpression::Number(2.0)))),
                )),
                BinaryOp::Mul,
                Box::new(AwkExpression::Field(Box::new(AwkExpression::Number(3.0)))),
            )),
            BinaryOp::Div,
            Box::new(AwkExpression::Number(2.0)),
        );
        
    let result = evaluate_awk_expression(&complex_expr, &mut ctx).unwrap();
        assert_eq!(to_number(&result), 450.0); // (10 + 20) * 30 / 2 = 450
    }

    #[test]
    fn test_pattern_matching() {
    let mut ctx = create_context_with_fields("test line with pattern");
        
        // Test regex pattern matching
        let regex = Regex::new(r".*pattern.*").unwrap();
        let pattern = AwkPattern::Regex(regex);
        
    let matches = match_awk_pattern(&pattern, &mut ctx, "test line with pattern").unwrap();
        assert!(matches);
        
    let no_match = match_awk_pattern(&pattern, &mut ctx, "no match here").unwrap();
        assert!(!no_match);
    }

    #[test]
    fn test_range_patterns() {
    let mut ctx = create_test_context();
        
        // Test range pattern: /start/,/end/
        let start_regex = Regex::new(r"start").unwrap();
        let end_regex = Regex::new(r"end").unwrap();
        let range_pattern = AwkPattern::Range(
            Box::new(AwkPattern::Regex(start_regex)),
            Box::new(AwkPattern::Regex(end_regex)),
        );
        
        // This would typically be tested in the context of processing multiple lines
        // For now, just verify the pattern structure is correct
        match range_pattern {
            AwkPattern::Range(start, end) => {
                assert!(matches!(**start, AwkPattern::Regex(_)));
                assert!(matches!(**end, AwkPattern::Regex(_)));
            }
            _ => panic!("Expected range pattern"),
        }
    }

    #[test]
    fn test_field_separator_handling() {
        let options = AwkOptions {
            field_separator: ",".to_string(),
            output_field_separator: " ".to_string(),
            record_separator: "\n".to_string(),
            output_record_separator: "\n".to_string(),
            program: String::new(),
            program_file: None,
            variables: HashMap::new(),
            files: vec![],
        };
        
        let mut ctx = AwkContext::new(&options);
        ctx.split_fields("a,b,c,d");
        
        assert_eq!(ctx.get_field(1), "a");
        assert_eq!(ctx.get_field(2), "b");
        assert_eq!(ctx.get_field(3), "c");
        assert_eq!(ctx.get_field(4), "d");
        assert_eq!(ctx.nf, 4);
    }

    #[test]
    fn test_uninitialized_variables() {
        let ctx = create_test_context();
        
        // Test accessing uninitialized variable
        let var_expr = AwkExpression::Variable("nonexistent".to_string());
    let result = evaluate_awk_expression(&var_expr, &mut ctx).unwrap();
        
        // Uninitialized variables should default to empty string
        assert_eq!(to_string_val(&result), "");
        assert_eq!(to_number(&result), 0.0);
    }

    #[test]
    fn test_type_coercion() {
    let mut ctx = create_test_context();
        
        // Test string to number coercion
        let str_num = AwkValue::String("42.5".to_string());
        assert_eq!(to_number(&str_num), 42.5);
        
        // Test number to string coercion
        let num_str = AwkValue::Number(123.0);
        assert_eq!(to_string_val(&num_str), "123");
        
        // Test number with decimal to string
        let decimal_str = AwkValue::Number(123.45);
        assert_eq!(to_string_val(&decimal_str), "123.45");
    }

    #[test]
    fn test_truthiness() {
        // Test various values for truthiness
        assert!(is_truthy(&AwkValue::Number(1.0)));
        assert!(is_truthy(&AwkValue::Number(-1.0)));
        assert!(!is_truthy(&AwkValue::Number(0.0)));
        
        assert!(is_truthy(&AwkValue::String("hello".to_string())));
        assert!(!is_truthy(&AwkValue::String("".to_string())));
        
        assert!(!is_truthy(&AwkValue::Uninitialized));
    }

    #[test]
    fn test_error_handling() {
        let ctx = create_test_context();
        
        // Test division by zero
        let div_zero = AwkExpression::Binary(
            Box::new(AwkExpression::Number(1.0)),
            BinaryOp::Div,
            Box::new(AwkExpression::Number(0.0)),
        );
        
    let result = evaluate_awk_expression(&div_zero, &mut ctx).unwrap();
        // Should handle division by zero gracefully (result is infinity)
        assert!(to_number(&result).is_infinite());
    }

    #[test]
    fn test_regex_not_match_operator() {
    let mut ctx = create_context_with_fields("hello world 123");

        // Test: $1 !~ /^z/  => should be true (1.0)
        let not_match_expr = AwkExpression::Binary(
            Box::new(AwkExpression::Field(Box::new(AwkExpression::Number(1.0)))),
            BinaryOp::NotMatch,
            Box::new(AwkExpression::String("^z".to_string())),
        );

    let result = evaluate_awk_expression(&not_match_expr, &mut ctx).unwrap();
        assert_eq!(to_number(&result), 1.0);
    }

    #[test]
    fn test_in_operator_for_arrays_negative() {
        let mut ctx = create_test_context();

        // Create associative array: arr["key1"] = 1
        let mut map = std::collections::HashMap::new();
        map.insert("key1".to_string(), AwkValue::Number(1.0));
        ctx.variables.insert("arr".to_string(), AwkValue::Map(map));

        // Test: "missing" in arr => 0.0 (false)
        let in_expr = AwkExpression::Binary(
            Box::new(AwkExpression::String("missing".to_string())),
            BinaryOp::In,
            Box::new(AwkExpression::Variable("arr".to_string())),
        );

    let result = evaluate_awk_expression(&in_expr, &mut ctx).unwrap();
        assert_eq!(to_number(&result), 0.0);
    }
}

