use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

// Global storage for pending objects (created before VM initialization)
// Using thread_local since Águila is single-threaded
thread_local! {
    pub static PENDING_OBJECTS: RefCell<Vec<Box<Obj>>> = RefCell::new(Vec::new());
}

// Thread-local accessor for object table (used by Value helper methods)
// This allows Value methods like .a_texto() to work without requiring a VM reference
thread_local! {
    static CURRENT_OBJECT_TABLE: RefCell<Option<*const crate::vm::object_table::ObjectTable>> = RefCell::new(None);
}

// NaN-Boxing en 64 bits (IEEE 754)
// Optimized with object table:
// - Obj (Handle): QNaN | 0x8000000000000000 | Object Handle (32 bits)
// Object handles use generational indices for safety and performance

pub const QNAN: u64 = 0x7ffc000000000000;
pub const SIGN_BIT: u64 = 0x8000000000000000;

pub const TAG_NULO: u64 = 1;
pub const TAG_FALSE: u64 = 2;
pub const TAG_TRUE: u64 = 3;
pub const TAG_NATIVE_FUNC: u64 = 4;
#[allow(dead_code)]
pub const TAG_MODULE: u64 = 5;
pub const TAG_RESOURCE: u64 = 6;

pub const TAG_INT: u64 = 0x0001000000000000; // Bit 48

#[derive(Debug, Clone, PartialEq)]
pub enum Obj {
    Texto(String),
    Lista(Vec<Value>),
    Diccionario(HashMap<String, Value>), // Claves string por simplicidad MVP
    Clase(Rc<RefCell<Clase>>),
    Instancia(Rc<RefCell<Instancia>>),
    MetodoAtado(Box<MetodoAtado>),
    Promesa(Rc<RefCell<Promesa>>),
    MetodoNativo(NativeFn),
}

pub type NativeFn = fn(Vec<Value>) -> Result<Value, String>;

#[derive(Debug, Clone, PartialEq)]
pub enum EstadoPromesa {
    Pendiente,
    Resuelta(Value),
    Rechazada(Value),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Promesa {
    pub estado: EstadoPromesa,
    // En un sistema real, aquí irían callbacks o wakers
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetodoAtado {
    pub receptor: Value,
    pub metodo: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clase {
    pub nombre: String,
    pub metodos: HashMap<String, Value>, // Value debe ser funcion (addr)
    pub padre: Option<Rc<RefCell<Clase>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instancia {
    pub clase: Rc<RefCell<Clase>>,
    pub campos: HashMap<String, Value>,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Value(pub u64);

impl Value {
    #[inline(always)]
    pub fn numero(n: f64) -> Self {
        Self(n.to_bits())
    }

    #[inline(always)]
    pub fn nulo() -> Self {
        Self(QNAN | TAG_NULO)
    }

    #[inline(always)]
    pub fn logico(b: bool) -> Self {
        if b {
            Self(QNAN | TAG_TRUE)
        } else {
            Self(QNAN | TAG_FALSE)
        }
    }

    #[inline(always)]
    pub fn entero(n: i32) -> Self {
        Self(QNAN | TAG_INT | (n as u32 as u64))
    }

    #[inline(always)]
    pub fn es_entero(&self) -> bool {
        (self.0 & (QNAN | TAG_INT)) == (QNAN | TAG_INT)
    }

    #[inline(always)]
    pub fn a_entero(&self) -> i32 {
        (self.0 & 0xFFFFFFFF) as i32
    }

    #[inline(always)]
    pub fn es_numero(&self) -> bool {
        ((self.0 & QNAN) != QNAN) || self.es_entero()
    }

    #[inline(always)]
    pub fn es_nulo(&self) -> bool {
        self.0 == (QNAN | TAG_NULO)
    }

    #[inline(always)]
    pub fn es_logico(&self) -> bool {
        (self.0 | 1) == (QNAN | TAG_TRUE)
    }

    #[inline(always)]
    pub fn a_numero(&self) -> f64 {
        if self.es_entero() {
            self.a_entero() as f64
        } else {
            f64::from_bits(self.0)
        }
    }

    #[inline(always)]
    pub fn a_logico(&self) -> bool {
        self.0 == (QNAN | TAG_TRUE)
    }

    #[inline(always)]
    pub fn nativa(idx: u32) -> Self {
        Self(QNAN | TAG_NATIVE_FUNC | ((idx as u64) << 32))
    }

    #[inline(always)]
    pub fn es_nativa(&self) -> bool {
        (self.0 & (QNAN | TAG_INT | 0x7)) == (QNAN | TAG_NATIVE_FUNC)
    }

    #[inline(always)]
    pub fn a_nativa_idx(&self) -> u32 {
        ((self.0 >> 32) & 0xFFFF) as u32
    }

    #[inline(always)]
    pub fn recurso(handle: u32) -> Self {
        Self(QNAN | TAG_RESOURCE | ((handle as u64) << 32))
    }

    #[inline(always)]
    pub fn es_recurso(&self) -> bool {
        (self.0 & (QNAN | TAG_INT | 0x7)) == (QNAN | TAG_RESOURCE)
    }

    #[inline(always)]
    pub fn a_recurso_handle(&self) -> u32 {
        ((self.0 >> 32) & 0xFFFF) as u32
    }

    #[inline(always)]
    pub fn modulo(idx: u32) -> Self {
        Self(QNAN | TAG_MODULE | ((idx as u64) << 32))
    }

    // --- Soporte de Objetos ---

    /// Create a Value from an Obj using an object table handle
    /// NOTE: This now requires an ObjectTable reference - use VM::alloc_object instead
    pub fn from_handle(handle: u32) -> Self {
        Self(QNAN | SIGN_BIT | (handle as u64))
    }

    pub fn texto(s: String) -> Self {
        // Try to allocate via thread-local object table if available
        // Otherwise store in global pending storage
        CURRENT_OBJECT_TABLE.with(|table_ref| {
            let table_opt = table_ref.borrow();
            if let Some(table_ptr) = *table_opt {
                drop(table_opt);
                unsafe {
                    let table = &mut *(table_ptr as *mut crate::vm::object_table::ObjectTable);
                    let handle = table.alloc(Obj::Texto(s));
                    Self::from_handle(handle)
                }
            } else {
                // No active table - store in global pending storage
                PENDING_OBJECTS.with(|pending_ref| {
                    let mut pending = pending_ref.borrow_mut();
                    let index = pending.len() as u32;
                    pending.push(Box::new(Obj::Texto(s)));
                    // Use bit 32 as pending marker, lower 32 bits as index into pending vec
                    Self(QNAN | SIGN_BIT | 0x0000_0001_0000_0000 | (index as u64))
                })
            }
        })
    }

    /// Create a Value from an Obj using an object table handle
    /// NOTE: This requires a mutable ObjectTable - use VM::alloc_object instead
    /// This is kept for stdlib compatibility where VM is not available
    pub fn objeto(obj: Obj) -> Self {
        CURRENT_OBJECT_TABLE.with(|table_ref| {
            let table_opt = table_ref.borrow();
            if let Some(table_ptr) = *table_opt {
                drop(table_opt);
                unsafe {
                    let table = &mut *(table_ptr as *mut crate::vm::object_table::ObjectTable);
                    let handle = table.alloc(obj);
                    Self::from_handle(handle)
                }
            } else {
                // No active table - store in global pending storage
                PENDING_OBJECTS.with(|pending_ref| {
                    let mut pending = pending_ref.borrow_mut();
                    let index = pending.len() as u32;
                    pending.push(Box::new(obj));
                    // Use bit 32 as pending marker
                    Self(QNAN | SIGN_BIT | 0x0000_0001_0000_0000 | (index as u64))
                })
            }
        })
    }

    /// Check if this value is a pending object (needs migration to object table)
    #[inline]
    pub fn is_pending_object(&self) -> bool {
        self.es_objeto() && (self.0 & 0x0000_0001_0000_0000) != 0
    }

    pub fn es_objeto(&self) -> bool {
        // Check for both regular objects and pending objects
        // Regular: QNAN | SIGN_BIT | handle (no bit 47)
        // Pending: QNAN | SIGN_BIT | 0x8000_0000_0000 | pointer
        (self.0 & (QNAN | SIGN_BIT)) == (QNAN | SIGN_BIT)
    }

    /// Extract object handle from NaN-boxed value
    #[inline(always)]
    pub fn a_handle(&self) -> u32 {
        (self.0 & 0xFFFFFFFF) as u32
    }

    // DEPRECATED: Use VM::get_object(value.a_handle()) instead
    // These methods are kept for compatibility but should not be used directly
    #[deprecated(note = "Use VM::get_object instead")]
    pub fn a_objeto(&self) -> &Obj {
        if !self.es_objeto() {
            panic!("a_objeto() called on non-object value");
        }
        
        // Check pending bit (bit 32)
        if (self.0 & 0x0000_0001_0000_0000) != 0 {
            // This is a pending object - get from global storage
            let index = (self.0 & 0xFFFFFFFF) as usize;
            PENDING_OBJECTS.with(|pending_ref| {
                let pending = pending_ref.borrow();
                // SAFETY: We need to return a reference that outlives the borrow
                // This is safe because pending objects are never removed, only added
                unsafe {
                    let ptr = pending[index].as_ref() as *const Obj;
                    &*ptr
                }
            })
        } else {
            // This is an object table handle
            CURRENT_OBJECT_TABLE.with(|table_ref| {
                let table_opt = table_ref.borrow();
                if let Some(table_ptr) = *table_opt {
                    unsafe {
                        (*table_ptr).get(self.a_handle())
                            .expect("Invalid object handle in a_objeto()")
                    }
                } else {
                    panic!("a_objeto() called without active object table context")
                }
            })
        }
    }

    #[deprecated(note = "Use VM::get_object_mut instead")]
    pub fn a_objeto_mut(&self) -> &mut Obj {
        panic!("a_objeto_mut() cannot be implemented with thread-local - use VM::get_object_mut()")
    }

    /// Set the current object table for this thread (called by VM)
    pub fn set_current_table(table: &mut crate::vm::object_table::ObjectTable) {
        CURRENT_OBJECT_TABLE.with(|table_ref| {
            *table_ref.borrow_mut() = Some(table as *mut _ as *const _);
        });
    }

    /// Clear the current object table for this thread
    pub fn clear_current_table() {
        CURRENT_OBJECT_TABLE.with(|table_ref| {
            *table_ref.borrow_mut() = None;
        });
    }

    // Helpers específicos
    pub fn es_texto(&self) -> bool {
        if !self.es_objeto() {
            return false;
        }
        matches!(self.a_objeto(), Obj::Texto(_))
    }

    pub fn a_texto(&self) -> &String {
        match self.a_objeto() {
            Obj::Texto(s) => s,
            _ => panic!("No es texto"),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.es_entero() {
            write!(f, "{}", self.a_entero())
        } else if self.es_numero() {
            write!(f, "{}", self.a_numero())
        } else if self.es_nulo() {
            write!(f, "nulo")
        } else if self.es_logico() {
            write!(f, "{}", self.a_logico())
        } else if self.es_objeto() {
            match self.a_objeto() {
                Obj::Texto(s) => write!(f, "\"{}\"", s),
                Obj::Lista(l) => write!(f, "{:?}", l),
                Obj::Diccionario(d) => write!(f, "{:?}", d),
                Obj::Clase(c) => write!(f, "<Clase {}>", c.borrow().nombre),
                Obj::Instancia(i) => write!(f, "<Instancia {}>", i.borrow().clase.borrow().nombre),
                Obj::MetodoAtado(m) => write!(f, "<MetodoAtado>"),
                Obj::Promesa(p) => write!(f, "<Promesa {:?}>", p.borrow().estado),
                Obj::MetodoNativo(_) => write!(f, "<MetodoNativo>"),
            }
        } else if self.es_nativa() {
            write!(f, "<nativa>")
        } else {
            write!(f, "<desconocido: {:#x}>", self.0)
        }
    }
}
