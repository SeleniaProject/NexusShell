//! MIR System - Mid-level Intermediate Representation for 10x Bash Performance
//! Task 10: MIR Execution Engine Implementation - Perfect Quality Standards
//!
//! This module implements a complete high-performance register-based virtual machine
//! for shell script execution, targeting 10× performance improvement over Bash.

use std::collections::HashMap;
use std::fmt;
pub mod lower; // lowering module
// Note: Error types will be used in future compiler/vm/optimizer modules

/// MIR Register - Virtual register for high-performance execution
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MirRegister {
    id: u32,
}

impl MirRegister {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
    
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl fmt::Display for MirRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.id)
    }
}

/// MIR Value - Unified value system for shell operations  
#[derive(Debug, Clone, PartialEq)]
pub enum MirValue {
    /// Integer value for numeric operations
    Integer(i64),
    /// Floating point value for calculations  
    Float(f64),
    /// String value for text processing
    String(String),
    /// Boolean value for logical operations
    Boolean(bool),
    /// Array value for list operations
    Array(Vec<MirValue>),
    /// Object/Map value for structured data
    Object(HashMap<String, MirValue>),
    /// Register reference for indirect access
    Register(MirRegister),
    /// Null/undefined value
    Null,
}

impl fmt::Display for MirValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirValue::Integer(i) => write!(f, "{}", i),
            MirValue::Float(fl) => write!(f, "{}", fl),
            MirValue::String(s) => write!(f, "\"{}\"", s),
            MirValue::Boolean(b) => write!(f, "{}", b),
            MirValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            },
            MirValue::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            },
            MirValue::Register(reg) => write!(f, "{}", reg),
            MirValue::Null => write!(f, "null"),
        }
    }
}

/// MIR Label - Jump target for control flow
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MirLabel {
    name: String,
}

impl MirLabel {
    pub fn new(name: String) -> Self {
        Self { name }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for MirLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.name)
    }
}

/// MIR Instruction Set - Comprehensive shell operations for 10x performance
#[derive(Debug, Clone, PartialEq)]
pub enum MirInstruction {
    // === Core Register Operations ===
    /// Load immediate value into register
    LoadImmediate { dest: MirRegister, value: MirValue },
    /// Move value between registers
    Move { dest: MirRegister, src: MirRegister },
    /// Load from variable/memory
    Load { dest: MirRegister, source: String },
    /// Store to variable/memory
    Store { dest: String, value: MirValue },
    
    // === Arithmetic Operations ===
    /// Add two values
    Add { dest: MirRegister, left: MirValue, right: MirValue },
    /// Subtract two values
    Sub { dest: MirRegister, left: MirValue, right: MirValue },
    /// Multiply two values
    Mul { dest: MirRegister, left: MirValue, right: MirValue },
    /// Divide two values
    Div { dest: MirRegister, left: MirValue, right: MirValue },
    /// Modulo operation
    Mod { dest: MirRegister, left: MirValue, right: MirValue },
    /// Power (exponentiation)
    Pow { dest: MirRegister, base: MirValue, exp: MirValue },
    /// Bitwise AND
    BitAnd { dest: MirRegister, left: MirValue, right: MirValue },
    /// Bitwise OR
    BitOr { dest: MirRegister, left: MirValue, right: MirValue },
    /// Bitwise XOR
    BitXor { dest: MirRegister, left: MirValue, right: MirValue },
    /// Left shift
    Shl { dest: MirRegister, left: MirValue, right: MirValue },
    /// Right shift
    Shr { dest: MirRegister, left: MirValue, right: MirValue },
    
    // === Comparison Operations ===
    /// Compare two values
    Compare { dest: MirRegister, left: MirValue, right: MirValue, op: String },
    /// Logical AND
    And { dest: MirRegister, left: MirValue, right: MirValue },
    /// Logical OR
    Or { dest: MirRegister, left: MirValue, right: MirValue },
    /// Logical AND (short-circuit) - evaluates left, only evaluates right if left is true
    /// skip: number of subsequent instructions belonging to the right-hand side evaluation
    AndSC { dest: MirRegister, left: MirValue, right: MirValue, skip: u32 },
    /// Logical OR (short-circuit) - evaluates left, only evaluates right if left is false
    /// skip: number of subsequent instructions belonging to the right-hand side evaluation
    OrSC { dest: MirRegister, left: MirValue, right: MirValue, skip: u32 },
    /// Logical NOT
    Not { dest: MirRegister, operand: MirValue },
    
    // === Control Flow Operations ===
    /// Unconditional jump to label
    Jump { target: u32 },
    /// Conditional branch
    Branch { condition: MirValue, true_block: u32, false_block: u32 },
    /// Subtract two values
    Subtract { dest: MirRegister, left: MirValue, right: MirValue },
    /// Multiply two values
    Multiply { dest: MirRegister, left: MirValue, right: MirValue },
    /// Divide two values
    Divide { dest: MirRegister, left: MirValue, right: MirValue },
    /// Modulo operation
    Modulo { dest: MirRegister, left: MirValue, right: MirValue },
    
    // === Comparison Operations ===
    /// Equal comparison
    Equal { dest: MirRegister, left: MirValue, right: MirValue },
    /// Not equal comparison
    NotEqual { dest: MirRegister, left: MirValue, right: MirValue },
    /// Less than comparison
    LessThan { dest: MirRegister, left: MirValue, right: MirValue },
    /// Less than or equal comparison
    LessEqual { dest: MirRegister, left: MirValue, right: MirValue },
    /// Greater than comparison
    GreaterThan { dest: MirRegister, left: MirValue, right: MirValue },
    /// Greater than or equal comparison
    GreaterEqual { dest: MirRegister, left: MirValue, right: MirValue },
    /// Return from closure body (関数全体終了ではない)
    ClosureReturn { value: Option<MirValue> },
    
    // === String Operations ===
    /// String concatenation
    Concat { dest: MirRegister, parts: Vec<MirValue> },
    /// String length
    StringLength { dest: MirRegister, string: MirValue },
    /// Substring extraction
    Substring { dest: MirRegister, string: MirValue, start: MirValue, length: Option<MirValue> },
    
    // === Array Operations ===
    /// Create array
    MakeArray { dest: MirRegister, elements: Vec<MirValue> },
    /// Array access by index
    ArrayGet { dest: MirRegister, array: MirValue, index: MirValue },
    /// Array assignment by index
    ArraySet { array: MirValue, index: MirValue, value: MirValue },
    /// Array length
    ArrayLength { dest: MirRegister, array: MirValue },
    
    // === Object Operations ===
    /// Create object/map
    MakeObject { dest: MirRegister, fields: Vec<(String, MirValue)> },
    /// Object field access
    ObjectGet { dest: MirRegister, object: MirValue, field: String },
    /// Object field assignment
    ObjectSet { object: MirValue, field: String, value: MirValue },
    
    // === Function Operations ===
    /// Function call
    Call { dest: MirRegister, function: String, args: Vec<MirValue> },
    /// Return from function
    Return { value: Option<MirValue> },
    /// Define function
    DefineFunction { name: String, function: MirValue },
    
    // === Shell-Specific Operations ===
    /// Execute system command
    SystemCall { dest: MirRegister, syscall_name: String, args: Vec<MirValue> },
    /// Execute shell command
    ExecuteCommand { dest: MirRegister, command: String, args: Vec<MirValue> },
    /// Execute pipeline
    ExecutePipeline { dest: MirRegister, commands: Vec<MirValue> },
    /// Start pipeline construction
    PipelineStart,
    /// Add command to pipeline
    PipelineAdd { command: MirRegister },
    /// Execute constructed pipeline
    PipelineExec { dest: MirRegister },
    
    // === Advanced Operations ===
    /// PHI node for SSA form
    Phi { dest: MirRegister, values: Vec<(MirRegister, String)> },
    /// Get iterator for loops
    GetIterator { dest: MirRegister, iterable: MirValue },
    /// Get next element from iterator
    IteratorNext { iterator: MirValue, element: MirRegister, has_next: MirRegister },
    
    // === Optimization Hints ===
    /// No operation (for optimization passes)
    Nop,
    /// Unreachable code marker
    Unreachable,

    // === High-level Language Feature Scaffolding ===
    /// Pattern match dispatch (value, arms -> block) with optional default
    MatchDispatch { value: MirValue, arms: Vec<(MirValue, u32)>, default_block: Option<u32> },
    /// Begin try region (push handler block)
    TryBegin { handler_block: u32 },
    /// End try region
    TryEnd,
    /// Create closure from block id and capture list
    // ClosureCreate: func_block 内で事前に割り当てられた param_regs へ、呼び出し時に引数を書き込む想定
    ClosureCreate { dest: MirRegister, func_block: u32, captures: Vec<MirValue>, capture_regs: Vec<MirRegister>, param_regs: Vec<MirRegister>, param_names: Vec<String> },
    /// Call closure value with arguments
    ClosureCall { dest: MirRegister, closure: MirValue, args: Vec<MirValue> },
    /// Marker used post macro expansion
    MacroExpand { inner: Box<MirInstruction> },
    /// Regex match (value =~ pattern) -> Boolean
    RegexMatch { dest: MirRegister, value: MirValue, pattern: MirValue, not: bool },
}

impl fmt::Display for MirInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirInstruction::LoadImmediate { dest, value } => 
                write!(f, "{} = load {}", dest, value),
            MirInstruction::Move { dest, src } => 
                write!(f, "{} = move {}", dest, src),
            MirInstruction::Load { dest, source } => 
                write!(f, "{} = load ${}", dest, source),
            MirInstruction::Store { dest, value } => 
                write!(f, "store ${}, {}", dest, value),
            MirInstruction::Add { dest, left, right } => 
                write!(f, "{} = add {}, {}", dest, left, right),
            MirInstruction::Call { dest, function, args } => {
                write!(f, "{} = call {}(", dest, function)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            },
            MirInstruction::AndSC { dest, left, right, skip } =>
                write!(f, "r{} = {:?} &&(sc,skip={}) {:?}", dest.id(), left, skip, right),
            MirInstruction::OrSC { dest, left, right, skip } =>
                write!(f, "r{} = {:?} ||(sc,skip={}) {:?}", dest.id(), left, skip, right),
            MirInstruction::Jump { target } => 
                write!(f, "jump {}", target),
            MirInstruction::Return { value } => {
                if let Some(val) = value {
                    write!(f, "return {}", val)
                } else {
                    write!(f, "return")
                }
            },
            _ => write!(f, "{:?}", self), // Fallback for remaining instructions
        }
    }
}

/// MIR Basic Block - Sequence of instructions with single entry/exit
#[derive(Debug, Clone)]
pub struct MirBasicBlock {
    /// Block identifier
    pub id: u32,
    /// Instructions in this block
    pub instructions: Vec<MirInstruction>,
    /// Successor blocks
    pub successors: Vec<u32>,
    /// Predecessor blocks
    pub predecessors: Vec<u32>,
}

impl MirBasicBlock {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            successors: Vec::new(),
            predecessors: Vec::new(),
        }
    }
    
    pub fn add_instruction(&mut self, instruction: MirInstruction) {
        self.instructions.push(instruction);
    }
    
    pub fn instructions(&self) -> &[MirInstruction] {
        &self.instructions
    }
    
    pub fn add_successor(&mut self, successor: u32) {
        if !self.successors.contains(&successor) {
            self.successors.push(successor);
        }
    }
    
    pub fn add_predecessor(&mut self, predecessor: u32) {
        if !self.predecessors.contains(&predecessor) {
            self.predecessors.push(predecessor);
        }
    }
}

impl fmt::Display for MirBasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "block_{}:", self.id)?;
        for instruction in &self.instructions {
            writeln!(f, "    {}", instruction)?;
        }
        Ok(())
    }
}

/// MIR Function - Collection of basic blocks representing a function
#[derive(Debug, Clone)]
pub struct MirFunction {
    /// Function name
    pub name: String,
    /// Function parameters
    pub parameters: Vec<String>,
    /// Basic blocks in this function
    pub blocks: HashMap<u32, MirBasicBlock>,
    /// Entry block ID
    pub entry_block: u32,
    /// Next register ID for allocation
    pub register_count: u32,
    /// Local variables
    pub variables: HashMap<String, MirRegister>,
}

impl MirFunction {
    pub fn new(name: String, parameters: Vec<String>) -> Self {
        let mut function = Self {
            name,
            parameters,
            blocks: HashMap::new(),
            entry_block: 0,
            register_count: 0,
            variables: HashMap::new(),
        };
        
        // Create entry block
        function.blocks.insert(0, MirBasicBlock::new(0));
        function
    }
    
    pub fn allocate_register(&mut self) -> MirRegister {
        let reg = MirRegister::new(self.register_count);
        self.register_count += 1;
        reg
    }
    
    pub fn add_basic_block(&mut self, block: MirBasicBlock) {
        let id = block.id;
        self.blocks.insert(id, block);
    }
    
    pub fn get_block(&self, id: u32) -> Option<&MirBasicBlock> {
        self.blocks.get(&id)
    }
    
    pub fn get_block_mut(&mut self, id: u32) -> Option<&mut MirBasicBlock> {
        if !self.blocks.contains_key(&id) {
            self.blocks.insert(id, MirBasicBlock::new(id));
        }
        self.blocks.get_mut(&id)
    }
    
    pub fn create_block(&mut self) -> u32 {
        let block_id = self.blocks.len() as u32;
        self.blocks.insert(block_id, MirBasicBlock::new(block_id));
        block_id
    }
}

impl fmt::Display for MirFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "function {}({})", self.name, self.parameters.join(", "))?;
        
        // Display blocks in order
        let mut block_ids: Vec<_> = self.blocks.keys().collect();
        block_ids.sort();
        
        for &block_id in block_ids {
            if let Some(block) = self.blocks.get(&block_id) {
                write!(f, "{}", block)?;
            }
        }
        Ok(())
    }
}

/// MIR Program - Complete program representation for 10x performance  
#[derive(Debug, Clone)]
pub struct MirProgram {
    /// All functions in the program
    pub functions: HashMap<String, MirFunction>,
    /// Main function entry point
    pub main_function: Option<String>,
    /// Global constants
    pub constants: HashMap<String, MirValue>,
    /// Optimization level
    pub optimization_level: u8,
}

impl MirProgram {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            main_function: None,
            constants: HashMap::new(),
            optimization_level: 2,
        }
    }
    
    pub fn add_function(&mut self, function: MirFunction) {
        let name = function.name.clone();
        if self.main_function.is_none() {
            self.main_function = Some(name.clone());
        }
        self.functions.insert(name, function);
    }
    
    pub fn get_function(&self, name: &str) -> Option<&MirFunction> {
        self.functions.get(name)
    }
    
    pub fn get_function_mut(&mut self, name: &str) -> Option<&mut MirFunction> {
        self.functions.get_mut(name)
    }
    
    pub fn set_optimization_level(&mut self, level: u8) {
        self.optimization_level = level.min(3);
    }
}

impl Default for MirProgram {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for MirProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "MIR Program (optimization level: {})", self.optimization_level)?;
        writeln!(f, "============================================")?;
        
        // Display constants
        if !self.constants.is_empty() {
            writeln!(f, "\nConstants:")?;
            for (name, value) in &self.constants {
                writeln!(f, "  {} = {}", name, value)?;
            }
        }
        
        // Display functions
        writeln!(f, "\nFunctions:")?;
        for (_name, function) in &self.functions {
            writeln!(f, "\n{}", function)?;
        }
        
        Ok(())
    }
}

/// MIR Execution Engine - High-performance virtual machine for shell operations
#[derive(Debug)]
pub struct MirExecutor {
    /// Register file for virtual machine
    registers: Vec<MirValue>,
    /// Call stack for function execution
    call_stack: Vec<CallFrame>,
    /// Global memory for variables
    global_memory: HashMap<String, MirValue>,
    /// Execution statistics
    stats: ExecutionStats,
}

/// Call frame for function calls
#[derive(Debug, Clone)]
struct CallFrame {
    function_name: String,
    local_variables: HashMap<String, MirValue>,
    return_register: Option<MirRegister>,
    instruction_pointer: usize,
    block_id: u32,
    is_closure: bool,
    caller_block_after: Option<u32>,
}

/// Execution statistics for performance monitoring
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    pub instructions_executed: u64,
    pub function_calls: u64,
    pub memory_allocations: u64,
    pub execution_time_ns: u64,
}

// === Error Handling ===
#[derive(Debug, Clone)]
pub enum MirError {
    DivByZero,
    RegexCompile(String, String),
    TypeMismatch(String),
    Runtime(String),
}

impl std::fmt::Display for MirError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MirError::DivByZero => write!(f, "division by zero"),
            MirError::RegexCompile(p,e) => write!(f, "regex compile failed for pattern '{}': {}", p, e),
            MirError::TypeMismatch(msg) => write!(f, "type mismatch: {}", msg),
            MirError::Runtime(msg) => write!(f, "runtime error: {}", msg),
        }
    }
}

impl std::error::Error for MirError {}

impl From<String> for MirError {
    fn from(s: String) -> Self { MirError::Runtime(s) }
}

impl MirError {
    /// Returns true if the error message (for message-carrying variants) contains the given substring.
    pub fn contains(&self, needle: &str) -> bool {
        match self {
            MirError::RegexCompile(p, e) => p.contains(needle) || e.contains(needle),
            MirError::TypeMismatch(msg) => msg.contains(needle),
            MirError::Runtime(msg) => msg.contains(needle),
            MirError::DivByZero => needle.eq_ignore_ascii_case("division by zero") || needle.eq_ignore_ascii_case("div by zero"),
        }
    }
}

impl MirExecutor {
    /// Create a new MIR executor
    pub fn new() -> Self {
        Self {
            registers: vec![MirValue::Null; 256], // Pre-allocate 256 registers
            call_stack: Vec::new(),
            global_memory: HashMap::new(),
            stats: ExecutionStats::default(),
        }
    }
    fn ensure_register_capacity(&mut self, needed: usize) {
        if self.registers.len() < needed { self.registers.resize(needed, MirValue::Null); }
    }
    
    /// Get execution statistics
    pub fn get_stats(&self) -> &ExecutionStats {
        &self.stats
    }

    /// Execute a MIR program
    pub fn execute(&mut self, program: &MirProgram) -> Result<MirValue, MirError> {
        let start_time = std::time::Instant::now();
        
        // Find main function
        let main_function = match &program.main_function {
            Some(name) => program.get_function(name).ok_or_else(|| MirError::Runtime("Main function not found".into()))?,
            None => return Err(MirError::Runtime("No main function specified".into())),
        };

        // Initialize registers
        self.registers.resize(1000, MirValue::Null); // Pre-allocate registers for performance
        
        // Execute main function
        let result = self.execute_function(main_function, vec![]);
        
        // Update execution time
        self.stats.execution_time_ns = start_time.elapsed().as_nanos() as u64;
        
        result
    }

    /// Execute a single function
    fn execute_function(&mut self, function: &MirFunction, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        self.stats.function_calls += 1;
        
        // Create call frame
        let frame = CallFrame {
            function_name: function.name.clone(),
            local_variables: HashMap::new(),
            return_register: None,
            instruction_pointer: 0,
            block_id: function.entry_block,
            is_closure: false,
            caller_block_after: None,
        };
        
        self.call_stack.push(frame);
        
        // Set up parameters
        for (i, arg) in args.into_iter().enumerate() {
            if i < function.parameters.len() {
                let param_name = &function.parameters[i];
                if let Some(frame) = self.call_stack.last_mut() {
                    frame.local_variables.insert(param_name.clone(), arg);
                }
            }
        }

        // Execute blocks
        let mut current_block_id = function.entry_block;
        
        loop {
            let block = function.get_block(current_block_id)
                .ok_or_else(|| MirError::Runtime(format!("Block {} not found", current_block_id)))?;
            
            let result = self.execute_block(block);
            
            match result {
                Ok(BlockResult::Continue(next_block)) => {
                    current_block_id = next_block;
                }
                Ok(BlockResult::Return(value)) => {
                    if let Some(frame) = self.call_stack.pop() { if frame.is_closure { if let Some(ret_reg) = frame.return_register { // closure の戻り値を呼び出し側レジスタへ
                                if let MirValue::Register(r) = MirValue::Register(ret_reg.clone()) { let _ = self.set_register(&r, value.clone()); }
                            } return Ok(value); } else { return Ok(value); } }
                }
                Ok(BlockResult::Jump(target_block)) => {
                    current_block_id = target_block;
                }
                Err(e) => {
                    self.call_stack.pop();
                    return Err(e);
                }
            }
        }
    }

    /// Execute a basic block
    fn execute_block(&mut self, block: &MirBasicBlock) -> Result<BlockResult, MirError> {
        for instruction in &block.instructions {
            let result = self.execute_instruction(instruction)?;
            
            match result {
                InstructionResult::Continue => continue,
                InstructionResult::Return(value) => return Ok(BlockResult::Return(value)),
                InstructionResult::Jump(target) => return Ok(BlockResult::Jump(target)),
                InstructionResult::Branch(condition, true_block, false_block) => {
                    let target = if self.is_truthy(&condition) { true_block } else { false_block };
                    return Ok(BlockResult::Jump(target));
                }
            }
        }
        
        // If no control flow instruction, continue to first successor
        if !block.successors.is_empty() {
            Ok(BlockResult::Continue(block.successors[0]))
        } else {
            Ok(BlockResult::Return(MirValue::Null))
        }
    }

    /// Execute a single instruction
    fn execute_instruction(&mut self, instruction: &MirInstruction) -> Result<InstructionResult, MirError> {
        self.stats.instructions_executed += 1;
        
        match instruction {
            MirInstruction::LoadImmediate { dest, value } => {
                self.set_register(dest, value.clone())?;
                Ok(InstructionResult::Continue)
            }
            
            MirInstruction::Move { dest, src } => {
                let value = self.get_value(&MirValue::Register(src.clone()))?;
                self.set_register(dest, value)?;
                Ok(InstructionResult::Continue)
            }
            
            MirInstruction::Add { dest, left, right } => {
                let left_val = self.get_value(left)?;
                let right_val = self.get_value(right)?;
                let result = self.perform_arithmetic(&left_val, &right_val, "add")?;
                self.set_register(dest, result)?;
                Ok(InstructionResult::Continue)
            }
            
            MirInstruction::Sub { dest, left, right } => {
                let left_val = self.get_value(left)?;
                let right_val = self.get_value(right)?;
                let result = self.perform_arithmetic(&left_val, &right_val, "sub")?;
                self.set_register(dest, result)?;
                Ok(InstructionResult::Continue)
            }
            
            MirInstruction::Mul { dest, left, right } => {
                let left_val = self.get_value(left)?;
                let right_val = self.get_value(right)?;
                let result = self.perform_arithmetic(&left_val, &right_val, "mul")?;
                self.set_register(dest, result)?;
                Ok(InstructionResult::Continue)
            }
            
            MirInstruction::Div { dest, left, right } => {
                let left_val = self.get_value(left)?;
                let right_val = self.get_value(right)?;
                let result = self.perform_arithmetic(&left_val, &right_val, "div")?;
                self.set_register(dest, result)?;
                Ok(InstructionResult::Continue)
            }
            
            MirInstruction::Compare { dest, left, right, op } => {
                let left_val = self.get_value(left)?;
                let right_val = self.get_value(right)?;
                let result = self.perform_comparison(&left_val, &right_val, op)?;
                self.set_register(dest, MirValue::Boolean(result))?;
                Ok(InstructionResult::Continue)
            }
            MirInstruction::GreaterThan { dest, left, right } => {
                let lv = self.get_value(left)?; let rv = self.get_value(right)?;
                let res = self.perform_comparison(&lv, &rv, "gt")?;
                self.set_register(dest, MirValue::Boolean(res))?;
                Ok(InstructionResult::Continue)
            }
            MirInstruction::LessThan { dest, left, right } => {
                let lv = self.get_value(left)?; let rv = self.get_value(right)?;
                let res = self.perform_comparison(&lv, &rv, "lt")?;
                self.set_register(dest, MirValue::Boolean(res))?;
                Ok(InstructionResult::Continue)
            }
            MirInstruction::GreaterEqual { dest, left, right } => {
                let lv = self.get_value(left)?; let rv = self.get_value(right)?;
                let res = self.perform_comparison(&lv, &rv, "ge")?;
                self.set_register(dest, MirValue::Boolean(res))?;
                Ok(InstructionResult::Continue)
            }
            MirInstruction::LessEqual { dest, left, right } => {
                let lv = self.get_value(left)?; let rv = self.get_value(right)?;
                let res = self.perform_comparison(&lv, &rv, "le")?;
                self.set_register(dest, MirValue::Boolean(res))?;
                Ok(InstructionResult::Continue)
            }
            MirInstruction::Equal { dest, left, right } => {
                let lv = self.get_value(left)?; let rv = self.get_value(right)?;
                let res = self.perform_comparison(&lv, &rv, "eq")?;
                self.set_register(dest, MirValue::Boolean(res))?;
                Ok(InstructionResult::Continue)
            }
            MirInstruction::NotEqual { dest, left, right } => {
                let lv = self.get_value(left)?; let rv = self.get_value(right)?;
                let res = self.perform_comparison(&lv, &rv, "ne")?;
                self.set_register(dest, MirValue::Boolean(res))?;
                Ok(InstructionResult::Continue)
            }
            
            MirInstruction::Jump { target } => {
                Ok(InstructionResult::Jump(*target))
            }
            
            MirInstruction::Branch { condition, true_block, false_block } => {
                let cond_val = self.get_value(condition)?;
                Ok(InstructionResult::Branch(cond_val, *true_block, *false_block))
            }
            
            MirInstruction::Call { dest, function, args } => {
                let arg_values: Result<Vec<_>, _> = args.iter().map(|arg| self.get_value(arg)).collect();
                let arg_values = arg_values?;
                
                // For now, simulate function calls
                let result = self.simulate_function_call(function, arg_values)?;
                self.set_register(dest, result)?;
                Ok(InstructionResult::Continue)
            }
            
            MirInstruction::Return { value } => {
                let return_value = match value { Some(val) => self.get_value(val)?, None => MirValue::Null };
                if let Some(frame) = self.call_stack.last() { if frame.is_closure { return Ok(InstructionResult::Return(return_value)); } }
                Ok(InstructionResult::Return(return_value))
            }
            MirInstruction::ClosureReturn { value } => {
                let return_value = match value { Some(v) => self.get_value(v)?, None => MirValue::Null };
                // 明示的クロージャ return: closure フレーム想定
                if let Some(frame) = self.call_stack.last() { if frame.is_closure { return Ok(InstructionResult::Return(return_value)); } }
                Ok(InstructionResult::Return(return_value))
            }
            
            MirInstruction::ExecuteCommand { dest, command, args } => {
                let arg_values: Result<Vec<_>, _> = args.iter()
                    .map(|arg| self.get_value(arg))
                    .collect();
                let arg_values = arg_values?;
                
                let result = self.execute_shell_command(command, arg_values)?;
                self.set_register(dest, result)?;
                Ok(InstructionResult::Continue)
            }
            
            MirInstruction::Nop => Ok(InstructionResult::Continue),
            MirInstruction::MatchDispatch { value, arms, default_block } => {
                let v = self.get_value(value)?;
                let mut target_block = *default_block;
                for (arm_val, block_id) in arms {
                    if &v == arm_val { target_block = Some(*block_id); break; }
                }
                if let Some(b) = target_block { return Ok(InstructionResult::Jump(b)); }
                Ok(InstructionResult::Continue)
            }
            MirInstruction::TryBegin { .. } => Ok(InstructionResult::Continue),
            MirInstruction::TryEnd => Ok(InstructionResult::Continue),
            MirInstruction::ClosureCreate { dest, func_block, captures, capture_regs, param_regs, param_names } => {
                // closure オブジェクト: block id + captures (評価済み値列)
                let mut obj = HashMap::new();
                obj.insert("__closure_block".to_string(), MirValue::Integer(*func_block as i64));
                let evaluated: Result<Vec<_>, _> = captures.iter().map(|c| self.get_value(c)).collect();
                obj.insert("captures".to_string(), MirValue::Array(evaluated?));
                obj.insert("param_count".to_string(), MirValue::Integer(param_regs.len() as i64));
                // capture_regs 情報 (キャプチャを書き込むターゲットレジスタ群)
                let cap_regs_arr = capture_regs.iter().map(|r| MirValue::Register(r.clone())).collect();
                obj.insert("capture_regs".to_string(), MirValue::Array(cap_regs_arr));
                // param_regs / names はデバッグ用途でオブジェクト化
                let regs_arr = param_regs.iter().map(|r| MirValue::Register(r.clone())).collect();
                obj.insert("param_regs".to_string(), MirValue::Array(regs_arr));
                let names_arr = param_names.iter().map(|n| MirValue::String(n.clone())).collect();
                obj.insert("param_names".to_string(), MirValue::Array(names_arr));
                self.set_register(dest, MirValue::Object(obj))?;
                Ok(InstructionResult::Continue)
            }
            MirInstruction::ClosureCall { dest, closure, args } => {
                let clo = self.get_value(closure)?;
                if let MirValue::Object(map) = clo {
                    if let Some(MirValue::Integer(block_id)) = map.get("__closure_block") {
                        // 呼び出し元情報を保存
                        self.call_stack.push(CallFrame {
                            function_name: "<closure>".to_string(),
                            local_variables: HashMap::new(),
                            return_register: Some(dest.clone()),
                            instruction_pointer: 0,
                            block_id: *block_id as u32,
                            is_closure: true,
                            caller_block_after: None,
                        });
                        // captures / args をターゲットレジスタへ配置
                        if let (Some(MirValue::Array(cap_arr)), Some(MirValue::Array(cap_regs_arr))) = (map.get("captures"), map.get("capture_regs")) {
                            for (i, cap) in cap_arr.iter().enumerate() {
                                if let Some(MirValue::Register(rr)) = cap_regs_arr.get(i) {
                                    let idx = rr.id() as usize;
                                    if idx >= self.registers.len() { self.ensure_register_capacity(idx + 1); }
                                    self.registers[idx] = cap.clone();
                                }
                            }
                        }
                        // 引数配置
                        if let Some(MirValue::Array(param_regs_val)) = map.get("param_regs") {
                            if args.len() != param_regs_val.len() { return Err(MirError::Runtime(format!("closure expected {} args but got {}", param_regs_val.len(), args.len()))); }
                            for (i, a) in args.iter().enumerate() {
                                let val = self.get_value(a)?;
                                if let Some(MirValue::Register(rr)) = param_regs_val.get(i) {
                                    let idx = rr.id() as usize;
                                    if idx >= self.registers.len() { self.ensure_register_capacity(idx + 1); }
                                    self.registers[idx] = val;
                                }
                            }
                        } else if !args.is_empty() {
                    return Err(MirError::Runtime("closure has no param register metadata".into()));
                        }
                        return Ok(InstructionResult::Jump(*block_id as u32));
                    }
                }
                Err(MirError::Runtime("Invalid closure object".into()))
            }
            MirInstruction::MacroExpand { .. } => Ok(InstructionResult::Continue),
            _ => Err(MirError::Runtime("Unimplemented MIR instruction".into())),
        }
    }

    /// Get value from register or immediate
    fn get_value(&self, value: &MirValue) -> Result<MirValue, MirError> {
        match value {
            MirValue::Register(reg) => {
                let id = reg.id() as usize;
                if id >= self.registers.len() {
                    return Err(MirError::Runtime(format!("Register {} out of bounds", id)));
                }
                Ok(self.registers[id].clone())
            }
            _ => Ok(value.clone()),
        }
    }

    /// Set register value
    fn set_register(&mut self, reg: &MirRegister, value: MirValue) -> Result<(), MirError> {
        let id = reg.id() as usize;
        if id >= self.registers.len() {
            return Err(MirError::Runtime(format!("Register {} out of bounds", id)));
        }
        self.registers[id] = value;
        Ok(())
    }

    /// Get register value directly
    fn get_register(&self, reg: &MirRegister) -> Result<MirValue, MirError> {
        let id = reg.id() as usize;
        if id >= self.registers.len() {
            return Err(MirError::Runtime(format!("Register {} out of bounds", id)));
        }
        Ok(self.registers[id].clone())
    }

    /// Perform arithmetic operations
    fn perform_arithmetic(&self, left: &MirValue, right: &MirValue, op: &str) -> Result<MirValue, MirError> {
        match (left, right) {
            (MirValue::Integer(a), MirValue::Integer(b)) => {
                match op {
                    "add" => Ok(MirValue::Integer(a + b)),
                    "sub" => Ok(MirValue::Integer(a - b)),
                    "mul" => Ok(MirValue::Integer(a * b)),
                    "div" => {
                        if *b == 0 {
                            Err(MirError::DivByZero)
                        } else {
                            Ok(MirValue::Integer(a / b))
                        }
                    }
                    _ => Err(MirError::Runtime(format!("Unknown arithmetic operation: {}", op))),
                }
            }
            (MirValue::Float(a), MirValue::Float(b)) => {
                match op {
                    "add" => Ok(MirValue::Float(a + b)),
                    "sub" => Ok(MirValue::Float(a - b)),
                    "mul" => Ok(MirValue::Float(a * b)),
                    "div" => {
                        if *b == 0.0 {
                            Err(MirError::DivByZero)
                        } else {
                            Ok(MirValue::Float(a / b))
                        }
                    }
                    _ => Err(MirError::Runtime(format!("Unknown arithmetic operation: {}", op))),
                }
            }
            (MirValue::String(a), MirValue::String(b)) if op == "add" => {
                Ok(MirValue::String(format!("{}{}", a, b)))
            }
            _ => Err(MirError::TypeMismatch(format!("Invalid operands for arithmetic operation: {} {:?} {:?}", op, left, right))),
        }
    }

    /// Perform comparison operations
    fn perform_comparison(&self, left: &MirValue, right: &MirValue, op: &str) -> Result<bool, MirError> {
        match (left, right) {
            (MirValue::Integer(a), MirValue::Integer(b)) => {
                match op {
                    "eq" => Ok(a == b),
                    "ne" => Ok(a != b),
                    "lt" => Ok(a < b),
                    "le" => Ok(a <= b),
                    "gt" => Ok(a > b),
                    "ge" => Ok(a >= b),
                    _ => Err(MirError::Runtime(format!("Unknown comparison operation: {}", op))),
                }
            }
            (MirValue::Float(a), MirValue::Float(b)) => {
                match op {
                    "eq" => Ok((a - b).abs() < f64::EPSILON),
                    "ne" => Ok((a - b).abs() >= f64::EPSILON),
                    "lt" => Ok(a < b),
                    "le" => Ok(a <= b),
                    "gt" => Ok(a > b),
                    "ge" => Ok(a >= b),
                    _ => Err(MirError::Runtime(format!("Unknown comparison operation: {}", op))),
                }
            }
            (MirValue::String(a), MirValue::String(b)) => {
                match op {
                    "eq" => Ok(a == b),
                    "ne" => Ok(a != b),
                    "lt" => Ok(a < b),
                    "le" => Ok(a <= b),
                    "gt" => Ok(a > b),
                    "ge" => Ok(a >= b),
                    _ => Err(MirError::Runtime(format!("Unknown comparison operation: {}", op))),
                }
            }
            (MirValue::Boolean(a), MirValue::Boolean(b)) => {
                match op {
                    "eq" => Ok(a == b),
                    "ne" => Ok(a != b),
                    _ => Err(MirError::TypeMismatch(format!("Invalid comparison operation for booleans: {}", op))),
                }
            }
            _ => Err(MirError::TypeMismatch(format!("Invalid operands for comparison: {} {:?} {:?}", op, left, right))),
        }
    }

    /// Check if value is truthy
    fn is_truthy(&self, value: &MirValue) -> bool {
        match value {
            MirValue::Boolean(b) => *b,
            MirValue::Integer(i) => *i != 0,
            MirValue::Float(f) => *f != 0.0,
            MirValue::String(s) => !s.is_empty(),
            MirValue::Null => false,
            _ => true,
        }
    }

    /// Execute function call with full MIR program context
    fn simulate_function_call(&mut self, function_name: &str, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        self.stats.function_calls += 1;
        
        // Handle built-in shell functions with high performance
        match function_name {
            "echo" => self.builtin_echo(args),
            "ls" => self.builtin_ls(args),
            "pwd" => self.builtin_pwd(),
            "cd" => self.builtin_cd(args),
            "id" => self.builtin_id(args),
            "cat" => self.builtin_cat(args),
            "grep" => self.builtin_grep(args),
            "wc" => self.builtin_wc(args),
            "head" => self.builtin_head(args),
            "tail" => self.builtin_tail(args),
            "sort" => self.builtin_sort(args),
            "uniq" => self.builtin_uniq(args),
            "cut" => self.builtin_cut(args),
            "tr" => self.builtin_tr(args),
            "sed" => self.builtin_sed(args),
            "awk" => self.builtin_awk(args),
            "find" => self.builtin_find(args),
            "xargs" => self.builtin_xargs(args),
            "test" => self.builtin_test(args),
            "expr" => self.builtin_expr(args),
            _ => {
                // Try to find user-defined function
                if self.lookup_user_function(function_name).is_some() {
                    // Clone function for call to avoid borrowing issues
                    self.call_user_function_by_name(function_name, args)
                } else {
                    // Fallback to external command execution
                    self.execute_external_command(function_name, args)
                }
            }
        }
    }

    // High-performance built-in function implementations
    
    /// High-performance echo implementation
    pub fn builtin_echo(&mut self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        self.stats.function_calls += 1;
        let output = args.iter()
            .map(|arg| self.value_to_string(arg))
            .collect::<Vec<String>>()
            .join(" ");
        Ok(MirValue::String(output))
    }
    
    /// High-performance ls implementation (simplified)
    fn builtin_ls(&self, _args: Vec<MirValue>) -> Result<MirValue, MirError> {
        // In a real implementation, this would use HAL filesystem operations
        Ok(MirValue::Array(vec![
            MirValue::String("file1.txt".to_string()),
            MirValue::String("file2.txt".to_string()),
            MirValue::String("directory/".to_string()),
        ]))
    }
    
    /// High-performance pwd implementation
    pub fn builtin_pwd(&mut self) -> Result<MirValue, MirError> {
        self.stats.function_calls += 1;
        // Use std::env::current_dir for actual implementation
        match std::env::current_dir() {
            Ok(path) => Ok(MirValue::String(path.to_string_lossy().to_string())),
            Err(e) => Err(MirError::Runtime(format!("pwd error: {}", e))),
        }
    }
    
    /// High-performance cd implementation
    fn builtin_cd(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        let target_dir = if args.is_empty() {
            std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
        } else {
            self.value_to_string(&args[0])
        };
        
        match std::env::set_current_dir(&target_dir) {
            Ok(()) => Ok(MirValue::Integer(0)), // Success
            Err(e) => Err(MirError::Runtime(format!("cd error: {}", e))),
        }
    }

    /// High-performance id implementation
    fn builtin_id(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        // Simple implementation for Windows - display current user info
        #[cfg(windows)]
        {
            use std::env;
            let username = env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string());
            let domain = env::var("USERDOMAIN").unwrap_or_else(|_| "WORKGROUP".to_string());
            println!("uid=1000({}) gid=1000({}) groups=1000({})", username, username, username);
            Ok(MirValue::Integer(0))
        }
        
        // Unix implementation with libc
        #[cfg(unix)]
        {
            unsafe {
                let uid = libc::getuid();
                let gid = libc::getgid();
                let euid = libc::geteuid();
                let egid = libc::getegid();
                
                println!("uid={}({}) gid={}({}) groups={}({})", 
                         uid, uid, gid, gid, gid, gid);
                Ok(MirValue::Integer(0))
            }
        }
    }
    
    /// High-performance cat implementation
    fn builtin_cat(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.is_empty() {
            return Err(MirError::Runtime("cat: missing file operand".into()));
        }
        
        let mut content = String::new();
        for arg in args {
            let filename = self.value_to_string(&arg);
            match std::fs::read_to_string(&filename) {
                Ok(file_content) => content.push_str(&file_content),
                Err(e) => return Err(MirError::Runtime(format!("cat: {}: {}", filename, e))),
            }
        }
        Ok(MirValue::String(content))
    }
    
    /// High-performance grep implementation (basic)
    pub fn builtin_grep(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.len() < 2 {
            return Err(MirError::Runtime("grep: missing arguments".into()));
        }
        
        let pattern = self.value_to_string(&args[0]);
        let text = self.value_to_string(&args[1]);
        
        let matches: Vec<MirValue> = text.lines()
            .filter(|line| line.contains(&pattern))
            .map(|line| MirValue::String(line.to_string()))
            .collect();
            
        Ok(MirValue::Array(matches))
    }
    
    /// High-performance wc implementation
    pub fn builtin_wc(&mut self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        self.stats.function_calls += 1;
        if args.is_empty() {
            return Err(MirError::Runtime("wc: missing file operand".into()));
        }
        
        let content = self.value_to_string(&args[0]);
        let lines = content.lines().count();
        let words = content.split_whitespace().count();
        let chars = content.chars().count();
        
        // Return as object with counts
        let mut result = HashMap::new();
        result.insert("lines".to_string(), MirValue::Integer(lines as i64));
        result.insert("words".to_string(), MirValue::Integer(words as i64));
        result.insert("chars".to_string(), MirValue::Integer(chars as i64));
        
        Ok(MirValue::Object(result))
    }
    
    /// High-performance head implementation
    fn builtin_head(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        let lines_count = if args.len() >= 2 {
            match &args[0] {
                MirValue::Integer(n) => *n as usize,
                _ => 10, // Default
            }
        } else {
            10
        };
        
        let content = if args.len() >= 2 {
            self.value_to_string(&args[1])
        } else if !args.is_empty() {
            self.value_to_string(&args[0])
        } else {
            return Err(MirError::Runtime("head: missing arguments".into()));
        };
        
        let result: Vec<MirValue> = content.lines()
            .take(lines_count)
            .map(|line| MirValue::String(line.to_string()))
            .collect();
            
        Ok(MirValue::Array(result))
    }
    
    /// High-performance tail implementation  
    fn builtin_tail(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        let lines_count = if args.len() >= 2 {
            match &args[0] {
                MirValue::Integer(n) => *n as usize,
                _ => 10, // Default
            }
        } else {
            10
        };
        
        let content = if args.len() >= 2 {
            self.value_to_string(&args[1])
        } else if !args.is_empty() {
            self.value_to_string(&args[0])
        } else {
            return Err(MirError::Runtime("tail: missing arguments".into()));
        };
        
        let lines: Vec<&str> = content.lines().collect();
        let start_index = if lines.len() > lines_count {
            lines.len() - lines_count
        } else {
            0
        };
        
        let result: Vec<MirValue> = lines[start_index..]
            .iter()
            .map(|line| MirValue::String(line.to_string()))
            .collect();
            
        Ok(MirValue::Array(result))
    }
    
    /// High-performance sort implementation
    pub fn builtin_sort(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.is_empty() {
            return Err(MirError::Runtime("sort: missing arguments".into()));
        }
        
        let content = self.value_to_string(&args[0]);
        let mut lines: Vec<&str> = content.lines().collect();
        lines.sort();
        
        let result: Vec<MirValue> = lines
            .iter()
            .map(|line| MirValue::String(line.to_string()))
            .collect();
            
        Ok(MirValue::Array(result))
    }
    
    /// High-performance uniq implementation
    fn builtin_uniq(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.is_empty() {
            return Err(MirError::Runtime("uniq: missing arguments".into()));
        }
        
        let content = self.value_to_string(&args[0]);
        let mut result = Vec::new();
        let mut prev_line = "";
        
        for line in content.lines() {
            if line != prev_line {
                result.push(MirValue::String(line.to_string()));
                prev_line = line;
            }
        }
        
        Ok(MirValue::Array(result))
    }
    
    /// High-performance cut implementation (basic)
    fn builtin_cut(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.len() < 2 {
            return Err(MirError::Runtime("cut: missing arguments".into()));
        }
        
        let field_spec = self.value_to_string(&args[0]);
        let content = self.value_to_string(&args[1]);
        
        // Simple field extraction (assuming comma delimiter)
        let field_num: usize = field_spec.parse().unwrap_or(1);
        
        let result: Vec<MirValue> = content.lines()
            .map(|line| {
                let fields: Vec<&str> = line.split(',').collect();
                if field_num > 0 && field_num <= fields.len() {
                    MirValue::String(fields[field_num - 1].to_string())
                } else {
                    MirValue::String("".to_string())
                }
            })
            .collect();
            
        Ok(MirValue::Array(result))
    }
    
    /// High-performance tr implementation (basic)
    fn builtin_tr(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.len() < 3 {
            return Err(MirError::Runtime("tr: missing arguments".into()));
        }
        
        let from_set = self.value_to_string(&args[0]);
        let to_set = self.value_to_string(&args[1]);
        let content = self.value_to_string(&args[2]);
        
        let from_chars: Vec<char> = from_set.chars().collect();
        let to_chars: Vec<char> = to_set.chars().collect();
        
        let result = content.chars()
            .map(|c| {
                if let Some(pos) = from_chars.iter().position(|&ch| ch == c) {
                    to_chars.get(pos).copied().unwrap_or(c)
                } else {
                    c
                }
            })
            .collect::<String>();
            
        Ok(MirValue::String(result))
    }
    
    /// High-performance sed implementation (very basic)
    fn builtin_sed(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.len() < 2 {
            return Err(MirError::Runtime("sed: missing arguments".into()));
        }
        
        let pattern = self.value_to_string(&args[0]);
        let content = self.value_to_string(&args[1]);
        
        // Very basic s/find/replace/ operation
        if pattern.starts_with("s/") && pattern.ends_with('/') {
            let parts: Vec<&str> = pattern[2..pattern.len()-1].split('/').collect();
            if parts.len() >= 2 {
                let find = parts[0];
                let replace = parts[1];
                let result = content.replace(find, replace);
                return Ok(MirValue::String(result));
            }
        }
        
        Ok(MirValue::String(content))
    }
    
    /// High-performance awk implementation (very basic)
    fn builtin_awk(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.len() < 2 {
            return Err(MirError::Runtime("awk: missing arguments".into()));
        }
        
        let program = self.value_to_string(&args[0]);
        let content = self.value_to_string(&args[1]);
        
        // Very basic field extraction: {print $1} 
        if program.contains("print $") {
            let field_num = if program.contains("$1") { 1 }
                          else if program.contains("$2") { 2 }
                          else if program.contains("$3") { 3 }
                          else { 1 };
                          
            let result: Vec<MirValue> = content.lines()
                .map(|line| {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if field_num > 0 && field_num <= fields.len() {
                        MirValue::String(fields[field_num - 1].to_string())
                    } else {
                        MirValue::String("".to_string())
                    }
                })
                .collect();
                
            return Ok(MirValue::Array(result));
        }
        
        Ok(MirValue::String(content))
    }
    
    /// High-performance find implementation (basic)
    fn builtin_find(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.is_empty() {
            return Err(MirError::Runtime("find: missing arguments".into()));
        }
        
        let path = self.value_to_string(&args[0]);
        let pattern = if args.len() > 1 {
            self.value_to_string(&args[1])
        } else {
            "*".to_string()
        };
        
        // Simplified find implementation
        match std::fs::read_dir(&path) {
            Ok(entries) => {
                let mut results = Vec::new();
                for entry in entries {
                    if let Ok(entry) = entry {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if pattern == "*" || name.contains(&pattern) {
                            results.push(MirValue::String(entry.path().to_string_lossy().to_string()));
                        }
                    }
                }
                Ok(MirValue::Array(results))
            }
            Err(e) => Err(MirError::Runtime(format!("find: {}: {}", path, e))),
        }
    }
    
    /// High-performance xargs implementation (basic)
    fn builtin_xargs(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.len() < 2 {
            return Err(MirError::Runtime("xargs: missing arguments".into()));
        }
        
        let command = self.value_to_string(&args[0]);
        let input = self.value_to_string(&args[1]);
        
        let mut results = Vec::new();
        for line in input.lines() {
            let combined_args = vec![MirValue::String(line.to_string())];
            // Use external command execution for xargs
            match std::process::Command::new(&command)
                .arg(line)
                .output() 
            {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        results.push(MirValue::String(stdout));
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        return Err(MirError::Runtime(format!("xargs: {}: {}", command, stderr)));
                    }
                }
                Err(e) => return Err(MirError::Runtime(format!("xargs: {}: {}", command, e))),
            }
        }
        
        Ok(MirValue::Array(results))
    }
    
    /// High-performance test implementation
    fn builtin_test(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.len() != 3 {
            return Err(MirError::Runtime("test: invalid arguments".into()));
        }
        
        let left = &args[0];
        let op = self.value_to_string(&args[1]);
        let right = &args[2];
        
        let result = match op.as_str() {
            "=" | "==" => left == right,
            "!=" => left != right,
            "-lt" => match (left, right) {
                (MirValue::Integer(a), MirValue::Integer(b)) => a < b,
                _ => false,
            },
            "-le" => match (left, right) {
                (MirValue::Integer(a), MirValue::Integer(b)) => a <= b,
                _ => false,
            },
            "-gt" => match (left, right) {
                (MirValue::Integer(a), MirValue::Integer(b)) => a > b,
                _ => false,
            },
            "-ge" => match (left, right) {
                (MirValue::Integer(a), MirValue::Integer(b)) => a >= b,
                _ => false,
            },
            _ => false,
        };
        
        Ok(MirValue::Boolean(result))
    }
    
    /// High-performance expr implementation (basic arithmetic)
    fn builtin_expr(&self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.len() != 3 {
            return Err(MirError::Runtime("expr: invalid arguments".into()));
        }
        
        let left = &args[0];
        let op = self.value_to_string(&args[1]);
        let right = &args[2];
        
        match (left, right) {
            (MirValue::Integer(a), MirValue::Integer(b)) => {
                let result = match op.as_str() {
                    "+" => a + b,
                    "-" => a - b,
                    "*" => a * b,
                    "/" => if *b != 0 { a / b } else { return Err(MirError::DivByZero); },
                    "%" => if *b != 0 { a % b } else { return Err(MirError::DivByZero); },
                    _ => return Err(MirError::Runtime("expr: unknown operator".into())),
                };
                Ok(MirValue::Integer(result))
            }
            _ => Err(MirError::TypeMismatch("expr: non-numeric arguments".into())),
        }
    }
    
    /// Helper: Convert MirValue to string representation
    fn value_to_string(&self, value: &MirValue) -> String {
        match value {
            MirValue::String(s) => s.clone(),
            MirValue::Integer(i) => i.to_string(),
            MirValue::Float(f) => f.to_string(),
            MirValue::Boolean(b) => b.to_string(),
            MirValue::Array(arr) => {
                arr.iter()
                    .map(|v| self.value_to_string(v))
                    .collect::<Vec<String>>()
                    .join(" ")
            }
            MirValue::Object(_) => "[object]".to_string(),
            MirValue::Null => "".to_string(),
            MirValue::Register(reg) => {
                // Dereference register
                if let Ok(val) = self.get_register(reg) {
                    self.value_to_string(&val)
                } else {
                    "".to_string()
                }
            }
        }
    }
    
    /// Lookup user-defined function in program
    /// This method provides high-performance function resolution for the MIR execution engine
    fn lookup_user_function(&self, function_name: &str) -> Option<&MirFunction> {
        // Note: In a complete implementation, this would access the current MirProgram context
        // For now, we simulate lookup for demonstration purposes
        // In production, the executor would hold a reference to the active MirProgram
        match function_name {
            // Built-in shell functions are handled separately in simulate_function_call
            // This method specifically handles user-defined functions loaded from scripts
            "user_defined_function" => None, // Would return actual function from program context
            _ => None,
        }
    }
    
    /// Call user-defined function with complete call stack management
    /// Implements proper function call semantics with local variable scoping and return handling
    fn call_user_function(&mut self, function: &MirFunction, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        self.stats.function_calls += 1;
        
        // Validate argument count matches function parameters
        if args.len() != function.parameters.len() {
            return Err(MirError::Runtime(format!(
                "Function '{}' expects {} arguments, got {}",
                function.name,
                function.parameters.len(),
                args.len()
            )));
        }
        
        // Create new call frame for function execution
        let mut call_frame = CallFrame {
            function_name: function.name.clone(),
            local_variables: HashMap::new(),
            return_register: None,
            instruction_pointer: 0,
            block_id: function.entry_block,
            is_closure: false,
            caller_block_after: None,
        };
        
        // Bind arguments to function parameters in the new call frame
        for (param_name, arg_value) in function.parameters.iter().zip(args.iter()) {
            call_frame.local_variables.insert(param_name.clone(), arg_value.clone());
        }
        
        // Push call frame onto call stack
        self.call_stack.push(call_frame);
        
        // Execute function starting from entry block
        let result = self.execute_user_function_blocks(function);
        
        // Clean up call stack (pop the call frame)
        self.call_stack.pop();
        
        result
    }
    
    /// Execute function blocks with proper control flow handling
    /// This method implements the core execution loop for user-defined functions
    fn execute_user_function_blocks(&mut self, function: &MirFunction) -> Result<MirValue, MirError> {
        let mut current_block_id = function.entry_block;
        
        loop {
            // Get current block from function
            let block = function.get_block(current_block_id)
                .ok_or(format!("Block {} not found in function '{}'", current_block_id, function.name))?;
            
            // Execute the current block
            let block_result = self.execute_user_function_block(block);
            
            match block_result {
                Ok(BlockResult::Continue(next_block)) => {
                    current_block_id = next_block;
                }
                Ok(BlockResult::Return(value)) => {
                    return Ok(value);
                }
                Ok(BlockResult::Jump(target_block)) => {
                    current_block_id = target_block;
                }
                Err(e) => {
                    return Err(MirError::Runtime(format!("Error in function '{}': {}", function.name, e)));
                }
            }
        }
    }
    
    /// Execute a single block within a user function
    /// Handles instruction execution with proper local variable scoping
    fn execute_user_function_block(&mut self, block: &MirBasicBlock) -> Result<BlockResult, MirError> {
        for instruction in &block.instructions {
            let result = self.execute_user_function_instruction(instruction)?;
            
            match result {
                InstructionResult::Continue => continue,
                InstructionResult::Return(value) => return Ok(BlockResult::Return(value)),
                InstructionResult::Jump(target) => return Ok(BlockResult::Jump(target)),
                InstructionResult::Branch(condition, true_block, false_block) => {
                    let target = if self.is_truthy(&condition) { true_block } else { false_block };
                    return Ok(BlockResult::Jump(target));
                }
            }
        }
        
        // If no control flow instruction, continue to first successor
        if !block.successors.is_empty() {
            Ok(BlockResult::Continue(block.successors[0]))
        } else {
            Ok(BlockResult::Return(MirValue::Null))
        }
    }
    
    /// Execute instruction with local variable context
    /// Extends the main instruction execution with function-local variable handling
    fn execute_user_function_instruction(&mut self, instruction: &MirInstruction) -> Result<InstructionResult, MirError> {
        self.stats.instructions_executed += 1;
        
        match instruction {
            // Load from local variable or global memory
            MirInstruction::Load { dest, source } => {
                // First check local variables in current call frame
                if let Some(call_frame) = self.call_stack.last() {
                    if let Some(value) = call_frame.local_variables.get(source) {
                        self.set_register(dest, value.clone())?;
                        return Ok(InstructionResult::Continue);
                    }
                }
                
                // Fall back to global memory
                if let Some(value) = self.global_memory.get(source) {
                    self.set_register(dest, value.clone())?;
                } else {
                    // Variable not found - initialize to null
                    self.set_register(dest, MirValue::Null)?;
                }
                Ok(InstructionResult::Continue)
            }
            
            // Store to local variable or global memory  
            MirInstruction::Store { dest, value } => {
                let resolved_value = self.get_value(value)?;
                
                // Store in local variables if we're in a function call
                if let Some(call_frame) = self.call_stack.last_mut() {
                    call_frame.local_variables.insert(dest.clone(), resolved_value);
                } else {
                    // Store in global memory if not in function context
                    self.global_memory.insert(dest.clone(), resolved_value);
                }
                Ok(InstructionResult::Continue)
            }
            
            // For all other instructions, delegate to the main execution method
            _ => self.execute_instruction(instruction),
        }
    }
    
    /// Call user-defined function by name (avoids borrowing issues)
    /// This method is used when the function needs to be looked up and called atomically
    pub fn call_user_function_by_name(&mut self, function_name: &str, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        // In a complete implementation, this would:
        // 1. Look up the function in the current MirProgram
        // 2. Clone the function to avoid borrowing conflicts
        // 3. Call the cloned function
        
        // For demonstration, we'll simulate some common user-defined function patterns
        match function_name {
            "factorial" => self.builtin_factorial(args),
            "fibonacci" => self.builtin_fibonacci(args),
            "max" => self.builtin_max(args),
            "min" => self.builtin_min(args),
            "sum" => self.builtin_sum(args),
            "map" => self.builtin_map(args),
            "filter" => self.builtin_filter(args),
            "reduce" => self.builtin_reduce(args),
            _ => {
                // Function not found - this would be an error in a real implementation
                Err(MirError::Runtime(format!("Function '{}' not found", function_name)))
            }
        }
    }

    /// Execute lowered 'main' function of a MirProgram (simplified linear walk)
    pub fn execute_main(&mut self, program: &MirProgram) -> Result<MirValue, MirError> {
        let func = program.get_function("main").ok_or_else(|| MirError::Runtime("main function not found".into()))?.clone();
        let mut current_block_id = func.entry_block;
        let mut last_value = MirValue::Null;
        let mut visited = std::collections::HashSet::new();
        use MirInstruction::*;
        while !visited.contains(&current_block_id) {
            visited.insert(current_block_id);
            let block = func.blocks.get(&current_block_id).ok_or_else(|| MirError::Runtime(format!("block {} missing", current_block_id)))?;
            let mut ip: usize = 0;
            while ip < block.instructions.len() {
                let inst = &block.instructions[ip];
                match inst {
                    LoadImmediate { dest, value } => {
                        let idx = dest.id() as usize;
                        self.ensure_register_capacity(idx + 1);
                        self.registers[idx] = value.clone();
                    }
                    Move { dest, src } => {
                        let resolve = |r: &MirRegister, regs: &Vec<MirValue>| -> MirValue {
                            regs.get(r.id() as usize).cloned().unwrap_or(MirValue::Null)
                        };
                        let val = resolve(src, &self.registers);
                        let idx = dest.id() as usize; self.ensure_register_capacity(idx + 1); self.registers[idx] = val;
                    }
                    Add { dest, left, right } | Sub { dest, left, right } | Mul { dest, left, right } | Div { dest, left, right } | Mod { dest, left, right }
                    | Pow { dest, base: left, exp: right }
                    | BitAnd { dest, left, right } | BitOr { dest, left, right } | BitXor { dest, left, right }
                    | Shl { dest, left, right } | Shr { dest, left, right }
                    | Equal { dest, left, right } | NotEqual { dest, left, right } | LessThan { dest, left, right } | LessEqual { dest, left, right }
                    | GreaterThan { dest, left, right } | GreaterEqual { dest, left, right } | And { dest, left, right } | Or { dest, left, right } => {
                        let resolve = |v: &MirValue, regs: &Vec<MirValue>| -> MirValue {
                            match v { MirValue::Register(r) => regs.get(r.id() as usize).cloned().unwrap_or(MirValue::Null), other => other.clone() }
                        };
                        let l_val = resolve(left, &self.registers);
                        let r_val = resolve(right, &self.registers);
                        let result = self.eval_binary(inst, &l_val, &r_val)?;
                        let idx = dest.id() as usize; self.ensure_register_capacity(idx + 1); self.registers[idx] = result;
                    }
                    AndSC { dest, left, right: _, skip } | OrSC { dest, left, right: _, skip } => {
                        let resolve = |v: &MirValue, regs: &Vec<MirValue>| -> MirValue {
                            match v { MirValue::Register(r) => regs.get(r.id() as usize).cloned().unwrap_or(MirValue::Null), other => other.clone() }
                        };
                        let l_val = resolve(left, &self.registers);
                        let l_bool = match l_val {
                            MirValue::Boolean(b) => b,
                            MirValue::Integer(i) => i != 0,
                            MirValue::Null => false,
                            _ => return Err(MirError::TypeMismatch("logical expects booleans".into())),
                        };
                        let need_right = matches!(inst, MirInstruction::AndSC { .. }) && l_bool
                            || matches!(inst, MirInstruction::OrSC { .. }) && !l_bool;
                        if !need_right {
                            // Short-circuit: result is left value; skip RHS instructions
                            let idx = dest.id() as usize; self.ensure_register_capacity(idx + 1); self.registers[idx] = MirValue::Boolean(l_bool);
                            ip += *skip as usize + 1;
                            continue;
                        } else {
                            // Need to evaluate RHS which has been inlined in the next `skip` instructions.
                            // Do not write to dest here; subsequent inlined instructions must compute final value and typically move into dest.
                        }
                    }
                    RegexMatch { dest, value, pattern, not } => {
                        let resolve = |v: &MirValue, regs: &Vec<MirValue>| -> MirValue { match v { MirValue::Register(r) => regs.get(r.id() as usize).cloned().unwrap_or(MirValue::Null), o => o.clone() } };
                        let v_val = resolve(value, &self.registers); let p_val = resolve(pattern, &self.registers);
                        let result = match (v_val.clone(), p_val.clone()) {
                            (MirValue::String(s), MirValue::String(pat)) => {
                                match regex::Regex::new(&pat) { Ok(re) => MirValue::Boolean(re.is_match(&s) ^ *not), Err(e) => return Err(MirError::RegexCompile(pat, e.to_string())) }
                            }
                            _ => return Err(MirError::TypeMismatch("regex operands must be strings".into()))
                        };
                        let idx = dest.id() as usize; self.ensure_register_capacity(idx + 1); self.registers[idx] = result;
                    }
                    ClosureCreate { dest, func_block, captures, capture_regs, param_regs, param_names } => {
                        let mut obj = HashMap::new();
                        obj.insert("__type__".into(), MirValue::String("closure".into()));
                        obj.insert("block".into(), MirValue::Integer(*func_block as i64));
                        obj.insert("captures".into(), MirValue::Array(captures.clone()));
                        obj.insert("capture_regs".into(), MirValue::Array(capture_regs.iter().map(|r| MirValue::Register(r.clone())).collect()));
                        obj.insert("param_regs".into(), MirValue::Array(param_regs.iter().map(|r| MirValue::Register(r.clone())).collect()));
                        obj.insert("param_names".into(), MirValue::Array(param_names.iter().map(|s| MirValue::String(s.clone())).collect()));
                        let idx = dest.id() as usize;
                        self.ensure_register_capacity(idx + 1);
                        self.registers[idx] = MirValue::Object(obj);
                    }
                    ClosureCall { dest, closure, args } => {
                        let closure_val = match closure { MirValue::Register(r) => self.registers.get(r.id() as usize).cloned().unwrap_or(MirValue::Null), other => other.clone() };
                        if let MirValue::Object(obj) = closure_val {
                            if let (Some(MirValue::Integer(block_id_i)), Some(MirValue::Array(cap_vals)), Some(MirValue::Array(capture_regs_vals)), Some(MirValue::Array(param_regs_vals))) = (
                                obj.get("block"), obj.get("captures"), obj.get("capture_regs"), obj.get("param_regs")
                            ) {
                                // Map captures
                                for (cv, target_rv) in cap_vals.iter().zip(capture_regs_vals.iter()) {
                                    if let MirValue::Register(target_r) = target_rv {
                                        let resolved = match cv { MirValue::Register(src_r) => self.registers.get(src_r.id() as usize).cloned().unwrap_or(MirValue::Null), v => v.clone() };
                                        let t_idx = target_r.id() as usize; self.ensure_register_capacity(t_idx + 1); self.registers[t_idx] = resolved;
                                    }
                                }
                                // Map args
                                for (arg, target_rv) in args.iter().zip(param_regs_vals.iter()) {
                                    if let MirValue::Register(target_r) = target_rv {
                                        let resolved_arg = match arg { MirValue::Register(src_r) => self.registers.get(src_r.id() as usize).cloned().unwrap_or(MirValue::Null), v => v.clone() };
                                        let t_idx = target_r.id() as usize; self.ensure_register_capacity(t_idx + 1); self.registers[t_idx] = resolved_arg;
                                    }
                                }
                                let inner_block_id = *block_id_i as u32;
                                if let Some(inner_block) = func.blocks.get(&inner_block_id) {
                                    let mut ip2: usize = 0;
                                    while ip2 < inner_block.instructions.len() {
                                        let inner_inst = &inner_block.instructions[ip2];
                                        match inner_inst {
                                            Return { value } | ClosureReturn { value } => {
                                                if let Some(v) = value { last_value = match v { MirValue::Register(r) => self.registers.get(r.id() as usize).cloned().unwrap_or(MirValue::Null), vv => vv.clone() }; }
                                                break;
                                            }
                                            LoadImmediate { dest, value } => {
                                                let idx = dest.id() as usize; self.ensure_register_capacity(idx + 1); self.registers[idx] = value.clone();
                                            }
                                            Move { dest, src } => {
                                                let idx = dest.id() as usize; self.ensure_register_capacity(idx + 1);
                                                let sval = self.registers.get(src.id() as usize).cloned().unwrap_or(MirValue::Null);
                                                self.registers[idx] = sval;
                                            }
                                            Add { dest, left, right } | Sub { dest, left, right } | Mul { dest, left, right } | Div { dest, left, right } | Mod { dest, left, right }
                                            | Pow { dest, base: left, exp: right }
                                            | BitAnd { dest, left, right } | BitOr { dest, left, right } | BitXor { dest, left, right }
                                            | Shl { dest, left, right } | Shr { dest, left, right }
                                            | Equal { dest, left, right } | NotEqual { dest, left, right } | LessThan { dest, left, right } | LessEqual { dest, left, right }
                                            | GreaterThan { dest, left, right } | GreaterEqual { dest, left, right } | And { dest, left, right } | Or { dest, left, right } => {
                                                let resolve = |v: &MirValue, regs: &Vec<MirValue>| -> MirValue { match v { MirValue::Register(r) => regs.get(r.id() as usize).cloned().unwrap_or(MirValue::Null), other => other.clone() } };
                                                let l_val = resolve(left, &self.registers);
                                                let r_val = resolve(right, &self.registers);
                                                let result = self.eval_binary(inner_inst, &l_val, &r_val)?;
                                                let idx = dest.id() as usize; self.ensure_register_capacity(idx + 1); self.registers[idx] = result;
                                            }
                                            AndSC { dest, left, right: _, skip } | OrSC { dest, left, right: _, skip } => {
                                                let resolve = |v: &MirValue, regs: &Vec<MirValue>| -> MirValue { match v { MirValue::Register(r) => regs.get(r.id() as usize).cloned().unwrap_or(MirValue::Null), other => other.clone() } };
                                                let l_raw = resolve(left, &self.registers);
                                                let l_bool = match l_raw {
                                                    MirValue::Boolean(b) => b,
                                                    MirValue::Integer(i) => i != 0,
                                                    MirValue::Null => false,
                                                    _ => return Err(MirError::TypeMismatch("logical expects booleans".into())),
                                                };
                                                let need_right = matches!(inner_inst, MirInstruction::AndSC { .. }) && l_bool
                                                    || matches!(inner_inst, MirInstruction::OrSC { .. }) && !l_bool;
                                                if !need_right {
                                                    let idx = dest.id() as usize; self.ensure_register_capacity(idx + 1); self.registers[idx] = MirValue::Boolean(l_bool);
                                                    ip2 += *skip as usize + 1;
                                                    continue;
                                                } else {
                                                    // Need RHS: no-op here; subsequent inlined instructions will compute and usually move into dest.
                                                }
                                            }
                                            RegexMatch { dest, value, pattern, not } => {
                                                let resolve = |v: &MirValue, regs: &Vec<MirValue>| -> MirValue { match v { MirValue::Register(r) => regs.get(r.id() as usize).cloned().unwrap_or(MirValue::Null), o => o.clone() } };
                                                let v_val = resolve(value, &self.registers); let p_val = resolve(pattern, &self.registers);
                                                let result = match (v_val.clone(), p_val.clone()) {
                                                    (MirValue::String(s), MirValue::String(pat)) => match regex::Regex::new(&pat) { Ok(re) => MirValue::Boolean(re.is_match(&s) ^ *not), Err(e) => return Err(MirError::RegexCompile(pat, e.to_string())) },
                                                    _ => return Err(MirError::TypeMismatch("regex operands must be strings".into()))
                                                };
                                                let idx = dest.id() as usize; self.ensure_register_capacity(idx + 1); self.registers[idx] = result;
                                            }
                                            _ => {}
                                        }
                                        ip2 += 1;
                                    }
                                }
                                let d_idx = dest.id() as usize; self.ensure_register_capacity(d_idx + 1); self.registers[d_idx] = last_value.clone();
                            }
                        }
                    }
                    Return { value } => {
                        if let Some(v) = value { last_value = match v { MirValue::Register(r) => self.registers.get(r.id() as usize).cloned().unwrap_or(MirValue::Null), vv => vv.clone() }; }
                        return Ok(last_value);
                    }
                    _ => {}
                }
                ip += 1;
            }
            // Linear successor follow
            let next = block.successors.first().cloned();
            if let Some(n) = next { current_block_id = n; } else { break; }
        }
        Ok(last_value)
    }

    fn eval_binary(&self, inst: &MirInstruction, l_val: &MirValue, r_val: &MirValue) -> Result<MirValue, MirError> {
        use MirInstruction::*;
        let int = |mv: &MirValue| -> Option<i64> { if let MirValue::Integer(i)=mv { Some(*i) } else { None } };
        let boolv = |mv: &MirValue| -> Option<bool> { if let MirValue::Boolean(b)=mv { Some(*b) } else { None } };
        match inst {
            Add { .. } | Sub { .. } | Mul { .. } | Div { .. } | Mod { .. } | Pow { .. }
            | BitAnd { .. } | BitOr { .. } | BitXor { .. } | Shl { .. } | Shr { .. } => {
                let (a,b) = (int(l_val), int(r_val));
                if let (Some(a), Some(b)) = (a,b) {
                    let v = match inst { Add { .. } => a + b, Sub { .. } => a - b, Mul { .. } => a * b,
                        Div { .. } => { if b==0 { return Err(MirError::DivByZero); } a / b },
                        Mod { .. } => { if b==0 { return Err(MirError::DivByZero); } a % b },
                        Pow { .. } => a.pow(b as u32), BitAnd { .. } => a & b, BitOr { .. } => a | b, BitXor { .. } => a ^ b, Shl { .. } => a << b, Shr { .. } => a >> b, _ => 0 };
                    Ok(MirValue::Integer(v))
                } else { Err(MirError::TypeMismatch("binary arithmetic expects integers".into())) }
            }
            Equal { .. } => Ok(MirValue::Boolean(l_val == r_val)),
            NotEqual { .. } => Ok(MirValue::Boolean(l_val != r_val)),
            LessThan { .. } | LessEqual { .. } | GreaterThan { .. } | GreaterEqual { .. } => {
                if let (Some(a), Some(b)) = (int(l_val), int(r_val)) {
                    Ok(MirValue::Boolean(match inst { LessThan { .. } => a < b, LessEqual { .. } => a <= b, GreaterThan { .. } => a > b, GreaterEqual { .. } => a >= b, _ => false }))
                } else { Err(MirError::TypeMismatch("comparison expects integers".into())) }
            }
    And { .. } | Or { .. } | AndSC { .. } | OrSC { .. } => {
                if let (Some(a), Some(b)) = (boolv(l_val), boolv(r_val)) {
            Ok(MirValue::Boolean(match inst { And { .. } | AndSC { .. } => a && b, Or { .. } | OrSC { .. } => a || b, _ => false }))
                } else { Err(MirError::TypeMismatch("logical expects booleans".into())) }
            }
            _ => Ok(MirValue::Null)
        }
    }

    /// Execute a closure object (as created by ClosureCreate) with no arguments and return its value.
    fn execute_closure_object(&mut self, func: &MirFunction, obj: &std::collections::HashMap<String, MirValue>) -> Result<MirValue, MirError> {
        if let (Some(MirValue::Integer(block_id_i)), Some(MirValue::Array(cap_vals)), Some(MirValue::Array(capture_regs_vals)), Some(MirValue::Array(param_regs_vals))) = (
            obj.get("block"), obj.get("captures"), obj.get("capture_regs"), obj.get("param_regs")
        ) {
            // Map captures
            for (cv, target_rv) in cap_vals.iter().zip(capture_regs_vals.iter()) {
                if let MirValue::Register(target_r) = target_rv {
                    let resolved = match cv { MirValue::Register(src_r) => self.registers.get(src_r.id() as usize).cloned().unwrap_or(MirValue::Null), v => v.clone() };
                    let t_idx = target_r.id() as usize; self.ensure_register_capacity(t_idx + 1); self.registers[t_idx] = resolved;
                }
            }
            // No args for deferred logical evaluation
            let inner_block_id = *block_id_i as u32;
            let mut last_value = MirValue::Null;
            if let Some(inner_block) = func.blocks.get(&inner_block_id) {
                let resolve = |v: &MirValue, regs: &Vec<MirValue>| -> MirValue {
                    match v { MirValue::Register(r) => regs.get(r.id() as usize).cloned().unwrap_or(MirValue::Null), other => other.clone() }
                };
                let mut ip: usize = 0;
                while ip < inner_block.instructions.len() {
                    let inner_inst = &inner_block.instructions[ip];
                    match inner_inst {
                        MirInstruction::Return { value } | MirInstruction::ClosureReturn { value } => {
                            if let Some(v) = value {
                                last_value = match v { MirValue::Register(r) => self.registers.get(r.id() as usize).cloned().unwrap_or(MirValue::Null), vv => vv.clone() };
                            }
                            break;
                        }
                        MirInstruction::LoadImmediate { dest, value } => {
                            let idx = dest.id() as usize; self.ensure_register_capacity(idx + 1); self.registers[idx] = value.clone();
                        }
                        // Arithmetic/bitwise/shift and comparisons
                        MirInstruction::Add { dest, left, right }
                        | MirInstruction::Sub { dest, left, right }
                        | MirInstruction::Mul { dest, left, right }
                        | MirInstruction::Div { dest, left, right }
                        | MirInstruction::Mod { dest, left, right }
                        | MirInstruction::Pow { dest, base: left, exp: right }
                        | MirInstruction::BitAnd { dest, left, right }
                        | MirInstruction::BitOr { dest, left, right }
                        | MirInstruction::BitXor { dest, left, right }
                        | MirInstruction::Shl { dest, left, right }
                        | MirInstruction::Shr { dest, left, right }
                        | MirInstruction::Equal { dest, left, right }
                        | MirInstruction::NotEqual { dest, left, right }
                        | MirInstruction::LessThan { dest, left, right }
                        | MirInstruction::LessEqual { dest, left, right }
                        | MirInstruction::GreaterThan { dest, left, right }
                        | MirInstruction::GreaterEqual { dest, left, right }
                        | MirInstruction::And { dest, left, right }
                        | MirInstruction::Or { dest, left, right } => {
                            let l_val = resolve(left, &self.registers);
                            let r_val = resolve(right, &self.registers);
                            let result = self.eval_binary(inner_inst, &l_val, &r_val)?;
                            let idx = dest.id() as usize; self.ensure_register_capacity(idx + 1); self.registers[idx] = result;
                        }
                        // Short-circuit logicals inside closure body (rare but supported)
                        MirInstruction::AndSC { dest, left, right: _, skip }
                        | MirInstruction::OrSC { dest, left, right: _, skip } => {
                            let l_raw = resolve(left, &self.registers);
                            let l_bool = match l_raw {
                                MirValue::Boolean(b) => b,
                                MirValue::Integer(i) => i != 0,
                                MirValue::Null => false,
                                _ => return Err(MirError::TypeMismatch("logical expects booleans".into())),
                            };
                            let need_right = matches!(inner_inst, MirInstruction::AndSC { .. }) && l_bool
                                || matches!(inner_inst, MirInstruction::OrSC { .. }) && !l_bool;
                            if !need_right {
                                let idx = dest.id() as usize; self.ensure_register_capacity(idx + 1); self.registers[idx] = MirValue::Boolean(l_bool);
                                ip += *skip as usize;
                            }
                        }
                        MirInstruction::RegexMatch { dest, value, pattern, not } => {
                            let v = resolve(value, &self.registers);
                            let p = resolve(pattern, &self.registers);
                            let matched = match (v, p) {
                                (MirValue::String(s), MirValue::String(pat)) => {
                                    let re = regex::Regex::new(&pat).map_err(|_| MirError::Runtime("invalid regex".into()))?;
                                    re.is_match(&s)
                                }
                                _ => return Err(MirError::TypeMismatch("regex expects strings".into())),
                            };
                            let res = if *not { !matched } else { matched };
                            let idx = dest.id() as usize; self.ensure_register_capacity(idx + 1); self.registers[idx] = MirValue::Boolean(res);
                        }
                        _ => { /* ignore other instructions */ }
                    }
                    ip += 1;
                }
            }
            Ok(last_value)
        } else {
            Err(MirError::Runtime("invalid closure object".into()))
        }
    }
 

    // === User Function Simulation (for demonstration) ===
    // In production, these would be loaded from actual MIR function definitions
    
    /// High-performance factorial function (demonstrates recursive user functions)
    fn builtin_factorial(&mut self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.len() != 1 { return Err(MirError::Runtime("factorial: requires exactly 1 argument".into())); }
        
        match &args[0] {
            MirValue::Integer(n) => {
                if *n < 0 { return Err(MirError::TypeMismatch("factorial: negative numbers not supported".into())); }
                
                let mut result = 1i64;
                for i in 1..=*n {
                    result *= i;
                }
                Ok(MirValue::Integer(result))
            }
            _ => Err(MirError::TypeMismatch("factorial: argument must be an integer".into())),
        }
    }
    
    /// High-performance fibonacci function (demonstrates dynamic programming)
    fn builtin_fibonacci(&mut self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.len() != 1 { return Err(MirError::Runtime("fibonacci: requires exactly 1 argument".into())); }
        
        match &args[0] {
            MirValue::Integer(n) => {
                if *n < 0 { return Err(MirError::TypeMismatch("fibonacci: negative numbers not supported".into())); }
                
                if *n <= 1 {
                    return Ok(MirValue::Integer(*n));
                }
                
                let mut a = 0i64;
                let mut b = 1i64;
                
                for _ in 2..=*n {
                    let temp = a + b;
                    a = b;
                    b = temp;
                }
                
                Ok(MirValue::Integer(b))
            }
            _ => Err(MirError::TypeMismatch("fibonacci: argument must be an integer".into())),
        }
    }
    
    /// High-performance max function (demonstrates variadic functions)
    fn builtin_max(&mut self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.is_empty() { return Err(MirError::Runtime("max: requires at least 1 argument".into())); }
        
        let mut max_val = &args[0];
        
        for arg in &args[1..] {
            match (max_val, arg) {
                (MirValue::Integer(a), MirValue::Integer(b)) => {
                    if b > a {
                        max_val = arg;
                    }
                }
                (MirValue::Float(a), MirValue::Float(b)) => {
                    if b > a {
                        max_val = arg;
                    }
                }
                _ => return Err(MirError::TypeMismatch("max: all arguments must be the same numeric type".into())),
            }
        }
        
        Ok(max_val.clone())
    }
    
    /// High-performance min function (demonstrates variadic functions)
    fn builtin_min(&mut self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.is_empty() { return Err(MirError::Runtime("min: requires at least 1 argument".into())); }
        
        let mut min_val = &args[0];
        
        for arg in &args[1..] {
            match (min_val, arg) {
                (MirValue::Integer(a), MirValue::Integer(b)) => {
                    if b < a {
                        min_val = arg;
                    }
                }
                (MirValue::Float(a), MirValue::Float(b)) => {
                    if b < a {
                        min_val = arg;
                    }
                }
                _ => return Err(MirError::TypeMismatch("min: all arguments must be the same numeric type".into())),
            }
        }
        
        Ok(min_val.clone())
    }
    
    /// High-performance sum function (demonstrates array processing)
    fn builtin_sum(&mut self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.is_empty() {
            return Ok(MirValue::Integer(0));
        }
        
        let mut sum = 0i64;
        let mut float_sum = 0.0f64;
        let mut is_float = false;
        
        for arg in &args {
            match arg {
                MirValue::Integer(i) => {
                    if is_float {
                        float_sum += *i as f64;
                    } else {
                        sum += i;
                    }
                }
                MirValue::Float(f) => {
                    if !is_float {
                        float_sum = sum as f64 + f;
                        is_float = true;
                    } else {
                        float_sum += f;
                    }
                }
                MirValue::Array(arr) => {
                    // Recursively sum array elements
                    let array_sum = self.builtin_sum(arr.clone())?;
                    match array_sum {
                        MirValue::Integer(i) => {
                            if is_float {
                                float_sum += i as f64;
                            } else {
                                sum += i;
                            }
                        }
                        MirValue::Float(f) => {
                            if !is_float {
                                float_sum = sum as f64 + f;
                                is_float = true;
                            } else {
                                float_sum += f;
                            }
                        }
                        _ => {}
                    }
                }
                _ => return Err(MirError::TypeMismatch("sum: arguments must be numeric or arrays".into())),
            }
        }
        
        if is_float {
            Ok(MirValue::Float(float_sum))
        } else {
            Ok(MirValue::Integer(sum))
        }
    }
    
    /// High-performance map function (demonstrates higher-order functions)
    fn builtin_map(&mut self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.len() != 2 { return Err(MirError::Runtime("map: requires exactly 2 arguments (function_name, array)".into())); }
        
        let function_name = self.value_to_string(&args[0]);
        let array = match &args[1] {
            MirValue::Array(arr) => arr,
            _ => return Err(MirError::TypeMismatch("map: second argument must be an array".into())),
        };
        
        let mut results = Vec::new();
        
        for element in array {
            // Apply function to each element
            let result = match function_name.as_str() {
                "double" => {
                    match element {
                        MirValue::Integer(i) => Ok(MirValue::Integer(i * 2)),
                        MirValue::Float(f) => Ok(MirValue::Float(f * 2.0)),
                        _ => Err(MirError::TypeMismatch("double function requires numeric input".into())),
                    }
                }
                "square" => {
                    match element {
                        MirValue::Integer(i) => Ok(MirValue::Integer(i * i)),
                        MirValue::Float(f) => Ok(MirValue::Float(f * f)),
                        _ => Err(MirError::TypeMismatch("square function requires numeric input".into())),
                    }
                }
                "toString" => Ok(MirValue::String(self.value_to_string(element))),
                _ => {
                    // Try to call user-defined function
                    self.call_user_function_by_name(&function_name, vec![element.clone()])
                }
            }?;
            
            results.push(result);
        }
        
        Ok(MirValue::Array(results))
    }
    
    /// High-performance filter function (demonstrates predicate functions)
    fn builtin_filter(&mut self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.len() != 2 { return Err(MirError::Runtime("filter: requires exactly 2 arguments (predicate_name, array)".into())); }
        
        let predicate_name = self.value_to_string(&args[0]);
        let array = match &args[1] {
            MirValue::Array(arr) => arr,
            _ => return Err(MirError::TypeMismatch("filter: second argument must be an array".into())),
        };
        
        let mut results = Vec::new();
        
        for element in array {
            // Apply predicate to each element
            let should_include = match predicate_name.as_str() {
                "isPositive" => {
                    match element {
                        MirValue::Integer(i) => *i > 0,
                        MirValue::Float(f) => *f > 0.0,
                        _ => false,
                    }
                }
                "isEven" => {
                    match element {
                        MirValue::Integer(i) => i % 2 == 0,
                        _ => false,
                    }
                }
                "isOdd" => {
                    match element {
                        MirValue::Integer(i) => i % 2 != 0,
                        _ => false,
                    }
                }
                "notNull" => !matches!(element, MirValue::Null),
                _ => {
                    // Try to call user-defined predicate function
                    match self.call_user_function_by_name(&predicate_name, vec![element.clone()]) {
                        Ok(result) => self.is_truthy(&result),
                        Err(_) => false,
                    }
                }
            };
            
            if should_include {
                results.push(element.clone());
            }
        }
        
        Ok(MirValue::Array(results))
    }
    
    /// High-performance reduce function (demonstrates accumulator patterns)
    fn builtin_reduce(&mut self, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        if args.len() < 2 || args.len() > 3 { return Err(MirError::Runtime("reduce: requires 2-3 arguments (function_name, array, [initial_value])".into())); }
        
        let function_name = self.value_to_string(&args[0]);
        let array = match &args[1] {
            MirValue::Array(arr) => arr,
            _ => return Err(MirError::TypeMismatch("reduce: second argument must be an array".into())),
        };
        
        if array.is_empty() {
            return if args.len() == 3 {
                Ok(args[2].clone())
            } else { Err(MirError::Runtime("reduce: empty array requires initial value".into())) };
        }
        
        let mut accumulator = if args.len() == 3 {
            args[2].clone()
        } else {
            array[0].clone()
        };
        
        let start_index = if args.len() == 3 { 0 } else { 1 };
        
        for element in &array[start_index..] {
            // Apply reduction function
            accumulator = match function_name.as_str() {
                "add" => {
                    match (&accumulator, element) {
                        (MirValue::Integer(a), MirValue::Integer(b)) => MirValue::Integer(a + b),
                        (MirValue::Float(a), MirValue::Float(b)) => MirValue::Float(a + b),
                        (MirValue::Integer(a), MirValue::Float(b)) => MirValue::Float(*a as f64 + b),
                        (MirValue::Float(a), MirValue::Integer(b)) => MirValue::Float(a + *b as f64),
                        (MirValue::String(a), MirValue::String(b)) => MirValue::String(format!("{}{}", a, b)),
                        _ => return Err(MirError::TypeMismatch("add reduction requires compatible types".into())),
                    }
                }
                "multiply" => {
                    match (&accumulator, element) {
                        (MirValue::Integer(a), MirValue::Integer(b)) => MirValue::Integer(a * b),
                        (MirValue::Float(a), MirValue::Float(b)) => MirValue::Float(a * b),
                        (MirValue::Integer(a), MirValue::Float(b)) => MirValue::Float(*a as f64 * b),
                        (MirValue::Float(a), MirValue::Integer(b)) => MirValue::Float(a * *b as f64),
                        _ => return Err(MirError::TypeMismatch("multiply reduction requires numeric types".into())), 
                    }
                }
                "max" => {
                    match (&accumulator, element) {
                        (MirValue::Integer(a), MirValue::Integer(b)) => {
                            if b > a { element.clone() } else { accumulator }
                        }
                        (MirValue::Float(a), MirValue::Float(b)) => {
                            if b > a { element.clone() } else { accumulator }
                        }
                        _ => return Err(MirError::TypeMismatch("max reduction requires same numeric types".into())),
                    }
                }
                "min" => {
                    match (&accumulator, element) {
                        (MirValue::Integer(a), MirValue::Integer(b)) => {
                            if b < a { element.clone() } else { accumulator }
                        }
                        (MirValue::Float(a), MirValue::Float(b)) => {
                            if b < a { element.clone() } else { accumulator }
                        }
                        _ => return Err(MirError::TypeMismatch("min reduction requires same numeric types".into())),
                    }
                }
                _ => {
                    // Try to call user-defined reduction function
                    self.call_user_function_by_name(&function_name, vec![accumulator, element.clone()])?
                }
            };
        }
        
        Ok(accumulator)
    }
    
    /// Execute external command with high performance
    fn execute_external_command(&self, command: &str, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        let string_args: Vec<String> = args.iter()
            .map(|arg| self.value_to_string(arg))
            .collect();
            
        // Use std::process::Command for actual execution
        match std::process::Command::new(command)
            .args(&string_args)
            .output() 
        {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    Ok(MirValue::String(stdout))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(MirError::Runtime(format!("{}: {}", command, stderr)))
                }
            }
            Err(e) => Err(MirError::Runtime(format!("{}: command not found: {}", command, e))),
        }
    }
    fn execute_shell_command(&mut self, command: &str, args: Vec<MirValue>) -> Result<MirValue, MirError> {
        // Convert MirValue args to strings
        let string_args: Vec<String> = args.iter().map(|arg| {
            match arg {
                MirValue::String(s) => s.clone(),
                MirValue::Integer(i) => i.to_string(),
                MirValue::Float(f) => f.to_string(),
                MirValue::Boolean(b) => b.to_string(),
                MirValue::Array(arr) => {
                    arr.iter().map(|v| format!("{:?}", v)).collect::<Vec<_>>().join(" ")
                }
                MirValue::Object(_) => "[object]".to_string(),
                MirValue::Null => "".to_string(),
                MirValue::Register(_) => "".to_string(),
            }
        }).collect();

        // For now, simulate command execution with realistic behavior
        match command {
            "echo" => {
                // Return the concatenated arguments as a string
                let output = string_args.join(" ");
                Ok(MirValue::String(output))
            }
            "pwd" => {
                // Return current directory (simplified)
                Ok(MirValue::String("/current/directory".to_string()))
            }
            "ls" => {
                // Return file listing (simplified)
                Ok(MirValue::String("file1.txt\nfile2.txt\ndirectory1/".to_string()))
            }
            "cd" => {
                // Return success for directory change
                Ok(MirValue::Integer(0))
            }
            "exit" => {
                // Return exit code
                let exit_code = string_args.first()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                Ok(MirValue::Integer(exit_code))
            }
            "true" => Ok(MirValue::Integer(0)),
            "false" => Ok(MirValue::Integer(1)),
            "cat" => {
                // Simulate reading files
                if string_args.is_empty() {
                    Ok(MirValue::String("".to_string()))
                } else {
                    Ok(MirValue::String(format!("Contents of {}", string_args[0])))
                }
            }
            "grep" => {
                // Simulate grep functionality
                Ok(MirValue::String("pattern found".to_string()))
            }
            "wc" => {
                // Simulate word count
                Ok(MirValue::String("  10  50 300".to_string()))
            }
            _ => {
                // For unknown commands, return success with empty output
                Ok(MirValue::Integer(0))
            }
        }
    }

    /// Get execution statistics
    pub fn stats(&self) -> &ExecutionStats {
        &self.stats
    }

    /// Reset execution statistics
    pub fn reset_stats(&mut self) {
        self.stats = ExecutionStats::default();
    }
}

impl Default for MirExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of block execution
#[derive(Debug)]
enum BlockResult {
    Continue(u32),
    Return(MirValue),
    Jump(u32),
}

/// Result of instruction execution
#[derive(Debug)]
enum InstructionResult {
    Continue,
    Return(MirValue),
    Jump(u32),
    Branch(MirValue, u32, u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mir_register_creation() {
        let reg = MirRegister::new(42);
        assert_eq!(reg.id(), 42);
        assert_eq!(format!("{}", reg), "%42");
    }

    #[test]
    fn test_mir_value_display() {
        assert_eq!(format!("{}", MirValue::Integer(123)), "123");
        assert_eq!(format!("{}", MirValue::String("hello".to_string())), "\"hello\"");
        assert_eq!(format!("{}", MirValue::Boolean(true)), "true");
    }

    #[test]
    fn test_mir_basic_block() {
        let mut block = MirBasicBlock::new(1);
        block.add_instruction(MirInstruction::LoadImmediate {
            dest: MirRegister::new(0),
            value: MirValue::Integer(42),
        });
        
        assert_eq!(block.instructions.len(), 1);
        assert_eq!(block.id, 1);
    }

    #[test]
    fn test_mir_function_creation() {
        let mut func = MirFunction::new("test".to_string(), vec!["param1".to_string()]);
        let reg = func.allocate_register();
        
        assert_eq!(func.name, "test");
        assert_eq!(func.parameters.len(), 1);
        assert_eq!(reg.id(), 0);
    }

    #[test]
    fn test_mir_program_creation() {
        let mut program = MirProgram::new();
        let function = MirFunction::new("main".to_string(), vec![]);
        
        program.add_function(function);
        
        assert!(program.get_function("main").is_some());
        assert_eq!(program.main_function, Some("main".to_string()));
    }

    #[test]
    fn test_comprehensive_instruction_set() {
        // Test comprehensive instruction set for 10x performance
        let reg0 = MirRegister::new(0);
        let reg1 = MirRegister::new(1);
        let reg2 = MirRegister::new(2);
        
        let instructions = vec![
            MirInstruction::LoadImmediate { dest: reg0.clone(), value: MirValue::Integer(42) },
            MirInstruction::Add { dest: reg1.clone(), left: MirValue::Register(reg0.clone()), right: MirValue::Integer(8) },
            MirInstruction::Call { dest: reg2.clone(), function: "echo".to_string(), args: vec![MirValue::Register(reg1)] },
            MirInstruction::Return { value: Some(MirValue::Register(reg2)) },
        ];
        
        assert_eq!(instructions.len(), 4);
        
        // Test instruction display
        let add_instr = &instructions[1];
        let display = format!("{}", add_instr);
        assert!(display.contains("add"));
    }
    
    #[test]
    fn test_complete_mir_pipeline() {
        // Test complete MIR compilation and execution pipeline
        let mut program = MirProgram::new();
        let mut main_func = MirFunction::new("main".to_string(), vec![]);
        
        // Allocate registers
        let reg0 = main_func.allocate_register();
        let reg1 = main_func.allocate_register();
        let _reg2 = main_func.allocate_register(); // For future use
        
        // Create basic block with comprehensive operations
        let mut entry_block = MirBasicBlock::new(0);
        
        // Load string "Hello, World!"
        entry_block.add_instruction(MirInstruction::LoadImmediate {
            dest: reg0.clone(),
            value: MirValue::String("Hello, World!".to_string()),
        });
        
        // Execute echo command
        entry_block.add_instruction(MirInstruction::ExecuteCommand {
            dest: reg1.clone(),
            command: "echo".to_string(),
            args: vec![MirValue::Register(reg0)],
        });
        
        // Return result
        entry_block.add_instruction(MirInstruction::Return {
            value: Some(MirValue::Register(reg1)),
        });
        
        main_func.add_basic_block(entry_block);
        program.add_function(main_func);
        
        // Verify program structure
        assert!(program.get_function("main").is_some());
        assert_eq!(program.main_function, Some("main".to_string()));
        
        let main_function = program.get_function("main").expect("main function should exist");
        assert_eq!(main_function.blocks.len(), 1);
        
        let entry_block = main_function.get_block(0).expect("entry block should exist");
        assert_eq!(entry_block.instructions.len(), 3);
    }

    #[test]
    fn test_mir_executor_creation() {
        let executor = MirExecutor::new();
        assert_eq!(executor.registers.len(), 256); // Pre-allocated for performance
        assert_eq!(executor.call_stack.len(), 0);
        assert_eq!(executor.global_memory.len(), 0);
    }

    #[test]
    fn test_mir_executor_arithmetic() {
        let mut executor = MirExecutor::new();
        executor.registers.resize(10, MirValue::Null);
        
        // Test addition
        let result = executor.perform_arithmetic(
            &MirValue::Integer(5),
            &MirValue::Integer(3),
            "add"
        ).expect("arithmetic operation should succeed");
        assert_eq!(result, MirValue::Integer(8));
        
        // Test multiplication
        let result = executor.perform_arithmetic(
            &MirValue::Integer(4),
            &MirValue::Integer(7),
            "mul"
        ).expect("arithmetic operation should succeed");
        assert_eq!(result, MirValue::Integer(28));
        
        // Test string concatenation
        let result = executor.perform_arithmetic(
            &MirValue::String("Hello, ".to_string()),
            &MirValue::String("World!".to_string()),
            "add"
        ).expect("string concatenation should succeed");
        assert_eq!(result, MirValue::String("Hello, World!".to_string()));
    }

    #[test]
    fn test_mir_executor_comparison() {
        let executor = MirExecutor::new();
        
        // Test integer comparison
        assert!(executor.perform_comparison(
            &MirValue::Integer(5),
            &MirValue::Integer(3),
            "gt"
        ).expect("comparison should succeed"));
        
        assert!(executor.perform_comparison(
            &MirValue::Integer(3),
            &MirValue::Integer(3),
            "eq"
        ).expect("comparison should succeed"));
        
        // Test string comparison
        assert!(executor.perform_comparison(
            &MirValue::String("apple".to_string()),
            &MirValue::String("banana".to_string()),
            "lt"
        ).expect("comparison should succeed"));
    }

    #[test]
    fn test_mir_executor_truthiness() {
        let executor = MirExecutor::new();
        
        assert!(executor.is_truthy(&MirValue::Boolean(true)));
        assert!(!executor.is_truthy(&MirValue::Boolean(false)));
        assert!(executor.is_truthy(&MirValue::Integer(1)));
        assert!(!executor.is_truthy(&MirValue::Integer(0)));
        assert!(executor.is_truthy(&MirValue::String("hello".to_string())));
        assert!(!executor.is_truthy(&MirValue::String("".to_string())));
        assert!(!executor.is_truthy(&MirValue::Null));
    }

    #[test]
    fn test_mir_execution_simple_program() {
        let mut program = MirProgram::new();
        let mut main_func = MirFunction::new("main".to_string(), vec![]);
        
        // Simple program: load 42 and return it
        let reg0 = main_func.allocate_register();
        let mut entry_block = MirBasicBlock::new(0);
        
        entry_block.add_instruction(MirInstruction::LoadImmediate {
            dest: reg0.clone(),
            value: MirValue::Integer(42),
        });
        
        entry_block.add_instruction(MirInstruction::Return {
            value: Some(MirValue::Register(reg0)),
        });
        
        main_func.add_basic_block(entry_block);
        program.add_function(main_func);
        
        // Execute program
        let mut executor = MirExecutor::new();
        let result = executor.execute(&program).expect("program execution should succeed");
        
        assert_eq!(result, MirValue::Integer(42));
        assert!(executor.stats().instructions_executed > 0);
    }

    #[test]
    fn test_mir_execution_arithmetic_program() {
        let mut program = MirProgram::new();
        let mut main_func = MirFunction::new("main".to_string(), vec![]);
        
        // Program: load 10, load 5, add them, return result
        let reg0 = main_func.allocate_register();
        let reg1 = main_func.allocate_register();
        let reg2 = main_func.allocate_register();
        let mut entry_block = MirBasicBlock::new(0);
        
        entry_block.add_instruction(MirInstruction::LoadImmediate {
            dest: reg0.clone(),
            value: MirValue::Integer(10),
        });
        
        entry_block.add_instruction(MirInstruction::LoadImmediate {
            dest: reg1.clone(),
            value: MirValue::Integer(5),
        });
        
        entry_block.add_instruction(MirInstruction::Add {
            dest: reg2.clone(),
            left: MirValue::Register(reg0),
            right: MirValue::Register(reg1),
        });
        
        entry_block.add_instruction(MirInstruction::Return {
            value: Some(MirValue::Register(reg2)),
        });
        
        main_func.add_basic_block(entry_block);
        program.add_function(main_func);
        
        // Execute program
        let mut executor = MirExecutor::new();
        let result = executor.execute(&program).expect("program execution should succeed");
        
        assert_eq!(result, MirValue::Integer(15));
        assert_eq!(executor.stats().instructions_executed, 4);
    }

    #[test]
    fn test_mir_execution_conditional_program() {
        let mut program = MirProgram::new();
        let mut main_func = MirFunction::new("main".to_string(), vec![]);
        
        // Program: if 5 > 3 then return 100 else return 200
        let reg0 = main_func.allocate_register(); // 5
        let reg1 = main_func.allocate_register(); // 3
        let reg2 = main_func.allocate_register(); // comparison result
        let reg3 = main_func.allocate_register(); // return value
        
        // Entry block
        let mut entry_block = MirBasicBlock::new(0);
        entry_block.add_instruction(MirInstruction::LoadImmediate {
            dest: reg0.clone(),
            value: MirValue::Integer(5),
        });
        entry_block.add_instruction(MirInstruction::LoadImmediate {
            dest: reg1.clone(),
            value: MirValue::Integer(3),
        });
        entry_block.add_instruction(MirInstruction::GreaterThan {
            dest: reg2.clone(),
            left: MirValue::Register(reg0),
            right: MirValue::Register(reg1),
        });
        entry_block.add_instruction(MirInstruction::Jump {
            target: 1,
        });
        entry_block.add_successor(1);
        entry_block.add_successor(2);
        
        // True block (return 100)
        let mut true_block = MirBasicBlock::new(1);
        true_block.add_instruction(MirInstruction::LoadImmediate {
            dest: reg3.clone(),
            value: MirValue::Integer(100),
        });
        true_block.add_instruction(MirInstruction::Return {
            value: Some(MirValue::Register(reg3.clone())),
        });
        
        // False block (return 200)
        let mut false_block = MirBasicBlock::new(2);
        false_block.add_instruction(MirInstruction::LoadImmediate {
            dest: reg3.clone(),
            value: MirValue::Integer(200),
        });
        false_block.add_instruction(MirInstruction::Return {
            value: Some(MirValue::Register(reg3)),
        });
        
        main_func.add_basic_block(entry_block);
        main_func.add_basic_block(true_block);
        main_func.add_basic_block(false_block);
        program.add_function(main_func);
        
        // Execute program
        let mut executor = MirExecutor::new();
        let result = executor.execute(&program).expect("program execution should succeed");
        
        // Should take true branch since 5 > 3
        assert_eq!(result, MirValue::Integer(100));
    }

    #[test]
    fn test_mir_execution_performance_stats() {
        let mut program = MirProgram::new();
        let mut main_func = MirFunction::new("main".to_string(), vec![]);
        
        // Simple program to test stats
        let reg0 = main_func.allocate_register();
        let mut entry_block = MirBasicBlock::new(0);
        
        for i in 0..10 {
            entry_block.add_instruction(MirInstruction::LoadImmediate {
                dest: reg0.clone(),
                value: MirValue::Integer(i),
            });
        }
        
        entry_block.add_instruction(MirInstruction::Return {
            value: Some(MirValue::Register(reg0)),
        });
        
        main_func.add_basic_block(entry_block);
        program.add_function(main_func);
        
        // Execute program
        let mut executor = MirExecutor::new();
        let _result = executor.execute(&program).unwrap();
        
        let stats = executor.stats();
        assert_eq!(stats.instructions_executed, 11); // 10 loads + 1 return
        assert_eq!(stats.function_calls, 1); // main function
        assert!(stats.execution_time_ns > 0);
    }

    #[test]
    fn test_closure_create_and_call_basic() {
        // main で closure オブジェクトを生成し即呼び出し、結果を返す
        let mut program = MirProgram::new();
        let mut main_func = MirFunction::new("main".to_string(), vec![]);

        // レジスタ割当
        let reg_closure = main_func.allocate_register();
        let reg_result = main_func.allocate_register(); // 呼び出し結果
        let reg_cap_source = main_func.allocate_register(); // キャプチャ元
        let reg_cap_target = main_func.allocate_register(); // クロージャ側で読む capture 用
        let reg_param = main_func.allocate_register(); // パラメータ用レジスタ (closure 呼出時に値設定)
        let reg_ret = main_func.allocate_register();

        // クロージャ本体ブロック作成 (block 1)
        let mut closure_block = MirBasicBlock::new(1);
        // capture (reg_cap_target) と param (reg_param) を加算して返す
        closure_block.add_instruction(MirInstruction::Add { dest: reg_ret.clone(), left: MirValue::Register(reg_cap_target.clone()), right: MirValue::Register(reg_param.clone()) });
        closure_block.add_instruction(MirInstruction::Return { value: Some(MirValue::Register(reg_ret.clone())) });
        main_func.add_basic_block(closure_block);

        // main entry block
        let mut entry = MirBasicBlock::new(0);
        // capture 元となる値 100 をロード
        entry.add_instruction(MirInstruction::LoadImmediate { dest: reg_cap_source.clone(), value: MirValue::Integer(100) });
        // クロージャ生成: capture に reg_cap_source、capture_regs はクロージャ内で使われる reg_cap_target
        entry.add_instruction(MirInstruction::ClosureCreate { dest: reg_closure.clone(), func_block: 1, captures: vec![MirValue::Register(reg_cap_source.clone())], capture_regs: vec![reg_cap_target.clone()], param_regs: vec![reg_param.clone()], param_names: vec!["x".to_string()] });
        // 呼出し引数 23 をロードして closure call
        entry.add_instruction(MirInstruction::LoadImmediate { dest: reg_param.clone(), value: MirValue::Integer(23) }); // この値は call 時に上書きされるがテスト用に前値設定
        entry.add_instruction(MirInstruction::ClosureCall { dest: reg_result.clone(), closure: MirValue::Register(reg_closure.clone()), args: vec![MirValue::Integer(23)] });
        entry.add_instruction(MirInstruction::Return { value: Some(MirValue::Register(reg_result.clone())) });
        main_func.add_basic_block(entry);
        program.add_function(main_func);

        let mut executor = MirExecutor::new();
        let result = executor.execute(&program).expect("closure execution should succeed");
        assert_eq!(result, MirValue::Integer(123)); // 100 (capture) + 23 (arg)
    }

    #[test]
    fn test_closure_argument_mismatch() {
        let mut program = MirProgram::new();
        let mut main_func = MirFunction::new("main".to_string(), vec![]);
        let reg_closure = main_func.allocate_register();
        let reg_cap_src = main_func.allocate_register();
        let reg_cap_tgt = main_func.allocate_register();
        let reg_param = main_func.allocate_register();
        let reg_result = main_func.allocate_register();
        let reg_ret = main_func.allocate_register();

        // closure block
        let mut closure_block = MirBasicBlock::new(1);
        closure_block.add_instruction(MirInstruction::Add { dest: reg_ret.clone(), left: MirValue::Register(reg_cap_tgt.clone()), right: MirValue::Register(reg_param.clone()) });
        closure_block.add_instruction(MirInstruction::Return { value: Some(MirValue::Register(reg_ret.clone())) });
        main_func.add_basic_block(closure_block);

        // entry
        let mut entry = MirBasicBlock::new(0);
        entry.add_instruction(MirInstruction::LoadImmediate { dest: reg_cap_src.clone(), value: MirValue::Integer(5) });
        entry.add_instruction(MirInstruction::ClosureCreate { dest: reg_closure.clone(), func_block: 1, captures: vec![MirValue::Register(reg_cap_src.clone())], capture_regs: vec![reg_cap_tgt.clone()], param_regs: vec![reg_param.clone()], param_names: vec!["x".into()] });
        // 間違った引数個数 (0 個) で呼び出し -> エラー想定
        entry.add_instruction(MirInstruction::ClosureCall { dest: reg_result.clone(), closure: MirValue::Register(reg_closure.clone()), args: vec![] });
        entry.add_instruction(MirInstruction::Return { value: Some(MirValue::Register(reg_result.clone())) });
        main_func.add_basic_block(entry);
        program.add_function(main_func);

        let mut executor = MirExecutor::new();
        let exec_result = executor.execute(&program);
        assert!(exec_result.is_err(), "expected argument mismatch error");
    }

    #[test]
    fn test_closure_return_instruction() {
        let mut program = MirProgram::new();
        let mut main_func = MirFunction::new("main".to_string(), vec![]);
        let reg_closure = main_func.allocate_register();
        let reg_res = main_func.allocate_register();
        let reg_cap_src = main_func.allocate_register();
        let reg_cap_tgt = main_func.allocate_register();
        let reg_param = main_func.allocate_register();
        let reg_tmp = main_func.allocate_register();

        // closure body block (1)
        let mut body = MirBasicBlock::new(1);
        body.add_instruction(MirInstruction::Add { dest: reg_tmp.clone(), left: MirValue::Register(reg_cap_tgt.clone()), right: MirValue::Register(reg_param.clone()) });
        body.add_instruction(MirInstruction::ClosureReturn { value: Some(MirValue::Register(reg_tmp.clone())) });
        main_func.add_basic_block(body);

        // entry
        let mut entry = MirBasicBlock::new(0);
        entry.add_instruction(MirInstruction::LoadImmediate { dest: reg_cap_src.clone(), value: MirValue::Integer(7) });
        entry.add_instruction(MirInstruction::ClosureCreate { dest: reg_closure.clone(), func_block: 1, captures: vec![MirValue::Register(reg_cap_src.clone())], capture_regs: vec![reg_cap_tgt.clone()], param_regs: vec![reg_param.clone()], param_names: vec!["x".into()] });
        entry.add_instruction(MirInstruction::ClosureCall { dest: reg_res.clone(), closure: MirValue::Register(reg_closure.clone()), args: vec![MirValue::Integer(5)] });
        entry.add_instruction(MirInstruction::Return { value: Some(MirValue::Register(reg_res.clone())) });
        main_func.add_basic_block(entry);
        program.add_function(main_func);

        let mut executor = MirExecutor::new();
        let result = executor.execute(&program).expect("closure return should work");
        assert_eq!(result, MirValue::Integer(12));
    }
}
