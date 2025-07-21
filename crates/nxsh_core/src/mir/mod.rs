//! Mid-level IR definitions and transformations for NexusShell.

pub type ValueId = u32;

#[derive(Debug, Clone)]
pub struct Program {
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,
    pub instrs: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    ConstInt { id: ValueId, value: i64 },
    Add { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Sub { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Mul { dst: ValueId, lhs: ValueId, rhs: ValueId },
    Div { dst: ValueId, lhs: ValueId, rhs: ValueId },
    // ... more operations later ...
}

impl Program {
    /// Apply constant folding optimization in-place.
    pub fn constant_fold(&mut self) {
        crate::mir::const_fold::fold_constants(self);
    }
}

pub mod ssa;
pub mod const_fold;

#[cfg(feature = "jit")]
pub mod jit; 