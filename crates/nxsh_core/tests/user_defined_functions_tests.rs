use nxsh_core::mir::{MirExecutor, MirValue};

#[test]
fn test_user_defined_functions() {
    let mut executor = MirExecutor::new();

    // Test factorial function
    let factorial_args = vec![MirValue::Integer(5)];
    let result = executor
        .call_user_function_by_name("factorial", factorial_args)
        .unwrap();
    assert_eq!(result, MirValue::Integer(120));

    // Test fibonacci function
    let fib_args = vec![MirValue::Integer(10)];
    let result = executor
        .call_user_function_by_name("fibonacci", fib_args)
        .unwrap();
    assert_eq!(result, MirValue::Integer(55));

    // Test max function
    let max_args = vec![
        MirValue::Integer(3),
        MirValue::Integer(7),
        MirValue::Integer(1),
        MirValue::Integer(5),
    ];
    let result = executor
        .call_user_function_by_name("max", max_args)
        .unwrap();
    assert_eq!(result, MirValue::Integer(7));

    // Test sum function
    let sum_args = vec![
        MirValue::Integer(1),
        MirValue::Integer(2),
        MirValue::Integer(3),
        MirValue::Integer(4),
    ];
    let result = executor
        .call_user_function_by_name("sum", sum_args)
        .unwrap();
    assert_eq!(result, MirValue::Integer(10));

    println!("✅ All user-defined function tests passed!");
}

#[test]
fn test_higher_order_functions() {
    let mut executor = MirExecutor::new();

    // Test map function with double
    let map_args = vec![
        MirValue::String("double".to_string()),
        MirValue::Array(vec![
            MirValue::Integer(1),
            MirValue::Integer(2),
            MirValue::Integer(3),
        ]),
    ];
    let result = executor
        .call_user_function_by_name("map", map_args)
        .unwrap();

    if let MirValue::Array(arr) = result {
        assert_eq!(
            arr,
            vec![
                MirValue::Integer(2),
                MirValue::Integer(4),
                MirValue::Integer(6)
            ]
        );
    } else {
        // Use safe assertion instead of panic
        panic!("Expected array result from map function, got: {result:?}");
    }

    // Test filter function
    let filter_args = vec![
        MirValue::String("isEven".to_string()),
        MirValue::Array(vec![
            MirValue::Integer(1),
            MirValue::Integer(2),
            MirValue::Integer(3),
            MirValue::Integer(4),
            MirValue::Integer(5),
            MirValue::Integer(6),
        ]),
    ];
    let result = executor
        .call_user_function_by_name("filter", filter_args)
        .unwrap();

    if let MirValue::Array(arr) = result {
        assert_eq!(
            arr,
            vec![
                MirValue::Integer(2),
                MirValue::Integer(4),
                MirValue::Integer(6)
            ]
        );
    } else {
        // Use safe assertion instead of panic
        panic!("Expected array result from filter function, got: {result:?}");
    }

    // Test reduce function
    let reduce_args = vec![
        MirValue::String("add".to_string()),
        MirValue::Array(vec![
            MirValue::Integer(1),
            MirValue::Integer(2),
            MirValue::Integer(3),
            MirValue::Integer(4),
        ]),
    ];
    let result = executor
        .call_user_function_by_name("reduce", reduce_args)
        .unwrap();
    assert_eq!(result, MirValue::Integer(10));

    println!("✅ All higher-order function tests passed!");
}

#[test]
fn test_performance_optimized_functions() {
    let mut executor = MirExecutor::new();

    // Test performance with larger datasets
    let large_array: Vec<MirValue> = (1..=1000).map(MirValue::Integer).collect();

    let start = std::time::Instant::now();
    let sum_args = vec![
        MirValue::String("add".to_string()),
        MirValue::Array(large_array.clone()),
    ];
    let result = executor
        .call_user_function_by_name("reduce", sum_args)
        .unwrap();
    let duration = start.elapsed();

    // Expected sum: 1 + 2 + ... + 1000 = 500500
    assert_eq!(result, MirValue::Integer(500500));

    // Performance assertion: should complete in under 1ms
    assert!(
        duration.as_micros() < 1000,
        "Reduce function took too long: {duration:?}"
    );

    println!("✅ Performance test passed! Large array sum in {duration:?}");
}
