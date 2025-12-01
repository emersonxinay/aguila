use crate::vm::value::{NativeFn, Obj, Value};

pub fn resolver(nombre: &str) -> Option<Value> {
    match nombre {
        "agregar" => Some(Value::objeto(Obj::MetodoNativo(agregar))),
        "eliminar" => Some(Value::objeto(Obj::MetodoNativo(eliminar))),
        "longitud" => Some(Value::objeto(Obj::MetodoNativo(longitud))),
        "unir" => Some(Value::objeto(Obj::MetodoNativo(unir))),
        "limpiar" => Some(Value::objeto(Obj::MetodoNativo(limpiar))),
        "invertir" => Some(Value::objeto(Obj::MetodoNativo(invertir))),
        _ => None,
    }
}

fn agregar(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("Se requiere el elemento a agregar".to_string());
    }
    // args[0] es la lista (self)
    // args[1] es el elemento
    let val = args[1].clone();
    
    match args[0].a_objeto_mut() {
        Obj::Lista(l) => {
            l.push(val);
            Ok(Value::nulo())
        }
        _ => Err("No es una lista".to_string()),
    }
}

fn eliminar(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("Se requiere el índice a eliminar".to_string());
    }
    let idx_val = &args[1];
    if !idx_val.es_entero() {
        return Err("El índice debe ser entero".to_string());
    }
    let idx = idx_val.a_entero();
    
    match args[0].a_objeto_mut() {
        Obj::Lista(l) => {
            if idx < 0 || idx as usize >= l.len() {
                return Err("Índice fuera de rango".to_string());
            }
            let removed = l.remove(idx as usize);
            Ok(removed)
        }
        _ => Err("No es una lista".to_string()),
    }
}

fn longitud(args: Vec<Value>) -> Result<Value, String> {
    match args[0].a_objeto() {
        Obj::Lista(l) => Ok(Value::entero(l.len() as i32)),
        _ => Err("No es una lista".to_string()),
    }
}

fn limpiar(args: Vec<Value>) -> Result<Value, String> {
    match args[0].a_objeto_mut() {
        Obj::Lista(l) => {
            l.clear();
            Ok(Value::nulo())
        }
        _ => Err("No es una lista".to_string()),
    }
}

fn invertir(args: Vec<Value>) -> Result<Value, String> {
    match args[0].a_objeto_mut() {
        Obj::Lista(l) => {
            l.reverse();
            Ok(Value::nulo())
        }
        _ => Err("No es una lista".to_string()),
    }
}

fn unir(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("Se requiere el separador".to_string());
    }
    let sep = args[1].a_texto();
    
    match args[0].a_objeto() {
        Obj::Lista(l) => {
            let mut res = String::new();
            for (i, v) in l.iter().enumerate() {
                if i > 0 {
                    res.push_str(sep);
                }
                if v.es_texto() {
                    res.push_str(v.a_texto());
                } else {
                    res.push_str(&format!("{:?}", v));
                }
            }
            Ok(Value::texto(res))
        }
        _ => Err("No es una lista".to_string()),
    }
}
