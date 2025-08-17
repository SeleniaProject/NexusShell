//! AST -> MIR Lowering (改良: ClosureCreate 対応)
use nxsh_parser::ast::AstNode;
use nxsh_parser::ast::BinaryOperator;
use super::{MirProgram, MirFunction, MirInstruction, MirRegister, MirValue};

pub struct Lowerer {
    reg_counter: u32,
    // 直前に lower 済みの変数 -> レジスタ 対応 (簡易キャプチャ用)
    var_env: std::collections::HashMap<String, MirRegister>,
    // クロージャ本体を lowering 中かどうか（短絡命令の生成方針に影響）
    in_closure: bool,
}

impl Lowerer {
    pub fn new() -> Self { Self { reg_counter: 0, var_env: std::collections::HashMap::new(), in_closure: false } }
}

impl Default for Lowerer {
    fn default() -> Self { Self::new() }
}
impl Lowerer {
    fn fresh_reg(&mut self) -> MirRegister { let id = self.reg_counter; self.reg_counter += 1; MirRegister::new(id) }

    pub fn lower_program(mut self, ast: &AstNode) -> MirProgram {
        let mut prog = MirProgram::new();
        let mut func = MirFunction::new("main".to_string(), Vec::new());
        let entry = func.entry_block;
        self.lower_node_prog(ast, &mut prog, &mut func, entry);
        if let Some(b) = func.get_block_mut(entry) {
            if !matches!(b.instructions.last(), Some(MirInstruction::Return { .. })) {
                b.instructions.push(MirInstruction::Return { value: Some(MirValue::Null) });
            }
        }
        prog.add_function(func);
        prog
    }

    fn lower_node_prog(&mut self, node: &AstNode, prog: &mut MirProgram, func: &mut MirFunction, current_block: u32) -> Option<MirRegister> {
        match node {
            AstNode::Program(stmts) | AstNode::StatementList(stmts) => {
                let mut last_reg: Option<MirRegister> = None;
                for s in stmts {
                    if let Some(r) = self.lower_node_prog(s, prog, func, current_block) {
                        last_reg = Some(r);
                    }
                    // 途中で明示的 Return/ClosureReturn が出たら以降は無視 (既にブロック末端確定)
                    if let Some(block) = func.get_block(current_block) {
                        if matches!(block.instructions.last(), Some(MirInstruction::Return { .. }) | Some(MirInstruction::ClosureReturn { .. })) {
                            return None;
                        }
                    }
                }
                if let Some(block) = func.get_block_mut(current_block) {
                    if !matches!(block.instructions.last(), Some(MirInstruction::Return { .. }) | Some(MirInstruction::ClosureReturn { .. })) {
                        if let Some(r) = last_reg { block.instructions.push(MirInstruction::Return { value: Some(MirValue::Register(r)) }); }
                        else { block.instructions.push(MirInstruction::Return { value: Some(MirValue::Null) }); }
                    }
                }
                None
            }
            AstNode::Function { name, params, body, .. } | AstNode::FunctionDeclaration { name, params, body, .. } => {
                // 別関数として MirProgram に登録し、本体を独立に lowering する
                let param_names: Vec<String> = params.iter().map(|p| p.name.to_string()).collect();
                let mut f = MirFunction::new((*name).to_string(), param_names);
                let entry_block = f.entry_block;
                // ネスト関数は独立した Lowerer で環境をリセットして lower する
                let mut nested = Lowerer::new();
                nested.lower_node_prog(body, prog, &mut f, entry_block);
                if let Some(bblk) = f.get_block_mut(entry_block) {
                    if !matches!(bblk.instructions.last(), Some(MirInstruction::Return { .. }) | Some(MirInstruction::ClosureReturn { .. })) {
                        bblk.instructions.push(MirInstruction::Return { value: Some(MirValue::Null) });
                    }
                }
                prog.add_function(f);
                None
            }
            AstNode::FunctionCall { name, args, .. } => {
                // Callee を lower。Word で var_env に登録済みなら closure 変数呼び出し、それ以外の Word は通常関数。
                let func_reg = self.lower_node_prog(name, prog, func, current_block);
                let mut arg_vals = Vec::new();
                for a in args { if let Some(r) = self.lower_node_prog(a, prog, func, current_block) { arg_vals.push(MirValue::Register(r)); } }
                let dest = self.fresh_reg();
                if let Some(block) = func.get_block_mut(current_block) {
                    match &**name {
                        AstNode::Word(w) => {
                            if let Some(reg) = self.var_env.get(*w) {
                                // 変数に格納されたクロージャ (または動的 callable) を ClosureCall
                                block.instructions.push(MirInstruction::ClosureCall { dest: dest.clone(), closure: MirValue::Register(reg.clone()), args: arg_vals });
                            } else {
                                // 通常の関数名呼び出し
                                block.instructions.push(MirInstruction::Call { dest: dest.clone(), function: w.to_string(), args: arg_vals });
                            }
                        }
                        _ => {
                            // 動的: lower された register を closure として呼ぶ
                            if let Some(r) = func_reg { block.instructions.push(MirInstruction::ClosureCall { dest: dest.clone(), closure: MirValue::Register(r), args: arg_vals }); }
                            else { block.instructions.push(MirInstruction::LoadImmediate { dest: dest.clone(), value: MirValue::Null }); }
                        }
                    }
                }
                Some(dest)
            }
            AstNode::NumberLiteral { value, .. } => {
                if let Ok(n) = value.parse::<i64>() {
                    let r = self.fresh_reg();
                    if let Some(block) = func.get_block_mut(current_block) {
                        block.instructions.push(MirInstruction::LoadImmediate { dest: r.clone(), value: MirValue::Integer(n) });
                    }
                    Some(r)
                } else { None }
            }
            AstNode::StringLiteral { value, .. } => {
                let r = self.fresh_reg();
                if let Some(block) = func.get_block_mut(current_block) { block.instructions.push(MirInstruction::LoadImmediate { dest: r.clone(), value: MirValue::String((*value).to_string()) }); }
                Some(r)
            }
            AstNode::Word(value) => {
                // 既知変数(レジスタ)ならロード不要で再利用
                if let Some(reg) = self.var_env.get(*value) { return Some(reg.clone()); }
                let r = self.fresh_reg();
                if let Some(block) = func.get_block_mut(current_block) { block.instructions.push(MirInstruction::LoadImmediate { dest: r.clone(), value: MirValue::String((*value).to_string()) }); }
                Some(r)
            }
            AstNode::Try { body, catch_clauses, finally_clause } => {
                self.lower_node_prog(body, prog, func, current_block);
                for clause in catch_clauses { self.lower_node_prog(&clause.body, prog, func, current_block); }
                if let Some(fin) = finally_clause { self.lower_node_prog(fin, prog, func, current_block); }
                None
            }
            AstNode::Closure { body, captures, params, .. } => {
                let body_block = func.create_block();
                // 既存環境保存 (ネスト用)
                let saved_env = self.var_env.clone();
                let was_in_closure = self.in_closure;
                self.in_closure = true;
                // パラメータレジスタ割当て & 環境反映
                let mut param_regs = Vec::new();
                let mut param_names = Vec::new();
                for p in params {
                    let r = self.fresh_reg();
                    self.var_env.insert(p.name.to_string(), r.clone());
                    param_regs.push(r);
                    param_names.push(p.name.to_string());
                }
                // capture_regs: body 内で参照するローカルコピー (実行時に ClosureCall で値注入)
                let mut capture_regs = Vec::new();
                for cap in captures {
                    // 外側 (saved_env) で見つかったらそのレジスタを capture 値として使用。body 内では新レジスタを割当。
                    let new_reg = self.fresh_reg();
                    self.var_env.insert((*cap).to_string(), new_reg.clone());
                    capture_regs.push(new_reg);
                }
                // body lowering（クロージャ内であることを示すフラグのもとで）
                self.lower_node_prog(body, prog, func, body_block);
                if let Some(bblk) = func.get_block_mut(body_block) {
                    if !matches!(bblk.instructions.last(), Some(MirInstruction::Return { .. }) | Some(MirInstruction::ClosureReturn { .. })) {
                        bblk.instructions.push(MirInstruction::Return { value: Some(MirValue::Null) });
                    }
                }
                let dest = self.fresh_reg();
                // capture_vals: 外側の元レジスタがあればそれ、無ければ文字列識別子
                let capture_vals: Vec<MirValue> = captures.iter().map(|c| {
                    if let Some(orig) = saved_env.get(*c) { MirValue::Register(orig.clone()) } else { MirValue::String(c.to_string()) }
                }).collect();
                if let Some(block) = func.get_block_mut(current_block) {
                    block.instructions.push(MirInstruction::ClosureCreate { dest: dest.clone(), func_block: body_block, captures: capture_vals, capture_regs, param_regs, param_names });
                }
                // 環境復元 (クロージャスコープ脱出)
                self.var_env = saved_env;
                self.in_closure = was_in_closure;
                Some(dest)
            }
            AstNode::VariableAssignment { name, value, .. } => {
                // 代入値を lower し、環境へ登録 (値がレジスタならキャプチャ対象として利用可能)
                if let Some(r) = self.lower_node_prog(value, prog, func, current_block) { self.var_env.insert(name.to_string(), r.clone()); Some(r) } else { None }
            }
            AstNode::MacroInvocation { name, .. } => {
                let reg = self.fresh_reg();
                if let Some(block) = func.get_block_mut(current_block) {
                    block.instructions.push(MirInstruction::LoadImmediate { dest: reg.clone(), value: MirValue::String(format!("macro:{name}")) });
                }
                Some(reg)
            }
            AstNode::Command { name, args, .. } => {
                let mut parts = Vec::new();
                if let AstNode::Word(w) = &**name { parts.push(w.to_string()); }
                for a in args { if let AstNode::Word(w) = a { parts.push(w.to_string()); } }
                let r = self.fresh_reg();
                if let Some(block) = func.get_block_mut(current_block) {
                    block.instructions.push(MirInstruction::ExecuteCommand { dest: r.clone(), command: parts.first().cloned().unwrap_or_default(), args: Vec::new() });
                }
                Some(r)
            }
            AstNode::Return(expr) => {
                // 先に式を lower (これで current_block へ追加) し終えてから、再度 block を取り直す
                let val = if let Some(e) = expr { self.lower_node_prog(e, prog, func, current_block).map(MirValue::Register).unwrap_or(MirValue::Null) } else { MirValue::Null };
                if let Some(block) = func.get_block_mut(current_block) {
                    block.instructions.push(MirInstruction::Return { value: Some(val) });
                }
                None
            }
            AstNode::BinaryExpression { left, operator, right } => {
                use BinaryOperator::*;
                // Lower left-hand side first
                let lreg = self.lower_node_prog(left, prog, func, current_block);
                let dest = self.fresh_reg();
                match operator {
                    LogicalAnd | LogicalOr => {
                        if let Some(lr) = lreg.clone() {
                            // Insert placeholder AndSC/OrSC, then inline-lower RHS and patch skip and right register
                            let and_idx: usize;
                            if let Some(block) = func.get_block_mut(current_block) {
                                and_idx = block.instructions.len();
                                let ins = match operator {
                                    LogicalAnd => MirInstruction::AndSC { dest: dest.clone(), left: MirValue::Register(lr.clone()), right: MirValue::Null, skip: 0 },
                                    LogicalOr => MirInstruction::OrSC { dest: dest.clone(), left: MirValue::Register(lr.clone()), right: MirValue::Null, skip: 0 },
                                    _ => unreachable!(),
                                };
                                block.instructions.push(ins);
                            } else { return None; }

                            // Record length before lowering RHS
                            let pre_len = func.get_block(current_block).map(|b| b.instructions.len()).unwrap_or(0);
                            let rreg = self.lower_node_prog(right, prog, func, current_block);
                            // Ensure RHS final value is written into dest to be consumed after short-circuit gate
                            if let Some(rr) = rreg.clone() {
                                if let Some(block) = func.get_block_mut(current_block) {
                                    block.instructions.push(MirInstruction::Move { dest: dest.clone(), src: rr.clone() });
                                }
                            }
                            let post_len = func.get_block(current_block).map(|b| b.instructions.len()).unwrap_or(pre_len);
                            let rhs_count = if post_len >= pre_len { (post_len - pre_len) as u32 } else { 0 };

                            if let Some(block) = func.get_block_mut(current_block) {
                                if let Some(entry) = block.instructions.get_mut(and_idx) {
                                    match entry {
                                        MirInstruction::AndSC { skip, right, .. } => { *skip = rhs_count; if let Some(rr) = rreg { *right = MirValue::Register(rr); } },
                                        MirInstruction::OrSC { skip, right, .. } => { *skip = rhs_count; if let Some(rr) = rreg { *right = MirValue::Register(rr); } },
                                        _ => {}
                                    }
                                }
                            }
                        } else if let Some(block) = func.get_block_mut(current_block) {
                            // Failed to lower LHS; produce null result
                            block.instructions.push(MirInstruction::LoadImmediate { dest: dest.clone(), value: MirValue::Null });
                        }
                    }
                    _ => {
                        let rreg = self.lower_node_prog(right, prog, func, current_block);
                        if let (Some(lr), Some(rr)) = (lreg.clone(), rreg.clone()) {
                            if let Some(block) = func.get_block_mut(current_block) {
                                let ins = match operator {
                                    Add => MirInstruction::Add { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    Subtract => MirInstruction::Sub { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    Multiply => MirInstruction::Mul { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    Divide => MirInstruction::Div { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    Modulo => MirInstruction::Mod { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    Power => MirInstruction::Pow { dest: dest.clone(), base: MirValue::Register(lr), exp: MirValue::Register(rr) },
                                    Equal => MirInstruction::Equal { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    NotEqual => MirInstruction::NotEqual { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    Less => MirInstruction::LessThan { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    LessEqual => MirInstruction::LessEqual { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    Greater => MirInstruction::GreaterThan { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    GreaterEqual => MirInstruction::GreaterEqual { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    BitwiseAnd => MirInstruction::BitAnd { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    BitwiseOr => MirInstruction::BitOr { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    BitwiseXor => MirInstruction::BitXor { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    LeftShift => MirInstruction::Shl { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    RightShift => MirInstruction::Shr { dest: dest.clone(), left: MirValue::Register(lr), right: MirValue::Register(rr) },
                                    Match => MirInstruction::RegexMatch { dest: dest.clone(), value: MirValue::Register(lr), pattern: MirValue::Register(rr), not: false },
                                    NotMatch => MirInstruction::RegexMatch { dest: dest.clone(), value: MirValue::Register(lr), pattern: MirValue::Register(rr), not: true },
                                    LogicalAnd | LogicalOr => unreachable!(),
                                };
                                block.instructions.push(ins);
                            }
                        } else if let Some(block) = func.get_block_mut(current_block) {
                            block.instructions.push(MirInstruction::LoadImmediate { dest: dest.clone(), value: MirValue::Null });
                        }
                    }
                }
                Some(dest)
            }
            AstNode::Match { expr, arms, .. } => {
                if let Some(val_reg) = self.lower_node_prog(expr, prog, func, current_block) {
                    let mut arm_pairs = Vec::new();
                    for (i, arm) in arms.iter().enumerate() { if let nxsh_parser::ast::Pattern::Literal(lit) = arm.pattern { arm_pairs.push((MirValue::String(lit.to_string()), i as u32 + 100)); } }
                    if let Some(block) = func.get_block_mut(current_block) {
                        block.instructions.push(MirInstruction::MatchDispatch { value: MirValue::Register(val_reg), arms: arm_pairs, default_block: None });
                    }
                }
                None
            }
            _ => None,
        }
    }
}
