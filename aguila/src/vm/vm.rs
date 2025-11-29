use crate::vm::chunk::{Chunk, OpCode};
use crate::vm::value::{Value, QNAN, TAG_NULO, TAG_FALSE, TAG_TRUE};
use crate::vm::jit::Jit;

const MAX_FRAMES: usize = 2048;
const MAX_REGISTERS: usize = 4096; // Aumentamos registros para soportar recursión profunda

#[derive(Clone, Copy, Debug)]
pub struct CallFrame {
    pub pc: usize,
    pub base_pointer: usize,
    pub return_reg: usize,
}

pub struct VM {
    pub registers: [Value; MAX_REGISTERS],
    pub frames: [CallFrame; MAX_FRAMES],
    pub frame_count: usize,
    pub pc: usize,
    pub base_pointer: usize,
    pub jit: Jit,
}

impl VM {
    pub fn new() -> Self {
        // Inicializar array de frames con valores dummy (unsafe para velocidad o loop)
        // Como CallFrame es Copy (si derivamos Copy) o simple, podemos usar inicialización segura iterativa
        // O `[const_val; N]` si implementamos Copy.
        // Por ahora, inicializamos con un loop o vector convertido (pero queremos array estático).
        // Rust requiere inicialización.
        // Hack: Usar MaybeUninit es lo más rápido, pero unsafe.
        // Por seguridad en este paso, usaremos un valor por defecto.
        
        let dummy_frame = CallFrame { pc: 0, base_pointer: 0, return_reg: 0 };
        
        Self {
            registers: [Value::nulo(); MAX_REGISTERS],
            frames: [dummy_frame; MAX_FRAMES],
            frame_count: 0,
            pc: 0,
            base_pointer: 0,
            jit: Jit::new(),
        }
    }

    pub fn run(&mut self, chunk: &Chunk) -> Result<(), String> {
        // Inicialización de estado local para velocidad (Register Caching)
        let mut pc = 0;
        let mut bp = 0;
        let mut frame_count = 0;
        
        // Punteros crudos para acceso directo
        let regs_ptr = self.registers.as_mut_ptr();
        let frames_ptr = self.frames.as_mut_ptr();
        let code_ptr = chunk.code.as_ptr();
        let code_len = chunk.code.len();
        let constants_ptr = chunk.constants.as_ptr();

        // Frame inicial
        unsafe {
            *frames_ptr.add(frame_count) = CallFrame {
                pc: 0,
                base_pointer: 0,
                return_reg: 0,
            };
        }
        frame_count += 1;

        while pc < code_len {
            unsafe {
                let instruction = *code_ptr.add(pc);
                let op_code = (instruction >> 24) as u8;
                pc += 1;

                let op: OpCode = std::mem::transmute(op_code);
                let a = ((instruction >> 16) & 0xFF) as usize;
                let b = ((instruction >> 8) & 0xFF) as usize;
                let c = (instruction & 0xFF) as usize;
                let bx = (instruction & 0xFFFF) as usize;

                match op {
                    OpCode::CargarConstante => {
                        *regs_ptr.add(bp + a) = *constants_ptr.add(bx);
                    }
                    OpCode::Mover => {
                        let val = *regs_ptr.add(bp + b);
                        *regs_ptr.add(bp + a) = val;
                    }
                    OpCode::Sumar => {
                        let val_b = *regs_ptr.add(bp + b);
                        let val_c = *regs_ptr.add(bp + c);
                        // Inlining de suma numérica para evitar llamadas a Value::numero
                        if (val_b.0 & QNAN) != QNAN && (val_c.0 & QNAN) != QNAN {
                             let res = f64::from_bits(val_b.0) + f64::from_bits(val_c.0);
                             *regs_ptr.add(bp + a) = Value::numero(res);
                        } else {
                            return Err("Operandos deben ser números".to_string());
                        }
                    }
                    OpCode::Restar => {
                        let val_b = *regs_ptr.add(bp + b);
                        let val_c = *regs_ptr.add(bp + c);
                        if (val_b.0 & QNAN) != QNAN && (val_c.0 & QNAN) != QNAN {
                             let res = f64::from_bits(val_b.0) - f64::from_bits(val_c.0);
                             *regs_ptr.add(bp + a) = Value::numero(res);
                        } else {
                            return Err("Operandos deben ser números".to_string());
                        }
                    }
                    OpCode::Multiplicar => {
                        let val_b = *regs_ptr.add(bp + b);
                        let val_c = *regs_ptr.add(bp + c);
                        if (val_b.0 & QNAN) != QNAN && (val_c.0 & QNAN) != QNAN {
                             let res = f64::from_bits(val_b.0) * f64::from_bits(val_c.0);
                             *regs_ptr.add(bp + a) = Value::numero(res);
                        } else {
                            return Err("Operandos deben ser números".to_string());
                        }
                    }
                    OpCode::Dividir => {
                        let val_b = *regs_ptr.add(bp + b);
                        let val_c = *regs_ptr.add(bp + c);
                        if (val_b.0 & QNAN) != QNAN && (val_c.0 & QNAN) != QNAN {
                             let res = f64::from_bits(val_b.0) / f64::from_bits(val_c.0);
                             *regs_ptr.add(bp + a) = Value::numero(res);
                        } else {
                            return Err("Operandos deben ser números".to_string());
                        }
                    }
                    OpCode::Multiplicar => {
                        let val_b = *regs_ptr.add(bp + b);
                        let val_c = *regs_ptr.add(bp + c);
                        if (val_b.0 & QNAN) != QNAN && (val_c.0 & QNAN) != QNAN {
                             let res = f64::from_bits(val_b.0) * f64::from_bits(val_c.0);
                             *regs_ptr.add(bp + a) = Value::numero(res);
                        } else {
                            return Err("Operandos deben ser números".to_string());
                        }
                    }
                    OpCode::Dividir => {
                        let val_b = *regs_ptr.add(bp + b);
                        let val_c = *regs_ptr.add(bp + c);
                        if (val_b.0 & QNAN) != QNAN && (val_c.0 & QNAN) != QNAN {
                             let res = f64::from_bits(val_b.0) / f64::from_bits(val_c.0);
                             *regs_ptr.add(bp + a) = Value::numero(res);
                        } else {
                            return Err("Operandos deben ser números".to_string());
                        }
                    }
                    OpCode::Retornar => {
                        frame_count -= 1;
                        if frame_count == 0 {
                            // Sincronizar estado antes de salir (opcional si solo salimos)
                            self.pc = pc;
                            self.base_pointer = bp;
                            self.frame_count = frame_count;
                            return Ok(());
                        }
                        
                        let frame = *frames_ptr.add(frame_count);
                        pc = frame.pc;
                        
                        let ret_val = *regs_ptr.add(bp + a);
                        
                        let prev_frame = *frames_ptr.add(frame_count - 1);
                        bp = prev_frame.base_pointer;
                        
                        *regs_ptr.add(bp + frame.return_reg) = ret_val;
                    }
                    OpCode::Imprimir => {
                        let val = *regs_ptr.add(bp + a);
                        if val.es_numero() {
                            println!("{}", val.a_numero());
                        } else if val.es_logico() {
                            println!("{}", val.a_logico());
                        } else if val.es_nulo() {
                            println!("nulo");
                        } else {
                            println!("{:?}", val);
                        }
                    }
                    OpCode::Menor => {
                        let val_b = *regs_ptr.add(bp + b);
                        let val_c = *regs_ptr.add(bp + c);
                        if (val_b.0 & QNAN) != QNAN && (val_c.0 & QNAN) != QNAN {
                            let res = f64::from_bits(val_b.0) < f64::from_bits(val_c.0);
                            *regs_ptr.add(bp + a) = Value::logico(res);
                        } else {
                            return Err("Operandos de comparación deben ser números".to_string());
                        }
                    }
                    OpCode::Igual => {
                        let val_b = *regs_ptr.add(bp + b);
                        let val_c = *regs_ptr.add(bp + c);
                        *regs_ptr.add(bp + a) = Value::logico(val_b == val_c);
                    }
                    OpCode::Saltar => {
                        pc += bx;
                    }
                    OpCode::SaltarSiFalso => {
                        let val = *regs_ptr.add(bp + a);
                        // Inlining de es_falso
                        let es_falso = val.0 == (QNAN | TAG_FALSE) || val.0 == (QNAN | TAG_NULO);
                        if es_falso {
                            pc += bx;
                        }
                    }
                    OpCode::SaltarAtras => {
                        pc -= bx;
                    }
                    OpCode::Llamar => {
                        let func_addr_val = *regs_ptr.add(bp + b);
                        // Check rápido de número
                        if (func_addr_val.0 & QNAN) == QNAN {
                             return Err(format!("Intentando llamar a algo que no es una función (dirección): {:?} en registro BP+{}", func_addr_val, b));
                        }
                        let func_addr = f64::from_bits(func_addr_val.0) as usize;
                        
                        // Intento de JIT (Method JIT)
                        // Si estamos llamando a una función, intentamos compilar el chunk (si no lo está ya)
                        // y ejecutarlo desde func_addr.
                        // Hack: Solo si es una llamada recursiva o intensiva?
                        // Por ahora, siempre intentamos si el JIT está habilitado.
                        // Nota: compile() compila todo el chunk.
                        // Debemos evitar recompilar si ya tenemos el código.
                        // Como no tenemos caché aún, compilamos siempre (lento) o asumimos que compile es rápido/cacheado internamente?
                        // No es cacheado.
                        // Pero para fib(30), la llamada inicial es UNA vez.
                        // Las llamadas recursivas ocurren DENTRO del JIT (call self).
                        // Así que esto solo se ejecuta una vez para la llamada raíz.
                        
                        // Solo intentamos JIT si NO estamos ya en JIT (obvio, estamos en interpreter)
                        // y si la función parece ser candidata (e.g. fib).
                        
                        // Hack: Probamos compilar. Si falla, seguimos interpretando.
                        // Pasamos el chunk actual.
                        // Necesitamos acceso mutable a JIT.
                        // self.jit.compile requiere &mut self.jit.
                        // self.registers requiere &self.registers.
                        // Split borrows.
                        // Unsafe para evitar conflictos de borrow checker en el loop.
                        let jit_ptr = &mut self.jit as *mut Jit;
                        let chunk_ref = &*chunk; // Chunk es referencia en run
                        
                        // Solo compilamos si es la primera vez? O siempre?
                        // Compile devuelve Result.
                        // JIT Compilation Attempt (Solo para funciones de 1 argumento por ahora)
                        // El JIT actual tiene una firma fija fn(pc, arg0, consts)
                        let arg_count = c;
                        if arg_count == 1 {
                            match unsafe { (*jit_ptr).compile(chunk_ref, func_addr) } {
                                Ok(jit_fn) => {
                                    let arg0_val = *regs_ptr.add(bp + b + 1); // Argumento 0 (n)
                                    let arg0 = if arg0_val.es_numero() { arg0_val.a_numero() } else { 0.0 };
                                    
                                    let res = jit_fn(func_addr, arg0, constants_ptr as *const u64);
                                    
                                    *regs_ptr.add(bp + a) = Value::numero(res);
                                    continue;
                                }
                                Err(e) => {
                                    println!("JIT compilation failed for chunk at {:?}: {}", func_addr, e);
                                }
                            }
                        }

                        if frame_count > 0 {
                            (*frames_ptr.add(frame_count - 1)).pc = pc;
                        }

                        let new_base = bp + b + 1;
                        
                        if frame_count >= MAX_FRAMES {
                            return Err("Stack overflow (demasiada recursión)".to_string());
                        }

                        *frames_ptr.add(frame_count) = CallFrame {
                            pc: pc,
                            base_pointer: new_base,
                            return_reg: a,
                        };
                        frame_count += 1;
                        
                        bp = new_base;
                        pc = func_addr;
                    }
                }
            }
        }

        // Sincronizar estado al salir (si sale por fin de código, aunque Retornar maneja la salida normal)
        self.pc = pc;
        self.base_pointer = bp;
        self.frame_count = frame_count;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::chunk::OpCode;

    #[test]
    fn test_suma_simple() {
        let mut chunk = Chunk::new();
        let mut vm = VM::new();

        // Constantes: 10, 20
        let c1 = chunk.add_constant(Value::numero(10.0));
        let c2 = chunk.add_constant(Value::numero(20.0));

        // R0 = 10
        chunk.write(Chunk::encode_abx(OpCode::CargarConstante, 0, c1));
        // R1 = 20
        chunk.write(Chunk::encode_abx(OpCode::CargarConstante, 1, c2));
        // R2 = R0 + R1
        chunk.write(Chunk::encode_abc(OpCode::Sumar, 2, 0, 1));
        // Imprimir R2
        chunk.write(Chunk::encode_abc(OpCode::Imprimir, 2, 0, 0));
        // Retornar
        chunk.write(Chunk::encode_abc(OpCode::Retornar, 0, 0, 0));

        let result = vm.run(&chunk);
        assert!(result.is_ok());
        
        // Verificar resultado en registro 2
        assert_eq!(vm.registers[2].a_numero(), 30.0);
    }
}
