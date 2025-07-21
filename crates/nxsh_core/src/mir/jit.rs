#![cfg(feature = "jit")]

use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_codegen::ir::{self, types};
use cranelift_codegen::Context;
use super::{Instruction, Program};

/// Compile MIR Program to machine code (native) and return code size.
pub fn compile(program: &Program) -> anyhow::Result<usize> {
    let mut flag_builder = settings::builder();
    flag_builder.set("is_pic", "true").ok();
    let isa_builder = cranelift_native::builder().map_err(|e| anyhow::anyhow!(e))?;
    let isa = isa_builder.finish(settings::Flags::new(flag_builder));

    let mut ctx = Context::new();
    ctx.func.signature.returns.push(ir::AbiParam::new(types::I64));
    ctx.func.signature.call_conv = CallConv::Fast;

    let mut func_ctx = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        builder.seal_block(entry);

        // Very naive: accumulate constant ints.
        let mut acc = 0i64;
        for block in &program.blocks {
            for instr in &block.instrs {
                if let Instruction::ConstInt { value, .. } = instr {
                    acc += value;
                }
            }
        }
        let acc_val = builder.ins().iconst(types::I64, acc);
        builder.ins().return_(&[acc_val]);
        builder.finalize();
    }

    ctx.compile(&*isa)?;
    Ok(ctx.code_size())
} 