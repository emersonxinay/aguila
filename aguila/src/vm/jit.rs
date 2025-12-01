use crate::vm::chunk::{Chunk, OpCode};
use crate::vm::value::{Value as AgValue, QNAN, TAG_FALSE, TAG_INT, TAG_NULO, TAG_TRUE};
use cranelift::codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift::codegen::ir::{
    types, AbiParam, Block, FuncRef, InstBuilder, MemFlags, StackSlot, StackSlotData, StackSlotKind,
};
use cranelift::codegen::settings::{self, Configurable};
use cranelift::codegen::Context;
use cranelift::prelude::*;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use std::collections::HashMap;

const MAX_INLINE_DEPTH: usize = 0;

pub struct Jit {
    builder_context: FunctionBuilderContext,
    ctx: Context,
    module: JITModule,
    counter: usize,
}

impl Jit {
    pub fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("opt_level", "speed").unwrap();
        flag_builder.set("enable_alias_analysis", "true").unwrap();
        flag_builder.set("enable_verifier", "true").unwrap();

        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();
        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        let module = JITModule::new(builder);

        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
            counter: 0,
        }
    }

    // Firma: fn(entry_pc: usize, arg0: i64, constants: *const u64) -> i64
    pub fn compile(
        &mut self,
        chunk: &Chunk,
        start_pc: usize,
        use_int_mode: bool,
    ) -> Result<usize, String> {
        self.module.clear_context(&mut self.ctx);

        let pointer_type = self.module.target_config().pointer_type();
        self.ctx
            .func
            .signature
            .params
            .push(AbiParam::new(pointer_type));
        self.ctx
            .func
            .signature
            .params
            .push(AbiParam::new(types::I64));
        self.ctx
            .func
            .signature
            .params
            .push(AbiParam::new(pointer_type));
        self.ctx
            .func
            .signature
            .returns
            .push(AbiParam::new(types::I64));

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);

        let entry_pc_val = builder.block_params(entry_block)[0];
        let arg0_val = builder.block_params(entry_block)[1];
        let consts_base = builder.block_params(entry_block)[2];

        let signature = builder.func.signature.clone();
        self.counter += 1;
        let func_name = format!("jit_func_{}_{}", start_pc, self.counter);
        let func_id = self
            .module
            .declare_function(&func_name, Linkage::Export, &signature)
            .map_err(|e| e.to_string())?;
        let func_ref = self.module.declare_func_in_func(func_id, builder.func);

        Self::compile_chunk(
            &mut builder,
            chunk,
            start_pc,
            use_int_mode,
            0,
            func_ref,
            entry_pc_val,
            consts_base,
            None,
            None,
            vec![arg0_val],
            pointer_type,
        )?;

        builder.seal_all_blocks();
        builder.finalize();

        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| e.to_string())?;
        self.module.clear_context(&mut self.ctx);
        self.module
            .finalize_definitions()
            .map_err(|e| e.to_string())?;

        let code = self.module.get_finalized_function(func_id);
        let ptr = unsafe { std::mem::transmute::<_, usize>(code) };
        Ok(ptr)
    }

    // Helper para inlining recursivo
    fn compile_chunk(
        builder: &mut FunctionBuilder,
        chunk: &Chunk,
        start_pc: usize,
        use_int_mode: bool,
        depth: usize,
        func_ref: FuncRef,
        entry_pc_val: Value,
        consts_base: Value,
        return_block: Option<Block>,
        return_ss: Option<StackSlot>,
        args: Vec<Value>,
        pointer_type: types::Type,
    ) -> Result<(), String> {
        let var_type = types::I64;

        // Slot para bitcast (usado solo si mezclamos tipos, pero con int mode usamos conversiones)
        let spill_slot =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 0));

        // Identificar bloques alcanzables (BFS)
        // 1. Identificar bloques alcanzables (BFS)
        let mut block_starts = std::collections::HashSet::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();

        block_starts.insert(start_pc);
        queue.push_back(start_pc);
        visited.insert(start_pc);

        let code_len = chunk.code.len();

        while let Some(popped_pc) = queue.pop_front() {
            let mut pc = popped_pc;
            while pc < code_len {
                // Si encontramos un inicio de bloque ya visitado (que no es el actual), terminamos este camino
                if pc != popped_pc && visited.contains(&pc) && block_starts.contains(&pc) {
                    break;
                }

                let instruction = chunk.code[pc];
                let op_code = (instruction >> 24) as u8;
                let bx = (instruction & 0xFFFF) as usize;

                let current_pc = pc;
                pc += 1;

                let op: OpCode = unsafe { std::mem::transmute(op_code) };

                match op {
                    OpCode::Saltar => {
                        let target = current_pc + 1 + bx;
                        if !visited.contains(&target) {
                            visited.insert(target);
                            block_starts.insert(target);
                            queue.push_back(target);
                        }
                        break; // Fin de bloque
                    }
                    OpCode::SaltarSiFalso => {
                        let target = current_pc + 1 + bx;
                        if !visited.contains(&target) {
                            visited.insert(target);
                            block_starts.insert(target);
                            queue.push_back(target);
                        }
                        // Fallthrough
                        let fallthrough = pc;
                        if !visited.contains(&fallthrough) {
                            visited.insert(fallthrough);
                            block_starts.insert(fallthrough);
                            queue.push_back(fallthrough);
                        }
                        break; // Fin de bloque
                    }
                    OpCode::SaltarAtras => {
                        let target = pc - bx;
                        if !visited.contains(&target) {
                            visited.insert(target);
                            block_starts.insert(target);
                            queue.push_back(target);
                        }
                        // Fallthrough (Loop header might fallthrough from previous block, but SaltarAtras itself usually jumps back.
                        // However, if condition is false? No, SaltarAtras is unconditional usually?
                        // In VM: `pc -= bx`. Unconditional.
                        // So NO fallthrough for SaltarAtras.
                        // Wait, VM implementation:
                        // OpCode::SaltarAtras => { pc -= bx; break; }
                        // It IS unconditional.
                        // So I should NOT add fallthrough.
                        break;
                    }
                    OpCode::Retornar => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        let mut sorted_starts: Vec<usize> = block_starts.iter().cloned().collect();
        sorted_starts.sort();

        let mut blocks = HashMap::new();
        for &pc in &sorted_starts {
            blocks.insert(pc, builder.create_block());
        }

        let mut vars = Vec::with_capacity(256);
        for _ in 0..256 {
            vars.push(builder.declare_var(var_type));
        }

        // Inicializar argumentos (R[0] = arg0, etc.)
        if !args.is_empty() {
            let arg0 = args[0];
            let val = if use_int_mode {
                let arg_f64 = builder.ins().bitcast(types::F64, MemFlags::new(), arg0);
                builder.ins().fcvt_to_sint(types::I64, arg_f64)
            } else {
                arg0
            };
            builder.def_var(vars[0], val);
        }

        // Saltar al bloque inicial del chunk
        builder.ins().jump(blocks[&start_pc], &[]);

        for &start_pc in &sorted_starts {
            let block = blocks[&start_pc];
            builder.switch_to_block(block);

            let mut pc = start_pc;
            let mut current_block_terminated = false;

            while pc < code_len {
                if pc != start_pc && block_starts.contains(&pc) {
                    if !current_block_terminated {
                        builder.ins().jump(blocks[&pc], &[]);
                    }
                    current_block_terminated = true;
                    break;
                }

                let instruction = chunk.code[pc];
                let op_code = (instruction >> 24) as u8;
                // let current_pc = pc;
                pc += 1;

                let op: OpCode = unsafe { std::mem::transmute(op_code) };
                let a = ((instruction >> 16) & 0xFF) as usize;
                let b = ((instruction >> 8) & 0xFF) as usize;
                let c = (instruction & 0xFF) as usize;
                let bx = (instruction & 0xFFFF) as usize;

                match op {
                    OpCode::CargarConstante => {
                        let offset_const = (bx * 8) as i32;

                        if use_int_mode {
                            // Constant Folding: Si es modo entero, intentamos leer la constante del chunk
                            // y emitir un iconst directo.
                            let const_val = chunk.constants[bx];
                            if const_val.es_entero() {
                                let int_val = const_val.a_entero() as i64;
                                let val_i64 = builder.ins().iconst(types::I64, int_val);
                                builder.def_var(vars[a], val_i64);
                            } else {
                                // Fallback: Cargar Value y des-etiquetar (Unsafe/Assume Int)
                                // Ojo: Si no es entero, esto romperá la lógica de IntFastPath.
                                // Asumimos que el código es correcto para este modo.
                                let val_tagged = builder.ins().load(
                                    types::I64,
                                    MemFlags::new(),
                                    consts_base,
                                    offset_const,
                                );
                                let val_i32 = builder.ins().ireduce(types::I32, val_tagged);
                                let val_i64 = builder.ins().sextend(types::I64, val_i32);
                                builder.def_var(vars[a], val_i64);
                            }
                        } else {
                            let val_i64 = builder.ins().load(
                                types::I64,
                                MemFlags::new(),
                                consts_base,
                                offset_const,
                            );
                            builder.def_var(vars[a], val_i64);
                        }
                    }
                    OpCode::Mover => {
                        let val = builder.use_var(vars[b]);
                        builder.def_var(vars[a], val);
                    }
                    OpCode::Sumar => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);

                        if use_int_mode {
                            // Fast Path: iadd directo
                            // Asumimos que val_b y val_c son i64 (raw ints)
                            let res = builder.ins().iadd(val_b, val_c);
                            builder.def_var(vars[a], res);
                        } else {
                            // val_b y val_c son I64 (bits de Value)
                            let val_b_i64 = val_b;
                            let val_c_i64 = val_c;

                            // Constantes para chequeo de tags
                            let tag_int = builder.ins().iconst(types::I64, TAG_INT as i64);
                            let qnan = builder.ins().iconst(types::I64, QNAN as i64);
                            let mask_tag = builder.ins().bor(qnan, tag_int); // QNAN | TAG_INT

                            // Chequear si ambos son enteros
                            let b_is_int = builder.ins().band(val_b_i64, mask_tag);
                            let c_is_int = builder.ins().band(val_c_i64, mask_tag);
                            let both_int_check = builder.ins().band(b_is_int, c_is_int);
                            let is_int_op =
                                builder.ins().icmp(IntCC::Equal, both_int_check, mask_tag);

                            let then_block = builder.create_block();
                            let else_block = builder.create_block();
                            let merge_block = builder.create_block();

                            builder
                                .ins()
                                .brif(is_int_op, then_block, &[], else_block, &[]);

                            // Bloque Entero
                            builder.switch_to_block(then_block);
                            let b_val_i32 = builder.ins().ireduce(types::I32, val_b_i64);
                            let c_val_i32 = builder.ins().ireduce(types::I32, val_c_i64);

                            let b_val_i64_clean = builder.ins().sextend(types::I64, b_val_i32);
                            let c_val_i64_clean = builder.ins().sextend(types::I64, c_val_i32);
                            let res_i64_calc = builder.ins().iadd(b_val_i64_clean, c_val_i64_clean);

                            let max_i32 = builder.ins().iconst(types::I64, i32::MAX as i64);
                            let min_i32 = builder.ins().iconst(types::I64, i32::MIN as i64);
                            let gt_max =
                                builder
                                    .ins()
                                    .icmp(IntCC::SignedGreaterThan, res_i64_calc, max_i32);
                            let lt_min =
                                builder
                                    .ins()
                                    .icmp(IntCC::SignedLessThan, res_i64_calc, min_i32);
                            let overflow_cond = builder.ins().bor(gt_max, lt_min);

                            let int_res_block = builder.create_block();
                            let float_fallback_block = builder.create_block();

                            builder.ins().brif(
                                overflow_cond,
                                float_fallback_block,
                                &[],
                                int_res_block,
                                &[],
                            );

                            builder.switch_to_block(int_res_block);
                            let res_u32_i64 = builder.ins().band_imm(res_i64_calc, 0xFFFFFFFF);
                            let res_tagged = builder.ins().bor(res_u32_i64, mask_tag);
                            // res_tagged es I64, listo para def_var
                            builder.def_var(vars[a], res_tagged);
                            builder.ins().jump(merge_block, &[]);

                            builder.switch_to_block(float_fallback_block);
                            let b_f64 = builder.ins().fcvt_from_sint(types::F64, b_val_i64_clean);
                            let c_f64 = builder.ins().fcvt_from_sint(types::F64, c_val_i64_clean);
                            let res_f64_overflow = builder.ins().fadd(b_f64, c_f64);
                            let res_i64_overflow = builder.ins().bitcast(
                                types::I64,
                                MemFlags::new(),
                                res_f64_overflow,
                            );
                            builder.def_var(vars[a], res_i64_overflow);
                            builder.ins().jump(merge_block, &[]);

                            // Bloque Float (Else)
                            builder.switch_to_block(else_block);

                            // Unbox B
                            let b_is_int_single =
                                builder.ins().icmp(IntCC::Equal, b_is_int, mask_tag);
                            let b_val_i32_reduced = builder.ins().ireduce(types::I32, val_b_i64);
                            let b_f64_conv =
                                builder.ins().fcvt_from_sint(types::F64, b_val_i32_reduced);
                            let val_b_f64 =
                                builder.ins().bitcast(types::F64, MemFlags::new(), val_b);
                            let b_real_f64 =
                                builder.ins().select(b_is_int_single, b_f64_conv, val_b_f64);

                            // Unbox C
                            let c_is_int_single =
                                builder.ins().icmp(IntCC::Equal, c_is_int, mask_tag);
                            let c_val_i32_reduced = builder.ins().ireduce(types::I32, val_c_i64);
                            let c_f64_conv =
                                builder.ins().fcvt_from_sint(types::F64, c_val_i32_reduced);
                            let val_c_f64 =
                                builder.ins().bitcast(types::F64, MemFlags::new(), val_c);
                            let c_real_f64 =
                                builder.ins().select(c_is_int_single, c_f64_conv, val_c_f64);

                            let res_float = builder.ins().fadd(b_real_f64, c_real_f64);
                            let res_i64_float =
                                builder
                                    .ins()
                                    .bitcast(types::I64, MemFlags::new(), res_float);
                            builder.def_var(vars[a], res_i64_float);
                            builder.ins().jump(merge_block, &[]);

                            builder.switch_to_block(merge_block);
                        }
                    }
                    OpCode::Restar => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);

                        if use_int_mode {
                            // Fast Path: isub directo
                            let res = builder.ins().isub(val_b, val_c);
                            builder.def_var(vars[a], res);
                        } else {
                            let val_b_i64 = val_b;
                            let val_c_i64 = val_c;

                            let tag_int = builder.ins().iconst(types::I64, TAG_INT as i64);
                            let qnan = builder.ins().iconst(types::I64, QNAN as i64);
                            let mask_tag = builder.ins().bor(qnan, tag_int);

                            let b_is_int = builder.ins().band(val_b_i64, mask_tag);
                            let c_is_int = builder.ins().band(val_c_i64, mask_tag);
                            let both_int_check = builder.ins().band(b_is_int, c_is_int);
                            let is_int_op =
                                builder.ins().icmp(IntCC::Equal, both_int_check, mask_tag);

                            let then_block = builder.create_block();
                            let else_block = builder.create_block();
                            let merge_block = builder.create_block();

                            builder
                                .ins()
                                .brif(is_int_op, then_block, &[], else_block, &[]);

                            builder.switch_to_block(then_block);
                            let b_val_i32 = builder.ins().ireduce(types::I32, val_b_i64);
                            let c_val_i32 = builder.ins().ireduce(types::I32, val_c_i64);

                            let b_val_i64_clean = builder.ins().sextend(types::I64, b_val_i32);
                            let c_val_i64_clean = builder.ins().sextend(types::I64, c_val_i32);
                            let res_i64_calc = builder.ins().isub(b_val_i64_clean, c_val_i64_clean);

                            let max_i32 = builder.ins().iconst(types::I64, i32::MAX as i64);
                            let min_i32 = builder.ins().iconst(types::I64, i32::MIN as i64);
                            let gt_max =
                                builder
                                    .ins()
                                    .icmp(IntCC::SignedGreaterThan, res_i64_calc, max_i32);
                            let lt_min =
                                builder
                                    .ins()
                                    .icmp(IntCC::SignedLessThan, res_i64_calc, min_i32);
                            let overflow_cond = builder.ins().bor(gt_max, lt_min);

                            let int_res_block = builder.create_block();
                            let float_fallback_block = builder.create_block();

                            builder.ins().brif(
                                overflow_cond,
                                float_fallback_block,
                                &[],
                                int_res_block,
                                &[],
                            );

                            builder.switch_to_block(int_res_block);
                            let res_u32_i64 = builder.ins().band_imm(res_i64_calc, 0xFFFFFFFF);
                            let res_tagged = builder.ins().bor(res_u32_i64, mask_tag);
                            builder.def_var(vars[a], res_tagged);
                            builder.ins().jump(merge_block, &[]);

                            builder.switch_to_block(float_fallback_block);
                            let b_f64 = builder.ins().fcvt_from_sint(types::F64, b_val_i64_clean);
                            let c_f64 = builder.ins().fcvt_from_sint(types::F64, c_val_i64_clean);
                            let res_f64_overflow = builder.ins().fsub(b_f64, c_f64);
                            let res_i64_overflow = builder.ins().bitcast(
                                types::I64,
                                MemFlags::new(),
                                res_f64_overflow,
                            );
                            builder.def_var(vars[a], res_i64_overflow);
                            builder.ins().jump(merge_block, &[]);

                            builder.switch_to_block(else_block);
                            // Unbox B
                            let b_is_int_single =
                                builder.ins().icmp(IntCC::Equal, b_is_int, mask_tag);
                            let b_val_i32_reduced = builder.ins().ireduce(types::I32, val_b_i64);
                            let b_f64_conv =
                                builder.ins().fcvt_from_sint(types::F64, b_val_i32_reduced);
                            let val_b_f64 =
                                builder.ins().bitcast(types::F64, MemFlags::new(), val_b);
                            let b_real_f64 =
                                builder.ins().select(b_is_int_single, b_f64_conv, val_b_f64);

                            // Unbox C
                            let c_is_int_single =
                                builder.ins().icmp(IntCC::Equal, c_is_int, mask_tag);
                            let c_val_i32_reduced = builder.ins().ireduce(types::I32, val_c_i64);
                            let c_f64_conv =
                                builder.ins().fcvt_from_sint(types::F64, c_val_i32_reduced);
                            let val_c_f64 =
                                builder.ins().bitcast(types::F64, MemFlags::new(), val_c);
                            let c_real_f64 =
                                builder.ins().select(c_is_int_single, c_f64_conv, val_c_f64);

                            let res_float = builder.ins().fsub(b_real_f64, c_real_f64);
                            let res_i64_float =
                                builder
                                    .ins()
                                    .bitcast(types::I64, MemFlags::new(), res_float);
                            builder.def_var(vars[a], res_i64_float);
                            builder.ins().jump(merge_block, &[]);

                            builder.switch_to_block(merge_block);
                        }
                    }
                    OpCode::Multiplicar => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);

                        if use_int_mode {
                            // Fast Path: imul directo
                            let res = builder.ins().imul(val_b, val_c);
                            builder.def_var(vars[a], res);
                        } else {
                            let val_b_i64 = val_b;
                            let val_c_i64 = val_c;

                            let tag_int = builder.ins().iconst(types::I64, TAG_INT as i64);
                            let qnan = builder.ins().iconst(types::I64, QNAN as i64);
                            let mask_tag = builder.ins().bor(qnan, tag_int);

                            let b_is_int = builder.ins().band(val_b_i64, mask_tag);
                            let c_is_int = builder.ins().band(val_c_i64, mask_tag);
                            let both_int_check = builder.ins().band(b_is_int, c_is_int);
                            let is_int_op =
                                builder.ins().icmp(IntCC::Equal, both_int_check, mask_tag);

                            let then_block = builder.create_block();
                            let else_block = builder.create_block();
                            let merge_block = builder.create_block();

                            builder
                                .ins()
                                .brif(is_int_op, then_block, &[], else_block, &[]);

                            // Bloque Entero
                            builder.switch_to_block(then_block);
                            let b_val_i32 = builder.ins().ireduce(types::I32, val_b_i64);
                            let c_val_i32 = builder.ins().ireduce(types::I32, val_c_i64);

                            let b_val_i64_clean = builder.ins().sextend(types::I64, b_val_i32);
                            let c_val_i64_clean = builder.ins().sextend(types::I64, c_val_i32);
                            let res_i64_calc = builder.ins().imul(b_val_i64_clean, c_val_i64_clean);

                            // Overflow check for multiplication
                            // i32 * i32 can fit in i64.
                            // But result must fit in i32 to stay integer.
                            let max_i32 = builder.ins().iconst(types::I64, i32::MAX as i64);
                            let min_i32 = builder.ins().iconst(types::I64, i32::MIN as i64);
                            let gt_max =
                                builder
                                    .ins()
                                    .icmp(IntCC::SignedGreaterThan, res_i64_calc, max_i32);
                            let lt_min =
                                builder
                                    .ins()
                                    .icmp(IntCC::SignedLessThan, res_i64_calc, min_i32);
                            let overflow_cond = builder.ins().bor(gt_max, lt_min);

                            let int_res_block = builder.create_block();
                            let float_fallback_block = builder.create_block();

                            builder.ins().brif(
                                overflow_cond,
                                float_fallback_block,
                                &[],
                                int_res_block,
                                &[],
                            );

                            builder.switch_to_block(int_res_block);
                            let res_u32_i64 = builder.ins().band_imm(res_i64_calc, 0xFFFFFFFF);
                            let res_tagged = builder.ins().bor(res_u32_i64, mask_tag);
                            builder.def_var(vars[a], res_tagged);
                            builder.ins().jump(merge_block, &[]);

                            builder.switch_to_block(float_fallback_block);
                            let b_f64 = builder.ins().fcvt_from_sint(types::F64, b_val_i64_clean);
                            let c_f64 = builder.ins().fcvt_from_sint(types::F64, c_val_i64_clean);
                            let res_f64_overflow = builder.ins().fmul(b_f64, c_f64);
                            let res_i64_overflow = builder.ins().bitcast(
                                types::I64,
                                MemFlags::new(),
                                res_f64_overflow,
                            );
                            builder.def_var(vars[a], res_i64_overflow);
                            builder.ins().jump(merge_block, &[]);

                            // Bloque Float
                            builder.switch_to_block(else_block);
                            // Unbox helpers
                            let b_is_int_single =
                                builder.ins().icmp(IntCC::Equal, b_is_int, mask_tag);
                            let b_val_i32_reduced = builder.ins().ireduce(types::I32, val_b_i64);
                            let b_f64_conv =
                                builder.ins().fcvt_from_sint(types::F64, b_val_i32_reduced);
                            let val_b_f64 =
                                builder.ins().bitcast(types::F64, MemFlags::new(), val_b);
                            let b_real_f64 =
                                builder.ins().select(b_is_int_single, b_f64_conv, val_b_f64);

                            let c_is_int_single =
                                builder.ins().icmp(IntCC::Equal, c_is_int, mask_tag);
                            let c_val_i32_reduced = builder.ins().ireduce(types::I32, val_c_i64);
                            let c_f64_conv =
                                builder.ins().fcvt_from_sint(types::F64, c_val_i32_reduced);
                            let val_c_f64 =
                                builder.ins().bitcast(types::F64, MemFlags::new(), val_c);
                            let c_real_f64 =
                                builder.ins().select(c_is_int_single, c_f64_conv, val_c_f64);

                            let res_float = builder.ins().fmul(b_real_f64, c_real_f64);
                            let res_i64_float =
                                builder
                                    .ins()
                                    .bitcast(types::I64, MemFlags::new(), res_float);
                            builder.def_var(vars[a], res_i64_float);
                            builder.ins().jump(merge_block, &[]);

                            builder.switch_to_block(merge_block);
                        }
                    }
                    OpCode::Dividir => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);

                        if use_int_mode {
                            // Fast Path: sdiv (división entera con signo)
                            // TODO: Chequear división por cero
                            let res = builder.ins().sdiv(val_b, val_c);
                            builder.def_var(vars[a], res);
                        } else {
                            let val_b_i64 = val_b;
                            let val_c_i64 = val_c;

                            let tag_int = builder.ins().iconst(types::I64, TAG_INT as i64);
                            let qnan = builder.ins().iconst(types::I64, QNAN as i64);
                            let mask_tag = builder.ins().bor(qnan, tag_int);

                            let b_is_int = builder.ins().band(val_b_i64, mask_tag);
                            let c_is_int = builder.ins().band(val_c_i64, mask_tag);

                            // Unbox B
                            let b_is_int_single =
                                builder.ins().icmp(IntCC::Equal, b_is_int, mask_tag);
                            let b_val_i32_reduced = builder.ins().ireduce(types::I32, val_b_i64);
                            let b_f64_conv =
                                builder.ins().fcvt_from_sint(types::F64, b_val_i32_reduced);
                            let val_b_f64 =
                                builder.ins().bitcast(types::F64, MemFlags::new(), val_b);
                            let b_real_f64 =
                                builder.ins().select(b_is_int_single, b_f64_conv, val_b_f64);

                            // Unbox C
                            let c_is_int_single =
                                builder.ins().icmp(IntCC::Equal, c_is_int, mask_tag);
                            let c_val_i32_reduced = builder.ins().ireduce(types::I32, val_c_i64);
                            let c_f64_conv =
                                builder.ins().fcvt_from_sint(types::F64, c_val_i32_reduced);
                            let val_c_f64 =
                                builder.ins().bitcast(types::F64, MemFlags::new(), val_c);
                            let c_real_f64 =
                                builder.ins().select(c_is_int_single, c_f64_conv, val_c_f64);

                            let res_float = builder.ins().fdiv(b_real_f64, c_real_f64);
                            let res_i64_float =
                                builder
                                    .ins()
                                    .bitcast(types::I64, MemFlags::new(), res_float);
                            builder.def_var(vars[a], res_i64_float);
                        }
                    }
                    OpCode::Modulo => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);

                        if use_int_mode {
                            let res = builder.ins().srem(val_b, val_c);
                            builder.def_var(vars[a], res);
                        } else {
                            // TODO: Implementar fmod para floats (requiere libcall o instrucción compleja)
                            // Por ahora, fallback a interpreter si no es int mode
                            return Err("JIT: Modulo no soportado en modo float".to_string());
                        }
                    }
                    OpCode::Potencia => {
                        // Potencia requiere libcall (pow), difícil de inline en Cranelift simple
                        return Err("JIT: Potencia no soportada".to_string());
                    }
                    OpCode::BitAnd => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);
                        if use_int_mode {
                            let res = builder.ins().band(val_b, val_c);
                            builder.def_var(vars[a], res);
                        } else {
                            return Err("JIT: BitAnd solo en modo int".to_string());
                        }
                    }
                    OpCode::BitOr => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);
                        if use_int_mode {
                            let res = builder.ins().bor(val_b, val_c);
                            builder.def_var(vars[a], res);
                        } else {
                            return Err("JIT: BitOr solo en modo int".to_string());
                        }
                    }
                    OpCode::BitXor => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);
                        if use_int_mode {
                            let res = builder.ins().bxor(val_b, val_c);
                            builder.def_var(vars[a], res);
                        } else {
                            return Err("JIT: BitXor solo en modo int".to_string());
                        }
                    }
                    OpCode::ShiftIzq => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);
                        if use_int_mode {
                            let res = builder.ins().ishl(val_b, val_c);
                            builder.def_var(vars[a], res);
                        } else {
                            return Err("JIT: ShiftIzq solo en modo int".to_string());
                        }
                    }
                    OpCode::ShiftDer => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);
                        if use_int_mode {
                            let res = builder.ins().ushr(val_b, val_c); // Logical shift right
                            builder.def_var(vars[a], res);
                        } else {
                            return Err("JIT: ShiftDer solo en modo int".to_string());
                        }
                    }
                    OpCode::Retornar => {
                        let val = builder.use_var(vars[a]);
                        let ret_val = if use_int_mode {
                            let val_f64 = builder.ins().fcvt_from_sint(types::F64, val);
                            builder.ins().bitcast(types::I64, MemFlags::new(), val_f64)
                        } else {
                            val
                        };
                        if let Some(ret_blk) = return_block {
                            if let Some(ss) = return_ss {
                                builder.ins().stack_store(val, ss, 0);
                            }
                            builder.ins().jump(ret_blk, &[]);
                        } else {
                            builder.ins().return_(&[ret_val]);
                        }
                        // current_block_terminated = true;
                        break;
                    }
                    OpCode::Saltar => {
                        let target = pc + bx;
                        let block = blocks[&target];
                        builder.ins().jump(block, &[]);
                        // current_block_terminated = true;
                        break;
                    }
                    OpCode::SaltarAtras => {
                        let target = pc - bx;
                        let block = blocks[&target];
                        builder.ins().jump(block, &[]);
                        // current_block_terminated = true;
                        break;
                    }
                    OpCode::SaltarSiFalso => {
                        let val = builder.use_var(vars[a]);

                        if use_int_mode {
                            // En modo entero, 0 es falso, cualquier otra cosa es verdadero?
                            // O seguimos usando tags?
                            // Si vars[a] es un entero crudo, no tiene tags de Falso/Nulo.
                            // PERO, SaltarSiFalso se usa para `si n <= 1`.
                            // `Menor` devuelve un booleano.
                            // En modo entero, `Menor` debería devolver 0 o 1 (entero).
                            // Así que aquí chequeamos si es 0.
                            let zero = builder.ins().iconst(types::I64, 0);
                            let is_false = builder.ins().icmp(IntCC::Equal, val, zero);

                            let target = pc + bx;
                            let target_block = blocks[&target];
                            let next_block = blocks[&pc];

                            builder
                                .ins()
                                .brif(is_false, target_block, &[], next_block, &[]);
                        } else {
                            // val es I64
                            let val_i64 = val;
                            let qnan = builder.ins().iconst(types::I64, QNAN as i64);
                            let tag_false = builder.ins().iconst(types::I64, TAG_FALSE as i64);
                            let tag_nulo = builder.ins().iconst(types::I64, TAG_NULO as i64);
                            let val_false = builder.ins().bor(qnan, tag_false);
                            let val_nulo = builder.ins().bor(qnan, tag_nulo);
                            let is_false = builder.ins().icmp(IntCC::Equal, val_i64, val_false);
                            let is_nulo = builder.ins().icmp(IntCC::Equal, val_i64, val_nulo);
                            let cond = builder.ins().bor(is_false, is_nulo);

                            let target = pc + bx;
                            let target_block = blocks[&target];
                            let next_block = blocks[&pc];

                            builder.ins().brif(cond, target_block, &[], next_block, &[]);
                        }
                        // current_block_terminated = true;
                        break;
                    }
                    OpCode::Menor => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);

                        if use_int_mode {
                            // Fast Path: icmp directo
                            // Devuelve 1 si true, 0 si false (como entero)
                            let cmp = builder.ins().icmp(IntCC::SignedLessThan, val_b, val_c);

                            // Usar select en lugar de bint si bint falla
                            let one = builder.ins().iconst(types::I64, 1);
                            let zero = builder.ins().iconst(types::I64, 0);
                            let res = builder.ins().select(cmp, one, zero);

                            builder.def_var(vars[a], res);
                        } else {
                            let val_b_i64 = val_b;
                            let val_c_i64 = val_c;

                            let tag_int = builder.ins().iconst(types::I64, TAG_INT as i64);
                            let qnan = builder.ins().iconst(types::I64, QNAN as i64);
                            let mask_tag = builder.ins().bor(qnan, tag_int);

                            let b_is_int = builder.ins().band(val_b_i64, mask_tag);
                            let c_is_int = builder.ins().band(val_c_i64, mask_tag);
                            let both_int_check = builder.ins().band(b_is_int, c_is_int);
                            let is_int_op =
                                builder.ins().icmp(IntCC::Equal, both_int_check, mask_tag);

                            let then_block = builder.create_block();
                            let else_block = builder.create_block();
                            let merge_block = builder.create_block();

                            builder
                                .ins()
                                .brif(is_int_op, then_block, &[], else_block, &[]);

                            // Bloque Entero
                            builder.switch_to_block(then_block);
                            let b_val_i32 = builder.ins().ireduce(types::I32, val_b_i64);
                            let c_val_i32 = builder.ins().ireduce(types::I32, val_c_i64);
                            let cmp_int =
                                builder
                                    .ins()
                                    .icmp(IntCC::SignedLessThan, b_val_i32, c_val_i32);

                            // Convertir bool a Value::logico
                            let tag_true = builder.ins().iconst(types::I64, TAG_TRUE as i64);
                            let tag_false = builder.ins().iconst(types::I64, TAG_FALSE as i64);
                            let val_true = builder.ins().bor(qnan, tag_true);
                            let val_false = builder.ins().bor(qnan, tag_false);
                            let res_i64_int = builder.ins().select(cmp_int, val_true, val_false);
                            builder.def_var(vars[a], res_i64_int);
                            builder.ins().jump(merge_block, &[]);

                            // Bloque Float
                            builder.switch_to_block(else_block);
                            // Unbox helpers
                            let b_is_int_single =
                                builder.ins().icmp(IntCC::Equal, b_is_int, mask_tag);
                            let b_val_i32_reduced = builder.ins().ireduce(types::I32, val_b_i64);
                            let b_f64_conv =
                                builder.ins().fcvt_from_sint(types::F64, b_val_i32_reduced);
                            let val_b_f64 =
                                builder.ins().bitcast(types::F64, MemFlags::new(), val_b);
                            let b_real_f64 =
                                builder.ins().select(b_is_int_single, b_f64_conv, val_b_f64);

                            let c_is_int_single =
                                builder.ins().icmp(IntCC::Equal, c_is_int, mask_tag);
                            let c_val_i32_reduced = builder.ins().ireduce(types::I32, val_c_i64);
                            let c_f64_conv =
                                builder.ins().fcvt_from_sint(types::F64, c_val_i32_reduced);
                            let val_c_f64 =
                                builder.ins().bitcast(types::F64, MemFlags::new(), val_c);
                            let c_real_f64 =
                                builder.ins().select(c_is_int_single, c_f64_conv, val_c_f64);

                            let cmp_float =
                                builder
                                    .ins()
                                    .fcmp(FloatCC::LessThan, b_real_f64, c_real_f64);

                            let val_true_f = builder.ins().bor(qnan, tag_true);
                            let val_false_f = builder.ins().bor(qnan, tag_false);
                            let res_i64_float =
                                builder.ins().select(cmp_float, val_true_f, val_false_f);
                            builder.def_var(vars[a], res_i64_float);
                            builder.ins().jump(merge_block, &[]);

                            builder.switch_to_block(merge_block);
                        }
                    }
                    OpCode::MenorIgual => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);

                        if use_int_mode {
                            // Fast Path: icmp directo
                            let cmp =
                                builder
                                    .ins()
                                    .icmp(IntCC::SignedLessThanOrEqual, val_b, val_c);

                            // Usar select en lugar de bint si bint falla
                            let one = builder.ins().iconst(types::I64, 1);
                            let zero = builder.ins().iconst(types::I64, 0);
                            let res = builder.ins().select(cmp, one, zero);

                            builder.def_var(vars[a], res);
                        } else {
                            let val_b_i64 = val_b;
                            let val_c_i64 = val_c;

                            let tag_int = builder.ins().iconst(types::I64, TAG_INT as i64);
                            let qnan = builder.ins().iconst(types::I64, QNAN as i64);
                            let mask_tag = builder.ins().bor(qnan, tag_int);

                            let b_is_int = builder.ins().band(val_b_i64, mask_tag);
                            let c_is_int = builder.ins().band(val_c_i64, mask_tag);
                            let both_int_check = builder.ins().band(b_is_int, c_is_int);
                            let is_int_op =
                                builder.ins().icmp(IntCC::Equal, both_int_check, mask_tag);

                            let then_block = builder.create_block();
                            let else_block = builder.create_block();
                            let merge_block = builder.create_block();

                            builder
                                .ins()
                                .brif(is_int_op, then_block, &[], else_block, &[]);

                            // Bloque Entero
                            builder.switch_to_block(then_block);
                            let b_val_i32 = builder.ins().ireduce(types::I32, val_b_i64);
                            let c_val_i32 = builder.ins().ireduce(types::I32, val_c_i64);
                            let cmp_int = builder.ins().icmp(
                                IntCC::SignedLessThanOrEqual,
                                b_val_i32,
                                c_val_i32,
                            );

                            // Convertir bool a Value::logico
                            let tag_true = builder.ins().iconst(types::I64, TAG_TRUE as i64);
                            let tag_false = builder.ins().iconst(types::I64, TAG_FALSE as i64);
                            let val_true = builder.ins().bor(qnan, tag_true);
                            let val_false = builder.ins().bor(qnan, tag_false);
                            let res_i64_int = builder.ins().select(cmp_int, val_true, val_false);
                            builder.def_var(vars[a], res_i64_int);
                            builder.ins().jump(merge_block, &[]);

                            // Bloque Float
                            builder.switch_to_block(else_block);
                            // Unbox helpers
                            let b_is_int_single =
                                builder.ins().icmp(IntCC::Equal, b_is_int, mask_tag);
                            let b_val_i32_reduced = builder.ins().ireduce(types::I32, val_b_i64);
                            let b_f64_conv =
                                builder.ins().fcvt_from_sint(types::F64, b_val_i32_reduced);
                            let val_b_f64 =
                                builder.ins().bitcast(types::F64, MemFlags::new(), val_b);
                            let b_real_f64 =
                                builder.ins().select(b_is_int_single, b_f64_conv, val_b_f64);

                            let c_is_int_single =
                                builder.ins().icmp(IntCC::Equal, c_is_int, mask_tag);
                            let c_val_i32_reduced = builder.ins().ireduce(types::I32, val_c_i64);
                            let c_f64_conv =
                                builder.ins().fcvt_from_sint(types::F64, c_val_i32_reduced);
                            let val_c_f64 =
                                builder.ins().bitcast(types::F64, MemFlags::new(), val_c);
                            let c_real_f64 =
                                builder.ins().select(c_is_int_single, c_f64_conv, val_c_f64);

                            let cmp_float = builder.ins().fcmp(
                                FloatCC::LessThanOrEqual,
                                b_real_f64,
                                c_real_f64,
                            );

                            let val_true_f = builder.ins().bor(qnan, tag_true);
                            let val_false_f = builder.ins().bor(qnan, tag_false);
                            let res_i64_float =
                                builder.ins().select(cmp_float, val_true_f, val_false_f);
                            builder.def_var(vars[a], res_i64_float);
                            builder.ins().jump(merge_block, &[]);

                            builder.switch_to_block(merge_block);
                        }
                    }
                    OpCode::ObtenerGlobal => {
                        if use_int_mode {
                            // Hack para IntFastPath: Asumimos que ObtenerGlobal carga la función para recursión.
                            // Como Llamar usa func_ref estático (o inlining), no necesitamos el valor real.
                            // Cargamos 0 para satisfacer el def_var.
                            let dummy = builder.ins().iconst(types::I64, 0);
                            builder.def_var(vars[a], dummy);
                        } else {
                            println!("JIT Error at PC {}: OpCode {:?} not supported", pc - 1, op);
                            return Err(format!(
                                "JIT: OpCode no soportado en modo normal {:?}",
                                op
                            ));
                        }
                    }
                    OpCode::Llamar => {
                        // Llamada recursiva (asumimos recursión por ahora para fib)
                        let func_val = builder.use_var(vars[b]);
                        let arg0 = builder.use_var(vars[b + 1]);

                        if use_int_mode {
                            if depth < MAX_INLINE_DEPTH {
                                // Inlining Recursivo
                                let merge_block = builder.create_block();
                                let ss = builder.create_sized_stack_slot(StackSlotData::new(
                                    StackSlotKind::ExplicitSlot,
                                    8,
                                    0,
                                ));

                                // Compilar recursivamente el mismo chunk (inlining)
                                // Asumimos que es la misma función (fib)
                                Self::compile_chunk(
                                    builder,
                                    chunk,
                                    0, // start_pc siempre 0 para fib
                                    true,
                                    depth + 1,
                                    func_ref,
                                    entry_pc_val,
                                    consts_base,
                                    Some(merge_block),
                                    Some(ss),
                                    vec![arg0],
                                    pointer_type,
                                )?;

                                builder.switch_to_block(merge_block);
                                let ret_val = builder.ins().stack_load(types::I64, ss, 0);
                                builder.def_var(vars[a], ret_val);
                            } else {
                                // Fast Path: Llamada directa (sin inlining)
                                let arg0_boxed = if use_int_mode {
                                    let val_f64 = builder.ins().fcvt_from_sint(types::F64, arg0);
                                    builder.ins().bitcast(types::I64, MemFlags::new(), val_f64)
                                } else {
                                    arg0
                                };

                                let call = builder
                                    .ins()
                                    .call(func_ref, &[entry_pc_val, arg0_boxed, consts_base]);
                                let res_boxed = builder.inst_results(call)[0];

                                let res = if use_int_mode {
                                    let res_f64 = builder.ins().bitcast(
                                        types::F64,
                                        MemFlags::new(),
                                        res_boxed,
                                    );
                                    builder.ins().fcvt_to_sint(types::I64, res_f64)
                                } else {
                                    res_boxed
                                };
                                builder.def_var(vars[a], res);
                            }
                        } else {
                            let func_val_f64 =
                                builder.ins().bitcast(types::F64, MemFlags::new(), func_val);
                            let func_val_ptr =
                                builder.ins().fcvt_to_uint(pointer_type, func_val_f64);

                            let call = builder
                                .ins()
                                .call(func_ref, &[func_val_ptr, arg0, consts_base]);
                            let ret_val = builder.inst_results(call)[0];

                            builder.def_var(vars[a], ret_val);
                        }
                    }
                    OpCode::DefinirGlobal => {
                        println!(
                            "JIT Error at PC {}: DefinirGlobal found inside function!",
                            pc - 1
                        );
                        return Err("JIT: DefinirGlobal no soportado".to_string());
                    }
                    _ => {
                        return Err(format!("JIT: OpCode no soportado {:?}", op));
                    }
                }
            }
        }

        Ok(())
    }
}
