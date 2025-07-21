use super::{BasicBlock, Instruction, Program, ValueId};

/// Convert the program to SSA form by renaming variables.
pub fn to_ssa(prog: &mut Program) {
    let mut next_id: ValueId = 1000; // start after reserved ids
    for block in &mut prog.blocks {
        for instr in &mut block.instrs {
            match instr {
                Instruction::ConstInt { id, .. } => {
                    *id = fresh(&mut next_id);
                }
                Instruction::Add { dst, lhs, rhs }
                | Instruction::Sub { dst, lhs, rhs }
                | Instruction::Mul { dst, lhs, rhs }
                | Instruction::Div { dst, lhs, rhs } => {
                    *dst = fresh(&mut next_id);
                    // lhs/rhs assumed already SSA
                }
            }
        }
    }
}

fn fresh(counter: &mut ValueId) -> ValueId {
    let id = *counter;
    *counter += 1;
    id
} 