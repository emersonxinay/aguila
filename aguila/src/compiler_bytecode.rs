use crate::ast::{Programa, Sentencia, Expresion};
use crate::vm::chunk::{Chunk, OpCode};
use crate::vm::value::Value;

use std::collections::HashMap;

pub struct Compiler {
    chunk: Chunk,
    reg_count: u8,
    locals: HashMap<String, u8>, // Mapa simple: Nombre -> Registro
    functions: HashMap<String, usize>, // Mapa: Nombre -> Dirección (para recursión)
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            reg_count: 0,
            locals: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn compile(mut self, programa: Programa) -> Chunk {
        for sentencia in programa.sentencias {
            self.compile_statement(sentencia);
        }
        // Emitir retorno final por seguridad
        self.emit(OpCode::Retornar, 0, 0, 0);
        self.chunk
    }

    fn compile_statement(&mut self, stmt: Sentencia) {
        match stmt {
            Sentencia::Imprimir(expr) => {
                let reg = self.compile_expression(expr);
                self.emit(OpCode::Imprimir, reg, 0, 0);
                // No liberamos registros en este MVP simple para simplificar debugging de variables
            }
            Sentencia::Asignacion { nombre, valor, .. } => {
                let reg_val = self.compile_expression(valor);
                if let Some(&reg_existing) = self.locals.get(&nombre) {
                    // Variable existente: Mover valor al registro original
                    self.emit(OpCode::Mover, reg_existing, reg_val, 0);
                } else {
                    // Variable nueva: Registrarla
                    self.locals.insert(nombre, reg_val);
                }
            }
            Sentencia::Expresion(expr) => {
                let _ = self.compile_expression(expr);
            }
            Sentencia::Funcion { nombre, parametros, bloque, .. } => {
                // 1. Saltar el cuerpo de la función
                let jump_over = self.emit_jump(OpCode::Saltar, 0);
                
                // 2. Registrar dirección de inicio
                let func_addr = self.chunk.code.len();
                
                // 3. Registrar en mapa global de funciones para recursión (esto es compile-time)
                self.functions.insert(nombre.clone(), func_addr);

                // 4. Nuevo scope para la función
                let old_locals = self.locals.clone();
                let old_reg_count = self.reg_count;
                
                self.locals.clear();
                self.reg_count = 0; // Reset registros para el nuevo frame

                // Registrar parámetros
                for (param_name, _) in parametros {
                    let reg = self.alloc_register();
                    self.locals.insert(param_name, reg);
                }
                
                // Compilar cuerpo
                for stmt in bloque {
                    self.compile_statement(stmt);
                }
                
                // Asegurar retorno
                self.emit(OpCode::Retornar, 0, 0, 0); // Retorno por defecto (nulo)

                // 5. Restaurar scope
                self.locals = old_locals;
                self.reg_count = old_reg_count;
                
                // 6. Parchear salto
                self.patch_jump(jump_over);
                
                // 7. AHORA inicializamos la variable de la función en el runtime
                // Usamos una constante para la dirección
                let addr_const = self.chunk.add_constant(Value::numero(func_addr as f64));
                let reg_func = self.alloc_register();
                self.chunk.write(Chunk::encode_abx(OpCode::CargarConstante, reg_func, addr_const));
                self.locals.insert(nombre.clone(), reg_func);
            }
            Sentencia::Retorno(expr_opt) => {
                let reg_ret = if let Some(expr) = expr_opt {
                    self.compile_expression(expr)
                } else {
                    let r = self.alloc_register();
                    let c = self.chunk.add_constant(Value::nulo());
                    self.chunk.write(Chunk::encode_abx(OpCode::CargarConstante, r, c));
                    r
                };
                self.emit(OpCode::Retornar, reg_ret, 0, 0);
            }
            Sentencia::Si { condicion, si_bloque, sino_bloque } => {
                self.compile_if(condicion, si_bloque, sino_bloque);
            }
            Sentencia::Mientras { condicion, bloque } => {
                self.compile_while(condicion, bloque);
            }
            _ => panic!("Sentencia no soportada en MVP Bytecode: {:?}", stmt),
        }
    }

    fn compile_expression(&mut self, expr: Expresion) -> u8 {
        match expr {
            Expresion::Numero(n) => {
                let reg = self.alloc_register();
                let const_idx = self.chunk.add_constant(Value::numero(n));
                self.chunk.write(Chunk::encode_abx(OpCode::CargarConstante, reg, const_idx));
                reg
            }
            Expresion::Identificador(nombre) => {
                if let Some(&reg) = self.locals.get(&nombre) {
                    reg
                } else {
                    // Si no es local, tal vez es una función definida anteriormente (hack para recursión simple)
                    // En un sistema real, buscaríamos en scopes superiores.
                    // Por ahora, panic.
                    panic!("Variable no definida: {}", nombre);
                }
            }
            Expresion::Llamada { nombre, args } => {
                let args_len = args.len();
                // 1. Buscar la función (debe ser una variable local con la dirección)
                let reg_func = if let Some(&r) = self.locals.get(&nombre) {
                    r
                } else if let Some(&addr) = self.functions.get(&nombre) {
                    // Función global/recursiva: Cargar dirección
                    let r = self.alloc_register();
                    let c = self.chunk.add_constant(Value::numero(addr as f64));
                    self.chunk.write(Chunk::encode_abx(OpCode::CargarConstante, r, c));
                    r
                } else {
                    panic!("Función no definida: {}", nombre);
                };

                // 2. Compilar argumentos
                // Deben quedar contiguos en memoria después del registro de la función
                // Para simplificar, movemos la función a un nuevo registro base y ponemos los args después.
                
                // Retrocedemos el alloc del 'base' anterior porque la lógica estaba mal
                // self.reg_count -= 1; // Free base (comentado porque ya no usamos esa lógica)
                
                // Calculamos cuántos registros necesitamos: 1 (func) + N (args)
                // Pero compile_expression necesita registros libres para trabajar.
                // Así que compilamos los args a temporales primero, y LUEGO los movemos al bloque de llamada.
                
                let mut arg_regs = Vec::new();
                for arg in &args { // Iterar sobre referencia para no consumir args
                    // arg es &Expresion, pero compile_expression toma Expresion (move)
                    // Necesitamos clonar la expresión.
                    arg_regs.push(self.compile_expression(arg.clone()));
                }
                
                // Ahora reservamos el bloque contiguo
                let call_base = self.alloc_register(); // Para la función
                self.emit(OpCode::Mover, call_base, reg_func, 0);
                
                for reg_arg in arg_regs {
                    let param_slot = self.alloc_register();
                    self.emit(OpCode::Mover, param_slot, reg_arg, 0);
                }
                
                let result_reg = self.alloc_register(); // Donde queremos el resultado
                
                self.emit(OpCode::Llamar, result_reg, call_base, args_len as u8);
                
                // Liberar registros del bloque de llamada (ya no se usan tras el retorno)
                // self.reg_count = call_base; // Unsafe optimization
                
                result_reg
            }
            Expresion::BinOp { izq, op, der } => {
                let r_left = self.compile_expression(*izq);
                let r_right = self.compile_expression(*der);
                let r_dest = self.alloc_register();

                match op.as_str() {
                    "+" => self.emit(OpCode::Sumar, r_dest, r_left, r_right),
                    "-" => self.emit(OpCode::Restar, r_dest, r_left, r_right),
                    "*" => self.emit(OpCode::Multiplicar, r_dest, r_left, r_right),
                    "/" => self.emit(OpCode::Dividir, r_dest, r_left, r_right),
                    "<" => self.emit(OpCode::Menor, r_dest, r_left, r_right),
                    "==" => self.emit(OpCode::Igual, r_dest, r_left, r_right),
                    _ => panic!("Operador no soportado en MVP: {}", op),
                }
                r_dest
            }
            _ => panic!("Expresión no soportada en MVP Bytecode: {:?}", expr),
        }
    }

    fn compile_if(&mut self, condicion: Expresion, si_bloque: Vec<Sentencia>, sino_bloque: Option<Vec<Sentencia>>) {
        let reg_cond = self.compile_expression(condicion);
        
        // Emitir salto si falso (placeholder)
        let jump_if_false = self.emit_jump(OpCode::SaltarSiFalso, reg_cond);

        for stmt in si_bloque {
            self.compile_statement(stmt);
        }

        if let Some(sino) = sino_bloque {
            // Salto incondicional al final del 'si' para saltar el 'sino'
            let jump_end = self.emit_jump(OpCode::Saltar, 0);
            
            // Parchear el salto si falso para que caiga aquí (inicio del sino)
            self.patch_jump(jump_if_false);

            for stmt in sino {
                self.compile_statement(stmt);
            }
            
            // Parchear el salto del final del 'si'
            self.patch_jump(jump_end);
        } else {
            // Si no hay sino, parcheamos el salto si falso para que caiga aquí (fin del si)
            self.patch_jump(jump_if_false);
        }
    }

    fn compile_while(&mut self, condicion: Expresion, bloque: Vec<Sentencia>) {
        let loop_start = self.chunk.code.len();

        let reg_cond = self.compile_expression(condicion);
        let jump_exit = self.emit_jump(OpCode::SaltarSiFalso, reg_cond);

        for stmt in bloque {
            self.compile_statement(stmt);
        }

        // Salto atrás al inicio del bucle
        self.emit_loop(loop_start);

        self.patch_jump(jump_exit);
    }

    // Helpers de registros
    fn alloc_register(&mut self) -> u8 {
        let r = self.reg_count;
        self.reg_count += 1;
        if self.reg_count >= 250 {
            panic!("Overflow de registros (MVP limitado a 250)");
        }
        r
    }

    fn free_register(&mut self, _reg: u8) {
        // En este MVP simple no liberamos agresivamente para evitar colisiones
        // en asignaciones. Un allocator real usaría liveness analysis.
    }

    fn emit(&mut self, op: OpCode, a: u8, b: u8, c: u8) {
        self.chunk.write(Chunk::encode_abc(op, a, b, c));
    }

    fn emit_jump(&mut self, op: OpCode, a: u8) -> usize {
        self.chunk.write(Chunk::encode_abx(op, a, 0xFFFF)); // 0xFFFF placeholder
        self.chunk.code.len() - 1
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code.len() - 1 - offset;
        if jump > 0xFFFF {
            panic!("Salto demasiado grande");
        }
        
        // Reconstruir instrucción con el salto correcto
        let old_inst = self.chunk.code[offset];
        // Extraer OP y A
        let op = (old_inst >> 24) as u8;
        let a = ((old_inst >> 16) & 0xFF) as u8;
        
        self.chunk.code[offset] = Chunk::encode_abx(OpCode::from(op), a, jump as u16);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let offset = self.chunk.code.len() - loop_start + 1; // +1 por la instrucción actual
        if offset > 0xFFFF {
            panic!("Bucle demasiado grande");
        }
        self.chunk.write(Chunk::encode_abx(OpCode::SaltarAtras, 0, offset as u16));
    }
}
