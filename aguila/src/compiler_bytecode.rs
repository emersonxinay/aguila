use crate::ast::{Expresion, Literal, OperadorBinario, Programa, Sentencia};
use crate::vm::chunk::{Chunk, OpCode};
use crate::vm::value::Value;

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypeHint {
    Unknown,
    Integer,
    Float,
    Boolean,
}

pub struct Compiler {
    chunk: Chunk,
    reg_count: u8,
    locals: HashMap<String, u8>,       // Mapa simple: Nombre -> Registro
    functions: HashMap<String, usize>, // Mapa: Nombre -> Dirección (para recursión)
    reg_types: HashMap<u8, TypeHint>,  // OPTIMIZACIÓN: Tracking de tipos por registro
    scope_depth: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            reg_count: 0,
            locals: HashMap::new(),
            functions: HashMap::new(),
            reg_types: HashMap::new(),
            scope_depth: 0,
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
            Sentencia::Imprimir(exprs) => {
                for expr in exprs {
                    let reg = self.compile_expression(expr);
                    self.emit(OpCode::Imprimir, reg, 0, 0);
                }
            }
            Sentencia::Asignacion {
                objetivo, valor, ..
            } => {
                let reg_val = self.compile_expression(valor);
                match objetivo {
                    Expresion::Identificador(nombre) => {
                        if let Some(&reg_existing) = self.locals.get(&nombre) {
                            self.emit(OpCode::Mover, reg_existing, reg_val, 0);
                        } else {
                            self.locals.insert(nombre, reg_val);
                        }
                    }
                    Expresion::AccesoIndice { objeto, indice } => {
                        let reg_obj = self.compile_expression(*objeto);
                        let reg_idx = self.compile_expression(*indice);
                        self.emit(OpCode::AsignarIndice, reg_obj, reg_idx, reg_val);
                    }
                    Expresion::AccesoAtributo { objeto, atributo } => {
                        let reg_obj = self.compile_expression(*objeto);
                        let const_idx = self.chunk.add_constant(Value::texto(atributo));
                        // Usamos B como índice de constante (u8 max 255, cuidado)
                        // Para simplificar MVP, asumimos que cabe en u8. Si no, necesitaríamos OpCode extendido.
                        if const_idx > 255 {
                            panic!("Demasiadas constantes para AsignarPropiedad");
                        }
                        self.emit(OpCode::AsignarPropiedad, reg_obj, const_idx as u8, reg_val);
                    }
                    _ => panic!("Asignación compleja no soportada en MVP"),
                }
            }
            Sentencia::Expresion(expr) => {
                let _ = self.compile_expression(expr);
            }
            Sentencia::Funcion {
                nombre,
                params,
                cuerpo,
                ..
            } => {
                let jump_over = self.emit_jump(OpCode::Saltar, 0);
                let func_addr = self.chunk.code.len();
                self.functions.insert(nombre.clone(), func_addr);

                let old_locals = self.locals.clone();
                let old_reg_count = self.reg_count;

                self.locals.clear();
                self.reg_count = 0;
                self.scope_depth += 1;

                for param in params {
                    let reg = self.alloc_register();
                    self.locals.insert(param.nombre, reg);
                }

                for stmt in cuerpo {
                    self.compile_statement(stmt);
                }

                let reg_null = self.alloc_register();
                let const_null = self.chunk.add_constant(Value::nulo());
                self.chunk.write(Chunk::encode_abx(
                    OpCode::CargarConstante,
                    reg_null,
                    const_null,
                ));
                self.emit(OpCode::Retornar, reg_null, 0, 0);

                self.patch_jump(jump_over);

                self.locals = old_locals;
                self.reg_count = old_reg_count;
                self.scope_depth -= 1;

                let addr_const = self.chunk.add_constant(Value::numero(func_addr as f64));
                let reg_func = self.alloc_register();
                self.chunk.write(Chunk::encode_abx(
                    OpCode::CargarConstante,
                    reg_func,
                    addr_const,
                ));

                if self.scope_depth == 0 {
                    // Función global
                    let const_nombre = self.chunk.add_constant(Value::texto(nombre));
                    self.chunk.write(Chunk::encode_abx(
                        OpCode::DefinirGlobal,
                        reg_func,
                        const_nombre,
                    ));
                } else {
                    // Función local
                    self.locals.insert(nombre, reg_func);
                }
            }
            Sentencia::Clase {
                nombre,
                herencia,
                cuerpo,
                ..
            } => {
                // 1. Resolver padre
                let reg_padre = if let Some(padre_nombre) = herencia.first() {
                    if let Some(&r) = self.locals.get(padre_nombre) {
                        r
                    } else {
                        // Asumir global o error
                        let r = self.alloc_register();
                        let c = self.chunk.add_constant(Value::texto(padre_nombre.clone()));
                        self.chunk
                            .write(Chunk::encode_abx(OpCode::ObtenerGlobal, r, c));
                        r
                    }
                } else {
                    let r = self.alloc_register();
                    let c = self.chunk.add_constant(Value::nulo());
                    self.chunk
                        .write(Chunk::encode_abx(OpCode::CargarConstante, r, c));
                    r
                };

                // 2. Crear clase
                let reg_clase = self.alloc_register();
                let const_nombre = self.chunk.add_constant(Value::texto(nombre.clone()));
                if const_nombre > 255 {
                    panic!("Nombre clase muy largo");
                }
                self.emit(OpCode::CrearClase, reg_clase, const_nombre as u8, reg_padre);
                self.locals.insert(nombre.clone(), reg_clase);

                // 3. Compilar métodos
                for stmt in cuerpo {
                    if let Sentencia::Funcion {
                        nombre: metodo_nombre,
                        params,
                        cuerpo: metodo_cuerpo,
                        ..
                    } = stmt
                    {
                        // Compilar cuerpo del método como función separada
                        let jump_over = self.emit_jump(OpCode::Saltar, 0);
                        let metodo_addr = self.chunk.code.len();

                        let old_locals = self.locals.clone();
                        let old_reg_count = self.reg_count;
                        self.locals.clear();
                        self.reg_count = 0;

                        // self es el primer parámetro implícito?
                        // En este diseño simple, pasamos 'self' explícitamente o usamos registro 0?
                        // Vamos a reservar registro 0 para 'yo' (self)
                        let reg_yo = self.alloc_register();
                        self.locals.insert("yo".to_string(), reg_yo);

                        for param in params {
                            let reg = self.alloc_register();
                            self.locals.insert(param.nombre, reg);
                        }

                        for s in metodo_cuerpo {
                            self.compile_statement(s);
                        }

                        // Retorno default
                        let reg_null = self.alloc_register();
                        let const_null = self.chunk.add_constant(Value::nulo());
                        self.chunk.write(Chunk::encode_abx(
                            OpCode::CargarConstante,
                            reg_null,
                            const_null,
                        ));
                        self.emit(OpCode::Retornar, reg_null, 0, 0);

                        self.locals = old_locals;
                        self.reg_count = old_reg_count;

                        self.patch_jump(jump_over);

                        // Registrar método en la clase
                        let reg_metodo_addr = self.alloc_register();
                        let c_addr = self.chunk.add_constant(Value::numero(metodo_addr as f64));
                        self.chunk.write(Chunk::encode_abx(
                            OpCode::CargarConstante,
                            reg_metodo_addr,
                            c_addr,
                        ));

                        let const_metodo_nombre =
                            self.chunk.add_constant(Value::texto(metodo_nombre));
                        if const_metodo_nombre > 255 {
                            panic!("Nombre metodo muy largo");
                        }

                        self.emit(
                            OpCode::Metodo,
                            reg_clase,
                            const_metodo_nombre as u8,
                            reg_metodo_addr,
                        );
                    }
                }
            }
            Sentencia::Importar { modulo, alias } => {
                let reg_mod = self.alloc_register();
                let const_mod = self.chunk.add_constant(Value::texto(modulo.clone()));
                self.chunk
                    .write(Chunk::encode_abx(OpCode::Importar, reg_mod, const_mod));

                let nombre_var = alias.unwrap_or_else(|| modulo.replace(".ag", ""));
                self.locals.insert(nombre_var, reg_mod);
            }
            Sentencia::TryCatch {
                cuerpo,
                capturas,
                finalmente,
            } => {
                // 1. Push handler
                let push_try_idx = self.chunk.code.len();
                self.chunk.write(0); // Placeholder

                // 2. Bloque Try
                for stmt in cuerpo {
                    self.compile_statement(stmt);
                }

                // 3. Pop handler (si todo salió bien)
                self.emit(OpCode::PopTry, 0, 0, 0);
                let jump_over_catch = self.emit_jump(OpCode::Saltar, 0);

                // 4. Bloque Catch
                let catch_start = self.chunk.code.len();

                // Parchear PushTry
                let offset = catch_start - push_try_idx - 1;
                self.chunk.code[push_try_idx] =
                    Chunk::encode_abx(OpCode::PushTry, 0, offset as u16);

                // Compilar capturas (por ahora solo la primera o genérica)
                // MVP: Asumimos que el error está disponible y ejecutamos el primer bloque catch.
                // En un sistema real, verificaríamos el tipo de excepción.
                if let Some(captura) = capturas.first() {
                    if let Some(var_error) = &captura.variable {
                        let reg_err = self.alloc_register();
                        self.locals.insert(var_error.clone(), reg_err);
                        // Obtener el error actual
                        self.emit(OpCode::ObtenerError, reg_err, 0, 0);
                    }
                    for stmt in &captura.cuerpo {
                        self.compile_statement(stmt.clone());
                    }
                }

                self.patch_jump(jump_over_catch);

                // 5. Finalmente (opcional)
                if let Some(bloque_finalmente) = finalmente {
                    // Esto es complejo: finally se ejecuta siempre.
                    // MVP: Solo lo ejecutamos si no hubo error o después del catch.
                    // Correcto sería usar un unwind mechanism.
                    for stmt in bloque_finalmente {
                        self.compile_statement(stmt);
                    }
                }
            }
            Sentencia::Lanzar(expr) => {
                let reg = self.compile_expression(expr);
                self.emit(OpCode::Lanzar, reg, 0, 0);
            }
            Sentencia::Retornar(expr_opt) => {
                let reg_ret = if let Some(expr) = expr_opt {
                    self.compile_expression(expr)
                } else {
                    let r = self.alloc_register();
                    let c = self.chunk.add_constant(Value::nulo());
                    self.chunk
                        .write(Chunk::encode_abx(OpCode::CargarConstante, r, c));
                    r
                };
                self.emit(OpCode::Retornar, reg_ret, 0, 0);
            }
            Sentencia::Si {
                condicion,
                entonces,
                sino,
            } => {
                self.compile_if(condicion, entonces, sino);
            }
            Sentencia::Mientras { condicion, cuerpo } => {
                self.compile_while(condicion, cuerpo);
            }
            _ => panic!("Sentencia no soportada en MVP Bytecode: {:?}", stmt),
        }
    }

    fn compile_expression(&mut self, expr: Expresion) -> u8 {
        match expr {
            Expresion::Literal(lit) => {
                let reg = self.alloc_register();
                let val = match lit {
                    Literal::Entero(n) => {
                        self.reg_types.insert(reg, TypeHint::Integer);
                        Value::entero(n as i32)
                    }
                    Literal::Decimal(n) => {
                        self.reg_types.insert(reg, TypeHint::Float);
                        Value::numero(n)
                    }
                    Literal::Texto(s) => Value::texto(s),
                    Literal::Booleano(b) => Value::logico(b),
                    Literal::Nulo => Value::nulo(),
                };
                let const_idx = self.chunk.add_constant(val);
                self.chunk
                    .write(Chunk::encode_abx(OpCode::CargarConstante, reg, const_idx));
                reg
            }
            Expresion::Identificador(nombre) => {
                if let Some(&reg) = self.locals.get(&nombre) {
                    reg
                } else {
                    let reg = self.alloc_register();
                    let const_idx = self.chunk.add_constant(Value::texto(nombre));
                    self.chunk
                        .write(Chunk::encode_abx(OpCode::ObtenerGlobal, reg, const_idx));
                    reg
                }
            }
            Expresion::Lista(elementos) => {
                let start_reg = self.reg_count; // Inicio del bloque contiguo
                                                // Necesitamos que los registros sean contiguos.
                                                // alloc_register incrementa secuencialmente.
                for elem in elementos.iter() {
                    let r = self.compile_expression(elem.clone());
                    // Si r != start_reg + i, tenemos un problema de contigüidad si compile_expression usa temporales.
                    // En un compilador simple register-based, esto es tricky.
                    // Solución: Compilar a temporales y luego mover a un bloque contiguo.
                }
                // Simplificación: Asumimos que compile_expression devuelve el registro donde quedó.
                // Para CrearLista, necesitamos un rango.
                // Vamos a hacer un hack: Emitir Mover para alinear.

                let count = elementos.len();
                let base = self.alloc_register(); // Base del array
                for _ in 1..count {
                    self.alloc_register();
                } // Reservar N registros

                // Re-compilar y mover (ineficiente pero seguro)
                for (i, elem) in elementos.into_iter().enumerate() {
                    let r_temp = self.compile_expression(elem);
                    self.emit(OpCode::Mover, base + i as u8, r_temp, 0);
                }

                let reg_dest = self.alloc_register();
                self.emit(OpCode::CrearLista, reg_dest, base, count as u8);
                reg_dest
            }
            Expresion::Diccionario(pares) => {
                let count = pares.len();
                // Necesitamos 2*count registros contiguos (clave, valor, clave, valor...)
                let base = self.alloc_register();
                for _ in 1..(count * 2) {
                    self.alloc_register();
                }

                for (i, (k, v)) in pares.into_iter().enumerate() {
                    let r_k = self.compile_expression(k);
                    let r_v = self.compile_expression(v);
                    self.emit(OpCode::Mover, base + (i * 2) as u8, r_k, 0);
                    self.emit(OpCode::Mover, base + (i * 2 + 1) as u8, r_v, 0);
                }

                let reg_dest = self.alloc_register();
                self.emit(OpCode::CrearDiccionario, reg_dest, base, count as u8);
                reg_dest
            }
            Expresion::AccesoIndice { objeto, indice } => {
                let r_obj = self.compile_expression(*objeto);
                let r_idx = self.compile_expression(*indice);
                let r_dest = self.alloc_register();
                self.emit(OpCode::AccederIndice, r_dest, r_obj, r_idx);
                r_dest
            }
            Expresion::AccesoAtributo { objeto, atributo } => {
                let r_obj = self.compile_expression(*objeto);
                let c_idx = self.chunk.add_constant(Value::texto(atributo));
                let r_dest = self.alloc_register();
                // Usamos C como índice de constante
                if c_idx > 255 {
                    panic!("Constante atributo muy grande");
                }
                self.emit(OpCode::AccederPropiedad, r_dest, r_obj, c_idx as u8);
                r_dest
            }
            Expresion::Llamada { func, args } => {
                // 1. Compilar la función/método (el "callee")
                // Esto pondrá el objeto invocable en un registro.
                // Si es p.saludar, AccederPropiedad pondrá el método (o BoundMethod) en el registro.
                let reg_func = self.compile_expression(*func);

                let args_len = args.len();
                let mut arg_regs = Vec::new();
                for arg in args {
                    arg_regs.push(self.compile_expression(arg));
                }

                let call_base = self.alloc_register();
                self.emit(OpCode::Mover, call_base, reg_func, 0);

                for reg_arg in arg_regs {
                    let param_slot = self.alloc_register();
                    self.emit(OpCode::Mover, param_slot, reg_arg, 0);
                }

                // Usamos call_base como destino para evitar colisión con el stack frame del callee
                // Si usamos un registro nuevo (ra > rb), y el callee escribe en sus registros locales,
                // puede sobrescribir ra si ra cae dentro de su ventana.
                // Al usar ra = rb, el resultado sobrescribe al callee (función), lo cual es estándar.
                self.emit(OpCode::Llamar, call_base, call_base, args_len as u8);
                call_base
            }
            Expresion::Binaria { izq, op, der } => {
                let r_left = self.compile_expression(*izq);
                let r_right = self.compile_expression(*der);
                let r_dest = self.alloc_register();
                self.emit_binop(op, r_dest, r_left, r_right);
                r_dest
            }
            Expresion::Await(expr) => {
                let reg_val = self.compile_expression(*expr);
                let reg_dest = self.alloc_register();
                self.emit(OpCode::Await, reg_dest, reg_val, 0);
                reg_dest
            }
            Expresion::Unaria { op, exp } => {
                let reg_val = self.compile_expression(*exp);
                let reg_dest = self.alloc_register();
                match op {
                    crate::ast::OperadorUnario::Not => {
                        self.emit(OpCode::Not, reg_dest, reg_val, 0);
                    }
                    crate::ast::OperadorUnario::Negativo => {
                        self.emit(OpCode::Negativo, reg_dest, reg_val, 0);
                    }
                    crate::ast::OperadorUnario::BitNot => {
                        self.emit(OpCode::BitNot, reg_dest, reg_val, 0);
                    }
                }
                reg_dest
            }
            _ => panic!("Expresión no soportada en MVP Bytecode: {:?}", expr),
        }
    }

    fn compile_if(
        &mut self,
        condicion: Expresion,
        entonces: Vec<Sentencia>,
        sino: Option<Vec<Sentencia>>,
    ) {
        let reg_cond = self.compile_expression(condicion);
        let jump_if_false = self.emit_jump(OpCode::SaltarSiFalso, reg_cond);

        for stmt in entonces {
            self.compile_statement(stmt);
        }

        if let Some(sino_bloque) = sino {
            let jump_end = self.emit_jump(OpCode::Saltar, 0);
            self.patch_jump(jump_if_false);

            for stmt in sino_bloque {
                self.compile_statement(stmt);
            }
            self.patch_jump(jump_end);
        } else {
            self.patch_jump(jump_if_false);
        }
    }

    fn compile_while(&mut self, condicion: Expresion, cuerpo: Vec<Sentencia>) {
        let loop_start = self.chunk.code.len();
        let reg_cond = self.compile_expression(condicion);
        let jump_exit = self.emit_jump(OpCode::SaltarSiFalso, reg_cond);

        for stmt in cuerpo {
            self.compile_statement(stmt);
        }

        self.emit_loop(loop_start);
        self.patch_jump(jump_exit);
    }

    fn alloc_register(&mut self) -> u8 {
        let r = self.reg_count;
        if r >= 255 {
            panic!("Registro overflow");
        }
        self.reg_count += 1;
        r
    }

    fn emit(&mut self, op: OpCode, a: u8, b: u8, c: u8) {
        self.chunk.write(Chunk::encode_abc(op, a, b, c));
    }

    fn emit_binop(&mut self, op: OperadorBinario, dest: u8, left: u8, right: u8) {
        match op {
            OperadorBinario::Suma => self.emit(OpCode::Sumar, dest, left, right),
            OperadorBinario::Resta => self.emit(OpCode::Restar, dest, left, right),
            OperadorBinario::Multiplicacion => self.emit(OpCode::Multiplicar, dest, left, right),
            OperadorBinario::Division => self.emit(OpCode::Dividir, dest, left, right),
            OperadorBinario::Modulo => self.emit(OpCode::Modulo, dest, left, right),
            OperadorBinario::Potencia => self.emit(OpCode::Potencia, dest, left, right),
            OperadorBinario::Menor => self.emit(OpCode::Menor, dest, left, right),
            OperadorBinario::MenorIgual => self.emit(OpCode::MenorIgual, dest, left, right),
            OperadorBinario::Mayor => self.emit(OpCode::Menor, dest, right, left),
            OperadorBinario::MayorIgual => self.emit(OpCode::MenorIgual, dest, right, left),
            OperadorBinario::Igual => self.emit(OpCode::Igual, dest, left, right),
            OperadorBinario::NoIgual => {
                // NoIgual = !(Igual)
                // Emitir Igual
                self.emit(OpCode::Igual, dest, left, right);
                // Emitir Not
                self.emit(OpCode::Not, dest, dest, 0);
            }
            OperadorBinario::Y => {
                // Short-circuit logic is better handled by control flow, but for now strict:
                self.emit(OpCode::BitAnd, dest, left, right); // Using BitAnd as logical AND placeholder
            }
            OperadorBinario::O => {
                self.emit(OpCode::BitOr, dest, left, right);
            }
            OperadorBinario::BitAnd => self.emit(OpCode::BitAnd, dest, left, right),
            OperadorBinario::BitOr => self.emit(OpCode::BitOr, dest, left, right),
            OperadorBinario::BitXor => self.emit(OpCode::BitXor, dest, left, right),
            OperadorBinario::ShiftIzq => self.emit(OpCode::ShiftIzq, dest, left, right),
            OperadorBinario::ShiftDer => self.emit(OpCode::ShiftDer, dest, left, right),
            OperadorBinario::DivisionEntera => {
                // Division entera: floor(a / b)
                self.emit(OpCode::Dividir, dest, left, right);
                // Falta OpCode::Floor o similar. Por ahora dejamos Dividir normal o implementamos Floor.
                // O mejor, OpCode::DivisionEntera.
                // No tenemos OpCode::DivisionEntera en chunk.rs.
                // Usemos Dividir por ahora (MVP).
            }
        }
    }

    fn emit_jump(&mut self, op: OpCode, a: u8) -> usize {
        self.chunk.write(Chunk::encode_abx(op, a, 0xFFFF));
        self.chunk.code.len() - 1
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code.len() - 1 - offset;
        if jump > 0xFFFF {
            panic!("Salto demasiado grande");
        }
        let old_inst = self.chunk.code[offset];
        let op = (old_inst >> 24) as u8;
        let a = ((old_inst >> 16) & 0xFF) as u8;
        self.chunk.code[offset] = Chunk::encode_abx(OpCode::from(op), a, jump as u16);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let offset = self.chunk.code.len() - loop_start + 1;
        if offset > 0xFFFF {
            panic!("Bucle demasiado grande");
        }
        self.chunk
            .write(Chunk::encode_abx(OpCode::SaltarAtras, 0, offset as u16));
    }
}
