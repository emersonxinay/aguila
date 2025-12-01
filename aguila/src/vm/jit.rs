use crate::vm::chunk::{Chunk, OpCode};
use crate::vm::value::{QNAN, TAG_FALSE, TAG_NULO, TAG_TRUE};
use cranelift::codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift::codegen::ir::{
    types, AbiParam, InstBuilder, MemFlags, StackSlotData, StackSlotKind,
};
use cranelift::codegen::Context; // Importar Context explícitamente
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use std::collections::HashMap;

pub struct Jit {
    builder_context: FunctionBuilderContext,
    ctx: Context,
    module: JITModule,
}

impl Jit {
    pub fn new() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
        let module = JITModule::new(builder);

        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
        }
    }

    // Firma: fn(entry_pc: usize, arg0: f64, constants: *const u64) -> f64
    pub fn compile(
        &mut self,
        chunk: &Chunk,
        start_pc: usize,
    ) -> Result<fn(usize, f64, *const u64) -> f64, String> {
        self.module.clear_context(&mut self.ctx);

        let pointer_type = self.module.target_config().pointer_type();
        // Param 0: entry_pc
        self.ctx
            .func
            .signature
            .params
            .push(AbiParam::new(pointer_type));
        // Param 1: arg0 (f64) - Hack para fib(n)
        self.ctx
            .func
            .signature
            .params
            .push(AbiParam::new(types::F64));
        // Param 2: constants pointer
        self.ctx
            .func
            .signature
            .params
            .push(AbiParam::new(pointer_type));

        self.ctx
            .func
            .signature
            .returns
            .push(AbiParam::new(types::F64));

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        // Crear slot de pila para bitcast (f64 <-> i64). Align shift 0 (default alignment)
        let spill_slot =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 0));

        // Identificar bloques alcanzables (BFS)
        let mut block_starts = std::collections::HashSet::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();

        block_starts.insert(start_pc);
        queue.push_back(start_pc);
        visited.insert(start_pc);

        let code_len = chunk.code.len();

        while let Some(mut pc) = queue.pop_front() {
            // Escanear bloque linealmente hasta encontrar terminador o salto
            while pc < code_len {
                let instruction = chunk.code[pc];
                let op_code = (instruction >> 24) as u8;
                let bx = (instruction & 0xFFFF) as usize;

                // Avanzar PC para la siguiente instrucción (si no saltamos)
                // let current_pc = pc;
                pc += 1;

                let op: OpCode = unsafe { std::mem::transmute(op_code) };

                match op {
                    OpCode::Saltar => {
                        let target = pc + bx;
                        if !visited.contains(&target) {
                            visited.insert(target);
                            block_starts.insert(target);
                            queue.push_back(target);
                        }
                        break; // Fin de bloque
                    }
                    OpCode::SaltarSiFalso => {
                        let target = pc + bx;
                        if !visited.contains(&target) {
                            visited.insert(target);
                            block_starts.insert(target);
                            queue.push_back(target);
                        }
                        // Fallthrough es un nuevo bloque si hay salto condicional?
                        // Cranelift requiere que los bloques terminen en salto.
                        // Si hay fallthrough, el siguiente PC es inicio de bloque.
                        if !visited.contains(&pc) {
                            visited.insert(pc);
                            block_starts.insert(pc);
                            queue.push_back(pc);
                        }
                        break;
                    }
                    OpCode::SaltarAtras => {
                        let target = pc - bx;
                        if !visited.contains(&target) {
                            visited.insert(target);
                            block_starts.insert(target);
                            queue.push_back(target);
                        }
                        break;
                    }
                    OpCode::Retornar => {
                        break; // Fin de función/bloque
                    }
                    OpCode::Llamar => {
                        // Llamar no termina bloque en Águila (retorna), pero en JIT
                        // podría ser tratado como instrucción normal.
                        // No rompe el flujo de control local.
                    }
                    _ => {}
                }
            }
        }

        // Ordenar block_starts para iteración determinista
        let mut sorted_starts: Vec<usize> = block_starts.iter().cloned().collect();
        sorted_starts.sort();

        // Crear bloques Cranelift
        let mut blocks = HashMap::new();
        for &pc in &sorted_starts {
            blocks.insert(pc, builder.create_block());
        }

        // Bloque de entrada (Dispatch)
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);

        let entry_pc_val = builder.block_params(entry_block)[0];
        let arg0_val = builder.block_params(entry_block)[1];
        let consts_base = builder.block_params(entry_block)[2];

        // Declarar variables Cranelift para registros (0..255)
        let mut vars = Vec::with_capacity(256);
        for _ in 0..256 {
            vars.push(builder.declare_var(types::F64));
        }

        // Inicializar R[0] con arg0
        builder.def_var(vars[0], arg0_val);

        // Crear Jump Table (Switch)
        let mut switch = cranelift_frontend::Switch::new();
        for &pc in &sorted_starts {
            switch.set_entry(pc as u128, blocks[&pc]);
        }
        // Default block? Debería ser unreachable o trap si entry_pc no es válido.
        // Usamos el bloque de start_pc como default por seguridad, o un bloque trap.
        // Por ahora, start_pc.
        switch.emit(&mut builder, entry_pc_val, blocks[&start_pc]);

        let signature = builder.func.signature.clone();
        let func_id = self
            .module
            .declare_function("jit_func", Linkage::Export, &signature)
            .map_err(|e| e.to_string())?;
        let func_ref = self.module.declare_func_in_func(func_id, builder.func);

        // Generar código para cada bloque alcanzable
        for &start_pc in &sorted_starts {
            let block = blocks[&start_pc];
            builder.switch_to_block(block);

            let mut pc = start_pc;
            let current_block_terminated = false;

            while pc < code_len {
                // Si encontramos otro block start (que no sea el inicio de este), terminamos
                if pc != start_pc && block_starts.contains(&pc) {
                    if !current_block_terminated {
                        builder.ins().jump(blocks[&pc], &[]);
                    }
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
                        // Cargar constante como F64 directamente (Value es f64 en memoria)
                        let val_f64 = builder.ins().load(
                            types::F64,
                            MemFlags::new(),
                            consts_base,
                            offset_const,
                        );
                        builder.def_var(vars[a], val_f64);
                    }
                    OpCode::Mover => {
                        let val = builder.use_var(vars[b]);
                        builder.def_var(vars[a], val);
                    }
                    OpCode::Sumar => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);
                        let res = builder.ins().fadd(val_b, val_c);
                        builder.def_var(vars[a], res);
                    }
                    OpCode::Restar => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);
                        let res = builder.ins().fsub(val_b, val_c);
                        builder.def_var(vars[a], res);
                    }
                    OpCode::Multiplicar => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);
                        let res = builder.ins().fmul(val_b, val_c);
                        builder.def_var(vars[a], res);
                    }
                    OpCode::Dividir => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);
                        let res = builder.ins().fdiv(val_b, val_c);
                        builder.def_var(vars[a], res);
                    }
                    OpCode::Retornar => {
                        let val = builder.use_var(vars[a]);
                        builder.ins().return_(&[val]);
                        // Break inner loop to stop generating code for this block
                        break;
                    }
                    OpCode::Saltar => {
                        let target = pc + bx;
                        let block = blocks[&target];
                        builder.ins().jump(block, &[]);
                        break;
                    }
                    OpCode::SaltarAtras => {
                        let target = pc - bx;
                        let block = blocks[&target];
                        builder.ins().jump(block, &[]);
                        break;
                    }
                    OpCode::SaltarSiFalso => {
                        let val_f64 = builder.use_var(vars[a]);
                        // Bitcast via stack spill
                        builder.ins().stack_store(val_f64, spill_slot, 0);
                        let val = builder.ins().stack_load(types::I64, spill_slot, 0);

                        let target = pc + bx;
                        let target_block = blocks[&target];
                        let next_block = blocks[&pc];

                        let qnan = builder.ins().iconst(types::I64, QNAN as i64);
                        let tag_false = builder.ins().iconst(types::I64, TAG_FALSE as i64);
                        let tag_nulo = builder.ins().iconst(types::I64, TAG_NULO as i64);

                        let val_false = builder.ins().bor(qnan, tag_false);
                        let val_nulo = builder.ins().bor(qnan, tag_nulo);

                        let is_false = builder.ins().icmp(IntCC::Equal, val, val_false);
                        let is_nulo = builder.ins().icmp(IntCC::Equal, val, val_nulo);
                        let cond = builder.ins().bor(is_false, is_nulo);

                        builder.ins().brif(cond, target_block, &[], next_block, &[]);
                        break;
                    }
                    OpCode::Menor => {
                        let val_b = builder.use_var(vars[b]);
                        let val_c = builder.use_var(vars[c]);

                        let cmp = builder.ins().fcmp(FloatCC::LessThan, val_b, val_c);

                        let qnan = builder.ins().iconst(types::I64, QNAN as i64);
                        let tag_true = builder.ins().iconst(types::I64, TAG_TRUE as i64);
                        let tag_false = builder.ins().iconst(types::I64, TAG_FALSE as i64);

                        let val_true = builder.ins().bor(qnan, tag_true);
                        let val_false = builder.ins().bor(qnan, tag_false);

                        let res_i64 = builder.ins().select(cmp, val_true, val_false);
                        // Bitcast via stack spill
                        builder.ins().stack_store(res_i64, spill_slot, 0);
                        let res_f64 = builder.ins().stack_load(types::F64, spill_slot, 0);

                        builder.def_var(vars[a], res_f64);
                    }
                    OpCode::Llamar => {
                        // Llamada recursiva: call self(target_pc, arg0, consts)
                        // target_pc = regs[b] (como usize)
                        // arg0 = regs[b+1] (convención de llamada simplificada para fib)

                        let func_val_f64 = builder.use_var(vars[b]);
                        let func_val_i64 = builder.ins().fcvt_to_uint(pointer_type, func_val_f64);

                        let arg0 = builder.use_var(vars[b + 1]);

                        let call = builder
                            .ins()
                            .call(func_ref, &[func_val_i64, arg0, consts_base]);
                        let ret_val = builder.inst_results(call)[0];

                        builder.def_var(vars[a], ret_val);
                    }
                    _ => {
                        return Err(format!("JIT: OpCode no soportado {:?}", op));
                    }
                }
            }
        }

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
        let ptr = unsafe { std::mem::transmute::<_, fn(usize, f64, *const u64) -> f64>(code) };
        Ok(ptr)
    }
}
