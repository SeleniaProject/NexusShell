use super::{BasicBlock, Instruction, Program};

/// Propagate and fold constants within each basic block.
pub fn fold_constants(prog: &mut Program) {
    for block in &mut prog.blocks {
        constant_fold_block(block);
    }
}

fn constant_fold_block(block: &mut BasicBlock) {
    use Instruction::*;
    let mut const_table = std::collections::HashMap::new();

    for instr in &mut block.instrs {
        match instr {
            ConstInt { id, value } => {
                const_table.insert(*id, *value);
            }
            Add { dst, lhs, rhs }
            | Sub { dst, lhs, rhs }
            | Mul { dst, lhs, rhs }
            | Div { dst, lhs, rhs } => {
                if let (Some(lv), Some(rv)) = (const_table.get(lhs), const_table.get(rhs)) {
                    let result = match instr {
                        Add { .. } => lv + rv,
                        Sub { .. } => lv - rv,
                        Mul { .. } => lv * rv,
                        Div { .. } => {
                            if *rv == 0 {
                                continue;
                            }
                            lv / rv
                        }
                        _ => unreachable!(),
                    };
                    // Replace instruction with constant and update table
                    *instr = ConstInt { id: *dst, value: result };
                    const_table.insert(*dst, result);
                }
            }
        }
    }
} 