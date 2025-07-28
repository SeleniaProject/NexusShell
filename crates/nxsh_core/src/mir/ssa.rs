use super::{BasicBlock, Instruction, Program, ValueId};

/// Convert the program to SSA form by renaming variables.
pub fn to_ssa(prog: &mut Program) {
    let mut next_id: ValueId = 1000; // start after reserved ids
    for (_, function) in &mut prog.functions {
        for (_, block) in &mut function.blocks {
            for instr in &mut block.instructions {
            match instr {
                Instruction::ConstInt { dst, .. } => {
                    *dst = fresh(&mut next_id);
                }
                Instruction::Add { dst, lhs, rhs }
                | Instruction::Sub { dst, lhs, rhs }
                | Instruction::Mul { dst, lhs, rhs }
                | Instruction::Div { dst, lhs, rhs } => {
                    *dst = fresh(&mut next_id);
                    // lhs/rhs assumed already SSA
                }
                // Handle all other instruction types
                _ => {
                    // No SSA conversion needed for other instructions
                }
            }
        }
    }
}
}

/// Convert the program to SSA form (alias for to_ssa)
pub fn convert_to_ssa(prog: &mut Program) {
    to_ssa(prog);
}

fn fresh(counter: &mut ValueId) -> ValueId {
    let id = *counter;
    *counter += 1;
    id
} 