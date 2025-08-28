use nxsh_core::mir::{MirExecutor, MirValue};

#[test]
fn test_error_handling_user_functions() {
    let mut executor = MirExecutor::new();
    
    // Test factorial with negative number
    let result = executor.call_user_function_by_name("factorial", vec![MirValue::Integer(-1)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("negative numbers not supported"));
    
    // Test factorial with non-integer
    let result = executor.call_user_function_by_name("factorial", vec![MirValue::String("abc".to_string())]);
    assert!(result.is_err());
    
    // Test max with empty arguments
    let result = executor.call_user_function_by_name("max", vec![]);
    assert!(result.is_err());
    
    // Test map with wrong argument count
    let result = executor.call_user_function_by_name("map", vec![MirValue::String("double".to_string())]);
    assert!(result.is_err());
    
    // Test unknown function
    let result = executor.call_user_function_by_name("unknown_function", vec![]);
    assert!(result.is_err());
    
    println!("✅ All error handling tests passed!");
}

#[test]
fn test_boundary_values() {
    let mut executor = MirExecutor::new();
    
    // Test factorial(0) = 1
    let result = executor.call_user_function_by_name("factorial", vec![MirValue::Integer(0)]).unwrap();
    assert_eq!(result, MirValue::Integer(1));
    
    // Test fibonacci(0) = 0
    let result = executor.call_user_function_by_name("fibonacci", vec![MirValue::Integer(0)]).unwrap();
    assert_eq!(result, MirValue::Integer(0));
    
    // Test fibonacci(1) = 1
    let result = executor.call_user_function_by_name("fibonacci", vec![MirValue::Integer(1)]).unwrap();
    assert_eq!(result, MirValue::Integer(1));
    
    // Test sum with empty array
    let result = executor.call_user_function_by_name("sum", vec![]).unwrap();
    assert_eq!(result, MirValue::Integer(0));
    
    // Test reduce with empty array and initial value
    let result = executor.call_user_function_by_name("reduce", vec![
        MirValue::String("add".to_string()),
        MirValue::Array(vec![]),
        MirValue::Integer(42)
    ]).unwrap();
    assert_eq!(result, MirValue::Integer(42));
    
    println!("✅ All boundary value tests passed!");
}

#[test]
fn test_type_compatibility() {
    let mut executor = MirExecutor::new();
    
    // Test mixed integer and float sum
    let result = executor.call_user_function_by_name("sum", vec![
        MirValue::Integer(5),
        MirValue::Float(3.5),
        MirValue::Integer(2)
    ]).unwrap();
    assert_eq!(result, MirValue::Float(10.5));
    
    // Test string concatenation in sum
    let result = executor.call_user_function_by_name("reduce", vec![
        MirValue::String("add".to_string()),
        MirValue::Array(vec![
            MirValue::String("Hello".to_string()),
            MirValue::String(" ".to_string()),
            MirValue::String("World".to_string())
        ])
    ]).unwrap();
    assert_eq!(result, MirValue::String("Hello World".to_string()));
    
    // Test filter with different predicates
    let result = executor.call_user_function_by_name("filter", vec![
        MirValue::String("notNull".to_string()),
        MirValue::Array(vec![
            MirValue::Integer(1),
            MirValue::Null,
            MirValue::String("test".to_string()),
            MirValue::Null,
            MirValue::Integer(2)
        ])
    ]).unwrap();
    
    if let MirValue::Array(arr) = result {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], MirValue::Integer(1));
        assert_eq!(arr[1], MirValue::String("test".to_string()));
        assert_eq!(arr[2], MirValue::Integer(2));
    } else {
        // Use safe assertion instead of panic
        panic!("Expected array result from filter function, got: {result:?}");
    }
    
    println!("✅ All type compatibility tests passed!");
}
