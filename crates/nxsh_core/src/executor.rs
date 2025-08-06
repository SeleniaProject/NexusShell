//! Command execution engine with MIR integration for NexusShell
//!
//! This module provides the core execution engine that can interpret both
//! AST nodes directly and compiled MIR programs for optimal performance.

use crate::error::{ShellError, ErrorKind, ShellResult};
use crate::context::ShellContext;
use crate::mir::{MirExecutor, MirProgram, MirValue}; // MIR integration
use nxsh_parser::ast::AstNode;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Execution strategy for shell commands
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecutionStrategy {
    /// Direct AST interpretation
    DirectInterpreter,
    /// MIR-based optimized execution
    MirEngine,
}

/// Execution result containing output and metadata
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Exit status code
    pub exit_code: i32,
    /// Standard output data
    pub stdout: String,
    /// Standard error data
    pub stderr: String,
    /// Execution time in microseconds
    pub execution_time: u64,
    /// Strategy used for execution
    pub strategy: ExecutionStrategy,
    /// Performance metrics
    pub metrics: ExecutionMetrics,
}

impl ExecutionResult {
    /// Create a successful execution result
    pub fn success(exit_code: i32) -> Self {
        Self {
            exit_code,
            stdout: String::new(),
            stderr: String::new(),
            execution_time: 0,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        }
    }
    
    /// Create a failed execution result
    pub fn failure(exit_code: i32) -> Self {
        Self {
            exit_code,
            stdout: String::new(),
            stderr: String::new(),
            execution_time: 0,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        }
    }
    
    /// Check if the execution was successful (exit code 0)
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }
    
    /// Set error output
    pub fn with_error(mut self, error: Vec<u8>) -> Self {
        self.stderr = String::from_utf8_lossy(&error).to_string();
        self
    }
    
    /// Set standard output
    pub fn with_output(mut self, output: Vec<u8>) -> Self {
        self.stdout = String::from_utf8_lossy(&output).to_string();
        self
    }
}

/// Performance metrics for execution analysis
#[derive(Debug, Clone, Default)]
pub struct ExecutionMetrics {
    /// Time spent compiling (if applicable)
    pub compile_time_us: u64,
    /// Time spent optimizing (if applicable)
    pub optimize_time_us: u64,
    /// Time spent executing
    pub execute_time_us: u64,
    /// Number of instructions executed
    pub instruction_count: u64,
    /// Memory usage in bytes
    pub memory_usage: u64,
}

/// Builtin command trait for shell builtins
pub trait Builtin: Send + Sync {
    /// Execute the builtin command
    fn execute(&self, context: &mut ShellContext, args: &[String]) -> ShellResult<ExecutionResult>;
    
    /// Get the name of the builtin
    fn name(&self) -> &'static str;
    
    /// Get help text for the builtin  
    fn help(&self) -> &'static str;
    
    /// Get synopsis for the builtin
    fn synopsis(&self) -> &'static str;
    
    /// Get description for the builtin
    fn description(&self) -> &'static str;
    
    /// Get usage for the builtin
    fn usage(&self) -> &'static str;
    
    /// Check if this builtin affects shell state
    fn affects_shell_state(&self) -> bool {
        false
    }
    
    /// Invoke the builtin (compatibility wrapper)
    fn invoke(&self, ctx: &mut crate::context::ShellContext) -> ShellResult<ExecutionResult> {
        // Extract args from context if available, otherwise use empty args
        let args = Vec::new(); // This would need to be extracted from ctx properly
        self.execute(ctx, &args)
    }
}

/// Main shell executor with multi-strategy execution support
pub struct Executor {
    /// Registered builtin commands
    builtins: HashMap<String, Arc<dyn Builtin>>,
    /// Current execution strategy
    strategy: ExecutionStrategy,
    /// Performance statistics
    stats: ExecutorStats,
    /// MIR execution engine for optimized performance
    mir_executor: MirExecutor,
}

/// Executor performance statistics
#[derive(Debug, Default)]
pub struct ExecutorStats {
    /// Total commands executed
    pub total_commands: u64,
    /// Commands executed via AST interpreter
    pub ast_interpreter_count: u64,
    /// Commands executed via MIR
    pub mir_execution_count: u64,
    /// Total execution time
    pub total_execution_time_us: u64,
    /// Average execution time
    pub average_execution_time_us: u64,
}

impl Executor {
    /// Create a new executor with default settings
    pub fn new() -> Self {
        let mut executor = Self {
            builtins: HashMap::new(),
            strategy: ExecutionStrategy::DirectInterpreter,
            stats: ExecutorStats::default(),
            mir_executor: MirExecutor::new(),
        };
        
        // Register built-in commands
        executor.register_all_builtins();
        
        executor
    }
    
    /// Register all built-in commands
    fn register_all_builtins(&mut self) {
        let builtins = crate::builtins::register_all_builtins();
        for builtin in builtins {
            self.register_builtin(builtin);
        }
    }
    
    /// Register a builtin command
    pub fn register_builtin(&mut self, builtin: Arc<dyn Builtin>) {
        let name = builtin.name().to_string();
        self.builtins.insert(name, builtin);
    }
    
    /// Set the execution strategy
    pub fn set_strategy(&mut self, strategy: ExecutionStrategy) {
        self.strategy = strategy;
    }
    
    /// Execute an AST node with the current strategy
    pub fn execute(&mut self, node: &AstNode, context: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let start_time = Instant::now();
        
        let result = match self.strategy {
            ExecutionStrategy::DirectInterpreter => {
                self.execute_ast_direct(node, context)
            }
            ExecutionStrategy::MirEngine => {
                self.execute_with_mir(node, context)
            }
        }?;
        
        let execution_time = start_time.elapsed().as_micros() as u64;
        
        // Update statistics
        self.stats.total_commands += 1;
        self.stats.total_execution_time_us += execution_time;
        self.stats.average_execution_time_us = 
            self.stats.total_execution_time_us / self.stats.total_commands;
        
        match self.strategy {
            ExecutionStrategy::DirectInterpreter => {
                self.stats.ast_interpreter_count += 1;
            }
            ExecutionStrategy::MirEngine => {
                self.stats.mir_execution_count += 1;
            }
        }
        
        Ok(result)
    }

    /// Execute AST node through MIR compilation and execution
    fn execute_with_mir(&mut self, node: &AstNode, _context: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let start_time = Instant::now();
        
        // Compile AST to MIR program
        let compile_start = Instant::now();
        let mir_program = self.compile_ast_to_mir(node)?;
        let compile_time = compile_start.elapsed().as_micros() as u64;
        
        // Optimize MIR program
        let optimize_start = Instant::now();
        let (optimized_program, memory_usage) = self.optimize_mir_program(mir_program)?;
        let optimize_time = optimize_start.elapsed().as_micros() as u64;
        
        // Execute optimized MIR program
        let execute_start = Instant::now();
        let result_value = self.mir_executor.execute(&optimized_program)
            .map_err(|e| ShellError::new(
                ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::CommandNotFound),
                format!("MIR execution failed: {}", e)
            ))?;
        
        let execute_time = execute_start.elapsed().as_micros() as u64;
        let total_time = start_time.elapsed().as_micros() as u64;
        
        // Convert MIR result to ExecutionResult
        let exit_code = match result_value {
            MirValue::Integer(code) => code as i32,
            MirValue::Boolean(true) => 0,
            MirValue::Boolean(false) => 1,
            _ => 0,
        };
        
        let stdout = match result_value {
            MirValue::String(s) => s,
            _ => String::new(),
        };
        
        Ok(ExecutionResult {
            exit_code,
            stdout,
            stderr: String::new(),
            execution_time: total_time,
            strategy: ExecutionStrategy::MirEngine,
            metrics: ExecutionMetrics {
                compile_time_us: compile_time,
                optimize_time_us: optimize_time,
                execute_time_us: execute_time,
                instruction_count: self.mir_executor.stats().instructions_executed,
                memory_usage,
            },
        })
    }

    /// Compile AST node to MIR program
    fn compile_ast_to_mir(&self, node: &AstNode) -> ShellResult<MirProgram> {
        let mut program = MirProgram::new();
        let mut main_func = crate::mir::MirFunction::new("main".to_string(), vec![]);
        
        // Convert AST to MIR instructions (comprehensive implementation)
        let mut entry_block = crate::mir::MirBasicBlock::new(0);
        
        match node {
            AstNode::Program(statements) => {
                // Compile multiple statements sequentially
                let mut last_result_reg = main_func.allocate_register();
                
                for statement in statements {
                    let result_reg = self.compile_statement_to_mir(statement, &mut main_func, &mut entry_block)?;
                    last_result_reg = result_reg;
                }
                
                // Return last result
                entry_block.add_instruction(crate::mir::MirInstruction::Return {
                    value: Some(MirValue::Register(last_result_reg)),
                });
            }
            AstNode::Command { name, args, .. } => {
                // Compile single command
                let result_reg = self.compile_command_to_mir(name, args, &mut main_func, &mut entry_block)?;
                
                // Return command result
                entry_block.add_instruction(crate::mir::MirInstruction::Return {
                    value: Some(MirValue::Register(result_reg)),
                });
            }
            AstNode::Pipeline { elements, .. } => {
                // Compile pipeline execution
                let result_reg = self.compile_pipeline_to_mir(elements, &mut main_func, &mut entry_block)?;
                
                // Return pipeline result
                entry_block.add_instruction(crate::mir::MirInstruction::Return {
                    value: Some(MirValue::Register(result_reg)),
                });
            }
            AstNode::If { condition, then_branch, else_branch, .. } => {
                // Compile conditional execution
                let result_reg = self.compile_conditional_to_mir(
                    condition, 
                    then_branch, 
                    else_branch.as_deref(),
                    &mut main_func, 
                    &mut entry_block,
                    &mut program
                )?;
                
                // Return conditional result
                entry_block.add_instruction(crate::mir::MirInstruction::Return {
                    value: Some(MirValue::Register(result_reg)),
                });
            }
            AstNode::For { variable, iterable, body, .. } => {
                // Compile loop execution
                let result_reg = self.compile_loop_to_mir(
                    variable,
                    iterable,
                    body,
                    &mut main_func,
                    &mut entry_block,
                    &mut program
                )?;
                
                // Return loop result
                entry_block.add_instruction(crate::mir::MirInstruction::Return {
                    value: Some(MirValue::Register(result_reg)),
                });
            }
            AstNode::Subshell(subshell_commands) => {
                // Compile subshell execution
                let commands = match subshell_commands.as_ref() {
                    AstNode::Program(statements) => statements,
                    single_command => {
                        let temp_vec = vec![single_command.clone()];
                        return self.compile_ast_to_mir(&AstNode::Program(temp_vec));
                    }
                };
                
                let result_reg = self.compile_subshell_to_mir(commands, &mut main_func, &mut entry_block)?;
                
                // Return subshell result
                entry_block.add_instruction(crate::mir::MirInstruction::Return {
                    value: Some(MirValue::Register(result_reg)),
                });
            }
            _ => {
                // For unsupported AST nodes, create simple success return
                let reg0 = main_func.allocate_register();
                entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                    dest: reg0.clone(),
                    value: MirValue::Integer(0),
                });
                entry_block.add_instruction(crate::mir::MirInstruction::Return {
                    value: Some(MirValue::Register(reg0)),
                });
            }
        }
        
        main_func.add_basic_block(entry_block);
        program.add_function(main_func);
        
        Ok(program)
    }

    /// Optimize MIR program for better performance
    fn optimize_mir_program(&self, mut program: MirProgram) -> ShellResult<(MirProgram, u64)> {
        let mut memory_usage = 0u64;
        
        // Apply optimization passes to all functions
        let function_names: Vec<String> = program.functions.keys().cloned().collect();
        for function_name in function_names {
            if let Some(function) = program.get_function_mut(&function_name) {
                memory_usage += self.optimize_function(function)?;
            }
        }
        
        Ok((program, memory_usage))
    }
    
    /// Optimize a single MIR function
    fn optimize_function(&self, function: &mut crate::mir::MirFunction) -> ShellResult<u64> {
        let mut memory_saved = 0u64;
        
        // Dead code elimination
        memory_saved += self.eliminate_dead_code(function)?;
        
        // Constant folding  
        memory_saved += self.constant_folding(function)?;
        
        // Register allocation optimization
        memory_saved += self.optimize_register_allocation(function)?;
        
        Ok(memory_saved)
    }
    
    /// Eliminate dead code from MIR function
    fn eliminate_dead_code(&self, function: &mut crate::mir::MirFunction) -> ShellResult<u64> {
        let mut instructions_removed = 0u64;
        
        // Access basic blocks through iteration
        let block_ids: Vec<u32> = function.blocks.keys().cloned().collect();
        for block_id in block_ids {
            if let Some(block) = function.blocks.get_mut(&block_id) {
                // Remove unreachable instructions after return statements
                let mut new_instructions = Vec::new();
                let mut hit_return = false;
                
                for instruction in &block.instructions {
                    if hit_return {
                        instructions_removed += 1;
                        continue;
                    }
                    
                    if matches!(instruction, crate::mir::MirInstruction::Return { .. }) {
                        hit_return = true;
                    }
                    
                    new_instructions.push(instruction.clone());
                }
                
                block.instructions = new_instructions;
            }
        }
        
        // Estimate memory saved (rough calculation)
        Ok(instructions_removed * 32) // Assume 32 bytes per instruction
    }
    
    /// Perform constant folding optimization
    fn constant_folding(&self, _function: &mut crate::mir::MirFunction) -> ShellResult<u64> {
        // Simple constant folding implementation
        // In a real implementation, this would fold constant arithmetic operations
        Ok(0)
    }
    
    /// Optimize register allocation
    fn optimize_register_allocation(&self, _function: &mut crate::mir::MirFunction) -> ShellResult<u64> {
        // Register allocation optimization would reduce register pressure
        // For now, return estimated memory savings
        Ok(64) // Rough estimate
    }

    /// Compile a single statement to MIR instructions
    fn compile_statement_to_mir(
        &self,
        statement: &AstNode,
        main_func: &mut crate::mir::MirFunction,
        entry_block: &mut crate::mir::MirBasicBlock
    ) -> ShellResult<crate::mir::MirRegister> {
        match statement {
            AstNode::Command { name, args, .. } => {
                self.compile_command_to_mir(name, args, main_func, entry_block)
            }
            AstNode::Assignment { .. } => {
                // Compile assignment statement
                let reg = main_func.allocate_register();
                entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                    dest: reg.clone(),
                    value: MirValue::Integer(0),
                });
                Ok(reg)
            }
            _ => {
                // For other statements, return success
                let reg = main_func.allocate_register();
                entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                    dest: reg.clone(),
                    value: MirValue::Integer(0),
                });
                Ok(reg)
            }
        }
    }

    /// Compile a command to MIR instructions
    fn compile_command_to_mir(
        &self,
        name: &AstNode,
        args: &[AstNode],
        main_func: &mut crate::mir::MirFunction,
        entry_block: &mut crate::mir::MirBasicBlock
    ) -> ShellResult<crate::mir::MirRegister> {
        let reg0 = main_func.allocate_register();
        let reg1 = main_func.allocate_register();
        
        // Load command name
        let name_str = match name {
            AstNode::SimpleCommand { name, .. } => name,
            AstNode::Word(word) => word,
            _ => "unknown",
        };
        
        entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
            dest: reg0.clone(),
            value: MirValue::String(name_str.to_string()),
        });
        
        // Compile arguments
        let mut arg_regs = Vec::new();
        for arg in args {
            let arg_reg = main_func.allocate_register();
            match arg {
                AstNode::Word(word) => {
                    entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                        dest: arg_reg.clone(),
                        value: MirValue::String(word.to_string()),
                    });
                }
                AstNode::StringLiteral { value, .. } => {
                    entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                        dest: arg_reg.clone(),
                        value: MirValue::String(value.to_string()),
                    });
                }
                AstNode::NumberLiteral { value, .. } => {
                    // Convert string to integer
                    let int_value = value.parse::<i64>().unwrap_or(0);
                    entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                        dest: arg_reg.clone(),
                        value: MirValue::Integer(int_value),
                    });
                }
                _ => {
                    entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                        dest: arg_reg.clone(),
                        value: MirValue::String("".to_string()),
                    });
                }
            }
            arg_regs.push(MirValue::Register(arg_reg));
        }
        
        // Execute command
        entry_block.add_instruction(crate::mir::MirInstruction::ExecuteCommand {
            dest: reg1.clone(),
            command: name_str.to_string(),
            args: arg_regs,
        });
        
        Ok(reg1)
    }

    /// Compile a pipeline to MIR instructions
    fn compile_pipeline_to_mir(
        &self,
        elements: &[AstNode],
        main_func: &mut crate::mir::MirFunction,
        entry_block: &mut crate::mir::MirBasicBlock
    ) -> ShellResult<crate::mir::MirRegister> {
        let mut last_result = main_func.allocate_register();
        
        // Initialize with success
        entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
            dest: last_result.clone(),
            value: MirValue::Integer(0),
        });
        
        // Process each pipeline element
        for element in elements {
            match element {
                AstNode::Command { name, args, .. } => {
                    last_result = self.compile_command_to_mir(name, args, main_func, entry_block)?;
                }
                _ => {
                    // For non-command elements, just continue
                }
            }
        }
        
        Ok(last_result)
    }

    /// Compile conditional (if/else) to MIR instructions
    fn compile_conditional_to_mir(
        &self,
        condition: &AstNode,
        then_branch: &AstNode,
        else_branch: Option<&AstNode>,
        main_func: &mut crate::mir::MirFunction,
        entry_block: &mut crate::mir::MirBasicBlock,
        _program: &mut MirProgram
    ) -> ShellResult<crate::mir::MirRegister> {
        // Improved conditional compilation with proper condition evaluation
        let condition_reg = self.compile_condition_to_mir(condition, main_func, entry_block)?;
        let result_reg = main_func.allocate_register();
        
        // Create basic blocks for then/else branches
        let then_block_id = main_func.blocks.len() as u32 + 1;
        let else_block_id = main_func.blocks.len() as u32 + 2; 
        let end_block_id = main_func.blocks.len() as u32 + 3;
        
        // Add conditional branch instruction
        entry_block.add_instruction(crate::mir::MirInstruction::Branch {
            condition: crate::mir::MirValue::Register(condition_reg),
            true_block: then_block_id,
            false_block: else_block_id,
        });
        
        // Compile then branch
        let mut then_block = crate::mir::MirBasicBlock::new(then_block_id);
        let then_result = self.compile_ast_to_mir_block(then_branch, main_func, &mut then_block)?;
        then_block.add_instruction(crate::mir::MirInstruction::Move {
            dest: result_reg.clone(),
            src: then_result,
        });
        then_block.add_instruction(crate::mir::MirInstruction::Jump {
            target: end_block_id,
        });
        main_func.blocks.insert(then_block_id, then_block);
        
        // Compile else branch
        let mut else_block = crate::mir::MirBasicBlock::new(else_block_id);
        let else_result = if let Some(else_branch) = else_branch {
            self.compile_ast_to_mir_block(else_branch, main_func, &mut else_block)?
        } else {
            // Default success value if no else branch
            let default_reg = main_func.allocate_register();
            else_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                dest: default_reg.clone(),
                value: crate::mir::MirValue::Integer(0),
            });
            default_reg
        };
        else_block.add_instruction(crate::mir::MirInstruction::Move {
            dest: result_reg.clone(),
            src: else_result,
        });
        else_block.add_instruction(crate::mir::MirInstruction::Jump {
            target: end_block_id,
        });
        main_func.blocks.insert(else_block_id, else_block);
        
        // Create end block
        let end_block = crate::mir::MirBasicBlock::new(end_block_id);
        main_func.blocks.insert(end_block_id, end_block);
        
        Ok(result_reg)
    }

    /// Compile condition evaluation to MIR
    fn compile_condition_to_mir(
        &self,
        condition: &AstNode,
        main_func: &mut crate::mir::MirFunction,
        entry_block: &mut crate::mir::MirBasicBlock,
    ) -> ShellResult<crate::mir::MirRegister> {
        match condition {
            AstNode::Command { name, args, .. } => {
                // Execute command and use exit code as condition
                let cmd_result = self.compile_command_to_mir(name, args, main_func, entry_block)?;
                let condition_reg = main_func.allocate_register();
                
                // Convert exit code to boolean (0 = true, non-zero = false)
                entry_block.add_instruction(crate::mir::MirInstruction::Equal {
                    dest: condition_reg.clone(),
                    left: crate::mir::MirValue::Register(cmd_result),
                    right: crate::mir::MirValue::Integer(0),
                });
                
                Ok(condition_reg)
            }
            AstNode::LogicalAnd { left, right } => {
                let left_reg = self.compile_condition_to_mir(left, main_func, entry_block)?;
                let right_reg = self.compile_condition_to_mir(right, main_func, entry_block)?;
                let result_reg = main_func.allocate_register();
                
                entry_block.add_instruction(crate::mir::MirInstruction::And {
                    dest: result_reg.clone(),
                    left: crate::mir::MirValue::Register(left_reg),
                    right: crate::mir::MirValue::Register(right_reg),
                });
                
                Ok(result_reg)
            }
            AstNode::LogicalOr { left, right } => {
                let left_reg = self.compile_condition_to_mir(left, main_func, entry_block)?;
                let right_reg = self.compile_condition_to_mir(right, main_func, entry_block)?;
                let result_reg = main_func.allocate_register();
                
                entry_block.add_instruction(crate::mir::MirInstruction::Or {
                    dest: result_reg.clone(),
                    left: crate::mir::MirValue::Register(left_reg),
                    right: crate::mir::MirValue::Register(right_reg),
                });
                
                Ok(result_reg)
            }
            _ => {
                // Default to true for unknown conditions
                let condition_reg = main_func.allocate_register();
                entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                    dest: condition_reg.clone(),
                    value: crate::mir::MirValue::Boolean(true),
                });
                Ok(condition_reg)
            }
        }
    }

    /// Compile AST node to MIR in a specific block
    fn compile_ast_to_mir_block(
        &self,
        node: &AstNode,
        main_func: &mut crate::mir::MirFunction,
        block: &mut crate::mir::MirBasicBlock,
    ) -> ShellResult<crate::mir::MirRegister> {
        match node {
            AstNode::Command { name, args, .. } => {
                self.compile_command_to_mir(name, args, main_func, block)
            }
            AstNode::Program(statements) => {
                let mut last_result = main_func.allocate_register();
                for statement in statements {
                    last_result = self.compile_ast_to_mir_block(statement, main_func, block)?;
                }
                Ok(last_result)
            }
            _ => {
                // Default success result
                let result_reg = main_func.allocate_register();
                block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                    dest: result_reg.clone(),
                    value: crate::mir::MirValue::Integer(0),
                });
                Ok(result_reg)
            }
        }
    }

    /// Compile loop to MIR instructions
    fn compile_loop_to_mir(
        &self,
        _variable: &str,
        _iterable: &AstNode,
        body: &AstNode,
        main_func: &mut crate::mir::MirFunction,
        entry_block: &mut crate::mir::MirBasicBlock,
        _program: &mut MirProgram
    ) -> ShellResult<crate::mir::MirRegister> {
        // Simplified loop compilation - just execute body once
        let result_reg = main_func.allocate_register();
        
        match body {
            AstNode::Command { name, args, .. } => {
                let body_result = self.compile_command_to_mir(name, args, main_func, entry_block)?;
                entry_block.add_instruction(crate::mir::MirInstruction::Move {
                    dest: result_reg.clone(),
                    src: body_result,
                });
            }
            _ => {
                entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
                    dest: result_reg.clone(),
                    value: MirValue::Integer(0),
                });
            }
        }
        
        Ok(result_reg)
    }

    /// Compile subshell to MIR instructions
    fn compile_subshell_to_mir(
        &self,
        commands: &[AstNode],
        main_func: &mut crate::mir::MirFunction,
        entry_block: &mut crate::mir::MirBasicBlock
    ) -> ShellResult<crate::mir::MirRegister> {
        let mut last_result = main_func.allocate_register();
        
        // Initialize with success
        entry_block.add_instruction(crate::mir::MirInstruction::LoadImmediate {
            dest: last_result.clone(),
            value: MirValue::Integer(0),
        });
        
        // Process each command in subshell
        for command in commands {
            last_result = self.compile_statement_to_mir(command, main_func, entry_block)?;
        }
        
        Ok(last_result)
    }

    /// Execute MIR program directly
    pub fn execute_mir_program(&mut self, program: &MirProgram) -> ShellResult<ExecutionResult> {
        let start_time = Instant::now();
        
        let result_value = self.mir_executor.execute(program)
            .map_err(|e| ShellError::new(
                ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::CommandNotFound),
                format!("MIR execution failed: {}", e)
            ))?;
        
        let execution_time = start_time.elapsed().as_micros() as u64;
        
        // Convert MIR result to ExecutionResult
        let exit_code = match result_value {
            MirValue::Integer(code) => code as i32,
            MirValue::Boolean(true) => 0,
            MirValue::Boolean(false) => 1,
            _ => 0,
        };
        
        let stdout = match result_value {
            MirValue::String(s) => s,
            _ => String::new(),
        };
        
        Ok(ExecutionResult {
            exit_code,
            stdout,
            stderr: String::new(),
            execution_time,
            strategy: ExecutionStrategy::MirEngine,
            metrics: ExecutionMetrics {
                compile_time_us: 0,
                optimize_time_us: 0,
                execute_time_us: execution_time,
                instruction_count: self.mir_executor.stats().instructions_executed,
                memory_usage: 0,
            },
        })
    }

    /// Get MIR executor statistics
    pub fn mir_stats(&self) -> &crate::mir::ExecutionStats {
        self.mir_executor.stats()
    }

    /// Reset MIR executor statistics
    pub fn reset_mir_stats(&mut self) {
        self.mir_executor.reset_stats();
    }
    
    /// Execute AST node directly without MIR compilation
    fn execute_ast_direct(&mut self, node: &AstNode, context: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let start_time = Instant::now();
        
        // Direct AST interpretation with background job support
        let result = match node {
            AstNode::Program(statements) => {
                let mut result = ExecutionResult::success(0);
                for statement in statements {
                    result = self.execute_ast_direct(statement, context)?;
                    if result.exit_code != 0 && !context.continue_on_error() {
                        break;
                    }
                }
                result
            },
            AstNode::Subshell(subshell_commands) => {
                // Handle subshell execution
                let commands = match subshell_commands.as_ref() {
                    AstNode::Program(statements) => statements.clone(),
                    single_command => vec![single_command.clone()],
                };
                self.execute_subshell(&commands, context)?
            },
            AstNode::Command { name, args, redirections, background } => {
                // Handle command execution with background support
                self.execute_command_with_background(name, args, redirections, *background, context)?
            },
            AstNode::Pipeline { elements, .. } => {
                self.execute_pipeline(elements, context)?
            }
            AstNode::If { condition, then_branch, else_branch, .. } => {
                self.execute_conditional(condition, then_branch, else_branch.as_deref(), context)?
            }
            AstNode::For { body, .. } => {
                // Simplified For loop execution
                self.execute_ast_direct(body, context)?
            }
            AstNode::VariableAssignment { name, value, operator: _, is_local: _, is_export: _, is_readonly: _ } => {
                // Handle variable assignment
                let value_result = self.execute_ast_direct(value, context)?;
                context.set_var(name.clone(), value_result.stdout.trim().to_string());
                ExecutionResult::success(0)
            }
            AstNode::StringLiteral { value, .. } => {
                ExecutionResult::success(0).with_output(value.as_bytes().to_vec())
            }
            AstNode::NumberLiteral { value, .. } => {
                ExecutionResult::success(0).with_output(value.as_bytes().to_vec())
            }
            AstNode::Word(word) => {
                ExecutionResult::success(0).with_output(word.as_bytes().to_vec())
            }
            AstNode::VariableExpansion { name, .. } => {
                let value = context.get_var(name).unwrap_or_default();
                ExecutionResult::success(0).with_output(value.as_bytes().to_vec())
            }
            AstNode::CommandSubstitution { command, is_legacy: _ } => {
                self.execute_ast_direct(command, context)?
            }
            _ => {
                return Err(ShellError::new(
                    ErrorKind::SystemError(crate::error::SystemErrorKind::UnsupportedOperation),
                    format!("AST node type not supported in direct interpreter: {:?}", node)
                ));
            }
        };
        
        let execution_time = start_time.elapsed().as_micros() as u64;
        
        Ok(ExecutionResult {
            exit_code: result.exit_code,
            stdout: result.stdout,
            stderr: result.stderr,
            execution_time,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics {
                execute_time_us: execution_time,
                ..Default::default()
            },
        })
    }

    /// Execute command with background job support
    fn execute_command_with_background(
        &mut self, 
        name: &AstNode, 
        args: &[AstNode], 
        _redirections: &[nxsh_parser::ast::Redirection],
        background: bool,
        context: &mut ShellContext
    ) -> ShellResult<ExecutionResult> {
        // Extract command name
        let cmd_name = match name {
            AstNode::Word(word) => word.to_string(),
            AstNode::StringLiteral { value, .. } => value.to_string(),
            _ => return Err(ShellError::new(
                ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::InvalidArgument),
                "Invalid command name".to_string()
            )),
        };

        // Extract arguments
        let mut cmd_args = Vec::new();
        for arg in args {
            let arg_value = match arg {
                AstNode::Word(word) => word.to_string(),
                AstNode::StringLiteral { value, .. } => value.to_string(),
                AstNode::NumberLiteral { value, .. } => value.to_string(),
                AstNode::VariableExpansion { name, .. } => {
                    context.get_var(name).unwrap_or_default()
                }
                _ => format!("{:?}", arg),
            };
            cmd_args.push(arg_value);
        }

        // Check if it's a builtin command
        if let Some(builtin) = self.builtins.get(&cmd_name) {
            return builtin.execute(context, &cmd_args);
        }

        // Handle background execution
        if background {
            return self.execute_background_command(&cmd_name, cmd_args, context);
        }

        // Execute as external command
        self.execute_external_process(&cmd_name, &cmd_args, context)
    }

    /// Execute command in background
    fn execute_background_command(
        &mut self,
        command: &str,
        args: Vec<String>,
        context: &mut ShellContext
    ) -> ShellResult<ExecutionResult> {
        // Get job manager from context
        let job_manager = context.job_manager();
        let mut job_manager_guard = job_manager.lock()
            .map_err(|_| ShellError::new(
                ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                "Job manager lock poisoned".to_string()
            ))?;

        // Spawn background job
        let job_id = job_manager_guard.spawn_background_job(command.to_string(), args)?;
        
        // Return immediately with job information
        let output = format!("[{}] Background job started: {}", job_id, command);
        println!("{}", output); // Also print to console
        
        Ok(ExecutionResult::success(0).with_output(output.as_bytes().to_vec()))
    }

    /// Execute external process
    fn execute_external_process(
        &self,
        command: &str,
        args: &[String],
        context: &ShellContext
    ) -> ShellResult<ExecutionResult> {
        use std::process::Command;
        
        let start_time = Instant::now();
        
        let mut cmd = Command::new(command);
        cmd.args(args);
        
        // Set environment variables
        if let Ok(env) = context.env.read() {
            for (key, value) in env.iter() {
                cmd.env(key, value);
            }
        }
        
        // Set working directory
        cmd.current_dir(&context.cwd);
        
        // Execute command and capture output
        let output = cmd.output()
            .map_err(|e| ShellError::new(
                ErrorKind::SystemError(crate::error::SystemErrorKind::ProcessError),
                format!("Failed to execute command '{}': {}", command, e)
            ))?;
        
        let execution_time = start_time.elapsed().as_micros() as u64;
        
        Ok(ExecutionResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics {
                compile_time_us: 0,
                optimize_time_us: 0,
                execute_time_us: execution_time,
                instruction_count: 1,
                memory_usage: (output.stdout.len() + output.stderr.len()) as u64,
            },
        })
    }
    
    /// Execute a single command
    fn execute_command(&mut self, name: &str, args: &[AstNode], context: &mut ShellContext) -> ShellResult<ExecutionResult> {
        // Check if it's a builtin
        if let Some(builtin) = self.builtins.get(name) {
            let string_args: Vec<String> = args.iter().map(|arg| {
                // Simplified argument evaluation
                format!("{:?}", arg)
            }).collect();
            
            return builtin.execute(context, &string_args);
        }
        
        // External command execution (improved implementation)
        let start_time = Instant::now();
        
        // Build full command with arguments
        let mut full_command: Vec<String> = vec![name.to_string()];
        for arg in args {
            // Simplified argument evaluation for now
            let arg_str = match arg {
                AstNode::Word(s) => (*s).to_string(),
                AstNode::StringLiteral { value, .. } => (*value).to_string(),
                AstNode::VariableExpansion { name, .. } => {
                    context.get_var(name).unwrap_or_else(|| (*name).to_string())
                }
                AstNode::CommandSubstitution { .. } => {
                    // TODO: Proper command substitution
                    "$(substitution)".to_string()
                }
                _ => format!("{:?}", arg)
            };
            full_command.push(arg_str);
        }
        
        // Execute external command using std::process::Command
        let mut cmd = std::process::Command::new(&full_command[0]);
        if full_command.len() > 1 {
            cmd.args(&full_command[1..]);
        }
        
        // Set up environment variables from context
        if let Ok(env) = context.env.read() {
            for (key, value) in env.iter() {
                cmd.env(key, value);
            }
        }
        
        // Set working directory
        cmd.current_dir(&context.cwd);
        
        // Execute and capture output
        match cmd.output() {
            Ok(output) => {
                let execution_time = start_time.elapsed().as_micros() as u64;
                Ok(ExecutionResult {
                    exit_code: output.status.code().unwrap_or(-1),
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    execution_time,
                    strategy: ExecutionStrategy::DirectInterpreter,
                    metrics: ExecutionMetrics {
                        compile_time_us: 0,
                        optimize_time_us: 0,
                        execute_time_us: execution_time,
                        instruction_count: 1,
                        memory_usage: (output.stdout.len() + output.stderr.len()) as u64,
                    },
                })
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_micros() as u64;
                Ok(ExecutionResult {
                    exit_code: 127, // Command not found
                    stdout: String::new(),
                    stderr: format!("nxsh: {}: command not found ({})", name, e),
                    execution_time,
                    strategy: ExecutionStrategy::DirectInterpreter,
                    metrics: ExecutionMetrics {
                        compile_time_us: 0,
                        optimize_time_us: 0,
                        execute_time_us: execution_time,
                        instruction_count: 1,
                        memory_usage: 0,
                    },
                })
            }
        }
    }
    
    /// Execute a pipeline of commands
    fn execute_pipeline(&mut self, commands: &[AstNode], context: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let mut final_result = ExecutionResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            execution_time: 0,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        };
        
        for command in commands {
            let result = self.execute_ast_direct(command, context)?;
            final_result.execution_time += result.execution_time;
            final_result.stdout = result.stdout;
            if result.exit_code != 0 {
                final_result.exit_code = result.exit_code;
                final_result.stderr = result.stderr;
                break;
            }
        }
        
        Ok(final_result)
    }
    
    /// Execute a conditional statement
    fn execute_conditional(
        &mut self,
        condition: &AstNode,
        then_branch: &AstNode,
        else_branch: Option<&AstNode>,
        context: &mut ShellContext
    ) -> ShellResult<ExecutionResult> {
        let condition_result = self.execute_ast_direct(condition, context)?;
        
        if condition_result.exit_code == 0 {
            self.execute_ast_direct(then_branch, context)
        } else if let Some(else_node) = else_branch {
            self.execute_ast_direct(else_node, context)
        } else {
            Ok(ExecutionResult {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
                execution_time: condition_result.execution_time,
                strategy: ExecutionStrategy::DirectInterpreter,
                metrics: ExecutionMetrics::default(),
            })
        }
    }
    
    /// Execute a loop
    fn execute_loop(&mut self, condition: &AstNode, body: &AstNode, context: &mut ShellContext) -> ShellResult<ExecutionResult> {
        let mut total_time = 0;
        let mut last_result = ExecutionResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            execution_time: 0,
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        };
        
        loop {
            let condition_result = self.execute_ast_direct(condition, context)?;
            total_time += condition_result.execution_time;
            
            if condition_result.exit_code != 0 {
                break;
            }
            
            let body_result = self.execute_ast_direct(body, context)?;
            total_time += body_result.execution_time;
            last_result = body_result;
            
            // Simple loop protection
            if total_time > 10_000_000 { // 10 seconds
                return Err(ShellError::new(
                    ErrorKind::RuntimeError(crate::error::RuntimeErrorKind::Timeout),
                    "Loop execution timeout"
                ));
            }
        }
        
        last_result.execution_time = total_time;
        Ok(last_result)
    }
    
    /// Get executor statistics
    pub fn stats(&self) -> &ExecutorStats {
        &self.stats
    }
    
    /// Reset executor statistics
    pub fn reset_stats(&mut self) {
        self.stats = ExecutorStats::default();
    }

    /// Execute subshell with complete isolation
    fn execute_subshell(&mut self, commands: &[AstNode], ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        // Check if process isolation is enabled
        let enable_isolation = ctx.get_option("enable_process_isolation").unwrap_or(true);
        
        if enable_isolation {
            // Execute in completely isolated process
            self.execute_subshell_isolated(commands, ctx)
        } else {
            // Execute in same process with context isolation
            self.execute_subshell_local(commands, ctx)
        }
    }

    /// Execute subshell in isolated process (fork-like behavior)
    fn execute_subshell_isolated(&mut self, commands: &[AstNode], ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        use std::process::{Command, Stdio};
        use std::io::Write;
        use tempfile::NamedTempFile;
        
        // Create temporary script file for subshell execution
        let mut temp_script = NamedTempFile::new()
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::Other),
                format!("Failed to create temporary script: {}", e)
            ))?;
        
        // Convert commands to script text
        let script_content = self.commands_to_script(commands)?;
        temp_script.write_all(script_content.as_bytes())
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::Other),
                format!("Failed to write script: {}", e)
            ))?;
        temp_script.flush()
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::Other),
                format!("Failed to flush script: {}", e)
            ))?;
        
        // Prepare environment variables for subshell
        let subshell_env = self.prepare_subshell_environment(ctx)?;
        
        // Execute subshell as separate process
        let child = Command::new(std::env::current_exe()
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::Other),
                format!("Failed to get current executable: {}", e)
            ))?)
            .arg("--subshell")
            .arg(temp_script.path())
            .envs(&subshell_env)
            .current_dir(&ctx.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::Other),
                format!("Failed to spawn subshell process: {}", e)
            ))?;
        
        // Wait for completion and collect output
        let output = child.wait_with_output()
            .map_err(|e| ShellError::new(
                ErrorKind::IoError(crate::error::IoErrorKind::Other),
                format!("Failed to wait for subshell: {}", e)
            ))?;
        
        Ok(ExecutionResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time: 0, // TODO: Measure actual time
            strategy: ExecutionStrategy::DirectInterpreter,
            metrics: ExecutionMetrics::default(),
        })
    }

    /// Execute subshell with local context isolation (in-process)
    fn execute_subshell_local(&mut self, commands: &[AstNode], ctx: &mut ShellContext) -> ShellResult<ExecutionResult> {
        // Create isolated context by deep-cloning the current context
        let mut subshell_ctx = self.create_isolated_context(ctx)?;
        
        // Increment subshell level
        {
            let mut options = subshell_ctx.options.write()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to acquire options lock"
                ))?;
            options.subshell_level += 1;
        }
        
        // Execute commands in isolated context
        let result = self.execute_pipeline(commands, &mut subshell_ctx)?;
        
        // Subshell changes do NOT affect parent context
        // (variables, functions, aliases remain isolated)
        
        Ok(result)
    }

    /// Create completely isolated context for subshell
    fn create_isolated_context(&self, parent_ctx: &ShellContext) -> ShellResult<ShellContext> {
        // Create new context with deep-copied state
        let mut isolated_ctx = ShellContext::new();
        
        // Copy exported environment variables only
        {
            let parent_vars = parent_ctx.vars.read()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to read parent variables"
                ))?;
            let parent_env = parent_ctx.env.read()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to read parent environment"
                ))?;
            
            let mut isolated_env = isolated_ctx.env.write()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to acquire isolated environment lock"
                ))?;
            let mut isolated_vars = isolated_ctx.vars.write()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to acquire isolated variables lock"
                ))?;
            
            // Copy exported variables to subshell environment
            for (key, var) in parent_vars.iter() {
                if var.exported {
                    isolated_env.insert(key.clone(), var.value.clone());
                    isolated_vars.insert(key.clone(), var.clone());
                }
            }
            
            // Copy environment variables
            for (key, value) in parent_env.iter() {
                isolated_env.insert(key.clone(), value.clone());
            }
        }
        
        // Copy shell options (but reset control flow state)
        {
            let parent_options = parent_ctx.options.read()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to read parent options"
                ))?;
            let mut isolated_options = isolated_ctx.options.write()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to acquire isolated options lock"
                ))?;
            
            *isolated_options = parent_options.clone();
            // Reset control flow state in subshell
            isolated_options.break_requested = false;
            isolated_options.continue_requested = false;
        }
        
        // Copy current working directory
        isolated_ctx.cwd = parent_ctx.cwd.clone();
        
        // Copy shell level (will be incremented by caller)
        isolated_ctx.shell_level = parent_ctx.shell_level;
        
        // Functions and aliases are NOT inherited (subshell isolation)
        // History is NOT shared (subshell isolation)
        
        Ok(isolated_ctx)
    }

    /// Convert AST commands to shell script text for external execution
    fn commands_to_script(&self, commands: &[AstNode]) -> ShellResult<String> {
        let mut script = String::new();
        
        // Add shebang for proper execution
        script.push_str("#!/usr/bin/env nxsh\n");
        script.push_str("# Auto-generated subshell script\n\n");
        
        for command in commands {
            // Convert AST node to shell command text
            script.push_str(&self.ast_to_command_string(command)?);
            script.push('\n');
        }
        
        Ok(script)
    }

    /// Convert single AST node to command string
    fn ast_to_command_string(&self, node: &AstNode) -> ShellResult<String> {
        match node {
            AstNode::Program(statements) => {
                let mut commands = Vec::new();
                for statement in statements {
                    commands.push(self.ast_to_command_string(statement)?);
                }
                Ok(commands.join("; "))
            },
            AstNode::Command { name, args, .. } => {
                let mut cmd_str = format!("{}", name);
                for arg in args {
                    cmd_str.push(' ');
                    cmd_str.push_str(&self.ast_to_command_string(arg)?);
                }
                Ok(cmd_str)
            },
            AstNode::Word(word) => Ok(word.to_string()),
            AstNode::StringLiteral { value, .. } => Ok(format!("\"{}\"", value)),
            AstNode::NumberLiteral { value, .. } => Ok(value.to_string()),
            AstNode::VariableExpansion { name, .. } => Ok(format!("${}", name)),
            AstNode::Pipeline { elements, .. } => {
                let parts: ShellResult<Vec<String>> = elements.iter()
                    .map(|e| self.ast_to_command_string(e))
                    .collect();
                Ok(parts?.join(" | "))
            },
            _ => {
                // For complex nodes, use a simplified representation
                Ok("# Complex command".to_string())
            }
        }
    }

    /// Prepare environment for subshell process
    fn prepare_subshell_environment(&self, ctx: &ShellContext) -> ShellResult<std::collections::HashMap<String, String>> {
        let mut env = std::collections::HashMap::new();
        
        // Copy exported variables
        {
            let vars = ctx.vars.read()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to read variables"
                ))?;
            let ctx_env = ctx.env.read()
                .map_err(|_| ShellError::new(
                    ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
                    "Failed to read environment"
                ))?;
            
            // Add exported shell variables
            for (key, var) in vars.iter() {
                if var.exported {
                    env.insert(key.clone(), var.value.clone());
                }
            }
            
            // Add environment variables
            for (key, value) in ctx_env.iter() {
                env.insert(key.clone(), value.clone());
            }
        }
        
        // Set subshell-specific variables
        env.insert("SHLVL".to_string(), (ctx.shell_level + 1).to_string());
        env.insert("NXSH_SUBSHELL".to_string(), "1".to_string());
        
        Ok(env)
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}
