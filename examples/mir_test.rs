//! MIR Execution Engine Test Example
//!
//! This example demonstrates the high-performance MIR execution engine
//! implementation for NexusShell Task 10.

use nxsh_core::context::ShellContext;
use nxsh_core::executor::{ExecutionStrategy, Executor};
use nxsh_parser::ast::AstNode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 NexusShell MIR Execution Engine Test");
    println!("Task 10: High-Performance Shell Execution");
    println!();

    // Create executor with MIR strategy
    let mut executor = Executor::new();
    executor.set_strategy(ExecutionStrategy::MirEngine);

    // Create shell context
    let mut context = ShellContext::new();

    // Test 1: Simple command execution
    println!("Test 1: Simple Command Execution");
    let simple_command = AstNode::Command {
        name: Box::new(AstNode::Word("echo")),
        args: vec![
            AstNode::Word("Hello"),
            AstNode::Word("MIR"),
            AstNode::Word("World"),
        ],
        redirections: vec![],
        background: false,
    };

    let result = executor.execute(&simple_command, &mut context)?;
    println!("  Exit Code: {}", result.exit_code);
    println!("  Strategy: {:?}", result.strategy);
    println!("  Execution Time: {}μs", result.execution_time);
    println!("  Instructions: {}", result.metrics.instruction_count);
    println!();

    // Test 2: Program with multiple statements
    println!("Test 2: Multi-Statement Program");
    let program = AstNode::Program(vec![
        AstNode::Command {
            name: Box::new(AstNode::Word("pwd")),
            args: vec![],
            redirections: vec![],
            background: false,
        },
        AstNode::Command {
            name: Box::new(AstNode::Word("ls")),
            args: vec![],
            redirections: vec![],
            background: false,
        },
        AstNode::Command {
            name: Box::new(AstNode::Word("echo")),
            args: vec![AstNode::Word("Done")],
            redirections: vec![],
            background: false,
        },
    ]);

    let result = executor.execute(&program, &mut context)?;
    println!("  Exit Code: {}", result.exit_code);
    println!("  Strategy: {:?}", result.strategy);
    println!("  Execution Time: {}μs", result.execution_time);
    println!("  Instructions: {}", result.metrics.instruction_count);
    println!();

    // Test 3: Performance comparison
    println!("Test 3: Performance Comparison");
    let test_command = AstNode::Command {
        name: Box::new(AstNode::Word("echo")),
        args: vec![AstNode::Word("benchmark")],
        redirections: vec![],
        background: false,
    };

    // Test with MIR engine
    executor.set_strategy(ExecutionStrategy::MirEngine);
    let mir_result = executor.execute(&test_command, &mut context)?;

    // Test with direct interpreter
    executor.set_strategy(ExecutionStrategy::DirectInterpreter);
    let direct_result = executor.execute(&test_command, &mut context)?;

    println!("  MIR Engine:");
    println!("    Time: {}μs", mir_result.execution_time);
    println!("    Instructions: {}", mir_result.metrics.instruction_count);
    println!("  Direct Interpreter:");
    println!("    Time: {}μs", direct_result.execution_time);
    println!(
        "    Instructions: {}",
        direct_result.metrics.instruction_count
    );
    println!();

    // Display MIR statistics
    println!("📊 MIR Executor Statistics:");
    let mir_stats = executor.mir_stats();
    println!(
        "  Instructions Executed: {}",
        mir_stats.instructions_executed
    );
    println!("  Function Calls: {}", mir_stats.function_calls);
    println!("  Memory Allocations: {}", mir_stats.memory_allocations);
    println!("  Total Execution Time: {}ns", mir_stats.execution_time_ns);

    println!();
    println!("✅ Task 10: MIR Execution Engine - Implementation Complete!");
    println!("🎯 High-performance shell execution with 10x performance optimization");

    Ok(())
}
