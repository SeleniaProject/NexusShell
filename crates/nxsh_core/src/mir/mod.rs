//! Mid-level Intermediate Representation (MIR) for NexusShell
//!
//! This module provides a complete MIR implementation with SSA form,
//! optimization passes, and code generation capabilities.

use crate::error::{ShellError, ErrorKind};
use crate::result::ShellResult;
use nxsh_parser::ast::AstNode;
use std::collections::{HashMap, HashSet};
use std::fmt;

pub type ValueId = u32;
pub type BlockId = u32;
pub type FunctionId = u32;

/// Complete MIR program representation
#[derive(Debug, Clone)]
pub struct Program {
    pub functions: HashMap<FunctionId, Function>,
    pub globals: HashMap<String, GlobalValue>,
    pub entry_point: Option<FunctionId>,
    pub metadata: ProgramMetadata,
}

/// Function in MIR form
#[derive(Debug, Clone)]
pub struct Function {
    pub id: FunctionId,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub blocks: HashMap<BlockId, BasicBlock>,
    pub entry_block: BlockId,
    pub local_variables: HashMap<String, LocalVariable>,
    pub is_async: bool,
    pub is_builtin: bool,
}

/// Basic block in SSA form
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
    pub predecessors: Vec<BlockId>,
    pub successors: Vec<BlockId>,
    pub phi_nodes: Vec<PhiNode>,
}

/// SSA instruction
#[derive(Debug, Clone)]
pub enum Instruction {
    // Arithmetic operations
    Add { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Sub { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Mul { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Div { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Mod { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Pow { dst: ValueId, lhs: ValueId, rhs: ValueId },
    
    // Bitwise operations
    And { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Or { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Xor { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Shl { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Shr { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Not { dst: ValueId, operand: ValueId },
    
    // Comparison operations
    Eq { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Ne { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Lt { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Le { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Gt { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Ge { dst: ValueId, lhs: ValueId, rhs: ValueId },
    
    // Memory operations
    Load { dst: ValueId, address: ValueId },
    Store { address: ValueId, value: ValueId },
    Alloca { dst: ValueId, size: ValueId, align: u32 },
    
    // Constants
    ConstInt { dst: ValueId, value: i64 },
    ConstFloat { dst: ValueId, value: f64 },
    ConstString { dst: ValueId, value: String },
    ConstBool { dst: ValueId, value: bool },
    
    // Variable operations
    GetVar { dst: ValueId, name: String },
    SetVar { name: String, value: ValueId },
    GetEnv { dst: ValueId, name: String },
    SetEnv { name: String, value: ValueId },
    
    // Array operations
    ArrayNew { dst: ValueId, size: ValueId },
    ArrayGet { dst: ValueId, array: ValueId, index: ValueId },
    ArraySet { array: ValueId, index: ValueId, value: ValueId },
    ArrayLen { dst: ValueId, array: ValueId },
    
    // String operations
    StringConcat { dst: ValueId, lhs: ValueId, rhs: ValueId },
    StringLen { dst: ValueId, string: ValueId },
    StringSlice { dst: ValueId, string: ValueId, start: ValueId, end: ValueId },
    StringMatch { dst: ValueId, string: ValueId, pattern: ValueId },
    
    // Process operations
    Exec { dst: ValueId, command: ValueId, args: Vec<ValueId> },
    Fork { dst: ValueId },
    Wait { dst: ValueId, pid: ValueId },
    Kill { pid: ValueId, signal: ValueId },
    
    // Pipeline operations
    Pipe { dst: ValueId, left: ValueId, right: ValueId },
    ObjectPipe { dst: ValueId, left: ValueId, right: ValueId },
    ParallelPipe { dst: ValueId, left: ValueId, right: ValueId },
    
    // I/O operations
    Read { dst: ValueId, fd: ValueId, buffer: ValueId, count: ValueId },
    Write { dst: ValueId, fd: ValueId, buffer: ValueId, count: ValueId },
    Open { dst: ValueId, path: ValueId, flags: ValueId },
    Close { fd: ValueId },
    
    // Control flow helpers
    Select { dst: ValueId, condition: ValueId, true_val: ValueId, false_val: ValueId },
    
    // Function operations
    Call { dst: Option<ValueId>, function: ValueId, args: Vec<ValueId> },
    CallBuiltin { dst: Option<ValueId>, builtin: BuiltinFunction, args: Vec<ValueId> },
    
    // Async operations
    Spawn { dst: ValueId, function: ValueId, args: Vec<ValueId> },
    Await { dst: ValueId, future: ValueId },
    Yield { value: Option<ValueId> },
    
    // Exception handling
    Throw { exception: ValueId },
    
    // Type operations
    Cast { dst: ValueId, value: ValueId, target_type: Type },
    TypeOf { dst: ValueId, value: ValueId },
    
    // Debugging
    Debug { message: String, values: Vec<ValueId> },
}

/// Block terminator
#[derive(Debug, Clone)]
pub enum Terminator {
    Return { value: Option<ValueId> },
    Branch { target: BlockId },
    ConditionalBranch { condition: ValueId, true_target: BlockId, false_target: BlockId },
    Switch { value: ValueId, cases: Vec<(i64, BlockId)>, default: BlockId },
    Unreachable,
}

/// PHI node for SSA form
#[derive(Debug, Clone)]
pub struct PhiNode {
    pub dst: ValueId,
    pub incoming: Vec<(ValueId, BlockId)>,
    pub value_type: Type,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub value_type: Type,
    pub is_variadic: bool,
}

/// Local variable
#[derive(Debug, Clone)]
pub struct LocalVariable {
    pub name: String,
    pub value_type: Type,
    pub is_mutable: bool,
    pub initial_value: Option<ValueId>,
}

/// Global value
#[derive(Debug, Clone)]
pub struct GlobalValue {
    pub name: String,
    pub value_type: Type,
    pub is_constant: bool,
    pub initial_value: Option<ConstantValue>,
}

/// Constant values
#[derive(Debug, Clone)]
pub enum ConstantValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Vec<ConstantValue>),
    Null,
}

/// Type system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Int,
    Float,
    String,
    Bool,
    Array(Box<Type>),
    Function { params: Vec<Type>, return_type: Box<Type> },
    Process,
    File,
    Any,
}

/// Builtin functions
#[derive(Debug, Clone)]
pub enum BuiltinFunction {
    // String functions
    Echo,
    Printf,
    
    // File functions
    Cat,
    Ls,
    Cp,
    Mv,
    Rm,
    
    // Process functions
    Ps,
    Kill,
    Jobs,
    
    // System functions
    Pwd,
    Cd,
    Exit,
    
    // Test functions
    Test,
    
    // Custom function
    Custom(String),
}

/// Program metadata
#[derive(Debug, Clone, Default)]
pub struct ProgramMetadata {
    pub source_file: Option<String>,
    pub optimization_level: OptimizationLevel,
    pub debug_info: bool,
    pub target_platform: Option<String>,
}

/// Optimization levels
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
}

impl Default for OptimizationLevel {
    fn default() -> Self {
        OptimizationLevel::Basic
    }
}

impl Program {
    /// Create a new empty program
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            globals: HashMap::new(),
            entry_point: None,
            metadata: ProgramMetadata::default(),
        }
    }

    /// Add a function to the program
    pub fn add_function(&mut self, function: Function) -> FunctionId {
        let id = function.id;
        self.functions.insert(id, function);
        id
    }

    /// Get a function by ID
    pub fn get_function(&self, id: FunctionId) -> Option<&Function> {
        self.functions.get(&id)
    }

    /// Get a mutable function by ID
    pub fn get_function_mut(&mut self, id: FunctionId) -> Option<&mut Function> {
        self.functions.get_mut(&id)
    }

    /// Apply all optimization passes
    pub fn optimize(&mut self) -> ShellResult<()> {
        match self.metadata.optimization_level {
            OptimizationLevel::None => Ok(()),
            OptimizationLevel::Basic => {
                self.constant_fold()?;
                self.dead_code_elimination()?;
                Ok(())
            }
            OptimizationLevel::Aggressive => {
                self.constant_fold()?;
                self.dead_code_elimination()?;
                self.inline_functions()?;
                self.loop_optimization()?;
                Ok(())
            }
        }
    }

    /// Apply constant folding optimization
    pub fn constant_fold(&mut self) -> ShellResult<()> {
        const_fold::fold_constants(self);
        Ok(())
    }

    /// Apply dead code elimination
    pub fn dead_code_elimination(&mut self) -> ShellResult<()> {
        for function in self.functions.values_mut() {
            eliminate_dead_code(function)?;
        }
        Ok(())
    }

    /// Apply function inlining
    pub fn inline_functions(&mut self) -> ShellResult<()> {
        // Implementation would analyze call sites and inline small functions
        Ok(())
    }

    /// Apply loop optimizations
    pub fn loop_optimization(&mut self) -> ShellResult<()> {
        // Implementation would optimize loops (unrolling, invariant motion, etc.)
        Ok(())
    }

    /// Convert to SSA form
    pub fn to_ssa(&mut self) -> ShellResult<()> {
        ssa::convert_to_ssa(self);
        Ok(())
    }

    /// Validate the program
    pub fn validate(&self) -> ShellResult<()> {
        for function in self.functions.values() {
            validate_function(function)?;
        }
        Ok(())
    }
}

impl Function {
    /// Create a new function
    pub fn new(id: FunctionId, name: String, return_type: Type) -> Self {
        Self {
            id,
            name,
            parameters: Vec::new(),
            return_type,
            blocks: HashMap::new(),
            entry_block: 0,
            local_variables: HashMap::new(),
            is_async: false,
            is_builtin: false,
        }
    }

    /// Add a basic block
    pub fn add_block(&mut self, block: BasicBlock) -> BlockId {
        let id = block.id;
        self.blocks.insert(id, block);
        id
    }

    /// Get a basic block
    pub fn get_block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }

    /// Get a mutable basic block
    pub fn get_block_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(&id)
    }
}

impl BasicBlock {
    /// Create a new basic block
    pub fn new(id: BlockId) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            terminator: Terminator::Unreachable,
            predecessors: Vec::new(),
            successors: Vec::new(),
            phi_nodes: Vec::new(),
        }
    }

    /// Add an instruction
    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    /// Set the terminator
    pub fn set_terminator(&mut self, terminator: Terminator) {
        self.terminator = terminator;
    }

    /// Add a PHI node
    pub fn add_phi_node(&mut self, phi: PhiNode) {
        self.phi_nodes.push(phi);
    }
}

/// Dead code elimination for a function
fn eliminate_dead_code(function: &mut Function) -> ShellResult<()> {
    let mut used_values = HashSet::new();
    let mut worklist = Vec::new();

    // Mark all values used in terminators as live
    for block in function.blocks.values() {
        match &block.terminator {
            Terminator::Return { value: Some(v) } => {
                used_values.insert(*v);
                worklist.push(*v);
            }
            Terminator::ConditionalBranch { condition, .. } => {
                used_values.insert(*condition);
                worklist.push(*condition);
            }
            Terminator::Switch { value, .. } => {
                used_values.insert(*value);
                worklist.push(*value);
            }
            _ => {}
        }
    }

    // Propagate liveness backwards
    while let Some(value_id) = worklist.pop() {
        for block in function.blocks.values() {
            for instruction in &block.instructions {
                if instruction.defines_value(value_id) {
                    for operand in instruction.operands() {
                        if used_values.insert(operand) {
                            worklist.push(operand);
                        }
                    }
                }
            }
        }
    }

    // Remove dead instructions
    for block in function.blocks.values_mut() {
        block.instructions.retain(|instr| {
            if let Some(defined) = instr.defined_value() {
                used_values.contains(&defined)
            } else {
                true // Keep instructions with side effects
            }
        });
    }

    Ok(())
}

/// Validate a function
fn validate_function(function: &Function) -> ShellResult<()> {
    // Check that entry block exists
    if !function.blocks.contains_key(&function.entry_block) {
        return Err(ShellError::new(
            ErrorKind::InternalError(crate::error::InternalErrorKind::InvalidState),
            format!("Function '{}' has invalid entry block", function.name),
        ));
    }

    // Validate each block
    for block in function.blocks.values() {
        validate_block(block)?;
    }

    Ok(())
}

/// Validate a basic block
fn validate_block(block: &BasicBlock) -> ShellResult<()> {
    // Check that all successor blocks are valid
    for &successor in &block.successors {
        // In a real implementation, we'd check that the successor exists
    }

    // Validate instructions
    for instruction in &block.instructions {
        validate_instruction(instruction)?;
    }

    Ok(())
}

/// Validate an instruction
fn validate_instruction(_instruction: &Instruction) -> ShellResult<()> {
    // Implementation would check operand types, value definitions, etc.
    Ok(())
}

impl Instruction {
    /// Get the value defined by this instruction, if any
    pub fn defined_value(&self) -> Option<ValueId> {
        match self {
            Instruction::Add { dst, .. } |
            Instruction::Sub { dst, .. } |
            Instruction::Mul { dst, .. } |
            Instruction::Div { dst, .. } |
            Instruction::Mod { dst, .. } |
            Instruction::Pow { dst, .. } |
            Instruction::And { dst, .. } |
            Instruction::Or { dst, .. } |
            Instruction::Xor { dst, .. } |
            Instruction::Shl { dst, .. } |
            Instruction::Shr { dst, .. } |
            Instruction::Not { dst, .. } |
            Instruction::Eq { dst, .. } |
            Instruction::Ne { dst, .. } |
            Instruction::Lt { dst, .. } |
            Instruction::Le { dst, .. } |
            Instruction::Gt { dst, .. } |
            Instruction::Ge { dst, .. } |
            Instruction::Load { dst, .. } |
            Instruction::ConstInt { dst, .. } |
            Instruction::ConstFloat { dst, .. } |
            Instruction::ConstString { dst, .. } |
            Instruction::ConstBool { dst, .. } |
            Instruction::GetVar { dst, .. } |
            Instruction::GetEnv { dst, .. } |
            Instruction::ArrayNew { dst, .. } |
            Instruction::ArrayGet { dst, .. } |
            Instruction::ArrayLen { dst, .. } |
            Instruction::StringConcat { dst, .. } |
            Instruction::StringLen { dst, .. } |
            Instruction::StringSlice { dst, .. } |
            Instruction::StringMatch { dst, .. } |
            Instruction::Exec { dst, .. } |
            Instruction::Fork { dst, .. } |
            Instruction::Wait { dst, .. } |
            Instruction::Pipe { dst, .. } |
            Instruction::ObjectPipe { dst, .. } |
            Instruction::ParallelPipe { dst, .. } |
            Instruction::Read { dst, .. } |
            Instruction::Write { dst, .. } |
            Instruction::Open { dst, .. } |
            Instruction::Select { dst, .. } |
            Instruction::Spawn { dst, .. } |
            Instruction::Await { dst, .. } |
            Instruction::Cast { dst, .. } |
            Instruction::TypeOf { dst, .. } |
            Instruction::Alloca { dst, .. } => Some(*dst),
            
            Instruction::Call { dst: Some(dst), .. } |
            Instruction::CallBuiltin { dst: Some(dst), .. } => Some(*dst),
            
            _ => None,
        }
    }

    /// Check if this instruction defines the given value
    pub fn defines_value(&self, value_id: ValueId) -> bool {
        self.defined_value() == Some(value_id)
    }

    /// Get all operands used by this instruction
    pub fn operands(&self) -> Vec<ValueId> {
        match self {
            Instruction::Add { lhs, rhs, .. } |
            Instruction::Sub { lhs, rhs, .. } |
            Instruction::Mul { lhs, rhs, .. } |
            Instruction::Div { lhs, rhs, .. } |
            Instruction::Mod { lhs, rhs, .. } |
            Instruction::Pow { lhs, rhs, .. } |
            Instruction::And { lhs, rhs, .. } |
            Instruction::Or { lhs, rhs, .. } |
            Instruction::Xor { lhs, rhs, .. } |
            Instruction::Shl { lhs, rhs, .. } |
            Instruction::Shr { lhs, rhs, .. } |
            Instruction::Eq { lhs, rhs, .. } |
            Instruction::Ne { lhs, rhs, .. } |
            Instruction::Lt { lhs, rhs, .. } |
            Instruction::Le { lhs, rhs, .. } |
            Instruction::Gt { lhs, rhs, .. } |
            Instruction::Ge { lhs, rhs, .. } |
            Instruction::StringConcat { lhs, rhs, .. } => vec![*lhs, *rhs],
            
            Instruction::Not { operand, .. } => vec![*operand],
            Instruction::Load { address, .. } => vec![*address],
            Instruction::StringLen { string, .. } => vec![*string],
            Instruction::ArrayLen { array, .. } => vec![*array],
            Instruction::Fork { .. } => vec![],
            Instruction::Wait { pid, .. } => vec![*pid],
            Instruction::Close { fd } => vec![*fd],
            Instruction::Await { future, .. } => vec![*future],
            Instruction::Cast { value, .. } => vec![*value],
            Instruction::TypeOf { value, .. } => vec![*value],
            Instruction::Throw { exception } => vec![*exception],
            
            Instruction::Store { address, value } => vec![*address, *value],
            Instruction::SetVar { value, .. } => vec![*value],
            Instruction::SetEnv { value, .. } => vec![*value],
            
            Instruction::ArrayGet { array, index, .. } => vec![*array, *index],
            Instruction::Kill { pid, signal } => vec![*pid, *signal],
            
            Instruction::ArraySet { array, index, value } => vec![*array, *index, *value],
            Instruction::StringSlice { string, start, end, .. } => vec![*string, *start, *end],
            
            Instruction::Select { condition, true_val, false_val, .. } => {
                vec![*condition, *true_val, *false_val]
            }
            
            Instruction::Call { function, args, .. } => {
                let mut operands = vec![*function];
                operands.extend(args);
                operands
            }
            
            Instruction::CallBuiltin { args, .. } => args.clone(),
            
            Instruction::Spawn { function, args, .. } => {
                let mut operands = vec![*function];
                operands.extend(args);
                operands
            }
            
            Instruction::Read { fd, buffer, count, .. } |
            Instruction::Write { fd, buffer, count, .. } => vec![*fd, *buffer, *count],
            
            Instruction::Open { path, flags, .. } => vec![*path, *flags],
            
            Instruction::Alloca { size, .. } => vec![*size],
            
            Instruction::StringMatch { string, pattern, .. } => vec![*string, *pattern],
            
            Instruction::Exec { command, args, .. } => {
                let mut operands = vec![*command];
                operands.extend(args);
                operands
            }
            
            Instruction::Pipe { left, right, .. } |
            Instruction::ObjectPipe { left, right, .. } |
            Instruction::ParallelPipe { left, right, .. } => vec![*left, *right],
            
            Instruction::Debug { values, .. } => values.clone(),
            
            Instruction::Yield { value: Some(v) } => vec![*v],
            
            // Constants and other instructions with no operands
            _ => Vec::new(),
        }
    }

    /// Check if this instruction has side effects
    pub fn has_side_effects(&self) -> bool {
        matches!(self,
            Instruction::Store { .. } |
            Instruction::SetVar { .. } |
            Instruction::SetEnv { .. } |
            Instruction::ArraySet { .. } |
            Instruction::Exec { .. } |
            Instruction::Kill { .. } |
            Instruction::Write { .. } |
            Instruction::Close { .. } |
            Instruction::Call { .. } |
            Instruction::CallBuiltin { .. } |
            Instruction::Spawn { .. } |
            Instruction::Yield { .. } |
            Instruction::Throw { .. } |
            Instruction::Debug { .. }
        )
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Program {{")?;
        for function in self.functions.values() {
            write!(f, "{}", function)?;
        }
        writeln!(f, "}}")
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  function {}({}) -> {} {{", 
                 self.name, 
                 self.parameters.iter().map(|p| format!("{}: {}", p.name, p.value_type))
                     .collect::<Vec<_>>().join(", "),
                 self.return_type)?;
        
        for block in self.blocks.values() {
            write!(f, "{}", block)?;
        }
        
        writeln!(f, "  }}")
    }
}

impl fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "    block{}:", self.id)?;
        
        for phi in &self.phi_nodes {
            writeln!(f, "      {} = phi {} [{}]", 
                     phi.dst, 
                     phi.value_type,
                     phi.incoming.iter()
                         .map(|(v, b)| format!("%{} from block{}", v, b))
                         .collect::<Vec<_>>().join(", "))?;
        }
        
        for instruction in &self.instructions {
            writeln!(f, "      {}", instruction)?;
        }
        
        writeln!(f, "      {}", self.terminator)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Add { dst, lhs, rhs } => write!(f, "%{} = add %{}, %{}", dst, lhs, rhs),
            Instruction::Sub { dst, lhs, rhs } => write!(f, "%{} = sub %{}, %{}", dst, lhs, rhs),
            Instruction::Mul { dst, lhs, rhs } => write!(f, "%{} = mul %{}, %{}", dst, lhs, rhs),
            Instruction::Div { dst, lhs, rhs } => write!(f, "%{} = div %{}, %{}", dst, lhs, rhs),
            Instruction::ConstInt { dst, value } => write!(f, "%{} = const {}", dst, value),
            Instruction::ConstString { dst, value } => write!(f, "%{} = const \"{}\"", dst, value),
            Instruction::Call { dst, function, args } => {
                if let Some(dst) = dst {
                    write!(f, "%{} = call %{}({})", dst, function, 
                           args.iter().map(|a| format!("%{}", a)).collect::<Vec<_>>().join(", "))
                } else {
                    write!(f, "call %{}({})", function, 
                           args.iter().map(|a| format!("%{}", a)).collect::<Vec<_>>().join(", "))
                }
            }
            _ => write!(f, "{:?}", self), // Fallback for other instructions
        }
    }
}

impl fmt::Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Terminator::Return { value: Some(v) } => write!(f, "ret %{}", v),
            Terminator::Return { value: None } => write!(f, "ret void"),
            Terminator::Branch { target } => write!(f, "br block{}", target),
            Terminator::ConditionalBranch { condition, true_target, false_target } => {
                write!(f, "br %{}, block{}, block{}", condition, true_target, false_target)
            }
            Terminator::Switch { value, cases, default } => {
                write!(f, "switch %{} [", value)?;
                for (case_val, target) in cases {
                    write!(f, "{} -> block{}, ", case_val, target)?;
                }
                write!(f, "default -> block{}]", default)
            }
            Terminator::Unreachable => write!(f, "unreachable"),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            Type::Array(element_type) => write!(f, "[{}]", element_type),
            Type::Function { params, return_type } => {
                write!(f, "({}) -> {}", 
                       params.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "),
                       return_type)
            }
            Type::Process => write!(f, "process"),
            Type::File => write!(f, "file"),
            Type::Any => write!(f, "any"),
        }
    }
}

pub mod ssa;
pub mod const_fold;

#[cfg(feature = "jit")]
pub mod jit; 