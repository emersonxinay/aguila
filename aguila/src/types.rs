use postgres;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use futures::future::{LocalBoxFuture, Shared};

#[derive(Clone)]
pub struct NativeFn(pub Rc<dyn Fn(&[Value]) -> Result<Value, String>>);

impl fmt::Debug for NativeFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<función nativa>")
    }
}

impl PartialEq for NativeFn {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Eq for NativeFn {}

impl Hash for NativeFn {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Las funciones nativas no son realmente hasheables por identidad fácilmente,
        // usamos un valor constante o dirección si fuera posible.
        // Por simplicidad, asumimos que no se usarán como claves frecuentemente o colisionarán.
        (self.0.as_ref() as *const _ as *const () as usize).hash(state);
    }
}

#[derive(Clone)]
pub struct DbClient(pub Rc<RefCell<postgres::Client>>);

impl fmt::Debug for DbClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<conexión db>")
    }
}

impl PartialEq for DbClient {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Eq for DbClient {}

impl Hash for DbClient {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        // No hasheable realmente
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Numero(f64),
    Texto(String),
    Logico(bool),
    Lista(Rc<RefCell<Vec<Value>>>),
    Diccionario(Rc<RefCell<HashMap<String, Value>>>),
    Conjunto(Rc<RefCell<HashSet<Value>>>),
    Nulo,
    Funcion(
        Vec<String>,
        Vec<crate::ast::Sentencia>,
        Rc<RefCell<HashMap<String, Value>>>,
        bool,
    ),
    FuncionNativa(NativeFn),
    #[allow(dead_code)]
    Clase(String, Rc<RefCell<HashMap<String, Value>>>), // Nombre, Scope de clase (métodos)
    Instancia {
        clase: String,
        atributos: Rc<RefCell<HashMap<String, Value>>>,
    },
    BaseDeDatos(DbClient),
    Promesa(Shared<LocalBoxFuture<'static, Result<Value, String>>>),
}

impl Value {
    pub fn a_texto(&self) -> String {
        match self {
            Value::Numero(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::Texto(s) => s.clone(),
            Value::Logico(b) => if *b { "verdadero" } else { "falso" }.to_string(),
            Value::Nulo => "nulo".to_string(),
            Value::Lista(items) => {
                let items = items.borrow();
                let strs: Vec<String> = items.iter().map(|v| v.a_texto()).collect();
                format!("[{}]", strs.join(", "))
            }
            Value::Diccionario(map) => {
                let map = map.borrow();
                let items: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v.a_texto()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            Value::Conjunto(set) => {
                let set = set.borrow();
                let strs: Vec<String> = set.iter().map(|v| v.a_texto()).collect();
                format!("#{{{}}}", strs.join(", "))
            }
            Value::Funcion(..) => "<función>".to_string(),
            Value::FuncionNativa(_) => "<función nativa>".to_string(),
            Value::Clase(nombre, _) => format!("<clase {}>", nombre),
            Value::Instancia { clase, .. } => format!("<instancia de {}>", clase),
            Value::BaseDeDatos(_) => "<conexión db>".to_string(),
            Value::Promesa(_) => "<promesa>".to_string(),
        }
    }

    pub fn a_booleano(&self) -> bool {
        match self {
            Value::Logico(b) => *b,
            Value::Nulo => false,
            Value::Numero(n) => *n != 0.0,
            Value::Texto(s) => !s.is_empty(),
            Value::Lista(items) => !items.borrow().is_empty(),
            Value::Diccionario(map) => !map.borrow().is_empty(),
            Value::Conjunto(set) => !set.borrow().is_empty(),
            _ => true,
        }
    }

    pub fn a_logico(&self) -> bool {
        self.a_booleano()
    }

    pub fn a_numero(&self) -> f64 {
        match self {
            Value::Numero(n) => *n,
            Value::Texto(s) => s.parse().unwrap_or(0.0),
            Value::Logico(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Numero(a), Value::Numero(b)) => a == b, // Precaución con NaN
            (Value::Texto(a), Value::Texto(b)) => a == b,
            (Value::Logico(a), Value::Logico(b)) => a == b,
            (Value::Nulo, Value::Nulo) => true,
            (Value::Lista(a), Value::Lista(b)) => a == b, // Punteros iguales? No, contenido. Pero por ahora punteros o recursivo?
            // Para simplificar y evitar ciclos infinitos en comparaciones profundas,
            // Rust por defecto en Rc compara punteros. Si queremos valor, necesitamos deep eq.
            // Por ahora, asumamos igualdad de valor para primitivos y referencia para complejos
            // O mejor, igualdad estructural básica.
            // Dado que Value contiene Rc<RefCell<...>>, PartialEq derivado no funciona directo si no lo implementamos.
            // Aquí estamos implementando manualmente.
            // Para conjuntos necesitamos Eq completo.
            // Implementación simplificada:
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Numero(n) => {
                // Hack para hashear floats: usar representación en bits
                // Nota: 0.0 y -0.0 deberían ser iguales, y NaN != NaN.
                // Para uso simple en sets, esto suele bastar si evitamos NaNs.
                let bits = n.to_bits();
                bits.hash(state);
            }
            Value::Texto(s) => s.hash(state),
            Value::Logico(b) => b.hash(state),
            Value::Nulo => 0.hash(state),
            Value::Lista(l) => {
                // Hashear la dirección del Rc para identidad referencial
                (l.as_ptr() as usize).hash(state);
            }
            Value::Diccionario(d) => {
                (d.as_ptr() as usize).hash(state);
            }
            Value::Conjunto(s) => {
                (s.as_ptr() as usize).hash(state);
            }
            Value::Funcion(_, _, _, _) => {
                // Difícil hashear funciones, usamos discriminante
                1.hash(state);
            }
            Value::FuncionNativa(f) => f.hash(state),
            Value::Clase(nombre, _) => nombre.hash(state),
            Value::Instancia { clase, .. } => clase.hash(state), // Podríamos hashear la identidad del objeto
            Value::BaseDeDatos(db) => db.hash(state),
            Value::Promesa(_) => 2.hash(state), // Discriminante para promesas
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Numero(a), Value::Numero(b)) => a.partial_cmp(b),
            (Value::Texto(a), Value::Texto(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}
