use super::{BasicBlock, Instruction, Program};

/// Propagate and fold constants within each basic block.
pub fn fold_constants(prog: &mut Program) {
    for (_, function) in &mut prog.functions {
        for (_, block) in &mut function.blocks {
            constant_fold_block(block);
        }
    }
}

fn constant_fold_block(block: &mut BasicBlock) {
    use Instruction::*;
    let mut const_table = std::collections::HashMap::new();

            for instr in &mut block.instructions {
        match instr {
            ConstInt { dst, value } => {
                const_table.insert(*dst, *value);
            }
            Add { dst, lhs, rhs }
            | Sub { dst, lhs, rhs }
            | Mul { dst, lhs, rhs }
            | Div { dst, lhs, rhs } => {
                let dst_val = *dst;
                let lhs_val = *lhs;
                let rhs_val = *rhs;
                
                if let (Some(&lv), Some(&rv)) = (const_table.get(&lhs_val), const_table.get(&rhs_val)) {
                    let result = match instr {
                        Add { .. } => lv + rv,
                        Sub { .. } => lv - rv,
                        Mul { .. } => lv * rv,
                        Div { .. } => {
                            if rv == 0 {
                                continue;
                            }
                            lv / rv
                        }
                        _ => unreachable!(),
                    };
                    // Replace instruction with constant and update table
                    *instr = ConstInt { dst: dst_val, value: result };
                    const_table.insert(dst_val, result);
                }
            }
            // Handle all other instruction types
            _ => {
                // No constant folding for other instructions
            }
        }
    }
} 