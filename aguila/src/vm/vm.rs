use crate::vm::chunk::{Chunk, OpCode};
use crate::vm::jit::Jit;
use crate::vm::object_table::ObjectTable;
use crate::vm::value::{Clase, Instancia, Obj, Value, QNAN, TAG_FALSE, TAG_NULO, TAG_TRUE};
use std::io::Write;

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

const MAX_FRAMES: usize = 2048;
const MAX_REGISTERS: usize = 4096;

#[derive(Clone, Copy, Debug)]
pub struct CallFrame {
    pub pc: usize,
    pub base_pointer: usize,
    pub return_reg: usize,
}

pub type NativeFunc = fn(&mut VM, &[Value]) -> Result<Value, String>;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub exports: HashMap<String, Value>,
}

pub struct VM {
    pub registers: [Value; MAX_REGISTERS],
    pub frames: [CallFrame; MAX_FRAMES],
    pub frame_count: usize,
    pub pc: usize,
    pub base_pointer: usize,
    pub jit: Jit,
    pub jit_cache: HashMap<usize, usize>,
    #[allow(dead_code)]
    pub jit_fast_cache: HashMap<usize, usize>,
    pub hotspot_counters: HashMap<usize, u32>,
    pub natives: Vec<NativeFunc>,
    pub modules: Vec<Module>,
    pub last_error: Option<Value>,
    pub resources: Vec<Option<Box<dyn Any + Send>>>,
    pub free_resources: Vec<usize>,
    pub resource_methods: HashMap<String, Value>,
    pub globals: HashMap<String, Value>,
    pub chunk: Option<Arc<Chunk>>,
    pub try_stack: Vec<usize>, // Stack de handlers de excepción (PC del catch)
    pub object_table: ObjectTable, // NEW: Optimized object storage
}

impl VM {
    pub fn new() -> Self {
        let frames = [CallFrame {
            pc: 0,
            base_pointer: 0,
            return_reg: 0,
        }; MAX_FRAMES];
        let registers = [Value::nulo(); MAX_REGISTERS];

        let mut vm = VM {
            registers,
            frames,
            frame_count: 0,
            pc: 0,
            base_pointer: 0,
            jit: Jit::new(),
            jit_cache: HashMap::new(),
            jit_fast_cache: HashMap::new(),
            hotspot_counters: HashMap::new(),
            natives: Vec::new(),
            modules: Vec::new(),
            last_error: None,
            resources: Vec::new(),
            free_resources: Vec::new(),
            resource_methods: HashMap::new(),
            globals: HashMap::new(),
            chunk: None,
            try_stack: Vec::new(),
            object_table: ObjectTable::new(), // Initialize optimized object storage
        };

        let reloj_idx = vm.registrar_nativa(|_, _| {
            let now = std::time::SystemTime::now();
            let since_the_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap();
            Ok(Value::numero(since_the_epoch.as_secs_f64()))
        });
        vm.globals
            .insert("reloj".to_string(), Value::nativa(reloj_idx as u32));

        vm
    }

    pub fn registrar_nativa(&mut self, f: NativeFunc) -> usize {
        self.natives.push(f);
        self.natives.len() - 1
    }

    pub fn alloc_resource(&mut self, res: Box<dyn Any + Send>) -> u32 {
        if let Some(idx) = self.free_resources.pop() {
            self.resources[idx] = Some(res);
            idx as u32
        } else {
            self.resources.push(Some(res));
            (self.resources.len() - 1) as u32
        }
    }

    pub fn get_resource<T: Any>(&self, handle: u32) -> Option<&T> {
        if let Some(Some(res)) = self.resources.get(handle as usize) {
            res.downcast_ref::<T>()
        } else {
            None
        }
    }

    pub fn get_resource_mut<T: Any>(&mut self, handle: u32) -> Option<&mut T> {
        if let Some(Some(res)) = self.resources.get_mut(handle as usize) {
            res.downcast_mut::<T>()
        } else {
            None
        }
    }

    pub fn registrar_modulo(&mut self, modulo: Module) {
        self.modules.push(modulo);
    }

    pub fn registrar_metodo_recurso(&mut self, nombre: &str, idx: usize) {
        self.resource_methods
            .insert(nombre.to_string(), Value::nativa(idx as u32));
    }

    // ===== Optimized Object Management Methods =====
    
    /// Allocate an object in the object table and return a NaN-boxed Value
    #[inline]
    pub fn alloc_object(&mut self, obj: Obj) -> Value {
        let handle = self.object_table.alloc(obj);
        Value::from_handle(handle)
    }

    /// Get a reference to an object by its Value handle
    #[inline]
    pub fn get_object(&self, value: Value) -> Option<&Obj> {
        if !value.es_objeto() {
            return None;
        }
        // Check if this is a pending object (bit 32 set)
        if (value.0 & 0x0000_0001_0000_0000) != 0 {
            let index = (value.0 & 0xFFFFFFFF) as usize;
            crate::vm::value::PENDING_OBJECTS.with(|pending_ref| {
                let pending = pending_ref.borrow();
                // SAFETY: Return reference that outlives the borrow
                // Safe because pending objects are never removed
                unsafe {
                    let ptr = pending.get(index)?.as_ref() as *const crate::vm::value::Obj;
                    Some(&*ptr)
                }
            })
        } else {
            self.object_table.get(value.a_handle())
        }
    }

    /// Get a mutable reference to an object by its Value handle
    #[inline]
    pub fn get_object_mut(&mut self, value: Value) -> Option<&mut Obj> {
        if !value.es_objeto() {
            return None;
        }
        // Pending objects are immutable (stored in global Vec)
        if (value.0 & 0x0000_0001_0000_0000) != 0 {
            return None;
        }
        self.object_table.get_mut(value.a_handle())
    }

    /// Convenience method to allocate a string (uses object pooling for small strings)
    #[inline]
    pub fn alloc_texto(&mut self, s: String) -> Value {
        self.alloc_object(Obj::Texto(s))
    }

    /// Convenience method to allocate a list
    #[inline]
    pub fn alloc_lista(&mut self, items: Vec<Value>) -> Value {
        self.alloc_object(Obj::Lista(items))
    }

    /// Convenience method to allocate a dictionary
    #[inline]
    pub fn alloc_diccionario(&mut self, map: HashMap<String, Value>) -> Value {
        self.alloc_object(Obj::Diccionario(map))
    }

    pub fn run_from(&mut self, pc: usize) -> Result<Value, String> {
        self.pc = pc;
        self.run()
    }

    pub fn interpretar(&mut self, chunk: Arc<Chunk>) -> Result<Value, String> {
        // Set thread-local object table early for constant creation
        Value::set_current_table(&mut self.object_table);
        
        self.chunk = Some(chunk);
        self.pc = 0;
        self.base_pointer = 0;
        self.frame_count = 0;
        self.try_stack.clear();
        self.run()
    }

    pub fn run(&mut self) -> Result<Value, String> {
        // Set thread-local object table for Value helper methods
        Value::set_current_table(&mut self.object_table);
        
        let chunk = self.chunk.as_ref().unwrap().clone();
        // Optimizacion: Cachear puntero a codigo si es posible, o usar slice
        // Rust safety impide raw pointers faciles aqui sin unsafe.
        // Usaremos indexing normal por ahora.

        loop {
            if self.pc >= chunk.code.len() {
                break;
            }

            let instruction = chunk.code[self.pc];
            self.pc += 1;

            let op_byte = (instruction >> 24) as u8;
            let op = OpCode::from(op_byte);
            let a = ((instruction >> 16) & 0xFF) as usize;
            let b = ((instruction >> 8) & 0xFF) as usize;
            let c = (instruction & 0xFF) as usize;
            let bx = (instruction & 0xFFFF) as usize;

            // Mapeo de registros con base_pointer
            let ra = self.base_pointer + a;
            let rb = self.base_pointer + b;
            let rc = self.base_pointer + c;

            match op {
                OpCode::CargarConstante => {
                    self.registers[ra] = chunk.constants[bx].clone();
                }
                OpCode::Mover => {
                    self.registers[ra] = self.registers[rb].clone();
                }
                OpCode::Sumar => {
                    let v1 = &self.registers[rb];
                    let v2 = &self.registers[rc];
                    if v1.es_numero() && v2.es_numero() {
                        self.registers[ra] = Value::numero(v1.a_numero() + v2.a_numero());
                    } else if v1.es_texto() && v2.es_texto() {
                        let s = format!("{}{}", v1.a_texto(), v2.a_texto());
                        self.registers[ra] = Value::texto(s);
                    } else {
                        return self.lanzar_error("Tipos incompatibles para suma");
                    }
                }
                OpCode::Restar => {
                    let v1 = self.registers[rb].a_numero();
                    let v2 = self.registers[rc].a_numero();
                    self.registers[ra] = Value::numero(v1 - v2);
                }
                OpCode::Multiplicar => {
                    let v1 = self.registers[rb].a_numero();
                    let v2 = self.registers[rc].a_numero();
                    self.registers[ra] = Value::numero(v1 * v2);
                }
                OpCode::Dividir => {
                    let v1 = self.registers[rb].a_numero();
                    let v2 = self.registers[rc].a_numero();
                    if v2 == 0.0 {
                        return self.lanzar_error("División por cero");
                    }
                    self.registers[ra] = Value::numero(v1 / v2);
                }
                OpCode::Modulo => {
                    let v1 = self.registers[rb].a_numero();
                    let v2 = self.registers[rc].a_numero();
                    self.registers[ra] = Value::numero(v1 % v2);
                }
                OpCode::Potencia => {
                    let v1 = self.registers[rb].a_numero();
                    let v2 = self.registers[rc].a_numero();
                    self.registers[ra] = Value::numero(v1.powf(v2));
                }
                OpCode::Imprimir => {
                    println!("{:?}", self.registers[ra]);
                }
                OpCode::Retornar => {
                    let result = self.registers[ra].clone();
                    if self.frame_count == 0 {
                        return Ok(result);
                    }
                    self.frame_count -= 1;
                    let frame = self.frames[self.frame_count];
                    self.pc = frame.pc;
                    self.base_pointer = frame.base_pointer;
                    if frame.return_reg != usize::MAX {
                        self.registers[frame.return_reg] = result;
                    }
                }
                OpCode::Saltar => {
                    self.pc += bx;
                }
                OpCode::SaltarSiFalso => {
                    let val = &self.registers[ra];
                    if val.es_nulo() || (val.es_logico() && !val.a_logico()) {
                        self.pc += bx;
                    }
                }
                OpCode::SaltarAtras => {
                    self.pc -= bx;
                }
                OpCode::Igual => {
                    let v1 = &self.registers[rb];
                    let v2 = &self.registers[rc];
                    // Comparación simple por ahora (bits)
                    // Para objetos complejos necesitaríamos DeepEqual
                    self.registers[ra] = Value::logico(v1.0 == v2.0);
                }
                OpCode::Menor => {
                    let v1 = self.registers[rb].a_numero();
                    let v2 = self.registers[rc].a_numero();
                    self.registers[ra] = Value::logico(v1 < v2);
                }
                OpCode::MenorIgual => {
                    let v1 = self.registers[rb].a_numero();
                    let v2 = self.registers[rc].a_numero();
                    self.registers[ra] = Value::logico(v1 <= v2);
                }
                OpCode::Not => {
                    let val = &self.registers[rb];
                    let b = val.es_nulo() || (val.es_logico() && !val.a_logico());
                    self.registers[ra] = Value::logico(b);
                }
                OpCode::Negativo => {
                    let val = self.registers[rb].a_numero();
                    self.registers[ra] = Value::numero(-val);
                }
                OpCode::BitNot => {
                    let val = self.registers[rb].a_numero();
                    // Convert to i64, negate, convert back
                    let i = val as i64;
                    self.registers[ra] = Value::numero((!i) as f64);
                }
                OpCode::ObtenerGlobal => {
                    let nombre_val = &chunk.constants[bx];
                    let nombre = nombre_val.a_texto();
                    if let Some(val) = self.globals.get(nombre) {
                        self.registers[ra] = val.clone();
                    } else {
                        return self
                            .lanzar_error(&format!("Variable global no definida: {}", nombre));
                    }
                }
                OpCode::DefinirGlobal => {
                    let nombre_val = &chunk.constants[bx];
                    let nombre = nombre_val.a_texto().clone();
                    let val = self.registers[ra].clone();
                    self.globals.insert(nombre, val);
                }
                OpCode::Llamar => {
                    let mut callee = self.registers[rb].clone();
                    let mut arg_count = c;
                    let mut new_base = rb + 1;

                    // 1. Resolver MetodoAtado (optimized with object table)
                    if callee.es_objeto() {
                        if let Some(Obj::MetodoAtado(bound)) = self.get_object(callee) {
                            let method = bound.metodo.clone();
                            let receiver = bound.receptor.clone();

                            // Inyectar 'self' en el registro rb (donde estaba el callee)
                            // Esto desplaza efectivamente los argumentos:
                            // Antes: [Callee(MetodoAtado), Arg1, Arg2]
                            // Ahora: [Self, Arg1, Arg2]
                            // Y el nuevo callee es 'method'.
                            // Los argumentos ahora empiezan en rb.
                            self.registers[rb] = receiver;
                            new_base = rb;
                            arg_count += 1;

                            callee = method;
                        }
                    }

                    // 2. Despachar llamada (optimized dispatch)
                    if callee.es_objeto() {
                        match self.get_object(callee) {
                            Some(Obj::MetodoNativo(func)) => {
                                let args_start = new_base;
                                let args_end = args_start + arg_count;
                                let args: Vec<Value> = self.registers[args_start..args_end].to_vec();
                                
                                match func(args) {
                                    Ok(res) => self.registers[ra] = res,
                                    Err(msg) => return self.lanzar_error(&msg),
                                }
                            }
                            Some(Obj::Clase(clase)) => {
                                // Instanciación: Persona()
                                // Clone clase first to avoid borrow conflict
                                let clase_cloned = clase.clone();
                                let instancia = crate::vm::value::Instancia {
                                    clase: clase_cloned.clone(),
                                    campos: HashMap::new(),
                                };
                                let inst_val = Value::objeto(Obj::Instancia(Rc::new(RefCell::new(instancia))));
                                self.registers[ra] = inst_val.clone();

                                // Buscar init en la cadena de prototipos
                                let mut current_class = Some(clase_cloned);
                                let mut init_method = None;

                                while let Some(c) = current_class {
                                    if let Some(m) = c.borrow().metodos.get("init") {
                                        init_method = Some(m.clone());
                                        break;
                                    }
                                    current_class = c.borrow().padre.clone();
                                }

                                if let Some(init_addr) = init_method {
                                    if self.frame_count == MAX_FRAMES {
                                        return self.lanzar_error("Stack overflow");
                                    }

                                    // Preparar llamada a init
                                    // self va en rb (que ahora tiene la instancia)
                                    self.registers[rb] = inst_val;

                                    self.frames[self.frame_count] = CallFrame {
                                        pc: self.pc,
                                        base_pointer: self.base_pointer,
                                        return_reg: usize::MAX, // Ignorar retorno de init
                                    };
                                    self.frame_count += 1;

                                    self.base_pointer = rb;
                                    self.pc = init_addr.a_numero() as usize;
                                }
                            }
                            _ => return self.lanzar_error("Objeto no es invocable (no es Clase ni MetodoNativo)"),
                        }
                    } else if callee.es_nativa() {
                        let idx = callee.a_nativa_idx() as usize;
                        let func = self.natives[idx];

                        let args_start = new_base;
                        let args_end = args_start + arg_count;
                        let args: Vec<Value> = self.registers[args_start..args_end].to_vec();

                        match func(self, &args) {
                            Ok(res) => self.registers[ra] = res,
                            Err(msg) => return self.lanzar_error(&msg),
                        }
                    } else if callee.es_numero() {
                        // Función definida en bytecode (dirección)
                        let func_addr = callee.a_numero() as usize;

                        // --- JIT Logic ---
                        let mut executed_jit = false;

                        // 1. Check Cache
                        if let Some(&jit_addr) = self.jit_cache.get(&func_addr) {
                            // Call JIT
                            let jit_fn: unsafe extern "C" fn(usize, Value, usize) -> Value =
                                unsafe { std::mem::transmute(jit_addr) };
                            
                            let arg0 = if arg_count > 0 {
                                self.registers[new_base].clone()
                            } else {
                                Value::nulo()
                            };

                            let consts_ptr = chunk.constants.as_ptr() as usize;

                            let res = unsafe { jit_fn(func_addr, arg0, consts_ptr) };
                            self.registers[ra] = res;
                            executed_jit = true;
                        } else {
                            // 2. Hotspot Detection
                            let counter = self.hotspot_counters.entry(func_addr).or_insert(0);
                            *counter += 1;

                            if *counter > 100 {
                                match self.jit.compile(&chunk, func_addr, true) {
                                    Ok(addr) => {
                                        self.jit_cache.insert(func_addr, addr);
                                        let jit_fn: unsafe extern "C" fn(
                                            usize,
                                            Value,
                                            usize,
                                        )
                                            -> Value = unsafe { std::mem::transmute(addr) };
                                        let arg0 = if arg_count > 0 {
                                            self.registers[new_base].clone()
                                        } else {
                                            Value::nulo()
                                        };
                                        let consts_ptr = chunk.constants.as_ptr() as usize;
                                        let res = unsafe { jit_fn(func_addr, arg0, consts_ptr) };
                                        self.registers[ra] = res;
                                        executed_jit = true;
                                    }
                                    Err(_) => {
                                        *counter = 0;
                                    }
                                }
                            }
                        }

                        if !executed_jit {
                            if self.frame_count >= MAX_FRAMES {
                                return self.lanzar_error("Stack overflow");
                            }

                            self.frames[self.frame_count] = CallFrame {
                                pc: self.pc,
                                base_pointer: self.base_pointer,
                                return_reg: ra,
                            };
                            self.frame_count += 1;

                            self.base_pointer = new_base;
                            self.pc = func_addr;
                        }
                    } else {
                        return self.lanzar_error("Intentando llamar a algo que no es función");
                    }
                }

                // --- Nuevos OpCodes ---
                OpCode::CrearLista => {
                    // R[A] = [R[B]...R[B+C]]
                    let start = self.base_pointer + b;
                    let count = c;
                    let mut lista = Vec::with_capacity(count);
                    for i in 0..count {
                        lista.push(self.registers[start + i].clone());
                    }
                    self.registers[ra] = Value::objeto(Obj::Lista(lista));
                }
                OpCode::CrearDiccionario => {
                    // R[A] = {R[B]: R[B+1], ...}
                    let start = self.base_pointer + b;
                    let count = c; // Pares
                    let mut dict = HashMap::with_capacity(count);
                    for i in 0..count {
                        let key_val = &self.registers[start + i * 2];
                        let val = &self.registers[start + i * 2 + 1];

                        // Claves deben ser strings por ahora
                        let key = if key_val.es_texto() {
                            key_val.a_texto().clone()
                        } else {
                            format!("{:?}", key_val) // Fallback stringify
                        };
                        dict.insert(key, val.clone());
                    }
                    self.registers[ra] = Value::objeto(Obj::Diccionario(dict));
                }
                OpCode::AccederIndice => {
                    // R[A] = R[B][R[C]]
                    let obj_val = &self.registers[rb];
                    let idx_val = &self.registers[rc];

                    if !obj_val.es_objeto() {
                        return self.lanzar_error("No es indexable");
                    }

                    let res = match self.get_object(*obj_val) {
                        Some(Obj::Lista(l)) => {
                            if !idx_val.es_entero() {
                                return self.lanzar_error("Índice de lista debe ser entero");
                            }
                            let idx = idx_val.a_entero();
                            if idx < 0 || idx as usize >= l.len() {
                                return self.lanzar_error("Índice fuera de rango");
                            }
                            l[idx as usize].clone()
                        }
                        Some(Obj::Diccionario(d)) => {
                            let key = if idx_val.es_texto() {
                                idx_val.a_texto().clone()
                            } else {
                                format!("{:?}", idx_val)
                            };
                            d.get(&key).cloned().unwrap_or(Value::nulo())
                        }
                        _ => return self.lanzar_error("Objeto no indexable"),
                    };
                    self.registers[ra] = res;
                }
                OpCode::AsignarIndice => {
                    // R[A][R[B]] = R[C]
                    // Ojo: A es el objeto destino, B indice, C valor.
                    // En bytecode.rs: emit(OpCode::AsignarIndice, reg_obj, reg_idx, reg_val);
                    // Aqui: A=obj, B=idx, C=val.
                    // Pero registers[ra] es mutable? Value es Copy (u64), pero apunta a Heap.
                    // Necesitamos mutabilidad interior. Value::a_objeto_mut es unsafe pero funciona si tenemos ownership único o RefCell.
                    // Nuestro modelo actual de Value es compartido (Rc implícito por ser puntero raw).
                    // Rust safety: Modificar un objeto compartido es peligroso sin RefCell.
                    // Para MVP, asumimos single-thread y "confiamos".
                    // En realidad, Obj debería usar RefCell si queremos seguridad.
                    // Vamos a usar a_objeto_mut().

                    let val = self.registers[rc].clone();
                    let idx_val = self.registers[rb].clone();
                    let obj_val = &self.registers[ra];

                    if !obj_val.es_objeto() {
                        return self.lanzar_error("No es indexable para asignación");
                    }

                    match obj_val.a_objeto_mut() {
                        Obj::Lista(l) => {
                            if !idx_val.es_entero() {
                                return self.lanzar_error("Índice de lista debe ser entero");
                            }
                            let idx = idx_val.a_entero();
                            if idx < 0 || idx as usize >= l.len() {
                                return self.lanzar_error("Índice fuera de rango");
                            }
                            l[idx as usize] = val;
                        }
                        Obj::Diccionario(d) => {
                            let key = if idx_val.es_texto() {
                                idx_val.a_texto().clone()
                            } else {
                                format!("{:?}", idx_val)
                            };
                            d.insert(key, val);
                        }
                        _ => return self.lanzar_error("Objeto no soporta asignación por índice"),
                    }
                }
                OpCode::AccederPropiedad => {
                    // R[A] = R[B].Const[C]
                    // Clone obj_val to avoid borrowing self.registers
                    let obj_val = self.registers[rb].clone();
                    let prop = chunk.constants[c].a_texto().clone();

                    if let Some(Obj::Instancia(instancia)) = self.get_object(obj_val) {
                        // Check field first
                        let field_val = instancia.borrow().campos.get(&prop).cloned();

                        if let Some(val) = field_val {
                            self.registers[ra] = val;
                        } else {
                            // Check method
                            let clase = instancia.borrow().clase.clone();
                            let method_val = clase.borrow().metodos.get(&prop).cloned();

                            if let Some(metodo) = method_val {
                                // Create MetodoAtado
                                let bound = crate::vm::value::MetodoAtado {
                                    receptor: obj_val.clone(),
                                    metodo: metodo,
                                };
                                self.registers[ra] =
                                    Value::objeto(Obj::MetodoAtado(Box::new(bound)));
                            } else {
                                return self
                                    .lanzar_error(&format!("Propiedad '{}' no encontrada", prop));
                            }
                        }
                    } else {
                        // Intentar resolver método nativo para primitivos
                        if let Some(metodo) = crate::stdlib::resolver_metodo_nativo(&obj_val, &prop) {
                             // Create MetodoAtado for native method
                             let bound = crate::vm::value::MetodoAtado {
                                receptor: obj_val.clone(),
                                metodo: metodo,
                            };
                            self.registers[ra] = Value::objeto(Obj::MetodoAtado(Box::new(bound)));
                        } else {
                            println!(
                                "Error: AccederPropiedad en objeto no instancia ni primitivo soportado: {:?}",
                                obj_val
                            );
                            return self.lanzar_error(&format!("Propiedad o método '{}' no encontrado", prop));
                        }
                    }
                }
                OpCode::AsignarPropiedad => {
                    // R[A].Const[B] = R[C]
                    let obj_val = &self.registers[ra];
                    let prop = chunk.constants[b].a_texto().clone();
                    let val = self.registers[rc].clone();

                    if let Some(Obj::Instancia(instancia)) = self.get_object_mut(*obj_val) {
                        instancia.borrow_mut().campos.insert(prop, val);
                    } else {
                        return self.lanzar_error("Solo instancias tienen propiedades");
                    }
                }
                OpCode::CrearClase => {
                    // R[A] = Class(Name=Const[B], Parent=R[C])
                    let nombre = chunk.constants[b].a_texto().clone();
                    let padre_val = &self.registers[rc];
                    let Value(p) = padre_val;
                    let padre = if *p == Value::nulo().0 {
                        None
                    } else if padre_val.es_objeto() {
                        match self.get_object(*padre_val) {
                            Some(Obj::Clase(c)) => Some(c.clone()),
                            _ => return self.lanzar_error("Padre debe ser una clase"),
                        }
                    } else {
                        None
                    };

                    let clase = Clase {
                        nombre,
                        metodos: HashMap::new(),
                        padre,
                    };
                    self.registers[ra] = Value::objeto(Obj::Clase(Rc::new(RefCell::new(clase))));
                }
                OpCode::Metodo => {
                    // Class[R[A]].Method(Name=Const[B], Body=R[C])
                    let clase_val = &self.registers[ra];
                    let nombre = chunk.constants[b].a_texto().clone();
                    let body = self.registers[rc].clone(); // Dirección de función

                    if let Some(Obj::Clase(c)) = self.get_object(*clase_val) {
                        c.borrow_mut().metodos.insert(nombre, body);
                    } else {
                        return self.lanzar_error("No es una clase");
                    }
                }
                OpCode::PushTry => {
                    // PushTryHandler(CatchBlock=Bx)
                    // Bx es offset relativo.
                    let catch_pc = self.pc + bx;
                    self.try_stack.push(catch_pc);
                }
                OpCode::PopTry => {
                    self.try_stack.pop();
                }
                OpCode::Lanzar => {
                    let err = self.registers[ra].clone();
                    // Buscar handler
                    if let Some(catch_pc) = self.try_stack.pop() {
                        self.pc = catch_pc;
                        // Guardar el error para que el catch lo recupere
                        self.last_error = Some(err);
                    } else {
                        return Err(format!("Excepción no capturada: {:?}", err));
                    }
                }
                OpCode::ObtenerError => {
                    if let Some(err) = self.last_error.take() {
                        self.registers[ra] = err;
                    } else {
                        self.registers[ra] = Value::nulo();
                    }
                }
                OpCode::Importar => {
                    // R[A] = Import(Const[Bx])
                    // Placeholder
                    self.registers[ra] = Value::nulo();
                }
                OpCode::AsyncCall => {
                    // R[A] = AsyncCall(R[B], Args...)
                    // MVP: Ejecutar síncronamente y envolver en Promesa Resuelta.
                    // En futuro: Spawn thread o corrutina.

                    // 1. Ejecutar llamada normal (simulada)
                    // Esto es complejo porque Llamar modifica el stack.
                    // Para MVP, simplemente creamos una Promesa dummy resuelta con Nulo.
                    // TODO: Implementar ejecución real.
                    let promesa = crate::vm::value::Promesa {
                        estado: crate::vm::value::EstadoPromesa::Resuelta(Value::nulo()),
                    };
                    self.registers[ra] =
                        Value::objeto(Obj::Promesa(Rc::new(RefCell::new(promesa))));
                }
                OpCode::Await => {
                    // R[A] = Await(R[B])
                    let prom_val = &self.registers[rb];
                    if let Some(Obj::Promesa(p)) = self.get_object(*prom_val) {
                        let estado = p.borrow().estado.clone();
                        match estado {
                            crate::vm::value::EstadoPromesa::Resuelta(val) => {
                                self.registers[ra] = val;
                            }
                            crate::vm::value::EstadoPromesa::Rechazada(err) => {
                                return Err(format!("Promesa rechazada: {:?}", err));
                            }
                            crate::vm::value::EstadoPromesa::Pendiente => {
                                return Err("Await en promesa pendiente no soportado en MVP (falta Event Loop)".to_string());
                            }
                        }
                    } else {
                        return self.lanzar_error("Await espera una Promesa");
                    }
                }
                _ => return self.lanzar_error(&format!("OpCode no implementado: {:?}", op)),
            }
        }
        Ok(Value::nulo())
    }

    fn lanzar_error(&mut self, msg: &str) -> Result<Value, String> {
        // Si hay try_stack, saltar al handler
        if let Some(catch_pc) = self.try_stack.pop() {
            self.pc = catch_pc;
            // Continuar ejecución
            // Hack: Retornar Ok(dummy) para que el loop continue?
            // Pero run() devuelve Result.
            // Necesitamos cambiar la estructura del loop para manejar errores internos como excepciones de usuario.
            // Por ahora, panic/error fatal.
            Err(msg.to_string())
        } else {
            Err(msg.to_string())
        }
    }
}
