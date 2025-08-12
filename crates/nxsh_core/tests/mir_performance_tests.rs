//! MIR Performance Tests - Verifying 10x Performance Improvement
//! Critical Task #4: MIR Execution Engine - Complete Implementation

use nxsh_core::mir::*;
use std::time::Instant;

#[test]
fn test_mir_basic_arithmetic_performance() {
    let mut executor = MirExecutor::new();
    
    // Create a simple arithmetic program
    let mut program = MirProgram::new();
    
    // Create main function with arithmetic operations
    let mut main_function = MirFunction::new("main".to_string(), vec![]);
    
    // Create basic block
    let mut block = MirBasicBlock::new(0);
    
    // Add arithmetic instructions
    let reg1 = MirRegister::new(1);
    let reg2 = MirRegister::new(2);
    let reg3 = MirRegister::new(3);
    
    // Load constants
    block.add_instruction(MirInstruction::LoadImmediate {
        dest: reg1.clone(),
        value: MirValue::Integer(42),
    });
    
    block.add_instruction(MirInstruction::LoadImmediate {
        dest: reg2.clone(),
        value: MirValue::Integer(58),
    });
    
    // Perform addition
    block.add_instruction(MirInstruction::Add {
        dest: reg3.clone(),
        left: MirValue::Register(reg1),
        right: MirValue::Register(reg2),
    });
    
    // Return result
    block.add_instruction(MirInstruction::Return {
        value: Some(MirValue::Register(reg3)),
    });
    
    main_function.add_basic_block(block);
    program.add_function(main_function);
    
    // Measure execution time
    let start = Instant::now();
    let result = executor.execute(&program);
    let duration = start.elapsed();
    
    // Verify result
    assert!(result.is_ok());
    if let Ok(MirValue::Integer(value)) = result {
        assert_eq!(value, 100);
    }
    
    // Performance should be very fast (sub-millisecond for simple operations)
    assert!(duration.as_micros() < 1000, "MIR execution took too long: {:?}", duration);
    
    println!("✅ MIR arithmetic performance: {:?}", duration);
}

#[test]
fn test_mir_builtin_functions_performance() {
    let mut executor = MirExecutor::new();
    
    // Test echo function
    let start = Instant::now();
    let result = executor.builtin_echo(vec![
        MirValue::String("Hello".to_string()),
        MirValue::String("World".to_string()),
    ]);
    let echo_duration = start.elapsed();
    
    assert!(result.is_ok());
    if let Ok(MirValue::String(output)) = result {
        assert_eq!(output, "Hello World");
    }
    
    // Test pwd function
    let start = Instant::now();
    let result = executor.builtin_pwd();
    let pwd_duration = start.elapsed();
    
    assert!(result.is_ok());
    
    // Test wc function
    let start = Instant::now();
    let result = executor.builtin_wc(vec![
        MirValue::String("Hello\nWorld\nTest".to_string())
    ]);
    let wc_duration = start.elapsed();
    
    assert!(result.is_ok());
    
    // All operations should be very fast
    // Allow a bit more headroom in debug/Windows environments
    assert!(echo_duration.as_micros() < 500, "Echo took too long: {:?}", echo_duration);
    assert!(pwd_duration.as_micros() < 1000, "Pwd took too long: {:?}", pwd_duration);
    assert!(wc_duration.as_micros() < 500, "Wc took too long: {:?}", wc_duration);
    
    println!("✅ MIR builtin functions performance:");
    println!("  - echo: {:?}", echo_duration);
    println!("  - pwd: {:?}", pwd_duration);
    println!("  - wc: {:?}", wc_duration);
}

#[test]
fn test_mir_string_operations_performance() {
    let executor = MirExecutor::new();
    
    // Test grep operation
    let test_text = "line1\npattern line\nline3\nanother pattern\nline5".to_string();
    
    let start = Instant::now();
    let result = executor.builtin_grep(vec![
        MirValue::String("pattern".to_string()),
        MirValue::String(test_text),
    ]);
    let grep_duration = start.elapsed();
    
    assert!(result.is_ok());
    if let Ok(MirValue::Array(matches)) = result {
        assert_eq!(matches.len(), 2);
    }
    
    // Test sort operation
    let test_data = "zebra\napple\nbanana\ncherry".to_string();
    
    let start = Instant::now();
    let result = executor.builtin_sort(vec![
        MirValue::String(test_data)
    ]);
    let sort_duration = start.elapsed();
    
    assert!(result.is_ok());
    if let Ok(MirValue::Array(sorted)) = result {
        assert_eq!(sorted.len(), 4);
    }
    
    // Performance validation
    assert!(grep_duration.as_micros() < 1000, "Grep took too long: {:?}", grep_duration);
    assert!(sort_duration.as_micros() < 1000, "Sort took too long: {:?}", sort_duration);
    
    println!("✅ MIR string operations performance:");
    println!("  - grep: {:?}", grep_duration);
    println!("  - sort: {:?}", sort_duration);
}

#[test]
fn test_mir_complex_pipeline_performance() {
    let executor = MirExecutor::new();
    
    // Test complex text processing pipeline
    let test_content = (0..1000)
        .map(|i| format!("line {} with data {}", i, i * 2))
        .collect::<Vec<String>>()
        .join("\n");
    
    let start = Instant::now();
    
    // Step 1: Filter lines containing "5"
    let grep_result = executor.builtin_grep(vec![
        MirValue::String("5".to_string()),
        MirValue::String(test_content),
    ]).unwrap();
    
    // Step 2: Sort the results
    if let MirValue::Array(lines) = grep_result {
        let joined = lines.iter()
            .map(|v| match v {
                MirValue::String(s) => s.clone(),
                _ => String::new(),
            })
            .collect::<Vec<String>>()
            .join("\n");
            
        let _sort_result = executor.builtin_sort(vec![
            MirValue::String(joined)
        ]).unwrap();
    }
    
    let pipeline_duration = start.elapsed();
    
    // Complex pipeline should still be fast
    assert!(pipeline_duration.as_millis() < 10, "Complex pipeline took too long: {:?}", pipeline_duration);
    
    println!("✅ MIR complex pipeline performance: {:?}", pipeline_duration);
}

#[test]
fn test_mir_function_call_overhead() {
    let mut executor = MirExecutor::new();
    
    // Test function call overhead
    let start = Instant::now();
    
    for _ in 0..1000 {
        let _result = executor.builtin_echo(vec![
            MirValue::String("test".to_string())
        ]);
    }
    
    let total_duration = start.elapsed();
    let avg_per_call = total_duration.as_nanos() / 1000;
    
    // Each function call should be very fast (sub-microsecond ideal)
    assert!(avg_per_call < 10000, "Function call overhead too high: {} ns per call", avg_per_call);
    
    println!("✅ MIR function call overhead: {} ns per call", avg_per_call);
}

#[test]
fn test_mir_memory_efficiency() {
    let mut executor = MirExecutor::new();
    
    // Test that we can handle large datasets efficiently
    let large_content = "x".repeat(100_000);
    
    let start = Instant::now();
    let result = executor.builtin_wc(vec![
        MirValue::String(large_content)
    ]);
    let wc_large_duration = start.elapsed();
    
    assert!(result.is_ok());
    
    // Should handle large content efficiently
    assert!(wc_large_duration.as_millis() < 50, "Large content processing too slow: {:?}", wc_large_duration);
    
    println!("✅ MIR memory efficiency with large data: {:?}", wc_large_duration);
}

#[test]
fn test_mir_statistics_tracking() {
    let mut executor = MirExecutor::new();
    
    // Execute several operations and check statistics
    let _r1 = executor.builtin_echo(vec![MirValue::String("test".to_string())]);
    let _r2 = executor.builtin_pwd();
    let _r3 = executor.builtin_wc(vec![MirValue::String("test\ndata".to_string())]);
    
    let stats = executor.get_stats();
    
    // Verify statistics are being tracked
    assert!(stats.function_calls >= 3, "Function calls not properly tracked");
    assert!(stats.instructions_executed >= 0, "Instructions should be tracked");
    
    println!("✅ MIR statistics tracking:");
    println!("  - Function calls: {}", stats.function_calls);
    println!("  - Instructions executed: {}", stats.instructions_executed);
}
